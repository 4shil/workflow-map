use crate::config::ParserConfig;
use crate::model::Framework;
use std::path::Path;

/// Detect the framework type for a given path (file or directory).
pub fn detect_framework(config: &ParserConfig, path: &Path) -> Option<Framework> {
    if path.is_dir() {
        // For directories, try to detect from the first recognizable file
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .collect();
            files.sort_by_key(|e| e.file_name());
            for entry in &files {
                if let Some(fw) = detect_file(config, &entry.path()) {
                    return Some(fw);
                }
            }
        }
        Some(Framework::Generic)
    } else {
        detect_file(config, path)
    }
}

/// Detect the framework type for a single file.
pub fn detect_file(_config: &ParserConfig, path: &Path) -> Option<Framework> {
    let ext = path.extension()?.to_str()?;
    let content = std::fs::read_to_string(path).unwrap_or_default();

    match ext {
        "py" => detect_python_framework(&content),
        "yaml" | "yml" => detect_yaml_framework(&content),
        "json" => Some(Framework::Generic),
        "md" => {
            if path.file_name()?.to_str()? == "SKILL.md" {
                Some(Framework::Hermes)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn detect_python_framework(content: &str) -> Option<Framework> {
    // Check for LangChain patterns
    if content.contains("langchain")
        && (content.contains("Chain")
            || content.contains("Runnable")
            || content.contains("LCEL")
            || content.contains("AgentExecutor")
            || content.contains("|")
                && (content.contains("PromptTemplate") || content.contains("ChatPromptTemplate")))
    {
        return Some(Framework::LangChain);
    }

    // Check for DSPy patterns
    if content.contains("dspy")
        && (content.contains("dspy.Module")
            || content.contains("dspy.Predict")
            || content.contains("dspy.ChainOfThought")
            || content.contains("dspy.Retrieve"))
    {
        return Some(Framework::DSPy);
    }

    // Check for AutoGen patterns
    if content.contains("autogen")
        && (content.contains("AssistantAgent")
            || content.contains("UserProxyAgent")
            || content.contains("GroupChat"))
    {
        return Some(Framework::AutoGen);
    }

    None
}

fn detect_yaml_framework(content: &str) -> Option<Framework> {
    // Check for CrewAI patterns
    if content.contains("agents:") && content.contains("tasks:") {
        return Some(Framework::CrewAI);
    }

    // Generic YAML workflow
    if content.contains("steps:") || content.contains("workflow:") {
        return Some(Framework::Generic);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_langchain() {
        let content = r#"
from langchain import Chain, PromptTemplate
chain = prompt | llm | parser
"#;
        assert_eq!(detect_python_framework(content), Some(Framework::LangChain));
    }

    #[test]
    fn test_detect_dspy() {
        let content = r#"
import dspy
class MyModule(dspy.Module):
    def forward(self, query):
        return dspy.Predict("query -> answer")
"#;
        assert_eq!(detect_python_framework(content), Some(Framework::DSPy));
    }

    #[test]
    fn test_detect_autogen() {
        let content = r#"
from autogen import AssistantAgent, UserProxyAgent
agent = AssistantAgent("assistant", llm_config={})
"#;
        assert_eq!(detect_python_framework(content), Some(Framework::AutoGen));
    }

    #[test]
    fn test_detect_crewai() {
        let content = r#"
agents:
  researcher:
    role: "Researcher"
tasks:
  research_task:
    description: "Research topic"
"#;
        assert_eq!(detect_yaml_framework(content), Some(Framework::CrewAI));
    }

    #[test]
    fn test_detect_generic_yaml() {
        let content = r#"
steps:
  - id: step1
    name: "Load data"
"#;
        assert_eq!(detect_yaml_framework(content), Some(Framework::Generic));
    }

    #[test]
    fn test_unknown_python() {
        let content = r#"
def hello():
    print("world")
"#;
        assert_eq!(detect_python_framework(content), None);
    }
}
