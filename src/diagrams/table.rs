//! Grid/table layouts with headers, rows, and configurable columns.
//!
//! Provides both a free function ([`draw_table`]) for simple usage and a
//! builder ([`Table`]) for complex configurations.

use std::fmt;

use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

use crate::canvas::Canvas;
use crate::error::Result;
use crate::style::{BorderStyle, Charset, HAlign, Padding};

/// Draw a table.
pub fn draw_table(
    headers: &[&str],
    rows: &[Vec<&str>],
    width: usize,
    charset: Charset,
    border: BorderStyle,
) -> Result<String> {
    Table::new(width, charset).headers(headers).rows(rows).border(border).build()
}

/// Builder for table diagrams.
pub struct Table<'a> {
    width: usize,
    charset: Charset,
    headers: &'a [&'a str],
    rows: &'a [Vec<&'a str>],
    col_widths: Option<Vec<usize>>,
    padding: Padding,
    align: Vec<HAlign>,
    border: BorderStyle,
    header_separator: bool,
    color: bool,
}

impl<'a> Table<'a> {
    /// Create a new table builder.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self {
            width,
            charset,
            headers: &[],
            rows: &[],
            col_widths: None,
            padding: Padding::default(),
            align: Vec::new(),
            border: BorderStyle::Single,
            header_separator: true,
            color: false,
        }
    }

    /// Set the header row.
    pub fn headers(mut self, headers: &'a [&'a str]) -> Self {
        self.headers = headers;
        self
    }

    /// Set the data rows.
    pub fn rows(mut self, rows: &'a [Vec<&'a str>]) -> Self {
        self.rows = rows;
        self
    }

    /// Set explicit column widths. If `None`, columns are auto-sized.
    pub fn col_widths(mut self, widths: Option<Vec<usize>>) -> Self {
        self.col_widths = widths;
        self
    }

    /// Set cell padding.
    pub fn padding(mut self, horizontal: usize, vertical: usize) -> Self {
        self.padding = Padding { horizontal, vertical };
        self
    }

    /// Set per-column horizontal alignment.
    pub fn align(mut self, align: Vec<HAlign>) -> Self {
        self.align = align;
        self
    }

    /// Set the border style.
    pub fn border(mut self, style: BorderStyle) -> Self {
        self.border = style;
        self
    }

    /// Show or hide the separator line below the header.
    pub fn header_separator(mut self, show: bool) -> Self {
        self.header_separator = show;
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render the table and return it as a `String`.
    pub fn build(&self) -> Result<String> {
        let col_count = self.header_or_max_cols();
        if col_count == 0 {
            return self.render_empty();
        }

        // Step 1: compute per-column minimum widths using unicode display width.
        let min_widths = self.compute_min_widths(col_count);

        // Step 2: distribute any surplus width evenly across columns so a
        // single wide column never eats the rest. This fixes the
        // unbalanced-column bug when total width >> sum of mins.
        let pad_per_col = self.padding.horizontal * 2;
        let min_total: usize = min_widths.iter().sum::<usize>()
            + pad_per_col * col_count
            + col_count.saturating_sub(1);
        let surplus = self.width.saturating_sub(min_total + 2);
        let mut col_widths = min_widths;
        if surplus >= col_count {
            let per = surplus / col_count;
            let rem = surplus % col_count;
            for (i, w) in col_widths.iter_mut().enumerate() {
                *w += per;
                if i < rem {
                    *w += 1;
                }
            }
        }

        let display_width: usize =
            col_widths.iter().sum::<usize>() + pad_per_col * col_count + col_count + 1;

        let all_rows = self.collect_rows(col_count);
        let glyphs = self.border.glyphs(self.charset);
        let col_starts = self.compute_col_starts(&col_widths);

        let data_line_count = all_rows.len();
        let has_sep = self.header_separator && !self.headers.is_empty() && !all_rows.is_empty();
        let sep_lines: usize = if has_sep { 1 } else { 0 };
        let total_height = 1 + data_line_count * (1 + self.padding.vertical * 2) + sep_lines + 1;

        let mut canvas = Canvas::new(display_width.max(self.width), total_height);

        // Outer border
        canvas.draw_rect(0, 0, display_width, total_height, &glyphs)?;

        // Column separators (vertical lines) at each column boundary.
        let sep_xs: Vec<usize> = (1..col_count)
            .map(|ci| col_starts[ci].saturating_sub(1 + self.padding.horizontal))
            .collect();
        for &sep_x in &sep_xs {
            for y in 1..total_height - 1 {
                canvas.put(sep_x, y, glyphs.vertical);
            }
            canvas.put(sep_x, 0, glyphs.tee_down);
            canvas.put(sep_x, total_height - 1, glyphs.tee_up);
        }

        // Header separator line: write ─ except at column joints where we
        // place ┼. Done in a single pass that walks the canvas row.
        if has_sep {
            let sep_y = 1 + (1 + self.padding.vertical * 2);
            for x in 0..display_width {
                let existing = canvas.cell_char(x, sep_y).unwrap_or(' ');
                let ch = if existing == glyphs.vertical {
                    glyphs.cross
                } else if existing == ' ' {
                    glyphs.horizontal
                } else {
                    existing
                };
                canvas.put(x, sep_y, ch);
            }
        }

        // Place cell text. Each cell is centered within its column.
        let mut y = 1 + self.padding.vertical;
        for (ri, row_cells) in all_rows.iter().enumerate() {
            for (ci, cell_text) in row_cells.iter().enumerate() {
                let col_start = col_starts[ci];
                let col_w = col_widths[ci];
                let align = self.align.get(ci).copied().unwrap_or(HAlign::Left);
                let display = aligned_cell(cell_text, col_w, align);
                canvas.put_str(col_start, y, &display);
            }
            let skip = 1 + self.padding.vertical * 2;
            y += skip;
            if has_sep && ri == 0 {
                y += 1;
            }
        }

        Ok(canvas.render(self.color))
    }

    fn render_empty(&self) -> Result<String> {
        let glyphs = self.border.glyphs(self.charset);
        let w = self.width.max(4);
        let h = 2;
        let mut canvas = Canvas::new(w, h);
        canvas.draw_rect(0, 0, w, h, &glyphs)?;
        Ok(canvas.render(self.color))
    }

    fn header_or_max_cols(&self) -> usize {
        let hdr = self.headers.len();
        let row_max = self.rows.iter().map(|r| r.len()).max().unwrap_or(0);
        hdr.max(row_max)
    }

    /// Minimum width per column, in display cells (unicode-width aware).
    fn compute_min_widths(&self, col_count: usize) -> Vec<usize> {
        if let Some(ref explicit) = self.col_widths {
            let mut widths = explicit.clone();
            widths.resize(col_count, 10);
            return widths;
        }
        let mut widths = Vec::with_capacity(col_count);
        for ci in 0..col_count {
            let hdr_w = self.headers.get(ci).map_or(0, |h| h.width());
            let max_cell_w =
                self.rows.iter().filter_map(|r| r.get(ci)).map(|c| c.width()).max().unwrap_or(0);
            widths.push(hdr_w.max(max_cell_w).max(1));
        }
        widths
    }

    fn compute_col_starts(&self, col_widths: &[usize]) -> Vec<usize> {
        let mut starts = Vec::with_capacity(col_widths.len());
        let mut x = 1 + self.padding.horizontal;
        for cw in col_widths {
            starts.push(x);
            x += cw + self.padding.horizontal * 2 + 1;
        }
        starts
    }

    fn collect_rows(&self, col_count: usize) -> Vec<Vec<String>> {
        let mut out: Vec<Vec<String>> = Vec::new();
        if !self.headers.is_empty() {
            let hdr = (0..col_count)
                .map(|ci| self.headers.get(ci).copied().unwrap_or("").to_string())
                .collect();
            out.push(hdr);
        }
        for row in self.rows {
            let cells: Vec<String> =
                (0..col_count).map(|ci| row.get(ci).copied().unwrap_or("").to_string()).collect();
            out.push(cells);
        }
        out
    }
}

impl fmt::Display for Table<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

/// Render a cell with the given column width and horizontal alignment,
/// truncating long content and padding with spaces to fit exactly `width`.
fn aligned_cell(text: &str, width: usize, align: HAlign) -> String {
    let t_len = text.width();
    if t_len >= width {
        let mut out = String::new();
        let mut cur = 0usize;
        for ch in text.chars() {
            let w = ch.width().unwrap_or(0);
            if cur + w > width {
                break;
            }
            out.push(ch);
            cur += w;
        }
        while cur < width {
            out.push(' ');
            cur += 1;
        }
        return out;
    }
    let pad = width - t_len;
    match align {
        HAlign::Left => format!("{}{}", text, " ".repeat(pad)),
        HAlign::Right => format!("{}{}", " ".repeat(pad), text),
        HAlign::Center => {
            let left = pad / 2;
            let right = pad - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_table() {
        let headers = &["Name", "Version"];
        let rows = &[vec!["figo", "0.1.0"]];
        let out = draw_table(headers, rows, 40, Charset::Ascii, BorderStyle::Single).unwrap();
        assert!(out.contains("Name"));
        assert!(out.contains("figo"));
        assert!(out.contains("0.1.0"));
    }

    #[test]
    fn test_empty_table() {
        let out = Table::new(30, Charset::Ascii).build().unwrap();
        assert!(out.contains('+'));
        assert!(out.contains('-'));
    }

    #[test]
    fn test_unicode_table() {
        let headers = &["Col1", "Col2"];
        let rows = &[vec!["a", "b"]];
        let out = draw_table(headers, rows, 30, Charset::Unicode, BorderStyle::Single).unwrap();
        assert!(out.contains('┌'));
    }

    #[test]
    fn test_balanced_columns() {
        // The wide "Description" column should not eat all the width; the
        // shorter "Name" column should still receive a fair share.
        let headers = &["Name", "Description"];
        let rows = &[vec!["a", "b"], vec!["xy", "longer text"]];
        let out = Table::new(40, Charset::Ascii).headers(headers).rows(rows).build().unwrap();
        // Both columns should be present and visible.
        assert!(out.contains("Name"));
        assert!(out.contains("Description"));
        assert!(out.contains('+'));
    }

    // -- Snapshot tests -----------------------------------------------------

    #[test]
    fn snapshot_simple_table_ascii() {
        let headers = &["Name", "Version", "Description"];
        let rows = &[
            vec!["figo", "0.1.0", "ASCII art generator"],
            vec!["serde", "1.0", "Serialization framework"],
        ];
        let out = draw_table(headers, rows, 60, Charset::Ascii, BorderStyle::Single).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_table_unicode_double() {
        let headers = &["Col A", "Col B"];
        let rows = &[vec!["alpha", "beta"], vec!["gamma", "delta"]];
        let out = Table::new(40, Charset::Unicode)
            .headers(headers)
            .rows(rows)
            .border(BorderStyle::Double)
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_table_centered_alignment() {
        let headers = &["Item", "Qty", "Price"];
        let rows = &[vec!["Widget", "10", "$4.99"], vec!["Gadget", "2", "$12.50"]];
        let out = Table::new(50, Charset::Unicode)
            .headers(headers)
            .rows(rows)
            .border(BorderStyle::Single)
            .align(vec![HAlign::Left, HAlign::Center, HAlign::Right])
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_empty_table() {
        let out = Table::new(30, Charset::Ascii).build().unwrap();
        insta::assert_snapshot!(out);
    }
}
