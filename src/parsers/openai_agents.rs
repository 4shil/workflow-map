use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

fn line_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset.min(content.len())]
        .lines()
        .count()
        .max(1)
}

pub fn parse(content: &str, path: &Path) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "openai-agents".to_string()),
        Framework::OpenAI,
    );

    let mut step_counter = 0;
    let path_str = path.to_string_lossy().to_string();

    // Agent definitions
    let agent_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*Agent\s*\(").ok();
    if let Some(ref re) = agent_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("oa_agent_{step_counter}"),
                    format!("agent_{step_counter}"),
                    StepType::Agent,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    // Runner definitions
    let runner_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*(?:Async)?Runner\s*\(").ok();
    if let Some(ref re) = runner_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("oa_runner_{step_counter}"),
                    format!("runner_{step_counter}"),
                    StepType::Chain,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    // Handoff definitions
    let handoff_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*handoff\s*\(").ok();
    if let Some(ref re) = handoff_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("oa_handoff_{step_counter}"),
                    format!("handoff_{step_counter}"),
                    StepType::Chain,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    // Tool definitions
    let tool_re = Regex::new(r"(?m)\s*(\w+)\s*=\s*function_tool\s*\(").ok();
    if let Some(ref re) = tool_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("oa_tool_{step_counter}"),
                    format!("tool_{step_counter}"),
                    StepType::Tool,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No OpenAI Agents SDK patterns detected");
    }

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_openai_agent() {
        let content = "from openai import Agent\nassistant = Agent(name=\"Helper\", instructions=\"Be helpful\")\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path).unwrap();
        assert!(!w.steps.is_empty());
        assert_eq!(w.steps[0].source_location.line, 2);
    }
}
