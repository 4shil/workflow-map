# LlamaIndex Parser

workflow-map includes a parser for [LlamaIndex](https://docs.llamaindex.ai/) workflow patterns.

## Supported Patterns

| Pattern | Step Type | Description |
|---------|-----------|-------------|
| `QueryEngine` | Chain | Query engine for RAG |
| `Workflow` | Chain | Multi-step LlamaIndex workflow |
| `StepEngine` | Step | Individual workflow step |
| `WorkflowRunner` | Chain | Synchronous workflow execution |
| `AsyncWorkflowRunner` | Chain | Asynchronous workflow execution |
| `VectorStoreIndex` | Retrieve | Vector index for retrieval |
| `ResponseSynthesizer` | Chain | Response synthesis |

## Example Input

```python
from llama_index.core import VectorStoreIndex, QueryEngine
from llama_index.core.workflow import Workflow, step

class RAGWorkflow(Workflow):
    @step
    async def retrieve(self, query: str) -> str:
        return self.index.query(query)

    @step
    async def synthesize(self, context: str) -> str:
        return self.llm.complete(context)

workflow = RAGWorkflow()
result = await workflow.run(query="What is RAG?")
```

## Parsed Output

```
1. [Chain] RAGWorkflow (Workflow)
  [Step] retrieve (StepEngine)
  [Step] synthesize (StepEngine)
2. [Chain] workflow.run (WorkflowRunner)
```

## CLI Usage

```
workflow-map ./llamaindex_workflow.py
workflow-map ./llamaindex_workflow.py --framework llamaindex
```

## Framework Detection

Auto-detection looks for:
- `from llama_index` imports
- `Workflow` class inheritance
- `@step` decorator
- `WorkflowRunner` or `AsyncWorkflowRunner`

## Current Status

The LlamaIndex parser is a scaffold. It detects basic patterns but does not yet:
- Parse `@step` decorated methods as children
- Extract event-driven step connections
- Handle `Workflow.run()` with keyword arguments

These features are planned for a future update.

## Limitations

- Complex event-driven workflows may not be fully traced
- Dynamic workflow construction is not detected
- Multi-file projects require directory parsing
