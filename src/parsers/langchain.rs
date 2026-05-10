use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

/// Count the line number (1-based) for a byte offset in content.
fn line_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset.min(content.len())]
        .lines()
        .count()
        .max(1)
}

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "langchain-workflow".to_string()),
        Framework::LangChain,
    );

    let mut step_counter = 0;
    let path_str = path.to_string_lossy().to_string();

    // Pattern 1: Pipe chains: `chain = ... | ...`
    let pipe_regex = Regex::new(r"(?m)^\s*(\w+)\s*=\s*(.+?)\|(.+?)(?:\s*(?:#.*)?)$").ok();

    if let Some(ref re) = pipe_regex {
        for mat in re.find_iter(content) {
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("chain");
            let left = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let right = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");
            let line = line_at(content, mat.start());

            step_counter += 1;
            let step_id = format!("lc_pipe_{step_counter}");

            let children = vec![
                Step::new(format!("{step_id}_a"), extract_name(left), classify(left))
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(left.to_string()),
                Step::new(format!("{step_id}_b"), extract_name(right), classify(right))
                    .with_source(SourceLocation::new(&path_str, line))
                    .with_snippet(right.to_string()),
            ];

            workflow.steps.push(
                Step::new(
                    &step_id,
                    format!("{var_name} (pipe chain)"),
                    StepType::Chain,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(format!("{var_name} = {left} | {right}"))
                .with_children(children),
            );
        }
    }

    // Pattern 2: AgentExecutor
    let agent_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*AgentExecutor\s*\(").ok();
    if let Some(ref re) = agent_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("agent");
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("lc_agent_{step_counter}"),
                    format!("{var_name} (AgentExecutor)"),
                    StepType::Agent,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(format!("{var_name} = AgentExecutor(...)")),
            );
        }
    }

    // Pattern 3: RunnableParallel
    let par_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*RunnableParallel\s*\(").ok();
    if let Some(ref re) = par_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("parallel");
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("lc_par_{step_counter}"),
                    format!("{var_name} (RunnableParallel)"),
                    StepType::Chain,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(format!("{var_name} = RunnableParallel(...)")),
            );
        }
    }

    // Pattern 4: RunnableLambda
    let lam_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*RunnableLambda\s*\(").ok();
    if let Some(ref re) = lam_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("lambda");
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("lc_lam_{step_counter}"),
                    format!("{var_name} (RunnableLambda)"),
                    StepType::Lambda,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(format!("{var_name} = RunnableLambda(...)")),
            );
        }
    }

    // Pattern 5: Conditional blocks (if/elif/else)
    let if_re = Regex::new(r"(?m)^(\s*)if\s+.+:").ok();
    if let Some(ref re) = if_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("lc_cond_{step_counter}"),
                    "Conditional branch".to_string(),
                    StepType::Conditional,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    // Pattern 6: Loop blocks (for/while)
    let loop_re = Regex::new(r"(?m)^(\s*)(for|while)\s+.+:").ok();
    if let Some(ref re) = loop_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let line = line_at(content, mat.start());
            workflow.steps.push(
                Step::new(
                    format!("lc_loop_{step_counter}"),
                    "Loop".to_string(),
                    StepType::Loop,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No LangChain patterns detected");
    }

    Ok(workflow)
}

fn extract_name(s: &str) -> String {
    let t = s.trim();
    if let Some(p) = t.find('(') {
        t[..p].trim().to_string()
    } else if let Some(d) = t[..t.len().min(40)].rfind('.') {
        t[..d].trim().to_string()
    } else {
        t[..t.len().min(30)].to_string()
    }
}

fn classify(s: &str) -> StepType {
    let l = s.to_lowercase();
    if l.contains("prompt") || l.contains("template") {
        StepType::Step
    } else if l.contains("llm") || l.contains("chat") || l.contains("openai") {
        StepType::Chain
    } else if l.contains("parser") || l.contains("output") {
        StepType::Tool
    } else if l.contains("retrieve") || l.contains("search") {
        StepType::Retrieve
    } else {
        StepType::Step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pipe() {
        let content = "chain = prompt | llm | parser\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(!w.steps.is_empty());
        assert_eq!(w.steps[0].source_location.line, 1);
    }

    #[test]
    fn test_line_numbers_accurate() {
        let content = "\n\nchain = prompt | llm\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(!w.steps.is_empty());
        assert_eq!(w.steps[0].source_location.line, 3);
    }

    #[test]
    fn test_conditional_detection() {
        let content = "if x > 0:\n    chain = prompt | llm\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w
            .steps
            .iter()
            .any(|s| matches!(s.step_type, StepType::Conditional)));
    }

    #[test]
    fn test_loop_detection() {
        let content = "for item in items:\n    chain = prompt | llm\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w
            .steps
            .iter()
            .any(|s| matches!(s.step_type, StepType::Loop)));
    }

    #[test]
    fn test_empty() {
        let content = "x = 1\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w.steps.is_empty());
    }
}
