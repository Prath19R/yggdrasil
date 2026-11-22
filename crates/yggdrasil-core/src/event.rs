use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    PlanFormulated {
        plan_id: String,
        goal: String,
        stages: Vec<String>,
    },
    SubAgentDispatched {
        agent: String,
        task: String,
    },
    ReasoningStep {
        agent: String,
        thought: String,
    },
    DiffGenerated {
        diff_id: String,
        path: PathBuf,
        summary: String,
    },
    TerminalOutput {
        agent: String,
        line: String,
        is_error: bool,
    },
    ApprovalRequested {
        request_id: String,
        title: String,
        description: String,
    },
    TaskCompleted {
        agent: String,
        summary: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdeEvent {
    WorkspaceOpened(PathBuf),
    FileOpened(PathBuf),
    FileSaved(PathBuf),
    ModelSwitched { provider: String, model: String },
    AgentPromptSubmitted(String),
}
