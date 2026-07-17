//! Smart rendering engine for figo diagrams.
//!
//! Provides a widget-based layout and rendering system on top of the
//! low-level [`crate::canvas::Canvas`]. The engine separates sizing,
//! layout, and painting into distinct passes so diagrams can compute
//! stable spacing, avoid overlaps, and route connectors efficiently.
//!
//! # Core concepts
//!
//! * [`Surface`] — a clipped drawing target backed by a [`Canvas`].
//! * [`BoxConstraints`] — min/max width/height passed down during layout.
//! * [`Widget`] — trait for anything that can be measured, laid out, and painted.
//! * [`Node`] — a bordered rectangular component with content and padding.
//! * [`Connector`] — an orthogonal connector between two anchors with arrowheads.
//!
//! [`Surface`]: surface::Surface
//! [`BoxConstraints`]: widget::BoxConstraints
//! [`Widget`]: widget::Widget
//! [`Node`]: node::Node
//! [`Connector`]: connector::Connector

pub mod connector;
pub mod layout;
pub mod node;
pub mod surface;
pub mod widget;

pub use connector::{Connector, ConnectorStyle, Marker};
pub use layout::{Axis, Flex, FlexItem, Stack, StackItem};
pub use node::Node;
pub use surface::Surface;
pub use widget::{BoxConstraints, Rect as WidgetRect, Size, Widget};
