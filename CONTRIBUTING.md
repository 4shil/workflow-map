# Contributing to workflow-map

Thank you for your interest in contributing! This document covers development setup, code style, and how to add new features.

## Development Setup

### Prerequisites
- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Cargo (included with rustup)
- Optional: `cargo-watch` for auto-rebuild during development

```bash
# Clone the repo
git clone https://github.com/4shil/workflow-map.git
cd workflow-map

# Build
cargo build

# Run all tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Auto-rebuild on changes (requires cargo-watch)
cargo watch -x build
```

## Project Structure

```
workflow-map/
├── Cargo.toml              # Project config, dependencies
├── README.md               # User-facing documentation
├── CONTRIBUTING.md         # This file
├── LICENSE                 # MIT License
├── .github/
│   └── workflows/
│       ├── ci.yml          # CI: build, test, clippy, fmt
│       └── release.yml     # Release: build binaries, create GitHub release
├── docs/
│   ├── ARCHITECTURE.md     # Technical architecture deep-dive
│   ├── TESTING.md          # Testing guidelines
│   ├── PARSERS.md          # Parser implementation guide
│   ├── TUI.md              # TUI renderer internals
│   └── EXPORT.md           # Export format specifications
├── src/
│   ├── main.rs             # CLI entry point
│   ├── lib.rs              # Library root, re-exports
│   ├── cli.rs              # CLI argument parsing (clap)
│   ├── config.rs           # Config file loading (TOML)
│   ├── model.rs            # Core data types
│   ├── export.rs           # Export engine (text, JSON, Markdown)
│   ├── renderer.rs         # TUI renderer (ratatui)
│   └── parsers/
│       ├── mod.rs          # Parser dispatcher
│       ├── detector.rs     # Framework auto-detection
│       ├── generic.rs      # Generic JSON/YAML parser
│       ├── crewai.rs       # CrewAI YAML parser
│       ├── langchain.rs    # LangChain Python parser
│       ├── dspy.rs         # DSPy Python parser
│       ├── autogen.rs      # AutoGen Python parser
│       └── hermes.rs       # Hermes Agent SKILL.md parser
└── tests/
    ├── integration.rs      # Integration tests
    └── fixtures/           # Sample configs for each framework
        ├── langchain/      # 5 sample Python files
        ├── crewai/         # 3 sample YAML files
        ├── dspy/           # 3 sample Python files
        ├── autogen/        # 2 sample Python files
        ├── hermes/         # 2 sample SKILL.md files
        ├── generic/        # 4 sample YAML/JSON files
        └── mixed/          # 3 files from different frameworks
```

## Code Style

### Formatting
- Run `cargo fmt` before every commit
- CI will reject unformatted code

### Linting
- Run `cargo clippy -- -D warnings` before every commit
- All clippy warnings must be resolved
- Use `#[allow(...)]` sparingly and only with a comment explaining why

### Commit Messages
- **NO prefixes** like `feat:`, `fix:`, `step:`, `docs:`, `chore:`
- Write descriptive, imperative-mood messages
- Examples:
  - `Add CrewAI YAML config parser`
  - `Fix detector for SKILL.md files with underscores`
  - `Add integration tests for all parsers`
  - `Update README with installation instructions`

### Testing
- Write tests for every new parser, feature, or bug fix
- Follow TDD: write the failing test first, then implement
- Unit tests go in the same file as the code (`#[cfg(test)] mod tests`)
- Integration tests go in `tests/integration.rs`
- Test fixtures go in `tests/fixtures/<framework>/`

## Adding a New Framework Parser

1. Create `src/parsers/<framework>.rs` with a `parse(content, path) -> Result<Workflow>` function
2. Add `pub mod <framework>;` to `src/parsers/mod.rs`
3. Add detection logic to `src/parsers/detector.rs`
4. Add a match arm in `src/parsers/mod.rs` `parse_workflow()`
5. Add test fixtures in `tests/fixtures/<framework>/`
6. Add unit tests in the parser module
7. Add integration tests in `tests/integration.rs`
8. Update the Supported Frameworks table in README.md
9. Commit with message: `Add <Framework> parser`

## Pull Request Process

1. Fork the repository (or branch if you have access)
2. Create a feature branch: `git checkout -b add-xyz-parser`
3. Make your changes, following the code style above
4. Run `cargo test` and `cargo clippy -- -D warnings`
5. Commit with descriptive messages (no prefixes)
6. Push and open a PR
7. CI must pass (build, test, clippy, fmt)
8. Address review feedback
9. Squash if requested, then merge

## Code Review Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt --check`)
- [ ] New code has tests
- [ ] Test fixtures added for new parsers
- [ ] README updated if adding new features
- [ ] No hardcoded secrets or credentials
- [ ] Error handling is graceful (no panics on bad input)

## Questions?

Open an issue on GitHub or reach out to the maintainers.
