//! Layout sizing utilities for distributing columns and fitting content
//! inside a parent width.
//!
//! These helpers are used by diagrams such as tables and flowcharts where
//! multiple sub-regions (columns, nodes) need to share a total width.

/// Distribute `extra` units of width across `n` columns, each starting
/// from `base_width`. The result is `base_width + extra_per_col`. If
/// `extra == 0`, all columns receive exactly `base_width`.
pub fn distribute(extra: usize, base_width: usize, n: usize) -> Vec<usize> {
    if n == 0 {
        return Vec::new();
    }
    let per = extra / n;
    let remainder = extra % n;
    (0..n).map(|i| base_width + per + if i < remainder { 1 } else { 0 }).collect()
}

/// Given minimum widths for N children and a target total width, return
/// adjusted widths that:
/// 1. Each child has at least its minimum.
/// 2. The sum equals `target_width` (when at least that large).
pub fn fit_to_width(min_widths: &[usize], target_width: usize) -> Vec<usize> {
    let n = min_widths.len();
    if n == 0 {
        return Vec::new();
    }
    let sum_min: usize = min_widths.iter().sum();
    if sum_min >= target_width {
        return min_widths.to_vec();
    }
    let extra = target_width - sum_min;
    distribute(extra, 0, n)
        .into_iter()
        .zip(min_widths.iter())
        .map(|(shared, min)| shared + min)
        .collect()
}

/// Compute column starts (left x positions) given a total width and
/// padding between columns. Each entry is the starting x of column i.
pub fn column_starts(total_width: usize, col_widths: &[usize], padding: usize) -> Vec<usize> {
    let mut starts = Vec::with_capacity(col_widths.len());
    let mut x = 0usize;
    for (i, cw) in col_widths.iter().enumerate() {
        starts.push(x);
        x += cw;
        if i + 1 < col_widths.len() {
            x += 1 + padding * 2;
        }
    }
    let _ = total_width;
    starts
}

/// Trim trailing whitespace from a row of text (used when rendering
/// pre-padded content into the canvas).
pub fn trim_trailing(s: &str) -> &str {
    s.trim_end_matches(' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_extra_evenly() {
        let r = distribute(6, 2, 3);
        assert_eq!(r, vec![4, 4, 4]);
    }

    #[test]
    fn distribute_with_remainder() {
        let r = distribute(7, 0, 3);
        assert_eq!(r, vec![3, 2, 2]);
    }

    #[test]
    fn fit_to_width_min_first() {
        let r = fit_to_width(&[3, 5, 2], 14);
        // sum_min = 10, extra = 4
        assert_eq!(r, vec![5, 6, 3]);
    }
}
