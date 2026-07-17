# CLI Usage — Detailed Reference

`figo` is a Rust-based ASCII/Unicode art generator for documentation, RFC-style diagrams, flowcharts, tables, trees, banners, and more. It is designed for engineers, technical writers, and architects who need clean, version-control-friendly diagrams embedded in Markdown, READMEs, RFCs, wikis, and terminal tooling.

## Table of Contents

- [Quick Start](#quick-start)
- [Input Methods](#input-methods)
- [Global Flags](#global-flags)
- [Subcommand Reference](#subcommand-reference)
- [Use-Case Guide](#use-case-guide)
- [JSON Schema Reference](#json-schema-reference)
  - [`box`](#figo-box--bordered-container)
  - [`table`](#figo-table--grid-table)
  - [`flowchart`](#figo-flowchart--flowchart-diagram)
  - [`packet`](#figo-packet--ietf-packet-header)
  - [`tree`](#figo-tree--hierarchical-tree)
  - [`arrow`](#figo-arrow--arrow-connector)
  - [`sequence`](#figo-sequence--sequence-diagram)
  - [`banner`](#figo-banner--figlet-text-banner)
  - [`gantt`](#figo-gantt--gantt-chart)
  - [`state`](#figo-state--uml-state-diagram)
- [Tips & Best Practices](#tips--best-practices)

---

## Quick Start

```sh
# Render a simple Unicode box
figo box '{"width":40,"charset":"unicode","title":"Hello","content":"World"}'

# Render the same box in ASCII
figo box '{"width":40,"charset":"ascii","title":"Hello","content":"World"}'
```

---

## Input Methods

Every subcommand accepts JSON input in one of three ways:

1. **Inline JSON** (positional argument):
   ```sh
   figo box '{"width":40,"charset":"unicode","content":"Hello"}'
   ```

2. **`--file <path>`** to read JSON from a file:
   ```sh
   figo box --file my-box.json
   ```

3. **stdin** (omit both of the above):
   ```sh
   echo '{"width":40,"charset":"unicode","content":"Hello"}' | figo box
   ```

> **Note:** Inline JSON takes precedence over `--file`, and `--file` takes precedence over stdin.

---

## Global Flags

| Flag | Description |
|------|-------------|
| `--output <path>`  | Write output to a file instead of stdout. |
| `--clipboard`      | Copy output to the system clipboard. |
| `--help`           | Show help for any subcommand. |

---

## Subcommand Reference

| Command     | Description |
|-------------|-------------|
| `box`       | Bordered box/container. |
| `table`     | Table/grid layout. |
| `flowchart` | Flowchart diagram. |
| `packet`    | IETF/RFC packet header diagram. |
| `tree`      | Hierarchical tree. |
| `arrow`     | Arrow/connector. |
| `sequence`  | Sequence diagram. |
| `banner`    | FIGlet text banner. |
| `gantt`     | Gantt chart. |
| `state`     | UML state machine diagram. |

---

## Use-Case Guide

| Use Case | Recommended Command | Why |
|----------|-------------------|-----|
| Highlight a configuration block or note in documentation. | `box` | Adds visual emphasis with borders, titles, and padding. |
| Present tabular data such as benchmarks or dependency versions. | `table` | Auto-sized columns with per-column alignment. |
| Document an decision flow or algorithm. | `flowchart` | Supports rectangle, rounded, and diamond nodes with auto-layout. |
| Describe a network protocol header. | `packet` | RFC-style 32-bit word diagrams with bit-position scale. |
| Show a directory structure or org chart. | `tree` | Renders nested hierarchies with branch characters. |
| Indicate direction or data flow between components. | `arrow` | Configurable direction, length, style, and label. |
| Illustrate API or service interactions. | `sequence` | Participant lifelines and labeled messages. |
| Create release banners or headings. | `banner` | Large FIGlet-style text. |
| Plan sprints or project timelines. | `gantt` | Sections, tasks, progress bars, milestones, and dependencies. |
| Model state machines. | `state` | Simple, composite, initial, final, and history states. |

---

## JSON Schema Reference

Fields marked with `*` are required. All other fields are optional.

### Common Fields

Most commands accept the following common fields:

| Field | Type | Description |
|-------|------|-------------|
| `width` | `number` | Total width of the rendered diagram in characters. |
| `charset` | `"ascii" \| "unicode"` | Character set to use for rendering. |
| `color` | `boolean` | Enable ANSI color output (default: `false`). |

---

### `figo box` — Bordered Container

Renders a bordered box with optional title, content, padding, alignment, and border style. Content is auto word-wrapped to fit.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the box. Must be at least `4`. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `title` | `string` | No | Title displayed in the top border. |
| `content` | `string` | No | Body text; auto-wrapped to fit. |
| `border` | `"single" \| "double" \| "rounded" \| "dashed" \| "bold"` | No | Border style (default: `"single"`). |
| `padding` | `{ horizontal: number, vertical: number }` | No | Inner padding (default: `horizontal=1`, `vertical=0`). |
| `align` | `{ horizontal: "left"\|"center"\|"right", vertical: "top"\|"middle"\|"bottom" }` | No | Text alignment (default: top-left). |
| `color` | `boolean` | No | Enable ANSI color. |

#### Example Commands

```sh
# Simple box
figo box '{"width":40,"charset":"unicode","title":"Hello","content":"World"}'

# Double border with padding and centered content
figo box '{"width":50,"charset":"unicode","title":"Config","content":"Auto-wrapped text","border":"double","padding":{"horizontal":2,"vertical":1},"align":{"horizontal":"center","vertical":"middle"}}'

# ASCII rounded border
figo box '{"width":30,"charset":"ascii","title":"Note","content":"Plain ASCII","border":"rounded"}'
```

#### Unicode Output

```text
┌─ Hello ──────────────────────────────┐
│ World                                │
└──────────────────────────────────────┘
```

#### ASCII Output

```text
+- Hello ------------------------------+
| World                                |
+--------------------------------------+
```

---

### `figo table` — Grid / Table

Renders a table with header row, rows, configurable column alignment, and padding.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the table. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `headers` | `string[]` | No | Header row values. |
| `rows` | `string[][]` | No | Data rows. Each inner array is one row. |
| `border` | `"single" \| "double" \| "rounded" \| "dashed" \| "bold"` | No | Border style (default: `"single"`). |
| `padding` | `{ horizontal: number, vertical: number }` | No | Cell padding (default: `horizontal=1`, `vertical=0`). |
| `align` | `"left"\|"center"\|"right"[]` | No | Per-column horizontal alignment. |
| `color` | `boolean` | No | Enable ANSI color. |

#### Example Commands

```sh
# Simple table
figo table '{"width":60,"charset":"unicode","headers":["Crate","Version"],"rows":[["figo","0.1.0"],["serde","1.0"]]}'

# With column alignment
figo table '{"width":60,"charset":"ascii","headers":["Name","Qty"],"rows":[["Apples","10"],["Oranges","5"]],"align":["left","center"]}'
```

#### Unicode Output

```text
┌────────────────────────────┬─────────────────────────────┐
│ Name                       │ Version                     │
┼────────────────────────────┼─────────────────────────────┤
│ figo                       │ 0.1.0                       │
│ serde                      │ 1.0                         │
└────────────────────────────┴─────────────────────────────┘
```

#### ASCII Output

```text
+----------------------------+-----------------------------+
| Name                       | Version                     |
+----------------------------+-----------------------------+
| figo                       | 0.1.0                       |
| serde                      | 1.0                         |
+----------------------------+-----------------------------+
```

---

### `figo flowchart` — Flowchart Diagram

Renders flowchart diagrams with nodes (rectangle, rounded, diamond) and orthogonal edge routing. Supports auto-layout (default) or manual positioning.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the diagram. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `layout` | `"auto" \| "manual"` | No | Layout mode (default: `"auto"`). |
| `nodes` | `Node[]` | Yes | Array of nodes. |
| `connections` | `Connection[]` | No | Array of edges. |
| `color` | `boolean` | No | Enable ANSI color. |

**`Node` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Unique identifier. |
| `label` | `string` | Yes | Display text. |
| `shape` | `"rectangle" \| "rounded" \| "diamond"` | No | Node shape (default: `"rectangle"`). |
| `position` | `{ x: number, y: number }` | No | Required when `layout` is `"manual"`. |

**`Connection` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | `string` | Yes | Source node `id`. |
| `to` | `string` | Yes | Target node `id`. |
| `label` | `string` | No | Optional edge label. |

#### Example Commands

```sh
# Auto-layout flowchart
figo flowchart '{"width":80,"charset":"unicode","nodes":[{"id":"start","label":"Start","shape":"rounded"},{"id":"check","label":"Valid?","shape":"diamond"},{"id":"end","label":"End","shape":"rounded"}],"connections":[{"from":"start","to":"check"},{"from":"check","to":"end","label":"yes"}]}'

# Manual layout
figo flowchart '{"width":80,"charset":"ascii","layout":"manual","nodes":[{"id":"a","label":"Step 1","shape":"rectangle","position":{"x":10,"y":2}},{"id":"b","label":"Step 2","shape":"rectangle","position":{"x":10,"y":8}}],"connections":[{"from":"a","to":"b"}]}'
```

#### Unicode Output

```text
                                   ╭───────╮
                                   │ Start │
                                   ╰───────╯
                                       │
                                       │
                                       │
                                       ↓
                                       ^
                                      / \
                                     /   \
                                    /     \
                                   /Valid? \
                                    \     /
                                     \   /
                                      \ /
                                       v
                                       │
                                       │yes
                                       │
                                       ↓
                                    ╭─────╮
                                    │ End │
                                    ╰─────╯
```

#### ASCII Output

```text
                                   +-------+
                                   | Start |
                                   +-------+
                                       |
                                       |
                                       |
                                       v
                                       ^
                                      / \
                                     /   \
                                    /     \
                                   /Valid? \
                                    \     /
                                     \   /
                                      \ /
                                       v
                                       |
                                       |yes
                                       |
                                       v
                                    +-----+
                                    | End |
                                    +-----+
```

---

### `figo packet` — IETF Packet Header

Renders RFC-style protocol packet header diagrams with a 32-bit word scale.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the diagram. Must be at least `36`. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `fields` | `Field[]` | Yes | Array of bit fields. |
| `color` | `boolean` | No | Enable ANSI color. |

**`Field` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | Yes | Field label. |
| `bits` | `number` | Yes | Bit width (must be greater than `0`). |

#### Example Commands

```sh
figo packet '{"width":80,"charset":"unicode","fields":[{"name":"Version","bits":4},{"name":"IHL","bits":4},{"name":"DSCP","bits":6},{"name":"ECN","bits":2},{"name":"Total Length","bits":16}]}'

figo packet '{"width":80,"charset":"ascii","fields":[{"name":"Version","bits":4},{"name":"IHL","bits":4},{"name":"DSCP","bits":6},{"name":"ECN","bits":2},{"name":"Total Length","bits":16}]}'
```

#### Unicode Output

```text
       0       4       8      12      16      20      24      28    31
       ┌────────┬───────┬───────────┬───┬───────────────────────────────┐
       │Version │  IHL  │   DSCP    │ECN│         Total Length          │
       └────────┴───────┴───────────┴───┴───────────────────────────────┘
```

#### ASCII Output

```text
       0       4       8      12      16      20      24      28    31
       +--------+-------+-----------+---+-------------------------------+
       |Version |  IHL  |   DSCP    |ECN|         Total Length          |
       +--------+-------+-----------+---+-------------------------------+
```

---

### `figo tree` — Hierarchical Tree

Renders directory/file tree structures with branch characters.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the diagram. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `root` | `string` | No | Root label. |
| `nodes` | `TreeNode[]` | No | Top-level child nodes. |
| `color` | `boolean` | No | Enable ANSI color. |

**`TreeNode` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `string` | Yes | Node label. |
| `children` | `TreeNode[]` | No | Nested child nodes. |

#### Example Commands

```sh
figo tree '{"width":40,"charset":"unicode","root":"project/","nodes":[{"label":"src/","children":[{"label":"main.rs"},{"label":"lib.rs"}]},{"label":"Cargo.toml"}]}'

figo tree '{"width":40,"charset":"ascii","root":"project/","nodes":[{"label":"src/","children":[{"label":"main.rs"},{"label":"lib.rs"}]},{"label":"Cargo.toml"}]}'
```

#### Unicode Output

```text
project/
├── src/
│   ├── main.rs
│   └── lib.rs
└── Cargo.toml
```

#### ASCII Output

```text
project/
+-- src/
│   +-- main.rs
│   \-- lib.rs
\-- Cargo.toml
```

---

### `figo arrow` — Arrow / Connector

Renders standalone arrows with configurable direction, length, and style.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | No | Total width of the diagram. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `direction` | `"left" \| "right" \| "up" \| "down" \| "bidirectional"` | Yes | Arrow direction. |
| `length` | `number` | Yes | Length of the arrow body in characters. |
| `style` | `"simple" \| "bold" \| "box_drawing"` | No | Line style (default: `"simple"`). |
| `label` | `string` | No | Label placed above (horizontal) or beside (vertical) the arrow. |
| `color` | `boolean` | No | Enable ANSI color. |

#### Example Commands

```sh
# Unicode arrow with label
figo arrow '{"width":40,"charset":"unicode","direction":"right","length":20,"style":"simple","label":"data flow"}'

# ASCII bold bidirectional arrow
figo arrow '{"width":40,"charset":"ascii","direction":"bidirectional","length":10,"style":"bold","label":"sync"}'
```

#### Unicode Output

```text
      data flow
────────────────────→
```

#### ASCII Output

```text
      data flow
-------------------->
```

---

### `figo sequence` — Sequence Diagram

Renders timeline-based message sequence diagrams with participant columns and message arrows.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the diagram. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `participants` | `string[]` | Yes | List of participant names. |
| `messages` | `Message[]` | Yes | Ordered list of messages. |
| `color` | `boolean` | No | Enable ANSI color. |

**`Message` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | `string` | Yes | Source participant. |
| `to` | `string` | Yes | Target participant. |
| `label` | `string` | Yes | Message label. |

#### Example Commands

```sh
figo sequence '{"width":80,"charset":"unicode","participants":["Client","Server","DB"],"messages":[{"from":"Client","to":"Server","label":"GET /api"},{"from":"Server","to":"DB","label":"SELECT"},{"from":"DB","to":"Server","label":"rows"},{"from":"Server","to":"Client","label":"200 OK"}]}'

figo sequence '{"width":80,"charset":"ascii","participants":["Client","Server","DB"],"messages":[{"from":"Client","to":"Server","label":"GET /api"},{"from":"Server","to":"DB","label":"SELECT"},{"from":"DB","to":"Server","label":"rows"},{"from":"Server","to":"Client","label":"200 OK"}]}'
```

#### Unicode Output

```text
 ╭──────────────────────╮  ╭──────────────────────╮  ╭──────────────────────╮
 │        Client        │  │        Server        │  │          DB          │
 ╰──────────┴───────────╯  ╰──────────┴───────────╯  ╰──────────┴───────────╯
            │        GET /api         │                         │
            ├────────────────────────▶│                         │
            │                         │                         │
            │                         │         SELECT          │
            │                         ├────────────────────────▶│
            │                         │                         │
            │                         │          rows           │
            │                         │◀────────────────────────┤
            │                         │                         │
            │         200 OK          │                         │
            │◀────────────────────────┤                         │
            │                         │                         │
            │                         │                         │
```

#### ASCII Output

```text
 +----------------------+  +----------------------+  +----------------------+
 |        Client        |  |        Server        |  |          DB          |
 +----------+-----------+  +----------+-----------+  +----------+-----------+
            |        GET /api         |                         |
            +------------------------>|                         |
            |                         |                         |
            |                         |         SELECT          |
            |                         +------------------------>|
            |                         |                         |
            |                         |          rows           |
            |                         |<------------------------+
            |                         |                         |
            |         200 OK          |                         |
            |<------------------------+                         |
            |                         |                         |
            |                         |                         |
```

---

### `figo banner` — FIGlet Text Banner

Renders large stylized text using a FIGlet bitmap font.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the banner. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set (currently rendered as ASCII art). |
| `text` | `string` | Yes | Text to render. |
| `color` | `boolean` | No | Enable ANSI color. |

#### Example Commands

```sh
figo banner '{"width":80,"charset":"ascii","text":"FIGO"}'
```

#### ASCII Output

```text
 _____  ___    ____   ___
|  ___||_ _|  / ___| / _ \
| |_    | |  | |  _ | | | |
|  _|   | |  | |_| || |_| |
|_|    |___|  \____| \___/
```

---

### `figo gantt` — Gantt Chart

Renders project management Gantt charts with sections, tasks, progress bars, milestones, and dependencies.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the chart. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `time_unit` | `"hour" \| "day" \| "week" \| "month"` | Yes | Time scale unit. |
| `total_units` | `number` | No | Total number of time units (default: `30`). |
| `today_marker` | `number` | No | Position of the "today" vertical marker. |
| `sections` | `Section[]` | Yes | Array of sections. |
| `color` | `boolean` | No | Enable ANSI color. |

**`Section` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `label` | `string` | Yes | Section name. |
| `tasks` | `Task[]` | Yes | Tasks in this section. |

**`Task` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | Yes | Task name. |
| `start` | `number` | Yes | Start offset in time units. |
| `duration` | `number` | Yes | Duration in time units. Use `0` for milestones. |
| `progress` | `number` | No | Completion percentage (`0`–`100`). |
| `milestone` | `boolean` | No | Render as a milestone diamond. |
| `depends_on` | `string` | No | Name of the task this task depends on. |

#### Example Commands

```sh
figo gantt '{"width":80,"charset":"unicode","time_unit":"day","total_units":14,"today_marker":7,"sections":[{"label":"Sprint 1","tasks":[{"name":"Design","start":0,"duration":5,"progress":100},{"name":"Build","start":5,"duration":7,"progress":50}]}]}'

figo gantt '{"width":80,"charset":"ascii","time_unit":"day","total_units":14,"today_marker":7,"sections":[{"label":"Sprint 1","tasks":[{"name":"Design","start":0,"duration":5,"progress":100},{"name":"Build","start":5,"duration":7,"progress":50}]}]}'
```

#### Unicode Output

```text
┌────────────────────┬───────────────────────────────────────────────────────┐
│                    │0   1   2   3   4   5   6   7   8   9   10  11  12  13 │
│                    │────────────────────────────│──────────────────────────│
│Sprint 1            │                            │                          │
│  Design            │████████████████████        │                          │
│  Build             │                    ██████████████░░░░░░░░░░░░░░       │
└────────────────────┴───────────────────────────────────────────────────────┘
```

#### ASCII Output

```text
+--------------------+-------------------------------------------------------+
|                    |0   1   2   3   4   5   6   7   8   9   10  11  12  13 |
|                    |----------------------------|--------------------------|
|Sprint 1            |                            |                          |
|  Design            |####################        |                          |
|  Build             |                    ##############..............       |
+--------------------+-------------------------------------------------------+
```

---

### `figo state` — UML State Diagram

Renders UML state machine diagrams with states, transitions, initial/final pseudostates, and composite states.

#### JSON Schema

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `width` | `number` | Yes | Total width of the diagram. |
| `charset` | `"ascii" \| "unicode"` | Yes | Character set. |
| `states` | `State[]` | Yes | Array of states. |
| `initial` | `string` | No | `id` of the initial state. |
| `transitions` | `Transition[]` | No | Array of transitions. |
| `color` | `boolean` | No | Enable ANSI color. |

**`State` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `string` | Yes | Unique identifier. |
| `label` | `string` | Yes | Display label. |
| `type` | `"simple" \| "composite" \| "initial" \| "final" \| "history"` | No | State type (default: `"simple"`). |
| `row` | `number` | No | Grid row for explicit layout. |
| `col` | `number` | No | Grid column for explicit layout. |
| `children` | `State[]` | No | Nested states for composite states. |

**`Transition` schema:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `from` | `string` | Yes | Source state `id`. |
| `to` | `string` | Yes | Target state `id`. |
| `label` | `string` | No | Transition label. |

#### Example Commands

```sh
figo state '{"width":80,"charset":"unicode","states":[{"id":"idle","label":"Idle","type":"simple"},{"id":"running","label":"Running","type":"simple"},{"id":"done","label":"Done","type":"final"}],"initial":"idle","transitions":[{"from":"idle","to":"running","label":"start"},{"from":"running","to":"done","label":"finish"},{"from":"running","to":"idle","label":"pause"}]}'

figo state '{"width":80,"charset":"ascii","states":[{"id":"idle","label":"Idle","type":"simple"},{"id":"running","label":"Running","type":"simple"},{"id":"done","label":"Done","type":"final"}],"initial":"idle","transitions":[{"from":"idle","to":"running","label":"start"},{"from":"running","to":"done","label":"finish"},{"from":"running","to":"idle","label":"pause"}]}'
```

#### Unicode Output

```text
                      pause

                      start         finish
               ───────────────────────────────
        ╭──────▼─────╮    ╭──────▼─────╮    ╭▼╮
  ●────>│    Idle    │    │  Running   │    │◎│
        ╰────────────╯    ╰────────────╯    ╰─╯
```

#### ASCII Output

```text
                      pause

                      start         finish
               -------------------------------
        +------v-----+    +------v-----+    +v+
  *────>|    Idle    |    |  Running   |    |O|
        +------------+    +------------+    +-+
```

---

## Tips & Best Practices

- **Always set `charset` explicitly.** ASCII output is safer for plain-text environments; Unicode looks best in modern terminals.
- **Use `--output` for CI/CD.** Generate diagrams into files that can be embedded in documentation.
- **Prefer `--file` for complex JSON.** Large payloads are easier to maintain in `.json` files.
- **Validate JSON first.** `figo` returns a clear error message when required fields are missing or malformed.
- **Match `width` to your target medium.** READMEs and wikis often render best around `80` characters.
- **Use `color: true` sparingly.** ANSI codes may not render in all Markdown viewers.

---

## Using `--file` and stdin

```sh
# From file
figo table --file my-table.json

# From stdin
echo '{"width":40,"charset":"unicode","root":"/","nodes":[]}' | figo tree

# With output options
figo box --output result.txt '{"width":30,"charset":"ascii","title":"Saved"}'
figo banner --clipboard '{"width":60,"charset":"ascii","text":"RELEASE"}'
```
