# Changelog

All notable changes to figo will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] — 2026-07-18

### Changed

- **State Diagram** (`figo state`) — complete rewrite from UML state machine to
  **FSM (Finite State Machine)** style. States are rendered as rounded pills;
  accepting states use a double rounded border. Automatic layered layout replaces
  the manual grid-based positioning. Transitions route from source bottom to
  target top through a gap-based corridor, with direction-aware arrowheads
  (▼ forward, ▲ back).
  - **Breaking:** Removed composite, initial (node type), final, and history
    state types. Use `"simple"` and `"accepting"` instead.
  - **Breaking:** Removed `row`, `col`, and `children` fields from state nodes.
    Layout is now fully automatic.
  - **Breaking:** `StateType::Final` renamed to `StateType::Accepting`.

## [0.1.0] — 2026-07-16

### Added

- **Box Art** (`figo box`) — bordered containers with title, content, word-wrap,
  padding, alignment, and 5 border styles (single, double, rounded, dashed, bold)
- **Table** (`figo table`) — grid/table layouts with headers, rows, configurable
  columns, per-column alignment, padding, and header separators
- **Flowchart** (`figo flowchart`) — rectangular/rounded/diamond nodes with
  auto-layout (Sugiyama-style) or manual positioning, orthogonally-routed edges
- **Packet Header** (`figo packet`) — IETF/RFC-style packet header diagrams with
  32-bit word scale and bordered field cells
- **Tree** (`figo tree`) — hierarchical tree diagrams with Unicode/ASCII branch
  characters and arbitrary nesting depth
- **Arrow** (`figo arrow`) — standalone arrows/connectors (horizontal, vertical,
  bidirectional) with configurable line styles and labels
- **Sequence Diagram** (`figo sequence`) — timeline-based message sequence
  diagrams with participant lanes, message arrows, and self-messages
- **Banner** (`figo banner`) — FIGlet text banners using the bundled "standard" font
- **Gantt Chart** (`figo gantt`) — project management Gantt charts with sections,
  tasks, progress bars, milestones, dependencies, and today markers
- **State Diagram** (`figo state`) — UML state machine diagrams with simple,
  composite, initial, final, and choice states, plus labeled transitions
- ASCII and Unicode character set support for all diagram types
- ANSI color support (opt-in via `--color` / `color: true`)
- CLI with inline JSON, `--file`, and stdin input; `--output`, `--clipboard` output
- Public library API with free functions and builder patterns for all diagram types
- 2D canvas rendering engine with word-wrapping and text alignment utilities

### Dependencies

- `clap` 4.5 for CLI argument parsing
- `serde` / `serde_json` for JSON deserialization
- `arboard` 3.4 for clipboard access
- `thiserror` 2.0 for error handling
- `unicode-width` 0.2 for accurate text measurement
- `insta` 1.42 (dev) for snapshot testing
