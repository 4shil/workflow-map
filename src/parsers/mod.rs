pub mod autogen;
pub mod crewai;
pub mod detector;
pub mod dspy;
pub mod generic;
pub mod hermes;
pub mod langchain;
pub mod llamaindex;
pub mod openai_agents;

use crate::config::ParserConfig;
use crate::model::Framework;
use anyhow::Result;
use std::path::Path;

/// Parse a workflow file/directory into a unified model.
pub fn parse_workflow(
    framework: Framework,
    config: &ParserConfig,
    path: &Path,
) -> Result<crate::model::Workflow> {
    if path.is_dir() {
        parse_directory(framework, config, path)
    } else {
        parse_file(framework, config, path)
    }
}

fn parse_file(
    framework: Framework,
    config: &ParserConfig,
    path: &Path,
) -> Result<crate::model::Workflow> {
    let content = std::fs::read_to_string(path)?;
    let mut workflow = match framework {
        Framework::LangChain => langchain::parse(&content, path, config)?,
        Framework::CrewAI => crewai::parse(&content, path)?,
        Framework::DSPy => dspy::parse(&content, path, config)?,
        Framework::AutoGen => autogen::parse(&content, path, config)?,
        Framework::Hermes => hermes::parse(&content, path)?,
        Framework::OpenAI => openai_agents::parse(&content, path)?,
        Framework::LlamaIndex => llamaindex::parse(&content, path)?,
        Framework::Generic => generic::parse(&content, path)?,
        Framework::Unknown => anyhow::bail!("Cannot parse unknown framework"),
    };
    workflow.source_path = path.to_path_buf();
    Ok(workflow)
}

fn parse_directory(
    default_framework: Framework,
    config: &ParserConfig,
    dir: &Path,
) -> Result<crate::model::Workflow> {
    let mut all_steps = Vec::new();
    let mut all_edges = Vec::new();
    let mut all_metadata = std::collections::HashMap::new();
    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workflow".to_string());

    let mut entries: Vec<_> = std::fs::read_dir(dir)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in &entries {
        let path = entry.path();
        if path.is_file() {
            let fw = if matches!(default_framework, Framework::Unknown) {
                detector::detect_file(&config, &path).unwrap_or(Framework::Unknown)
            } else {
                default_framework.clone()
            };

            match fw {
                Framework::Unknown => continue,
                _ => {
                    match parse_file(fw.clone(), config, &path) {
                        Ok(mut wf) => {
                            all_steps.append(&mut wf.steps);
                            all_edges.append(&mut wf.edges);
                            all_metadata.extend(wf.metadata);
                        }
                        Err(e) => {
                            // Create a synthetic error step for parse failures
                            use crate::model::*;
                            let file_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let error_step = Step::new(
                                format!("parse-error-{}", all_steps.len()),
                                file_name,
                                StepType::Step,
                            )
                            .with_status(Status::Error)
                            .with_source(SourceLocation::new(path.to_string_lossy().to_string(), 0))
                            .with_snippet(format!("{e}"))
                            .with_error(ErrorDetail {
                                message: format!("Failed to parse: {e}"),
                                stack_trace: None,
                                suggestion: Some(
                                    "Check file format and framework type".to_string(),
                                ),
                            });
                            all_steps.push(error_step);
                        }
                    }
                }
            }
        }
    }

    Ok(crate::model::Workflow::new(dir_name, Framework::Generic)
        .with_source(dir)
        .with_steps(all_steps)
        .with_edges(all_edges)
        .with_metadata("source", "mixed directory"))
}

/// Parse a framework name string into a Framework enum.
pub fn parse_framework_name(name: &str) -> Option<Framework> {
    match name.to_lowercase().as_str() {
        "langchain" => Some(Framework::LangChain),
        "crewai" => Some(Framework::CrewAI),
        "dspy" => Some(Framework::DSPy),
        "autogen" | "pyautogen" => Some(Framework::AutoGen),
        "hermes" => Some(Framework::Hermes),
        "openai" | "openai-agents" => Some(Framework::OpenAI),
        "llamaindex" | "llama-index" => Some(Framework::LlamaIndex),
        "generic" => Some(Framework::Generic),
        _ => None,
    }
}

/// Re-export the detector's main function.
pub use detector::detect_framework;
