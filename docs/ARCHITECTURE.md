# Architecture

This document describes the internal architecture of workflow-map, including
module breakdown, data flow, and key design decisions.

## High-Level Architecture

    +----------------------------------------------------------+
    |                        CLI (clap)                        |
    |  Parses flags: --format, --framework, --watch, --output  |
    +----------------------------+-----------------------------+
                                 |
                                 v
    +----------------------------------------------------------+
    |                      Config (config.rs)                  |
    |  AppConfig, ParserConfig, RenderConfig, StatusMarkers    |
    |  Loaded from TOML file or defaults                       |
    +----------------------------+-----------------------------+
                                 |
                                 v
    +----------------------------------------------------------+
    |                   Detector (detector.rs)                 |
    |  Inspects file extension + content to determine          |
    |  the framework: LangChain, CrewAI, DSPy, AutoGen, etc.   |
    +----------------------------+-----------------------------+
                                 |
                                 v
    +----------------------------------------------------------+
    |                   Parsers (parsers/*)                     |
    |  Framework-specific parsing logic:                       |
    |                                                          |
    |  +-----------+  +--------+  +-----+  +--------+  +-----+ |
    │  │ LangChain │  │ CrewAI │  │ DSPy│  │AutoGen │  │Hermes│
    │  └─────┬─────┘  └───┬────┘  └──┬──┘  └───┬────┘  └──┬──┘ |
    │        │             │          │          │           |    |
    │        v             v          v          v           v    |
    │  +-----------------------------------------------------+  |
    │  │          Unified Workflow Model (model.rs)          |  │
    │  │  Workflow { name, framework, steps[], edges[], ... }|  │
    │  └-----------------------------------------------------+  │
    +----------------------------+-----------------------------+
                                 |
                    +------------+------------+
                    |                         |
                    v                         v
    +---------------------------+ +---------------------------+
    |   Renderer (renderer.rs)  | |   Export (export.rs)      |
    |   Interactive TUI via     | |   to_text()               |
    |   ratatui + crossterm     | |   to_json()               |
    |                           | |   to_markdown()           |
    |   - Scroll, filter, search| |                           |
    |   - Color-coded status    | |   Writes to stdout or     |
    |   - Detail panel          | |   file (--output)         |
    +---------------------------+ +---------------------------+

## Module Breakdown

### `model.rs` -- Core Data Model

Defines all shared types used throughout the application.

- `Framework` -- Enum: LangChain, CrewAI, DSPy, AutoGen, Hermes, Generic, Unknown
- `StepType` -- Enum: Agent, Tool, Chain, Task, Step, Predict, Retrieve, Lambda, Module
- `Status` -- Enum: Ok, Error, Running, Waiting, Skipped
- `EdgeType` -- Enum: Sequential, Conditional, Parallel
- `Step` -- A single node: id, name, type, status, children, dependencies, collapse state
- `Edge` -- A directed connection between two steps
- `Workflow` -- The unified graph: name, source path, framework, steps, edges, metadata
- `FlatStep` -- Flattened step representation for rendering (includes depth, has_children)
- `SearchFilter` -- Name-based text filter
- `FilterMode` -- Display filter: All, FailedOnly, RunningOnly
- `SourceLocation` -- File path + line number + optional column
- `ErrorDetail` -- Message, stack trace suggestion

All types implement `Serialize`/`Deserialize` for JSON export.

### `parsers/` -- Framework-Specific Parsing

Each parser module reads raw file text and produces a `Workflow` model.

**`mod.rs`** -- Framework dispatch and directory parsing.
- `parse_workflow()` -- Entry point; dispatches to `parse_file` or `parse_directory`.
- `parse_file()` -- Routes to the correct framework parser.
- `parse_directory()` -- Walks a directory, parsing each file and merging results.
- `parse_framework_name()` -- String-to-enum conversion for CLI flag.

**`detector.rs`** -- Auto-detection of framework from file extension and content.
- Uses file extension as the primary signal (`.py`, `.yaml`, `.json`, `.md`).
- Content-based heuristics as fallback (e.g., `langchain` imports, `crewai` keys).
- Returns `Framework::Unknown` if no match.

**`langchain.rs`** -- Regex-based Python parser for LangChain scripts.
- Detects: `AgentExecutor`, `LLMChain`, `PromptTemplate`, `RunnableParallel`, etc.
- Extracts step names, types, and source locations from AST-like patterns.

**`crewai.rs`** -- Serde-based YAML parser for CrewAI configurations.
- Parses `agents`, `tasks`, and workflow definitions from YAML structure.
- Maps CrewAI concepts to generic Step/Edge model.

**`dspy.rs`** -- Regex-based Python parser for DSPy programs.
- Detects: `Predict`, `Retrieve`, `Module` classes and function calls.
- Captures RAG pipelines and chained compositions.

**`autogen.py`** -- Regex-based Python parser for AutoGen scripts.
- Detects: `AssistantAgent`, `UserProxyAgent`, `GroupChat`, etc.
- Extracts agent definitions and conversation patterns.

**`hermes.rs`** -- Custom parser for Hermes Agent SKILL.md files.
- Parses Markdown frontmatter (YAML between `---` delimiters).
- Extracts skill name, tools, and workflow steps.

**`generic.rs`** -- Generic YAML/JSON parser for simple workflow definitions.
- Accepts flat or hierarchical step definitions.
- Used as fallback or when `--framework generic` is specified.

### `renderer.rs` -- Interactive TUI

Implements the interactive terminal user interface using `ratatui` and `crossterm`.

**`TuiApp`** -- Application state:
- The `Workflow` model and `AppConfig`.
- Scroll offset, cursor position, filter mode, search query.
- Pagination state.

**Input handling**:
- `j`/`k` or arrow keys -- Navigate steps.
- `g`/`G` -- Jump to first/last step.
- `f` -- Cycle filter mode (all -> failed -> running).
- `/` -- Enter search mode.
- `Enter` -- Toggle step collapse.
- `q` -- Quit.

**Layout**:
- Title bar: workflow name, framework, step count.
- Main area: scrollable step list with tree-style indentation.
- Status bar: filter mode, position, error summary.
- Detail panel (future): selected step details + config snippet.

**Rendering**:
- Color-coded by status (green=OK, red=Error, yellow=Waiting, blue=Running).
- Respects `NO_COLOR` environment variable and `--no-color` flag.
- Unicode box-drawing characters for tree connectors.
- Collapsed steps show a `+` prefix; expanded show `-`.

### `export.rs` -- Static Output

Produces non-interactive output in three formats:

- `to_text()` -- Plain ASCII diagram with status indicators and source locations.
- `to_json()` -- Full JSON serialization of the `Workflow` model.
- `to_markdown()` -- Markdown document with step list and metadata.

All three flatten the workflow tree and include status, type, and source location
for each step. Output is written to stdout or to a file via `--output`.

### `config.rs` -- Configuration

Manages application configuration from TOML files.

- `AppConfig` -- Top-level config (parser + render sections).
- `ParserConfig` -- Parser behavior (use_regex_parser, max_parse_errors).
- `RenderConfig` -- Rendering options (color_scheme, respect_no_color, status_markers).
- `StatusMarkers` -- Customizable per-status display strings.
- `AppConfig::load()` -- Searches standard paths:
  - `./.workflow-map.toml`
  - `~/.config/workflow-map/config.toml`
  - Or custom path from `--config`.

### `cli.rs` -- Command-Line Interface

Defines CLI arguments and flags using `clap`'s derive API.

- `Cli` struct with `path`, `format`, `output`, `watch`, `framework`, `no_color`, `no_config`, `config`.
- `OutputFormat` enum: Interactive, Text, Json, Markdown.

## Data Flow

The end-to-end data flow for a single invocation:

    1. User runs:  workflow-map ./pipeline.yaml --format text

    2. cli.rs:     Cli::parse() extracts path="./pipeline.yaml", format=Text.

    3. config.rs:  AppConfig::load() reads defaults or TOML config file.

    4. detector.rs:detect_framework(&config, &path)
                  - File extension .yaml -> likely CrewAI or Generic
                  - Content scan for "agents:", "tasks:" keys -> CrewAI
                  - Returns Framework::CrewAI

    5. parsers/mod.rs: parse_workflow(CrewAI, &config, &path)
       parsers/crewai.rs: CrewAI YAML -> Workflow model
                  - serde_yaml parses YAML structure
                  - Each task becomes a Step { id, name, type=Task, ... }
                  - Task dependencies become Edge { from, to }
                  - Returns Workflow { name, framework=CrewAI, steps, edges }

    6. export.rs:  Exporter::new(workflow).to_text()
                  - Flattens the workflow tree
                  - Formats each step with status marker, name, type, source
                  - Returns a String

    7. main.rs:    Prints the string to stdout.

## Design Decisions

### Regex vs. AST Parsing

**Decision:** Use regex-based parsing for Python frameworks (LangChain, DSPy,
AutoGen) and serde-based parsing for structured formats (YAML, JSON).

**Rationale:**
- Full AST parsing (e.g., via `tree-sitter` or `syn`) adds significant
  dependencies and compile times. Regex is sufficient for extracting top-level
  class/function definitions from workflow scripts.
- Workflow scripts are typically declarative configurations, not arbitrary
  code. The patterns we need to match (class instantiation, function calls) are
  predictable.
- For tree-sitter support, see Future Improvements below.

**Tradeoff:** Regex cannot handle all edge cases (e.g., dynamically constructed
workflows, nested lambdas). These are reported as parse warnings rather than
errors, allowing partial results.

### Why Rust?

**Decision:** Implement the tool in Rust.

**Rationale:**
- **Performance:** Near-instant parsing and rendering, even for large workflow
  files. No garbage collection pauses in the TUI.
- **Safety:** Compile-time guarantees prevent null pointer dereferences, data
  races, and buffer overflows -- critical for a tool that processes untrusted
  input files.
- **Distribution:** Single static binary with no runtime dependency. Easy
  `cargo install` experience.
- **Ecosystem:** Excellent libraries for the domain:
  `clap` (CLI), `ratatui` (TUI), `serde` (serialization), `regex` (parsing),
  `notify` (file watching).

### Why ratatui?

**Decision:** Use `ratatui` for the interactive terminal UI.

**Rationale:**
- Declarative layout system (similar to Elm/React) makes complex terminal UIs
  maintainable.
- First-class support for colors, styles, and Unicode in the terminal.
- Active development and good documentation.
- Alternatives considered:
  - `cursive` -- Heavier, opinionated, slower release cycle.
  - Raw `crossterm` -- More control but significantly more boilerplate.
  - `--watch` mode with external tools -- Fragile; built-in file watching via
    `notify` is more reliable.

## Future Improvements

### Tree-Sitter Integration

Replace regex parsers with tree-sitter grammars for Python and other languages.
This would enable:
- Accurate parsing of nested expressions and multi-line configurations.
- Extraction of dynamically constructed workflows.
- Better error recovery and partial parsing.

Planned as an optional feature flag (`--features tree-sitter`) to avoid forcing
the dependency on all users.

### Plugin System

Allow users to add custom parsers without modifying the core codebase.

- Define a `ParserPlugin` trait with `detect()` and `sign()` methods.
- Load plugins from a config directory or as shared libraries (`.so`/`.dll`).
- Plugins declare which file types they handle and return standard `Workflow`
  models.

### Runtime Status Files

Extend the tool to read runtime status files (e.g., `workflow-status.json`)
produced by workflow execution engines, enabling:
- Live status display: show which steps are running, completed, or failed.
- Error propagation: display actual error messages from execution.
- Historical view: replay past workflow runs from status logs.

### Additional Export Formats

- **DOT/Graphviz** -- For visual diagram generation.
- **Mermaid** -- For embedding workflow diagrams in Markdown docs.
- **HTML** -- Self-contained interactive HTML with collapsible tree.

### Multi-File Project Support

- Detect workflow projects (directories with a `workflow-map.toml` manifest).
- Parse all related files as a single project with cross-file references.
- Show inter-file dependencies in the diagram.

### Remote Source Parsing

- Fetch workflow configs from URLs (GitHub raw, HTTP endpoints).
- Parse remote LangChain/CrewAI scripts without downloading them first.
- Support for private repos via environment-based authentication.
