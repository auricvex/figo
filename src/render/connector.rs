//! Orthogonal connector routing between two rectangles.

use crate::canvas::Layer;
use crate::render::surface::Surface;
use crate::render::widget::{PaintContext, Rect};
use crate::style::{Charset, LineStyle};

/// Marker placed at the target end of a connector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    Arrow,
    CrowFoot,
    Dot,
    None,
}

/// Style of a connector.
#[derive(Clone, Copy, Debug)]
pub struct ConnectorStyle {
    pub line: LineStyle,
    pub marker: Marker,
    pub charset: Charset,
}

impl ConnectorStyle {
    pub fn new(line: LineStyle, marker: Marker, charset: Charset) -> Self {
        Self { line, marker, charset }
    }
}

/// Anchor point on a rectangle's perimeter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    North,
    South,
    East,
    West,
}

/// An orthogonal connector between two rectangles.
pub struct Connector {
    pub from: Rect,
    pub to: Rect,
    pub from_anchor: Anchor,
    pub to_anchor: Anchor,
    pub style: ConnectorStyle,
    pub label: Option<String>,
    pub avoid: Vec<Rect>,
}

impl Connector {
    /// Create a connector between two rectangles using their perimeters.
    pub fn new(from: Rect, to: Rect, style: ConnectorStyle) -> Self {
        Self {
            from,
            to,
            from_anchor: Anchor::South,
            to_anchor: Anchor::North,
            style,
            label: None,
            avoid: Vec::new(),
        }
    }

    /// Set explicit anchor points.
    pub fn anchors(mut self, from: Anchor, to: Anchor) -> Self {
        self.from_anchor = from;
        self.to_anchor = to;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn avoid(mut self, rects: impl IntoIterator<Item = Rect>) -> Self {
        self.avoid.extend(rects);
        self
    }

    fn line_char(&self) -> char {
        match (self.style.line, self.style.charset) {
            (LineStyle::Bold, _) => '━',
            (_, Charset::Unicode) | (LineStyle::BoxDrawing, _) => '─',
            (_, Charset::Ascii) => '-',
        }
    }

    fn vert_char(&self) -> char {
        match (self.style.line, self.style.charset) {
            (LineStyle::Bold, _) => '┃',
            (_, Charset::Unicode) | (LineStyle::BoxDrawing, _) => '│',
            (_, Charset::Ascii) => '|',
        }
    }

    fn arrow_char(&self, dx: i32, dy: i32) -> char {
        match (self.style.charset, dx, dy) {
            (Charset::Unicode, 1, 0) => '→',
            (Charset::Unicode, -1, 0) => '←',
            (Charset::Unicode, 0, 1) => '↓',
            (Charset::Unicode, 0, -1) => '↑',
            (Charset::Ascii, 1, 0) => '>',
            (Charset::Ascii, -1, 0) => '<',
            (Charset::Ascii, 0, 1) => 'v',
            (Charset::Ascii, 0, -1) => '^',
            _ => '>',
        }
    }

    fn perimeter_point(rect: &Rect, anchor: Anchor) -> (usize, usize) {
        let (cx, cy) = rect.center();
        match anchor {
            Anchor::North => (cx, rect.y),
            Anchor::South => (cx, rect.bottom().saturating_sub(1)),
            Anchor::East => (rect.right().saturating_sub(1), cy),
            Anchor::West => (rect.x, cy),
        }
    }

    /// Render the connector onto the surface.
    pub fn paint(&self, _ctx: &PaintContext, surface: &mut Surface<'_>) {
        let (fx, fy) = Self::perimeter_point(&self.from, self.from_anchor);
        let (tx, ty) = Self::perimeter_point(&self.to, self.to_anchor);
        let h_ch = self.line_char();
        let v_ch = self.vert_char();

        // Determine exit direction from source anchor.
        let (edx, edy) = match self.from_anchor {
            Anchor::North => (0i32, -1i32),
            Anchor::South => (0, 1),
            Anchor::East => (1, 0),
            Anchor::West => (-1, 0),
        };

        // Start one cell outside the source perimeter and end at the
        // target perimeter so the arrowhead sits exactly on the edge.
        let sx = (fx as i32 + edx).max(0) as usize;
        let sy = (fy as i32 + edy).max(0) as usize;
        let ex = (tx as i32 - edx).max(0) as usize;
        let ey = ty;

        // Manhattan routing: vertical then horizontal then vertical.
        let mid_y = if sy == ey { sy } else { (sy + ey) / 2 };

        if sy != mid_y {
            let (y, len) = if sy < mid_y { (sy, mid_y - sy) } else { (mid_y, sy - mid_y) };
            surface.put_vertical(sx, y, len + 1, v_ch, Layer::Connector);
        }
        if sx != ex {
            let (x, len) = if sx < ex { (sx, ex - sx) } else { (ex, sx - ex) };
            surface.put_horizontal(x, mid_y, len + 1, h_ch, Layer::Connector);
        }
        if mid_y != ey {
            let (y, len) = if mid_y < ey { (mid_y, ey - mid_y) } else { (ey, mid_y - ey) };
            surface.put_vertical(ex, y, len + 1, v_ch, Layer::Connector);
        }

        // Arrowhead at target perimeter.
        let dx = if tx > fx {
            1
        } else if tx < fx {
            -1
        } else {
            0
        };
        let dy = if ty > fy {
            1
        } else if ty < fy {
            -1
        } else {
            0
        };
        let arrow = self.arrow_char(dx, dy);
        surface.put_layered(tx, ty, arrow, Layer::ConnectorEnd);

        // Label placement.
        if let Some(label) = &self.label {
            if sx == ex {
                // Purely vertical connector: place label to the right.
                let label_x = ex + 1;
                let label_y = mid_y;
                surface.put_str_layered(label_x, label_y, label, Layer::Label);
            } else {
                // Place the label just below the horizontal segment, biased
                // toward the target side so it reads as belonging to the
                // arrow that points at the target.
                let label_x = ((fx + tx) / 2).saturating_sub(label.len() / 2);
                let label_y = mid_y + 1;
                surface.put_str_layered(label_x, label_y, label, Layer::Label);
            }
        }
    }
}
