use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

mod parser_integration {
    use super::*;
    use workflow_map::config::ParserConfig;
    use workflow_map::model::Framework;
    use workflow_map::parsers::{detect_framework, parse_workflow};

    #[test]
    fn test_langchain_simple_chain() {
        let path = fixture("langchain/simple_chain.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::LangChain);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn test_langchain_agent_executor() {
        let path = fixture("langchain/agent_executor.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::LangChain);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(wf.steps.iter().any(|s| s.name.contains("AgentExecutor")));
    }

    #[test]
    fn test_langchain_runnable_parallel() {
        let path = fixture("langchain/runnable_parallel.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::LangChain);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn test_crewai_data_pipeline() {
        let path = fixture("crewai/data_pipeline.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::CrewAI);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(wf.step_count() >= 4);
        assert!(wf.metadata.contains_key("process"));
    }

    #[test]
    fn test_dspy_rag_program() {
        let path = fixture("dspy/rag_program.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::DSPy);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn test_dspy_simple_predict() {
        let path = fixture("dspy/simple_predict.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::DSPy);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(!wf.steps.is_empty());
    }

    #[test]
    fn test_autogen_two_agents() {
        let path = fixture("autogen/two_agents.py");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::AutoGen);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert!(wf.step_count() >= 2);
    }

    #[test]
    fn test_hermes_skill() {
        let path = fixture("hermes/test_skill.md");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::Hermes);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert_eq!(wf.step_count(), 4);
        assert_eq!(wf.name, "test-skill");
    }

    #[test]
    fn test_generic_yaml_workflow() {
        let path = fixture("generic/simple_workflow.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        assert_eq!(fw, Framework::Generic);
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert_eq!(wf.step_count(), 3);
        assert_eq!(wf.name, "Simple Pipeline");
    }

    #[test]
    fn test_generic_json_workflow() {
        let path = fixture("generic/workflow.json");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert_eq!(wf.step_count(), 2);
    }

    #[test]
    fn test_generic_parallel_workflow() {
        let path = fixture("generic/parallel_workflow.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        assert_eq!(wf.step_count(), 3);
        assert_eq!(wf.edges.len(), 2);
    }

    #[test]
    fn test_mixed_directory() {
        let path = fixture("mixed");
        let wf = parse_workflow(Framework::Unknown, &ParserConfig::default(), &path).unwrap();
        assert!(!wf.steps.is_empty() || wf.has_parse_errors());
    }

    #[test]
    fn test_framework_forced_override() {
        let path = fixture("langchain/simple_chain.py");
        let wf = parse_workflow(Framework::Generic, &ParserConfig::default(), &path).unwrap();
        assert_eq!(wf.framework, Framework::Generic);
    }
}

mod export_integration {
    use super::*;
    use workflow_map::config::ParserConfig;
    use workflow_map::export::Exporter;
    use workflow_map::parsers::{detect_framework, parse_workflow};

    #[test]
    fn test_export_text_contains_header() {
        let path = fixture("crewai/data_pipeline.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        let exporter = Exporter::new(wf);
        let text = exporter.to_text();
        assert!(text.contains("Workflow:"));
        assert!(text.contains("steps"));
    }

    #[test]
    fn test_export_json_is_valid() {
        let path = fixture("generic/simple_workflow.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        let exporter = Exporter::new(wf);
        let json = exporter.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("steps").is_some());
    }

    #[test]
    fn test_export_markdown_structure() {
        let path = fixture("hermes/test_skill.md");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        let exporter = Exporter::new(wf);
        let md = exporter.to_markdown();
        assert!(md.contains("# test-skill"));
        assert!(md.contains("## Table of Contents"));
        assert!(md.contains("Framework:"));
    }

    #[test]
    fn test_export_all_formats_from_same_workflow() {
        let path = fixture("generic/parallel_workflow.yaml");
        let fw = detect_framework(&ParserConfig::default(), &path).unwrap();
        let wf = parse_workflow(fw, &ParserConfig::default(), &path).unwrap();
        let exporter = Exporter::new(wf);

        let _text = exporter.to_text();
        let _json = exporter.to_json().unwrap();
        let _md = exporter.to_markdown();
    }
}

mod error_handling {
    use super::*;
    use workflow_map::config::ParserConfig;
    use workflow_map::model::Framework;
    use workflow_map::parsers::parse_workflow;

    #[test]
    fn test_malformed_yaml_graceful() {
        let path = fixture("crewai/malformed.yaml");
        let result = parse_workflow(Framework::CrewAI, &ParserConfig::default(), &path);
        match result {
            Ok(wf) => assert!(wf.has_parse_errors()),
            Err(_) => {} // Also acceptable: hard error from serde_yaml
        }
    }

    #[test]
    fn test_empty_python_file() {
        let path = fixture("langchain/empty.py");
        let result = parse_workflow(Framework::LangChain, &ParserConfig::default(), &path).unwrap();
        assert!(result.steps.is_empty());
        assert!(result.has_parse_errors());
    }

    #[test]
    fn test_empty_yaml_file() {
        let path = fixture("crewai/empty.yaml");
        let result = parse_workflow(Framework::CrewAI, &ParserConfig::default(), &path).unwrap();
        assert!(result.steps.is_empty());
        assert!(result.has_parse_errors());
    }

    #[test]
    fn test_file_not_found() {
        let path = fixture("nonexistent/file.yaml");
        let result = parse_workflow(Framework::Generic, &ParserConfig::default(), &path);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_framework_file() {
        let path = fixture("generic/empty.yaml");
        let result = parse_workflow(Framework::Generic, &ParserConfig::default(), &path).unwrap();
        assert!(result.steps.is_empty());
        assert!(result.has_parse_errors());
    }

    #[test]
    fn test_malformed_python_graceful() {
        let path = fixture("langchain/malformed.py");
        let result = parse_workflow(Framework::LangChain, &ParserConfig::default(), &path);
        // Should either parse with errors or return Err, not panic
        match result {
            Ok(wf) => {
                // If it parsed, it should have no meaningful steps or have errors
                assert!(wf.steps.is_empty() || wf.has_parse_errors());
            }
            Err(_) => {}
        }
    }
}
