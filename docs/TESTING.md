# Testing Guide

This document describes the testing strategy for workflow-map, including how to
run tests, how tests are organized, and how to add new ones.

## How to Run Tests

### All Tests

    cargo test

### Sequential Execution

Some tests (particularly snapshot tests) may interfere with each other when run
in parallel. Use a single thread:

    cargo test -- --test-threads=1

### Filtered Tests

Run tests matching a pattern:

    cargo test langchain       # All langchain tests
    cargo test parser          # All parser tests
    cargo test detector        # Framework detection tests

### Verbose Output

Show `println!` and `eprintln!` output from passing tests:

    cargo test -- --nocapture

### Specific Test Binary

Run only integration tests:

    cargo test --test integration

## Test Organization

Tests are organized into three layers:

### 1. Unit Tests (per module)

Each source module contains inline unit tests in a `#[cfg(test)] mod tests`
block. These test individual functions and internal logic in isolation.

    src/
    |-- model.rs         -- Step, Workflow, FlatStep construction and queries
    |-- export.rs        -- Text, JSON, Markdown export correctness
    |-- parsers/
    |   |-- langchain.rs -- LangChain regex matching and step extraction
    |   |-- crewai.rs    -- YAML key mapping and task ordering
    |   |-- autogen.py   -- Agent extraction patterns
    |   |-- dspy.rs      -- Predict/Retrieve module detection
    |   |-- hermes.rs    -- SKILL.md frontmatter parsing
    |   |-- detector.rs  -- Framework auto-detection heuristics
    |-- config.rs        -- TOML config file loading and defaults

Run all unit tests:

    cargo test --lib

### 2. Integration Tests (`tests/` directory)

Integration tests exercise the full pipeline: file input -> detection -> parsing
-> model -> export. Fixture files under `tests/fixtures/` serve as test inputs.

Currently, integration tests are embedded as unit tests in each parser module
that read from the `tests/fixtures/` directory at test time. Each parser module
includes tests that:

1. Read each fixture file.
2. Parse it into a `Workflow` model.
3. Assert structural properties (step count, edge count, step names, etc.).

Fixture directory layout:

    tests/fixtures/
    |-- langchain/
    |   |-- simple_chain.py        # Minimal valid LangChain script
    |   |-- agent_executor.py      # Agent + tool composition
    |   |-- runnable_parallel.py   # Parallel chain syntax
    |   |-- empty.py               # Empty file edge case
    |   |-- malformed.py           # Invalid Python syntax
    |-- crewai/
    |   |-- data_pipeline.yaml     # Valid CrewAI workflow
    |   |-- empty.yaml             # Empty YAML
    |   |-- malformed.yaml         # Invalid YAML structure
    |-- dspy/
    |   |-- simple_predict.py      # Basic DSPy Predict module
    |   |-- rag_program.py         # Multi-step RAG pipeline
    |   |-- empty.py               # Empty file
    |-- autogen/
    |   |-- two_agents.py          # Two-agent AutoGen setup
    |   |-- empty.py               # Empty file
    |-- hermes/
    |   |-- test_skill.md          # Valid SKILL.md
    |   |-- empty_skill.md         # Empty skill file
    |-- generic/
    |   |-- simple_workflow.yaml   # Generic YAML workflow
    |   |-- parallel_workflow.yaml # Parallel step definitions
    |   |-- workflow.json          # JSON format workflow
    |   |-- empty.yaml             # Empty file
    |-- mixed/
    |   |-- pipeline.yaml          # CrewAI YAML
    |   |-- langchain_script.py    # LangChain Python
    |   |-- SKILL.md               # Hermes skill

### 3. Snapshot Tests

Snapshot tests capture the full text/JSON/Markdown output of the tool and
compare it against stored expected outputs. These prevent regressions in
rendering and export.

Snapshots are stored as strings inline in test functions (or in dedicated
snapshot files under `tests/snapshots/` in future iterations).

To update snapshots after an intentional change:

    cargo test -- --nocapture    # Inspect new output
    # Then update the expected strings in the test files

## How to Add New Test Fixtures

### For Existing Frameworks

1. Add the fixture file under `tests/fixtures/<framework>/`.
2. Add a test case in the corresponding parser's `#[cfg(test)]` module that:
   - Reads the fixture with `std::fs::read_to_string`.
   - Parses it with the parser's `parse()` function.
   - Asserts expected properties.

Example:

    #[test]
    fn test_simple_chain() {
        let path = PathBuf::from("tests/fixtures/langchain/simple_chain.py");
        let content = fs::read_to_string(&path).unwrap();
        let config = ParserConfig::default();
        let workflow = langchain::parse(&content, &path, &config).unwrap();
        assert_eq!(workflow.step_count(), 3);
        assert_eq!(workflow.steps[0].name, "prompt_template");
    }

### For New Frameworks

1. Create a new fixture directory: `tests/fixtures/<new_framework>/`.
2. Add at least three fixtures: valid, empty, malformed.
3. Create test functions in the new parser module.
4. Add integration-level tests that verify the full pipeline.

## CI Pipeline

Tests run automatically on every push and pull request via GitHub Actions.

### CI Steps

1. **Checkout** -- Clone the repository.
2. **Install Rust** -- Use `dtolnay/rust-toolchain@stable`.
3. **Cache** -- Cache `target/` and `~/.cargo/` for faster builds.
4. **Format check** -- `cargo fmt --check`.
5. **Lint** -- `cargo clippy -- -D warnings`.
6. **Build** -- `cargo build --release`.
7. **Test** -- `cargo test -- --test-threads=1`.
8. **Coverage** (optional) -- `cargo tarpaulin` or `cargo llvm-cov`.

### CI Configuration

The workflow is defined in `.github/workflows/ci.yml` and runs on:

- `push` to `main`
- `pull_request` targeting `main`

## Coverage Goals

| Area                  | Target | Notes                                |
|-----------------------|--------|--------------------------------------|
| Parser modules        | 80%+   | All branches: valid, empty, malformed|
| Model (Workflow/Step) | 90%+   | Core data structures                 |
| Export (text/JSON/MD) | 85%+   | All output formats                   |
| Detector              | 75%+   | All framework heuristics             |
| Config                | 70%+   | Default values, file loading         |
| Renderer (TUI)        | 50%+   | Interactive; hard to unit test       |
| Overall               | 75%+   |                                      |

To generate a local coverage report:

    cargo install cargo-tarpaulin
    cargo tarpaulin --out Html
    open tarpaulin-report.html
