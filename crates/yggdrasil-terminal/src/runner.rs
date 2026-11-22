use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub struct TerminalRunner;

impl TerminalRunner {
    pub async fn run_command<P: AsRef<Path>>(
        working_dir: Option<P>,
        cmd_str: &str,
    ) -> Result<CommandOutput, String> {
        let (program, args) = if cfg!(target_os = "windows") {
            ("powershell", vec!["-NoProfile", "-Command", cmd_str])
        } else {
            ("sh", vec!["-c", cmd_str])
        };

        let mut cmd = Command::new(program);
        cmd.args(&args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("Failed to spawn command '{cmd_str}': {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let success = output.status.success();
        let exit_code = output.status.code();

        Ok(CommandOutput {
            exit_code,
            stdout,
            stderr,
            success,
        })
    }

    pub async fn run_cargo_check<P: AsRef<Path>>(dir: P) -> Result<CommandOutput, String> {
        Self::run_command(Some(dir), "cargo check --message-format=short").await
    }

    pub async fn run_cargo_test<P: AsRef<Path>>(dir: P) -> Result<CommandOutput, String> {
        Self::run_command(Some(dir), "cargo test").await
    }
}
