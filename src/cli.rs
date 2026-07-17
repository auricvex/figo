//! CLI argument parsing and dispatch logic.

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;

/// figo — ASCII Art Generator
#[derive(Parser, Debug)]
#[command(name = "figo", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Write output to file instead of stdout
    #[arg(long, global = true)]
    pub output: Option<PathBuf>,

    /// Copy output to system clipboard
    #[arg(long, global = true)]
    pub clipboard: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Render a bordered box/container
    Box {
        /// Inline JSON string
        json: Option<String>,

        /// Read JSON from file
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a table/grid
    Table {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a flowchart diagram
    Flowchart {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render an IETF packet header diagram
    Packet {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a hierarchical tree
    Tree {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render an arrow/connector
    Arrow {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a sequence diagram
    Sequence {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a FIGlet text banner
    Banner {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a Gantt chart
    Gantt {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Render a UML state diagram
    State {
        json: Option<String>,

        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

/// Resolve the JSON input string using the priority order:
/// 1. Inline JSON argument
/// 2. `--file <path>`
/// 3. stdin
pub fn resolve_json_input(
    json_arg: Option<&str>,
    file_arg: Option<&PathBuf>,
) -> figo::error::Result<String> {
    if let Some(json) = json_arg {
        if !json.trim().is_empty() {
            return Ok(json.to_string());
        }
    }
    if let Some(path) = file_arg {
        return Ok(std::fs::read_to_string(path)?);
    }
    // Try stdin. Cap the read at 10 MiB so a malicious or accidental
    // huge payload piped into the command cannot exhaust memory.
    let cap: u64 = 10 * 1024 * 1024;
    let mut stdin = std::io::stdin().take(cap);
    let mut buf = String::new();
    stdin.read_to_string(&mut buf)?;
    if !buf.trim().is_empty() {
        return Ok(buf);
    }
    Err(figo::error::FigoError::General("no input provided (inline JSON, --file, or stdin)".into()))
}
