use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use std::path::PathBuf;

use workflow_map::cli::{Cli, OutputFormat};
use workflow_map::config::AppConfig;
use workflow_map::export::Exporter;
use workflow_map::parsers::{detect_framework, parse_workflow};
use workflow_map::remote::{fetch_to_cache, is_remote_path};
use workflow_map::renderer::TuiApp;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if cli.no_config {
        AppConfig::default()
    } else {
        AppConfig::load().unwrap_or_default()
    };

    let mut path = PathBuf::from(&cli.path);

    if is_remote_path(&cli.path) {
        let cached = fetch_to_cache(&cli.path, cli.refresh)?;
        path = cached;
    }

    let mut parser_config = config.parsers.clone();
    if cli.no_cache {
        parser_config.disable_cache = true;
    }

    let framework = if let Some(fw) = &cli.framework {
        workflow_map::parsers::parse_framework_name(fw)
            .with_context(|| format!("Unknown framework: {fw}"))?
    } else {
        detect_framework(&parser_config, &path)
            .with_context(|| format!("Cannot detect framework for: {}", path.display()))?
    };

    let workflow = parse_workflow(framework, &parser_config, &path)
        .with_context(|| format!("Failed to parse workflow from: {}", path.display()))?;

    if workflow.has_parse_errors() {
        for err in workflow.parse_errors() {
            eprintln!("[parse warning] {err}");
        }
    }

    match cli.format {
        OutputFormat::Interactive => {
            let mut app = TuiApp::new(workflow, config);
            app.run()?;
        }
        OutputFormat::Text => {
            let exporter = Exporter::new(workflow);
            let output = exporter.to_text();
            match &cli.output {
                Some(path) => std::fs::write(path, output)?,
                None => print!("{output}"),
            }
        }
        OutputFormat::Json => {
            let exporter = Exporter::new(workflow);
            let output = exporter.to_json()?;
            match &cli.output {
                Some(path) => std::fs::write(path, output)?,
                None => println!("{output}"),
            }
        }
        OutputFormat::Markdown => {
            let exporter = Exporter::new(workflow);
            let output = exporter.to_markdown();
            match &cli.output {
                Some(path) => std::fs::write(path, output)?,
                None => print!("{output}"),
            }
        }
    }

    if let Some(graph) = cli.graph.as_deref() {
        let exporter = Exporter::new(workflow);
        let output = match graph {
            "dot" => exporter.to_dot(),
            "mermaid" => exporter.to_mermaid(),
            _ => String::new(),
        };
        if !output.is_empty() {
            match &cli.output {
                Some(path) => std::fs::write(path, output)?,
                None => print!("{output}"),
            }
        }
    }

    Ok(())
}
