use crate::model::{Framework, Step, Workflow};
use std::collections::HashSet;

pub fn validate(workflow: &mut Workflow) {
    validate_structure(workflow);

    match workflow.framework {
        Framework::CrewAI => validate_crewai(workflow),
        Framework::Hermes => validate_hermes(workflow),
        Framework::Generic => validate_generic(workflow),
        Framework::OpenAI => validate_openai(workflow),
        Framework::LlamaIndex => validate_llamaindex(workflow),
        _ => {}
    }
}

fn validate_structure(workflow: &mut Workflow) {
    let mut ids = HashSet::new();
    let mut duplicates = Vec::new();
    collect_step_ids(&workflow.steps, &mut ids, &mut duplicates);

    duplicates.sort();
    duplicates.dedup();
    for id in duplicates {
        workflow.add_validation_warning(format!("Duplicate step id: {id}"));
    }

    let edges = workflow.edges.clone();
    for edge in &edges {
        if !ids.contains(&edge.from) {
            workflow.add_validation_warning(format!(
                "Edge references missing source step: {}",
                edge.from
            ));
        }
        if !ids.contains(&edge.to) {
            workflow.add_validation_warning(format!(
                "Edge references missing target step: {}",
                edge.to
            ));
        }
    }

    let steps = workflow.steps.clone();
    for step in &steps {
        validate_dependencies(step, &ids, workflow);
    }
}

fn collect_step_ids(steps: &[Step], ids: &mut HashSet<String>, duplicates: &mut Vec<String>) {
    for step in steps {
        if !ids.insert(step.id.clone()) {
            duplicates.push(step.id.clone());
        }
        collect_step_ids(&step.children, ids, duplicates);
    }
}

fn validate_dependencies(step: &Step, ids: &HashSet<String>, workflow: &mut Workflow) {
    for dependency in &step.dependencies {
        if !ids.contains(dependency) {
            workflow.add_validation_warning(format!(
                "Step {} depends on missing step: {}",
                step.id, dependency
            ));
        }
    }

    for child in &step.children {
        validate_dependencies(child, ids, workflow);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Edge, Framework, Step, StepType, Workflow};

    #[test]
    fn reports_duplicate_step_ids() {
        let mut workflow = Workflow::new("dupes", Framework::Generic).with_steps(vec![
            Step::new("load", "Load", StepType::Step),
            Step::new("load", "Load again", StepType::Step),
        ]);

        validate(&mut workflow);

        assert!(workflow
            .validation_warnings()
            .iter()
            .any(|warning| warning == "Duplicate step id: load"));
    }

    #[test]
    fn reports_missing_edge_endpoints() {
        let mut workflow = Workflow::new("bad edge", Framework::Generic)
            .with_steps(vec![Step::new("load", "Load", StepType::Step)])
            .with_edges(vec![Edge::new("load", "missing")]);

        validate(&mut workflow);

        assert!(workflow
            .validation_warnings()
            .iter()
            .any(|warning| warning == "Edge references missing target step: missing"));
    }

    #[test]
    fn reports_missing_dependencies() {
        let mut workflow = Workflow::new("bad deps", Framework::Generic)
            .with_steps(vec![Step::new("clean", "Clean", StepType::Step)
                .with_dependencies(vec!["load".to_string()])]);

        validate(&mut workflow);

        assert!(workflow
            .validation_warnings()
            .iter()
            .any(|warning| warning == "Step clean depends on missing step: load"));
    }
}
