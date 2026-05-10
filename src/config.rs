use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Parser-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Enable regex-based Python parsing (v1 default).
    #[serde(default = "default_true")]
    pub use_regex_parser: bool,

    /// Maximum number of parse errors before aborting.
    #[serde(default = "default_max_errors")]
    pub max_parse_errors: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            use_regex_parser: true,
            max_parse_errors: 50,
        }
    }
}

/// Rendering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Color scheme name.
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,

    /// Respect NO_COLOR environment variable.
    #[serde(default = "default_true")]
    pub respect_no_color: bool,

    /// Status markers.
    #[serde(default)]
    pub status_markers: StatusMarkers,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            color_scheme: "default".to_string(),
            respect_no_color: true,
            status_markers: StatusMarkers::default(),
        }
    }
}

/// Customizable status markers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMarkers {
    pub ok: String,
    pub error: String,
    pub running: String,
    pub waiting: String,
    pub skipped: String,
}

impl Default for StatusMarkers {
    fn default() -> Self {
        Self {
            ok: "OK".to_string(),
            error: "ERR".to_string(),
            running: "RUN".to_string(),
            waiting: "WAIT".to_string(),
            skipped: "SKIP".to_string(),
        }
    }
}

/// Watch mode configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    /// Debounce interval in milliseconds.
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 500,
        }
    }
}

/// Application configuration loaded from config file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub parsers: ParserConfig,

    #[serde(default)]
    pub render: RenderConfig,

    #[serde(default)]
    pub watch: WatchConfig,

    /// Default export format.
    #[serde(default = "default_export_format")]
    pub default_export_format: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            parsers: ParserConfig::default(),
            render: RenderConfig::default(),
            watch: WatchConfig::default(),
            default_export_format: "text".to_string(),
        }
    }
}

impl AppConfig {
    /// Load config from the default location, or return None if not found.
    pub fn load() -> Option<Self> {
        let config_dir = dirs::config_dir()?;
        let config_path = config_dir.join("workflow-map").join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).ok()?;
            toml::from_str(&content).ok()
        } else {
            None
        }
    }

    /// Load config from a specific path.
    pub fn load_from(path: &PathBuf) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        toml::from_str(&content).ok()
    }
}

fn default_true() -> bool {
    true
}

fn default_max_errors() -> usize {
    50
}

fn default_color_scheme() -> String {
    "default".to_string()
}

fn default_debounce() -> u64 {
    500
}

fn default_export_format() -> String {
    "text".to_string()
}
