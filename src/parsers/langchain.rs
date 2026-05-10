use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "langchain-workflow".to_string()),
        Framework::LangChain,
    );

    let mut step_counter = 0;

    // Pattern 1: Pipe chains: `chain = ... | ...`
    let pipe_regex = Regex::new(
        r"(?m)^\s*(\w+)\s*=\s*(.+?)\|(.+?)(?:\s*(?:#.*)?)$",
    ).ok();

    if let Some(ref re) = pipe_regex {
        for captures in re.captures_iter(content) {
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("chain");
            let left = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            let right = captures.get(3).map(|m| m.as_str().trim()).unwrap_or("");

            step_counter += 1;
            let step_id = format!("lc_pipe_{step_counter}");

            let children = vec![
                Step::new(format!("{step_id}_a"), extract_name(left), classify(left))
                    .with_snippet(left.to_string()),
                Step::new(format!("{step_id}_b"), extract_name(right), classify(right))
                    .with_snippet(right.to_string()),
            ];

            workflow.steps.push(
                Step::new(&step_id, format!("{var_name} (pipe chain)"), StepType::Chain)
                    .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    .with_snippet(format!("{var_name} = {left} | {right}"))
                    .with_children(children),
            );
        }
    }

    // Pattern 2: AgentExecutor
    let agent_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*AgentExecutor\s*\(").ok();
    if let Some(ref re) = agent_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("agent");
            workflow.steps.push(
                Step::new(format!("lc_agent_{step_counter}"), format!("{var_name} (AgentExecutor)"), StepType::Agent)
                    .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    .with_snippet(format!("{var_name} = AgentExecutor(...)")),
            );
        }
    }

    // Pattern 3: RunnableParallel
    let par_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*RunnableParallel\s*\(").ok();
    if let Some(ref re) = par_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("parallel");
            workflow.steps.push(
                Step::new(format!("lc_par_{step_counter}"), format!("{var_name} (RunnableParallel)"), StepType::Chain)
                    .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    .with_snippet(format!("{var_name} = RunnableParallel(...)")),
            );
        }
    }

    // Pattern 4: RunnableLambda
    let lam_re = Regex::new(r"(?m)^\s*(\w+)\s*=\s*RunnableLambda\s*\(").ok();
    if let Some(ref re) = lam_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("lambda");
            workflow.steps.push(
                Step::new(format!("lc_lam_{step_counter}"), format!("{var_name} (RunnableLambda)"), StepType::Lambda)
                    .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    .with_snippet(format!("{var_name} = RunnableLambda(...)")),
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
    if let Some(p) = t.find('(') { t[..p].trim().to_string() }
    else if let Some(d) = t[..t.len().min(40)].rfind('.') { t[..d].trim().to_string() }
    else { t[..t.len().min(30)].to_string() }
}

fn classify(s: &str) -> StepType {
    let l = s.to_lowercase();
    if l.contains("prompt") || l.contains("template") { StepType::Step }
    else if l.contains("llm") || l.contains("chat") || l.contains("openai") { StepType::Chain }
    else if l.contains("parser") || l.contains("output") { StepType::Tool }
    else if l.contains("retrieve") || l.contains("search") { StepType::Retrieve }
    else { StepType::Step }
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
    }

    #[test]
    fn test_empty() {
        let content = "x = 1\n";
        let path = std::path::Path::new("t.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w.steps.is_empty());
    }
}
