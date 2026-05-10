# Roadmap

## v0.1.1 -- Parser Improvements

- [ ] tree-sitter Python parsing for accurate AST-based detection
- [ ] Support for LangChain LCEL multi-line pipe chains
- [ ] Support for DSPy program composition patterns
- [ ] Support for AutoGen nested GroupChat configurations
- [ ] Better error messages with source line highlighting

## v0.2.0 -- Runtime Integration

- [ ] Read execution status from log files
- [ ] `--status-file` flag for manual status override
- [ ] Live execution monitoring (attach to running processes)
- [ ] Framework-specific status detection (LangChain callbacks, CrewAI output)

## v0.3.0 -- Plugin System

- [ ] Custom parser plugins via WASM or dynamic libraries
- [ ] Plugin discovery from `~/.config/workflow-map/plugins/`
- [ ] Plugin API documentation
- [ ] Example plugins for popular frameworks

## v0.4.0 -- Remote Sources

- [ ] Parse configs from GitHub URLs
- [ ] Parse configs from HTTP/HTTPS URLs
- [ ] Local caching of remote configs
- [ ] `--remote` flag for explicit remote mode

## v0.5.0 -- Additional Export Formats

- [ ] DOT format (Graphviz)
- [ ] Mermaid diagram format
- [ ] HTML report with embedded SVG
- [ ] PDF output via headless Chrome

## v1.0.0 -- Stable Release

- [ ] Crates.io publish
- [ ] Homebrew tap
- [ ] AUR package
- [ ] Windows installer
- [ ] Comprehensive benchmark suite
- [ ] Performance regression tests
- [ ] Security audit

## Future Ideas

- [ ] Multi-file project parsing (entire agent directories)
- [ ] Configuration validation against framework schemas
- [ ] Diff mode (compare two workflow versions)
- [ ] Interactive workflow editor (read-write mode)
- [ ] LLM-powered workflow analysis and suggestions
- [ ] Integration with CI/CD pipelines
- [ ] WebAssembly build for browser-based TUI
