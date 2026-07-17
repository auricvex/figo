//! Output targets: stdout, file, and clipboard.

use crate::error::Result;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Where to write rendered output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    /// Write to standard output.
    Stdout,
    /// Write to a file at the given path.
    File(PathBuf),
    /// Copy to the system clipboard.
    Clipboard,
}

impl Destination {
    /// Build a destination from CLI-style arguments.
    ///
    /// `clipboard` takes precedence, then an explicit file path; otherwise
    /// output goes to stdout.
    pub fn from_args(output: Option<PathBuf>, clipboard: bool) -> Self {
        if clipboard {
            Self::Clipboard
        } else if let Some(path) = output {
            Self::File(path)
        } else {
            Self::Stdout
        }
    }
}

/// Write rendered output to the chosen destination.
pub fn write(output: &str, destination: Destination) -> Result<()> {
    match destination {
        Destination::Stdout => write_stdout(output),
        Destination::File(path) => write_file(output, &path),
        Destination::Clipboard => write_clipboard(output),
    }
}

/// Write rendered output to stdout.
pub fn write_stdout(output: &str) -> Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    handle.write_all(output.as_bytes())?;
    handle.flush()?;
    Ok(())
}

/// Write rendered output to a file at the given path.
/// Creates the file if it doesn't exist; overwrites if it does.
pub fn write_file(output: &str, path: &Path) -> Result<()> {
    std::fs::write(path, output)?;
    Ok(())
}

/// Copy rendered output to the system clipboard.
pub fn write_clipboard(output: &str) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| crate::error::FigoError::Clipboard(format!("cannot open clipboard: {e}")))?;
    clipboard
        .set_text(output)
        .map_err(|e| crate::error::FigoError::Clipboard(format!("cannot set clipboard: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destination_prefers_clipboard() {
        let dest = Destination::from_args(Some(PathBuf::from("out.txt")), true);
        assert_eq!(dest, Destination::Clipboard);
    }

    #[test]
    fn destination_uses_file_when_no_clipboard() {
        let dest = Destination::from_args(Some(PathBuf::from("out.txt")), false);
        assert_eq!(dest, Destination::File(PathBuf::from("out.txt")));
    }

    #[test]
    fn destination_defaults_to_stdout() {
        let dest = Destination::from_args(None, false);
        assert_eq!(dest, Destination::Stdout);
    }
}
