//! Orthogonal routing helpers shared by the connector router.
//!
//! Keeping these in a separate file lets the [`crate::layout::connector`]
//! module stay under the project's 250-line file cap while the routing
//! geometry is testable in isolation.

use super::geom::{Anchor, Rect};

/// One straight segment of a connector path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Horizontal segment starting at `(x, y)` with length `len`.
    H { x: usize, y: usize, len: usize },
    /// Vertical segment starting at `(x, y)` with length `len`.
    V { x: usize, y: usize, len: usize },
}

/// Pick the horizontal-corridor row (`mid_y`) for a vertical-flow
/// connector so the H segment sits in the **gap between** the source and
/// target — never inside either rect.
///
/// When `sy < ty` (flow downward) the corridor is placed at the
/// midpoint of the vertical gap between the source bottom and target
/// top. When `sy > ty` (flow upward) the corridor sits in the gap
/// between the target bottom and source top.
pub fn natural_mid_y(sy: usize, ty: usize, src: &Rect, tgt: &Rect) -> usize {
    if sy < ty {
        let lo = src.bottom();
        let hi = tgt.y;
        let mid = if hi > lo { lo + (hi - lo) / 2 } else { lo };
        mid.max(lo)
    } else if sy > ty {
        let lo = tgt.bottom();
        let hi = src.y;
        let mid = if hi > lo { lo + (hi - lo) / 2 } else { lo };
        if mid >= src.y { src.y.saturating_sub(1) } else { mid }
    } else {
        sy
    }
}

/// Detoured `mid_y` for the vertical-flow path when the natural path
/// intersects an avoidance rectangle. Pushes the corridor above or below
/// every obstacle, preferring the direction of the flow. Only obstacles
/// that lie vertically between the source and target are considered.
pub fn detoured_mid_y(sy: usize, ty: usize, avoid: &[Rect], src: &Rect, tgt: &Rect) -> usize {
    if avoid.is_empty() || sy == ty {
        return natural_mid_y(sy, ty, src, tgt);
    }
    if sy < ty {
        let max_bottom =
            avoid.iter().filter(|r| r.bottom() <= tgt.y).map(|r| r.bottom()).max().unwrap_or(0);
        let safe = src.bottom().max(max_bottom) + 1;
        safe.min(tgt.y.saturating_sub(1))
    } else {
        let min_top =
            avoid.iter().filter(|r| r.y >= tgt.bottom()).map(|r| r.y).min().unwrap_or(usize::MAX);
        let safe = src.y.min(min_top).saturating_sub(1);
        safe.max(tgt.bottom() + 1)
    }
}

/// Detoured `mid_x` for the horizontal-flow path.
pub fn detoured_mid_x(avoid: &[Rect]) -> usize {
    if avoid.is_empty() {
        return 1;
    }
    let max_right = avoid.iter().map(|r| r.right()).max().unwrap_or(0);
    max_right + 1
}

/// A vertical-corridor column to the right of all rectangles, used to
/// route back-edges around intermediate nodes. Returns the column index
/// two cells right of the rightmost rect, clamped to the canvas width.
pub fn side_route_column(all: &[Rect], canvas_w: usize) -> usize {
    let max_right = all.iter().map(|r| r.right()).max().unwrap_or(0);
    let candidate = max_right + 2;
    candidate.min(canvas_w.saturating_sub(1).max(max_right + 1))
}

/// Snap a point one cell outside the rect's perimeter along the anchor
/// direction, so connector lines start/end just outside the border.
pub fn snap_outside(rect: Rect, anchor: Anchor) -> (usize, usize) {
    let (x, y) = rect.anchor(anchor);
    let (dx, dy) = match anchor {
        Anchor::North => (0i32, -1i32),
        Anchor::South => (0, 1),
        Anchor::East => (1, 0),
        Anchor::West => (-1, 0),
        Anchor::NorthEast => (1, -1),
        Anchor::NorthWest => (-1, -1),
        Anchor::SouthEast => (1, 1),
        Anchor::SouthWest => (-1, 1),
        Anchor::Center => (0, 0),
    };
    let nx = (x as i32 + dx).max(0) as usize;
    let ny = (y as i32 + dy).max(0) as usize;
    (nx, ny)
}

/// True if any segment of `path` intersects any rectangle in `avoid`.
pub fn path_intersects_any(path: &[Segment], avoid: &[Rect]) -> bool {
    if avoid.is_empty() {
        return false;
    }
    path.iter().any(|seg| {
        let r: Rect = match *seg {
            Segment::H { x, y, len } => Rect::new(x, y, len.max(1), 1),
            Segment::V { x, y, len } => Rect::new(x, y, 1, len.max(1)),
        };
        avoid.iter().any(|a| a.overlaps(&r))
    })
}

/// A straight vertical segment (or two stacked V segments) from `(x, sy)`
/// to `(x, ty)`.
pub fn straight_vertical(x: usize, sy: usize, ty: usize) -> Vec<Segment> {
    if sy < ty {
        vec![Segment::V { x, y: sy, len: ty - sy + 1 }]
    } else {
        vec![Segment::V { x, y: ty, len: sy - ty + 1 }]
    }
}

/// Build a 3-segment V-H-V path through the horizontal corridor at
/// `mid_y`.
pub fn build_three_segment(
    sx: usize,
    sy: usize,
    tx: usize,
    ty: usize,
    mid_y: usize,
) -> Vec<Segment> {
    let mut segs: Vec<Segment> = Vec::new();
    if sy <= mid_y {
        segs.push(Segment::V { x: sx, y: sy, len: mid_y - sy + 1 });
    } else {
        segs.push(Segment::V { x: sx, y: mid_y, len: sy - mid_y + 1 });
    }
    if sx < tx {
        segs.push(Segment::H { x: sx, y: mid_y, len: tx - sx + 1 });
    } else {
        segs.push(Segment::H { x: tx, y: mid_y, len: sx - tx + 1 });
    }
    if ty < mid_y {
        segs.push(Segment::V { x: tx, y: ty, len: mid_y - ty + 1 });
    } else {
        segs.push(Segment::V { x: tx, y: mid_y, len: ty - mid_y + 1 });
    }
    segs
}

/// Build a 3-segment H-V-H path through the vertical corridor at
/// `mid_x`.
pub fn build_three_h_segment(
    sx: usize,
    sy: usize,
    tx: usize,
    ty: usize,
    mid_x: usize,
) -> Vec<Segment> {
    let mut segs: Vec<Segment> = Vec::new();
    if sx <= mid_x {
        segs.push(Segment::H { x: sx, y: sy, len: mid_x - sx + 1 });
    } else {
        segs.push(Segment::H { x: mid_x, y: sy, len: sx - mid_x + 1 });
    }
    if sy < ty {
        segs.push(Segment::V { x: mid_x, y: sy, len: ty - sy + 1 });
    } else {
        segs.push(Segment::V { x: mid_x, y: ty, len: sy - ty + 1 });
    }
    if tx < mid_x {
        segs.push(Segment::H { x: tx, y: ty, len: mid_x - tx + 1 });
    } else {
        segs.push(Segment::H { x: mid_x, y: ty, len: tx - mid_x + 1 });
    }
    segs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mid_y_sits_between_source_and_target() {
        let src = Rect::new(10, 1, 8, 3);
        let tgt = Rect::new(30, 12, 8, 3);
        let mid = natural_mid_y(4, 12, &src, &tgt);
        assert!(mid >= src.bottom() && mid < tgt.y, "mid {mid} must be in gap [4,12)");
    }

    #[test]
    fn mid_y_collapses_when_same_row() {
        let src = Rect::new(0, 5, 4, 3);
        let tgt = Rect::new(10, 5, 4, 3);
        assert_eq!(natural_mid_y(5, 5, &src, &tgt), 5);
    }
}
