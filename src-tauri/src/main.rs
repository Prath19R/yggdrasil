// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItemDto {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub kind: String,
    pub content: String,
    pub size_bytes: u64,
    pub is_expanded: bool,
    pub children: Vec<FileItemDto>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileContentDto {
    pub path: String,
    pub kind: String,
    pub content: String,
    pub size_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandOutputDto {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmResponseDto {
    pub content: String,
    pub thinking: Option<String>,
}

fn get_config_dir() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE") {
        PathBuf::from(user_dirs).join(".yggdrasil")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join(".yggdrasil")
    } else {
        PathBuf::from(".yggdrasil")
    }
}

fn detect_file_kind(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg" => "Image",
        "mp4" | "webm" | "mov" | "mkv" | "avi" | "mp3" | "wav" => "Video",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => "Document",
        "zip" | "tar" | "gz" | "wasm" | "exe" | "pdb" => "Binary",
        _ => "Text",
    }
}

fn scan_directory_fast(dir_path: &Path, depth: usize) -> FileItemDto {
    let name = dir_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "folder".to_string());

    let mut children = Vec::new();

    // Limit depth to 8 levels for maximum responsiveness and to prevent deep traversal stalls
    if depth < 8 {
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let entry_name = file_name.to_string_lossy();

                // Skip hidden files/directories and common heavy build/cache folders
                if entry_name.starts_with('.') {
                    continue;
                }
                match entry_name.as_ref() {
                    "target" | "node_modules" | "dist" | "build" | ".cache"
                    | "__pycache__" | "venv" | ".venv" | "obj" | "bin"
                    | ".idea" | ".vscode" | "coverage" | "tmp" => continue,
                    _ => {}
                }

                let file_type = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };

                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();

                if file_type.is_dir() {
                    children.push(scan_directory_fast(&path, depth + 1));
                } else if file_type.is_file() {
                    let kind = detect_file_kind(&path);
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                    // ULTRA-FAST: Do not read file content during folder scan!
                    // Content is lazy-loaded on demand when clicked or opened.
                    children.push(FileItemDto {
                        id: path_str.clone(),
                        name: entry_name.to_string(),
                        path: path_str,
                        is_dir: false,
                        kind: kind.to_string(),
                        content: String::new(),
                        size_bytes: size,
                        is_expanded: false,
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    // Sort: directories first, then alphabetical
    children.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    FileItemDto {
        id: dir_path.to_string_lossy().to_string(),
        name,
        path: dir_path.to_string_lossy().to_string(),
        is_dir: true,
        kind: "Directory".into(),
        content: String::new(),
        size_bytes: 0,
        is_expanded: depth == 0,
        children,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAURI COMMANDS
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn open_folder_dialog() -> Result<Option<FileItemDto>, String> {
    tokio::task::spawn_blocking(|| {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            let tree = scan_directory_fast(&folder, 0);
            Ok(Some(tree))
        } else {
            Ok(None)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn read_file_content(path: String) -> Result<FileContentDto, String> {
    let p = PathBuf::from(&path);
    let kind = detect_file_kind(&p);
    let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);

    let content = if kind == "Text" {
        std::fs::read_to_string(&p).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    Ok(FileContentDto {
        path,
        kind: kind.to_string(),
        content,
        size_bytes: size,
    })
}

#[tauri::command]
async fn write_file_content(path: String, content: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&p, content).map_err(|e| format!("Failed to write {}: {}", path, e))?;
    Ok(())
}

#[tauri::command]
async fn run_terminal_command(cmd: String, cwd: Option<String>) -> Result<CommandOutputDto, String> {
    let mut command = if cfg!(target_os = "windows") {
        let mut c = std::process::Command::new("powershell");
        c.args(["-NoProfile", "-Command", &cmd]);
        c
    } else {
        let mut c = std::process::Command::new("sh");
        c.args(["-c", &cmd]);
        c
    };

    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    let output = command.output().map_err(|e| format!("Failed to spawn command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok(CommandOutputDto {
        stdout,
        stderr,
        exit_code,
    })
}

#[tauri::command]
async fn get_api_key() -> Result<Option<String>, String> {
    let path = get_config_dir().join("config.json");
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(s) = val["nvidia_api_key"].as_str() {
                if !s.trim().is_empty() {
                    return Ok(Some(s.trim().to_string()));
                }
            }
        }
    }
    Ok(std::env::var("NVIDIA_API_KEY").ok())
}

#[tauri::command]
async fn save_api_key(key: String) -> Result<(), String> {
    let dir = get_config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.json");
    let val = serde_json::json!({
        "nvidia_api_key": key
    });
    std::fs::write(path, val.to_string()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn query_llm(
    mode: String,
    model: String,
    api_key: String,
    system_prompt: String,
    user_prompt: String,
) -> Result<LlmResponseDto, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Client build error: {}", e))?;

    let messages = serde_json::json!([
        { "role": "system", "content": system_prompt },
        { "role": "user", "content": user_prompt }
    ]);

    if mode == "local" {
        let url = "http://localhost:11434/v1/chat/completions";
        let payload = serde_json::json!({
            "model": if model.is_empty() { "qwen2.5-coder:7b" } else { &model },
            "messages": messages,
            "temperature": 0.3,
            "max_tokens": 4096
        });

        let resp = client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ollama connection error (check if Ollama is running on port 11434): {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama HTTP Error: {}", resp.status()));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("Ollama JSON decode error: {}", e))?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(LlmResponseDto {
            content,
            thinking: None,
        })
    } else {
        // Cloud API (NVIDIA NIM)
        let url = "https://integrate.api.nvidia.com/v1/chat/completions";
        let is_reasoning = model.contains("nemotron") || model.contains("deepseek");

        let mut payload = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": 0.2,
            "max_tokens": 4096
        });

        if is_reasoning {
            payload["reasoning_effort"] = serde_json::json!("high");
            payload["chat_template_kwargs"] = serde_json::json!({ "enable_thinking": true });
        }

        let resp = client
            .post(url)
            .header("Authorization", format!("Bearer {}", api_key.trim()))
            .header("Accept", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("NVIDIA NIM network error: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("NVIDIA NIM HTTP {} error: {}", status, text));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("NVIDIA NIM JSON error: {}", e))?;
        let choice = &json["choices"][0];
        let content = choice["message"]["content"].as_str().unwrap_or("").to_string();
        let thinking = choice["message"]["reasoning_content"]
            .as_str()
            .map(|s| s.to_string());

        Ok(LlmResponseDto { content, thinking })
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_folder_dialog,
            read_file_content,
            write_file_content,
            run_terminal_command,
            get_api_key,
            save_api_key,
            query_llm
        ])
        .run(tauri::generate_context!())
        .expect("error while running yggdrasil tauri application");
}
