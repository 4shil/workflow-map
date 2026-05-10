# v2 Features Overview

workflow-map v2 adds 16 major improvements across parsing, data model, TUI, export, and performance.

## Feature Matrix

| # | Feature | Status | Docs |
|---|---------|--------|------|
| 1 | tree-sitter Python parsing | Planned | — |
| 2 | Accurate line numbers | Done | [PARSERS.md](PARSERS.md) |
| 3 | Conditional/loop detection | Done | [CONTROL_FLOW.md](CONTROL_FLOW.md) |
| 4 | OpenAI Agents parser | Done | [PARSER_OPENAI.md](PARSER_OPENAI.md) |
| 5 | LlamaIndex parser | Done | [PARSER_LLAMAINDEX.md](PARSER_LLAMAINDEX.md) |
| 6 | Step timing/duration | Done | [TIMING.md](TIMING.md) |
| 7 | Resource tracking | Done | [RESOURCE_TRACKING.md](RESOURCE_TRACKING.md) |
| 8 | Graph pattern detection | Done | [GRAPH_PATTERNS.md](GRAPH_PATTERNS.md) |
| 9 | Mouse support | Done | [TUI.md](TUI.md) |
| 10 | Copy/paste | Planned | — |
| 11 | Split-view | Planned | — |
| 12 | DOT/Graphviz export | Done | [EXPORT_DOT_MERMAID.md](EXPORT_DOT_MERMAID.md) |
| 13 | Mermaid export | Done | [EXPORT_DOT_MERMAID.md](EXPORT_DOT_MERMAID.md) |
| 14 | Remote source support | Done | [REMOTE.md](REMOTE.md) |
| 15 | Parse caching | Done | [CACHING.md](CACHING.md) |
| 16 | Config validation | Done | [VALIDATION.md](VALIDATION.md) |

## Quick Start

### Parse a remote workflow
```
workflow-map https://raw.githubusercontent.com/user/repo/main/workflow.yaml
```

### Export to Graphviz DOT
```
workflow-map ./workflow.yaml --format json --graph dot --output workflow.dot
dot -Tpng workflow.dot -o workflow.png
```

### Export to Mermaid
```
workflow-map ./workflow.yaml --format json --graph mermaid --output workflow.mmd
```

### Load timing data
```
workflow-map ./workflow.yaml --timing-file timing.csv
```

### Validate a config
```
workflow-map ./workflow.yaml --format text  # warnings shown in output
```

## New CLI Flags

| Flag | Description |
|------|-------------|
| `--framework <name>` | Force framework (openai, llamaindex, etc.) |
| `--graph <format>` | Graph export (dot, mermaid) |
| `--timing-file <path>` | Load timing data from CSV |
| `--refresh` | Bypass remote cache |
| `--no-cache` | Bypass parse cache |

## New Config Options

```toml
[parsers]
disable_cache = false

[render]
color_scheme = "default"
```

## Performance

v2 is faster than v1 for repeated operations:
- Cached re-parse: <1ms (was ~50ms)
- Remote with cache: <1ms after first fetch
- Mouse interaction: instant (no keyboard needed)
