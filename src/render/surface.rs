//! Clipped drawing surface backed by the low-level [`Canvas`].

use crate::canvas::{Canvas, Layer};
use crate::style::{BorderGlyphs, BorderStyle, Charset};

/// A rectangular drawing region with an optional clip rectangle.
///
/// `Surface` wraps a [`Canvas`] and translates all coordinates by an offset,
/// so widgets can paint in their own local coordinate system. It also
/// supports sub-surfaces for nested clipping.
#[derive(Debug)]
pub struct Surface<'a> {
    canvas: &'a mut Canvas,
    offset_x: usize,
    offset_y: usize,
    clip_w: usize,
    clip_h: usize,
}

impl<'a> Surface<'a> {
    /// Create a surface that covers the entire canvas.
    pub fn new(canvas: &'a mut Canvas) -> Self {
        let w = canvas.width();
        let h = canvas.height();
        Self { canvas, offset_x: 0, offset_y: 0, clip_w: w, clip_h: h }
    }

    /// Create a sub-surface clipped to the given rectangle (local coords).
    pub fn sub(&mut self, x: usize, y: usize, w: usize, h: usize) -> Surface<'_> {
        Surface {
            canvas: &mut *self.canvas,
            offset_x: self.offset_x + x,
            offset_y: self.offset_y + y,
            clip_w: w.min(self.clip_w.saturating_sub(x)),
            clip_h: h.min(self.clip_h.saturating_sub(y)),
        }
    }

    /// Width of the clipped region.
    pub fn width(&self) -> usize {
        self.clip_w
    }

    /// Height of the clipped region.
    pub fn height(&self) -> usize {
        self.clip_h
    }

    fn global(&self, x: usize, y: usize) -> (usize, usize) {
        (self.offset_x + x, self.offset_y + y)
    }

    fn inside(&self, x: usize, y: usize) -> bool {
        x < self.clip_w && y < self.clip_h
    }

    /// Put a single character at `(x, y)`.
    pub fn put(&mut self, x: usize, y: usize, ch: char) {
        if self.inside(x, y) {
            let (gx, gy) = self.global(x, y);
            self.canvas.put(gx, gy, ch);
        }
    }

    /// Put a single character at a specific layer.
    pub fn put_layered(&mut self, x: usize, y: usize, ch: char, layer: Layer) {
        if self.inside(x, y) {
            let (gx, gy) = self.global(x, y);
            self.canvas.put_layered(gx, gy, ch, layer, None);
        }
    }

    /// Write a horizontal string.
    pub fn put_str(&mut self, x: usize, y: usize, s: &str) {
        if y >= self.clip_h {
            return;
        }
        for (i, ch) in s.chars().enumerate() {
            if x + i >= self.clip_w {
                break;
            }
            self.put(x + i, y, ch);
        }
    }

    /// Write a horizontal string at a specific layer.
    pub fn put_str_layered(&mut self, x: usize, y: usize, s: &str, layer: Layer) {
        if y >= self.clip_h {
            return;
        }
        for (i, ch) in s.chars().enumerate() {
            if x + i >= self.clip_w {
                break;
            }
            self.put_layered(x + i, y, ch, layer);
        }
    }

    /// Draw a horizontal line.
    pub fn put_horizontal(&mut self, x: usize, y: usize, len: usize, ch: char, layer: Layer) {
        for dx in 0..len {
            self.put_layered(x + dx, y, ch, layer);
        }
    }

    /// Draw a vertical line.
    pub fn put_vertical(&mut self, x: usize, y: usize, len: usize, ch: char, layer: Layer) {
        for dy in 0..len {
            self.put_layered(x, y + dy, ch, layer);
        }
    }

    /// Draw a rectangle border using the given glyphs.
    pub fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, glyphs: &BorderGlyphs) {
        if w < 2 || h < 2 {
            return;
        }
        self.put_layered(x, y, glyphs.top_left, Layer::NodeBorder);
        self.put_layered(x + w - 1, y, glyphs.top_right, Layer::NodeBorder);
        self.put_layered(x, y + h - 1, glyphs.bottom_left, Layer::NodeBorder);
        self.put_layered(x + w - 1, y + h - 1, glyphs.bottom_right, Layer::NodeBorder);
        self.put_horizontal(x + 1, y, w - 2, glyphs.horizontal, Layer::NodeBorder);
        self.put_horizontal(x + 1, y + h - 1, w - 2, glyphs.horizontal, Layer::NodeBorder);
        self.put_vertical(x, y + 1, h - 2, glyphs.vertical, Layer::NodeBorder);
        self.put_vertical(x + w - 1, y + 1, h - 2, glyphs.vertical, Layer::NodeBorder);
    }

    /// Fill a rectangular region with a character.
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, ch: char) {
        for dy in 0..h {
            for dx in 0..w {
                self.put(x + dx, y + dy, ch);
            }
        }
    }

    /// Draw a border using a style and charset.
    pub fn draw_border(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        style: BorderStyle,
        charset: Charset,
    ) {
        let glyphs = style.glyphs(charset);
        self.draw_rect(x, y, w, h, &glyphs);
    }

    /// Render the underlying canvas to a string.
    pub fn render(&self, color: bool) -> String {
        self.canvas.render(color)
    }
}
