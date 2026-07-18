# figo — ASCII Art Generator

[![CI](https://github.com/auricvex/figo/actions/workflows/ci.yml/badge.svg)](https://github.com/auricvex/figo/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/figo.svg)](https://crates.io/crates/figo)
[![Docs.rs](https://docs.rs/figo/badge.svg)](https://docs.rs/figo)
[![License](https://img.shields.io/badge/license-Apache%202.0%20OR%20MIT-blue.svg)](#license)

**figo** is a Rust-based ASCII art generator — usable both as a CLI application
and as a library. Generate structured ASCII/Unicode art for documentation,
RFC/IETF-style diagrams, flowcharts, tables, trees, banners, and more.

## Installation

### Prerequisites

`figo` requires a **Rust toolchain of at least version 1.85** (the project's
MSRV). The crate uses **Rust Edition 2024**.

- **Linux / macOS / Windows:** Install Rust via [rustup](https://rustup.rs/):
  ```sh
  curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh
  ```
- **Verify your toolchain:**
  ```sh
  rustc --version   # should be 1.85 or newer
  cargo --version
  ```

> **Note for Linux users:** Clipboard support (`--clipboard`) is provided by the
> [`arboard`](https://crates.io/crates/arboard) crate. On some Linux
> distributions you may need additional system libraries for X11 or Wayland.
> See the [clipboard support](#clipboard-support) section below.

### Method 1: Install from crates.io (recommended)

The simplest way to install the latest release is with `cargo install`:

```sh
cargo install figo
```

This downloads the latest published version, compiles it, and installs the
`figo` binary into `~/.cargo/bin/` (or the equivalent location on Windows).

After installation, make sure `~/.cargo/bin` is on your `PATH`:

```sh
# Linux / macOS
export PATH="$HOME/.cargo/bin:$PATH"

# Windows (PowerShell)
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
```

### Method 2: Download a pre-built binary

Pre-built binaries are attached to every
[GitHub Release](https://github.com/auricvex/figo/releases). The CI pipeline
publishes archives for the following targets:

| Target | Archive |
|--------|---------|
| `x86_64-unknown-linux-gnu` | `.tar.gz` |
| `x86_64-unknown-linux-musl` | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `.tar.gz` |
| `x86_64-apple-darwin` | `.tar.gz` |
| `aarch64-apple-darwin` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `.zip` |

#### Linux / macOS

```sh
# Download the latest release (adjust the version and target as needed)
VERSION=$(curl -s https://api.github.com/repos/auricvex/figo/releases/latest | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')
TARGET=x86_64-unknown-linux-gnu

curl -L -o figo.tar.gz "https://github.com/auricvex/figo/releases/download/v${VERSION}/figo-${TARGET}.tar.gz"

# Extract and install
tar -xzf figo.tar.gz
sudo mv figo /usr/local/bin/figo
sudo chmod +x /usr/local/bin/figo
```

#### macOS (Homebrew, optional)

If a Homebrew tap is published, you can install with:

```sh
brew tap auricvex/figo
brew install figo
```

> **Note:** A Homebrew tap is not yet available. Use `cargo install` or the
> pre-built binary until it is.

#### Windows (PowerShell)

```powershell
$version = (Invoke-RestMethod -Uri https://api.github.com/repos/auricvex/figo/releases/latest).tag_name
$target = "x86_64-pc-windows-msvc"

Invoke-WebRequest -Uri "https://github.com/auricvex/figo/releases/download/${version}/figo-${target}.zip" -OutFile "figo.zip"

Expand-Archive -Path "figo.zip" -DestinationPath "$env:USERPROFILE\bin"
# Ensure $env:USERPROFILE\bin is on your PATH
```

### Method 3: Build from source

Building from source is useful if you want the latest development version or
plan to contribute.

```sh
# Clone the repository
git clone https://github.com/auricvex/figo.git
cd figo

# Build an optimized release binary
cargo build --release

# The compiled binary is located at:
#   ./target/release/figo
```

You can then run it directly or install it locally:

```sh
# Run from the build directory
./target/release/figo --help

# Install into ~/.cargo/bin
cargo install --path .
```

#### Build options

- **All features:**
  ```sh
  cargo build --release --all-features
  ```

- **Run tests:**
  ```sh
  cargo test --all-features
  ```

- **Run lints:**
  ```sh
  cargo clippy --all-targets --all-features
  cargo fmt --check
  ```

### Verify the installation

After installing by any method, verify that `figo` is available:

```sh
figo --version
figo --help
```

Try rendering a quick diagram:

```sh
figo box '{"width":40,"charset":"unicode","title":"Hello","content":"World"}'
```

### Clipboard support

The `--clipboard` flag copies rendered output to the system clipboard. It works
out of the box on macOS and Windows. On Linux, the underlying `arboard` crate
may require X11 or Wayland development libraries:

- **Debian / Ubuntu:**
  ```sh
  sudo apt-get install libx11-dev libxcb1-dev
  ```

- **Fedora / RHEL:**
  ```sh
  sudo dnf install libX11-devel libxcb-devel
  ```

- **Arch Linux:**
  ```sh
  sudo pacman -S libx11 libxcb
  ```

### Updating

If you installed with `cargo install`, update to the latest release with:

```sh
cargo install figo --force
```

If you installed a pre-built binary manually, download the latest release
archive and replace the existing binary.

### Uninstalling

To remove the `cargo`-installed binary:

```sh
cargo uninstall figo
```

To remove a manually installed binary, delete the `figo` executable from the
location where you placed it (for example, `/usr/local/bin/figo`).

## CLI Usage

All subcommands accept JSON input via inline argument, file (`--file`), or stdin.

```sh
figo <subcommand> [JSON]
figo <subcommand> --file <path>
figo <subcommand>                  # reads from stdin
```

### Quick Examples

```sh
figo box '{"width":40,"charset":"unicode","title":"Hello","content":"World"}'
figo table --file my-table.json
figo banner --clipboard '{"width":60,"charset":"ascii","text":"FIGO"}'
figo sequence --output diagram.txt '{"width":80,"charset":"unicode","participants":["A","B"],"messages":[{"from":"A","to":"B","label":"msg"}]}'
```

### Global Flags

| Flag | Description |
|------|-------------|
| `--output <path>`  | Write output to file instead of stdout |
| `--clipboard`      | Copy output to the system clipboard |
| `--help`           | Show help for any subcommand |

### Subcommands

| Command     | Description |
|-------------|-------------|
| `box`       | Bordered box/container |
| `table`     | Table/grid layout |
| `flowchart` | Flowchart diagram |
| `packet`    | IETF packet header diagram |
| `tree`      | Hierarchical tree |
| `arrow`     | Arrow/connector |
| `sequence`  | Sequence diagram |
| `banner`    | FIGlet text banner |
| `gantt`     | Gantt chart |
| `state`     | FSM state machine diagram |

See [docs/cli-usage.md](docs/cli-usage.md) for full examples and JSON field
reference for every subcommand.

## Library Usage

Add `figo` to your `Cargo.toml`:

```toml
[dependencies]
figo = "0.1"
```

Each diagram type exposes both a **free function** (for simple usage) and a
**builder** (for complex configurations). All functions return
`Result<String, FigoError>`.

### Box Art

```rust
use figo::diagrams::box_art::{self, BoxArt};
use figo::style::{BorderStyle, Charset, HAlign, VAlign};

// Free function
let output = box_art::draw_box(
    Some("My Box"),
    Some("Hello, world!"),
    60,
    Charset::Unicode,
    BorderStyle::Single,
)?;

// Builder with full options
let output = BoxArt::new(60, Charset::Unicode)
    .title(Some("Configuration"))
    .content(Some("This text wraps automatically to fit inside the box."))
    .border(BorderStyle::Double)
    .padding(2, 1)
    .align(HAlign::Center, VAlign::Middle)
    .color(true)
    .build()?;
```

### Table

```rust
use figo::diagrams::table::{self, Table};
use figo::style::{Charset, HAlign};

// Free function
let output = table::draw_table(
    &["Name", "Version"],
    &[vec!["figo", "0.1.0"], vec!["serde", "1.0"]],
    60,
    Charset::Unicode,
)?;

// Builder
let output = Table::new(60, Charset::Unicode)
    .headers(&["Crate", "Version", "Description"])
    .rows(&[
        vec!["figo", "0.1.0", "ASCII art generator"],
        vec!["serde", "1.0", "Serialization framework"],
    ])
    .align(vec![HAlign::Left, HAlign::Center, HAlign::Left])
    .padding(1, 0)
    .build()?;
```

### Flowchart

```rust
use figo::diagrams::flowchart::{Flowchart, FlowNode, Layout, NodeShape};
use figo::style::Charset;

let mut fc = Flowchart::new(80, Charset::Unicode);

fc = fc
    .add_node(FlowNode {
        id: "start".into(),
        label: "Start".into(),
        shape: NodeShape::Rounded,
        position: None,
    })
    .add_node(FlowNode {
        id: "check".into(),
        label: "Valid?".into(),
        shape: NodeShape::Rounded,
        position: None,
    })
    .add_node(FlowNode {
        id: "end".into(),
        label: "End".into(),
        shape: NodeShape::Rounded,
        position: None,
    })
    .connect("start", "check", None)
    .connect("check", "end", Some("yes"));

// For manual layout, use `.layout(Layout::Manual)` and provide positions
let output = fc.build()?;
```

### Packet Header

```rust
use figo::diagrams::packet::{PacketDiagram, PacketField};
use figo::style::Charset;

let fields = vec![
    PacketField { name: "Version".into(), bits: 4 },
    PacketField { name: "IHL".into(), bits: 4 },
    PacketField { name: "Total Length".into(), bits: 16 },
];

let output = PacketDiagram::new(80, Charset::Ascii)
    .fields(&fields)
    .build()?;
```

### Tree

```rust
use figo::diagrams::tree::{self, TreeNode};
use figo::style::Charset;

let nodes = vec![
    TreeNode {
        label: "src/".into(),
        children: vec![
            TreeNode { label: "main.rs".into(), children: vec![] },
            TreeNode { label: "lib.rs".into(), children: vec![] },
        ],
    },
    TreeNode { label: "Cargo.toml".into(), children: vec![] },
];

let output = tree::draw_tree(Some("project/"), &nodes, 40, Charset::Unicode)?;
```

### Arrow

```rust
use figo::diagrams::arrow;
use figo::style::{Charset, LineStyle};

let output = arrow::draw_arrow(
    "right",
    30,
    LineStyle::Simple,
    Charset::Unicode,
    Some("data flow"),
)?;
```

### Sequence Diagram

```rust
use figo::diagrams::sequence::SequenceDiagram;
use figo::style::Charset;

let output = SequenceDiagram::new(100, Charset::Unicode)
    .add_participant("Client")
    .add_participant("Server")
    .add_participant("Database")
    .add_message("Client", "Server", "GET /api")
    .add_message("Server", "Database", "SELECT")
    .add_message("Database", "Server", "rows")
    .add_message("Server", "Client", "200 OK")
    .build()?;
```

### Banner

```rust
use figo::diagrams::banner;

let output = banner::draw_banner("FIGO", 80)?;
```

### Gantt Chart

```rust
use figo::diagrams::gantt::{GanttChart, GanttSection, GanttTask, TimeUnit};
use figo::style::Charset;

let output = GanttChart::new(100, Charset::Unicode, TimeUnit::Day, 30)
    .today_marker(15)
    .add_section(GanttSection {
        label: "Planning".into(),
        tasks: vec![
            GanttTask {
                name: "Design".into(),
                start: 0,
                duration: 7,
                progress: 100,
                milestone: false,
                depends_on: None,
            },
            GanttTask {
                name: "v1.0".into(),
                start: 24,
                duration: 0,
                progress: 0,
                milestone: true,
                depends_on: None,
            },
        ],
    })
    .build()?;
```

### State Diagram

```rust
use figo::diagrams::state::{StateDiagram, StateNode, StateType};
use figo::style::Charset;

let output = StateDiagram::new(80, Charset::Unicode)
    .add_state(StateNode {
        id: "idle".into(),
        label: "Idle".into(),
        state_type: StateType::Simple,
    })
    .add_state(StateNode {
        id: "done".into(),
        label: "Done".into(),
        state_type: StateType::Accepting,
    })
    .initial("idle")
    .add_transition("idle", "done", Some("complete"))
    .build()?;
```

See the [full API documentation on docs.rs](https://docs.rs/figo) for all
available types and methods.

## Supported Styles

### Character Sets

| Charset | Description |
|---------|-------------|
| `ascii` | 7-bit ASCII only (`-`, `|`, `+`, `>`, `<`, `^`, `v`) |
| `unicode` | Unicode box-drawing (`─│┌┐└┘├┤┬┴┼`) and arrows (`→←↑↓⇒⇐`) |

### Border Styles

| Style | Unicode | ASCII |
|-------|---------|-------|
| `single` | `┌┐└┘─│` | `+`, `-`, `\|` |
| `double` | `╔╗╚╝═║` | `+`, `=`, `‖` |
| `rounded` | `╭╮╰╯─│` | `+`, `-`, `\|` |
| `dashed` | `┌┐└┘╌╎` | `+`, `-`, `\|` (gapped) |
| `bold` | `┏┓┗┛━┃` | `+`, `━`, `┃` |

### Line / Arrow Styles

| Style | Unicode | ASCII |
|-------|---------|-------|
| `simple` | `→←↑↓` | `-->`, `<--`, `^`, `v` |
| `bold` | `⇒⇐⇑⇓` | `==>`, `<==`, `^^`, `vv` |
| `box_drawing` | `──│┼` | `---`, `\|`, `+` |

### Alignment

| Axis | Options |
|------|---------|
| Horizontal | `left`, `center`, `right` |
| Vertical | `top`, `middle`, `bottom` |

## Design Principles

- **KISS / YAGNI** — Build only what is needed. No speculative abstractions.
- **No unsafe code / no panics** in production code.
- **Single crate** with both `lib` and `bin` targets.
- **MSRV 1.85**, Edition 2024.

## Development

```sh
# Run tests
cargo test

# Run lints
cargo clippy --all-targets

# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt
```

All three checks (`clippy`, `fmt`, `test`) must pass before committing.

- **File length**: No Rust file exceeds 250 lines (tests excluded).
- **Imports**: Always use `use` statements; no fully qualified paths inline.
- **Documentation**: All public items must have doc comments.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for
details on our code of conduct and the pull request process.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE-2.0) or
[MIT License](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual-licensed as above, without any additional terms or conditions.
