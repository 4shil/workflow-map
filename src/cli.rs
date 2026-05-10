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

    /// Output file path (default: stdout for non-interactive).
    #[arg(short, long)]
    pub output: Option<String>,

    /// Watch mode: re-render on file changes.
    #[arg(short, long)]
    pub watch: bool,

    /// Force framework detection.
    #[arg(short, long, value_parser = ["langchain", "crewai", "dspy", "autogen", "hermes", "openai", "llamaindex", "generic"])]
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
}
