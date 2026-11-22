use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTCheckpoint {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub label: String,
    pub file_snapshots: HashMap<PathBuf, String>,
}

#[derive(Debug, Default, Clone)]
pub struct ASTHistoryManager {
    checkpoints: Vec<ASTCheckpoint>,
}

impl ASTHistoryManager {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
        }
    }

    pub fn capture(&mut self, label: impl Into<String>, snapshots: HashMap<PathBuf, String>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let checkpoint = ASTCheckpoint {
            id: id.clone(),
            timestamp: Utc::now(),
            label: label.into(),
            file_snapshots: snapshots,
        };
        self.checkpoints.push(checkpoint);
        id
    }

    pub fn revert_to(&mut self, checkpoint_id: &str) -> Option<&ASTCheckpoint> {
        self.checkpoints.iter().find(|c| c.id == checkpoint_id)
    }

    pub fn latest(&self) -> Option<&ASTCheckpoint> {
        self.checkpoints.last()
    }
}
