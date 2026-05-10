# FAQ

## General

### What is workflow-map?
A terminal-native tool that parses agentic workflow configs (LangChain, CrewAI, DSPy, AutoGen, Hermes Agent) and renders them as interactive ASCII diagrams. No browser, no cloud, works over SSH.

### Which frameworks are supported?
- LangChain (Python)
- CrewAI (YAML)
- DSPy (Python)
- AutoGen (Python)
- Hermes Agent (SKILL.md)
- Generic JSON/YAML workflows

### What does it cost?
Nothing. It's open source (MIT License).

## Installation

### How do I install it?
```bash
git clone https://github.com/4shil/workflow-map.git
cd workflow-map
cargo build --release
```
The binary will be at `./target/release/workflow-map`.

### What are the requirements?
- Rust 1.75+ (if building from source)
- No runtime dependencies -- it's a single static binary

### Does it work on macOS/Windows?
Yes. The CI builds for Linux x64, macOS x64, macOS ARM64, and Windows x64.

## Usage

### How do I use it?
```bash
# Interactive TUI (default)
workflow-map ./my_workflow.yaml

# Text output
workflow-map ./pipeline.py --format text

# JSON export
workflow-map ./config.yaml --format json --output workflow.json
```

### Can I parse a directory of workflow files?
Yes. Pass a directory path and it will scan all files, auto-detect each framework, and render a unified map.

### How do I force a specific framework?
Use `--framework <name>`:
```bash
workflow-map ./my_file.py --framework langchain
```

### What does the status mean?
- `[OK]` -- Step completed successfully
- `[ERR]` -- Step failed (expand with Enter for error details)
- `[RUN]` -- Step is currently running
- `[WAIT]` -- Step is queued (default for static analysis)
- `[SKIP]` -- Step was skipped

In v1, all steps show `[WAIT]` since the tool performs static analysis only. Runtime status detection is planned for v1.1.

## TUI

### How do I navigate?
| Key | Action |
|-----|--------|
| Arrow keys | Move cursor |
| Enter | Expand/collapse step detail |
| f | Cycle filter (all/failed/running) |
| / | Search steps |
| e | Export current view |
| r | Trigger reload |
| ? | Help overlay |
| q | Quit |

### Can I use it over SSH?
Yes. It works in any ANSI terminal including over SSH, in tmux, or in screen.

### My terminal is small. Will it work?
Minimum recommended size is 80x24. The tool adapts to terminal width and paginates long workflows.

## Parsing

### How does framework detection work?
The detector checks file extension first, then scans content for framework-specific patterns:
- `.py` with `langchain` + `Chain`/`Runnable` -> LangChain
- `.py` with `dspy` + `Module`/`Predict` -> DSPy
- `.py` with `autogen` + `AssistantAgent` -> AutoGen
- `.yaml` with `agents:` + `tasks:` -> CrewAI
- `SKILL.md` or `*_skill.md` -> Hermes Agent
- `.yaml`/`.json` with `steps:` -> Generic

### Can I parse non-standard configs?
Use the Generic parser with `--framework generic` for custom YAML/JSON workflows that follow the standard schema.

### What if my config has errors?
The parser will report parse errors to stderr but continue rendering what it can. Partial results are still useful.

## Contributing

### How do I contribute?
See [CONTRIBUTING.md](../CONTRIBUTING.md). Fork, branch, implement, test, PR.

### How do I add a new framework parser?
See the [Parser Implementation Guide](PARSERS.md). Create a parser module, add detection, register in the dispatcher, add tests.

### What's the code style?
- `cargo fmt` for formatting
- `cargo clippy -- -D warnings` for linting
- No commit message prefixes (just descriptive messages)
- TDD: write tests first

## Troubleshooting

### The TUI doesn't render correctly
Ensure your terminal supports ANSI escape codes. Try `export TERM=xterm-256color`.

### Parse errors for valid configs
The regex-based parser may miss complex patterns. Try `--framework` to force the correct parser. Open an issue with your config (sanitized) so we can improve detection.

### Binary is too large
Use `cargo build --release` which enables LTO and stripping. The release binary should be under 15MB.
