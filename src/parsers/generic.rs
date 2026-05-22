use crate::model::*;
use anyhow::Result;
use std::path::Path;

/// Parse a generic JSON or YAML workflow definition.
///
/// Supports the standard schema:
/// ```yaml
/// name: "Workflow Name"
/// steps:
///   - id: "step1"
///     name: "Load data"
///     type: "tool"
///     depends_on: []
/// ```
pub fn parse(content: &str, path: &Path) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "workflow".to_string()),
        Framework::Generic,
    );

    // Try YAML first, then JSON. If both fail, report parse error but return workflow.
    let value: serde_yaml::Value = if path.extension().map(|e| e == "json").unwrap_or(false) {
        serde_json::from_str(content)
            .or_else(|_| serde_yaml::from_str(content))
            .map_err(|e| anyhow::anyhow!("Failed to parse as JSON/YAML: {e}"))?
    } else {
        match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(e) => {
                workflow.add_parse_error(format!("YAML parse error (not a workflow file?): {e}"));
                return Ok(workflow);
            }
        }
    };

    // Extract name
    if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
        workflow.name = name.to_string();
    }

    // Extract optional description
    if let Some(desc) = value.get("description").and_then(|v| v.as_str()) {
        workflow
            .metadata
            .insert("description".to_string(), desc.to_string());
    }

    if let Some(steps) = value.get("steps").and_then(|v| v.as_sequence()) {
        for (i, step_val) in steps.iter().enumerate() {
            workflow.steps.push(parse_step_value(
                step_val,
                path,
                i + 1,
                &format!("step_{i}"),
            ));
        }
    }

    // Extract explicit edges if present
    if let Some(edges) = value.get("edges").and_then(|v| v.as_sequence()) {
        for edge_val in edges {
            let from = edge_val.get("from").and_then(|v| v.as_str());
            let to = edge_val.get("to").and_then(|v| v.as_str());
            let edge_type = edge_val.get("type").and_then(|v| v.as_str());

            if let (Some(from), Some(to)) = (from, to) {
                let et = match edge_type {
                    Some("conditional") => EdgeType::Conditional,
                    Some("parallel") => EdgeType::Parallel,
                    _ => EdgeType::Sequential,
                };
                workflow.edges.push(Edge::new(from, to).with_type(et));
            }
        }
    } else {
        // Auto-infer edges from depends_on
        for step in &workflow.steps {
            for dep in &step.dependencies {
                workflow
                    .edges
                    .push(Edge::new(dep.clone(), step.id.clone()).with_type(EdgeType::Sequential));
            }
        }
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No steps found in workflow definition");
    }

    Ok(workflow)
}

fn parse_step_value(
    step_val: &serde_yaml::Value,
    path: &Path,
    line: usize,
    fallback_id: &str,
) -> Step {
    let id = step_val
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or(fallback_id)
        .to_string();

    let name = step_val
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();

    let step_type = step_val
        .get("type")
        .and_then(|v| v.as_str())
        .map(parse_step_type)
        .unwrap_or(StepType::Step);

    let dependencies = parse_string_list(step_val.get("depends_on"));
    let description = step_val
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let children = step_val
        .get("children")
        .and_then(|v| v.as_sequence())
        .map(|children| {
            children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    parse_step_value(child, path, line + idx + 1, &format!("{id}_child_{idx}"))
                })
                .collect()
        })
        .unwrap_or_default();

    let mut step = Step::new(&id, &name, step_type)
        .with_source(SourceLocation::new(
            path.to_string_lossy().to_string(),
            line,
        ))
        .with_snippet(if description.is_empty() {
            format!("{id}: {name}")
        } else {
            format!("{id}: {name} -- {description}")
        })
        .with_dependencies(dependencies)
        .with_children(children);

    if let Some(status) = step_val
        .get("status")
        .and_then(|v| v.as_str())
        .and_then(parse_status)
    {
        step.status = status;
    }

    step
}

fn parse_step_type(value: &str) -> StepType {
    match value.to_lowercase().as_str() {
        "agent" => StepType::Agent,
        "tool" => StepType::Tool,
        "chain" => StepType::Chain,
        "task" => StepType::Task,
        "predict" => StepType::Predict,
        "retrieve" => StepType::Retrieve,
        "lambda" => StepType::Lambda,
        "module" => StepType::Module,
        "conditional" => StepType::Conditional,
        "loop" => StepType::Loop,
        "error_handler" | "errorhandler" => StepType::ErrorHandler,
        _ => StepType::Step,
    }
}

fn parse_status(value: &str) -> Option<Status> {
    match value.to_lowercase().as_str() {
        "ok" | "success" => Some(Status::Ok),
        "error" | "err" | "failed" => Some(Status::Error),
        "running" | "run" => Some(Status::Running),
        "waiting" | "pending" | "wait" => Some(Status::Waiting),
        "skipped" | "skip" => Some(Status::Skipped),
        _ => None,
    }
}

fn parse_string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_generic_yaml() {
        let content = r#"
name: "Data Pipeline"
description: "ETL pipeline"
steps:
  - id: load
    name: "Load CSV"
    type: tool
    depends_on: []
  - id: clean
    name: "Clean data"
    type: agent
    depends_on:
      - load
  - id: save
    name: "Save to DB"
    type: tool
    depends_on:
      - clean
"#;
        let path = std::path::Path::new("workflow.yaml");
        let w = parse(content, path).unwrap();
        assert_eq!(w.name, "Data Pipeline");
        assert_eq!(w.step_count(), 3);
        assert_eq!(w.edges.len(), 2);
    }

    #[test]
    fn test_parse_generic_json() {
        let content = r#"{
  "name": "Simple Workflow",
  "steps": [
    {"id": "a", "name": "Step A", "type": "step"},
    {"id": "b", "name": "Step B", "type": "step", "depends_on": ["a"]}
  ]
}"#;
        let path = std::path::Path::new("workflow.json");
        let w = parse(content, path).unwrap();
        assert_eq!(w.name, "Simple Workflow");
        assert_eq!(w.step_count(), 2);
    }

    #[test]
    fn test_parse_with_explicit_edges() {
        let content = r#"
name: "Parallel Workflow"
steps:
  - id: a
    name: "Task A"
    type: task
  - id: b
    name: "Task B"
    type: task
  - id: c
    name: "Merge"
    type: step
    depends_on:
      - a
      - b
edges:
  - from: a
    to: c
    type: parallel
  - from: b
    to: c
    type: parallel
"#;
        let path = std::path::Path::new("workflow.yaml");
        let w = parse(content, path).unwrap();
        assert_eq!(w.edges.len(), 2);
        assert!(w
            .edges
            .iter()
            .all(|e| matches!(e.edge_type, EdgeType::Parallel)));
    }

    #[test]
    fn test_parse_nested_status_steps() {
        let content = r#"
name: "Nested Workflow"
steps:
  - id: parent
    name: "Parent"
    type: loop
    status: running
    children:
      - id: child
        name: "Child"
        type: retrieve
        status: ok
"#;
        let path = std::path::Path::new("workflow.yaml");
        let w = parse(content, path).unwrap();

        assert_eq!(w.total_step_count(), 2);
        assert_eq!(w.steps[0].step_type, StepType::Loop);
        assert_eq!(w.steps[0].status, Status::Running);
        assert_eq!(w.steps[0].children[0].step_type, StepType::Retrieve);
        assert_eq!(w.steps[0].children[0].status, Status::Ok);
    }

    #[test]
    fn test_parse_empty() {
        let content = "other_key: value\n";
        let path = std::path::Path::new("workflow.yaml");
        let w = parse(content, path).unwrap();
        assert!(w.steps.is_empty());
        assert!(w.has_parse_errors());
    }
}
