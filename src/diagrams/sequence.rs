//! Sequence diagrams with participants and messages.
//!
//! Each participant occupies a "lane" of width `lane_width` cells. The
//! participant's box is inset by `LANE_GAP_HALF` cells on each side, so
//! adjacent boxes display a `LANE_GAP` (2-cell) horizontal gap regardless of
//! how many participants are stacked.
//!
//! Messages are routed through [`crate::layout::Connector`] with a one-cell
//! invisible rect at each lifeline column. Connector's z-order puts the
//! arrowhead at [`Layer::ConnectorEnd`] (sits cleanly outside the lifeline)
//! and the label at [`Layer::Label`] which never overlaps a lifeline column
//! because the label is clamped to the gap between the two endpoints.

use std::collections::HashMap;
use std::fmt;

use crate::canvas::{Canvas, Layer};
use crate::error::{FigoError, Result};
use crate::layout::{Anchor, Connector, Rect};
use crate::style::{BorderStyle, Charset, LineStyle};

/// Horizontal gap (in cells) between adjacent participant boxes.
const LANE_GAP: usize = 2;

/// One-sided inset within a lane (cells reserved on the left of every box).
const LANE_GAP_HALF: usize = LANE_GAP / 2;

/// A message in a sequence diagram.
#[derive(Debug, Clone)]
pub struct SequenceMessage {
    /// The sending participant's name.
    pub from: String,
    /// The receiving participant's name.
    pub to: String,
    /// The label displayed on the message arrow.
    pub label: String,
}

/// Builder for sequence diagrams.
pub struct SequenceDiagram<'a> {
    width: usize,
    charset: Charset,
    participants: Vec<&'a str>,
    messages: Vec<SequenceMessage>,
    color: bool,
}

impl<'a> SequenceDiagram<'a> {
    /// Create a new sequence diagram builder.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self { width, charset, participants: Vec::new(), messages: Vec::new(), color: false }
    }

    /// Add a participant.
    pub fn add_participant(mut self, name: &'a str) -> Self {
        self.participants.push(name);
        self
    }

    /// Add a message between participants.
    pub fn add_message(mut self, from: &str, to: &str, label: &str) -> Self {
        self.messages.push(SequenceMessage {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
        });
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render and return as a `String`.
    pub fn build(&self) -> Result<String> {
        if self.participants.is_empty() {
            return Err(FigoError::MissingFields("no participants specified".into()));
        }
        if self.messages.is_empty() {
            return Err(FigoError::MissingFields("no messages specified".into()));
        }

        let rounded = BorderStyle::Rounded.glyphs(self.charset);
        let v_ch = rounded.vertical;
        let tee_up = rounded.tee_up;

        let n = self.participants.len();
        let max_name_len = self.participants.iter().map(|p| p.chars().count()).max().unwrap_or(5);

        // Box width = lane_width - LANE_GAP so adjacent boxes are separated
        // by a 2-cell horizontal gap. Lane width is the per-participant
        // allotment; the box is centered horizontally with 1 cell padding
        // on each side.
        let minimum_box_w = max_name_len + 4;
        let minimum_lane_w = minimum_box_w + LANE_GAP;
        let ideal_lane_w = self.width / n.max(1);
        let lane_width = ideal_lane_w.max(minimum_lane_w).min(40);
        let box_width = lane_width - LANE_GAP;
        let actual_width = (lane_width * n).max(self.width);

        if box_width < 6 {
            return Err(FigoError::InvalidDimensions(
                "width too small for participant names".into(),
            ));
        }

        // Vertical layout:
        //   row 0                header top border
        //   row 1                header name row
        //   row 2                header bottom border + tee-junction into lifeline
        //   rows 3..total_height lifelines (Layer::Connector) interleaved with messages
        let header_height: usize = 3;
        let msg_spacing: usize = 3;
        let msg_rows = self.messages.len() * msg_spacing;
        let total_height = header_height + msg_rows + 1;

        let mut canvas = Canvas::new(actual_width, total_height);

        let name_to_idx: HashMap<&str, usize> =
            self.participants.iter().enumerate().map(|(i, &p)| (p, i)).collect();

        // Compute lifeline x as the exact center column of each header box
        // so the vertical lifeline visually anchors to the box regardless of
        // box width parity.
        let lifeline_x_for =
            |i: usize| -> usize { i * lane_width + LANE_GAP_HALF + (box_width - 1) / 2 };

        // Paint pass 1: lifelines at Layer::Connector (low) drawn FIRST so
        // they extend from the header bottom down to the canvas bottom.
        // The header box drawn next will cleanly cover the lifeline start.
        let lifeline_start = header_height;
        let lifeline_end = total_height.saturating_sub(1);
        for i in 0..n {
            let lifeline_x = lifeline_x_for(i);
            canvas.put_vertical_layered(
                lifeline_x,
                lifeline_start,
                lifeline_end.saturating_sub(lifeline_start) + 1,
                v_ch,
                Layer::Connector,
            );
        }

        // Paint pass 2: header boxes at Layer::NodeBorder, names at
        // Layer::NodeContent, and the tee-junction where lifeline meets box.
        for (i, name) in self.participants.iter().enumerate() {
            let hx = i * lane_width + LANE_GAP_HALF;
            canvas.draw_rect(hx, 0, box_width, header_height, &rounded)?;
            let name_x = hx + (box_width.saturating_sub(name.chars().count())) / 2;
            canvas.put_str_layered(name_x, 1, name, Layer::NodeContent, None);
            // Tee-junction glyph anchors the lifeline visually to the header.
            let lifeline_x = lifeline_x_for(i);
            canvas.put_layered(lifeline_x, header_height - 1, tee_up, Layer::NodeBorder, None);
        }

        // Paint pass 3: messages rendered through Connector. Each message
        // sees TWO 1×1 invisible rects at the lifeline columns; the
        // straight horizontal path is the arrow body and the arrowhead sits
        // one cell outside the target lifeline (at Layer::ConnectorEnd).
        for (mi, msg) in self.messages.iter().enumerate() {
            let from_idx = name_to_idx.get(msg.from.as_str()).copied().unwrap_or(0);
            let to_idx = name_to_idx.get(msg.to.as_str()).copied().unwrap_or(0);
            let from_x = lifeline_x_for(from_idx);
            let to_x = lifeline_x_for(to_idx);
            let arrow_y = header_height + 1 + mi * msg_spacing;
            let label_y = arrow_y.saturating_sub(1);

            if from_x == to_x {
                // Self-message: small loop to the right of the lifeline.
                Self::draw_self_message(&mut canvas, from_x, arrow_y, v_ch, &msg.label);
            } else {
                let (left_x, right_x, left_to_right) =
                    if from_x < to_x { (from_x, to_x, true) } else { (to_x, from_x, false) };

                // 1×1 invisible endpoint rects on the arrow row. Connector
                // computes a single horizontal segment between them.
                let source_rect = Rect::new(from_x, arrow_y, 1, 1);
                let target_rect = Rect::new(to_x, arrow_y, 1, 1);
                let (src_anchor, tgt_anchor, arrow_glyph) = if left_to_right {
                    (Anchor::East, Anchor::West, Self::east_glyph(self.charset))
                } else {
                    (Anchor::West, Anchor::East, Self::west_glyph(self.charset))
                };

                let mut c = Connector::new(
                    source_rect,
                    target_rect,
                    src_anchor,
                    tgt_anchor,
                    LineStyle::Simple,
                    self.charset,
                );
                c.arrow_head = arrow_glyph;
                c.render(&mut canvas);

                // Activation emphasis: draw a single activation cell on each
                // lifeline at the arrow row. This sits at Layer::Connector,
                // same level as the lifeline, so visual continuity is kept.
                canvas.put_layered(from_x, arrow_y, v_ch, Layer::Connector, None);
                canvas.put_layered(to_x, arrow_y, v_ch, Layer::Connector, None);

                // Label clamped to the gap between the two lifelines so it
                // never overlaps a lifeline column. One cell of margin on
                // each side keeps the label flush within the empty space.
                let inner_left = left_x + 1;
                let inner_right = right_x.saturating_sub(1);
                let label_w = msg.label.chars().count();
                let inner_w = inner_right.saturating_sub(inner_left) + 1;
                let label_x = if label_w <= inner_w {
                    inner_left + (inner_w - label_w) / 2
                } else {
                    inner_left
                };
                canvas.put_str_layered(label_x, label_y, &msg.label, Layer::Label, None);
            }
        }

        // Repair connector junctions so corners and crossings use proper
        // Unicode box-drawing glyphs.
        canvas.repair_connector_junctions(LineStyle::Simple, self.charset);

        Ok(canvas.render(self.color))
    }

    /// Render and return as a `String`. Equivalent to [`Self::build`].
    pub fn render(&self) -> Result<String> {
        self.build()
    }

    /// Draw a 2-row self-message loop to the right of the lifeline.
    fn draw_self_message(
        canvas: &mut Canvas,
        from_x: usize,
        arrow_y: usize,
        v_ch: char,
        label: &str,
    ) {
        let corner_top_right = '┐';
        let corner_bot_right = '┘';
        let h_ch = '─';
        let loop_top_x = from_x + 2;
        let loop_bot_x = from_x + 2;
        canvas.put_layered(from_x, arrow_y, h_ch, Layer::Connector, None);
        canvas.put_layered(from_x + 1, arrow_y, corner_top_right, Layer::Connector, None);
        canvas.put_vertical_layered(loop_top_x, arrow_y + 1, 2, v_ch, Layer::Connector);
        canvas.put_layered(loop_bot_x, arrow_y + 2, corner_bot_right, Layer::Connector, None);
        canvas.put_layered(from_x, arrow_y + 2, '<', Layer::ConnectorEnd, None);
        canvas.put_horizontal_layered(from_x + 1, arrow_y + 2, 2, h_ch, Layer::Connector);
        // Label to the right of the loop, on the same row as the loop top.
        canvas.put_str_layered(loop_top_x + 2, arrow_y, label, Layer::Label, None);
    }

    /// Eastward arrowhead glyph for the given charset.
    fn east_glyph(charset: Charset) -> char {
        match charset {
            Charset::Unicode => '▶',
            Charset::Ascii => '>',
        }
    }

    /// Westward arrowhead glyph for the given charset.
    fn west_glyph(charset: Charset) -> char {
        match charset {
            Charset::Unicode => '◀',
            Charset::Ascii => '<',
        }
    }
}
impl fmt::Display for SequenceDiagram<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        let sd = SequenceDiagram::new(100, Charset::Unicode)
            .add_participant("Client")
            .add_participant("Server")
            .add_message("Client", "Server", "GET /api");
        let out = sd.build().unwrap();
        assert!(out.contains("Client"));
        assert!(out.contains("Server"));
        assert!(out.contains("GET /api"));
    }

    #[test]
    fn test_no_participants() {
        assert!(SequenceDiagram::new(80, Charset::Unicode).build().is_err());
    }

    #[test]
    fn test_no_messages() {
        assert!(SequenceDiagram::new(80, Charset::Unicode).add_participant("A").build().is_err());
    }

    #[test]
    fn test_lane_gap_between_adjacent_boxes() {
        // Two participants of equal name length — verify there is a 2-cell
        // gap between the two rendered boxes (count consecutive spaces).
        let sd = SequenceDiagram::new(40, Charset::Unicode)
            .add_participant("A")
            .add_participant("B")
            .add_message("A", "B", "m");
        let out = sd.build().unwrap();
        let first_line = out.lines().next().unwrap();
        // The top row contains both box top borders; between the two `╭`
        // characters there must be at least one space (border end to start),
        // and the middle row has the participant names. Check the row with
        // the *top border* (`╭...─...╮╭...─...╮`) contains a 2-cell gap.
        let chars: Vec<char> = first_line.chars().collect();
        let first_close = chars.iter().position(|&c| c == '╮').expect("first box right border");
        let second_open = chars
            .iter()
            .enumerate()
            .skip(first_close + 1)
            .find(|&(_, &c)| c == '╭')
            .map(|(i, _)| i)
            .expect("second box left border");
        let gap = second_open.saturating_sub(first_close + 1);
        // Box left-border starts one cell into the lane (LANE_GAP_HALF = 1),
        // so the actual boxed border-to-border gap is `LANE_GAP`.
        assert!(gap >= LANE_GAP, "expected ≥ {LANE_GAP}-cell gap, got {gap}: {first_line:?}");
    }
    #[test]
    fn test_arrow_does_not_pierce_lifeline() {
        // Regression: the arrowhead is the corridor cell that visually
        // touches the target lifeline. A incorrect router would either
        // (a) leave a body glyph on top of the lifeline or (b) fail to
        // reach the lifeline at all. Both directions and both charsets
        // are exercised:
        //   * A → B : arrowhead (▶ / >) is the right corridor cell, the
        //              target lifeline must be IMMEDIATELY after it.
        //   * B → A : arrowhead (◀ / <) is the left corridor cell, the
        //              target lifeline must be IMMEDIATELY before it.
        for charset in [Charset::Unicode, Charset::Ascii] {
            let (lifeline, gt_arrow, lt_arrow) = match charset {
                Charset::Unicode => ('│', '▶', '◀'),
                Charset::Ascii => ('|', '>', '<'),
            };

            // A → B: arrow flows rightward; arrowhead sits in the right
            // corridor cell and is followed by the target lifeline.
            let out_a_to_b = SequenceDiagram::new(60, charset)
                .add_participant("A")
                .add_participant("B")
                .add_message("A", "B", "hi")
                .build()
                .unwrap();
            assert_arrow_touches_target_lifeline(
                &out_a_to_b,
                gt_arrow,
                lifeline,
                /*reversed=*/ false,
            );

            // B → A: arrow flows leftward; arrowhead sits in the left
            // corridor cell and is preceded by the target lifeline.
            let out_b_to_a = SequenceDiagram::new(60, charset)
                .add_participant("A")
                .add_participant("B")
                .add_message("B", "A", "hi")
                .build()
                .unwrap();
            assert_arrow_touches_target_lifeline(
                &out_b_to_a,
                lt_arrow,
                lifeline,
                /*reversed=*/ true,
            );
        }
    }

    /// Asserts the arrowhead glyph in the first message's arrow row is
    /// exactly adjacent to the target lifeline glyph. The corridor flow
    /// direction flips the perspective: a left-to-right arrow places
    /// the arrowhead in the right corridor cell with the target
    /// lifeline cell next to it on the right; a right-to-left arrow
    /// places the arrowhead cell against the target lifeline on the
    /// left.
    fn assert_arrow_touches_target_lifeline(
        rendered: &str,
        arrow_glyph: char,
        target_lifeline: char,
        reversed: bool,
    ) {
        // Arrow row index = header_height (3) + 1 = 4 for the first message.
        let arrow_row =
            rendered.lines().nth(4).unwrap_or_else(|| panic!("missing row 4: {rendered:?}"));
        assert!(
            arrow_row.contains(arrow_glyph),
            "arrowhead {arrow_glyph:?} missing from {arrow_row:?}",
        );
        let pos = arrow_row.find(arrow_glyph).unwrap();
        if reversed {
            let prev = arrow_row[..pos].chars().last();
            assert_eq!(
                prev,
                Some(target_lifeline),
                "expected {target_lifeline:?} immediately before {arrow_glyph:?}; \
                 row={arrow_row:?}",
            );
        } else {
            let next = arrow_row[pos + arrow_glyph.len_utf8()..].chars().next();
            assert_eq!(
                next,
                Some(target_lifeline),
                "expected {target_lifeline:?} immediately after {arrow_glyph:?}; \
                 row={arrow_row:?}",
            );
        }
    }
}
