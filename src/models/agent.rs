use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Orchestrator,
    Coder,
    Debugger,
    Deployer,
    SecurityAuditor,
    TestEngineer,
    VisualInspector,
    DocsAgent,
}

impl AgentRole {
    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "👑 Master Orchestrator",
            AgentRole::Coder => "💻 Coder & Architect",
            AgentRole::Debugger => "🐞 Compiler & Debugger",
            AgentRole::Deployer => "🚀 DevOps & Deployer",
            AgentRole::SecurityAuditor => "🛡️ Security Auditor",
            AgentRole::TestEngineer => "🧪 Test & QA Engineer",
            AgentRole::VisualInspector => "👁️ Visual Inspector",
            AgentRole::DocsAgent => "📝 Docs & Release Agent",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "👑",
            AgentRole::Coder => "💻",
            AgentRole::Debugger => "🐞",
            AgentRole::Deployer => "🚀",
            AgentRole::SecurityAuditor => "🛡️",
            AgentRole::TestEngineer => "🧪",
            AgentRole::VisualInspector => "👁️",
            AgentRole::DocsAgent => "📝",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "Master Orchestrator",
            AgentRole::Coder => "Coder & Architect",
            AgentRole::Debugger => "Compiler & Debugger",
            AgentRole::Deployer => "DevOps & Deployer",
            AgentRole::SecurityAuditor => "Security Auditor",
            AgentRole::TestEngineer => "Test & QA Engineer",
            AgentRole::VisualInspector => "Visual Inspector",
            AgentRole::DocsAgent => "Docs & Release Agent",
        }
    }

    pub fn color_class(&self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "text-amber-400 bg-amber-500/10 border-amber-500/20",
            AgentRole::Coder => "text-blue-400 bg-blue-500/10 border-blue-500/20",
            AgentRole::Debugger => "text-red-400 bg-red-500/10 border-red-500/20",
            AgentRole::Deployer => "text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
            AgentRole::SecurityAuditor => "text-purple-400 bg-purple-500/10 border-purple-500/20",
            AgentRole::TestEngineer => "text-cyan-400 bg-cyan-500/10 border-cyan-500/20",
            AgentRole::VisualInspector => "text-pink-400 bg-pink-500/10 border-pink-500/20",
            AgentRole::DocsAgent => "text-zinc-400 bg-zinc-500/10 border-zinc-500/20",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StageStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanStage {
    pub id: String,
    pub agent: AgentRole,
    pub title: String,
    pub description: String,
    pub status: StageStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub goal: String,
    pub stages: Vec<PlanStage>,
}
