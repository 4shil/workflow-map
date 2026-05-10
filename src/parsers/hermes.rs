use crate::model::*;
use anyhow::Result;
use std::path::Path;

/// Parse a Hermes Agent SKILL.md file into a workflow.
pub fn parse(content: &str, path: &Path) -> Result<Workflow> {
    let mut workflow = Workflow::new(
        path.parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "hermes-skill".to_string()),
        Framework::Hermes,
    );

    // Extract YAML frontmatter
    let (frontmatter, body) = extract_frontmatter(content);

    // Parse frontmatter fields
    if let Some(name) = frontmatter.get("name").and_then(|v| v.as_str()) {
        workflow.name = name.to_string();
        workflow
            .metadata
            .insert("name".to_string(), name.to_string());
    }
    if let Some(desc) = frontmatter.get("description").and_then(|v| v.as_str()) {
        workflow
            .metadata
            .insert("description".to_string(), desc.to_string());
    }
    if let Some(cat) = frontmatter.get("category").and_then(|v| v.as_str()) {
        workflow
            .metadata
            .insert("category".to_string(), cat.to_string());
    }

    // Parse body sections (## headers become steps)
    let mut step_counter = 0;
    let mut current_section: Option<(String, String)> = None;
    let mut section_lines: Vec<String> = Vec::new();

    for line in body.lines() {
        if line.starts_with("## ") {
            // Save previous section
            if let Some((ref title, _)) = current_section {
                step_counter += 1;
                let snippet = section_lines.join("\n").trim().to_string();
                workflow.steps.push(
                    Step::new(
                        format!("hermes_step_{step_counter}"),
                        title.clone(),
                        StepType::Step,
                    )
                    .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
                    .with_snippet(if snippet.len() > 200 {
                        format!("{}...", &snippet[..200])
                    } else {
                        snippet
                    }),
                );
            }
            current_section = Some((line[3..].trim().to_string(), String::new()));
            section_lines.clear();
        } else if current_section.is_some() {
            section_lines.push(line.to_string());
        }
    }

    // Don't forget the last section
    if let Some((ref title, _)) = current_section {
        step_counter += 1;
        let snippet = section_lines.join("\n").trim().to_string();
        workflow.steps.push(
            Step::new(
                format!("hermes_step_{step_counter}"),
                title.clone(),
                StepType::Step,
            )
            .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 1))
            .with_snippet(if snippet.is_empty() {
                "(empty section)".to_string()
            } else if snippet.len() > 200 {
                format!("{}...", &snippet[..200])
            } else {
                snippet
            }),
        );
    }

    // Create sequential edges between sections
    for i in 0..workflow.steps.len().saturating_sub(1) {
        workflow.edges.push(
            Edge::new(
                format!("hermes_step_{}", i + 1),
                format!("hermes_step_{}", i + 2),
            )
            .with_type(EdgeType::Sequential),
        );
    }

    if workflow.steps.is_empty() {
        workflow.add_parse_error("No ## sections found in SKILL.md body");
    }

    Ok(workflow)
}

/// Extract YAML frontmatter from a markdown file (delimited by ---).
fn extract_frontmatter(content: &str) -> (serde_yaml::Mapping, String) {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || lines[0].trim() != "---" {
        return (serde_yaml::Mapping::new(), content.to_string());
    }

    // Find the closing ---
    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(i);
            break;
        }
    }

    match end_idx {
        Some(end) => {
            let yaml_content = lines[1..end].join("\n");
            let body = lines[end + 1..].join("\n");

            let frontmatter =
                serde_yaml::from_str(&yaml_content).unwrap_or_else(|_| serde_yaml::Mapping::new());

            (frontmatter, body)
        }
        None => (serde_yaml::Mapping::new(), content.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_md() {
        let content = r#"---
name: test-skill
description: A test skill
category: testing
---

# Test Skill

## Setup
Install dependencies.

## Execution
Run the main logic.

## Cleanup
Clean up resources.
"#;
        let path = std::path::Path::new("SKILL.md");
        let w = parse(content, path).unwrap();
        assert_eq!(w.framework, Framework::Hermes);
        assert_eq!(w.name, "test-skill");
        assert_eq!(w.step_count(), 3);
        assert_eq!(w.edges.len(), 2);
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "# Just a doc\n\n## Section 1\nContent\n";
        let path = std::path::Path::new("SKILL.md");
        let w = parse(content, path).unwrap();
        assert_eq!(w.step_count(), 1);
    }

    #[test]
    fn test_parse_empty_skill() {
        let content = "---\nname: empty\n---\n# Nothing here\n";
        let path = std::path::Path::new("SKILL.md");
        let w = parse(content, path).unwrap();
        assert!(w.steps.is_empty());
        assert!(w.has_parse_errors());
    }
}
