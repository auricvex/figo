//! Text processing helpers: word wrapping and alignment.

pub mod figlet;

use unicode_width::UnicodeWidthStr;

/// Wrap `text` so that each line fits within `max_width` characters (measured
/// in display width, not raw `str::len`).
///
/// Words are broken at whitespace boundaries. A single word longer than
/// `max_width` is broken mid-word. Blank input yields an empty vector.
pub fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return text.lines().map(String::from).collect();
    }

    let mut lines: Vec<String> = Vec::new();
    for input_line in text.lines() {
        if input_line.is_empty() {
            lines.push(String::new());
            continue;
        }
        let mut current = String::new();
        let mut current_width = 0usize;

        for word in input_line.split(' ') {
            // Handle consecutive spaces in the source; skip empty tokens
            // but maintain *single* spaces between words.
            if word.is_empty() {
                continue;
            }

            let word_width = UnicodeWidthStr::width(word);
            let space_width = if current.is_empty() { 0 } else { 1 };

            if current_width + space_width + word_width <= max_width {
                if !current.is_empty() {
                    current.push(' ');
                    current_width += 1;
                }
                current.push_str(word);
                current_width += word_width;
            } else {
                // Flush the current line if it has content.
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                    current_width = 0;
                }

                // If the word itself exceeds max_width, break it.
                if word_width > max_width {
                    // Chunk by max_width display width.
                    let mut remaining = word.to_string();
                    while UnicodeWidthStr::width(&*remaining) > max_width {
                        let (chunk, rest) = split_at_display_width(&remaining, max_width);
                        lines.push(chunk);
                        remaining = rest;
                    }
                    if !remaining.is_empty() {
                        current = remaining;
                        current_width = UnicodeWidthStr::width(&*current);
                    }
                } else {
                    current.push_str(word);
                    current_width = word_width;
                }
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }
    }
    lines
}

/// Split `s` at `display_width` counting Unicode display width.
fn split_at_display_width(s: &str, width: usize) -> (String, String) {
    let mut w = 0usize;
    let mut byte_idx = 0usize;
    for (i, ch) in s.char_indices() {
        let chw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + chw > width {
            break;
        }
        w += chw;
        byte_idx = i + ch.len_utf8();
    }
    if byte_idx == 0 {
        // Even one char doesn't fit; force at least one.
        if let Some(ch) = s.chars().next() {
            byte_idx = ch.len_utf8();
        }
    }
    let left = s[..byte_idx].to_string();
    let right = s[byte_idx..].to_string();
    (left, right)
}

/// Align text horizontally within `width`.
///
/// `text` should already be wrapped to fit `width`.
pub fn align_horizontal(
    lines: &[String],
    width: usize,
    align: crate::style::HAlign,
) -> Vec<String> {
    lines
        .iter()
        .map(|line| {
            let lw = UnicodeWidthStr::width(line.as_str());
            let padding = width.saturating_sub(lw);
            match align {
                crate::style::HAlign::Left => format!("{}{}", line, " ".repeat(padding)),
                crate::style::HAlign::Right => format!("{}{}", " ".repeat(padding), line),
                crate::style::HAlign::Center => {
                    let left = padding / 2;
                    let right = padding - left;
                    format!("{}{}{}", " ".repeat(left), line, " ".repeat(right))
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::HAlign;

    #[test]
    fn test_basic_wrapping() {
        let wrapped = word_wrap("hello world foo", 8);
        assert_eq!(wrapped, vec!["hello", "world", "foo"]);
    }

    #[test]
    fn test_long_word_breaking() {
        let wrapped = word_wrap("supercalifragilistic", 5);
        assert_eq!(wrapped, vec!["super", "calif", "ragil", "istic"]);
    }

    #[test]
    fn test_multiline_input() {
        let wrapped = word_wrap("line one\nline two", 20);
        assert_eq!(wrapped, vec!["line one", "line two"]);
    }

    #[test]
    fn test_center_align() {
        let lines = vec!["Hi".to_string()];
        let aligned = align_horizontal(&lines, 6, HAlign::Center);
        assert_eq!(aligned, vec!["  Hi  "]);
    }
}
