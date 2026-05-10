# Graph Pattern Detection

workflow-map analyzes edge topology to classify the overall workflow structure into a `GraphPattern`.

## Pattern Types

| Pattern | Description | Topology |
|---------|-------------|----------|
| Pipeline | Linear chain of steps | Each node has in-degree 1, out-degree 1 |
| FanOut | One step branches to multiple | A node has out-degree > 1 |
| FanIn | Multiple steps converge to one | A node has in-degree > 1 |
| Diamond | Fan-out followed by fan-in | Both fan-out and fan-in nodes present |
| DAG | General directed acyclic graph | Complex topology, no single pattern |
| Unknown | No edges detected | Empty or single-node workflow |

## Detection Algorithm

The detector counts in-degree and out-degree for all nodes:

```
fan_out_count = nodes with out-degree > 1
fan_in_count  = nodes with in-degree > 1

if fan_out > 0 and fan_in > 0 -> Diamond
if fan_out > 0               -> FanOut
if fan_in > 0                -> FanIn
if edges > 2                 -> Pipeline
else                         -> DAG
```

## Usage

Pattern detection runs automatically after parsing:

```rust
workflow.detect_graph_pattern();
```

The pattern is included in JSON and Markdown exports under `metadata.graph_pattern`.

## Example Output

### Pipeline
```
A -> B -> C -> D
```
Pattern: `pipeline`

### FanOut
```
A -> B
A -> C
A -> D
```
Pattern: `fan-out`

### FanIn
```
A -> D
B -> D
C -> D
```
Pattern: `fan-in`

### Diamond
```
A -> B
A -> C
B -> D
C -> D
```
Pattern: `diamond`

## TUI Display

The detected pattern appears in the title bar:
```
My Workflow (LangChain) | Step 1/4 | Filter: All | Pattern: diamond
```

## Export Integration

JSON export includes:
```json
{
  "metadata": {
    "graph_pattern": "diamond"
  }
}
```

Markdown export includes:
```
## Metadata

| Key | Value |
|-----|-------|
| graph_pattern | diamond |
```
