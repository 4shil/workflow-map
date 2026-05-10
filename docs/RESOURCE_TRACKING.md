# Resource Usage Tracking

workflow-map tracks resource consumption per step: token counts, API calls, and estimated cost.

## Data Model

```rust
pub struct ResourceUsage {
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub api_calls: Option<u32>,
    pub cost_usd: Option<f64>,
}
```

All fields are optional. A step with no resource data shows `none`.

## Builder API

```rust
let usage = ResourceUsage::new()
    .with_tokens(250, 410)
    .with_api_calls(1)
    .with_cost(0.0021);
```

## Display Format

```
in:250 out:410 calls:1 $0.0021
```

## Sources

Resource data can come from:

1. **Timing CSV file** — Loaded via `--timing-file timing.csv`
2. **Framework callbacks** — LangChain callbacks, DSPy history (future)
3. **Manual annotation** — Via config or status files (future)

## CSV Format

```
step_id,duration_ms,tokens_in,tokens_out,api_calls,cost_usd
load_data,1200,250,410,1,0.0021
clean_data,800,120,300,1,0.0013
```

## TUI Display

The detail panel shows resources when non-empty:
```
  Resources: in:250 out:410 calls:1 $0.0021
```

## Export Integration

### Text Export
```
5 steps, 3 ok, 1 errors, 1 waiting
Total tokens: 1090 (in:620 out:470)
Total API calls: 3
Total cost: $0.0057
```

### JSON Export
```json
{
  "steps": [
    {
      "id": "load_data",
      "resources": {
        "tokens_in": 250,
        "tokens_out": 410,
        "api_calls": 1,
        "cost_usd": 0.0021
      }
    }
  ]
}
```

### Markdown Export
```markdown
- **Resources:** in:250 out:410 calls:1 $0.0021
```

## Aggregation

The `total_tokens()` method sums input and output tokens:
```rust
usage.total_tokens(); // 660
```

Future versions will aggregate across all steps for a workflow summary.
