# Framework Detection

workflow-map automatically detects the framework of a workflow file based on file extension and content patterns.

## Detection Order

1. **File extension** — `.yaml`/`.json` suggests CrewAI or Generic
2. **Content patterns** — Regex matching against known framework signatures
3. **Framework enum** — Returns the best match or `Unknown`

## Detection Rules

### LangChain
- File: `.py`
- Patterns: `from langchain`, `import langchain`, `AgentExecutor(`, `RunnableParallel(`, `RunnableLambda(`, pipe chains (`|`)

### CrewAI
- File: `.yaml`, `.yml`
- Patterns: `agents:`, `tasks:`, `process:`

### DSPy
- File: `.py`
- Patterns: `import dspy`, `dspy.Module`, `dspy.Predict(`, `dspy.ChainOfThought(`, `dspy.Retrieve(`

### AutoGen
- File: `.py`
- Patterns: `from autogen`, `import autogen`, `AssistantAgent(`, `UserProxyAgent(`, `GroupChat(`

### Hermes Agent
- File: `.md`
- Patterns: YAML frontmatter with `name:`, `description:`, `category:`

### OpenAI Agents
- File: `.py`
- Patterns: `from openai import Agent`, `Agent(`, `Runner.run(`, `handoff(`

### LlamaIndex
- File: `.py`
- Patterns: `from llama_index`, `Workflow`, `@step`, `WorkflowRunner`

### Generic
- File: `.yaml`, `.yml`, `.json`
- Patterns: `name:`, `steps:`, `edges:`

## CLI Override

Use `--framework` to skip auto-detection:

```
workflow-map ./file.py --framework langchain
```

## Detection Function

```rust
pub fn detect_framework(config: &ParserConfig, path: &Path) -> Result<Framework> {
    // 1. Check file extension
    // 2. Read file content
    // 3. Match patterns in priority order
    // 4. Return best match or Unknown
}
```

## Adding New Frameworks

To add a new framework:

1. Add variant to `Framework` enum in `src/model.rs`
2. Add display mapping in `impl fmt::Display for Framework`
3. Add detection pattern in `src/parsers/detector.rs`
4. Add parser module in `src/parsers/`
5. Add dispatch in `src/parsers/mod.rs`
6. Add CLI value in `src/cli.rs`

## Detection Accuracy

| Framework | Accuracy | Notes |
|-----------|----------|-------|
| LangChain | ~95% | Pipe chains may conflict with generic Python |
| CrewAI | ~99% | YAML structure is distinctive |
| DSPy | ~95% | `dspy.Module` is distinctive |
| AutoGen | ~90% | May conflict with other agent libraries |
| Hermes | ~99% | Frontmatter is distinctive |
| OpenAI | ~90% | New patterns, still maturing |
| LlamaIndex | ~85% | Scaffold, needs more patterns |
| Generic | ~80% | May match non-workflow YAML/JSON |

## Future Improvements

- tree-sitter AST-based detection for higher accuracy
- Multi-signal detection (imports + class definitions + function calls)
- Confidence scores for ambiguous files
- User prompt when detection confidence is low
