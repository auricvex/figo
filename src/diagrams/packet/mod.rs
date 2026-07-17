//! IETF/RFC-style packet header diagrams.
//!
//! Adjacent fields share walls via T-junctions (`┬`/`┴`, `+` in ASCII),
//! consecutive 32-bit words SHARE their horizontal border so the
//! diagram has no doubled lines, and the bit-position scale marks every
//! 4 bits plus the final 31.

mod render;

use std::fmt;

use crate::canvas::Canvas;
use crate::error::{FigoError, Result};
use crate::style::{BorderStyle, Charset};

use render::{
    draw_border_row, draw_bottom_row, draw_middle_row, draw_scale, split_per_word, walls_for_spans,
};

/// A single field in a packet header.
#[derive(Debug, Clone)]
pub struct PacketField {
    pub name: String,
    pub bits: usize,
}

/// Draw a packet header diagram.
pub fn draw_packet(fields: &[PacketField], width: usize, charset: Charset) -> Result<String> {
    PacketDiagram::new(width, charset).fields(fields).build()
}

/// Builder for packet header diagrams.
pub struct PacketDiagram {
    width: usize,
    charset: Charset,
    fields: Vec<PacketField>,
    color: bool,
}

/// Computed canvas layout shared by all render helpers.
#[derive(Clone, Copy)]
pub(crate) struct Layout {
    pub(crate) packet_left: usize,
    pub(crate) packet_right: usize,
    pub(crate) col_per_bit: usize,
}

impl PacketDiagram {
    /// Create a new packet diagram builder.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self { width, charset, fields: Vec::new(), color: false }
    }

    /// Set all fields at once (replaces any previously added fields).
    pub fn fields(mut self, fields: &[PacketField]) -> Self {
        self.fields = fields.to_vec();
        self
    }

    /// Append a single field.
    pub fn add_field(mut self, field: PacketField) -> Self {
        self.fields.push(field);
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render and return as a `String`.
    pub fn build(&self) -> Result<String> {
        let layout = validate(self.width, &self.fields)?;
        let glyphs = BorderStyle::Single.glyphs(self.charset);
        let total_bits: usize = self.fields.iter().map(|f| f.bits).sum();
        let words = total_bits.div_ceil(32);
        let mut canvas = Canvas::new(self.width, 2 + 2 * words);

        draw_scale(&mut canvas, layout);
        let word_spans = split_per_word(&self.fields);
        let word_walls: Vec<_> = word_spans.iter().map(|s| walls_for_spans(layout, s)).collect();

        for word_idx in 0..words {
            let (Some(spans_below), Some(walls_below)) =
                (word_spans.get(word_idx), word_walls.get(word_idx))
            else {
                continue;
            };
            let walls_above = word_idx.checked_sub(1).and_then(|i| word_walls.get(i));
            let y_border = 1 + 2 * word_idx;
            let y_middle = y_border + 1;
            draw_border_row(&mut canvas, y_border, layout, walls_below, walls_above, &glyphs);
            draw_middle_row(&mut canvas, y_middle, layout, spans_below, &glyphs);
            if word_idx + 1 == words {
                draw_bottom_row(&mut canvas, y_border + 2, layout, walls_below, &glyphs);
            }
        }
        Ok(canvas.render(self.color))
    }

    /// Convenience: `build()`.
    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for PacketDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

fn validate(width: usize, fields: &[PacketField]) -> Result<Layout> {
    if fields.is_empty() {
        return Err(FigoError::MissingFields("no fields specified".into()));
    }
    for field in fields {
        if field.bits == 0 {
            return Err(FigoError::InvalidInput(format!("field \"{}\" has 0 bits", field.name)));
        }
    }
    let inner_w = width.saturating_sub(2);
    let col_per_bit = inner_w / 32;
    if col_per_bit == 0 {
        return Err(FigoError::InvalidDimensions(format!(
            "width too small for packet diagram (got width {width}, need at least 36)"
        )));
    }
    let bit_w = 32 * col_per_bit;
    let row_w = bit_w + 2;
    let left_pad = inner_w.saturating_sub(row_w) / 2;
    Ok(Layout { packet_left: 1 + left_pad, packet_right: 1 + left_pad + bit_w + 1, col_per_bit })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Charset;

    #[test]
    fn test_ipv4_header_ascii() {
        let fields = vec![
            PacketField { name: "Version".into(), bits: 4 },
            PacketField { name: "IHL".into(), bits: 4 },
            PacketField { name: "ToS".into(), bits: 8 },
            PacketField { name: "Total Length".into(), bits: 16 },
        ];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        assert!(out.contains("Version"));
        assert!(out.contains("Total Length"));
        assert!(out.contains('0'));
        assert!(!out.contains('┌'), "ascii must not use unicode glyphs");
        assert!(!out.contains('│'));
    }

    #[test]
    fn test_ipv4_header_unicode_uses_box_drawing() {
        let fields = vec![
            PacketField { name: "Version".into(), bits: 4 },
            PacketField { name: "IHL".into(), bits: 4 },
            PacketField { name: "ToS".into(), bits: 8 },
            PacketField { name: "Total Length".into(), bits: 16 },
        ];
        let out = draw_packet(&fields, 80, Charset::Unicode).unwrap();
        assert!(out.contains('┌'));
        assert!(out.contains('┐'));
        assert!(out.contains('└'));
        assert!(out.contains('┘'));
        assert!(out.contains('│'));
    }

    #[test]
    fn test_no_double_walls_between_adjacent_fields() {
        let fields = vec![
            PacketField { name: "Version".into(), bits: 4 },
            PacketField { name: "IHL".into(), bits: 4 },
            PacketField { name: "Total Length".into(), bits: 16 },
        ];
        let out = draw_packet(&fields, 80, Charset::Unicode).unwrap();
        assert!(!out.contains("┐┌"), "no double walls: {out:?}");
        assert!(!out.contains("┘┌"));
    }

    #[test]
    fn test_no_corner_then_space_anywhere() {
        let fields = vec![
            PacketField { name: "Version".into(), bits: 4 },
            PacketField { name: "IHL".into(), bits: 4 },
            PacketField { name: "ToS".into(), bits: 8 },
            PacketField { name: "Total Length".into(), bits: 16 },
        ];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        assert!(!out.contains("+ "), "no corner-then-space: {out:?}");
    }

    #[test]
    fn test_scale_includes_31_mark() {
        let fields = vec![PacketField { name: "V".into(), bits: 4 }];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        assert!(out.contains(" 31"), "scale label '31' must appear on row 0: {out:?}");
    }

    #[test]
    fn test_scale_labels_left_to_right() {
        let fields = vec![PacketField { name: "V".into(), bits: 4 }];
        let scale_row =
            draw_packet(&fields, 80, Charset::Ascii).unwrap().lines().next().unwrap().to_string();
        let order = ["0", "4", "8", "12", "16", "20", "24", "28", "31"];
        let mut last = 0;
        for mark in order {
            let pos =
                scale_row.find(mark).unwrap_or_else(|| panic!("missing {mark:?} in {scale_row:?}"));
            assert!(pos >= last, "{mark:?} must appear after previous: {scale_row:?}");
            last = pos + mark.len();
        }
    }

    #[test]
    fn test_multiword_field_renders_per_word() {
        let fields = vec![PacketField { name: "Address".into(), bits: 64 }];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        assert!(out.matches("Address").count() >= 2, "name must appear per word: {out:?}");
    }

    #[test]
    fn test_empty_fields_returns_error() {
        assert!(PacketDiagram::new(80, Charset::Ascii).build().is_err());
        assert!(PacketDiagram::new(80, Charset::Unicode).build().is_err());
    }

    #[test]
    fn test_width_too_small_returns_error() {
        let fields = vec![PacketField { name: "V".into(), bits: 4 }];
        let err = draw_packet(&fields, 10, Charset::Ascii).unwrap_err();
        assert!(matches!(err, FigoError::InvalidDimensions(_)));
    }

    #[test]
    fn test_zero_bits_field_rejected() {
        let fields = vec![
            PacketField { name: "Version".into(), bits: 4 },
            PacketField { name: "Bad".into(), bits: 0 },
        ];
        let err = draw_packet(&fields, 80, Charset::Ascii).unwrap_err();
        assert!(matches!(err, FigoError::InvalidInput(_)));
    }

    #[test]
    fn test_truncation_without_ellipsis() {
        let fields = vec![PacketField { name: "TypeOfService".into(), bits: 8 }];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        assert!(!out.contains("..."), "no ellipsis allowed: {out:?}");
    }

    #[test]
    fn test_bottom_row_ends_with_corner() {
        let fields = vec![PacketField { name: "Full".into(), bits: 32 }];
        let out = draw_packet(&fields, 80, Charset::Ascii).unwrap();
        let bottom = out.lines().next_back().unwrap().trim_end();
        assert!(bottom.ends_with('+'), "bottom row must end with +: {bottom:?}");
    }
}
