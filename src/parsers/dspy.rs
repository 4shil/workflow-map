use crate::config::ParserConfig;
use crate::model::*;
use anyhow::Result;
use regex::Regex;
use std::path::Path;

pub fn parse(content: &str, path: &Path, _config: &ParserConfig) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "dspy-program".to_string()),
        Framework::DSPy,
    );

    let mut step_counter = 0;

    // Pattern 1: dspy.Module subclasses
    let module_re = Regex::new(
        r"(?m)class\s+(\w+)\s*\(\s*dspy\.Module\s*\)",
    ).ok();

    if let Some(ref re) = module_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let class_name = captures.get(1).map(|m| m.as_str()).unwrap_or("Module");

            // Find forward method to extract sub-steps
            let forward_content = extract_method_content(content, "forward");

            let mut step = Step::new(
                format!("dspy_module_{step_counter}"),
                class_name,
                StepType::Module,
            )
            .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
            .with_snippet(format!("class {class_name}(dspy.Module)"));

            // Parse forward() sub-calls
            if let Some(forward) = forward_content {
                let sub_steps = parse_dspy_forward(&forward, path);
                step = step.with_children(sub_steps);
            }

            workflow.steps.push(step);
        }
    }

    // Pattern 2: dspy.Predict / dspy.ChainOfThought direct usage
    let predict_re = Regex::new(
        r"(?m)\s*(\w+)\s*=\s*dspy\.(Predict|ChainOfThought|Retrieve)\s*\(",
    ).ok();

    if let Some(ref re) = predict_re {
        for captures in re.captures_iter(content) {
            step_counter += 1;
            let var_name = captures.get(1).map(|m| m.as_str()).unwrap_or("predict");
            let pred_type = captures.get(2).map(|m| m.as_str()).unwrap_or("Predict");

            let step_type = match pred_type {
                "Retrieve" => StepType::Retrieve,
                _ => StepType::Predict,
            };

            let sig_start = captures.get(0).map(|m| m.start()).unwrap_or(0);
            let line_num = content[..sig_start].lines().count();

            workflow.steps.push(
                Step::new(
                    format!("dspy_pred_{step_counter}"),
                    format!("{var_name} (dspy.{pred_type})"),
                    step_type,
                )
                .with_source(SourceLocation::new(path.to_string_lossy().to_string(), line_num))
                .with_snippet(format!("{var_name} = dspy.{pred_type}(...)")),
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

    // Extract until next method/class at same or lower indentation
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

fn parse_dspy_forward(content: &str, path: &Path) -> Vec<Step> {
    let mut steps = Vec::new();
    let call_re = Regex::new(r"(?m)\s*self\.(\w+)\s*\(").ok();

    if let Some(ref re) = call_re {
        for (i, captures) in re.captures_iter(content).enumerate() {
            let method_name = captures.get(1).map(|m| m.as_str()).unwrap_or("call");
            if method_name == "forward" {
                continue;
            }
            steps.push(
                Step::new(
                    format!("dspy_sub_{i}"),
                    method_name.to_string(),
                    StepType::Predict,
                )
                .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
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
        let content = r#"
import dspy

class RAG(dspy.Module):
    def __init__(self):
        self.retrieve = dspy.Retrieve(k=3)
        self.generate = dspy.ChainOfThought("context, question -> answer")

    def forward(self, question):
        context = self.retrieve(question)
        return self.generate(context=context, question=question)
"#;
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(!w.steps.is_empty());
        assert_eq!(w.framework, Framework::DSPy);
    }

    #[test]
    fn test_parse_dspy_predict() {
        let content = r#"
import dspy
qa = dspy.Predict("question -> answer")
"#;
        let path = std::path::Path::new("test.py");
        let w = parse(content, path, &ParserConfig::default()).unwrap();
        assert!(w.steps.iter().any(|s| s.name.contains("qa")));
    }
}
