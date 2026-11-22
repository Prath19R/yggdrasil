use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffStatus {
    Pending,
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileDiff {
    pub id: String,
    pub path: String,
    pub original_content: String,
    pub proposed_content: String,
    pub status: DiffStatus,
    pub agent_name: String,
    pub summary: String,
}

impl FileDiff {
    pub fn new(path: &str, original: &str, proposed: &str, agent: &str, summary: &str) -> Self {
        Self {
            id: js_sys::Date::now().to_string(),
            path: path.to_string(),
            original_content: original.to_string(),
            proposed_content: proposed.to_string(),
            status: DiffStatus::Pending,
            agent_name: agent.to_string(),
            summary: summary.to_string(),
        }
    }
}
