//! Simple flex and stack layout helpers.

use crate::render::surface::Surface;
use crate::render::widget::{LayoutContext, MeasureContext, PaintContext, Rect, Size, Widget};

/// Axis for stacks.
#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// A stack of widgets along one axis.
pub struct Stack {
    pub axis: Axis,
    pub spacing: usize,
    pub children: Vec<StackItem>,
    pub rect: Rect,
}

impl Stack {
    pub fn new(axis: Axis) -> Self {
        Self { axis, spacing: 0, children: Vec::new(), rect: Rect::default() }
    }

    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn add(mut self, widget: impl Widget + 'static, flex: usize) -> Self {
        self.children.push(StackItem { widget: Box::new(widget), flex, rect: Rect::default() });
        self
    }

    pub fn add_boxed(mut self, widget: Box<dyn Widget>, flex: usize) -> Self {
        self.children.push(StackItem { widget, flex, rect: Rect::default() });
        self
    }
}

pub struct StackItem {
    widget: Box<dyn Widget>,
    flex: usize,
    rect: Rect,
}

impl Widget for Stack {
    fn measure(&self, ctx: &MeasureContext) -> Size {
        let mut total_w = 0usize;
        let mut total_h = 0usize;
        let max_cross = 0usize;
        for (i, item) in self.children.iter().enumerate() {
            let size = item.widget.measure(ctx);
            match self.axis {
                Axis::Horizontal => {
                    total_w += size.w;
                    total_h = total_h.max(size.h);
                }
                Axis::Vertical => {
                    total_h += size.h;
                    total_w = total_w.max(size.w);
                }
            }
            if i + 1 < self.children.len() {
                match self.axis {
                    Axis::Horizontal => total_w += self.spacing,
                    Axis::Vertical => total_h += self.spacing,
                }
            }
        }
        Size::new(total_w, total_h.max(max_cross))
    }

    #[allow(clippy::manual_checked_ops)]
    fn layout(&mut self, ctx: &mut LayoutContext, rect: Rect) -> Rect {
        self.rect = rect;
        let total_flex: usize = self.children.iter().map(|c| c.flex).sum();
        let total_spacing = self.spacing * self.children.len().saturating_sub(1);
        let available = match self.axis {
            Axis::Horizontal => rect.w.saturating_sub(total_spacing),
            Axis::Vertical => rect.h.saturating_sub(total_spacing),
        };

        let child_count = self.children.len();
        let mut offset = 0usize;
        for item in &mut self.children {
            let share = if total_flex == 0 {
                available.checked_div(child_count).unwrap_or(0)
            } else {
                available * item.flex / total_flex
            };
            let child_rect = match self.axis {
                Axis::Horizontal => Rect::new(rect.x + offset, rect.y, share, rect.h),
                Axis::Vertical => Rect::new(rect.x, rect.y + offset, rect.w, share),
            };
            item.rect = child_rect;
            item.widget.layout(ctx, child_rect);
            offset += share + self.spacing;
        }
        rect
    }

    fn paint(&self, ctx: &PaintContext, surface: &mut Surface<'_>) {
        for item in &self.children {
            item.widget.paint(ctx, surface);
        }
    }
}

/// A flex container that distributes space by flex factor.
pub struct Flex {
    pub axis: Axis,
    pub spacing: usize,
    pub children: Vec<FlexItem>,
    pub rect: Rect,
}

impl Flex {
    pub fn new(axis: Axis) -> Self {
        Self { axis, spacing: 0, children: Vec::new(), rect: Rect::default() }
    }

    pub fn spacing(mut self, spacing: usize) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn add(mut self, widget: impl Widget + 'static, flex: usize) -> Self {
        self.children.push(FlexItem { widget: Box::new(widget), flex, rect: Rect::default() });
        self
    }
}

pub struct FlexItem {
    widget: Box<dyn Widget>,
    flex: usize,
    rect: Rect,
}

impl Widget for Flex {
    fn measure(&self, ctx: &MeasureContext) -> Size {
        let mut total_main = 0usize;
        let mut max_cross = 0usize;
        for (i, item) in self.children.iter().enumerate() {
            let size = item.widget.measure(ctx);
            match self.axis {
                Axis::Horizontal => {
                    total_main += size.w;
                    max_cross = max_cross.max(size.h);
                }
                Axis::Vertical => {
                    total_main += size.h;
                    max_cross = max_cross.max(size.w);
                }
            }
            if i + 1 < self.children.len() {
                total_main += self.spacing;
            }
        }
        match self.axis {
            Axis::Horizontal => Size::new(total_main, max_cross),
            Axis::Vertical => Size::new(max_cross, total_main),
        }
    }

    #[allow(clippy::manual_checked_ops)]
    fn layout(&mut self, ctx: &mut LayoutContext, rect: Rect) -> Rect {
        self.rect = rect;
        let total_flex: usize = self.children.iter().map(|c| c.flex).sum();
        let total_spacing = self.spacing * self.children.len().saturating_sub(1);
        let available = match self.axis {
            Axis::Horizontal => rect.w.saturating_sub(total_spacing),
            Axis::Vertical => rect.h.saturating_sub(total_spacing),
        };

        let mut offset = 0usize;
        for item in &mut self.children {
            let share = if total_flex == 0 { 0 } else { available * item.flex / total_flex };
            let child_rect = match self.axis {
                Axis::Horizontal => Rect::new(rect.x + offset, rect.y, share, rect.h),
                Axis::Vertical => Rect::new(rect.x, rect.y + offset, rect.w, share),
            };
            item.rect = child_rect;
            item.widget.layout(ctx, child_rect);
            offset += share + self.spacing;
        }
        rect
    }

    fn paint(&self, ctx: &PaintContext, surface: &mut Surface<'_>) {
        for item in &self.children {
            item.widget.paint(ctx, surface);
        }
    }
}
