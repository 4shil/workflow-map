# Architecture

## Overview

workflow-map follows a pipeline architecture: input file goes through detection, parsing, validation, and then rendering or export.

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
|   DSPy, AutoGen, |    |                     |
|   OpenAI,        |    |                     |
|   LlamaIndex)    |    |                     |
+--------+---------+    +----------+----------+
         |                         |
         +------------+------------+
                      v
+--------------------------------------------------+
|            Unified Workflow Model                 |
|  (Vec<Step>, Vec<Edge>, Metadata, Status,        |
|   Resources, Timing, GraphPattern, Validation)    |
+----------------------+---------------------------+
                       |
          +------------+------------+
          v                         v
+------------------+    +---------------------+
|  TUI Renderer    |    |  Export Engine      |
|  (ratatui:       |    |  (text, JSON, MD,   |
|   interactive    |    |   DOT, Mermaid)     |
|   ASCII diagram, |    |                     |
|   mouse, cache)  |    |                     |
+------------------+    +---------------------+
```

## Module Breakdown

### `src/model.rs` -- Core Data Types
- `Workflow` -- Top-level container with name, framework, steps, edges, metadata, graph_pattern, validation_warnings
- `Step` -- Single workflow node with id, name, type, status, children, dependencies, duration, resources
- `Edge` -- Directed connection between steps (sequential, conditional, parallel)
- `Framework` -- Enum of supported frameworks (LangChain, CrewAI, DSPy, AutoGen, Hermes, OpenAI, LlamaIndex, Generic)
- `Status` -- Step execution status (Ok, Error, Running, Waiting, Skipped)
- `StepType` -- Step classification (Agent, Tool, Chain, Task, Step, Predict, Retrieve, Lambda, Module, Conditional, Loop, ErrorHandler)
- `ResourceUsage` -- Token counts, API calls, cost tracking
- `GraphPattern` -- Detected topology (Pipeline, Diamond, FanIn, FanOut, DAG, Unknown)
- `FlatStep` -- Flattened step representation for rendering

### `src/cli.rs` -- CLI Interface
- Defines `Cli` struct with clap derive
- Arguments: path, format, output, graph, watch, framework, no-color, no-config, config, refresh, no-cache, timing-file

### `src/config.rs` -- Configuration
- `AppConfig` loaded from `~/.config/workflow-map/config.toml`
- Parser config (use_regex_parser, max_parse_errors, disable_cache)
- Render config (color_scheme, respect_no_color, status_markers)
- Watch config (debounce_ms)

### `src/parsers/` -- Parsing Engine
- `mod.rs` -- Dispatcher: routes to correct parser based on framework, manages parse cache
- `detector.rs` -- Auto-detects framework from file extension + content patterns
- `generic.rs` -- Generic JSON/YAML workflow schema parser
- `crewai.rs` -- CrewAI YAML config parser (agents, tasks, process)
- `langchain.rs` -- LangChain Python parser (pipe chains, AgentExecutor, RunnableParallel, RunnableLambda)
- `dspy.rs` -- DSPy Python parser (Module subclasses, Predict, ChainOfThought, Retrieve)
- `autogen.rs` -- AutoGen Python parser (AssistantAgent, UserProxyAgent, GroupChat)
- `hermes.rs` -- Hermes Agent SKILL.md parser (YAML frontmatter + markdown sections)
- `openai_agents.rs` -- OpenAI Agents SDK parser (Agent, Runner, handoff)
- `llamaindex.rs` -- LlamaIndex parser (QueryEngine, Workflow, StepEngine)

### `src/renderer.rs` -- TUI Renderer
- `TuiApp` struct manages all TUI state
- Handles input (keyboard + mouse), filtering, search, pagination, collapse/expand
- Renders ASCII diagram with status markers, indentation, colors
- Detail panel for selected steps with timing/resources
- Help overlay and search prompt
- Toast notifications for export and copy actions

### `src/export.rs` -- Export Engine
- `Exporter` struct with `to_text()`, `to_json()`, `to_markdown()`, `to_dot()`, `to_mermaid()` methods
- Text: ASCII diagram with status markers and summary
- JSON: Full workflow serialization via serde_json
- Markdown: Document with TOC, step details, edges, metadata, validation warnings
- DOT: Graphviz format with colored nodes and labeled edges
- Mermaid: Flowchart format for GitHub/GitLab embedding

### `src/remote.rs` -- Remote Source Support
- Fetches workflow files from HTTP/HTTPS URLs
- Caches downloaded content in `~/.cache/workflow-map/remote/`
- Supports `--refresh` flag to bypass cache

### `src/cache.rs` -- Parse Cache
- In-memory cache keyed by file path + mtime
- Avoids re-parsing unchanged files
- Thread-safe via Mutex

### `src/timing.rs` -- Timing Data
- Loads timing/resource data from CSV files
- Applies duration and resource usage to parsed steps
- Supports tokens, API calls, and cost tracking

### `src/validate.rs` -- Configuration Validation
- Per-framework validation rules
- Reports warnings for suspicious configs
- Checks for missing agents, tasks, empty workflows

## Data Flow

1. User runs `workflow-map ./config.yaml`
2. CLI parses arguments via clap
3. Config file loaded from `~/.config/workflow-map/config.toml`
4. If path is a URL, fetch and cache the remote file
5. Framework detector identifies the framework from file extension + content
6. Parse cache checked — skip if file unchanged
7. Appropriate parser produces a unified `Workflow` model
8. Timing data applied if `--timing-file` provided
9. Validation rules run, warnings collected
10. Graph pattern detected from edge topology
11. Based on format flag:
    - Interactive: TUI renderer takes over terminal, draws ASCII diagram
    - Text/JSON/Markdown/DOT/Mermaid: Exporter serializes and writes to stdout or file

## Design Decisions

### Regex vs AST Parsing
v2 uses regex-based heuristic parsing for Python files. This covers 80% of real-world patterns and is fast to implement. A future tree-sitter integration (Upgrade 1 from v2 plan) will add accurate AST parsing.

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
| Cached re-parse | < 5ms | <1ms |

## v2 Features

See individual documentation files for details:
- `GRAPH_PATTERNS.md` -- Graph pattern detection
- `RESOURCE_TRACKING.md` -- Token/cost tracking
- `CONTROL_FLOW.md` -- Conditional/loop detection
- `CACHING.md` -- Parse caching system
- `REMOTE.md` -- Remote URL support
- `TIMING.md` -- Timing data from CSV
- `VALIDATION.md` -- Configuration validation
- `EXPORT_DOT_MERMAID.md` -- DOT and Mermaid export
