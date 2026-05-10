# Timing + Resource Data

workflow-map can ingest a CSV file to attach durations and resource usage to steps.

## CLI

```
workflow-map ./workflow.yaml --timing-file timing.csv
```

## CSV Format

Columns:

```
step_id,duration_ms,tokens_in,tokens_out,api_calls,cost_usd
```

Example:

```
load_data,1200,250,410,1,0.0021
clean_data,800,120,300,1,0.0013
```

Notes:
- `step_id` must match the step id in the parsed workflow.
- Only `step_id` and `duration_ms` are required.
- Missing fields are ignored.
