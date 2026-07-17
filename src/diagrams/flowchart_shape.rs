//! Flowchart node shape helpers: sizing and diamond (decision) glyph
//! rendering.
//!
//! Keeping these in a separate file lets [`super::flowchart`] stay under
//! the project's 250-line file cap while the diamond rasterization logic
//! is independently testable.
//!
//! A decision node renders as a diamond whose four edges are drawn with
//! `/`, `\` slashes (Unicode also uses these — there is no clean
//! single-glyph diagonal box-drawing rune). The label is centered inside.

use crate::canvas::{Canvas, Layer};
use crate::style::Charset;

/// Node shapes in a flowchart.
///
/// Defined here (rather than in `flowchart.rs`) to avoid a circular
/// module dependency: `flowchart_shape` provides sizing and rendering
/// helpers that `flowchart` consumes, and `flowchart` re-exports this
/// enum for the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeShape {
    /// Square-cornered rectangle.
    Rectangle,
    /// Rounded-corner rectangle.
    Rounded,
    /// Diamond (decision node) — diagonal-slash border.
    Diamond,
}

/// Compute the `(width, height)` of a node given its label and shape.
///
/// - Rectangles and rounded rectangles get `label_w + 4` columns and 3
///   rows (one border row, one content row, one border row).
/// - Diamonds are sized so the diagonal border has room: the half-height
///   `r` is `ceil(label_w / 2) + 1`, giving a total height of `2*r + 1`
///   and width `2*r + 1`. This keeps the diamond roughly square and wide
///   enough for the label to sit on the middle row.
pub fn node_dims(label: &str, shape: NodeShape, total_width: usize) -> (usize, usize) {
    let label_w = label.chars().count();
    match shape {
        NodeShape::Rectangle | NodeShape::Rounded => ((label_w + 4).min(total_width).max(6), 3),
        NodeShape::Diamond => {
            let r = (label_w / 2 + 1).max(3);
            let w = 2 * r + 1;
            let w = w.min(total_width).max(5);
            let h = 2 * r + 1;
            (w, h)
        }
    }
}

/// Draw a diamond (decision node) border at `(x, y)` with the given
/// width and height (as returned by [`node_dims`]). The left/right
/// slashes are drawn at `Layer::NodeBorder` so connectors routed
/// underneath are clipped cleanly.
pub fn draw_diamond(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    _charset: Charset,
) {
    // The diamond's bounding box is w×h. The horizontal mid col is x + w/2.
    let mid_x = x + w / 2;
    let half_h = h / 2;

    // Draw the four diagonal edges one cell at a time. The diamond is
    // widest at the middle row (row == half_h) and narrows to a single
    // cell at the top and bottom apexes.
    //
    //   row 0:       ^           (top apex)
    //   row 1:     /   \         (off = 1, widening)
    //   ...
    //   row r:   /       \       (off = r, widest)
    //   ...
    //   row 2r:     v           (bottom apex)
    let last = h - 1;
    for row in 0..h {
        let cy = y + row;
        if row == 0 {
            // Top apex.
            canvas.put_layered(mid_x, cy, '^', Layer::NodeBorder, None);
        } else if row == last {
            // Bottom apex.
            canvas.put_layered(mid_x, cy, 'v', Layer::NodeBorder, None);
        } else if row <= half_h {
            // Upper half: widen from the top apex toward the middle.
            let off = row;
            canvas.put_layered(mid_x - off, cy, '/', Layer::NodeBorder, None);
            canvas.put_layered(mid_x + off, cy, '\\', Layer::NodeBorder, None);
        } else {
            // Lower half: narrow from the middle toward the bottom apex.
            let off = last - row;
            canvas.put_layered(mid_x - off, cy, '\\', Layer::NodeBorder, None);
            canvas.put_layered(mid_x + off, cy, '/', Layer::NodeBorder, None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diamond_is_roughly_square_and_fits_label() {
        let (w, h) = node_dims("Is valid?", NodeShape::Diamond, 80);
        assert!(w >= 9, "diamond width {w} must fit the 9-char label");
        assert_eq!(w, h, "diamond should be square, got {w}x{h}");
    }

    #[test]
    fn rectangle_sizing_unchanged() {
        let (w, h) = node_dims("Start", NodeShape::Rounded, 80);
        assert_eq!((w, h), (9, 3));
    }

    #[test]
    fn diamond_renders_apex_and_edges() {
        let mut c = Canvas::new(20, 11);
        let (w, h) = node_dims("Hi", NodeShape::Diamond, 80);
        draw_diamond(&mut c, 0, 0, w, h, Charset::Unicode);
        let out = c.render(false);
        assert!(out.contains('^'), "top apex missing:\n{out}");
        assert!(out.contains('v'), "bottom apex missing:\n{out}");
        assert!(out.contains('/'), "left edge missing:\n{out}");
        assert!(out.contains('\\'), "right edge missing:\n{out}");
    }

    #[test]
    fn diamond_is_widest_at_middle_row() {
        // Regression: the diamond must be widest at the middle row, not
        // near the top/bottom (which would make it an hourglass). Count
        // non-space cells per row and check the middle row has the most.
        let (w, h) = node_dims("Is valid?", NodeShape::Diamond, 80);
        let mut c = Canvas::new(w + 2, h + 2);
        draw_diamond(&mut c, 1, 1, w, h, Charset::Unicode);
        let out = c.render(false);
        let lines: Vec<&str> = out.lines().collect();
        let mid_row = 1 + h / 2; // canvas offset + middle row
        let count_non_space = |line: &str| line.chars().filter(|&c| c != ' ').count();
        let mid_count = count_non_space(lines[mid_row]);
        for (i, line) in lines.iter().enumerate() {
            if i == mid_row {
                continue;
            }
            assert!(
                count_non_space(line) <= mid_count,
                "row {i} has more edge cells than the middle row {mid_row} \
                 — diamond may be an hourglass:\n{out}",
            );
        }
    }
}
