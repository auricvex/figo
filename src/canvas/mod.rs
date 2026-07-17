//! Internal 2-D grid rendering engine used by all diagram types.
//!
//! The canvas stores a `Vec<Cell>` indexed by `y * width + x`, zero-indexed
//! with the origin at the top-left. Each cell carries a z-order [`Layer`]
//! (see [`cell::Layer`]) so connectors and node borders can be drawn in any
//! order while still producing a clean, non-overlapping final image.
//!
//! Typical draw pipeline for a diagram with nodes + connectors:
//!
//! 1. Place all nodes (`Canvas::put_layered` at `Layer::NodeBorder` and
//!    `Layer::NodeContent`).
//! 2. Connect them with [`crate::layout::Connector`] (which writes at
//!    `Layer::Connector` first, then the arrowhead at `Layer::ConnectorEnd`).
//!
//! Because node borders are drawn AFTER connector lines but BEFORE the
//! arrowhead, a connector that ends at a node's border is naturally
//! truncated by the border, while the arrowhead sits one cell outside the
//! node and points inward.

mod cell;

pub use cell::{Cell, Layer};

use crate::error::Result;
use crate::style::{Charset, Color, LineStyle};

/// A 2-D grid buffer for building text diagrams.
#[derive(Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    pub(crate) cells: Vec<Cell>,
}

impl Canvas {
    /// Create a new canvas of the given width and height, filled with empty
    /// background cells. Zero dimensions are normalized to 1×1 to avoid
    /// out-of-bounds writes; callers should pre-validate required sizes.
    pub fn new(width: usize, height: usize) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let cells = vec![Cell::default(); width * height];
        Self { width, height, cells }
    }

    /// Canvas width in cells.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Canvas height in cells.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Look up a cell at `(x, y)`. Returns `None` for out-of-bounds.
    pub fn cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height { Some(&self.cells[y * self.width + x]) } else { None }
    }

    /// Convenience: the character at `(x, y)`, or `None` out-of-bounds.
    pub fn cell_char(&self, x: usize, y: usize) -> Option<char> {
        self.cell(x, y).map(|c| c.ch)
    }

    /// Write a single character at `Layer::NodeContent` (highest common
    /// layer for cell text). Higher layers already in place are preserved.
    /// Use [`put_layered`](Self::put_layered) for explicit z-order control.
    pub fn put(&mut self, x: usize, y: usize, ch: char) {
        self.put_layered(x, y, ch, Layer::NodeContent, None);
    }

    /// Write a single character at a specific z-layer.
    ///
    /// If the existing cell is at a strictly higher layer, the write is
    /// silently dropped — this is the mechanism that prevents connectors
    /// from punching through node borders.
    pub fn put_layered(&mut self, x: usize, y: usize, ch: char, layer: Layer, fg: Option<Color>) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y * self.width + x;
        let should_write = match self.cells.get(idx) {
            Some(existing) => layer >= existing.layer,
            None => true,
        };
        if should_write {
            self.cells[idx] = Cell::at_colored(ch, layer, fg);
        }
    }

    /// Write a horizontal string at `Layer::NodeContent` by default — the
    /// common case is placing text inside a node or a label, which should
    /// appear above any connector lines or borders. New code that needs
    /// explicit z-order should use [`put_str_layered`](Self::put_str_layered).
    pub fn put_str(&mut self, x: usize, y: usize, s: &str) {
        self.put_str_layered(x, y, s, Layer::NodeContent, None);
    }

    /// Write a horizontal string at a specific layer.
    pub fn put_str_layered(
        &mut self,
        x: usize,
        y: usize,
        s: &str,
        layer: Layer,
        fg: Option<Color>,
    ) {
        for (i, ch) in s.chars().enumerate() {
            self.put_layered(x + i, y, ch, layer, fg);
        }
    }

    /// Draw a vertical line of `ch` at `Layer::Connector` by default —
    /// safe for connector lines, lifelines, and timeline bars that should
    /// sit below node borders. New code that wants explicit z-order should
    /// call [`put_vertical_layered`].
    ///
    /// [`put_vertical_layered`]: Self::put_vertical_layered
    pub fn put_vertical(&mut self, x: usize, y: usize, len: usize, ch: char) {
        self.put_vertical_layered(x, y, len, ch, Layer::Connector);
    }

    /// Draw a horizontal line of `ch` at `Layer::Connector` by default.
    /// See [`put_vertical`](Self::put_vertical) for rationale.
    pub fn put_horizontal(&mut self, x: usize, y: usize, len: usize, ch: char) {
        self.put_horizontal_layered(x, y, len, ch, Layer::Connector);
    }

    /// Vertical line drawing with explicit z-layer.
    pub fn put_vertical_layered(&mut self, x: usize, y: usize, len: usize, ch: char, layer: Layer) {
        for dy in 0..len {
            self.put_layered(x, y + dy, ch, layer, None);
        }
    }

    /// Horizontal line drawing with explicit z-layer.
    pub fn put_horizontal_layered(
        &mut self,
        x: usize,
        y: usize,
        len: usize,
        ch: char,
        layer: Layer,
    ) {
        for dx in 0..len {
            self.put_layered(x + dx, y, ch, layer, None);
        }
    }

    /// Draw a rectangle border using a `BorderGlyphs` struct.
    pub fn draw_rect(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        glyphs: &crate::style::BorderGlyphs,
    ) -> Result<()> {
        self.draw_rect_raw(
            x,
            y,
            w,
            h,
            glyphs.top_left,
            glyphs.top_right,
            glyphs.bottom_left,
            glyphs.bottom_right,
            glyphs.horizontal,
            glyphs.vertical,
        )
    }

    /// Draw a rectangle border using individual corner and edge characters.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_rect_raw(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        tl: char,
        tr: char,
        bl: char,
        br: char,
        horiz: char,
        vert: char,
    ) -> Result<()> {
        if w < 2 || h < 2 {
            return Err(crate::error::FigoError::InvalidDimensions(format!(
                "rectangle at ({x},{y}) must be at least 2x2, got {w}x{h}"
            )));
        }
        self.put_layered(x, y, tl, Layer::NodeBorder, None);
        self.put_layered(x + w - 1, y, tr, Layer::NodeBorder, None);
        self.put_layered(x, y + h - 1, bl, Layer::NodeBorder, None);
        self.put_layered(x + w - 1, y + h - 1, br, Layer::NodeBorder, None);
        self.put_horizontal_layered(x + 1, y, w - 2, horiz, Layer::NodeBorder);
        self.put_horizontal_layered(x + 1, y + h - 1, w - 2, horiz, Layer::NodeBorder);
        self.put_vertical_layered(x, y + 1, h - 2, vert, Layer::NodeBorder);
        self.put_vertical_layered(x + w - 1, y + 1, h - 2, vert, Layer::NodeBorder);
        Ok(())
    }

    /// Draw an `BoxDrawing` border using the canonical glyphs for the
    /// given charset and border style.
    pub fn draw_border(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        border: crate::style::BorderStyle,
        charset: crate::style::Charset,
    ) -> Result<()> {
        let g = border.glyphs(charset);
        self.draw_rect(x, y, w, h, &g)
    }

    /// Grow the canvas vertically by adding `extra` blank rows at the bottom.
    pub fn extend_vertical(&mut self, extra: usize) {
        let new_cells = vec![Cell::default(); self.width * extra];
        self.cells.extend(new_cells);
        self.height += extra;
    }

    /// Grow the canvas horizontally (right side) by `extra` columns.
    pub fn extend_horizontal(&mut self, extra: usize) {
        if extra == 0 {
            return;
        }
        let new_w = self.width + extra;
        let mut new_cells: Vec<Cell> = Vec::with_capacity(new_w * self.height);
        for _y in 0..self.height {
            for _ in 0..self.width {
                new_cells.push(Cell::default());
            }
            for _ in 0..extra {
                new_cells.push(Cell::default());
            }
        }
        self.cells = new_cells;
        self.width = new_w;
    }

    /// Ensure the canvas is at least `min_height` rows tall.
    pub fn ensure_height(&mut self, min_height: usize) {
        if self.height < min_height {
            self.extend_vertical(min_height - self.height);
        }
    }

    /// Ensure the canvas is at least `min_width` columns wide.
    pub fn ensure_width(&mut self, min_width: usize) {
        if self.width < min_width {
            self.extend_horizontal(min_width - self.width);
        }
    }

    /// Clear all cells back to background.
    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    /// Fill the entire canvas with the given character at `Layer::Background`.
    pub fn fill(&mut self, ch: char) {
        for cell in &mut self.cells {
            cell.ch = ch;
            cell.layer = Layer::Background;
        }
    }

    /// Repair connector-line junctions so corners and crossings use the
    /// correct box-drawing glyph.
    ///
    /// Call this after all connectors have been drawn. It scans cells at
    /// [`Layer::Connector`] and replaces straight line characters with the
    /// appropriate corner, tee, or cross character when horizontal and
    /// vertical connector segments meet.
    pub fn repair_connector_junctions(&mut self, style: LineStyle, charset: Charset) {
        if charset == Charset::Ascii {
            self.repair_ascii_junctions();
        } else {
            self.repair_unicode_junctions(style);
        }
    }

    fn repair_ascii_junctions(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_connector_line(x, y) {
                    continue;
                }
                let dirs = self.connector_directions(x, y);
                // Only convert to '+' where horizontal and vertical segments
                // actually meet (corners, tees, crosses). Straight runs must
                // keep their '-' or '|' glyph.
                if dirs.count() >= 2 && (dirs.e || dirs.w) && (dirs.n || dirs.s) {
                    self.put_layered(x, y, '+', Layer::Connector, None);
                }
            }
        }
    }

    fn repair_unicode_junctions(&mut self, style: LineStyle) {
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.is_connector_line(x, y) {
                    continue;
                }
                let dirs = self.connector_directions(x, y);
                if let Some(ch) = junction_char(dirs, style) {
                    self.put_layered(x, y, ch, Layer::Connector, None);
                }
            }
        }
    }

    fn is_connector_line(&self, x: usize, y: usize) -> bool {
        let Some(cell) = self.cell(x, y) else { return false };
        cell.layer == Layer::Connector && cell.ch != ' '
    }

    fn connector_directions(&self, x: usize, y: usize) -> Directions {
        Directions {
            n: y > 0 && self.is_connector_line(x, y - 1),
            s: y + 1 < self.height && self.is_connector_line(x, y + 1),
            e: x + 1 < self.width && self.is_connector_line(x + 1, y),
            w: x > 0 && self.is_connector_line(x - 1, y),
        }
    }

    /// Render the canvas as a string. Trailing whitespace per row is
    /// trimmed. When `color` is true, ANSI escapes are emitted for
    /// foreground colors per cell.
    pub fn render(&self, color: bool) -> String {
        let mut out = String::with_capacity(self.width * (self.height + 1));
        for y in 0..self.height {
            let row = &self.cells[y * self.width..(y + 1) * self.width];
            let last_non_space = row.iter().rposition(|c| c.ch != ' ');
            let end = last_non_space.map_or(0, |i| i + 1);

            if color {
                let mut current_fg: Option<Color> = None;
                for cell in row.iter().take(end) {
                    if cell.fg != current_fg {
                        current_fg = cell.fg;
                        match current_fg {
                            Some(c) => out.push_str(c.fg_code()),
                            None => out.push_str("\x1b[0m"),
                        }
                    }
                    out.push(cell.ch);
                }
                if current_fg.is_some() {
                    out.push_str("\x1b[0m");
                }
            } else {
                for cell in row.iter().take(end) {
                    out.push(cell.ch);
                }
            }
            out.push('\n');
        }
        out
    }
}

/// Cardinal directions used when deciding which junction glyph a
/// connector cell should become.
#[derive(Clone, Copy, Debug)]
struct Directions {
    n: bool,
    s: bool,
    e: bool,
    w: bool,
}

impl Directions {
    fn count(self) -> usize {
        [self.n, self.s, self.e, self.w].iter().filter(|&&b| b).count()
    }
}

fn junction_char(dirs: Directions, style: LineStyle) -> Option<char> {
    if dirs.count() < 2 {
        return None;
    }
    match style {
        LineStyle::Bold => match (dirs.n, dirs.s, dirs.e, dirs.w) {
            (true, false, true, false) => Some('┗'),
            (true, false, false, true) => Some('┛'),
            (false, true, true, false) => Some('┏'),
            (false, true, false, true) => Some('┓'),
            (true, true, true, false) => Some('┣'),
            (true, true, false, true) => Some('┫'),
            (true, false, true, true) => Some('┳'),
            (false, true, true, true) => Some('┻'),
            (true, true, true, true) => Some('╋'),
            _ => None,
        },
        LineStyle::Simple | LineStyle::BoxDrawing => match (dirs.n, dirs.s, dirs.e, dirs.w) {
            (true, false, true, false) => Some('└'),
            (true, false, false, true) => Some('┘'),
            (false, true, true, false) => Some('┌'),
            (false, true, false, true) => Some('┐'),
            (true, true, true, false) => Some('├'),
            (true, true, false, true) => Some('┤'),
            (true, false, true, true) => Some('┬'),
            (false, true, true, true) => Some('┴'),
            (true, true, true, true) => Some('┼'),
            _ => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::BorderStyle;
    use crate::style::Charset;

    #[test]
    fn test_canvas_put_and_render() {
        let mut c = Canvas::new(5, 3);
        c.put_str(0, 0, "Hello");
        let out = c.render(false);
        assert_eq!(out, "Hello\n\n\n");
    }

    #[test]
    fn test_canvas_layer_protects_border() {
        // Connector at lower layer must NOT overwrite node border.
        let mut c = Canvas::new(8, 3);
        c.draw_border(2, 0, 4, 3, BorderStyle::Single, Charset::Ascii).unwrap();
        c.put_horizontal_layered(0, 1, 8, '-', Layer::Connector);
        let out = c.render(false);
        // Border verticals should survive at x=2 and x=5.
        assert!(out.contains("|"), "expected vertical border |: {out:?}");
        assert!(
            !out.lines().nth(1).unwrap().chars().nth(2).unwrap_or(' ').eq(&'-'),
            "border should protect against connector overwrite"
        );
    }

    #[test]
    fn test_canvas_extend_vertical() {
        let mut c = Canvas::new(5, 1);
        c.put_str(0, 0, "A");
        c.extend_vertical(2);
        assert_eq!(c.height(), 3);
        c.put_str(0, 2, "B");
        assert_eq!(c.render(false), "A\n\nB\n");
    }

    #[test]
    fn test_zero_dim_normalized() {
        let c = Canvas::new(0, 5);
        assert_eq!(c.width(), 1);
        let c = Canvas::new(5, 0);
        assert_eq!(c.height(), 1);
    }

    #[test]
    fn repair_makes_cross_at_intersection() {
        let mut c = Canvas::new(5, 5);
        c.put_horizontal_layered(0, 2, 5, '─', Layer::Connector);
        c.put_vertical_layered(2, 0, 5, '│', Layer::Connector);
        c.repair_connector_junctions(LineStyle::Simple, Charset::Unicode);
        assert_eq!(c.cell_char(2, 2), Some('┼'));
    }

    #[test]
    fn repair_makes_corner_for_vhv_path() {
        let mut c = Canvas::new(6, 6);
        // Vertical down from (1,0) to (1,3), horizontal to (4,3), vertical down to (4,5).
        c.put_vertical_layered(1, 0, 4, '│', Layer::Connector);
        c.put_horizontal_layered(1, 3, 4, '─', Layer::Connector);
        c.put_vertical_layered(4, 3, 3, '│', Layer::Connector);
        c.repair_connector_junctions(LineStyle::Simple, Charset::Unicode);
        // At (1,3) lines go up and right -> bottom-left corner.
        assert_eq!(c.cell_char(1, 3), Some('└'));
        // At (4,3) lines go down and left -> top-right corner.
        assert_eq!(c.cell_char(4, 3), Some('┐'));
    }

    #[test]
    fn repair_uses_plus_for_ascii_junctions() {
        let mut c = Canvas::new(5, 5);
        c.put_horizontal_layered(0, 2, 5, '-', Layer::Connector);
        c.put_vertical_layered(2, 0, 5, '|', Layer::Connector);
        c.repair_connector_junctions(LineStyle::Simple, Charset::Ascii);
        assert_eq!(c.cell_char(2, 2), Some('+'));
    }
}
