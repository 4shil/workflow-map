use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

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
    Conditional,
    Loop,
    ErrorHandler,
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
            StepType::Conditional => write!(f, "Conditional"),
            StepType::Loop => write!(f, "Loop"),
            StepType::ErrorHandler => write!(f, "ErrorHandler"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Ok,
    Error,
    Running,
    Waiting,
    Skipped,
}

impl Status {
    pub fn marker(&self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Error => "ERR",
            Status::Running => "RUN",
            Status::Waiting => "WAIT",
            Status::Skipped => "SKIP",
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Status::Error)
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Waiting
    }
}

/// Resource usage tracking for a step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub api_calls: Option<u32>,
    pub cost_usd: Option<f64>,
}

impl ResourceUsage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tokens(mut self, input: u64, output: u64) -> Self {
        self.tokens_in = Some(input);
        self.tokens_out = Some(output);
        self
    }

    pub fn with_api_calls(mut self, calls: u32) -> Self {
        self.api_calls = Some(calls);
        self
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_usd = Some(cost);
        self
    }

    pub fn total_tokens(&self) -> u64 {
        self.tokens_in.unwrap_or(0) + self.tokens_out.unwrap_or(0)
    }
}

impl fmt::Display for ResourceUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(t) = self.tokens_in {
            parts.push(format!("in:{t}"));
        }
        if let Some(t) = self.tokens_out {
            parts.push(format!("out:{t}"));
        }
        if let Some(c) = self.api_calls {
            parts.push(format!("calls:{c}"));
        }
        if let Some(c) = self.cost_usd {
            parts.push(format!("${c:.4}"));
        }
        if parts.is_empty() {
            write!(f, "none")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

/// Detected graph pattern for a group of steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphPattern {
    Pipeline,
    Diamond,
    FanIn,
    FanOut,
    DAG,
    Unknown,
}

impl fmt::Display for GraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphPattern::Pipeline => write!(f, "pipeline"),
            GraphPattern::Diamond => write!(f, "diamond"),
            GraphPattern::FanIn => write!(f, "fan-in"),
            GraphPattern::FanOut => write!(f, "fan-out"),
            GraphPattern::DAG => write!(f, "dag"),
            GraphPattern::Unknown => write!(f, "unknown"),
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    pub stack_trace: Option<String>,
    pub suggestion: Option<String>,
}

/// A single step (node) in a workflow graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// Timing information for this step.
    pub duration: Option<Duration>,
    /// Resource usage for this step.
    pub resources: ResourceUsage,
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
            duration: None,
            resources: ResourceUsage::new(),
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

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn with_resources(mut self, resources: ResourceUsage) -> Self {
        self.resources = resources;
        self
    }

    pub fn is_collapsed(&self) -> bool {
        self.collapsed
    }

    pub fn set_collapse(&mut self, collapsed: bool) {
        self.collapsed = collapsed
    }

    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed
    }

    pub fn depth(&self) -> usize {
        0
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    Sequential,
    Conditional,
    Parallel,
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub source_path: PathBuf,
    pub framework: Framework,
    pub steps: Vec<Step>,
    pub edges: Vec<Edge>,
    pub metadata: HashMap<String, String>,
    /// Detected graph pattern for the overall workflow.
    pub graph_pattern: GraphPattern,
    /// Validation warnings from configuration checking.
    pub validation_warnings: Vec<String>,
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
            graph_pattern: GraphPattern::Unknown,
            validation_warnings: Vec::new(),
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

    pub fn add_validation_warning(&mut self, warning: impl Into<String>) {
        self.validation_warnings.push(warning.into());
    }

    pub fn validation_warnings(&self) -> &[String] {
        &self.validation_warnings
    }

    pub fn has_validation_warnings(&self) -> bool {
        !self.validation_warnings.is_empty()
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn total_step_count(&self) -> usize {
        self.steps.iter().map(|s| s.visible_count()).sum()
    }

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
            duration: step.duration,
            resources: step.resources.clone(),
        });
        if !step.collapsed {
            for child in &step.children {
                Self::flatten_step(child, depth + 1, result);
            }
        }
    }

    pub fn error_steps(&self) -> Vec<FlatStep> {
        self.flatten_steps()
            .into_iter()
            .filter(|s| s.status.is_error())
            .collect()
    }

    pub fn running_steps(&self) -> Vec<FlatStep> {
        self.flatten_steps()
            .into_iter()
            .filter(|s| matches!(s.status, Status::Running))
            .collect()
    }

    /// Detect the overall graph pattern from edge topology.
    pub fn detect_graph_pattern(&mut self) {
        self.graph_pattern = Self::analyze_pattern(&self.edges);
    }

    fn analyze_pattern(edges: &[Edge]) -> GraphPattern {
        if edges.is_empty() {
            return GraphPattern::Unknown;
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut out_degree: HashMap<String, usize> = HashMap::new();

        for edge in edges {
            *out_degree.entry(edge.from.clone()).or_insert(0) += 1;
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let fan_out_count = out_degree.values().filter(|&&d| d > 1).count();
        let fan_in_count = in_degree.values().filter(|&&d| d > 1).count();

        if fan_out_count > 0 && fan_in_count > 0 {
            GraphPattern::Diamond
        } else if fan_out_count > 0 {
            GraphPattern::FanOut
        } else if fan_in_count > 0 {
            GraphPattern::FanIn
        } else if edges.len() > 2 {
            GraphPattern::Pipeline
        } else {
            GraphPattern::DAG
        }
    }
}

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
    pub duration: Option<Duration>,
    pub resources: ResourceUsage,
}

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
