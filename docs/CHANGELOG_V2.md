# v2 Changelog

All notable changes in workflow-map v2.

## v2.0.0 -- 2026-05-09

### New Features

**Parsing**
- OpenAI Agents SDK parser (`src/parsers/openai_agents.rs`)
- LlamaIndex parser scaffold (`src/parsers/llamaindex.rs`)
- Conditional/loop/error-handler detection in Python parsers
- Accurate line number tracking across all parsers
- Framework detection for OpenAI and LlamaIndex

**Data Model**
- `ResourceUsage` struct for token/cost tracking
- `Duration` field on steps for timing data
- `GraphPattern` enum for topology classification
- `ValidationWarning` system for config checking

**TUI**
- Mouse support (click to select, scroll to navigate)
- Export toast notifications in status bar
- Timing and resource display in detail panel

**Export**
- DOT/Graphviz format (`--graph dot`)
- Mermaid diagram format (`--graph mermaid`)
- Validation warnings in Markdown export
- Resource/timing data in all export formats

**Remote + Cache**
- Remote URL parsing with local cache (`src/remote.rs`)
- In-memory parse caching (`src/cache.rs`)
- `--refresh` flag to bypass remote cache
- `--no-cache` flag to bypass parse cache

**CLI**
- `--timing-file <path>` for CSV timing data
- `--framework openai` and `--framework llamaindex` options
- `--graph <dot|mermaid>` for graph export
- `--refresh` for remote cache refresh
- `--no-cache` to disable parse caching

### Documentation

- `GRAPH_PATTERNS.md` -- Graph pattern detection
- `RESOURCE_TRACKING.md` -- Token/cost tracking
- `CONTROL_FLOW.md` -- Conditional/loop detection
- `CACHING.md` -- Parse caching system
- `PARSER_OPENAI.md` -- OpenAI Agents parser
- `PARSER_LLAMAINDEX.md` -- LlamaIndex parser
- `DETECTION.md` -- Framework detection
- `KEYBINDINGS.md` -- Keyboard shortcuts
- Updated `ARCHITECTURE.md` for v2
- Updated `CONFIG.md` with new options
- Updated `TUI.md` with mouse/toast features
- Updated `README.md` with new examples

### Dependencies Added

- `ureq` -- HTTP client for remote sources
- `dirs` -- Cache directory resolution

### Breaking Changes

None. All v2 changes are additive:
- New fields are `Option<T>` with defaults
- New CLI flags are optional
- Existing tests continue to pass
