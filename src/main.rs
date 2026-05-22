use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use std::path::PathBuf;

use workflow_map::cli::{Cli, OutputFormat};
use workflow_map::config::AppConfig;
use workflow_map::export::Exporter;
use workflow_map::parsers::{detect_framework, parse_workflow};
use workflow_map::remote::{fetch_to_cache, is_remote_path};
use workflow_map::renderer::TuiApp;
use workflow_map::timing::{apply_timing, load_timing};
use workflow_map::validate;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = if cli.no_config {
        AppConfig::default()
    } else if let Some(config_path) = cli.config.as_deref() {
        AppConfig::load_from(&PathBuf::from(config_path))
            .with_context(|| format!("Failed to load config from custom path: {config_path}"))?
    } else {
        AppConfig::load().unwrap_or_default()
    };

    if cli.no_color {
        config.render.respect_no_color = true;
        config.render.color_scheme = "none".to_string();
    }

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

    let mut workflow = parse_workflow(framework.clone(), &parser_config, &path)
        .with_context(|| format!("Failed to parse workflow from: {}", path.display()))?;

    if let Some(timing_path) = cli.timing_file.as_deref() {
        let timing_path = PathBuf::from(timing_path);
        if let Some(records) = load_timing(&timing_path) {
            apply_timing(&mut workflow.steps, &records);
        }
    }

    validate::validate(&mut workflow);

    if cli.strict && workflow.has_validation_warnings() {
        anyhow::bail!(
            "Workflow validation failed with {} warning(s): {}",
            workflow.validation_warnings().len(),
            workflow.validation_warnings().join("; ")
        );
    }

    if workflow.has_parse_errors() {
        for err in workflow.parse_errors() {
            eprintln!("[parse warning] {err}");
        }
    }

    let graph_format = match cli.format {
        OutputFormat::Dot => Some("dot"),
        OutputFormat::Mermaid => Some("mermaid"),
        _ => cli.graph.as_deref(),
    };

    if let Some(graph) = graph_format {
        let exporter = Exporter::new(workflow);
        let output = match graph {
            "dot" => exporter.to_dot(),
            "mermaid" => exporter.to_mermaid(),
            _ => unreachable!("graph format is validated by clap"),
        };
        match &cli.output {
            Some(path) => std::fs::write(path, output)?,
            None => print!("{output}"),
        }
        return Ok(());
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
        OutputFormat::Dot | OutputFormat::Mermaid => unreachable!("graph formats return earlier"),
    }

    Ok(())
}
