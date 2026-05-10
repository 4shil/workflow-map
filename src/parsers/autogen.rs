use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "autogen-workflow".to_string()),
        Framework::AutoGen,
    );

    let mut step_counter = 0;
    let mut agent_names: Vec<String> = Vec::new();

    // Pattern 1: AssistantAgent definitions
    let agent_re = Regex::new(
        r"(?m)\s*(\w+)\s*=\s*AssistantAgent\s*\(",
    ).ok();

    if let Some(ref re) = agent_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("agent");
            agent_names.push(var_name.to_string());

            workflow.steps.push(
                Step::new(
                    format!("ag_agent_{step_counter}"),
                    format!("{var_name} (AssistantAgent)"),
                    StepType::Agent,
                )
                .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                .with_snippet(format!("{var_name} = AssistantAgent(...)")),
            );
        }
    }

    // Pattern 2: UserProxyAgent definitions
    let proxy_re = Regex::new(
        r"(?m)\s*(\w+)\s*=\s*UserProxyAgent\s*\(",
    ).ok();

    if let Some(ref re) = proxy_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("proxy");
            agent_names.push(var_name.to_string());

            workflow.steps.push(
                Step::new(
                    format!("ag_proxy_{step_counter}"),
                    format!("{var_name} (UserProxyAgent)"),
                    StepType::Agent,
                )
                .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                .with_snippet(format!("{var_name} = UserProxyAgent(...)")),
            );
        }
    }

    // Pattern 3: GroupChat definitions
    let gc_re = Regex::new(
        r"(?m)\s*(\w+)\s*=\s*GroupChat\s*\(",
    ).ok();

    if let Some(ref re) = gc_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("groupchat");

            workflow.steps.push(
                Step::new(
                    format!("ag_gc_{step_counter}"),
                    format!("{var_name} (GroupChat)"),
                    StepType::Chain,
                )
                .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                .with_snippet(format!("{var_name} = GroupChat(...)"))
                .with_children(
                    agent_names
                        .iter()
                        .enumerate()
                        .map(|(i, name)| {
                            Step::new(
                                format!("ag_gc_{step_counter}_member_{i}"),
                                name.clone(),
                                StepType::Agent,
                            )
                        })
                        .collect(),
                ),
            );
        }
    }

    // Create edges between sequential agents
    for i in 0..agent_names.len().saturating_sub(1) {
        workflow.edges.push(
            Edge::new(
                format!("ag_agent_{}", i + 1),
                format!("ag_agent_{}", i + 2),
            )
            .with_type(EdgeType::Sequential),
        );
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No AutoGen patterns detected");
    }

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_autogen_agents() {
        let content = r#"
from autogen import AssistantAgent, UserProxyAgent, GroupChat

assistant = AssistantAgent("assistant", llm_config={"model": "gpt-4"})
proxy = UserProxyAgent("user_proxy", human_input_mode="NEVER")
group = GroupChat(agents=[assistant, proxy], messages=[])
"#;
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert_eq!(w.framework, Framework::AutoGen);
        assert!(w.step_count() >= 3);
    }
}
