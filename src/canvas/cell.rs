//! Cell storage and z-order layer definitions.
//!
//! Cells carry a character, an optional foreground color, and a [`Layer`]
//! representing their z-order. Higher layers overwrite lower layers when
//! written. This lets the renderer draw connectors first, then node borders
//! on top, then labels last — so connectors never poke through nodes.

use crate::style::Color;

/// Z-order layers, from lowest (drawn first / overwritten first) to highest.
///
/// The renderer writes canvases in this implicit pipeline:
/// 1. Background and grid.
/// 2. Connector lines ([`Layer::Connector`]).
/// 3. Node borders ([`Layer::NodeBorder`]).
/// 4. Connector arrowheads adjacent to nodes ([`Layer::ConnectorEnd`]).
/// 5. Node interior content / labels ([`Layer::NodeContent`]).
/// 6. Free-floating labels that must always be visible ([`Layer::Label`]).
///
/// Higher layers always win, so a border cleanly covers a connector that
/// ends at a node's edge, while a connector arrowhead always escapes the
/// border to point at the node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Layer {
    /// Empty background cell.
    #[default]
    Background = 0,
    /// Connector line segment (horizontal / vertical line).
    Connector = 1,
    /// Static grid or scale marker (e.g. gantt time tick).
    Grid = 2,
    /// Node / box border edge.
    NodeBorder = 3,
    /// Connector arrowhead placed at the endpoint of a connector.
    ConnectorEnd = 4,
    /// Node interior content: text labels inside boxes.
    NodeContent = 5,
    /// Free-floating label (e.g. connector label).
    Label = 6,
}

/// One cell in the canvas grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    /// Visible character. Space (`' '`) is the default empty cell.
    pub ch: char,
    /// Optional ANSI foreground color for this cell.
    pub fg: Option<Color>,
    /// Z-order layer used to decide who wins when two cells overlap.
    pub layer: Layer,
}

impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: None, layer: Layer::Background }
    }
}

impl Cell {
    /// Build a cell with the given character and layer.
    pub fn at(ch: char, layer: Layer) -> Self {
        Self { ch, fg: None, layer }
    }

    /// Build a colored cell at the given layer.
    pub fn at_colored(ch: char, layer: Layer, fg: Option<Color>) -> Self {
        Self { ch, fg, layer }
    }

    /// Returns true if writing `incoming` over `self` should win — i.e.
    /// when the new cell's layer is greater than or equal to the existing
    /// one. Equal layers resolve by simple overwrite (last writer wins).
    pub fn should_replace(existing: &Cell, incoming: &Cell) -> bool {
        incoming.layer >= existing.layer
    }
}
