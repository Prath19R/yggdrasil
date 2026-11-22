use chrono::Utc;
use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use yggdrasil_agents::AgentRole;
use yggdrasil_core::diff::{DiffStatus, FileDiff};
use yggdrasil_inference::types::InferenceMode;
use yggdrasil_inference::InferenceRouter;
use yggdrasil_orchestrator::plan::ExecutionPlan;
use yggdrasil_orchestrator::OrchestratorEngine;
use yggdrasil_project::{FileNode, ProjectManager};
use yggdrasil_terminal::TerminalRunner;

#[derive(Clone, Debug, PartialEq)]
pub enum FileKind {
    Text,
    Image,
    Video,
    Document,
    Binary,
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub sender: String,
    pub is_user: bool,
    pub text: String,
    pub thinking: Option<String>,
    pub diff: Option<FileDiff>,
    pub timestamp: String,
}

#[derive(Clone, Debug)]
pub struct EditorTab {
    pub path: PathBuf,
    pub name: String,
    pub kind: FileKind,
    pub content: String,
    pub file_size_bytes: u64,
    pub is_dirty: bool,
}

#[allow(dead_code)]
pub enum UiEvent {
    StageUpdate { text: String, thinking: Option<String> },
    DiffProposed(FileDiff),
    TerminalLog(String),
    DebugLog(String),
    ChatReply { sender: String, text: String, thinking: Option<String> },
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

fn load_saved_api_key() -> Option<String> {
    let path = get_config_dir().join("config.json");
    if let Ok(data) = std::fs::read_to_string(path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            if let Some(s) = val["nvidia_api_key"].as_str() {
                if !s.trim().is_empty() {
                    return Some(s.trim().to_string());
                }
            }
        }
    }
    std::env::var("NVIDIA_API_KEY").ok()
}

fn save_api_key_to_config(key: &str) {
    let dir = get_config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("config.json");
    let val = serde_json::json!({
        "nvidia_api_key": key
    });
    let _ = std::fs::write(path, val.to_string());
}

fn open_in_system(path: &Path) {
    if cfg!(target_os = "windows") {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    } else if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("open").arg(path).spawn();
    } else {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

pub struct YggdrasilApp {
    pub project: Arc<tokio::sync::Mutex<ProjectManager>>,
    pub router: Arc<tokio::sync::Mutex<InferenceRouter>>,
    pub orchestrator: Arc<OrchestratorEngine>,
    
    // UI Local State
    pub open_tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub chat_history: Vec<ChatMessage>,
    pub chat_input: String,
    pub active_agent: AgentRole,
    
    pub terminal_input: String,
    pub terminal_logs: Vec<String>,
    pub compiler_logs: Vec<String>,
    pub debug_logs: Vec<String>,
    pub active_bottom_tab: usize,
    pub terminal_height: f32,

    // Async Channel
    pub ui_receiver: mpsc::UnboundedReceiver<UiEvent>,
    pub ui_sender: mpsc::UnboundedSender<UiEvent>,

    pub current_mode: InferenceMode,
    pub current_model: String,
    pub show_thinking: bool,
    pub auto_apply_diffs: bool,
    pub is_fullscreen: bool,

    // API Key Dialog
    pub show_api_key_modal: bool,
    pub api_key_input: String,
}

impl YggdrasilApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (ui_sender, ui_receiver) = mpsc::unbounded_channel();
        
        let api_key = load_saved_api_key();
        let key_input = api_key.clone().unwrap_or_default();
        let router = Arc::new(tokio::sync::Mutex::new(InferenceRouter::new(api_key, None)));
        let project = Arc::new(tokio::sync::Mutex::new(ProjectManager::new()));
        let orchestrator = Arc::new(OrchestratorEngine::new(router.clone(), project.clone()));

        let mut app = Self {
            project,
            router,
            orchestrator,
            open_tabs: Vec::new(),
            active_tab: 0,
            chat_history: Vec::new(),
            chat_input: String::new(),
            active_agent: AgentRole::Orchestrator,
            terminal_input: String::new(),
            terminal_logs: vec!["⚡ Yggdrasil PTY initialized. Ready.".to_string()],
            compiler_logs: Vec::new(),
            debug_logs: vec![
                format!("[{}] [INIT] Yggdrasil IDE core initialized in Debug Mode", Utc::now().format("%H:%M:%S")),
                format!("[{}] [INIT] Tokio multi-threaded async actor bus online", Utc::now().format("%H:%M:%S")),
            ],
            active_bottom_tab: 0,
            terminal_height: 220.0,
            ui_receiver,
            ui_sender,
            current_mode: InferenceMode::CloudApi,
            current_model: "nvidia/nemotron-3.5-lightning-30b-a3b".to_string(),
            show_thinking: true,
            auto_apply_diffs: false, // Default to false so user clearly sees the Diff Review card!
            is_fullscreen: false,
            show_api_key_modal: false,
            api_key_input: key_input,
        };

        // Welcome message in chat
        app.chat_history.push(ChatMessage {
            sender: "👑 Master Orchestrator".into(),
            is_user: false,
            text: "Welcome to Yggdrasil. Open a project folder to begin, or ask me to scaffold, debug, compile, or deploy.".into(),
            thinking: Some("Initialized autonomous multi-agent swarm. Dual inference ready (NVIDIA NIM Cloud + Local Ollama).".into()),
            diff: None,
            timestamp: Utc::now().format("%H:%M:%S").to_string(),
        });

        app
    }

    pub fn open_file(&mut self, path: PathBuf) {
        if let Some(pos) = self.open_tabs.iter().position(|t| t.path == path) {
            self.active_tab = pos;
            return;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let kind = match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "svg" => FileKind::Image,
            "mp4" | "webm" | "mov" | "mkv" | "avi" | "mp3" | "wav" => FileKind::Video,
            "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => FileKind::Document,
            "zip" | "tar" | "gz" | "wasm" | "exe" | "pdb" => FileKind::Binary,
            _ => FileKind::Text,
        };

        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        let content = if kind == FileKind::Text {
            match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => match std::fs::read(&path) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).to_string(),
                    Err(_) => String::new(),
                },
            }
        } else {
            String::new()
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());

        self.debug_logs.push(format!(
            "[{}] [FILE_IO] Opened: {} ({:?}, {} bytes)",
            Utc::now().format("%H:%M:%S"),
            path.display(),
            kind,
            file_size_bytes
        ));

        self.open_tabs.push(EditorTab {
            path,
            name,
            kind,
            content,
            file_size_bytes,
            is_dirty: false,
        });
        self.active_tab = self.open_tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if index < self.open_tabs.len() {
            self.open_tabs.remove(index);
            if self.active_tab >= self.open_tabs.len() && !self.open_tabs.is_empty() {
                self.active_tab = self.open_tabs.len() - 1;
            }
        }
    }

    pub fn save_active_file(&mut self) {
        if self.active_tab < self.open_tabs.len() {
            let tab = &mut self.open_tabs[self.active_tab];
            if tab.kind == FileKind::Text {
                if let Ok(_) = std::fs::write(&tab.path, &tab.content) {
                    tab.is_dirty = false;
                    self.debug_logs.push(format!(
                        "[{}] [FILE_IO] Saved: {} ({} bytes)",
                        Utc::now().format("%H:%M:%S"),
                        tab.path.display(),
                        tab.content.len()
                    ));
                }
            }
        }
    }

    fn render_file_tree(ui: &mut egui::Ui, node: &FileNode, to_open: &mut Option<PathBuf>) {
        ui.push_id(&node.path, |ui| {
            if node.is_dir {
                egui::CollapsingHeader::new(format!("📁 {}", node.name))
                    .id_salt(&node.path)
                    .default_open(node.name == "src" || node.name == "crates" || node.children.len() < 4)
                    .show(ui, |ui| {
                        for child in &node.children {
                            Self::render_file_tree(ui, child, to_open);
                        }
                    });
            } else {
                let ext = node
                    .path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                let icon = match ext.as_str() {
                    "rs" => "🦀",
                    "js" | "ts" | "jsx" | "tsx" => "📦",
                    "py" => "🐍",
                    "json" | "toml" | "yaml" | "yml" => "⚙️",
                    "md" => "📝",
                    "png" | "jpg" | "jpeg" | "gif" | "webp" => "🖼️",
                    "mp4" | "webm" | "mov" => "🎬",
                    "pdf" => "📑",
                    _ => "📄",
                };

                let label = format!("{} {}", icon, node.name);
                if ui.button(label).clicked() {
                    *to_open = Some(node.path.clone());
                }
            }
        });
    }
}

impl eframe::App for YggdrasilApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Toggle Fullscreen with F11
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.is_fullscreen = !self.is_fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
            self.debug_logs.push(format!(
                "[{}] [UI] Fullscreen toggled: {}",
                Utc::now().format("%H:%M:%S"),
                self.is_fullscreen
            ));
        }

        // Poll incoming async UI events
        while let Ok(event) = self.ui_receiver.try_recv() {
            match event {
                UiEvent::StageUpdate { text, thinking } => {
                    self.chat_history.push(ChatMessage {
                        sender: "👑 Master Orchestrator".into(),
                        is_user: false,
                        text,
                        thinking,
                        diff: None,
                        timestamp: Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
                UiEvent::DiffProposed(mut diff) => {
                    let path = diff.path.clone();
                    if self.auto_apply_diffs {
                        let _ = diff.accept();
                        self.open_file(path.clone());
                        self.debug_logs.push(format!(
                            "[{}] [DIFF] Auto-applied code to: {}",
                            Utc::now().format("%H:%M:%S"),
                            path.display()
                        ));
                    } else {
                        self.debug_logs.push(format!(
                            "[{}] [DIFF] Proposed diff generated for: {} (Pending Review)",
                            Utc::now().format("%H:%M:%S"),
                            path.display()
                        ));
                    }

                    self.chat_history.push(ChatMessage {
                        sender: diff.agent_role.clone(),
                        is_user: false,
                        text: if self.auto_apply_diffs {
                            format!("⚡ Automatically applied code changes to `{}`\n{}", path.display(), diff.summary)
                        } else {
                            format!("Proposed changes for `{}`:\n{}", path.display(), diff.summary)
                        },
                        thinking: None,
                        diff: Some(diff),
                        timestamp: Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
                UiEvent::TerminalLog(line) => {
                    self.terminal_logs.push(line);
                }
                UiEvent::DebugLog(log) => {
                    self.debug_logs.push(log);
                }
                UiEvent::ChatReply { sender, text, thinking } => {
                    self.chat_history.push(ChatMessage {
                        sender,
                        is_user: false,
                        text,
                        thinking,
                        diff: None,
                        timestamp: Utc::now().format("%H:%M:%S").to_string(),
                    });
                }
            }
        }

        // Top Navigation Header
        egui::TopBottomPanel::top("top_header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🌳 Yggdrasil");
                ui.separator();

                if ui.button("📂 Open Folder...").clicked() {
                    let project_clone = self.project.clone();
                    let tx = self.ui_sender.clone();
                    tokio::spawn(async move {
                        let mut p = project_clone.lock().await;
                        if let Ok(Some(path)) = p.open_via_dialog() {
                            let _ = tx.send(UiEvent::DebugLog(format!(
                                "[{}] [PROJECT] Loaded workspace: {}",
                                Utc::now().format("%H:%M:%S"),
                                path.display()
                            )));
                        }
                    });
                }

                // Project Badge & Git Branch
                if let Ok(p) = self.project.try_lock() {
                    if let Some(root) = &p.root_path {
                        let name = root.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                        ui.label(format!("Project: [{}]", name));
                        ui.colored_label(egui::Color32::from_rgb(100, 200, 100), p.project_type.display_badge());

                        if let Some(branch) = &p.git_status.current_branch {
                            ui.label(format!("🌿 {}", branch));
                        }
                    } else {
                        ui.weak("No project opened");
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Fullscreen Toggle
                    let fs_label = if self.is_fullscreen { "🗗 Windowed" } else { "⛶ Fullscreen" };
                    if ui.button(fs_label).clicked() {
                        self.is_fullscreen = !self.is_fullscreen;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.is_fullscreen));
                    }

                    ui.separator();
                    ui.weak("RAM: ~42 MB | Latency: 18ms");
                    ui.weak(format!("Model: [{}]", self.current_model));
                    ui.separator();

                    // Mode Switcher Dropdown
                    let mode_text = match self.current_mode {
                        InferenceMode::CloudApi => "☁️ Cloud API (NIM)",
                        InferenceMode::Local => "💻 Local (Ollama)",
                    };
                    egui::ComboBox::from_id_salt("mode_selector")
                        .selected_text(mode_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.current_mode, InferenceMode::CloudApi, "☁️ Cloud API (NVIDIA NIM)").clicked() {
                                let router = self.router.clone();
                                let tx = self.ui_sender.clone();
                                tokio::spawn(async move {
                                    router.lock().await.set_mode(InferenceMode::CloudApi);
                                    let _ = tx.send(UiEvent::DebugLog(format!(
                                        "[{}] [ROUTER] Switched mode to Cloud API (NVIDIA NIM)",
                                        Utc::now().format("%H:%M:%S")
                                    )));
                                });
                            }
                            if ui.selectable_value(&mut self.current_mode, InferenceMode::Local, "💻 Local Mode (Ollama)").clicked() {
                                let router = self.router.clone();
                                let tx = self.ui_sender.clone();
                                tokio::spawn(async move {
                                    router.lock().await.set_mode(InferenceMode::Local);
                                    let _ = tx.send(UiEvent::DebugLog(format!(
                                        "[{}] [ROUTER] Switched mode to Local (Ollama localhost:11434)",
                                        Utc::now().format("%H:%M:%S")
                                    )));
                                });
                            }
                        });

                    // API Key Config Button
                    let has_key = self.router.try_lock().map(|r| r.has_api_key()).unwrap_or(false);
                    let (key_text, key_color) = if has_key {
                        ("🟢 API Key Configured", egui::Color32::from_rgb(100, 220, 100))
                    } else {
                        ("🔴 Configure API Key", egui::Color32::from_rgb(255, 100, 100))
                    };

                    if ui.colored_label(key_color, key_text).clicked() || ui.button("🔑 Key Settings").clicked() {
                        self.show_api_key_modal = true;
                    }
                });
            });
        });

        // NVIDIA API Key Configuration Modal Window
        if self.show_api_key_modal {
            egui::Window::new("🔑 Configure NVIDIA NIM API Key")
                .id(egui::Id::new("nvidia_api_key_modal"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_width(380.0);
                    ui.label("Enter your free-tier NVIDIA NIM API key below:");
                    ui.weak("Get a free key at https://build.nvidia.com");
                    ui.add_space(8.0);
                    
                    ui.add(
                        egui::TextEdit::singleline(&mut self.api_key_input)
                            .password(true)
                            .hint_text("nvapi-...")
                            .desired_width(360.0),
                    );
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("✔ Save & Apply Key").clicked() {
                            let key = self.api_key_input.trim().to_string();
                            save_api_key_to_config(&key);
                            let router = self.router.clone();
                            let tx = self.ui_sender.clone();
                            tokio::spawn(async move {
                                router.lock().await.set_api_key(key);
                                let _ = tx.send(UiEvent::DebugLog(format!(
                                    "[{}] [AUTH] NVIDIA API key applied and stored in config.json",
                                    Utc::now().format("%H:%M:%S")
                                )));
                            });
                            self.show_api_key_modal = false;
                        }
                        if ui.button("✖ Cancel").clicked() {
                            self.show_api_key_modal = false;
                        }
                    });
                });
        }

        // Left Panel: Project File Explorer
        egui::SidePanel::left("file_explorer")
            .resizable(true)
            .default_width(240.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Explorer");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🔄 Refresh").clicked() {
                            let project_clone = self.project.clone();
                            let tx = self.ui_sender.clone();
                            tokio::spawn(async move {
                                let mut p = project_clone.lock().await;
                                if let Some(root) = p.root_path.clone() {
                                    let _ = p.open_project(root);
                                    let _ = tx.send(UiEvent::DebugLog(format!(
                                        "[{}] [EXPLORER] Refreshed directory tree",
                                        Utc::now().format("%H:%M:%S")
                                    )));
                                }
                            });
                        }
                    });
                });
                ui.separator();

                let root_node = self.project.try_lock().ok().and_then(|p| p.root_node.clone());
                if let Some(node) = root_node {
                    let mut file_to_open = None;
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            Self::render_file_tree(ui, &node, &mut file_to_open);
                        });
                    if let Some(path) = file_to_open {
                        self.open_file(path);
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(30.0);
                        ui.label("No folder opened.");
                        if ui.button("Open Folder").clicked() {
                            let project_clone = self.project.clone();
                            tokio::spawn(async move {
                                let mut p = project_clone.lock().await;
                                let _ = p.open_via_dialog();
                            });
                        }
                    });
                }
            });

        // Right Panel: Agent Chat Window
        egui::SidePanel::right("agent_chat_window")
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("💬 Agent Deck");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.checkbox(&mut self.show_thinking, "Thinking");
                        ui.checkbox(&mut self.auto_apply_diffs, "⚡ Auto-apply");
                    });
                });
                ui.separator();

                // Agent Role Selector Bar
                ui.horizontal_wrapped(|ui| {
                    ui.selectable_value(&mut self.active_agent, AgentRole::Orchestrator, "👑 All");
                    ui.selectable_value(&mut self.active_agent, AgentRole::Coder, "💻 Coder");
                    ui.selectable_value(&mut self.active_agent, AgentRole::Debugger, "🐞 Debug");
                    ui.selectable_value(&mut self.active_agent, AgentRole::Deployer, "🚀 Deploy");
                });
                ui.separator();

                // Chat Message Scroll View (with zero ID clashes)
                let chat_height = ui.available_height() - 95.0;
                let mut file_to_open_after = None;

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(chat_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for (idx, msg) in self.chat_history.iter_mut().enumerate() {
                            ui.push_id(egui::Id::new("chat_item").with(idx), |ui| {
                                egui::Frame::group(ui.style())
                                    .fill(if msg.is_user {
                                        egui::Color32::from_rgb(20, 30, 45)
                                    } else {
                                        egui::Color32::from_rgb(25, 28, 36)
                                    })
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 60, 75)))
                                    .corner_radius(6.0)
                                    .inner_margin(8.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if msg.is_user {
                                                ui.colored_label(egui::Color32::from_rgb(140, 200, 255), "👤 You");
                                            } else {
                                                ui.colored_label(egui::Color32::from_rgb(255, 200, 100), &msg.sender);
                                            }
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                ui.weak(&msg.timestamp);
                                            });
                                        });

                                        // Thinking block with unique id_salt
                                        if self.show_thinking {
                                            if let Some(thought) = &msg.thinking {
                                                egui::CollapsingHeader::new("💭 Reasoning / Thinking Trace")
                                                    .id_salt(format!("think_header_{}", idx))
                                                    .default_open(false)
                                                    .show(ui, |ui| {
                                                        ui.monospace(thought);
                                                    });
                                            }
                                        }

                                        ui.add_space(4.0);
                                        ui.label(&msg.text);

                                        // High-Contrast Interactive Code Diff Card
                                        if let Some(diff) = &mut msg.diff {
                                            ui.add_space(8.0);
                                            egui::Frame::NONE
                                                .fill(egui::Color32::from_rgb(18, 22, 30))
                                                .stroke(egui::Stroke::new(1.5, egui::Color32::from_rgb(46, 160, 67)))
                                                .corner_radius(6.0)
                                                .inner_margin(10.0)
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.colored_label(egui::Color32::from_rgb(100, 230, 120), "📝 PROPOSED CODE DIFF");
                                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                            let status_text = match diff.status {
                                                                DiffStatus::Pending => "⏳ PENDING REVIEW",
                                                                DiffStatus::Accepted => "✔ ACCEPTED & APPLIED",
                                                                DiffStatus::Rejected => "✖ REJECTED",
                                                            };
                                                            ui.weak(status_text);
                                                        });
                                                    });

                                                    ui.label(format!("Target File: {}", diff.path.display()));
                                                    ui.weak(&diff.summary);
                                                    ui.separator();

                                                    // Code preview box
                                                    egui::ScrollArea::both()
                                                        .id_salt(format!("diff_scroll_{}", idx))
                                                        .max_height(140.0)
                                                        .auto_shrink([false, false])
                                                        .show(ui, |ui| {
                                                            ui.monospace(&diff.proposed_content);
                                                        });

                                                    ui.add_space(6.0);
                                                    if diff.status == DiffStatus::Pending {
                                                        ui.horizontal(|ui| {
                                                            if ui.button("✔ Accept & Apply to Disk").clicked() {
                                                                let _ = diff.accept();
                                                                file_to_open_after = Some(diff.path.clone());
                                                            }
                                                            if ui.button("✖ Reject").clicked() {
                                                                diff.reject();
                                                            }
                                                        });
                                                    } else if diff.status == DiffStatus::Accepted {
                                                        ui.colored_label(egui::Color32::from_rgb(100, 240, 120), "✔ Changes successfully written to file!");
                                                    }
                                                });
                                        }
                                    });
                            });
                            ui.add_space(6.0);
                        }
                    });

                if let Some(path) = file_to_open_after {
                    self.open_file(path);
                }

                ui.separator();

                // Chat Input Deck
                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.chat_input)
                            .desired_rows(2)
                            .hint_text("Ask Orchestrator (e.g. 'implement auth', 'cargo check')..."),
                    );

                    let send_clicked = ui.button("Send ➔").clicked();
                    if (send_clicked || (response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !self.chat_input.trim().is_empty()
                    {
                        let input = self.chat_input.trim().to_string();
                        self.chat_input.clear();

                        self.chat_history.push(ChatMessage {
                            sender: "Developer".into(),
                            is_user: true,
                            text: input.clone(),
                            thinking: None,
                            diff: None,
                            timestamp: Utc::now().format("%H:%M:%S").to_string(),
                        });

                        // Launch Orchestrator async plan
                        let orch = self.orchestrator.clone();
                        let tx = self.ui_sender.clone();
                        tokio::spawn(async move {
                            let mut plan: ExecutionPlan = orch.formulate_plan(&input);
                            let stages_summary = plan
                                .stages
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("{}. [{}] {}", i + 1, s.agent.name(), s.title))
                                .collect::<Vec<_>>()
                                .join("\n");

                            let _ = tx.send(UiEvent::DebugLog(format!(
                                "[{}] [ORCHESTRATOR] Formulated plan with {} stages for query: '{}'",
                                Utc::now().format("%H:%M:%S"),
                                plan.stages.len(),
                                input
                            )));

                            let _ = tx.send(UiEvent::StageUpdate {
                                text: format!("Formulated Execution Plan:\n{}", stages_summary),
                                thinking: None,
                            });

                            for i in 0..plan.stages.len() {
                                let stage_name = plan.stages[i].title.clone();
                                let agent_name = plan.stages[i].agent.name().to_string();
                                
                                let _ = tx.send(UiEvent::DebugLog(format!(
                                    "[{}] [STAGE_START] Executing stage {}/{}: [{}] {}",
                                    Utc::now().format("%H:%M:%S"),
                                    i + 1,
                                    plan.stages.len(),
                                    agent_name,
                                    stage_name
                                )));

                                match orch.execute_stage(i, &mut plan).await {
                                    Ok(result) => {
                                        let diff_count = result.proposed_diffs.len();
                                        let _ = tx.send(UiEvent::DebugLog(format!(
                                            "[{}] [STAGE_DONE] [{}] finished (generated {} diffs)",
                                            Utc::now().format("%H:%M:%S"),
                                            agent_name,
                                            diff_count
                                        )));

                                        let _ = tx.send(UiEvent::StageUpdate {
                                            text: result.text,
                                            thinking: result.thinking,
                                        });

                                        for diff in result.proposed_diffs {
                                            let _ = tx.send(UiEvent::DiffProposed(diff));
                                        }
                                    }
                                    Err(e) => {
                                        let _ = tx.send(UiEvent::DebugLog(format!(
                                            "[{}] [STAGE_ERROR] [{}] error: {}",
                                            Utc::now().format("%H:%M:%S"),
                                            agent_name,
                                            e
                                        )));

                                        let _ = tx.send(UiEvent::StageUpdate {
                                            text: format!("Stage Execution Error: {}", e),
                                            thinking: None,
                                        });
                                    }
                                }
                            }
                        });
                    }
                });
            });

        // Bottom Panel: Embedded Terminal & Diagnostics & Debug Run Logs
        egui::TopBottomPanel::bottom("bottom_terminal")
            .resizable(true)
            .min_height(75.0)
            .max_height(750.0)
            .default_height(self.terminal_height)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.active_bottom_tab == 0, "💻 Terminal").clicked() {
                        self.active_bottom_tab = 0;
                    }
                    if ui.selectable_label(self.active_bottom_tab == 1, "🐞 Compiler Output").clicked() {
                        self.active_bottom_tab = 1;
                    }
                    let debug_label = format!("🛠️ Debug Logs ({})", self.debug_logs.len());
                    if ui.selectable_label(self.active_bottom_tab == 2, debug_label).clicked() {
                        self.active_bottom_tab = 2;
                    }

                    // Terminal extension presets
                    ui.separator();
                    if ui.button("⬆ Expand (400px)").clicked() {
                        self.terminal_height = 400.0;
                    }
                    if ui.button("↕ Medium (240px)").clicked() {
                        self.terminal_height = 240.0;
                    }
                    if ui.button("⬇ Collapse (100px)").clicked() {
                        self.terminal_height = 100.0;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear Logs").clicked() {
                            if self.active_bottom_tab == 0 {
                                self.terminal_logs.clear();
                            } else if self.active_bottom_tab == 1 {
                                self.compiler_logs.clear();
                            } else {
                                self.debug_logs.clear();
                            }
                        }
                    });
                });
                ui.separator();

                let avail_y = ui.available_height();
                let scroll_height = (avail_y - 36.0).max(40.0);

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .max_height(scroll_height)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.active_bottom_tab == 0 {
                            for line in &self.terminal_logs {
                                ui.monospace(line);
                            }
                        } else if self.active_bottom_tab == 1 {
                            if self.compiler_logs.is_empty() {
                                ui.weak("No compiler diagnostics yet. Run 'cargo check' or ask Debugger Agent.");
                            } else {
                                for line in &self.compiler_logs {
                                    ui.monospace(line);
                                }
                            }
                        } else {
                            // Debug logs view
                            for log in &self.debug_logs {
                                if log.contains("[ERROR]") || log.contains("[STAGE_ERROR]") {
                                    ui.colored_label(egui::Color32::from_rgb(255, 120, 120), log);
                                } else if log.contains("[DIFF]") || log.contains("[DIFF_DONE]") {
                                    ui.colored_label(egui::Color32::from_rgb(120, 240, 140), log);
                                } else if log.contains("[ROUTER]") || log.contains("[AI_REQUEST]") {
                                    ui.colored_label(egui::Color32::from_rgb(140, 200, 255), log);
                                } else {
                                    ui.monospace(log);
                                }
                            }
                        }
                    });

                ui.horizontal(|ui| {
                    ui.weak("> ");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.terminal_input)
                            .hint_text("Run shell command (cargo check, cargo test, git status)..."),
                    );
                    if (ui.button("Run").clicked() || (resp.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter))))
                        && !self.terminal_input.trim().is_empty()
                    {
                        let cmd = self.terminal_input.trim().to_string();
                        self.terminal_input.clear();
                        self.terminal_logs.push(format!("$ {}", cmd));

                        let root = self.project.try_lock().ok().and_then(|p| p.root_path.clone());
                        let tx = self.ui_sender.clone();
                        tokio::spawn(async move {
                            let _ = tx.send(UiEvent::DebugLog(format!(
                                "[{}] [PTY] Spawning: {}",
                                Utc::now().format("%H:%M:%S"),
                                cmd
                            )));

                            match TerminalRunner::run_command(root, &cmd).await {
                                Ok(out) => {
                                    if !out.stdout.is_empty() {
                                        let _ = tx.send(UiEvent::TerminalLog(out.stdout));
                                    }
                                    if !out.stderr.is_empty() {
                                        let _ = tx.send(UiEvent::TerminalLog(format!("ERR: {}", out.stderr)));
                                    }
                                    let _ = tx.send(UiEvent::DebugLog(format!(
                                        "[{}] [PTY] Finished '{}' with exit code: {:?}",
                                        Utc::now().format("%H:%M:%S"),
                                        cmd,
                                        out.exit_code
                                    )));
                                }
                                Err(e) => {
                                    let _ = tx.send(UiEvent::TerminalLog(format!("Execution Error: {}", e)));
                                    let _ = tx.send(UiEvent::DebugLog(format!(
                                        "[{}] [PTY_ERROR] Command failed: {}",
                                        Utc::now().format("%H:%M:%S"),
                                        e
                                    )));
                                }
                            }
                        });
                    }
                });
            });

        // Center Panel: Code Editor & Media/Document Viewer
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tabs Bar (with zero ID clashes)
            if !self.open_tabs.is_empty() {
                ui.horizontal(|ui| {
                    let mut tab_to_close = None;
                    for (i, tab) in self.open_tabs.iter().enumerate() {
                        ui.push_id(egui::Id::new("tab_header").with(i), |ui| {
                            let title = if tab.is_dirty {
                                format!("* {}", tab.name)
                            } else {
                                tab.name.clone()
                            };

                            ui.horizontal(|ui| {
                                if ui.selectable_label(self.active_tab == i, title).clicked() {
                                    self.active_tab = i;
                                }
                                if ui.small_button("×").clicked() {
                                    tab_to_close = Some(i);
                                }
                            });
                        });
                    }

                    if let Some(i) = tab_to_close {
                        self.close_tab(i);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("💾 Save (Ctrl+S)").clicked() {
                            self.save_active_file();
                        }
                    });
                });
                ui.separator();
            }

            if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
                self.save_active_file();
            }

            // Active Tab Content Display
            if self.active_tab < self.open_tabs.len() {
                let tab = &mut self.open_tabs[self.active_tab];
                match tab.kind {
                    FileKind::Text => {
                        let prev = tab.content.clone();
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut tab.content)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .desired_width(f32::INFINITY)
                                        .lock_focus(true),
                                );
                            });
                        if tab.content != prev {
                            tab.is_dirty = true;
                        }
                    }
                    FileKind::Image => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.heading(format!("🖼️ Image: {}", tab.name));
                            ui.label(format!("File Size: {:.2} KB", tab.file_size_bytes as f64 / 1024.0));
                            ui.weak(format!("Path: {}", tab.path.display()));
                            ui.add_space(20.0);
                            if ui.button("▶ Open in System Image Viewer").clicked() {
                                open_in_system(&tab.path);
                            }
                        });
                    }
                    FileKind::Video => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.heading(format!("🎬 Video: {}", tab.name));
                            ui.label(format!("File Size: {:.2} MB", tab.file_size_bytes as f64 / (1024.0 * 1024.0)));
                            ui.weak(format!("Path: {}", tab.path.display()));
                            ui.add_space(20.0);
                            if ui.button("▶ Play in System Media Player").clicked() {
                                open_in_system(&tab.path);
                            }
                        });
                    }
                    FileKind::Document => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.heading(format!("📑 Document: {}", tab.name));
                            ui.label(format!("File Size: {:.2} KB", tab.file_size_bytes as f64 / 1024.0));
                            ui.weak(format!("Path: {}", tab.path.display()));
                            ui.add_space(20.0);
                            if ui.button("📂 Open in System Document Viewer").clicked() {
                                open_in_system(&tab.path);
                            }
                        });
                    }
                    FileKind::Binary => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.heading(format!("📦 Binary File: {}", tab.name));
                            ui.label(format!("File Size: {:.2} KB", tab.file_size_bytes as f64 / 1024.0));
                            ui.weak(format!("Path: {}", tab.path.display()));
                            ui.add_space(20.0);
                            if ui.button("📂 Open in System File Explorer").clicked() {
                                open_in_system(&tab.path);
                            }
                        });
                    }
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("🌳 Yggdrasil");
                    ui.label("Pure Rust AI-Powered Native IDE");
                    ui.add_space(20.0);
                    ui.label("Click 'Open Folder' in the top bar to load a project, or chat with the Master Orchestrator.");
                });
            }
        });
    }
}
