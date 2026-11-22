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

        // If stage is Debugger and root_path exists, run cargo check first to feed into AI!
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
            });
        }

        let user_msg = format!(
            "Project Context:\n{}\n\nGoal: {}\nStage: {}\nTask: {}\n\
             Please execute this task thoroughly. If writing or proposing code changes, output exact file paths and code blocks.",
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
                let full_text = if compiler_diag.is_empty() {
                    format!("⚡ [{}]\n{}", role.name(), resp.text)
                } else {
                    format!("⚡ [{}]\n{}\n\n{}", role.name(), compiler_diag, resp.text)
                };
                Ok(StageResult {
                    text: full_text,
                    thinking: resp.thinking,
                })
            }
            Err(e) => {
                // If AI call failed, provide helpful debug message and any compiler diag gathered
                let err_text = format!(
                    "⚡ [{}] Execution Note:\nAI Endpoint: {}\n{}",
                    role.name(),
                    e,
                    compiler_diag
                );
                Ok(StageResult {
                    text: err_text,
                    thinking: None,
                })
            }
        }
    }

    pub async fn propose_diff(
        &self,
        rel_path: PathBuf,
        new_content: String,
        agent: AgentRole,
        summary: String,
    ) -> Result<String, String> {
        let (root, original) = {
            let p = self.project.lock().await;
            let orig = p.read_file(&rel_path).unwrap_or_default();
            (p.root_path.clone().unwrap_or_else(|| PathBuf::from(".")), orig)
        };

        let full_path = root.join(&rel_path);
        let diff = FileDiff::new(full_path, original, new_content, agent.name(), summary);
        let id = diff.id.clone();
        self.pending_diffs.lock().await.push(diff);
        Ok(id)
    }
}
