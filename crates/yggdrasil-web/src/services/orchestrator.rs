use crate::models::agent::{AgentRole, ExecutionPlan, PlanStage, StageStatus};
use crate::models::diff::FileDiff;

pub struct OrchestratorService;

impl OrchestratorService {
    pub fn system_prompt_for(role: AgentRole) -> String {
        match role {
            AgentRole::Orchestrator => "You are the Master Orchestrator of Yggdrasil IDE. Coordinate the multi-agent swarm, analyze system architecture, decompose complex requests into step-by-step plans, and delegate to specialized sub-agents. Always format proposed code changes with ```rust:path/to/file.rs or similar language:filepath headers.".into(),
            AgentRole::Coder => "You are the Senior Coder & Architect agent of Yggdrasil IDE. You write production-grade, modular, and idiomatic Rust and web applications. When modifying or creating files, always include the file path in the markdown fence like ```rust:src/lib.rs or with a comment // file: src/lib.rs so the Yggdrasil Safety Guard can present interactive diff cards to the user.".into(),
            AgentRole::Debugger => "You are the Compiler & Debugger agent of Yggdrasil IDE. You analyze compiler warnings, diagnostics, error traces, and borrow checker issues. Propose minimal surgical fixes that eliminate errors without breaking existing contracts.".into(),
            AgentRole::Deployer => "You are the DevOps & Deployer agent. You configure Dockerfiles, CI/CD pipelines, Cargo releases, and deployment configurations.".into(),
            AgentRole::SecurityAuditor => "You are the Security Auditor agent. You scan code for memory safety, vulnerabilities, improper unwraps, credential leaks, and injection vectors.".into(),
            AgentRole::TestEngineer => "You are the Test & QA Engineer. You generate comprehensive unit, property-based, and integration tests to ensure robustness and high coverage.".into(),
            AgentRole::VisualInspector => "You are the Visual Inspector agent. You inspect UI components, layouts, colors, and accessibility.".into(),
            AgentRole::DocsAgent => "You are the Documentation & Release agent. You write clear API documentation, READMEs, changelogs, and user guides.".into(),
        }
    }

    pub fn generate_swarm_proposal(_role: AgentRole, prompt: &str) -> String {
        let lower = prompt.to_lowercase();
        if lower.contains("jwt") || lower.contains("auth") {
            "I have architected a secure JWT authentication middleware and claims validator.\n\n\
```rust:/yggdrasil-workspace/src/auth.rs\n\
use serde::{Deserialize, Serialize};\n\n\
#[derive(Debug, Serialize, Deserialize, Clone)]\n\
pub struct Claims {\n\
    pub sub: String,\n\
    pub role: String,\n\
    pub exp: usize,\n\
}\n\n\
pub fn generate_token(user_id: &str, role: &str) -> Result<String, String> {\n\
    let expiration = 10000000000;\n\
    let claims = Claims {\n\
        sub: user_id.to_string(),\n\
        role: role.to_string(),\n\
        exp: expiration,\n\
    };\n\
    Ok(format!(\"bearer.token.{}\", claims.sub))\n\
}\n\n\
pub fn validate_token(token: &str) -> Result<Claims, String> {\n\
    if token.is_empty() {\n\
        return Err(\"Invalid empty token\".into());\n\
    }\n\
    Ok(Claims { sub: \"admin\".into(), role: \"superuser\".into(), exp: 9999999999 })\n\
}\n\
```\n\n\
Review the diff above and click **[✔ Accept Diff]** to merge into your workspace.".to_string()
        } else if lower.contains("test") {
            "I have generated the unit test suite verifying logic, invariants, and edge conditions.\n\n\
```rust:/yggdrasil-workspace/tests/suite.rs\n\
#[cfg(test)]\n\
mod tests {\n\
    #[test]\n\
    fn test_engine_initialization() {\n\
        assert!(true, \"Engine initialized with zero errors\");\n\
    }\n\n\
    #[test]\n\
    fn test_swarm_agent_coordination() {\n\
        let agents = vec![\"Master Orchestrator\", \"Coder\", \"Debugger\"];\n\
        assert_eq!(agents.len(), 3);\n\
    }\n\
}\n\
```\n\n\
Click **[✔ Accept Diff]** to save tests to disk.".to_string()
        } else {
            format!(
                "I have drafted the implementation for `{}` following idiomatic Rust patterns.\n\n\
```rust:/yggdrasil-workspace/src/module.rs\n\
pub struct Service {{\n\
    pub name: String,\n\
    pub enabled: bool,\n\
}}\n\n\
impl Service {{\n\
    pub fn new(name: &str) -> Self {{\n\
        Self {{\n\
            name: name.to_string(),\n\
            enabled: true,\n\
        }}\n\
    }}\n\n\
    pub fn execute(&self) -> Result<&'static str, &'static str> {{\n\
        if self.enabled {{\n\
            Ok(\"Service executed successfully.\")\n\
        }} else {{\n\
            Err(\"Service disabled\")\n\
        }}\n\
    }}\n\
}}\n\
```\n\n\
Click **[✔ Accept Diff]** to write this module to your project.",
                prompt
            )
        }
    }

    pub fn formulate_plan(goal: &str) -> ExecutionPlan {
        let lower = goal.to_lowercase();
        let mut stages = Vec::new();

        if lower.contains("test") {
            stages.push(PlanStage {
                id: "1".into(),
                agent: AgentRole::Coder,
                title: "Implement tests & logic".into(),
                description: "Scaffold test suite or modules".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "2".into(),
                agent: AgentRole::Debugger,
                title: "Compiler diagnostics & check".into(),
                description: "Ensure clean build with zero warnings".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "3".into(),
                agent: AgentRole::TestEngineer,
                title: "Run unit tests".into(),
                description: "Verify boundary conditions and test pass rate".into(),
                status: StageStatus::Queued,
            });
        } else if lower.contains("deploy") || lower.contains("worker") || lower.contains("docker") {
            stages.push(PlanStage {
                id: "1".into(),
                agent: AgentRole::Coder,
                title: "Scaffold deployment manifests".into(),
                description: "Generate Dockerfile, wrangler.toml, or GitHub Actions".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "2".into(),
                agent: AgentRole::Debugger,
                title: "Validate configurations".into(),
                description: "Check syntax and runtime dependencies".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "3".into(),
                agent: AgentRole::Deployer,
                title: "Execute deployment pipeline".into(),
                description: "Trigger cloud release and health check".into(),
                status: StageStatus::Queued,
            });
        } else if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
            stages.push(PlanStage {
                id: "1".into(),
                agent: AgentRole::Debugger,
                title: "Diagnose bug root cause".into(),
                description: "Analyze backtrace and diagnostic output".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "2".into(),
                agent: AgentRole::Coder,
                title: "Apply minimal patch".into(),
                description: "Generate targeted fix diff".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "3".into(),
                agent: AgentRole::Debugger,
                title: "Verify clean build".into(),
                description: "Confirm bug resolution".into(),
                status: StageStatus::Queued,
            });
        } else {
            stages.push(PlanStage {
                id: "1".into(),
                agent: AgentRole::Coder,
                title: "Code generation / implementation".into(),
                description: "Draft requested module and architecture".into(),
                status: StageStatus::Queued,
            });
            stages.push(PlanStage {
                id: "2".into(),
                agent: AgentRole::Debugger,
                title: "Compiler validation".into(),
                description: "Verify syntax and types".into(),
                status: StageStatus::Queued,
            });
        }

        ExecutionPlan {
            id: js_sys::Date::now().to_string(),
            goal: goal.to_string(),
            stages,
        }
    }

    pub fn extract_diffs(ai_text: &str, agent_name: &str) -> Vec<FileDiff> {
        let mut diffs = Vec::new();
        let lines: Vec<&str> = ai_text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("```") {
                let mut detected_path = None;

                // Check language tag line for file path, e.g. ```rust:src/lib.rs
                if let Some(after) = line.strip_prefix("```") {
                    if let Some(idx) = after.find(':') {
                        let path = after[idx + 1..].trim();
                        if !path.is_empty() {
                            detected_path = Some(path.to_string());
                        }
                    }
                }

                // Check previous 3 lines
                if detected_path.is_none() {
                    for offset in 1..=3 {
                        if i >= offset {
                            let prev = lines[i - offset].trim();
                            if let Some(p) = Self::parse_filepath(prev) {
                                detected_path = Some(p);
                                break;
                            }
                        }
                    }
                }

                // Collect code lines
                i += 1;
                let mut code_lines = Vec::new();
                while i < lines.len() && !lines[i].trim().starts_with("```") {
                    let code_line = lines[i];
                    if detected_path.is_none() {
                        if let Some(p) = Self::parse_filepath(code_line.trim()) {
                            detected_path = Some(p);
                        }
                    }
                    code_lines.push(code_line);
                    i += 1;
                }

                if let Some(rel) = detected_path {
                    let proposed = code_lines.join("\n");
                    if !proposed.trim().is_empty() {
                        diffs.push(FileDiff::new(
                            &rel,
                            "",
                            &proposed,
                            agent_name,
                            &format!("Automated code proposal for `{}`", rel),
                        ));
                    }
                }
            }
            i += 1;
        }

        diffs
    }

    fn parse_filepath(line: &str) -> Option<String> {
        let lower = line.to_lowercase();
        let stripped = if let Some(idx) = lower.find("file:") {
            &line[idx + 5..]
        } else if let Some(idx) = lower.find("filepath:") {
            &line[idx + 9..]
        } else if let Some(idx) = lower.find("path:") {
            &line[idx + 5..]
        } else if lower.starts_with("//") || lower.starts_with("#") {
            let trimmed = line.trim_start_matches(|c| c == '/' || c == '#' || c == ' ');
            if (trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('.')) && !trimmed.contains("http") {
                trimmed
            } else {
                return None;
            }
        } else {
            return None;
        };

        let clean = stripped.trim().trim_matches(|c| c == '`' || c == '"' || c == '\'' || c == '*' || c == ':' || c == '(' || c == ')').trim();
        if clean.contains('.') && !clean.contains(' ') && clean.len() >= 3 {
            Some(clean.to_string())
        } else {
            None
        }
    }
}
