use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

/// Represents the detected or specified framework type for a workflow source.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Framework {
    LangChain,
    CrewAI,
    DSPy,
    AutoGen,
    Hermes,
    Generic,
    Unknown,
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::LangChain => write!(f, "LangChain"),
            Framework::CrewAI => write!(f, "CrewAI"),
            Framework::DSPy => write!(f, "DSPy"),
            Framework::AutoGen => write!(f, "AutoGen"),
            Framework::Hermes => write!(f, "Hermes Agent"),
            Framework::Generic => write!(f, "Generic"),
            Framework::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Framework {
    /// All supported framework variants.
    pub fn all() -> &'static [Framework] {
        &[
            Framework::LangChain,
            Framework::CrewAI,
            Framework::DSPy,
            Framework::AutoGen,
            Framework::Hermes,
            Framework::Generic,
        ]
    }
}

/// The type of a single step in a workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    Agent,
    Tool,
    Chain,
    Task,
    Step,
    Predict,
    Retrieve,
    Lambda,
    Module,
}

impl fmt::Display for StepType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StepType::Agent => write!(f, "Agent"),
            StepType::Tool => write!(f, "Tool"),
            StepType::Chain => write!(f, "Chain"),
            StepType::Task => write!(f, "Task"),
            StepType::Step => write!(f, "Step"),
            StepType::Predict => write!(f, "Predict"),
            StepType::Retrieve => write!(f, "Retrieve"),
            StepType::Lambda => write!(f, "Lambda"),
            StepType::Module => write!(f, "Module"),
        }
    }
}

/// Execution status of a step. In v1, all steps default to Waiting
/// since the tool performs static analysis only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Error,
    Running,
    Waiting,
    Skipped,
}

impl Status {
    /// Returns the single-character marker used in ASCII rendering.
    pub fn marker(&self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Error => "ERR",
            Status::Running => "RUN",
            Status::Waiting => "WAIT",
            Status::Skipped => "SKIP",
        }
    }

    /// Returns true if this status represents a failure.
    pub fn is_error(&self) -> bool {
        matches!(self, Status::Error)
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Waiting
    }
}

/// Source file location for a step definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
}

impl SourceLocation {
    pub fn new(file: impl Into<PathBuf>, line: usize) -> Self {
        Self {
            file: file.into(),
            line,
            column: None,
        }
    }

    pub fn with_column(mut self, col: usize) -> Self {
        self.column = Some(col);
        self
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.file.display(), self.line)?;
        if let Some(col) = self.column {
            write!(f, ":{col}")?;
        }
        Ok(())
    }
}

/// Error detail associated with a failed step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    pub stack_trace: Option<String>,
    pub suggestion: Option<String>,
}

/// A single step (node) in a workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub status: Status,
    pub source_location: SourceLocation,
    pub config_snippet: String,
    pub error: Option<ErrorDetail>,
    pub children: Vec<Step>,
    pub dependencies: Vec<String>,
    collapsed: bool,
}

impl Step {
    pub fn new(id: impl Into<String>, name: impl Into<String>, step_type: StepType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            step_type,
            status: Status::default(),
            source_location: SourceLocation::new("unknown", 0),
            config_snippet: String::new(),
            error: None,
            children: Vec::new(),
            dependencies: Vec::new(),
            collapsed: false,
        }
    }

    pub fn with_status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn with_source(mut self, loc: SourceLocation) -> Self {
        self.source_location = loc;
        self
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.config_snippet = snippet.into();
        self
    }

    pub fn with_error(mut self, error: ErrorDetail) -> Self {
        self.status = Status::Error;
        self.error = Some(error);
        self
    }

    pub fn with_children(mut self, children: Vec<Step>) -> Self {
        self.children = children;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn set_collapse(&mut self, collapsed: bool) {
        self.collapsed = collapsed;
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed
    }

    /// Nesting depth: 0 if no parent context, incremented by callers.
    pub fn depth(&self) -> usize {
        // Depth is set by the parser; defaults to 0 for top-level steps.
        // Children inherit parent depth + 1.
        0
    }

    /// Total number of visible steps including children (respecting collapse).
    pub fn visible_count(&self) -> usize {
        let mut count = 1;
        if !self.collapsed {
            for child in &self.children {
                count += child.visible_count();
            }
        }
        count
    }
}

/// The type of connection between two steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Sequential,
    Conditional,
    Parallel,
}

/// A directed edge between two steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

impl Edge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            edge_type: EdgeType::Sequential,
        }
    }

    pub fn with_type(mut self, edge_type: EdgeType) -> Self {
        self.edge_type = edge_type;
        self
    }
}

/// The unified workflow model produced by all parsers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub source_path: PathBuf,
    pub framework: Framework,
    pub steps: Vec<Step>,
    pub edges: Vec<Edge>,
    pub metadata: HashMap<String, String>,
    parse_errors: Vec<String>,
}

impl Workflow {
    pub fn new(name: impl Into<String>, framework: Framework) -> Self {
        Self {
            name: name.into(),
            source_path: PathBuf::new(),
            framework,
            steps: Vec::new(),
            edges: Vec::new(),
            metadata: HashMap::new(),
            parse_errors: Vec::new(),
        }
    }

    pub fn with_source(mut self, path: impl Into<PathBuf>) -> Self {
        self.source_path = path.into();
        self
    }

    pub fn with_steps(mut self, steps: Vec<Step>) -> Self {
        self.steps = steps;
        self
    }

    pub fn with_edges(mut self, edges: Vec<Edge>) -> Self {
        self.edges = edges;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn add_parse_error(&mut self, error: impl Into<String>) {
        self.parse_errors.push(error.into());
    }

    pub fn parse_errors(&self) -> &[String] {
        &self.parse_errors
    }

    pub fn has_parse_errors(&self) -> bool {
        !self.parse_errors.is_empty()
    }

    /// Total number of top-level steps.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Total number of steps including all nested children.
    pub fn total_step_count(&self) -> usize {
        self.steps.iter().map(|s| s.visible_count()).sum()
    }

    /// Returns all steps flattened into a single list, respecting collapse state.
    pub fn flatten_steps(&self) -> Vec<FlatStep> {
        let mut result = Vec::new();
        for step in &self.steps {
            Self::flatten_step(step, 0, &mut result);
        }
        result
    }

    fn flatten_step(step: &Step, depth: usize, result: &mut Vec<FlatStep>) {
        result.push(FlatStep {
            id: step.id.clone(),
            name: step.name.clone(),
            step_type: step.step_type.clone(),
            status: step.status.clone(),
            source_location: step.source_location.clone(),
            config_snippet: step.config_snippet.clone(),
            error: step.error.clone(),
            depth,
            has_children: !step.children.is_empty(),
            collapsed: step.collapsed,
        });
        if !step.collapsed {
            for child in &step.children {
                Self::flatten_step(child, depth + 1, result);
            }
        }
    }

    /// Steps filtered to show only errors.
    pub fn error_steps(&self) -> Vec<FlatStep> {
        self.flatten_steps()
            .into_iter()
            .filter(|s| s.status.is_error())
            .collect()
    }

    /// Steps filtered to show only running.
    pub fn running_steps(&self) -> Vec<FlatStep> {
        self.flatten_steps()
            .into_iter()
            .filter(|s| matches!(s.status, Status::Running))
            .collect()
    }
}

/// A flattened representation of a step for rendering.
#[derive(Debug, Clone)]
pub struct FlatStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub status: Status,
    pub source_location: SourceLocation,
    pub config_snippet: String,
    pub error: Option<ErrorDetail>,
    pub depth: usize,
    pub has_children: bool,
    pub collapsed: bool,
}

/// Search filter for step names.
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    pub query: Option<String>,
}

impl SearchFilter {
    pub fn matches(&self, step: &FlatStep) -> bool {
        match &self.query {
            None => true,
            Some(q) => step.name.to_lowercase().contains(&q.to_lowercase()),
        }
    }
}

/// Display filter mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    FailedOnly,
    RunningOnly,
}

impl FilterMode {
    pub fn cycle(self) -> Self {
        match self {
            FilterMode::All => FilterMode::FailedOnly,
            FilterMode::FailedOnly => FilterMode::RunningOnly,
            FilterMode::RunningOnly => FilterMode::All,
        }
    }

    pub fn matches(self, status: &Status) -> bool {
        match self {
            FilterMode::All => true,
            FilterMode::FailedOnly => matches!(status, Status::Error),
            FilterMode::RunningOnly => matches!(status, Status::Running),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FilterMode::All => "all",
            FilterMode::FailedOnly => "failed",
            FilterMode::RunningOnly => "running",
        }
    }
}
