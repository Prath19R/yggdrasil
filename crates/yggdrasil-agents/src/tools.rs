use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use yggdrasil_terminal::TerminalRunner;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCall {
    ReadFile { path: PathBuf },
    WriteFile { path: PathBuf, content: String },
    CargoCheck,
    CargoTest,
    RunCommand { command: String },
    ListFiles { subpath: Option<PathBuf> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

pub struct AgentTool;

impl AgentTool {
    pub async fn execute<P: AsRef<Path>>(root: P, call: ToolCall) -> ToolExecutionResult {
        let root = root.as_ref();
        match call {
            ToolCall::ReadFile { path } => Self::read_file(root, &path),
            ToolCall::WriteFile { path, content } => Self::write_file(root, &path, &content),
            ToolCall::CargoCheck => Self::cargo_check(root).await,
            ToolCall::CargoTest => Self::cargo_test(root).await,
            ToolCall::RunCommand { command } => Self::run_command(root, &command).await,
            ToolCall::ListFiles { subpath } => Self::list_files(root, subpath.as_deref()),
        }
    }

    pub fn read_file<P: AsRef<Path>>(root: P, rel: P) -> ToolExecutionResult {
        let full = root.as_ref().join(rel);
        match std::fs::read_to_string(&full) {
            Ok(content) => ToolExecutionResult {
                success: true,
                output: content,
                error: None,
            },
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to read {}: {}", full.display(), e)),
            },
        }
    }

    pub fn write_file<P: AsRef<Path>>(root: P, rel: P, content: &str) -> ToolExecutionResult {
        let full = root.as_ref().join(rel);
        if let Some(parent) = full.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::write(&full, content) {
            Ok(_) => ToolExecutionResult {
                success: true,
                output: format!("Successfully wrote {} bytes to {}", content.len(), full.display()),
                error: None,
            },
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to write {}: {}", full.display(), e)),
            },
        }
    }

    pub async fn cargo_check<P: AsRef<Path>>(root: P) -> ToolExecutionResult {
        match TerminalRunner::run_cargo_check(root).await {
            Ok(out) => ToolExecutionResult {
                success: out.success,
                output: format!("{}\n{}", out.stdout, out.stderr),
                error: if out.success { None } else { Some("Compiler errors encountered".into()) },
            },
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(e),
            },
        }
    }

    pub async fn cargo_test<P: AsRef<Path>>(root: P) -> ToolExecutionResult {
        match TerminalRunner::run_cargo_test(root).await {
            Ok(out) => ToolExecutionResult {
                success: out.success,
                output: format!("{}\n{}", out.stdout, out.stderr),
                error: if out.success { None } else { Some("Tests failed".into()) },
            },
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(e),
            },
        }
    }

    pub async fn run_command<P: AsRef<Path>>(root: P, command: &str) -> ToolExecutionResult {
        match TerminalRunner::run_command(Some(root), command).await {
            Ok(out) => ToolExecutionResult {
                success: out.success,
                output: format!("{}\n{}", out.stdout, out.stderr),
                error: if out.success { None } else { Some(format!("Command exited with code {:?}", out.exit_code)) },
            },
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(e),
            },
        }
    }

    pub fn list_files<P: AsRef<Path>>(root: P, subpath: Option<&Path>) -> ToolExecutionResult {
        let target = match subpath {
            Some(p) => root.as_ref().join(p),
            None => root.as_ref().to_path_buf(),
        };

        match std::fs::read_dir(&target) {
            Ok(entries) => {
                let mut list = Vec::new();
                for entry in entries.flatten() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = entry.path().is_dir();
                    list.push(format!("{} {}", if is_dir { "📁" } else { "📄" }, file_name));
                }
                list.sort();
                ToolExecutionResult {
                    success: true,
                    output: list.join("\n"),
                    error: None,
                }
            }
            Err(e) => ToolExecutionResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to list {}: {}", target.display(), e)),
            },
        }
    }
}
