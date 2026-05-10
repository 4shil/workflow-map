use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::model::Workflow;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub workflow: Workflow,
    pub modified: SystemTime,
}

#[derive(Debug, Default)]
pub struct WorkflowCache {
    entries: HashMap<PathBuf, CacheEntry>,
}

impl WorkflowCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, path: &Path, modified: SystemTime) -> Option<Workflow> {
        self.entries.get(path).and_then(|entry| {
            if entry.modified == modified {
                Some(entry.workflow.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, path: PathBuf, modified: SystemTime, workflow: Workflow) {
        self.entries.insert(path, CacheEntry { workflow, modified });
    }
}

pub fn file_mtime(path: &Path) -> Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}
