//! Rendering helpers for the packet module.
//!
//! These functions share a `Layout` value (from the parent module) and
//! translate per-word field spans onto a [`Canvas`]. They are intended
//! to be called only from [`super::PacketDiagram::build`].

use std::collections::HashSet;

use crate::canvas::{Canvas, Layer};
use crate::style::BorderGlyphs;

use super::Layout;

/// A field's portion within a single 32-bit word. A field may be split
/// across multiple word spans if its bit-width exceeds 32 bits.
#[derive(Debug, Clone)]
pub(super) struct WordSpan {
    pub(super) name: String,
    pub(super) bit_in_word: usize,
    pub(super) bits: usize,
}

/// Split fields into per-word spans, breaking fields that cross a 32-bit
/// boundary into one span per word they occupy.
pub(super) fn split_per_word(fields: &[super::PacketField]) -> Vec<Vec<WordSpan>> {
    let mut out: Vec<Vec<WordSpan>> = Vec::new();
    let mut bit_offset = 0usize;
    for field in fields {
        let mut remaining = field.bits;
        let mut local_offset = bit_offset;
        while remaining > 0 {
            let word_idx = local_offset / 32;
            let bit_in_word = local_offset % 32;
            let take = remaining.min(32 - bit_in_word);
            while out.len() <= word_idx {
                out.push(Vec::new());
            }
            out[word_idx].push(WordSpan { name: field.name.clone(), bit_in_word, bits: take });
            local_offset += take;
            remaining -= take;
        }
        bit_offset += field.bits;
    }
    out
}

/// Canvas column of the wall to the LEFT of bit `b`. For `b=0` this is
/// `packet_left` (the row's left wall); for `b>0` it's one column past
/// the previous bit's last cell.
pub(super) fn wall_left(layout: Layout, bit_in_word: usize) -> usize {
    layout.packet_left + bit_in_word * layout.col_per_bit + usize::from(bit_in_word > 0)
}

/// Canvas column of the wall to the RIGHT of the field's last bit.
pub(super) fn wall_right(layout: Layout, bit_in_word: usize, bits: usize) -> usize {
    layout.packet_left + (bit_in_word + bits) * layout.col_per_bit + 1
}

/// Vertical-wall column positions for a single word's field spans.
pub(super) fn walls_for_spans(layout: Layout, spans: &[WordSpan]) -> HashSet<usize> {
    let mut s = HashSet::new();
    for span in spans {
        s.insert(wall_left(layout, span.bit_in_word));
        s.insert(wall_right(layout, span.bit_in_word, span.bits));
    }
    s
}

/// Render the bit-position scale labels on row 0, right-aligned to each
/// bit position.
pub(super) fn draw_scale(canvas: &mut Canvas, layout: Layout) {
    for bit in [0usize, 4, 8, 12, 16, 20, 24, 28, 31] {
        let x = layout.packet_left + bit * layout.col_per_bit;
        let label = bit.to_string();
        let label_x = x.saturating_sub(label.chars().count().saturating_sub(1));
        canvas.put_str_layered(label_x, 0, &label, Layer::Grid, None);
    }
}

/// Render a horizontal border row that may sit between two consecutive
/// 32-bit words. The chosen glyph reflects wall direction and edge
/// position.
pub(super) fn draw_border_row(
    canvas: &mut Canvas,
    y: usize,
    layout: Layout,
    walls_below: &HashSet<usize>,
    walls_above: Option<&HashSet<usize>>,
    glyphs: &BorderGlyphs,
) {
    for x in layout.packet_left..=layout.packet_right {
        let up = walls_above.is_some_and(|s| s.contains(&x));
        let down = walls_below.contains(&x);
        let ch =
            pick_border_glyph(up, down, x == layout.packet_left, x == layout.packet_right, glyphs);
        canvas.put_layered(x, y, ch, Layer::NodeBorder, None);
    }
}

/// Render the final bottom border (after the last word's middle row).
pub(super) fn draw_bottom_row(
    canvas: &mut Canvas,
    y: usize,
    layout: Layout,
    walls: &HashSet<usize>,
    glyphs: &BorderGlyphs,
) {
    for x in layout.packet_left..=layout.packet_right {
        let ch = if x == layout.packet_left {
            glyphs.bottom_left
        } else if x == layout.packet_right {
            glyphs.bottom_right
        } else if walls.contains(&x) {
            glyphs.tee_up
        } else {
            glyphs.horizontal
        };
        canvas.put_layered(x, y, ch, Layer::NodeBorder, None);
    }
}

/// Render a middle row: vertical walls at each field boundary and the
/// centered field name (truncated without ellipsis if too long).
pub(super) fn draw_middle_row(
    canvas: &mut Canvas,
    y: usize,
    layout: Layout,
    spans: &[WordSpan],
    glyphs: &BorderGlyphs,
) {
    for span in spans {
        let fl = wall_left(layout, span.bit_in_word);
        let fr = wall_right(layout, span.bit_in_word, span.bits);
        canvas.put_layered(fl, y, glyphs.vertical, Layer::NodeBorder, None);
        canvas.put_layered(fr, y, glyphs.vertical, Layer::NodeBorder, None);
        write_centered_label(canvas, y, fl + 1, fr.saturating_sub(fl + 1), &span.name);
    }
}

/// Drop the field name centered into the interior columns.
pub(super) fn write_centered_label(
    canvas: &mut Canvas,
    y: usize,
    inner_left: usize,
    inner_w: usize,
    name: &str,
) {
    if inner_w == 0 {
        return;
    }
    let total = name.chars().count();
    let (label, chars) = if total <= inner_w {
        (name.to_string(), total)
    } else {
        (name.chars().take(inner_w).collect(), inner_w)
    };
    let pad = (inner_w - chars) / 2;
    canvas.put_str_layered(inner_left + pad, y, &label, Layer::NodeContent, None);
}

/// Pick the right border glyph given wall directions and edge position.
pub(super) fn pick_border_glyph(
    up: bool,
    down: bool,
    is_left: bool,
    is_right: bool,
    g: &BorderGlyphs,
) -> char {
    match (up, down) {
        (true, true) => g.cross,
        (false, true) => {
            if is_left {
                g.top_left
            } else if is_right {
                g.top_right
            } else {
                g.tee_down
            }
        }
        (true, false) => {
            if is_left {
                g.bottom_left
            } else if is_right {
                g.bottom_right
            } else {
                g.tee_up
            }
        }
        (false, false) => g.horizontal,
    }
}
