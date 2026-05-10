# Testing Guide

## Running Tests

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run specific test
cargo test test_parse_crewai

# Run tests matching a pattern
cargo test langchain

# Run integration tests only
cargo test --test integration

# Run sequentially (useful for TUI tests)
cargo test -- --test-threads=1
```

## Test Organization

### Unit Tests
Each source module contains inline unit tests:

```
src/parsers/langchain.rs    -> 4 unit tests (pipe, agent, empty, classify)
src/parsers/crewai.rs       -> 3 unit tests (config, malformed, empty)
src/parsers/dspy.rs         -> 2 unit tests (module, predict)
src/parsers/autogen.rs      -> 1 unit test (agents)
src/parsers/hermes.rs       -> 3 unit tests (skill, no frontmatter, empty)
src/parsers/generic.rs      -> 4 unit tests (yaml, json, edges, empty)
src/parsers/detector.rs     -> 6 unit tests (all framework detections)
```

### Integration Tests
`tests/integration.rs` tests end-to-end parsing and export:

- **Parser integration** -- Parse each framework's fixture files, verify step counts
- **Export integration** -- Export to text/JSON/Markdown, verify structure
- **Error handling** -- Malformed files, empty files, missing files

### Test Fixtures
`tests/fixtures/` contains sample configs for each framework:

```
tests/fixtures/
├── langchain/     # 5 files: simple_chain, agent_executor, runnable_parallel, empty, malformed
├── crewai/        # 3 files: data_pipeline, empty, malformed
├── dspy/          # 3 files: rag_program, simple_predict, empty
├── autogen/       # 2 files: two_agents, empty
├── hermes/        # 2 files: test_skill, empty_skill
├── generic/       # 4 files: simple_workflow, parallel_workflow, workflow.json, empty
└── mixed/         # 3 files: pipeline.yaml, langchain_script.py, SKILL.md
```

## Adding New Tests

### For a new parser:
1. Add unit tests in the parser module (`#[cfg(test)] mod tests`)
2. Add fixture files in `tests/fixtures/<framework>/`
3. Add integration tests in `tests/integration.rs`

### Test fixture naming:
- `<descriptive_name>.<ext>` -- Valid config files
- `empty.<ext>` -- File with no workflow patterns
- `malformed.<ext>` -- File with syntax errors

### Example unit test:
```rust
#[test]
fn test_parse_my_framework() {
    let content = include_str!("../../tests/fixtures/myframework/sample.yaml");
    let path = std::path::Path::new("sample.yaml");
    let workflow = my_framework::parse(content, path).unwrap();
    assert!(!workflow.steps.is_empty());
    assert_eq!(workflow.framework, Framework::MyFramework);
}
```

## CI Pipeline

Every push and PR triggers the CI pipeline:

1. Checkout code
2. Install Rust stable
3. Cache cargo dependencies
4. `cargo fmt --check` -- Verify formatting
5. `cargo clippy -- -D warnings` -- Lint
6. `cargo build --release` -- Build release binary
7. `cargo test` -- Run all tests

All steps must pass before merging.

## Coverage Goals

| Area | Target |
|------|--------|
| Parser detection | 90% |
| Parser logic | 80% |
| Export engine | 85% |
| Error handling | 75% |
| Overall | 75% |

## Snapshot Testing

For complex output verification, use snapshot tests:

```rust
#[test]
fn test_text_output_snapshot() {
    let workflow = parse_fixture("crewai/data_pipeline.yaml");
    let exporter = Exporter::new(workflow);
    let text = exporter.to_text();
    insta::assert_snapshot!("crewai_data_pipeline", text);
}
```
