# Graph Export Formats (DOT + Mermaid)

## DOT (Graphviz)

Command:
```
workflow-map ./workflow.yaml --format dot --output workflow.dot
```

Example output:
```
digraph workflow {
  rankdir=LR;
  node [shape=box, style=rounded];
  "step_1" [label="Load data\nStep", color="#2e7d32"];
  "step_2" [label="Clean data\nStep", color="#c62828"];
  "step_1" -> "step_2" [label="seq"];
}
```

Notes:
- Nodes are labeled with the step name and type.
- Colors map to status.
- Edge labels: `seq`, `cond`, `par`.

## Mermaid

Command:
```
workflow-map ./workflow.yaml --format mermaid --output workflow.mmd
```

Example output:
```
flowchart LR
  step_1["Load data"]
  step_2["Clean data"]
  step_1  step_2
```

Notes:
- Mermaid nodes use the step id and name.
- Edge labels follow the same mapping as DOT.
- `flowchart LR` creates a left-to-right graph.
