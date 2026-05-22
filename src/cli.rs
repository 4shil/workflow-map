use clap::Parser as ClapParser;

/// Output format for the workflow map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    /// Interactive TUI (default).
    Interactive,
    /// Plain text ASCII diagram.
    Text,
    /// JSON representation.
    Json,
    /// Markdown document.
    Markdown,
    /// Graphviz DOT graph.
    Dot,
    /// Mermaid flowchart.
    Mermaid,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Interactive
    }
}

/// Terminal-native agentic workflow visualizer.
///
/// Parse LangChain, CrewAI, DSPy, AutoGen, and Hermes Agent configs
/// and render interactive ASCII workflow maps.
#[derive(ClapParser, Debug)]
#[command(
    name = "workflow-map",
    version,
    about = "Parse agentic workflow configs and render interactive ASCII diagrams",
    long_about = None,
)]
pub struct Cli {
    /// File or directory to parse.
    pub path: String,

    /// Output format.
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Interactive)]
    pub format: OutputFormat,

    /// Graph export format (dot, mermaid). Prefer --format dot|mermaid for new scripts.
    #[arg(long, value_parser = ["dot", "mermaid"])]
    pub graph: Option<String>,

    /// Output file path (default: stdout for non-interactive).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Watch mode: re-render on file changes.
    #[arg(short, long)]
    pub watch: bool,

    /// Disable parse cache.
    #[arg(long)]
    pub no_cache: bool,

    /// Force framework detection.
    #[arg(short = 'F', long, value_parser = ["langchain", "crewai", "dspy", "autogen", "hermes", "openai", "llamaindex", "generic"])]
    pub framework: Option<String>,

    /// Disable colored output.
    #[arg(long)]
    pub no_color: bool,

    /// Ignore config file.
    #[arg(long)]
    pub no_config: bool,

    /// Custom config file path.
    #[arg(long)]
    pub config: Option<String>,

    /// Refresh remote cache.
    #[arg(long)]
    pub refresh: bool,

    /// Load timing/resource data from CSV.
    #[arg(long)]
    pub timing_file: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn command_debug_asserts_pass() {
        Cli::command().debug_assert();
    }

    #[test]
    fn framework_uses_uppercase_short_flag() {
        let cli = Cli::parse_from(["workflow-map", "workflow.yaml", "-F", "generic"]);
        assert_eq!(cli.framework.as_deref(), Some("generic"));
    }

    #[test]
    fn graph_formats_are_output_formats() {
        let cli = Cli::parse_from(["workflow-map", "workflow.yaml", "--format", "dot"]);
        assert_eq!(cli.format, OutputFormat::Dot);

        let cli = Cli::parse_from(["workflow-map", "workflow.yaml", "--format", "mermaid"]);
        assert_eq!(cli.format, OutputFormat::Mermaid);
    }
}
