# v2 Upgrade Plan -- 16 Improvements

## Analysis of v1 Weaknesses

### Parsing
1. **Regex-only Python parsing** -- Misses multi-line pipe chains, dynamic chains, nested calls
2. **No line number accuracy** -- All steps report line 1; parsers don't track actual source lines
3. **No conditional/loop detection** -- if/else branches and for-loops in workflow code are invisible
4. **Limited framework coverage** -- Missing OpenAI Agents SDK, LlamaIndex, Semantic Kernel, Pydantic AI

### Data Model
5. **No step timing/duration** -- Can't track how long steps take
6. **No resource tracking** -- No token counts, API calls, cost estimates
7. **Flat edge model** -- No support for complex graph patterns (diamond, fan-in/fan-out)

### TUI/UX
8. **No mouse support** -- Can't click to select steps
9. **No copy/paste** -- Can't copy step names or config snippets
10. **No split-view for large workflows** -- Single list view doesn't scale

### Export/Integration
11. **No DOT/Graphviz export** -- Can't generate proper graph visualizations
12. **No Mermaid export** -- Can't embed in GitHub/GitLab markdown
13. **No remote source support** -- Can't parse from GitHub URLs

### Performance/Quality
14. **No incremental parsing** -- Re-parses entire file on every change
15. **No caching** -- Repeated parses of same file are redundant
16. **No configuration validation** -- Invalid configs are silently accepted

---

## 16 v2 Upgrades

### Upgrade 1: tree-sitter Python Parsing
**Problem:** Regex misses multi-line chains, nested calls, dynamic construction
**Solution:** Integrate tree-sitter-python for accurate AST-based parsing
**Impact:** LangChain, DSPy, AutoGen parsers all benefit
**Effort:** Medium (2-3 days)
**Files:** `src/parsers/langchain.rs`, `src/parsers/dspy.rs`, `src/parsers/autogen.rs`
**Details:**
- Add `tree-sitter` and `tree-sitter-python` crates
- Parse Python AST once, then walk the tree to find workflow patterns
- Detect: assignments, function calls, class definitions, method calls
- Extract actual line numbers from AST nodes
- Fallback to regex if tree-sitter fails

### Upgrade 2: Accurate Line Number Tracking
**Problem:** All steps report line 1
**Solution:** Track actual line numbers during parsing
**Impact:** All parsers, error messages, source locations
**Effort:** Small (1 day)
**Files:** All parser files
**Details:**
- Count line numbers from byte offsets in source content
- Store actual line in `SourceLocation`
- Update all `SourceLocation::new()` calls to pass correct line numbers

### Upgrade 3: Conditional and Loop Detection
**Problem:** if/else branches and loops in workflow code are invisible
**Solution:** Detect control flow patterns in Python parsers
**Impact:** LangChain, DSPy, AutoGen parsers
**Effort:** Medium (2 days)
**Files:** `src/parsers/langchain.rs`, `src/parsers/dspy.rs`, `src/parsers/autogen.rs`
**Details:**
- Detect `if/elif/else` blocks -> create conditional edges
- Detect `for`/`while` loops -> create loop step with children
- Detect `try/except` -> create error-handling step
- Add `StepType::Conditional`, `StepType::Loop`, `StepType::ErrorHandler`

### Upgrade 4: OpenAI Agents SDK Parser
**Problem:** Missing support for OpenAI's Agents SDK
**Solution:** New parser for OpenAI Agents patterns
**Impact:** New framework support
**Effort:** Medium (2 days)
**Files:** New `src/parsers/openai_agents.rs`
**Details:**
- Detect `Agent()`, `Runner()`, `handoff()` patterns
- Parse agent definitions, tool registrations, handoff chains
- Support both sync and async runner patterns

### Upgrade 5: LlamaIndex Parser
**Problem:** Missing support for LlamaIndex workflows
**Solution:** New parser for LlamaIndex patterns
**Impact:** New framework support
**Effort:** Medium (2 days)
**Files:** New `src/parsers/llamaindex.rs`
**Details:**
- Detect `QueryEngine`, `Workflow`, `StepEngine` patterns
- Parse multi-step workflows with event-driven steps
- Support `WorkflowRunner` and `AsyncWorkflowRunner`

### Upgrade 6: Step Timing and Duration
**Problem:** No way to track how long steps take
**Solution:** Add timing fields to Step model
**Impact:** Model, TUI, export
**Effort:** Small (1 day)
**Files:** `src/model.rs`, `src/renderer.rs`, `src/export.rs`
**Details:**
- Add `started_at: Option<DateTime>`, `duration: Option<Duration>` to Step
- Parse timing from log files or status files
- Display duration in TUI detail panel and exports
- Add `--timing-file` flag for external timing data

### Upgrade 7: Resource Tracking (Tokens, API Calls, Cost)
**Problem:** No visibility into resource usage
**Solution:** Add resource tracking to Step model
**Impact:** Model, TUI, export
**Effort:** Small (1 day)
**Files:** `src/model.rs`, `src/renderer.rs`, `src/export.rs`
**Details:**
- Add `ResourceUsage { tokens_in, tokens_out, api_calls, cost_usd }` to Step
- Parse from framework-specific outputs (LangChain callbacks, DSPy history)
- Display in TUI detail panel
- Add summary row in text/Markdown export

### Upgrade 8: Complex Graph Edge Patterns
**Problem:** Flat edge model doesn't represent diamond/fan-in/fan-out patterns
**Solution:** Add graph pattern detection and visualization
**Impact:** Model, TUI, export
**Effort:** Medium (2 days)
**Files:** `src/model.rs`, `src/renderer.rs`, `src/export.rs`
**Details:**
- Add `GraphPattern` enum: Diamond, FanIn, FanOut, Pipeline, DAG
- Detect patterns from edge topology
- Render pattern indicators in TUI
- Export pattern metadata in JSON/Markdown

### Upgrade 9: Mouse Support in TUI
**Problem:** Can't click to select steps
**Solution:** Enable mouse events in crossterm
**Impact:** TUI renderer
**Effort:** Small (1 day)
**Files:** `src/renderer.rs`
**Details:**
- Enable `MouseCapture` in crossterm setup
- Handle `MouseEvent::Down` to select step under cursor
- Handle `MouseEvent::ScrollUp/ScrollDown` for scrolling
- Add visual feedback for clickable elements

### Upgrade 10: Copy/Paste Support
**Problem:** Can't copy step names or config snippets
**Solution:** Add clipboard integration
**Impact:** TUI renderer
**Effort:** Small (1 day)
**Files:** `src/renderer.rs`
**Details:**
- Add `y` key to yank (copy) step name to clipboard
- Add `Y` key to yank full step details
- Use `copypasta` crate for cross-platform clipboard
- Show "Copied!" toast notification in status bar

### Upgrade 11: Split-View for Large Workflows
**Problem:** Single list view doesn't scale to 100+ steps
**Solution:** Add tree view + detail panel layout
**Impact:** TUI renderer
**Effort:** Medium (2 days)
**Files:** `src/renderer.rs`
**Details:**
- Default: tree view (left) + step list (right) + detail (bottom)
- Tree view shows workflow hierarchy with collapse/expand
- Toggle between list-only, tree-only, and split views with `v` key
- Add minimap showing position in large workflows

### Upgrade 12: DOT/Graphviz Export
**Problem:** Can't generate proper graph visualizations
**Solution:** Add DOT format export
**Impact:** Export engine
**Effort:** Small (1 day)
**Files:** `src/export.rs`
**Details:**
- Add `to_dot()` method to Exporter
- Generate valid DOT syntax with nodes, edges, labels
- Support `--format dot` CLI flag
- Include styling: colors for status, shapes for step types

### Upgrade 13: Mermaid Export
**Problem:** Can't embed workflow diagrams in GitHub/GitLab markdown
**Solution:** Add Mermaid diagram format
**Impact:** Export engine
**Effort:** Small (1 day)
**Files:** `src/export.rs`
**Details:**
- Add `to_mermaid()` method to Exporter
- Generate Mermaid flowchart syntax
- Support `--format mermaid` CLI flag
- Include clickable nodes with tooltips

### Upgrade 14: Remote Source Support
**Problem:** Can't parse configs from GitHub URLs
**Solution:** Add HTTP client for remote configs
**Impact:** CLI, parsers
**Effort:** Medium (2 days)
**Files:** `src/main.rs`, new `src/remote.rs`
**Details:**
- Detect URLs (http/https) in path argument
- Fetch content via `ureq` or `reqwest` (with `rustls`, no openssl)
- Cache fetched content in `~/.cache/workflow-map/`
- Support GitHub raw URLs, generic HTTP URLs
- Add `--refresh` flag to bypass cache

### Upgrade 15: Incremental Parsing and Caching
**Problem:** Re-parses entire file on every change
**Solution:** Cache parsed workflows, invalidate on file change
**Impact:** Performance, watch mode
**Effort:** Medium (2 days)
**Files:** `src/parsers/mod.rs`, new `src/cache.rs`
**Details:**
- Cache `Workflow` keyed by file path + mtime
- On watch mode change: check mtime, skip parse if unchanged
- Store cache in `~/.cache/workflow-map/parsed/`
- Add `--no-cache` flag to bypass
- LRU eviction for cache size management

### Upgrade 16: Configuration Validation
**Problem:** Invalid configs are silently accepted
**Solution:** Add schema validation for each framework
**Impact:** All parsers, error reporting
**Effort:** Medium (2 days)
**Files:** All parser files, new `src/validate.rs`
**Details:**
- Define validation rules per framework (e.g., CrewAI requires agents + tasks)
- Validate after parsing, report warnings for suspicious configs
- Check for: missing dependencies, duplicate IDs, unreachable steps
- Add `--strict` flag to treat validation warnings as errors
- Output validation report in TUI and exports

---

## Implementation Order

### Phase 1: Foundation (Week 1)
1. Upgrade 2 -- Accurate Line Numbers (quick win, improves everything)
2. Upgrade 6 -- Step Timing (small model change)
3. Upgrade 7 -- Resource Tracking (small model change)
4. Upgrade 8 -- Complex Graph Edges (model + rendering)

### Phase 2: Parsing Improvements (Week 2)
5. Upgrade 1 -- tree-sitter Python Parsing (biggest impact)
6. Upgrade 3 -- Conditional/Loop Detection
7. Upgrade 4 -- OpenAI Agents SDK Parser
8. Upgrade 5 -- LlamaIndex Parser

### Phase 3: UX Improvements (Week 3)
9. Upgrade 9 -- Mouse Support
10. Upgrade 10 -- Copy/Paste
11. Upgrade 11 -- Split-View for Large Workflows
12. Upgrade 15 -- Incremental Parsing and Caching

### Phase 4: Export + Integration (Week 4)
13. Upgrade 12 -- DOT/Graphviz Export
14. Upgrade 13 -- Mermaid Export
15. Upgrade 14 -- Remote Source Support
16. Upgrade 16 -- Configuration Validation

---

## New Dependencies

```toml
# Parsing
tree-sitter = "0.24"
tree-sitter-python = "0.23"

# Remote
ureq = "2.10"  # or reqwest with rustls

# Clipboard
copypasta = "0.10"

# Time
chrono = "0.4"

# Caching
directories = "5.0"  # already a dep
```

## New Files

```
src/
├── parsers/
│   ├── openai_agents.rs    # Upgrade 4
│   ├── llamaindex.rs       # Upgrade 5
│   └── mod.rs              # Updated dispatcher
├── remote.rs               # Upgrade 14
├── cache.rs                # Upgrade 15
├── validate.rs             # Upgrade 16
└── model.rs                # Upgrades 6, 7, 8
```

## Test Plan

Each upgrade requires:
- Unit tests for new parsing logic
- Integration tests with new fixture files
- Updated snapshot tests for export formats
- TUI manual testing for UX changes

## Backward Compatibility

All v2 changes are additive:
- New fields in `Step` are `Option<T>` with defaults
- New export formats are new methods, existing ones unchanged
- New CLI flags are optional with sensible defaults
- Existing tests continue to pass without modification
