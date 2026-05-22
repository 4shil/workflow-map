use crate::model::{ErrorDetail, ResourceUsage, Status, Step};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct StatusFile {
    #[serde(flatten)]
    steps: HashMap<String, StatusOverride>,
}

#[derive(Debug, Deserialize)]
struct StatusOverride {
    status: Option<String>,
    error: Option<String>,
    suggestion: Option<String>,
    duration_ms: Option<u64>,
    tokens_in: Option<u64>,
    tokens_out: Option<u64>,
    api_calls: Option<u32>,
    cost_usd: Option<f64>,
}

pub fn apply_status_file(steps: &mut [Step], path: &Path) -> Result<usize> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read status file: {}", path.display()))?;
    let overrides: StatusFile = if path.extension().is_some_and(|ext| ext == "json") {
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse status file as JSON: {}", path.display()))?
    } else {
        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse status file as YAML: {}", path.display()))?
    };

    Ok(apply_overrides(steps, &overrides.steps))
}

fn apply_overrides(steps: &mut [Step], overrides: &HashMap<String, StatusOverride>) -> usize {
    let mut applied = 0;
    for step in steps {
        if let Some(override_data) = overrides.get(&step.id) {
            apply_override(step, override_data);
            applied += 1;
        }
        applied += apply_overrides(&mut step.children, overrides);
    }
    applied
}

fn apply_override(step: &mut Step, override_data: &StatusOverride) {
    if let Some(status) = override_data.status.as_deref().and_then(parse_status) {
        step.status = status;
    }

    if let Some(duration_ms) = override_data.duration_ms {
        step.duration = Some(Duration::from_millis(duration_ms));
    }

    if override_data.tokens_in.is_some()
        || override_data.tokens_out.is_some()
        || override_data.api_calls.is_some()
        || override_data.cost_usd.is_some()
    {
        step.resources = ResourceUsage {
            tokens_in: override_data.tokens_in,
            tokens_out: override_data.tokens_out,
            api_calls: override_data.api_calls,
            cost_usd: override_data.cost_usd,
        };
    }

    if let Some(message) = &override_data.error {
        step.status = Status::Error;
        step.error = Some(ErrorDetail {
            message: message.clone(),
            stack_trace: None,
            suggestion: override_data.suggestion.clone(),
        });
    }
}

fn parse_status(value: &str) -> Option<Status> {
    match value.to_ascii_lowercase().as_str() {
        "ok" | "success" | "passed" => Some(Status::Ok),
        "error" | "err" | "failed" | "failure" => Some(Status::Error),
        "running" | "run" => Some(Status::Running),
        "waiting" | "wait" | "pending" => Some(Status::Waiting),
        "skipped" | "skip" => Some(Status::Skipped),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Step, StepType};
    use std::io::Write;

    #[test]
    fn applies_json_status_file() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
  "load": {{
    "status": "ok",
    "duration_ms": 125,
    "tokens_in": 10,
    "tokens_out": 20,
    "api_calls": 1,
    "cost_usd": 0.003
  }},
  "clean": {{
    "error": "Column missing",
    "suggestion": "Check upstream schema"
  }}
}}"#
        )
        .unwrap();

        let path = file.path().with_extension("json");
        std::fs::copy(file.path(), &path).unwrap();

        let mut steps = vec![
            Step::new("load", "Load", StepType::Tool),
            Step::new("clean", "Clean", StepType::Step),
        ];

        let applied = apply_status_file(&mut steps, &path).unwrap();

        assert_eq!(applied, 2);
        assert_eq!(steps[0].status, Status::Ok);
        assert_eq!(steps[0].duration, Some(Duration::from_millis(125)));
        assert_eq!(steps[0].resources.total_tokens(), 30);
        assert_eq!(steps[1].status, Status::Error);
        assert_eq!(steps[1].error.as_ref().unwrap().message, "Column missing");
    }

    #[test]
    fn ignores_unknown_status_values() {
        assert_eq!(parse_status("unknown"), None);
    }
}
