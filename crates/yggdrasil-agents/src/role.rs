use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

    pub fn tag(&self) -> &'static str {
        match self {
            AgentRole::Orchestrator => "orchestrator",
            AgentRole::Coder => "coder",
            AgentRole::Debugger => "debugger",
            AgentRole::Deployer => "deployer",
            AgentRole::SecurityAuditor => "security",
            AgentRole::TestEngineer => "tester",
            AgentRole::VisualInspector => "vision",
            AgentRole::DocsAgent => "docs",
        }
    }
}
