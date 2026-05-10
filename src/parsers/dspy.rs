use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

fn line_at(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset.min(content.len())]
        .lines()
        .count()
        .wrapping_add(1)
        .max(1)
}

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "dspy-program".to_string()),
        Framework::DSPy,
    );

    let mut step_counter = 0;
    let path_str = path.to_string_lossy().to_string();

    let module_re = Regex::new(r"(?m)class\s+(\w+)\s*\(\s*dspy\.Module\s*\)").ok();

    if let Some(ref re) = module_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let class_name = captures.get(1).map(|m| m.as_str()).unwrap_or("Module");
            let line = line_at(content, mat.start());

            let forward_content = extract_method_content(content, "forward");

            let mut step = Step::new(
                format!("dspy_module_{step_counter}"),
                class_name,
                StepType::Module,
            )
            .with_source(SourceLocation::new(&path_str, line))
            .with_snippet(format!("class {class_name}(dspy.Module)"));

            if let Some(forward) = forward_content {
                let sub_steps = parse_dspy_forward(&forward, path, &path_str);
                step = step.with_children(sub_steps);
            }

            workflow.steps.push(step);
        }
    }

    let predict_re =
        Regex::new(r"(?m)\s*(\w+)\s*=\s*dspy\.(Predict|ChainOfThought|Retrieve)\s*\(").ok();

    if let Some(ref re) = predict_re {
        for mat in re.find_iter(content) {
            step_counter += 1;
            let captures = re.captures(mat.as_str()).unwrap();
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("predict");
            let pred_type = captures.get(2).map(|m| m.as_str()).unwrap_or("Predict");
            let line = line_at(content, mat.start());

            let step_type = match pred_type {
                "Retrieve" => StepType::Retrieve,
                _ => StepType::Predict,
            };

            workflow.steps.push(
                Step::new(
                    format!("dspy_pred_{step_counter}"),
                    format!("{var_name} (dspy.{pred_type})"),
                    step_type,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(format!("{var_name} = dspy.{pred_type}(...)")),
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
                Step::new(
                    format!("dspy_cond_{step_counter}"),
                    "Conditional branch".to_string(),
                    StepType::Conditional,
                )
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
                Step::new(
                    format!("dspy_loop_{step_counter}"),
                    "Loop".to_string(),
                    StepType::Loop,
                )
                .with_source(SourceLocation::new(&path_str, line))
                .with_snippet(mat.as_str().trim().to_string()),
            );
        }
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No DSPy patterns detected");
    }

    Ok(workflow)
}

fn extract_method_content(content: &str, method_name: &str) -> Option<String> {
    let pattern = format!(r"def\s+{method_name}\s*\([^)]*\)\s*(?:->\s*\w+\s*)?:");
    let re = Regex::new(&pattern).ok()?;
    let mat = re.find(content)?;
    let start = mat.end();
    let rest = &content[start..];
    let mut lines = Vec::new();
    for line in rest.lines() {
        if line.trim().is_empty() {
            lines.push(line.to_string());
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if indent == 0 && (line.starts_with("def ") || line.starts_with("class ")) {
            break;
        }
        lines.push(line.to_string());
    }
    Some(lines.join("\n"))
}

fn parse_dspy_forward(content: &str, _path: &Path, path_str: &str) -> Vec<Step> {
    let mut steps = Vec::new();
    let call_re = Regex::new(r"(?m)\s*self\.(\w+)\s*\(").ok();
    if let Some(ref re) = call_re {
        for (i, mat) in re.find_iter(content).enumerate() {
            let captures = re.captures(mat.as_str()).unwrap();
            let method_name = captures.get(1).map(|m| m.as_str()).unwrap_or("call");
            if method_name == "forward" {
                continue;
            }
            let line = line_at(content, mat.start());
            steps.push(
                Step::new(
                    format!("dspy_sub_{i}"),
                    method_name.to_string(),
                    StepType::Predict,
                )
                .with_source(SourceLocation::new(path_str, line))
                .with_snippet(format!("self.{method_name}(...)")),
            );
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dspy_module() {
        let content =
            "import dspy\n\nclass RAG(dspy.Module):\n    def forward(self, q):\n        pass\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(!w.steps.is_empty());
        assert_eq!(w.framework, Framework::DSPy);
        assert_eq!(w.steps[0].source_location.line, 3);
    }

    #[test]
    fn test_parse_dspy_predict() {
        let content = "import dspy\nqa = dspy.Predict(\"q -> a\")\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w.steps.iter().any(|s| s.name.contains("qa")));
        assert_eq!(w.steps[0].source_location.line, 2);
    }

    #[test]
    fn test_conditional_detection() {
        let content = "if x > 0:\n    qa = dspy.Predict(\"q -> a\")\n";
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w
            .steps
            .iter()
            .any(|s| matches!(s.step_type, StepType::Conditional)));
    }
}
