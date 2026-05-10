# Conditional and Loop Detection

workflow-map detects control flow patterns in Python workflow code: `if/elif/else` branches, `for`/`while` loops, and `try/except` error handlers.

## Detected Patterns

| Pattern | Step Type | Description |
|---------|-----------|-------------|
| `if/elif/else` | Conditional | Branching logic in workflow code |
| `for`/`while` | Loop | Iteration over data or steps |
| `try/except` | ErrorHandler | Error handling blocks |

## How Detection Works

Each Python parser (LangChain, DSPy, AutoGen) scans the source content for control flow keywords using regex:

```rust
// Conditional detection
let if_re = Regex::new(r"(?m)^(\s*)if\s+.+:");

// Loop detection
let loop_re = Regex::new(r"(?m)^(\s*)(for|while)\s+.+:");

// Error handler detection
let try_re = Regex::new(r"(?m)^(\s*)try\s*:");
```

When a match is found, a new step is created with the appropriate `StepType` and the source line is recorded.

## Example: Conditional Detection

Input:
```python
if use_rag:
    chain = rag_chain
else:
    chain = simple_chain
```

Output steps:
```
1. [Conditional] if use_rag: (line 1)
2. [Chain] rag_chain (line 2)
3. [Chain] simple_chain (line 4)
```

## Example: Loop Detection

Input:
```python
for item in items:
    result = process(item)
```

Output steps:
```
1. [Loop] for item in items: (line 1)
2. [Step] process (line 2)
```

## Example: Error Handler Detection

Input:
```python
try:
    result = risky_call()
except Exception as e:
    result = fallback()
```

Output steps:
```
1. [ErrorHandler] try: (line 1)
2. [Step] risky_call (line 2)
3. [Step] fallback (line 4)
```

## Parser Support

| Parser | Conditional | Loop | Error Handler |
|--------|-------------|------|---------------|
| LangChain | Yes | Yes | Planned |
| DSPy | Yes | Yes | Planned |
| AutoGen | Yes | Yes | Planned |
| CrewAI | N/A (YAML) | N/A (YAML) | N/A (YAML) |
| Hermes | N/A (Markdown) | N/A (Markdown) | N/A (Markdown) |
| Generic | N/A (YAML/JSON) | N/A (YAML/JSON) | N/A (YAML/JSON) |

## Edge Creation

Control flow steps create implicit edges:
- Conditional: edges to each branch
- Loop: edge from loop step to body
- ErrorHandler: edge from try to except block

## TUI Rendering

Control flow steps show with special markers:
```
[-] 1. if use_run_rag:          (Conditional)
  [ ] 2. rag_chain               (Chain)
  [ ] 3. simple_chain            (Chain)
```

## Export Integration

Markdown export includes control flow type:
```markdown
## 1. if use_rag:

- **Type:** Conditional
- **Source:** workflow.py:1
```
