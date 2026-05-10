# Architecture

## Overview

workflow-map follows a pipeline architecture: input file goes through detection, parsing, and then rendering or export.

```
+--------------------------------------------------+
|                   CLI Interface                   |
|              (clap argument parsing)              |
+----------------------+---------------------------+
                       |
                       v
+--------------------------------------------------+
|               Framework Detector                  |
|  (file extension + content pattern matching)      |
+----------------------+---------------------------+
                       |
          +------------+------------+
          v                         v
+------------------+    +---------------------+
|  Python Parsers  |    |  YAML/JSON Parsers  |
|  (LangChain,     |    |  (CrewAI, Generic)  |
|   DSPy, AutoGen) |    |                     |
+--------+---------+    +----------+----------+
         |                         |
         +------------+------------+
                      v
+--------------------------------------------------+
|            Unified Workflow Model                 |
|  (Vec<Step>, Vec<Edge>, Metadata, Status)        |
+----------------------+---------------------------+
                       |
          +------------+------------+
          v                         v
+------------------+    +---------------------+
|  TUI Renderer    |    |  Export Engine      |
|  (ratatui:       |    |  (text, JSON, MD)   |
|   interactive    |    |                     |
|   ASCII diagram) |    |                     |
+------------------+    +---------------------+
```

## Module Breakdown

### `src/model.rs` -- Core Data Types
- `Workflow` -- Top-level container with name, framework, steps, edges, metadata
- `Step` -- Single workflow node with id, name, type, status, children, dependencies
- `Edge` -- Directed connection between steps (sequential, conditional, parallel)
- `Framework` -- Enum of supported frameworks (LangChain, CrewAI, DSPy, AutoGen, Hermes, Generic)
- `Status` -- Step execution status (Ok, Error, Running, Waiting, Skipped)
- `FlatStep` -- Flattened step representation for rendering

### `src/cli.rs` -- CLI Interface
- Defines `Cli` struct with clap derive
- Arguments: path, format, output, watch, framework, no-color, no-config, config

### `src/config.rs` -- Configuration
- `AppConfig` loaded from `~/.config/workflow-map/config.toml`
- Parser config, render config, watch config, status markers

### `src/parsers/` -- Parsing Engine
- `mod.rs` -- Dispatcher: routes to correct parser based on framework
- `detector.rs` -- Auto-detects framework from file extension + content patterns
- `generic.rs` -- Generic JSON/YAML workflow schema parser
- `crewai.rs` -- CrewAI YAML config parser (agents, tasks, process)
- `langchain.rs` -- LangChain Python parser (pipe chains, AgentExecutor, RunnableParallel)
- `dspy.rs` -- DSPy Python parser (Module subclasses, Predict, ChainOfThought)
- `autogen.rs` -- AutoGen Python parser (AssistantAgent, UserProxyAgent, GroupChat)
- `hermes.rs` -- Hermes Agent SKILL.md parser (YAML frontmatter + markdown sections)

### `src/renderer.rs` -- TUI Renderer
- `TuiApp` struct manages all TUI state
- Handles input, filtering, search, pagination, collapse/expand
- Renders ASCII diagram with status markers, indentation, colors
- Detail panel for selected steps
- Help overlay and search prompt

### `src/export.rs` -- Export Engine
- `Exporter` struct with `to_text()`, `to_json()`, `to_markdown()` methods
- Text: ASCII diagram with status markers and summary
- JSON: Full workflow serialization via serde_json
- Markdown: Document with TOC, step details, edges, metadata

## Data Flow

1. User runs `workflow-map ./config.yaml`
2. CLI parses arguments via clap
3. Config file loaded from `~/.config/workflow-map/config.toml`
4. Framework detector identifies the framework from file extension + content
5. Appropriate parser produces a unified `Workflow` model
6. Based on format flag:
   - Interactive: TUI renderer takes over terminal, draws ASCII diagram
   - Text/JSON/Markdown: Exporter serializes and writes to stdout or file

## Design Decisions

### Regex vs AST Parsing
v1 uses regex-based heuristic parsing for Python files. This covers 80% of real-world patterns and is fast to implement. v1.1 will add tree-sitter for accurate AST parsing.

### Why Rust
- Single static binary with zero runtime dependencies
- Fast parsing and rendering (< 2s for 200 steps)
- Excellent TUI libraries (ratatui, crossterm)
- Strong type safety catches bugs at compile time

### Why ratatui
- Pure Rust, actively maintained
- Flexible layout system
- Good performance for terminal rendering
- Cross-platform (Linux, macOS, Windows)

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Parse + render (50 steps) | < 500ms | ~50ms |
| Parse + render (200 steps) | < 2s | ~200ms |
| Memory (200 steps) | < 10MB | ~2MB |
| Binary size | < 15MB | ~8MB |
| Cold startup | < 100ms | ~20ms |

## Future Improvements

1. **tree-sitter parsing** -- Accurate Python AST parsing for complex LangChain/DSPy/AutoGen code
2. **Plugin system** -- Custom parsers for proprietary frameworks
3. **Runtime status** -- Read execution status from log files or framework outputs
4. **Remote sources** -- Parse configs from GitHub URLs
5. **Additional export formats** -- DOT (Graphviz), Mermaid, HTML
6. **Multi-file projects** -- Parse entire agent project directories
7. **Configuration validation** -- Validate workflow configs against framework schemas
