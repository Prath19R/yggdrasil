use serde::{Deserialize, Serialize};
use yggdrasil_agents::AgentRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStage {
    pub id: String,
    pub agent: AgentRole,
    pub title: String,
    pub description: String,
    pub status: StageStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub goal: String,
    pub stages: Vec<PlanStage>,
}

impl ExecutionPlan {
    pub fn new(goal: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.into(),
            stages: Vec::new(),
        }
    }

    pub fn add_stage(&mut self, agent: AgentRole, title: impl Into<String>, desc: impl Into<String>) {
        self.stages.push(PlanStage {
            id: uuid::Uuid::new_v4().to_string(),
            agent,
            title: title.into(),
            description: desc.into(),
            status: StageStatus::Queued,
        });
    }
}
