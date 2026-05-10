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
            .unwrap_or_else(|| "llamaindex-workflow".to_string()),
        Framework::LlamaIndex,
    );

    let path_str = path.to_string_lossy().to_string();
    let mut step_counter = 0;

    let workflow_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*Workflow\s*\(")?;
    for mat in workflow_re.find_iter(content) {
        step_counter += 1;
        let line = line_at(content, mat.start());
        workflow.steps.push(
            Step::new(
                format!("li_workflow_{step_counter}"),
                format!("workflow_{step_counter}"),
                StepType::Chain,
            )
            .with_source(SourceLocation::new(&path_str, line))
            .with_snippet(mat.as_str().trim().to_string()),
        );
    }

    let step_re = Regex::new(r"(?m)^\s*class\s+(\w+)\s*\(\s*WorkflowStep\s*\)")?;
    for mat in step_re.find_iter(content) {
        step_counter += 1;
        let line = line_at(content, mat.start());
        workflow.steps.push(
            Step::new(
                format!("li_step_{step_counter}"),
                format!("step_{step_counter}"),
                StepType::Step,
            )
            .with_source(SourceLocation::new(&path_str, line))
            .with_snippet(mat.as_str().trim().to_string()),
        );
    }

    let engine_re =
        Regex::new(r"(?m)^\s*(\w+)\s*=\s*(QueryEngine|WorkflowRunner|AsyncWorkflowRunner)\s*\(")?;
    for mat in engine_re.find_iter(content) {
        step_counter += 1;
        let line = line_at(content, mat.start());
        workflow.steps.push(
            Step::new(
                format!("li_engine_{step_counter}"),
                format!("engine_{step_counter}"),
                StepType::Tool,
            )
            .with_source(SourceLocation::new(&path_str, line))
            .with_snippet(mat.as_str().trim().to_string()),
        );
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No LlamaIndex workflow patterns detected");
    }

    Ok(workflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_llamaindex() {
        let content = "from llama_index.core.workflow import Workflow, WorkflowStep\nclass StepA(WorkflowStep):\n    pass\n";
        let path = std::path::Path::new("li.py");
        let workflow = parse(content, path).unwrap();
        assert!(!workflow.steps.is_empty());
        assert_eq!(workflow.steps[0].source_location.line, 2);
    }
}
