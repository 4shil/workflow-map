# Changelog

## v0.1.0 -- 2026-05-08

Initial release.

### Features
- **6 framework parsers** -- LangChain, CrewAI, DSPy, AutoGen, Hermes Agent, Generic JSON/YAML
- **Interactive TUI** -- Navigate with arrow keys, search, filter, collapse/expand
- **Export formats** -- Plain text, JSON, Markdown
- **Watch mode** -- Auto-reload on file changes with debounce
- **Framework auto-detection** -- Detects framework from file extension + content patterns
- **Error drill-down** -- View error messages, source locations, and suggestions
- **Pagination** -- Handles workflows with 200+ steps
- **Configurable** -- Config file at `~/.config/workflow-map/config.toml`
- **Zero dependencies** -- Single static binary

### Parsers
- **LangChain** -- Pipe chains, AgentExecutor, RunnableParallel, RunnableLambda
- **CrewAI** -- Agents, tasks, dependencies, process type
- **DSPy** -- Module subclasses, Predict, ChainOfThought, Retrieve
- **AutoGen** -- AssistantAgent, UserProxyAgent, GroupChat
- **Hermes Agent** -- SKILL.md with YAML frontmatter and markdown sections
- **Generic** -- Standard JSON/YAML workflow schema

### Quality
- 44 tests (21 unit + 23 integration)
- Zero clippy warnings
- CI with GitHub Actions
- 22 test fixture files

### Documentation
- README with quick start and examples
- Contributing guide
- Architecture documentation
- Testing guide
- Parser implementation guide
- TUI internals
- Export format specifications
- Configuration reference
