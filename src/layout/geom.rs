//! Geometry primitives: rectangles, anchor points, and cardinal directions.
//!
//! These types are shared between the smart canvas and the layout / node /
//! connector modules. Keeping them in one small file avoids a tangle of
//! cross-module type definitions.

/// An inclusive rectangle on the canvas, anchored at the top-left corner
/// with positive `(x, y)` and dimensions `(w, h)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl Rect {
    /// Construct a new rectangle.
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self { x, y, w, h }
    }

    /// The right x-coordinate (exclusive).
    pub fn right(&self) -> usize {
        self.x + self.w
    }

    /// The bottom y-coordinate (exclusive).
    pub fn bottom(&self) -> usize {
        self.y + self.h
    }

    /// The center x-coordinate.
    pub fn cx(&self) -> usize {
        self.x + self.w / 2
    }

    /// The center y-coordinate.
    pub fn cy(&self) -> usize {
        self.y + self.h / 2
    }

    /// Returns true if `(x, y)` is inside this rectangle.
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x && y >= self.y && x < self.right() && y < self.bottom()
    }

    /// Returns true if the two rectangles touch or overlap.
    pub fn overlaps(&self, other: &Rect) -> bool {
        !(self.right() <= other.x
            || other.right() <= self.x
            || self.bottom() <= other.y
            || other.bottom() <= self.y)
    }

    /// Anchor point on the rect's perimeter.
    pub fn anchor(&self, a: Anchor) -> (usize, usize) {
        match a {
            Anchor::North => (self.cx(), self.y),
            Anchor::South => (self.cx(), self.bottom().saturating_sub(1)),
            Anchor::East => (self.right().saturating_sub(1), self.cy()),
            Anchor::West => (self.x, self.cy()),
            Anchor::NorthEast => (self.right().saturating_sub(1), self.y),
            Anchor::NorthWest => (self.x, self.y),
            Anchor::SouthEast => (self.right().saturating_sub(1), self.bottom().saturating_sub(1)),
            Anchor::SouthWest => (self.x, self.bottom().saturating_sub(1)),
            Anchor::Center => (self.cx(), self.cy()),
        }
    }
}

/// Anchor point on a [`Rect`]'s perimeter (cardinal or corner).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Center,
}

impl Anchor {
    /// The unit vector pointing outward from an anchor. North/South are
    /// represented as (0, -1)/(0, +1) — used by connectors that need to
    /// know which way a line should exit a node.
    pub fn direction_to(&self, other: Anchor) -> (i32, i32) {
        let (dx, dy) = match (self, other) {
            (Anchor::North, _) | (_, Anchor::North) => (0, -1),
            (Anchor::South, _) | (_, Anchor::South) => (0, 1),
            (Anchor::East, _) | (_, Anchor::East) => (1, 0),
            (Anchor::West, _) | (_, Anchor::West) => (-1, 0),
            _ => (0, 0),
        };
        (dx, dy)
    }
}
