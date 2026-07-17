//! # figo — ASCII Art Generator
//!
//! **figo** is a Rust-based ASCII art generator, usable both as a CLI application
//! and as a library. It generates structured ASCII/Unicode art for documentation,
//! RFC/IETF-style diagrams, flowcharts, tables, trees, banners, and more.
//!
//! ## Library usage
//!
//! Each diagram type exposes both free functions (for simple usage) and builder
//! structs (for complex configurations).
//!
//! ```rust,no_run
//! use figo::diagrams::box_art;
//! use figo::style::{Charset, BorderStyle};
//!
//! let output = box_art::draw_box(
//!     Some("My Box"),
//!     Some("Hello, world!"),
//!     60,
//!     Charset::Unicode,
//!     BorderStyle::Single,
//! ).unwrap();
//! println!("{output}");
//! ```

pub mod canvas;
pub mod diagrams;
pub mod error;
pub mod layout;
pub mod output;
pub mod render;
pub mod style;
pub mod text;

// Re-export key types for convenience
pub use error::{FigoError, Result};
pub use style::{Alignment, BorderStyle, Charset, Color, HAlign, LineStyle, VAlign};
