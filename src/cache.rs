use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::model::{Framework, Workflow};

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub workflow: Workflow,
    pub modified: SystemTime,
    pub framework: Framework,
}

#[derive(Debug, Default)]
pub struct WorkflowCache {
    entries: HashMap<(PathBuf, Framework), CacheEntry>,
}

impl WorkflowCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(
        &self,
        path: &Path,
        modified: SystemTime,
        framework: &Framework,
    ) -> Option<Workflow> {
        let key = (path.to_path_buf(), framework.clone());
        self.entries.get(&key).and_then(|entry| {
            if entry.modified == modified {
                Some(entry.workflow.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(
        &mut self,
        path: PathBuf,
        modified: SystemTime,
        framework: Framework,
        workflow: Workflow,
    ) {
        let key = (path, framework.clone());
        self.entries.insert(
            key,
            CacheEntry {
                workflow,
                modified,
                framework,
            },
        );
    }
}

pub fn file_mtime(path: &Path) -> Result<SystemTime> {
    Ok(fs::metadata(path)?.modified()?)
}
