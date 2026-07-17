//! CLI binary entry point for figo.
//!
//! Parses CLI arguments, resolves JSON input, dispatches to the appropriate
//! diagram command handler, and writes output to stdout, file, or clipboard.

mod cli;
mod commands;

use clap::Parser;
use cli::resolve_json_input;
use figo::error::Result;
use figo::output;

fn main() {
    let args = cli::Cli::parse();

    if let Err(e) = run(args) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

/// Dispatch the parsed CLI command to the appropriate handler and produce output.
fn run(args: cli::Cli) -> Result<()> {
    let output = match &args.command {
        cli::Command::Box { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_box(&input)?
        }
        cli::Command::Table { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_table(&input)?
        }
        cli::Command::Flowchart { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_flowchart(&input)?
        }
        cli::Command::Packet { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_packet(&input)?
        }
        cli::Command::Tree { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_tree(&input)?
        }
        cli::Command::Arrow { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_arrow(&input)?
        }
        cli::Command::Sequence { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_sequence(&input)?
        }
        cli::Command::Banner { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_banner(&input)?
        }
        cli::Command::Gantt { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_gantt(&input)?
        }
        cli::Command::State { json, file } => {
            let input = resolve_json_input(json.as_deref(), file.as_ref())?;
            commands::run_state(&input)?
        }
    };

    // Dispatch output
    let destination = output::Destination::from_args(args.output, args.clipboard);
    output::write(&output, destination)?;
    Ok(())
}
