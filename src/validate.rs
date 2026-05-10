use crate::model::{Framework, Workflow};

pub fn validate(workflow: &mut Workflow) {
    match workflow.framework {
        Framework::CrewAI => validate_crewai(workflow),
        Framework::Hermes => validate_hermes(workflow),
        Framework::Generic => validate_generic(workflow),
        Framework::OpenAI => validate_openai(workflow),
        Framework::LlamaIndex => validate_llamaindex(workflow),
        _ => {}
    }
}

fn validate_crewai(workflow: &mut Workflow) {
    let has_agents = workflow
        .steps
        .iter()
        .any(|s| s.step_type.to_string() == "Agent");
    let has_tasks = workflow
        .steps
        .iter()
        .any(|s| s.step_type.to_string() == "Task");
    if !has_agents {
        workflow.add_validation_warning("CrewAI config missing agents".to_string());
    }
    if !has_tasks {
        workflow.add_validation_warning("CrewAI config missing tasks".to_string());
    }
}

fn validate_hermes(workflow: &mut Workflow) {
    if workflow.steps.is_empty() {
        workflow.add_validation_warning("Hermes skill has no sections".to_string());
    }
}

fn validate_generic(workflow: &mut Workflow) {
    if workflow.steps.is_empty() {
        workflow.add_validation_warning("Generic workflow has no steps".to_string());
    }
}

fn validate_openai(workflow: &mut Workflow) {
    let has_agent = workflow
        .steps
        .iter()
        .any(|s| s.step_type.to_string() == "Agent");
    if !has_agent {
        workflow.add_validation_warning("OpenAI Agents config missing Agent()".to_string());
    }
}

fn validate_llamaindex(workflow: &mut Workflow) {
    let has_workflow = workflow
        .steps
        .iter()
        .any(|s| s.step_type.to_string() == "Chain");
    if !has_workflow {
        workflow.add_validation_warning("LlamaIndex workflow missing Workflow()".to_string());
    }
}
