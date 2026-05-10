  _______________________________________________
 /                                               \
|      _       __                 __  ___        |
|     | |     / /___ __________  /  |/  /        |
|     | | /| / / __ `/ ___/ _ \/ /|_/ /         |
|     | |/ |/ / /_/ / /  /  __/ /  / /          |
|     |__/|__/\__,_/_/   \___/_/  /_/           |
|                                               |
|      Agentic Workflow Visualizer              |
 \_______________________________________________/

# workflow-map

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/workflow-map.svg)](https://crates.io/crates/workflow-map)
[![Build Status](https://img.shields.io/github/actions/workflow/status/4shil/workflow-map/ci.yml?branch=main)](https://github.com/4shil/workflow-map/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

Terminal-native agentic workflow visualizer. Parse LangChain, CrewAI, DSPy,
AutoGen, and Hermes Agent configurations and render interactive ASCII workflow
maps directly in your terminal.

## Screenshots

```
Workflow: data_pipeline (CrewAI)
============================================================
[WAIT] 1. extract_data     Task     src/pipeline.yaml:12
[WAIT] 2. transform_data   Task     src/pipeline.yaml:24
[WAIT] 3. load_data        Task     src/pipeline.yaml:38
[WAIT] 4. validate         Task     src/pipeline.yaml:51
------------------------------------------------------------
Steps: 4 | Errors: 0 | Filter: all
j/k: navigate  f: filter  /: search  q: quit
```

## Installation

### From crates.io

    cargo install workflow-map

### From Source

    git clone https://github.com/4shil/workflow-map.git
    cd workflow-map
    cargo build --release

The binary will be at `target/release/workflow-map`.

## Quick Start

    # Interactive TUI mode (default)
    workflow-map ./my_workflow.yaml

    # Parse a LangChain Python script
    workflow-map ./chain.py --framework langchain

    # Export as plain text
    workflow-map ./pipeline.yaml --format text

    # Export as JSON
    workflow-map ./agents.py --format json --output agents.json

    # Watch mode: re-render on file changes
    workflow-map ./workflow.yaml --watch

## Supported Frameworks

| Framework    | File Type      | Parser    | Status       |
|--------------|----------------|-----------|--------------|
| LangChain    | .py            | regex     | stable       |
| CrewAI       | .yaml          | serde_yaml| stable       |
| DSPy         | .py            | regex     | stable       |
| AutoGen      | .py            | regex     | stable       |
| Hermes Agent | .md (SKILL.md) | custom    | stable       |
| Generic      | .yaml, .json   | serde     | stable       |

## Usage

    workflow-map <PATH> [OPTIONS]

### Arguments

- `<PATH>` -- File or directory to parse.

### Options

| Flag            | Description                                      |
|-----------------|--------------------------------------------------|
| `-f, --format`  | Output format: interactive, text, json, markdown |
| `-o, --output`  | Output file path (default: stdout)               |
| `-w, --watch`   | Watch mode: re-render on file changes            |
| `-F, --framework`| Force: langchain, crewai, dspy, autogen, hermes, generic |
| `--no-color`    | Disable colored output                           |
| `--no-config`   | Ignore config file                               |
| `--config PATH` | Custom config file path                          |

### Examples

    # Force CrewAI parsing
    workflow-map ./config.yaml --framework crewai

    # Markdown export to file
    workflow-map ./workflow.yaml --format markdown --output diagram.md

    # Use custom config
    workflow-map ./workflow.yaml --config ./my_config.toml

## Building from Source

Requires Rust 1.75 or later.

    cargo build --release          # Optimized binary
    cargo test                     # Run all tests
    cargo clippy                   # Lint
    cargo fmt --check              # Check formatting

## License

MIT License. See [LICENSE](LICENSE) for details.
