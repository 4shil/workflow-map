use crate::model::{EdgeType, Workflow};
use anyhow::Result;

pub struct Exporter {
    workflow: Workflow,
}

impl Exporter {
    pub fn new(workflow: Workflow) -> Self {
        Self { workflow }
    }

    pub fn to_text(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        let name = &self.workflow.name;
        let fw = self.workflow.framework.to_string();
        lines.push(format!("Workflow: {name} ({fw})"));
        lines.push("\u{2500}".repeat(60));

        let flat = self.workflow.flatten_steps();
        let mut total: usize = 0;
        let mut ok_count: usize = 0;
        let mut err_count: usize = 0;
        let mut waiting_count: usize = 0;
        for step in &flat {
            total += 1;
            match step.status {
                crate::model::Status::Ok => ok_count += 1,
                crate::model::Status::Error => err_count += 1,
                crate::model::Status::Waiting => waiting_count += 1,
                _ => {}
            }
        }

        for (idx, step) in flat.iter().enumerate() {
            let step_num = idx + 1;
            let indent = "  ".repeat(step.depth);
            let collapse = if step.has_children {
                if step.collapsed { "[+]" } else { "[-]" }
            } else {
                "[ ]"
            };
            let error_prefix = if step.status.is_error() { "*" } else { "" };
            lines.push(format!("{indent}{collapse} {error_prefix}{step_num}. {}", step.name));
        }

        lines.push(String::new());
        lines.push(format!(
            "{total} steps, {ok_count} ok, {err_count} errors, {waiting_count} waiting"
        ));
        lines.join("\n")
    }

    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.workflow)?)
    }

    pub fn to_markdown(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("# {}", self.workflow.name));
        lines.push(String::new());
        lines.push(format!("Framework: {}", self.workflow.framework));
        lines.push(String::new());

        lines.push("## Table of Contents".to_string());
        lines.push(String::new());
        let flat = self.workflow.flatten_steps();
        for (idx, step) in flat.iter().enumerate() {
            let step_num = idx + 1;
            let anchor = step
                .name
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "-");
            lines.push(format!("{step_num}. [{}](#{})", step.name, anchor));
        }
        lines.push(String::new());

        for (idx, step) in flat.iter().enumerate() {
            let step_num = idx + 1;
            lines.push(format!("## {step_num}. {}", step.name));
            lines.push(String::new());
            lines.push(format!("- **Type:** {}", step.step_type));
            lines.push(format!("- **Status:** {}", step.status.marker()));
            lines.push(format!("- **Source:** {}", step.source_location));
            let snippet: &str = if step.config_snippet.len() > 200 {
                &step.config_snippet[..200]
            } else {
                &step.config_snippet
            };
            lines.push(format!("- **Description:** {snippet}"));
            if let Some(ref err) = step.error {
                lines.push(format!("- **Error:** {}", err.message));
                if let Some(ref suggestion) = err.suggestion {
                    lines.push(format!("- **Suggestion:** {suggestion}"));
                }
            }
            lines.push(String::new());
        }

        if !self.workflow.edges.is_empty() {
            lines.push("## Edges".to_string());
            lines.push(String::new());
            for edge in &self.workflow.edges {
                let edge_label = match edge.edge_type {
                    EdgeType::Sequential => "sequential",
                    EdgeType::Conditional => "conditional",
                    EdgeType::Parallel => "parallel",
                };
                lines.push(format!("- {} -> {} ({})", edge.from, edge.to, edge_label));
            }
            lines.push(String::new());
        }

        if !self.workflow.metadata.is_empty() {
            lines.push("## Metadata".to_string());
            lines.push(String::new());
            lines.push("| Key | Value |".to_string());
            lines.push("|-----|-------|".to_string());
            let mut keys: Vec<&String> = self.workflow.metadata.keys().collect();
            keys.sort();
            for key in keys {
                let value = &self.workflow.metadata[key];
                lines.push(format!("| {key} | {value} |"));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }
}
