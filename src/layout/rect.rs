use crate::style::values::computed::length::CSSPixelLength;
use crate::style::values::CSSFloat;

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    /// The exact point where the rectangle begins on the x-axis.
    pub start_x: CSSFloat,
    /// The exact point where the rectangle begins on the y-axis.
    pub start_y: CSSFloat,
    pub width: CSSPixelLength,
    pub height: CSSPixelLength,
}

impl Rect {
    pub fn expanded_by_edges(self, edge: EdgeSizes) -> Rect {
        Rect {
            start_x: (self.start_x - edge.left).px(),
            start_y: (self.start_y - edge.top).px(),
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }

    /// Expands this rect by another.  The resulting rect has the same `start_x` and `start_y` as
    /// the `self` rect, but with width and height expanded by other rect.
    pub fn expanded_by_rect(self, rect: Rect) -> Rect {
        Rect {
            start_x: self.start_x,
            start_y: self.start_y,
            width: self.width + rect.width,
            height: self.height + rect.height,
        }
    }

    pub fn scaled_by(&self, scale_factor: f32) -> Rect {
        Rect {
            start_x: self.start_x * scale_factor,
            start_y: self.start_y * scale_factor,
            width: self.width * scale_factor,
            height: self.height * scale_factor,
        }
    }
}

/// A collection of edges, e.g. borders, margins, padding.
#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeSizes {
    pub left: CSSPixelLength,
    pub right: CSSPixelLength,
    pub top: CSSPixelLength,
    pub bottom: CSSPixelLength,
}

impl EdgeSizes {
    pub fn scale_by(&mut self, scale_factor: f32) {
        self.left *= scale_factor;
        self.right *= scale_factor;
        self.top *= scale_factor;
        self.bottom *= scale_factor;
    }

    pub fn expanded_by_edges(self, edges: EdgeSizes) -> EdgeSizes {
        EdgeSizes {
            left: self.left + edges.left,
            right: self.right + edges.right,
            bottom: self.bottom + edges.bottom,
            top: self.top + edges.top,
        }
    }
}
