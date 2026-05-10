# workflow-map

```
  _    _            _       _   _      __  __
 | |  | |          | |     | | | |    |  \/  |
 | |  | | ___  _ __| | __  | |_| | __ | \  / | __ _ _ __
 | |  | |/ _ \| '__| |/ /  |  _  |/ _` | |\/| |/ _` | '_ \
 | |__| | (_) | |  |   <   | | | | (_| | |  | | (_| | |_) |
  \___/ \___/|_|  |_|\_\  |_| |_|\__,_|_|  |_|\__,_| .__/
                                                    | |
                                                    |_|
         Agentic Workflow ASCII Visualizer
```

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Build](https://github.com/4shil/workflow-map/actions/workflows/ci.yml/badge.svg)](https://github.com/4shil/workflow-map/actions)

**Parse agentic workflow configs and render interactive ASCII diagrams in your terminal.**

Supports LangChain, CrewAI, DSPy, AutoGen, and Hermes Agent out of the box. Point it at a config file, get an interactive workflow map in under 2 seconds. No browser. No cloud. Works over SSH, in tmux, in any ANSI terminal.

## Quick Start

```bash
# Install from source
git clone https://github.com/4shil/workflow-map.git
cd workflow-map
cargo build --release
./target/release/workflow-map ./my_workflow.yaml

# Interactive TUI (default)
workflow-map ./crewai_config.yaml

# Static text output
workflow-map ./pipeline.py --format text

# JSON export
workflow-map ./dspy_program.py --format json --output workflow.json

# Graphviz DOT export
workflow-map ./workflow.yaml --format json --graph dot --output workflow.dot

# Mermaid export
workflow-map ./workflow.yaml --format json --graph mermaid --output workflow.mmd

# Watch mode
workflow-map ./my_workflow.yaml --watch
```

## Supported Frameworks

| Framework | File Type | Parser | Status |
|-----------|-----------|--------|--------|
| LangChain | `.py` | Regex-based | Stable |
| CrewAI | `.yaml` | YAML schema | Stable |
| DSPy | `.py` | Regex-based | Stable |
| AutoGen | `.py` | Regex-based | Stable |
| Hermes Agent | `SKILL.md` | YAML frontmatter + markdown | Stable |
| Generic | `.yaml`, `.json` | Standard schema | Stable |

## Features

- **Interactive TUI** -- Navigate workflows with arrow keys, search, filter, collapse/expand
- **Multi-framework** -- Parse 6+ agentic frameworks, mix them in a single directory
- **Export** -- Plain text, JSON, Markdown output
- **Watch mode** -- Auto-reload on file changes with debounce
- **Zero dependencies** -- Single static binary, no runtime requirements
- **Terminal-native** -- Works over SSH, in tmux/screen, any ANSI terminal

## Installation

### From source
```bash
cargo build --release
# Binary at ./target/release/workflow-map
```

### Crates.io (coming soon)
```bash
cargo install workflow-map
```

## Usage

```
workflow-map [OPTIONS] <PATH>

Arguments:
  <PATH>    File or directory to parse

Options:
  -f, --format <FORMAT>     Output format: interactive, text, json, md [default: interactive]
  -o, --output <PATH>       Output file path [default: stdout]
  -w, --watch               Watch mode: re-render on file changes
  -F, --framework <NAME>    Force framework: langchain, crewai, dspy, autogen, hermes, generic
      --no-color            Disable colored output
      --no-config           Ignore config file
      --config <PATH>       Custom config file path
  -h, --help                Print help
  -V, --version             Print version
```

## Example Output

```
Workflow: data-pipeline (CrewAI)          Step 3/12    Filter: all
============================================================

  [OK]  1. Load CSV
  [OK]  2. Validate schema
  [ERR] *3. Clean data                    <-- cursor
  [WAIT] 4. Transform
  [WAIT] 5. Aggregate
  [WAIT] 6. Generate report
  [SKIP] 7. Upload to S3

  -----------------------------------------------------------
  Step 3: Clean data
  Type: Task          Source: crewai_config.yaml:42
  Error: ValueError: Column 'price' has 23 null values
  Suggestion: Add a null-handling step before cleaning
  -----------------------------------------------------------

  [?] Help  [f] Filter  [/] Search  [e] Export  [r] Reload  [q] Quit
```

## Configuration

Config file at `~/.config/workflow-map/config.toml`:

```toml
[parsers]
use_regex_parser = true
max_parse_errors = 50

[render]
color_scheme = "default"
respect_no_color = true

[watch]
debounce_ms = 500

default_export_format = "text"
```

## Building from source

Requires Rust 1.75+.

```bash
git clone https://github.com/4shil/workflow-map.git
cd workflow-map

# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings
```

## License

MIT License. See [LICENSE](LICENSE) for details.
