# Contributing to workflow-map

Thank you for your interest in contributing! This document covers how to set up
your development environment, the project structure, coding standards, and the
pull request process.

## Development Setup

### Prerequisites

- **Rust 1.75+** -- Install via [rustup](https://rustup.rs/).
- **cargo** -- Included with Rust.
- **cargo-watch** (optional) -- For auto-rebuilding during development:

      cargo install cargo-watch

### Clone and Build

    git clone https://github.com/4shil/workflow-map.git
    cd workflow-map
    cargo build

### Running Tests

    cargo test
    cargo test -- --test-threads=1    # Sequential (for snapshot tests)

## Project Structure

    workflow-map/
    |-- Cargo.toml
    |-- Cargo.lock
    |-- README.md
    |-- LICENSE
    |-- .gitignore
    |-- src/
    |   |-- main.rs              # Entry point, CLI dispatch
    |   |-- cli.rs               # Argument parsing (clap)
    |   |-- config.rs            # AppConfig, ParserConfig, RenderConfig
    |   |-- model.rs             # Workflow, Step, Edge, Framework, enums
    |   |-- export.rs            # Exporter: text, JSON, Markdown output
    |   |-- renderer.rs          # TUI app (ratatui): input, layout, render
    |   |-- parsers/
    |       |-- mod.rs           # Framework dispatch, directory walking
    |       |-- detector.rs      # Auto-detect framework from file content
    |       |-- langchain.rs     # LangChain Python parser
    |       |-- crewai.rs        # CrewAI YAML parser
    |       |-- dspy.rs          # DSPy Python parser
    |       |-- autogen.rs       # AutoGen Python parser
    |       |-- hermes.rs        # Hermes Agent SKILL.md parser
    |       |-- generic.rs       # Generic YAML/JSON workflow parser
    |-- tests/
    |   |-- fixtures/
    |       |-- langchain/       # LangChain test inputs (.py)
    |       |-- crewai/          # CrewAI test inputs (.yaml)
    |       |-- dspy/            # DSPy test inputs (.py)
    |       |-- autogen/         # AutoGen test inputs (.py)
    |       |-- hermes/          # Hermes test inputs (.md)
    |       |-- generic/         # Generic test inputs (.yaml, .json)
    |       |-- mixed/           # Mixed-format directories
    |-- docs/
        |-- TESTING.md           # Testing guide
        |-- ARCHITECTURE.md      # Architecture overview

## Code Style

### Formatting

All code must be formatted with `rustfmt`:

    cargo fmt

Check without modifying:

    cargo fmt --check

### Linting

Run Clippy on every commit:

    cargo clippy -- -D warnings

### Commit Messages

Use clear, descriptive commit messages. Do **not** use conventional-commit
prefixes (`feat:`, `fix:`, `step:`, `docs:`, etc.). Instead, write plain
imperative sentences:

    Add support for CrewAI task dependencies

    Handle empty YAML files in generic parser

    Fix off-by-one error in TUI scroll offset

## How to Add a New Framework Parser

1. **Create the parser module** at `src/parsers/<name>.rs`. Implement a
   `parse(content: &str, path: &Path, config: &ParserConfig) -> Result<Workflow>`
   function.

2. **Register the framework** in `src/model.rs`:
   - Add a variant to the `Framework` enum.
   - Update `Framework::all()`, `Display`, and serialization.

3. **Add detection rules** in `src/parsers/detector.rs`:
   - Add file-extension and content-based heuristics.

4. **Wire it into the dispatcher** in `src/parsers/mod.rs`:
   - Add a match arm in `parse_file()` and `parse_framework_name()`.

5. **Update the CLI** in `src/cli.rs`:
   - Add the framework name to the `--framework` value parser list.

6. **Add test fixtures** under `tests/fixtures/<name>/` with at least:
   - A valid input file.
   - An empty/edge-case file.
   - A malformed input file.

7. **Document it** in the Supported Frameworks table in `README.md`.

## Testing Guidelines

### Test-Driven Development

Write tests before or alongside new parser logic. Every parser should have:

- **Unit tests** covering each parsing branch (valid, empty, malformed).
- **Integration tests** that parse fixture files and assert on the resulting
  `Workflow` model.

### Fixture Files

Place test inputs under `tests/fixtures/<framework>/`. Each fixture should be a
minimal, representative example:

    tests/fixtures/
    |-- langchain/
    |   |-- simple_chain.py     # Valid input
    |   |-- empty.py            # Edge case
    |   |-- malformed.py        # Invalid input

### Running Tests

    cargo test                              # All tests
    cargo test langchain                    # Filter by name
    cargo test -- --test-threads=1          # Sequential (for snapshots)
    cargo test -- --nocapture               # Show println! output

## Pull Request Process

1. **Fork the repository** and create a feature branch.

2. **Make your changes** with tests.

3. **Verify everything passes**:

       cargo fmt --check
       cargo clippy -- -D warnings
       cargo test

4. **Write a clear PR description** summarizing:
   - What changed and why.
   - Which frameworks or modules are affected.
   - How you tested the changes.

5. **Reference any related issues** with `Closes #N` or `Relates to #N`.

6. **Request review** from a maintainer.

7. **Address review feedback** and re-verify CI passes.

### PR Checklist

- [ ] Code formatted (`cargo fmt --check`)
- [ ] No Clippy warnings (`cargo clippy -- -D warnings`)
- [ ] All tests pass (`cargo test`)
- [ ] New functionality includes tests
-   Fixtures added for new parsers
- [ ] Documentation updated (README, docs/)
