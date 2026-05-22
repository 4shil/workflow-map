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
        let mut tokens: u64 = 0;
        let mut api_calls: u32 = 0;
        let mut cost: f64 = 0.0;
        for step in &flat {
            total += 1;
            match step.status {
                crate::model::Status::Ok => ok_count += 1,
                crate::model::Status::Error => err_count += 1,
                crate::model::Status::Waiting => waiting_count += 1,
                _ => {}
            }
            tokens += step.resources.total_tokens();
            api_calls += step.resources.api_calls.unwrap_or(0);
            cost += step.resources.cost_usd.unwrap_or(0.0);
        }

        lines.push(format!("Pattern: {}", self.workflow.graph_pattern));
        if !self.workflow.validation_warnings.is_empty() {
            lines.push("Validation warnings:".to_string());
            for warning in &self.workflow.validation_warnings {
                lines.push(format!("  - {warning}"));
            }
            lines.push(String::new());
        }
        if self.workflow.has_parse_errors() {
            lines.push("Parse warnings:".to_string());
            for warning in self.workflow.parse_errors() {
                lines.push(format!("  - {warning}"));
            }
            lines.push(String::new());
        }

        for (idx, step) in flat.iter().enumerate() {
            let step_num = idx + 1;
            let indent = "  ".repeat(step.depth);
            let collapse = if step.has_children {
                if step.collapsed {
                    "[+]"
                } else {
                    "[-]"
                }
            } else {
                "[ ]"
            };
            let error_prefix = if step.status.is_error() { "*" } else { "" };
            lines.push(format!(
                "{indent}{collapse} {error_prefix}{step_num}. {}",
                step.name
            ));
        }

        lines.push(String::new());
        lines.push(format!(
            "{total} steps, {ok_count} ok, {err_count} errors, {waiting_count} waiting"
        ));
        if tokens > 0 || api_calls > 0 || cost > 0.0 {
            lines.push(format!(
                "Resources: {tokens} tokens, {api_calls} api calls, ${cost:.4}"
            ));
        }
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

        if !self.workflow.validation_warnings.is_empty() {
            lines.push("## Validation warnings".to_string());
            lines.push(String::new());
            for warning in &self.workflow.validation_warnings {
                lines.push(format!("- {warning}"));
            }
            lines.push(String::new());
        }

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
            if let Some(duration) = step.duration {
                lines.push(format!("- **Duration:** {:?}", duration));
            }
            if step.resources.total_tokens() > 0 || step.resources.api_calls.is_some() {
                lines.push(format!("- **Resources:** {}", step.resources));
            }
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
                lines.push(format!(
                    "| {} | {} |",
                    escape_markdown_table_cell(key),
                    escape_markdown_table_cell(value)
                ));
            }
            lines.push(String::new());
        }

        lines.join("\n")
    }

    pub fn to_dot(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("digraph workflow {".to_string());
        lines.push("  rankdir=LR;".to_string());
        lines.push("  node [shape=box, style=rounded];".to_string());

        for step in self.workflow.flatten_steps() {
            let label = format!("{}\\n{}", step.name, step.step_type);
            let color = match step.status {
                crate::model::Status::Ok => "#2e7d32",
                crate::model::Status::Error => "#c62828",
                crate::model::Status::Running => "#f9a825",
                crate::model::Status::Waiting => "#546e7a",
                crate::model::Status::Skipped => "#9e9e9e",
            };
            lines.push(format!(
                "  \"{}\" [label=\"{}\", color=\"{}\"];",
                escape_dot(&step.id),
                escape_dot(&label),
                color
            ));
        }

        for edge in &self.workflow.edges {
            let label = match edge.edge_type {
                EdgeType::Sequential => "seq",
                EdgeType::Conditional => "cond",
                EdgeType::Parallel => "par",
            };
            lines.push(format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];",
                escape_dot(&edge.from),
                escape_dot(&edge.to),
                label
            ));
        }

        lines.push("}".to_string());
        lines.join("\n")
    }

    pub fn to_mermaid(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("flowchart LR".to_string());
        for step in self.workflow.flatten_steps() {
            let node = format!(
                "{}[\"{}\"]",
                mermaid_id(&step.id),
                escape_mermaid_label(&step.name)
            );
            lines.push(format!("  {node}"));
        }
        for edge in &self.workflow.edges {
            let label = match edge.edge_type {
                EdgeType::Sequential => "-->",
                EdgeType::Conditional => "-->|cond|",
                EdgeType::Parallel => "-->|par|",
            };
            lines.push(format!(
                "  {} {} {}",
                mermaid_id(&edge.from),
                label,
                mermaid_id(&edge.to)
            ));
        }
        lines.join("\n")
    }
}

fn escape_dot(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_mermaid_label(input: &str) -> String {
    input.replace('"', "&quot;")
}

fn escape_markdown_table_cell(input: &str) -> String {
    input.replace('\\', "\\\\").replace('|', "\\|")
}

fn mermaid_id(input: &str) -> String {
    let mut output = String::from("n_");
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Edge, Framework, Step, StepType, Workflow};

    fn workflow_with_unsafe_names() -> Workflow {
        Workflow::new("unsafe", Framework::Generic)
            .with_steps(vec![
                Step::new("load:data", "Load \"data\"", StepType::Tool),
                Step::new("clean-data", "Clean | normalize", StepType::Step),
            ])
            .with_edges(vec![Edge::new("load:data", "clean-data")])
            .with_metadata("owner|team", "data|platform")
    }

    #[test]
    fn dot_escapes_labels_and_ids() {
        let dot = Exporter::new(workflow_with_unsafe_names()).to_dot();

        assert!(dot.contains("\"load:data\""));
        assert!(dot.contains("Load \\\"data\\\""));
    }

    #[test]
    fn mermaid_sanitizes_ids_and_escapes_labels() {
        let mermaid = Exporter::new(workflow_with_unsafe_names()).to_mermaid();

        assert!(mermaid.contains("n_load_data[\"Load &quot;data&quot;\"]"));
        assert!(mermaid.contains("n_load_data --> n_clean_data"));
    }

    #[test]
    fn markdown_escapes_metadata_table_cells() {
        let markdown = Exporter::new(workflow_with_unsafe_names()).to_markdown();

        assert!(markdown.contains("| owner\\|team | data\\|platform |"));
    }

    #[test]
    fn text_export_includes_validation_and_resource_summary() {
        let mut workflow = workflow_with_unsafe_names();
        workflow.add_validation_warning("Duplicate step id: load:data");
        workflow.steps[0].resources = crate::model::ResourceUsage::new()
            .with_tokens(10, 20)
            .with_api_calls(2)
            .with_cost(0.125);

        let text = Exporter::new(workflow).to_text();

        assert!(text.contains("Validation warnings:"));
        assert!(text.contains("Duplicate step id: load:data"));
        assert!(text.contains("Resources: 30 tokens, 2 api calls, $0.1250"));
    }
}
