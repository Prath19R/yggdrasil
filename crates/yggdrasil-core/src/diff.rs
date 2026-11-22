use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiffStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDiff {
    pub id: String,
    pub path: PathBuf,
    pub original_content: String,
    pub proposed_content: String,
    pub status: DiffStatus,
    pub agent_role: String,
    pub summary: String,
}

impl FileDiff {
    pub fn new(
        path: PathBuf,
        original_content: String,
        proposed_content: String,
        agent_role: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path,
            original_content,
            proposed_content,
            status: DiffStatus::Pending,
            agent_role: agent_role.into(),
            summary: summary.into(),
        }
    }

    pub fn accept(&mut self) -> std::io::Result<()> {
        std::fs::write(&self.path, &self.proposed_content)?;
        self.status = DiffStatus::Accepted;
        Ok(())
    }

    pub fn reject(&mut self) {
        self.status = DiffStatus::Rejected;
    }
}
