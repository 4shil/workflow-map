use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

fn line_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset.min(content.len())].lines().count().max(1)
}

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "autogen-workflow".to_string()),
        Framework::AutoGen,
    );

    let mut step_counter = 0;
    let mut agent_names: Vec<String> = Vec::new();
    let path_str = path.to_string_lossy().to_string();

    let agent_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*AssistantAgent\s*\(").ok();
    if let Some(ref re) = agent_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("agent");
            agent_names.push(var_name.to_string());
            let line = line_at(content, mat.start());

            workflow.steps.push(
                Step::new(format!("ag_agent_{step_counter}"), format!("{var_name} (AssistantAgent)"), StepType::Agent)
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(format!("{var_name} = AssistantAgent(...)")),
            );
        }
    }

    let proxy_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*UserProxyAgent\s*\(").ok();
    if let Some(ref re) = proxy_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("proxy");
            agent_names.push(var_name.to_string());
            let line = line_at(content, mat.start());

            workflow.steps.push(
                Step::new(format!("ag_proxy_{step_counter}"), format!("{var_name} (UserProxyAgent)"), StepType::Agent)
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(format!("{var_name} = UserProxyAgent(...)")),
            );
        }
    }

    let gc_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*GroupChat\s*\(").ok();
    if let Some(ref re) = gc_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("groupchat");
            let line = line_at(content, mat.start());

            workflow.steps.push(
                Step::new(format!("ag_gc_{step_counter}"), format!("{var_name} (GroupChat)"), StepType::Chain)
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(format!("{var_name} = GroupChat(...)"))
                    .with_children(
                        agent_names.iter().enumerate().map(|(i, name)| {
                            Step::new(format!("ag_gc_{step_counter}_member_{i}"), name.clone(), StepType::Agent)
                        }).collect(),
                    ),
            );
        }
    }

    // Conditional detection
    let if_re = Regex::new(r"(?m)^(\s*)if\s+.+:").ok();
    if let Some(ref re) = if_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(format!("ag_cond_{step_counter}"), "Conditional branch".to_string(), StepType::Conditional)
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    // Loop detection
    let loop_re = Regex::new(r"(?m)^(\s*)(for|while)\s+.+:").ok();
    if let Some(ref re) = loop_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(format!("ag_loop_{step_counter}"), "Loop".to_string(), StepType::Loop)
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    for i in 0..agent_names.len().saturating_sub(1) {
        workflow.edges.push(
            Edge::new(format!("ag_agent_{}", i + 1), format!("ag_agent_{}", i + 2))
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
        let content = "from autogen import AssistantAgent, UserProxyAgent\n\nassistant = AssistantAgent(\"asst\", llm_config={})\nproxy = UserProxyAgent(\"proxy\", human_input_mode=\"NEVER\")\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert_eq!(w.framework, Framework::AutoGen);
        assert!(w.step_count() >= 2);
        assert_eq!(w.steps[0].source_location.line, 3);
    }

    #[test]
    fn test_conditional_detection() {
        let content = "if x > 0:\n    agent = AssistantAgent(\"a\", llm_config={})\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w.steps.iter().any(|s| matches!(s.step_type, StepType::Conditional)));
    }
}
