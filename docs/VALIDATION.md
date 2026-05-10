# Validation

workflow-map performs lightweight validation after parsing. Warnings appear in the Markdown export and TUI detail panel.

## Current Rules

- CrewAI: requires at least one agent and one task.
- Hermes: warns if no sections are present.
- Generic: warns if no steps are present.
- OpenAI Agents: warns if no Agent() is detected.
- LlamaIndex: warns if no Workflow() is detected.

## Notes

Validation is non-fatal by default. Future versions can add a strict mode.
