# OpenAI Agents SDK Parser

workflow-map includes a dedicated parser for the [OpenAI Agents SDK](https://github.com/openai/openai-agents-python).

## Supported Patterns

| Pattern | Step Type | Description |
|---------|-----------|-------------|
| `Agent()` | Agent | Agent definition with name, instructions, tools |
| `Runner.run()` | Chain | Synchronous agent execution |
| `Runner.run_sync()` | Chain | Synchronous alias |
| `handoff()` | Chain | Agent-to-agent handoff |
| `function_tool()` | Tool | Function tool registration |
| `CodeInterpreterTool()` | Tool | Code interpreter tool |
| `FileSearchTool()` | Tool | File search tool |
| `WebSearchTool()` | Tool | Web search tool |

## Example Input

```python
from openai import Agent, Runner, function_tool

@function_tool
def get_weather(city: str) -> str:
    return f"Weather in {city}: sunny"

agent = Agent(
    name="Weather Assistant",
    instructions="Help users with weather queries",
    tools=[get_weather]
)

result = Runner.run(agent, "What's the weather in Tokyo?")
```

## Parsed Output

```
1. [Agent] Weather Assistant (Agent)
2. [Tool] get_weather (function_tool)
3. [Chain] Runner.run (Runner.run)
```

## Handoff Chains

When agents use `handoff()`, the parser creates edges between them:

```python
triage = Agent(name="Triage")
billing = Agent(name="Billing")
support = Agent(name="Support")

triage_agent = Agent(
    name="Triage",
    handoffs=[handoff(billing), handoff(support)]
)
```

Output:
```
1. [Agent] Triage
2. [Agent] Billing
3. [Agent] Support
Edges: Triage -> Billing, Triage -> Support
```

## CLI Usage

```
workflow-map ./openai_workflow.py
workflow-map ./openai_workflow.py --framework openai
```

## Framework Detection

Auto-detection looks for:
- `from openai import Agent`
- `Agent(` with `name=`
- `Runner.run(` or `Runner.run_sync(`
- `handoff(`

## Limitations

- Async patterns (`await Runner.run()`) are detected but not fully traced
- Dynamic agent construction (agents built from config) may not be detected
- Multi-file agent projects require directory parsing
