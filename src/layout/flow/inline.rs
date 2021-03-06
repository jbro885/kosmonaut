use crate::apply_page_relative_properties_base_box_passthrough_impls;
use crate::dom::tree::NodeRef;
use crate::layout::behavior::{ApplyPageRelativeProperties, BaseLayoutBoxBehavior};
use crate::layout::containing_block::ContainingBlock;
use crate::layout::dimensions::Dimensions;
use crate::layout::formatting_context::FormattingContextRef;
use crate::layout::layout_box::{BaseBox, LayoutBox};
use crate::layout::{DumpLayoutFormat, Layout, LayoutContext};
use crate::layout_box_behavior_base_box_passthrough_impls;
use crate::style::values::computed::ComputedValues;
use accountable_refcell::Ref;
use enum_dispatch::enum_dispatch;

/// Content that participates in inline layout. Specifically, inline-level boxes and text runs.
///
/// https://drafts.csswg.org/css-display/#inline-level
#[enum_dispatch]
#[derive(Clone, Debug, IntoStaticStr)]
pub enum InlineLevelContent {
    InlineLevelBox(InlineLevelBox),
    /// A representation of the contents of a text DOM node.
    ///
    /// https://drafts.csswg.org/css-display-3/#text-run
    TextRun(TextRun),
}

impl InlineLevelContent {
    pub fn is_anonymous_inline(&self) -> bool {
        match self {
            InlineLevelContent::InlineLevelBox(ilb) => ilb.is_anonymous_inline(),
            InlineLevelContent::TextRun(_) => false,
        }
    }
}

impl ApplyPageRelativeProperties for InlineLevelContent {
    fn apply_block_page_relative_properties(&mut self, containing_block: ContainingBlock) {
        match self {
            InlineLevelContent::InlineLevelBox(ilb) => {
                ilb.apply_block_page_relative_properties(containing_block)
            }
            // The underlying implementation of this method applies computed values, which can't be targeted at text runs by authors.  So do nothing here.
            InlineLevelContent::TextRun(_) => {}
        }
    }

    fn apply_inline_page_relative_properties(&mut self, containing_block: ContainingBlock) {
        match self {
            InlineLevelContent::InlineLevelBox(ilb) => {
                ilb.apply_inline_page_relative_properties(containing_block)
            }
            // The underlying implementation of this method applies computed values, which can't be targeted at text runs by authors.  So do nothing here.
            InlineLevelContent::TextRun(_) => {}
        }
    }
}

impl Layout for InlineLevelContent {
    fn layout(&mut self, context: LayoutContext) {
        match self {
            InlineLevelContent::InlineLevelBox(ilb) => ilb.layout(context),
            InlineLevelContent::TextRun(tr) => unimplemented!(
                "layout called on text run with contents '{}'",
                tr.contents.clone()
            ),
        }
    }
}

#[enum_dispatch]
#[derive(Clone, Debug, IntoStaticStr)]
pub enum InlineLevelBox {
    /// An inline-level box not associated with any element.
    ///
    /// An example of an anonymous inline box is the root inline box generated by block containers
    /// who have inline content that needs a place to go.
    ///
    /// For more information about this box type, see: https://drafts.csswg.org/css-inline-3/#model
    ///
    /// An aside, quoting https://drafts.csswg.org/css-display/#block-container:
    ///   > Note, this root inline box concept effectively replaces the "anonymous inline element"
    ///     concept introduced in CSS2§9.2.2.1.
    AnonymousInline(AnonymousInlineBox),
    /// A non-replaced inline-level box whose inner display type is flow. The contents of an inline
    /// box participate in the same inline formatting context as the inline box itself.
    ///
    /// This is also known as an "inline-block".
    ///
    /// https://drafts.csswg.org/css-display/#inline-box
    InlineBox(InlineBox),
}

impl InlineLevelBox {
    pub fn add_child(&mut self, new_child: LayoutBox) {
        match self {
            InlineLevelBox::AnonymousInline(aib) => aib.children.push(new_child),
            InlineLevelBox::InlineBox(ib) => ib.children.push(new_child),
        }
    }

    pub fn children(&self) -> &Vec<LayoutBox> {
        match self {
            InlineLevelBox::AnonymousInline(aib) => aib.children(),
            InlineLevelBox::InlineBox(ib) => ib.children(),
        }
    }

    pub fn is_anonymous_inline(&self) -> bool {
        match self {
            InlineLevelBox::AnonymousInline(_) => true,
            InlineLevelBox::InlineBox(_) => false,
        }
    }
}

impl Layout for InlineLevelBox {
    fn layout(&mut self, _context: LayoutContext) {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct AnonymousInlineBox {
    base: BaseBox,
    children: Vec<LayoutBox>,
}

impl AnonymousInlineBox {
    pub fn new(node: NodeRef, formatting_context: FormattingContextRef) -> Self {
        Self {
            base: BaseBox::new(node, formatting_context),
            children: Vec::new(),
        }
    }

    pub fn children(&self) -> &Vec<LayoutBox> {
        &self.children
    }
}

impl BaseLayoutBoxBehavior for AnonymousInlineBox {
    layout_box_behavior_base_box_passthrough_impls!();
}

impl ApplyPageRelativeProperties for AnonymousInlineBox {
    apply_page_relative_properties_base_box_passthrough_impls!();
}

impl DumpLayoutFormat for AnonymousInlineBox {
    fn dump_layout_format(&self) -> String {
        // Anonymous boxes are not generated by an element of the DOM, so simply print the name
        // of this struct for the dump-layout display.
        "AnonymousInlineBox".to_string()
    }
}

#[derive(Clone, Debug)]
pub struct InlineBox {
    base: BaseBox,
    children: Vec<LayoutBox>,
}

impl InlineBox {
    pub fn new(node: NodeRef, formatting_context: FormattingContextRef) -> Self {
        Self {
            base: BaseBox::new(node, formatting_context),
            children: Vec::new(),
        }
    }

    fn children(&self) -> &Vec<LayoutBox> {
        &self.children
    }
}

impl BaseLayoutBoxBehavior for InlineBox {
    layout_box_behavior_base_box_passthrough_impls!();
}

impl ApplyPageRelativeProperties for InlineBox {
    apply_page_relative_properties_base_box_passthrough_impls!();
}

impl DumpLayoutFormat for InlineBox {
    fn dump_layout_format(&self) -> String {
        let node_data = self.node().data().dump_layout_format();
        if node_data.is_empty() {
            "InlineBox".to_string()
        } else {
            format!("{} {}", node_data, "InlineBox")
        }
    }
}

/// A representation of the contents of a text DOM node.
///
/// https://drafts.csswg.org/css-display-3/#text-run
#[derive(Clone, Debug)]
pub struct TextRun {
    base: BaseBox,
    /// The text contents of the node.
    ///
    /// TODO: This can be an owned String for now for simplicity's sake, but it would be probably
    /// be more efficient if this were a `&'DOM_LIFETIME str`.
    contents: String,
}

impl TextRun {
    pub fn new(node: NodeRef, formatting_context: FormattingContextRef, contents: String) -> Self {
        Self {
            base: BaseBox::new(node, formatting_context),
            contents,
        }
    }

    pub fn contents(&self) -> String {
        self.contents.clone()
    }
}

impl BaseLayoutBoxBehavior for TextRun {
    layout_box_behavior_base_box_passthrough_impls!();
}

impl DumpLayoutFormat for TextRun {
    fn dump_layout_format(&self) -> String {
        let node_data = self.node().data().dump_layout_format();
        if node_data.is_empty() {
            "TextRun".to_string()
        } else {
            format!("{} {}", node_data, "TextRun")
        }
    }
}
