use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
}
