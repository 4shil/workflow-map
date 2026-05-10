mod cli;
mod config;
mod export;
mod model;
mod parsers;
mod renderer;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use cli::{Cli, OutputFormat};
use config::AppConfig;
use export::Exporter;
use parsers::{detect_framework, parse_workflow};
use renderer::TuiApp;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load config file (if it exists and --no-config is not set)
    let config = if cli.no_config {
        AppConfig::default()
    } else {
        AppConfig::load().unwrap_or_default()
    };

    let path = PathBuf::from(&cli.path);

    // Parse the workflow
    let framework = if let Some(fw) = &cli.framework {
        parsers::parse_framework_name(fw)
            .with_context(|| format!("Unknown framework: {fw}"))?
    } else {
        detect_framework(&config.parsers, &path)
            .with_context(|| format!("Cannot detect framework for: {}", path.display()))?
    };

    let workflow = parse_workflow(framework, &config.parsers, &path)
        .with_context(|| format!("Failed to parse workflow from: {}", path.display()))?;

    // Report parse errors to stderr but continue
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

    Ok(())
}
