//! Internal layout engine — nodes, connectors, geometry, and sizing
//! helpers shared by all diagram modules.
//!
//! The pieces here are deliberately small so each diagram file can stay
//! under the project's 250-line limit. They are NOT part of the public
//! API and live behind the `diagrams` re-exports.
//!
//! - [`geom`]: `Rect`, `Anchor`
//! - [`node`]: `Node`, `NodeBuilder`
//! - [`connector`]: `Connector` and orthogonal routing helpers
//! - [`routing`]: shared routing geometry helpers (`natural_mid_y`, etc.)
//! - [`sizing`]: column distribution, fit-to-width helpers

pub mod connector;
pub mod geom;
pub mod node;
pub mod routing;
pub mod sizing;

// Re-exports for diagram internals.
pub use connector::{Connector, arrow_from_path, arrow_glyph_for_pair};
pub use geom::{Anchor, Rect};
pub use node::{Node, NodeBuilder, NodeId, NodeLine, horizontal_line_glyph, vertical_line_glyph};
pub use routing::{
    Segment, build_three_h_segment, build_three_segment, detoured_mid_x, detoured_mid_y,
    natural_mid_y, path_intersects_any, side_route_column, snap_outside, straight_vertical,
};
pub use sizing::{column_starts, distribute, fit_to_width, trim_trailing};
