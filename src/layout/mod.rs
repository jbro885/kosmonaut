// Useful links:
//  * https://www.w3.org/TR/css-display-3/#css-box
//  * https://www.w3.org/TR/2018/WD-css-box-3-20181218/#intro
pub mod dimensions;
pub mod flow;
pub mod formatting_context;
pub mod layout_box;
pub mod rect;
pub mod values;

use crate::dom::tree::{NodeData, NodeRef};
use crate::layout::dimensions::PhysicalDimensions;
use crate::layout::layout_box::{BoxType, LayoutBox};
use crate::layout::rect::Rect;
use crate::style::values::computed::length::CSSPixelLength;
use crate::style::values::computed::Display;
use crate::style::values::CSSFloat;
use std::io::Write;

/// Takes a DOM node and builds the corresponding layout tree of it and its children.  Returns
/// `None` if `node` is a `Display::None`.
pub fn build_layout_tree(node: NodeRef) -> Option<LayoutBox> {
    let computed_values = &*node.computed_values();
    // TODO: We need to think about the validity of making strong-ref clones to nodes here (and elsewhere).
    // Will things get properly dropped?  Maybe LayoutBox should store a `Weak` ref?
    let mut layout_box = match computed_values.display {
        Display::Block => LayoutBox::new(
            BoxType::Block,
            node.clone(),
            computed_values.direction,
            computed_values.writing_mode,
        ),
        Display::Inline => LayoutBox::new(
            BoxType::Inline,
            node.clone(),
            computed_values.direction,
            computed_values.writing_mode,
        ),
        Display::None => {
            return None;
        }
    };

    for child in node.children() {
        let child_computed_values = &*child.computed_values();
        match child_computed_values.display {
            Display::Block => {
                if let Some(child_box) = build_layout_tree(child.clone()) {
                    // TODO: We don't handle the case where a block-flow child box is added to an inline box.
                    // This current behavior is wrong — we should be checking if `node` is an `Display::Inline` and
                    // doing something different here.  To fix, see: https://www.w3.org/TR/CSS2/visuren.html#box-gen
                    // Namely, the paragraph that begins with "When an inline box contains an in-flow block-level box"
                    // This concept _might_ be called "fragmenting".
                    layout_box.add_child(child_box)
                }
            }
            Display::Inline => {
                if let Some(child_box) = build_layout_tree(child.clone()) {
                    layout_box.add_child_inline(child_box)
                }
            }
            Display::None => {}
        }
    }
    Some(layout_box)
}

/// Given a `window` and what probably should be the root of a `layout_tree`, perform a layout
/// with the dimensions of the `window`.
pub fn global_layout(
    layout_tree: &mut LayoutBox,
    inner_window_width: f32,
    inner_window_height: f32,
    scale_factor: f32,
) {
    layout_tree.layout(
        PhysicalDimensions {
            content: Rect {
                start_x: 0.0,
                start_y: 0.0,
                width: CSSPixelLength::new(inner_window_width),
                height: CSSPixelLength::new(inner_window_height),
            },
            padding: Default::default(),
            border: Default::default(),
            margin: Default::default(),
        },
        scale_factor,
    );
}

/// https://drafts.csswg.org/css-writing-modes-4/#logical-directions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalDirection {
    BlockStart,
    BlockEnd,
    InlineStart,
    InlineEnd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxComponent {
    Border,
    Margin,
    Padding,
}

/// Trait describing behavior necessary for dumping the layout tree, used in the `dump-layout`
/// tests and debugging.  Should be implemented by "container"-style entities, such as members
/// of the layout tree, formatting individual struct members via the `DumpLayoutFormat` trait.
pub trait DumpLayout {
    fn dump_layout<W: Write>(&self, write_to: &mut W, indent_spaces: usize, verbose: bool);
}

/// Trait describing behavior necessary for formatting ones data in preparation for a layout tree
/// dump.
pub trait DumpLayoutFormat {
    fn dump_layout_format(&self) -> String;
}

impl DumpLayoutFormat for CSSFloat {
    fn dump_layout_format(&self) -> String {
        let px = format!("{:.2}", self);
        let mut px_trimmed = px.trim_end_matches('0');
        px_trimmed = px_trimmed.trim_end_matches('.');
        px_trimmed.to_owned()
    }
}

impl DumpLayoutFormat for CSSPixelLength {
    fn dump_layout_format(&self) -> String {
        self.px().dump_layout_format()
    }
}

impl DumpLayoutFormat for NodeData {
    fn dump_layout_format(&self) -> String {
        let possibly_lowercase = match self {
            NodeData::Comment(_) => "COMMENT",
            NodeData::Document(_) => "DOCUMENT",
            NodeData::Doctype(_) => "DOCTYPE",
            NodeData::DocumentFragment => "DOCUMENT_FRAGMENT",
            NodeData::Element(element_data) => &element_data.name.local,
            NodeData::Text(_) => "TEXT",
            NodeData::ProcessingInstruction(_) => "PROCESSING_INSTRUCTION",
        }
        .to_owned();
        possibly_lowercase.to_uppercase()
    }
}
