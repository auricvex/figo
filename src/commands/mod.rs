//! CLI command handlers: JSON parsing and diagram dispatch.
//!
//! Each diagram type has a JSON input struct and a `run_*` function that
//! deserializes the input and calls the appropriate library function.
//!
//! Sub-modules are organized by diagram category to keep files under 250 lines.

mod flowchart;
mod network;
mod shapes;
mod state;
mod timeline;

pub use flowchart::run_flowchart;
pub use network::{run_arrow, run_packet, run_tree};
pub use shapes::{run_box, run_table};
pub use state::run_state;
pub use timeline::{run_banner, run_gantt, run_sequence};

use figo::style::{BorderStyle, Charset, HAlign};
use serde::Deserialize;

/// JSON character set — always "ascii" or "unicode".
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum JsonCharset {
    Ascii,
    Unicode,
}

impl From<JsonCharset> for Charset {
    fn from(c: JsonCharset) -> Self {
        match c {
            JsonCharset::Ascii => Charset::Ascii,
            JsonCharset::Unicode => Charset::Unicode,
        }
    }
}

/// JSON border style.
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum JsonBorder {
    Single,
    Double,
    Rounded,
    Dashed,
    Bold,
}

pub(crate) fn border_from_json(b: JsonBorder) -> BorderStyle {
    match b {
        JsonBorder::Single => BorderStyle::Single,
        JsonBorder::Double => BorderStyle::Double,
        JsonBorder::Rounded => BorderStyle::Rounded,
        JsonBorder::Dashed => BorderStyle::Dashed,
        JsonBorder::Bold => BorderStyle::Bold,
    }
}

/// JSON padding specification.
#[derive(Deserialize)]
pub(crate) struct JsonPadding {
    pub horizontal: usize,
    pub vertical: usize,
}

/// JSON alignment specification.
#[derive(Deserialize)]
pub(crate) struct JsonAlignment {
    pub horizontal: String,
    pub vertical: String,
}

/// JSON position (x, y).
#[derive(Deserialize)]
pub(crate) struct JsonPosition {
    pub x: usize,
    pub y: usize,
}

/// Parse a JSON horizontal alignment string.
pub(crate) fn parse_halign(s: &str) -> HAlign {
    match s {
        "center" => HAlign::Center,
        "right" => HAlign::Right,
        _ => HAlign::Left,
    }
}

/// Parse a JSON vertical alignment string.
pub(crate) fn parse_valign(s: &str) -> figo::style::VAlign {
    match s {
        "middle" => figo::style::VAlign::Middle,
        "bottom" => figo::style::VAlign::Bottom,
        _ => figo::style::VAlign::Top,
    }
}
