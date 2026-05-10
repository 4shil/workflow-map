use crate::model::*;
use anyhow::Result;
use std::path::Path;

/// Parse a CrewAI YAML config into a workflow.
pub fn parse(content: &str, path: &Path) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "crewai-workflow".to_string()),
        Framework::CrewAI,
    );

    let yaml: serde_yaml::Value = match serde_yaml::from_str(content) {
        Ok(v) => v,
        Err(e) => {
            workflow.add_parse_error(format!("YAML parse error: {e}"));
            return Ok(workflow);
        }
    };

    let mut step_id = 0;

    // Parse agents as steps
    if let Some(agents) = yaml.get("agents").and_then(|a| a.as_mapping()) {
        for (key, val) in agents {
            step_id += 1;
            let agent_name = key.as_str().unwrap_or("unknown");
            let role = val.get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("");

            let mut step = Step::new(
                format!("crew_agent_{step_id}"),
                format!("{agent_name} ({role})"),
                StepType::Agent,
            )
            .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
            .with_snippet(format!("{key:?}:"));

            // Add tools as children
            if let Some(tools) = val.get("tools").and_then(|t| t.as_sequence()) {
                let tool_children: Vec<Step> = tools
                    .iter()
                    .enumerate()
                    .map(|(i, tool)| {
                        let tool_name = tool.as_str()
                            .or_else(|| tool.as_mapping().and_then(|m| m.keys().next()?.as_str()))
                            .unwrap_or("tool");
                        Step::new(
                            format!("crew_agent_{step_id}_tool_{i}"),
                            tool_name.to_string(),
                            StepType::Tool,
                        )
                        .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    })
                    .collect();
                step = step.with_children(tool_children);
            }

            workflow.steps.push(step);
        }
    }

    // Parse tasks as steps with dependencies
    if let Some(tasks) = yaml.get("tasks").and_then(|t| t.as_mapping()) {
        for (key, val) in tasks {
            step_id += 1;
            let task_name = key.as_str().unwrap_or("unknown");
            let description = val.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let agent = val.get("agent")
                .and_then(|a| a.as_str())
                .unwrap_or("unassigned");

            let deps: Vec<String> = val.get("context")
                .and_then(|c| c.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let step = Step::new(
                format!("crew_task_{step_id}"),
                format!("{task_name} -> {agent}"),
                StepType::Task,
            )
            .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
            .with_snippet(format!("{task_name}: {description}"))
            .with_dependencies(deps);

            workflow.steps.push(step);

            // Create edges from agent to task
            workflow.edges.push(
                Edge::new(format!("crew_agent_{step_id}"), format!("crew_task_{step_id}"))
                    .with_type(EdgeType::Sequential),
            );
        }
    }

    // Detect process type
    let process_type = yaml.get("process")
        .and_then(|p| p.as_str())
        .unwrap_or("sequential");
    workflow.metadata.insert("process".to_string(), process_type.to_string());

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No agents or tasks found in CrewAI config");
    }

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_crewai_config() {
        let content = r#"
agents:
  researcher:
    role: "Senior Researcher"
    goal: "Find information"
    tools:
      - search_tool
  writer:
    role: "Content Writer"
    goal: "Write articles"

tasks:
  research_task:
    description: "Research the topic"
    agent: researcher
    expected_output: "Research notes"
  write_task:
    description: "Write the article"
    agent: writer
    context:
      - research_task
    expected_output: "Article"

process: sequential
"#;
        let path = std::path::Path::new("test.yaml");
        let workflow = parse(content, path).unwrap();
        assert_eq!(workflow.framework, Framework::CrewAI);
        assert!(workflow.step_count() >= 4); // 2 agents + 2 tasks
        assert!(workflow.metadata.contains_key("process"));
    }

    #[test]
    fn test_parse_malformed_yaml() {
        let content = "agents: [invalid\n";
        let path = std::path::Path::new("test.yaml");
        let workflow = parse(content, path).unwrap();
        assert!(workflow.has_parse_errors());
    }

    #[test]
    fn test_parse_empty_yaml() {
        let content = "other_key: value\n";
        let path = std::path::Path::new("test.yaml");
        let workflow = parse(content, path).unwrap();
        assert!(workflow.steps.is_empty());
        assert!(workflow.has_parse_errors());
    }
}
