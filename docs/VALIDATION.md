# Validation

workflow-map performs lightweight validation after parsing. Warnings appear in the Markdown export and TUI detail panel.

## Current Rules

- CrewAI: requires at least one agent and one task.
- Hermes: warns if no sections are present.
- Generic: warns if no steps are present.
- OpenAI Agents: warns if no Agent() is detected.
- LlamaIndex: warns if no Workflow() is detected.

## Notes

Validation is non-fatal by default unless `--strict` is used.

## Strict Mode

Use `--strict` in CI or scripts when validation warnings should fail the
command:

```bash
workflow-map ./workflow.yaml --format text --strict
```

Strict mode runs after parsing, timing data, and status-file overrides are
applied. The command exits with an error if duplicate IDs, missing dependencies,
invalid edges, empty workflows, or framework-specific validation issues are
reported.

## Status Overrides

Runtime status can be overlaid from JSON or YAML:

```yaml
load:
  status: ok
  duration_ms: 125
  tokens_in: 100
  tokens_out: 40
  api_calls: 1
  cost_usd: 0.003
clean:
  error: Column missing
  suggestion: Check upstream schema
```

Run it with:

```bash
workflow-map ./workflow.yaml --format text --status-file status.yaml
```
