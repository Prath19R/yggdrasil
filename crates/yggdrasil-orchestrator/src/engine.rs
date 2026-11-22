use crate::plan::{ExecutionPlan, StageStatus};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use yggdrasil_agents::{get_system_prompt, AgentRole};
use yggdrasil_core::diff::FileDiff;
use yggdrasil_inference::types::{InferenceMode, MessageContent, UnifiedChatRequest, UnifiedMessage};
use yggdrasil_inference::InferenceRouter;
use yggdrasil_project::ProjectManager;
use yggdrasil_terminal::TerminalRunner;

#[derive(Debug, Clone)]
pub struct StageResult {
    pub text: String,
    pub thinking: Option<String>,
    pub proposed_diffs: Vec<FileDiff>,
}

pub struct OrchestratorEngine {
    pub router: Arc<Mutex<InferenceRouter>>,
    pub project: Arc<Mutex<ProjectManager>>,
    pub pending_diffs: Arc<Mutex<Vec<FileDiff>>>,
}

impl OrchestratorEngine {
    pub fn new(router: Arc<Mutex<InferenceRouter>>, project: Arc<Mutex<ProjectManager>>) -> Self {
        Self {
            router,
            project,
            pending_diffs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn formulate_plan(&self, user_instruction: &str) -> ExecutionPlan {
        let mut plan = ExecutionPlan::new(user_instruction);
        let lower = user_instruction.to_lowercase();

        if lower.contains("test") {
            plan.add_stage(AgentRole::Coder, "Implement code/tests", "Scaffold logic or test module");
            plan.add_stage(AgentRole::Debugger, "Run cargo check", "Ensure zero compiler errors");
            plan.add_stage(AgentRole::TestEngineer, "Run cargo test", "Execute and verify test cases");
        } else if lower.contains("deploy") || lower.contains("worker") || lower.contains("docker") {
            plan.add_stage(AgentRole::Coder, "Prepare build & config", "Create or edit deployment config");
            plan.add_stage(AgentRole::Debugger, "Validate configuration", "Check syntax and dependencies");
            plan.add_stage(AgentRole::Deployer, "Trigger deployment pipeline", "Execute release or container build");
        } else if lower.contains("fix") || lower.contains("bug") || lower.contains("error") {
            plan.add_stage(AgentRole::Debugger, "Diagnose error root cause", "Parse diagnostics and logs");
            plan.add_stage(AgentRole::Coder, "Apply patch", "Generate minimal bug fix diff");
            plan.add_stage(AgentRole::Debugger, "Verify fix", "Re-run compiler to confirm clean build");
        } else {
            plan.add_stage(AgentRole::Coder, "Code generation / refactor", "Draft code implementation");
            plan.add_stage(AgentRole::Debugger, "Compiler check", "Verify syntax and types with cargo check");
        }

        plan
    }

    pub async fn execute_stage(
        &self,
        stage_idx: usize,
        plan: &mut ExecutionPlan,
    ) -> Result<StageResult, String> {
        if stage_idx >= plan.stages.len() {
            return Err("Stage index out of bounds".into());
        }

        let stage = &mut plan.stages[stage_idx];
        stage.status = StageStatus::Running;
        let role = stage.agent;
        let title = stage.title.clone();

        let (root_path, proj_type_name) = {
            let p = self.project.lock().await;
            (p.root_path.clone(), p.project_type.display_badge().to_string())
        };

        // If stage is Debugger and root_path exists, run cargo check first to feed into AI
        let mut compiler_diag = String::new();
        if role == AgentRole::Debugger {
            if let Some(dir) = &root_path {
                if let Ok(output) = TerminalRunner::run_cargo_check(dir).await {
                    compiler_diag = format!(
                        "\n\n[Real-time Compiler Output]\nStatus: {}\nStdout: {}\nStderr: {}",
                        if output.success { "Clean Build (0 errors)" } else { "Build Errors Detected" },
                        output.stdout,
                        output.stderr
                    );
                }
            }
        }

        let context_str = format!(
            "Workspace: {:?}\nProject Type: {}\nCurrent Goal: {}\nCurrent Task: {}{}",
            root_path, proj_type_name, plan.goal, title, compiler_diag
        );

        let system_prompt = get_system_prompt(role, &context_str);

        // Check if API key is missing when in CloudApi mode
        let (has_key, mode, active_model) = {
            let r = self.router.lock().await;
            (r.has_api_key(), r.mode, r.active_model.clone())
        };

        if mode == InferenceMode::CloudApi && !has_key {
            stage.status = StageStatus::Completed;
            let msg = format!(
                "⚠️ [NVIDIA NIM Key Required]\nNo API key is configured yet for Cloud Mode.\n\
                 Click '🔴 Configure API Key' in the top bar to enter your free key (from https://build.nvidia.com), \
                 or toggle the top-right dropdown to '💻 Local Mode (Ollama)'.\n\n\
                 Current Stage: [{}] {}",
                role.name(),
                title
            );
            return Ok(StageResult {
                text: msg,
                thinking: None,
                proposed_diffs: Vec::new(),
            });
        }

        let user_msg = format!(
            "Project Context:\n{}\n\nGoal: {}\nStage: {}\nTask: {}\n\
             Please execute this task thoroughly. If writing or proposing code changes, output exact file paths (e.g. `// File: src/example.rs`) and complete code blocks.",
            context_str, plan.goal, stage.title, stage.description
        );

        let req = UnifiedChatRequest {
            model: active_model,
            messages: vec![
                UnifiedMessage {
                    role: "system".into(),
                    content: MessageContent::Text(system_prompt),
                },
                UnifiedMessage {
                    role: "user".into(),
                    content: MessageContent::Text(user_msg),
                },
            ],
            temperature: 0.3,
            max_tokens: 4096,
            stream: false,
            enable_thinking: true,
            reasoning_effort: Some("high".into()),
        };

        let ai_result = {
            let r = self.router.lock().await;
            r.chat(req).await
        };

        stage.status = StageStatus::Completed;

        match ai_result {
            Ok(resp) => {
                let body_text = if resp.text.trim().is_empty() {
                    resp.thinking.clone().unwrap_or_else(|| "Completed with no output.".to_string())
                } else {
                    resp.text.clone()
                };

                let full_text = if compiler_diag.is_empty() {
                    format!("⚡ [{}]\n{}", role.name(), body_text)
                } else {
                    format!("⚡ [{}]\n{}\n\n{}", role.name(), compiler_diag, body_text)
                };

                let mut diffs = Vec::new();
                if let Some(root) = &root_path {
                    diffs = Self::extract_code_diffs(root, &resp.text, role.name());
                    if diffs.is_empty() {
                        if let Some(thought) = &resp.thinking {
                            diffs = Self::extract_code_diffs(root, thought, role.name());
                        }
                    }
                }

                Ok(StageResult {
                    text: full_text,
                    thinking: resp.thinking,
                    proposed_diffs: diffs,
                })
            }
            Err(e) => {
                let err_text = format!(
                    "⚡ [{}] Execution Note:\nAI Endpoint: {}\n{}",
                    role.name(),
                    e,
                    compiler_diag
                );
                Ok(StageResult {
                    text: err_text,
                    thinking: None,
                    proposed_diffs: Vec::new(),
                })
            }
        }
    }

    fn extract_code_diffs(root: &std::path::Path, ai_text: &str, agent_name: &str) -> Vec<FileDiff> {
        let mut diffs = Vec::new();
        let lines: Vec<&str> = ai_text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("```") {
                // Look for filepath indicator in previous 2 lines or on the first code line
                let mut detected_path: Option<PathBuf> = None;

                for offset in 1..=2 {
                    if i >= offset {
                        let prev = lines[i - offset].trim();
                        if let Some(p) = Self::parse_filepath(prev) {
                            detected_path = Some(p);
                            break;
                        }
                    }
                }

                // Collect code block lines
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
                    let full = root.join(&rel);
                    let orig = std::fs::read_to_string(&full).unwrap_or_default();
                    let proposed = code_lines.join("\n");
                    if !proposed.trim().is_empty() && proposed != orig {
                        diffs.push(FileDiff::new(
                            full,
                            orig,
                            proposed,
                            agent_name,
                            format!("Automated code proposal from {}", agent_name),
                        ));
                    }
                }
            }
            i += 1;
        }

        diffs
    }

    fn parse_filepath(line: &str) -> Option<PathBuf> {
        let lower = line.to_lowercase();
        let stripped = if let Some(idx) = lower.find("file:") {
            &line[idx + 5..]
        } else if let Some(idx) = lower.find("filepath:") {
            &line[idx + 9..]
        } else if lower.starts_with("//") || lower.starts_with("#") {
            let trimmed = line.trim_start_matches(|c| c == '/' || c == '#' || c == ' ');
            if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('.') {
                trimmed
            } else {
                return None;
            }
        } else {
            return None;
        };

        let clean = stripped.trim().trim_matches(|c| c == '`' || c == '"' || c == '\'' || c == '*' || c == ':').trim();
        if clean.contains('.') && !clean.contains(' ') {
            Some(PathBuf::from(clean))
        } else {
            None
        }
    }
}
