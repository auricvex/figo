//! FIGlet text banner rendering.

use crate::error::Result;
use crate::text::figlet;

/// Draw a FIGlet-style text banner.
///
/// This is a thin wrapper around the FIGlet rendering engine.
pub fn draw_banner(text: &str, _width: usize) -> Result<String> {
    figlet::render_figlet(text)
}
