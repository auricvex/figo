//! Bordered rectangular node widget.

use unicode_width::UnicodeWidthStr;

use crate::canvas::Layer;
use crate::render::surface::Surface;
use crate::render::widget::{LayoutContext, MeasureContext, PaintContext, Rect, Size, Widget};
use crate::style::{BorderStyle, Charset, HAlign, Padding, VAlign};
use crate::text::{align_horizontal, word_wrap};

/// A bordered rectangular node with optional title and content.
pub struct Node {
    pub title: Option<String>,
    pub content: Vec<String>,
    pub border: BorderStyle,
    pub padding: Padding,
    pub align: (HAlign, VAlign),
    pub charset: Charset,
    pub rect: Rect,
    pub min_size: Size,
}

impl Node {
    pub fn new(charset: Charset) -> Self {
        Self {
            title: None,
            content: Vec::new(),
            border: BorderStyle::Single,
            padding: Padding::default(),
            align: (HAlign::Left, VAlign::Top),
            charset,
            rect: Rect::default(),
            min_size: Size::new(4, 3),
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn content(mut self, lines: Vec<String>) -> Self {
        self.content = lines;
        self
    }

    pub fn border(mut self, border: BorderStyle) -> Self {
        self.border = border;
        self
    }

    pub fn padding(mut self, horizontal: usize, vertical: usize) -> Self {
        self.padding = Padding { horizontal, vertical };
        self
    }

    pub fn align(mut self, h: HAlign, v: VAlign) -> Self {
        self.align = (h, v);
        self
    }

    pub fn min_size(mut self, w: usize, h: usize) -> Self {
        self.min_size = Size::new(w, h);
        self
    }

    fn border_thickness(&self) -> usize {
        2
    }

    fn inner_width(&self) -> usize {
        let border = self.border_thickness();
        self.rect.w.saturating_sub(self.padding.horizontal * 2 + border)
    }

    fn inner_height(&self) -> usize {
        let border = self.border_thickness();
        self.rect.h.saturating_sub(self.padding.vertical * 2 + border)
    }
}

impl Widget for Node {
    fn measure(&self, _ctx: &MeasureContext) -> Size {
        let border = self.border_thickness();
        let title_w = self.title.as_ref().map_or(0, |t| UnicodeWidthStr::width(t.as_str()));
        let content_w =
            self.content.iter().map(|l| UnicodeWidthStr::width(l.as_str())).max().unwrap_or(0);
        let inner_w = title_w.max(content_w);
        let inner_h = if self.title.is_some() { 1usize } else { 0 } + self.content.len();
        let w = inner_w + self.padding.horizontal * 2 + border;
        let h = inner_h + self.padding.vertical * 2 + border;
        Size::new(w.max(self.min_size.w), h.max(self.min_size.h))
    }

    fn layout(&mut self, _ctx: &mut LayoutContext, rect: Rect) -> Rect {
        self.rect = rect;
        rect
    }

    fn paint(&self, ctx: &PaintContext, surface: &mut Surface<'_>) {
        let glyphs = self.border.glyphs(ctx.charset);
        surface.draw_rect(self.rect.x, self.rect.y, self.rect.w, self.rect.h, &glyphs);

        let inner_x = self.rect.x + 1 + self.padding.horizontal;
        let inner_y = self.rect.y + 1 + self.padding.vertical;
        let inner_w = self.inner_width();
        let inner_h = self.inner_height();

        if inner_w == 0 || inner_h == 0 {
            return;
        }

        let mut lines: Vec<String> = Vec::new();
        if let Some(title) = &self.title {
            lines.push(title.clone());
        }
        lines.extend(self.content.clone());

        let wrapped: Vec<String> = lines
            .iter()
            .flat_map(|line| {
                if UnicodeWidthStr::width(line.as_str()) <= inner_w {
                    vec![line.clone()]
                } else {
                    word_wrap(line, inner_w)
                }
            })
            .collect();

        let aligned = align_horizontal(&wrapped, inner_w, self.align.0);
        let total = aligned.len().min(inner_h);
        let start_y = match self.align.1 {
            VAlign::Top => 0,
            VAlign::Middle => (inner_h.saturating_sub(total)) / 2,
            VAlign::Bottom => inner_h.saturating_sub(total),
        };

        for (i, line) in aligned.iter().take(total).enumerate() {
            surface.put_str_layered(inner_x, inner_y + start_y + i, line, Layer::NodeContent);
        }
    }
}
