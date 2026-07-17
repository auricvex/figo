//! Node component — a labeled, possibly-bordered rectangular region on the
//! canvas.
//!
//! A node owns a [`Rect`] (position + size), an optional border style, an
//! optional title, and a list of content rows. It computes its own minimum
//! size from its content and is responsible for drawing itself at the
//! `Layer::NodeBorder` and `Layer::NodeContent` z-layers so connectors
//! drawn earlier are cleanly clipped at its borders.
//!
//! Use the [`NodeBuilder`] to construct nodes ergonomically.

use unicode_width::UnicodeWidthStr;

use crate::canvas::{Canvas, Layer};
use crate::style::{BorderStyle, Charset, HAlign, LineStyle, Padding, VAlign};
use crate::text::{align_horizontal, word_wrap};

use super::geom::{Anchor, Rect};

/// Stable identifier for a node. Assigned by the builder when nodes are
/// added; diagrams keep a `Vec<Node>` and address nodes by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// A single content row inside a [`Node`]. Either a plain string or a
/// pre-formatted line that is exactly `width` characters wide.
#[derive(Clone, Debug)]
pub enum NodeLine {
    /// Plain text. Will be word-wrapped to the available width when the
    /// node's content area is sized.
    Text(String),
    /// Pre-wrapped text, one cell per `chars` entry. Used when content has
    /// already been wrapped (table cells, packet fields).
    Raw(Vec<String>),
}

impl NodeLine {
    fn widest(&self) -> usize {
        match self {
            NodeLine::Text(s) => UnicodeWidthStr::width(s.as_str()),
            NodeLine::Raw(rows) => {
                rows.iter().map(|r| UnicodeWidthStr::width(r.as_str())).max().unwrap_or(0)
            }
        }
    }
}

/// A node to be drawn on the canvas.
///
/// Nodes are drawn in two passes by [`Node::render`]:
/// 1. Border at `Layer::NodeBorder`.
/// 2. Content text at `Layer::NodeContent`.
///
/// The border protects against earlier connector lines; the content
/// overwrites anything inside the border.
#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub rect: Rect,
    pub border: Option<BorderStyle>,
    pub fill: Option<char>,
    pub title: Option<String>,
    pub title_align: HAlign,
    pub content: Vec<NodeLine>,
    pub padding: Padding,
    pub align: (HAlign, VAlign),
    pub charset: Charset,
    /// If true, the node's title is placed inside the top border edge as
    /// an inline label rather than as the first content row.
    pub title_in_border: bool,
}

impl Node {
    /// Create a node with required fields. Use [`NodeBuilder`] for a more
    /// readable API.
    pub fn new(id: NodeId, charset: Charset, rect: Rect) -> Self {
        Self {
            id,
            rect,
            border: None,
            fill: None,
            title: None,
            title_align: HAlign::Left,
            content: Vec::new(),
            padding: Padding::default(),
            align: (HAlign::Left, VAlign::Top),
            charset,
            title_in_border: false,
        }
    }

    /// Compute the minimum width this node requires to display its
    /// content. Includes border + padding.
    pub fn min_width(&self) -> usize {
        let inner = self.content.iter().map(NodeLine::widest).max().unwrap_or(0);
        let title_w = self.title.as_deref().map_or(0, UnicodeWidthStr::width);
        let border = if self.border.is_some() { 2 } else { 0 };
        inner.max(title_w) + self.padding.horizontal * 2 + border
    }

    /// Compute the minimum height (rows) this node needs.
    pub fn min_height(&self) -> usize {
        let title_row = if self.title.is_some() && !self.title_in_border { 1 } else { 0 };
        let mut content_rows = 0usize;
        for line in &self.content {
            content_rows += match line {
                NodeLine::Text(_) => 1,
                NodeLine::Raw(rows) => rows.len(),
            };
        }
        let border = if self.border.is_some() { 2 } else { 0 };
        title_row + content_rows + self.padding.vertical * 2 + border
    }

    /// Anchor point on this node's perimeter.
    pub fn anchor(&self, a: Anchor) -> (usize, usize) {
        self.rect.anchor(a)
    }

    /// Render this node into `canvas`. Borders go at `Layer::NodeBorder`,
    /// content at `Layer::NodeContent`.
    pub fn render(&self, canvas: &mut Canvas) {
        if let Some(b) = self.border {
            let g = b.glyphs(self.charset);
            let _ = canvas.draw_rect(self.rect.x, self.rect.y, self.rect.w, self.rect.h, &g);
        }
        if let Some(fill_ch) = self.fill {
            for y in (self.rect.y + 1)..(self.rect.bottom().saturating_sub(1)) {
                for x in (self.rect.x + 1)..(self.rect.right().saturating_sub(1)) {
                    canvas.put_layered(x, y, fill_ch, Layer::NodeContent, None);
                }
            }
        }

        let inner_x =
            self.rect.x + self.padding.horizontal + if self.border.is_some() { 1 } else { 0 };
        let inner_y =
            self.rect.y + self.padding.vertical + if self.border.is_some() { 1 } else { 0 };
        let inner_w = self.rect.w.saturating_sub(
            self.padding.horizontal * 2 + if self.border.is_some() { 2 } else { 0 },
        );
        let inner_h = self
            .rect
            .h
            .saturating_sub(self.padding.vertical * 2 + if self.border.is_some() { 2 } else { 0 });

        if inner_w == 0 || inner_h == 0 {
            return;
        }

        // Optional border-overlaid title.
        if self.title_in_border {
            if let Some(title) = &self.title {
                self.draw_inline_border_title(canvas, title);
            }
        }

        // Compose content rows.
        let mut rows: Vec<String> = Vec::new();
        if let Some(title) = &self.title {
            if !self.title_in_border {
                rows.push(title.clone());
            }
        }
        for line in &self.content {
            match line {
                NodeLine::Text(s) => rows.push(s.clone()),
                NodeLine::Raw(lines) => rows.extend(lines.iter().cloned()),
            }
        }
        let wrapped: Vec<Vec<String>> = rows
            .iter()
            .map(|r| {
                if UnicodeWidthStr::width(r.as_str()) <= inner_w {
                    vec![r.clone()]
                } else {
                    word_wrap(r, inner_w)
                }
            })
            .collect();
        let mut flat: Vec<String> = Vec::new();
        for group in wrapped {
            flat.extend(group);
        }

        // Apply horizontal alignment per line.
        let aligned = align_horizontal(&flat, inner_w, self.align.0);

        // Compute vertical start so content is vertically aligned.
        let total = aligned.len();
        let start_y = match self.align.1 {
            VAlign::Top => 0,
            VAlign::Middle => inner_h.saturating_sub(total) / 2,
            VAlign::Bottom => inner_h.saturating_sub(total),
        };

        for (i, line) in aligned.iter().enumerate() {
            let y = inner_y + start_y + i;
            if y >= inner_y + inner_h {
                break;
            }
            canvas.put_str_layered(inner_x, y, line, Layer::NodeContent, None);
        }
    }

    fn draw_inline_border_title(&self, canvas: &mut Canvas, title: &str) {
        let top_y = self.rect.y;
        let max_title_w = self.rect.w.saturating_sub(4);
        let display: String = title.chars().take(max_title_w).collect();
        let display_w = UnicodeWidthStr::width(display.as_str());
        match self.title_align {
            HAlign::Left => {
                let x = self.rect.x + 2;
                canvas.put_str_layered(x, top_y, &format!(" {display} "), Layer::NodeContent, None);
            }
            HAlign::Center => {
                let x = self.rect.x + (self.rect.w.saturating_sub(display_w + 2)) / 2;
                canvas.put_str_layered(x, top_y, &format!(" {display} "), Layer::NodeContent, None);
            }
            HAlign::Right => {
                let x = self.rect.x + self.rect.w.saturating_sub(display_w + 3);
                canvas.put_str_layered(x, top_y, &format!(" {display} "), Layer::NodeContent, None);
            }
        }
    }
}

/// A builder for [`Node`] with optional ergonomic helpers. Diagram code
/// constructs nodes directly, so this is more of a convenience.
pub struct NodeBuilder {
    charset: Charset,
    id: NodeId,
    border: Option<BorderStyle>,
    title: Option<String>,
    content: Vec<NodeLine>,
    padding: Padding,
    align: (HAlign, VAlign),
    min_size: (usize, usize),
}

impl NodeBuilder {
    /// Start a new node builder.
    pub fn new(id: NodeId, charset: Charset) -> Self {
        Self {
            charset,
            id,
            border: Some(BorderStyle::Single),
            title: None,
            content: Vec::new(),
            padding: Padding::default(),
            align: (HAlign::Left, VAlign::Top),
            min_size: (4, 3),
        }
    }

    /// Hide the border.
    pub fn no_border(mut self) -> Self {
        self.border = None;
        self
    }

    /// Set the title (rendered as the first content row by default).
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Mark the title to be drawn inside the top border edge.
    pub fn title_in_border(mut self) -> Self {
        // Recorded in the node when built.
        self.title = self.title.take().or(Some(String::new()));
        self
    }

    /// Add a row of plain text content.
    pub fn row(mut self, text: impl Into<String>) -> Self {
        self.content.push(NodeLine::Text(text.into()));
        self
    }

    /// Add a row of pre-formatted lines (e.g. from `word_wrap`).
    pub fn raw(mut self, lines: Vec<String>) -> Self {
        self.content.push(NodeLine::Raw(lines));
        self
    }

    /// Set padding around the content.
    pub fn padding(mut self, p: Padding) -> Self {
        self.padding = p;
        self
    }

    /// Set content alignment.
    pub fn align(mut self, h: HAlign, v: VAlign) -> Self {
        self.align = (h, v);
        self
    }

    /// Force a minimum size (width, height).
    pub fn min_size(mut self, w: usize, h: usize) -> Self {
        self.min_size = (w.max(1), h.max(1));
        self
    }

    /// Build the node, computing its size.
    pub fn build(self) -> Node {
        let mut node = Node::new(self.id, self.charset, Rect::default());
        node.border = self.border;
        node.title = self.title;
        node.content = self.content;
        node.padding = self.padding;
        node.align = self.align;
        node.title_in_border = self.border.is_some();
        let w = node.min_width().max(self.min_size.0);
        let h = node.min_height().max(self.min_size.1);
        node.rect = Rect::new(0, 0, w, h);
        node
    }
}

/// Convenience: a horizontal arrow line glyph for the given line style
/// and charset.
pub fn horizontal_line_glyph(style: LineStyle, charset: Charset) -> char {
    match (style, charset) {
        (LineStyle::Bold, _) => '━',
        (LineStyle::BoxDrawing, _) | (LineStyle::Simple, Charset::Unicode) => '─',
        (LineStyle::Simple, Charset::Ascii) => '-',
    }
}

/// Convenience: a vertical arrow line glyph.
pub fn vertical_line_glyph(style: LineStyle, charset: Charset) -> char {
    match (style, charset) {
        (LineStyle::Bold, _) => '┃',
        (LineStyle::BoxDrawing, _) | (LineStyle::Simple, Charset::Unicode) => '│',
        (LineStyle::Simple, Charset::Ascii) => '|',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_min_width_includes_border_and_padding() {
        let n = NodeBuilder::new(NodeId(1), Charset::Ascii)
            .row("hi")
            .padding(Padding { horizontal: 2, vertical: 0 })
            .build();
        // inner width = 2 (max of all content), padding 2 each side, border 2
        assert!(n.min_width() >= 2 + 4 + 2);
    }
}
