//! Standalone arrows, lines, and connectors.
//!
//! Provides both a free function ([`draw_arrow`]) for simple usage and a
//! builder ([`Arrow`]) for complex configurations.

use std::fmt;

use crate::canvas::Canvas;
use crate::error::Result;
use crate::style::{Charset, LineStyle};

/// Draw a standalone arrow.
///
/// Returns the rendered arrow as a `String`.
///
/// # Arguments
/// * `direction` — One of `"right"`, `"left"`, `"up"`, `"down"`, or `"bidirectional"`.
/// * `length` — The arrow length in characters (excluding arrowhead).
/// * `style` — Line style to use.
/// * `charset` — Character set mode.
/// * `label` — Optional label placed above the arrow.
pub fn draw_arrow(
    direction: &str,
    length: usize,
    style: LineStyle,
    charset: Charset,
    label: Option<&str>,
) -> Result<String> {
    let mut arrow = Arrow::new(direction, length, style, charset)?;
    if let Some(l) = label {
        arrow = arrow.label(l);
    }
    arrow.build()
}

/// Builder for drawing arrows.
pub struct Arrow {
    direction: String,
    length: usize,
    style: LineStyle,
    charset: Charset,
    label: Option<String>,
}

impl Arrow {
    /// Create a new arrow builder.
    pub fn new(direction: &str, length: usize, style: LineStyle, charset: Charset) -> Result<Self> {
        if !matches!(direction, "right" | "left" | "up" | "down" | "bidirectional") {
            return Err(crate::error::FigoError::InvalidDimensions(format!(
                "unknown direction '{direction}'"
            )));
        }
        Ok(Self { direction: direction.to_string(), length, style, charset, label: None })
    }

    /// Set an optional label for the arrow.
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Render and return the arrow as a `String`.
    ///
    /// This is the primary rendering method. For `Display`-based output,
    /// format the builder directly.
    pub fn build(&self) -> Result<String> {
        match self.direction.as_str() {
            "right" | "left" | "bidirectional" => self.draw_horizontal(),
            "up" | "down" => self.draw_vertical(),
            _ => Err(crate::error::FigoError::General("unreachable".into())),
        }
    }

    fn arrow_chars(&self) -> (&'static str, &'static str) {
        match (self.style, self.charset) {
            (LineStyle::Simple, Charset::Unicode) => ("─", "→"),
            (LineStyle::Simple, Charset::Ascii) => ("-", ">"),
            (LineStyle::Bold, Charset::Unicode) => ("━", "⇒"),
            (LineStyle::Bold, Charset::Ascii) => ("=", ">"),
            (LineStyle::BoxDrawing, Charset::Unicode) => ("─", "→"),
            (LineStyle::BoxDrawing, Charset::Ascii) => ("-", ">"),
        }
    }

    fn left_arrow_chars(&self) -> (&'static str, &'static str) {
        match (self.style, self.charset) {
            (LineStyle::Simple, Charset::Unicode) => ("─", "←"),
            (LineStyle::Simple, Charset::Ascii) => ("-", "<"),
            (LineStyle::Bold, Charset::Unicode) => ("━", "⇐"),
            (LineStyle::Bold, Charset::Ascii) => ("=", "<"),
            (LineStyle::BoxDrawing, Charset::Unicode) => ("─", "←"),
            (LineStyle::BoxDrawing, Charset::Ascii) => ("-", "<"),
        }
    }

    fn draw_horizontal(&self) -> Result<String> {
        let (line, right_head) = self.arrow_chars();
        let (_, left_head) = self.left_arrow_chars();

        let label = self.label.as_deref().unwrap_or("");
        let label_width = label.len();
        let height = if label.is_empty() { 1usize } else { 3usize };
        let left_head_width = left_head.chars().count();
        let right_head_width = right_head.chars().count();
        let total_width = if self.direction == "bidirectional" {
            self.length + left_head_width + right_head_width
        } else {
            self.length + right_head_width
        };
        let width = total_width.max(label_width);

        let mut canvas = Canvas::new(width, height);
        let arrow_y = if label.is_empty() { 0 } else { 1 };

        match self.direction.as_str() {
            "right" => {
                let body: String = line.repeat(self.length);
                canvas.put_str(0, arrow_y, &body);
                canvas.put_str(self.length, arrow_y, right_head);
            }
            "left" => {
                canvas.put_str(0, arrow_y, left_head);
                let body: String = line.repeat(self.length);
                // Use char count, not byte length, for the offset
                let head_width = left_head.chars().count();
                canvas.put_str(head_width, arrow_y, &body);
            }
            "bidirectional" => {
                canvas.put_str(0, arrow_y, left_head);
                let body: String = line.repeat(self.length);
                let head_width = left_head.chars().count();
                canvas.put_str(head_width, arrow_y, &body);
                canvas.put_str(head_width + self.length, arrow_y, right_head);
            }
            _ => unreachable!(),
        }

        if !label.is_empty() {
            let start_x = (total_width.saturating_sub(label_width)) / 2;
            canvas.put_str(start_x, 0, label);
        }

        Ok(canvas.render(false))
    }

    fn draw_vertical(&self) -> Result<String> {
        let (head, line) = if self.direction == "up" {
            match (self.style, self.charset) {
                (LineStyle::Simple, Charset::Unicode) => ("↑", "│"),
                (LineStyle::Simple, Charset::Ascii) => ("^", "|"),
                (LineStyle::Bold, Charset::Unicode) => ("⇑", "┃"),
                (LineStyle::Bold, Charset::Ascii) => ("^", "|"),
                (LineStyle::BoxDrawing, Charset::Unicode) => ("↑", "│"),
                (LineStyle::BoxDrawing, Charset::Ascii) => ("^", "|"),
            }
        } else {
            match (self.style, self.charset) {
                (LineStyle::Simple, Charset::Unicode) => ("↓", "│"),
                (LineStyle::Simple, Charset::Ascii) => ("v", "|"),
                (LineStyle::Bold, Charset::Unicode) => ("⇓", "┃"),
                (LineStyle::Bold, Charset::Ascii) => ("v", "|"),
                (LineStyle::BoxDrawing, Charset::Unicode) => ("↓", "│"),
                (LineStyle::BoxDrawing, Charset::Ascii) => ("v", "|"),
            }
        };

        let label = self.label.as_deref().unwrap_or("");
        let label_len = label.chars().count();
        // Center the arrow horizontally
        let width = label_len.max(1);
        let height = self.length + 1;

        let mut canvas = Canvas::new(width, height);
        if self.direction == "up" {
            canvas.put_str(0, 0, head);
            for i in 0..self.length {
                canvas.put_str(0, i + 1, line);
            }
        } else {
            for i in 0..self.length {
                canvas.put_str(0, i, line);
            }
            canvas.put_str(0, self.length, head);
        }

        // Label on the right
        if !label.is_empty() {
            canvas.put_str(1, 0, label);
        }

        Ok(canvas.render(false))
    }
    /// Render and return as a `String`.
    ///
    /// Alias for [`build`](Self::build).
    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for Arrow {
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
    fn test_right_arrow_ascii() {
        let out = draw_arrow("right", 5, LineStyle::Simple, Charset::Ascii, None).unwrap();
        assert_eq!(out.trim(), "----->");
    }

    #[test]
    fn test_left_arrow_unicode() {
        let out = draw_arrow("left", 3, LineStyle::Simple, Charset::Unicode, None).unwrap();
        assert_eq!(out.trim(), "←───");
    }

    #[test]
    fn test_with_label() {
        let out = draw_arrow("right", 5, LineStyle::Simple, Charset::Ascii, Some("flow")).unwrap();
        assert!(out.contains("flow"));
        assert!(out.contains("----->"));
    }

    #[test]
    fn test_bidirectional() {
        let out = draw_arrow("bidirectional", 3, LineStyle::Simple, Charset::Ascii, None).unwrap();
        assert_eq!(out.trim(), "<--->");
    }

    #[test]
    fn test_invalid_direction() {
        assert!(draw_arrow("north", 5, LineStyle::Simple, Charset::Ascii, None).is_err());
    }
}
