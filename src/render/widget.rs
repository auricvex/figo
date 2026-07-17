//! Core widget trait and sizing constraints.

/// A 2-D size in cells.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub w: usize,
    pub h: usize,
}

impl Size {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h }
    }
}

/// A rectangle in local coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    pub fn right(&self) -> usize {
        self.x + self.w
    }

    pub fn bottom(&self) -> usize {
        self.y + self.h
    }

    pub fn center(&self) -> (usize, usize) {
        (self.x + self.w / 2, self.y + self.h / 2)
    }
}

/// Constraints passed to a widget during layout.
#[derive(Clone, Copy, Debug)]
pub struct BoxConstraints {
    pub min_w: usize,
    pub max_w: usize,
    pub min_h: usize,
    pub max_h: usize,
}

impl BoxConstraints {
    pub fn new(min_w: usize, max_w: usize, min_h: usize, max_h: usize) -> Self {
        Self { min_w, max_w, min_h, max_h }
    }

    pub fn tight(w: usize, h: usize) -> Self {
        Self { min_w: w, max_w: w, min_h: h, max_h: h }
    }

    pub fn loose(max_w: usize, max_h: usize) -> Self {
        Self { min_w: 0, max_w, min_h: 0, max_h }
    }

    pub fn unconstrained() -> Self {
        Self { min_w: 0, max_w: usize::MAX, min_h: 0, max_h: usize::MAX }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size { w: size.w.clamp(self.min_w, self.max_w), h: size.h.clamp(self.min_h, self.max_h) }
    }
}

/// Anything that can be measured, laid out, and painted.
pub trait Widget {
    /// Compute the widget's desired size given the constraints.
    fn measure(&self, ctx: &MeasureContext) -> Size;

    /// Assign the widget's final rectangle and return it.
    fn layout(&mut self, ctx: &mut LayoutContext, rect: Rect) -> Rect;

    /// Paint the widget into the given surface.
    fn paint(&self, ctx: &PaintContext, surface: &mut crate::render::surface::Surface<'_>);
}

/// Context available during the measure pass.
pub struct MeasureContext {
    pub charset: crate::style::Charset,
}

/// Context available during the layout pass.
pub struct LayoutContext {
    pub charset: crate::style::Charset,
    pub bounds: Rect,
}

/// Context available during the paint pass.
pub struct PaintContext {
    pub charset: crate::style::Charset,
    pub color: bool,
}
