# Export Format Specifications

## Text Format

Plain text ASCII diagram. No ANSI escape codes. Suitable for piping, logging, or printing.

### Structure
```
Workflow: {name} ({framework)
────────────────────────────────────────────────────────────
{collapse} {error_prefix}{step_num}. {name}
...
{summary} steps, {ok} ok, {err} errors, {waiting} waiting
```

### Step Line Format
```
{indent}{collapse} {error_prefix}{step_num}. {name}
```

| Component | Description | Example |
|-----------|-------------|---------|
| `indent` | 2 spaces per nesting depth | `    ` (depth 2) |
| `collapse` | `[+]` collapsed, `[-]` expanded, `[ ]` leaf | `[-]` |
| `error_prefix` | `*` if error, space otherwise | `*` |
| `step_num` | Sequential number (1-based) | `3` |
| `name` | Step name, max 30 chars | `Clean data` |

### Status Markers
| Status | Marker |
|--------|--------|
| Ok | `OK` |
| Error | `ERR` |
| Running | `RUN` |
| Waiting | `WAIT` |
| Skipped | `SKIP` |

### Example
```
Workflow: data-pipeline (CrewAI)
────────────────────────────────────────────────────────────
[-]  1. researcher (Senior Researcher)
    [ ]  1. researcher_tool_0 (search_tool)
    [ ]  1. researcher_tool_1 (browser_tool)
[-]  2. analyst (Data Analyst)
[ ]  *3. Clean data
[ ]  4. Transform
[ ]  5. Aggregate

6 steps, 2 ok, 1 errors, 3 waiting
```

## JSON Format

Full workflow serialization. Pretty-printed with 2-space indent.

### Schema
```json
{
  "name": "string",
  "source_path": "string",
  "framework": "LangChain | CrewAI | DSPy | AutoGen | Hermes | Generic | Unknown",
  "steps": [
    {
      "id": "string",
      "name": "string",
      "step_type": "Agent | Tool | Chain | Task | Step | Predict | Retrieve | Lambda | Module",
      "status": "Ok | Error | Running | Waiting | Skipped",
      "source_location": {
        "file": "string",
        "line": "number",
        "column": "number | null"
      },
      "config_snippet": "string",
      "error": {
        "message": "string",
        "stack_trace": "string | null",
        "suggestion": "string | null"
      } | null,
      "children": ["Step"],
      "dependencies": ["string"],
      "collapsed": "boolean"
    }
  ],
  "edges": [
    {
      "from": "string",
      "to": "string",
      "edge_type": "Sequential | Conditional | Parallel"
    }
  ],
  "metadata": {
    "key": "value"
  }
}
```

## Graph Formats

DOT and Mermaid are supported via `--graph`:

```
workflow-map ./workflow.yaml --format json --graph dot --output workflow.dot
workflow-map ./workflow.yaml --format json --graph mermaid --output workflow.mmd
```

See `docs/EXPORT_DOT_MERMAID.md` for details.

## Markdown Format

Human-readable document. Suitable for Obsidium notes, GitHub rendering, or documentation sites.

### Structure
```markdown
# {name}

Framework: {framework}

## Table of Contents
1. [Step Name](#step-name-anchor)
...

## 1. Step Name
- **Type:** {step_type}
- **Status:** {status_marker}
- **Source:** {file}:{line}
- **Description:** {config_snippet}

## Edges
- from -> to (edge_type)

## Metadata
| Key | Value |
|-----|-------|
| key | value |
```

### Markdown Anchors
Step names are converted to anchors by:
1. Lowercasing
2. Replacing non-alphanumeric characters with `-`
3. Collapsing multiple `-` into one

Example: `Clean Data` -> `#clean-data`

### Example
```markdown
# Data Pipeline

Framework: CrewAI

## Table of Contents
1. [Load CSV](#load-csv)
2. [Clean Data](#clean-data)
3. [Transform](#transform)

## 1. Load CSV
- **Type:** Task
- **Status:** OK
- **Source:** config.yaml:5
- **Description:** Load data from CSV file

## 2. Clean Data
- **Type:** Task
- **Status:** ERR
- **Source:** config.yaml:12
- **Description:** Clean and validate data
- **Error:** ValueError: Column 'price' has 23 null values
- **Suggestion:** Add null-handling step before cleaning

## Edges
- researcher -> research_task (sequential)
- research_task -> analysis_task (sequential)

## Metadata
| Key | Value |
|-----|-------|
| process | sequential |
```
