//! Bordered box/container diagrams with optional title and body text.
//!
//! Provides both a free function ([`draw_box`]) for simple usage and a
//! builder ([`BoxArt`]) for complex configurations.

use std::fmt;

use crate::canvas::Canvas;
use crate::error::{FigoError, Result};
use crate::style::{Alignment, BorderStyle, Charset, HAlign, Padding, VAlign};
use crate::text::{align_horizontal, word_wrap};

/// Draw a bordered box with optional title and content.
///
/// This is the simple free-function API. For full configuration use `BoxArt`.
pub fn draw_box(
    title: Option<&str>,
    content: Option<&str>,
    width: usize,
    charset: Charset,
    border: BorderStyle,
) -> Result<String> {
    BoxArt::new(width, charset).title(title).content(content).border(border).build()
}

/// Builder for drawing bordered boxes.
pub struct BoxArt<'a> {
    width: usize,
    charset: Charset,
    title: Option<&'a str>,
    content: Option<&'a str>,
    border: BorderStyle,
    padding: Padding,
    align: Alignment,
    color: bool,
}

impl<'a> BoxArt<'a> {
    /// Create a new box builder with the given width and charset.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self {
            width,
            charset,
            title: None,
            content: None,
            border: BorderStyle::Single,
            padding: Padding::default(),
            align: Alignment::TOP_LEFT,
            color: false,
        }
    }

    /// Set the title (placed in the top border).
    pub fn title(mut self, title: Option<&'a str>) -> Self {
        self.title = title;
        self
    }

    /// Set the content text (auto-wrapped to fit).
    pub fn content(mut self, content: Option<&'a str>) -> Self {
        self.content = content;
        self
    }

    /// Set the border style.
    pub fn border(mut self, style: BorderStyle) -> Self {
        self.border = style;
        self
    }

    /// Set padding inside the box.
    pub fn padding(mut self, horizontal: usize, vertical: usize) -> Self {
        self.padding = Padding { horizontal, vertical };
        self
    }

    /// Set text alignment inside the box.
    pub fn align(mut self, horizontal: HAlign, vertical: VAlign) -> Self {
        self.align = Alignment { horizontal, vertical };
        self
    }

    /// Enable or disable ANSI color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render the box and return it as a `String`.
    ///
    /// This is the primary rendering method. For `Display`-based output,
    /// format the builder directly (e.g., `println!("{box_art}")`).
    pub fn build(&self) -> Result<String> {
        if self.width < 4 {
            return Err(FigoError::InvalidDimensions(format!(
                "width must be at least 4, got {}",
                self.width
            )));
        }

        let glyphs = self.border.glyphs(self.charset);
        let inner_width = self.width.saturating_sub(2 + self.padding.horizontal * 2);
        if inner_width == 0 {
            return Err(FigoError::InvalidDimensions("width too small for padding".into()));
        }

        // Wrap the content text
        let content_lines: Vec<String> = match self.content {
            Some(text) if !text.is_empty() => word_wrap(text, inner_width),
            _ => Vec::new(),
        };

        // Calculate height
        let content_height = content_lines.len();
        let inner_height = self.padding.vertical * 2 + content_height;
        let total_height = (inner_height + 2).max(3); // minimum 3 rows for visible borders

        let mut canvas = Canvas::new(self.width, total_height);

        // Draw outer border
        canvas.draw_rect(0, 0, self.width, total_height, &glyphs)?;

        // Draw title in the top border if given
        if let Some(title) = self.title {
            if !title.is_empty() && self.width > 2 {
                let max_title = self.width.saturating_sub(4);
                let display: String = title.chars().take(max_title).collect();
                // Place title left-aligned after the top-left corner
                let start = 2;
                canvas.put_str(start, 0, &format!(" {display} "));
            }
        }

        // Vertical alignment for content
        let content_start_y = match self.align.vertical {
            VAlign::Top => 1 + self.padding.vertical,
            VAlign::Middle => 1 + (inner_height.saturating_sub(content_height)) / 2,
            VAlign::Bottom => {
                1 + inner_height.saturating_sub(self.padding.vertical + content_height)
            }
        };

        // Draw content lines
        let aligned = align_horizontal(&content_lines, inner_width, self.align.horizontal);
        for (i, line) in aligned.iter().enumerate() {
            let x = 1 + self.padding.horizontal;
            let y = content_start_y + i;
            canvas.put_str(x, y, line);
        }

        Ok(canvas.render(self.color))
    }
    /// Render the box and return it as a `String`.
    ///
    /// Alias for [`build`](Self::build).
    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for BoxArt<'_> {
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
    fn test_empty_box() {
        let out = draw_box(None, None, 20, Charset::Ascii, BorderStyle::Single).unwrap();
        assert!(out.contains('+'));
        assert!(out.contains('-'));
        assert!(out.contains('|'));
    }

    #[test]
    fn test_box_with_title() {
        let out = draw_box(Some("Test"), Some("Hello"), 30, Charset::Unicode, BorderStyle::Single)
            .unwrap();
        assert!(out.contains("Test"));
        assert!(out.contains("Hello"));
    }

    #[test]
    fn test_box_with_content_wrapping() {
        let out = draw_box(
            None,
            Some("This is a long line that should wrap"),
            20,
            Charset::Ascii,
            BorderStyle::Single,
        )
        .unwrap();
        // Should produce more than just the border lines (3+ lines)
        assert!(out.lines().count() > 3);
    }

    #[test]
    fn test_width_too_small() {
        assert!(draw_box(None, None, 3, Charset::Ascii, BorderStyle::Single).is_err());
    }

    #[test]
    fn test_rounded_border() {
        let out = draw_box(None, None, 10, Charset::Unicode, BorderStyle::Rounded).unwrap();
        assert!(out.contains('╭'));
    }

    #[test]
    fn test_double_border() {
        let out = draw_box(None, None, 10, Charset::Unicode, BorderStyle::Double).unwrap();
        assert!(out.contains('╔'));
    }

    // -- Snapshot tests -----------------------------------------------------

    #[test]
    fn snapshot_empty_box_ascii() {
        let out = draw_box(None, None, 20, Charset::Ascii, BorderStyle::Single).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_with_title_unicode() {
        let out = BoxArt::new(30, Charset::Unicode)
            .title(Some("Overview"))
            .content(Some("This is a sample box with a title."))
            .border(BorderStyle::Single)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_wrapped_content() {
        let out = BoxArt::new(24, Charset::Ascii)
            .title(Some("Note"))
            .content(Some(
                "This is a long line that should wrap onto multiple lines inside the box.",
            ))
            .border(BorderStyle::Single)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_double_border_unicode() {
        let out = BoxArt::new(20, Charset::Unicode)
            .content(Some("Double border"))
            .border(BorderStyle::Double)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_rounded_with_padding() {
        let out = BoxArt::new(30, Charset::Unicode)
            .content(Some("Centered text with padding"))
            .border(BorderStyle::Rounded)
            .padding(3, 1)
            .align(HAlign::Center, VAlign::Middle)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_dashed_ascii() {
        let out = BoxArt::new(16, Charset::Ascii)
            .content(Some("Dashed"))
            .border(BorderStyle::Dashed)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_box_bold_unicode() {
        let out = BoxArt::new(22, Charset::Unicode)
            .title(Some("Bold"))
            .content(Some("Bold border style"))
            .border(BorderStyle::Bold)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }
}
