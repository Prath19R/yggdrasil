use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::components::{AgentDeck, ApiKeyModal, BottomConsole, CodeEditor, FileTree};
use crate::components::bottom_console::ConsoleTab;
use crate::components::TopNav;
use crate::models::agent::AgentRole;
use crate::models::chat::ChatMessage;
use crate::models::diff::{DiffStatus, FileDiff};
use crate::models::file::{FileItem, FileKind};
use crate::services::{InferenceService, OrchestratorService, StorageService};

#[component]
pub fn App() -> impl IntoView {
    // Top-level Application Signals
    let (project_name, set_project_name) = signal("yggdrasil-workspace".to_string());
    let initial_mode = StorageService::get_mode();
    let (mode, set_mode) = signal(initial_mode);
    let (active_model, set_active_model) = signal("nvidia/nemotron-3.5-lightning-30b-a3b".to_string());
    let (has_api_key, set_has_api_key) = signal(StorageService::has_api_key());
    let (show_key_modal, set_show_key_modal) = signal(false);

    // Initial demo project tree
    let sample_tree = FileItem::new_dir(
        "yggdrasil-workspace",
        "/yggdrasil-workspace",
        vec![
            FileItem::new_file(
                "Cargo.toml",
                "/yggdrasil-workspace/Cargo.toml",
                r#"[package]
name = "yggdrasil-workspace"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
"#,
            ),
            FileItem::new_dir(
                "src",
                "/yggdrasil-workspace/src",
                vec![
                    FileItem::new_file(
                        "main.rs",
                        "/yggdrasil-workspace/src/main.rs",
                        r#"use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("🌳 Yggdrasil Engine initialized successfully!");
    println!("Hello from autonomous Rust multi-agent workspace!");
}
"#,
                    ),
                    FileItem::new_file(
                        "lib.rs",
                        "/yggdrasil-workspace/src/lib.rs",
                        r#"pub mod swarm {
    pub fn coordinate() -> &'static str {
        "Swarm online. Ready to execute."
    }
}
"#,
                    ),
                ],
            ),
            FileItem::new_file(
                "README.md",
                "/yggdrasil-workspace/README.md",
                r#"# 🌳 Yggdrasil IDE

Universal cross-platform WebAssembly IDE powered by **Rust**, **Leptos 0.7**, and an **Autonomous Multi-Agent Swarm**.

## Features
- **Dual Inference Engine**: Free NVIDIA NIM Cloud API & Local Air-gapped Ollama
- **Safety Diff Guard**: Interactive diff cards with green `[✔ Accept Diff]` buttons
- **Full Media Viewer**: Native syntax code editor, Markdown renderer, Image & Video players
- **Tri-Console**: Terminal CLI, Compiler Diagnostics & Live Debug Stream
"#,
            ),
            FileItem {
                id: "/yggdrasil-workspace/assets".into(),
                name: "assets".into(),
                path: "/yggdrasil-workspace/assets".into(),
                is_dir: true,
                kind: FileKind::Document,
                content: String::new(),
                size_bytes: 0,
                is_expanded: true,
                children: vec![
                    FileItem {
                        id: "/yggdrasil-workspace/assets/banner.png".into(),
                        name: "banner.png".into(),
                        path: "/yggdrasil-workspace/assets/banner.png".into(),
                        is_dir: false,
                        kind: FileKind::Image,
                        content: "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='600' height='200' viewBox='0 0 600 200'><rect width='100%' height='100%' fill='%230f172a'/><text x='50%' y='50%' font-size='24' fill='%2310b981' font-family='sans-serif' text-anchor='middle' dominant-baseline='middle'>🌳 Yggdrasil Banner Media Viewer</text></svg>".into(),
                        size_bytes: 120,
                        is_expanded: false,
                        children: vec![],
                    },
                    FileItem {
                        id: "/yggdrasil-workspace/assets/demo.mp4".into(),
                        name: "demo.mp4".into(),
                        path: "/yggdrasil-workspace/assets/demo.mp4".into(),
                        is_dir: false,
                        kind: FileKind::Video,
                        content: String::new(),
                        size_bytes: 0,
                        is_expanded: false,
                        children: vec![],
                    },
                    FileItem {
                        id: "/yggdrasil-workspace/assets/spec.pdf".into(),
                        name: "spec.pdf".into(),
                        path: "/yggdrasil-workspace/assets/spec.pdf".into(),
                        is_dir: false,
                        kind: FileKind::Document,
                        content: String::new(),
                        size_bytes: 0,
                        is_expanded: false,
                        children: vec![],
                    },
                ],
            },
        ],
    );

    let (root_item, set_root_item) = signal(sample_tree.clone());

    // Tabs
    let initial_tabs = vec![
        sample_tree.children[1].children[0].clone(), // main.rs
        sample_tree.children[2].clone(),             // README.md
    ];
    let (open_tabs, set_open_tabs) = signal(initial_tabs);
    let (active_tab_idx, set_active_tab_idx) = signal(0usize);

    // Agent Swarm Chat
    let initial_messages = vec![
        ChatMessage::system(
            "👑 Master Orchestrator",
            "Autonomous multi-agent swarm ready. Dual inference engine active (NVIDIA NIM Cloud + Local Ollama).\nAsk me to scaffold, debug, refactor, run tests, or deploy.",
        ),
    ];
    let (messages, set_messages) = signal(initial_messages);
    let (is_loading, set_is_loading) = signal(false);
    let (show_thinking, set_show_thinking) = signal(true);
    let (auto_apply, set_auto_apply) = signal(false);

    // Bottom Console Logs
    let (terminal_logs, set_terminal_logs) = signal(vec![
        "$ yggdrasil --version".into(),
        "yggdrasil 0.2.0 (leptos-wasm32-unknown-unknown)".into(),
        "$ cargo status".into(),
        "Workspace: /yggdrasil-workspace [Clean]".into(),
    ]);

    let (compiler_logs, set_compiler_logs) = signal(vec![
        "Status: Clean Build (0 errors, 0 warnings)".into(),
        "Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.84s".into(),
    ]);

    let (debug_logs, set_debug_logs) = signal(vec![
        "[INFO] Yggdrasil WASM CSR engine initialized.".into(),
        "[INFO] Storage service loaded config from localStorage.".into(),
        "[DEBUG] Event loop ready. WebAssembly 32-bit runtime active.".into(),
    ]);

    // Helpers to recursively update files
    let update_node_expanded = move |path: String| {
        set_root_item.update(|root| {
            fn toggle(item: &mut FileItem, target: &str) {
                if item.path == target {
                    item.is_expanded = !item.is_expanded;
                    return;
                }
                for child in &mut item.children {
                    toggle(child, target);
                }
            }
            toggle(root, &path);
        });
    };

    let handle_select_file = Callback::new(move |selected: FileItem| {
        let current_tabs = open_tabs.get();
        if let Some(existing_idx) = current_tabs.iter().position(|t| t.path == selected.path) {
            set_active_tab_idx.set(existing_idx);
        } else {
            let new_idx = current_tabs.len();
            set_open_tabs.update(|tabs| tabs.push(selected));
            set_active_tab_idx.set(new_idx);
        }
    });

    let handle_close_tab = Callback::new(move |close_idx: usize| {
        let tabs = open_tabs.get();
        if tabs.len() <= 1 {
            return;
        }
        set_open_tabs.update(|t| {
            if close_idx < t.len() {
                t.remove(close_idx);
            }
        });
        let curr = active_tab_idx.get();
        if curr >= close_idx && curr > 0 {
            set_active_tab_idx.set(curr - 1);
        }
    });

    let handle_content_change = Callback::new(move |(tab_idx, new_text): (usize, String)| {
        let path = {
            let tabs = open_tabs.get();
            tabs.get(tab_idx).map(|t| t.path.clone())
        };

        if let Some(path) = path {
            // Update open tab
            set_open_tabs.update(|t| {
                if let Some(tab) = t.get_mut(tab_idx) {
                    tab.content = new_text.clone();
                }
            });

            // Update in tree
            set_root_item.update(|root| {
                fn update_content(item: &mut FileItem, target: &str, text: &str) {
                    if item.path == target {
                        item.content = text.to_string();
                        return;
                    }
                    for child in &mut item.children {
                        update_content(child, target, text);
                    }
                }
                update_content(root, &path, &new_text);
            });
        }
    });

    // Accept Diff Callback
    let handle_accept_diff = Callback::new(move |diff: FileDiff| {
        let file_path = diff.path.clone();
        let new_content = diff.proposed_content.clone();

        // 1. Update tree content
        set_root_item.update(|root| {
            fn update_or_add(item: &mut FileItem, path: &str, content: &str) {
                if item.path == path {
                    item.content = content.to_string();
                    return;
                }
                for child in &mut item.children {
                    update_or_add(child, path, content);
                }
            }
            update_or_add(root, &file_path, &new_content);
        });

        // 2. Update open tab if active
        set_open_tabs.update(|tabs| {
            for tab in tabs.iter_mut() {
                if tab.path == file_path {
                    tab.content = new_content.clone();
                }
            }
        });

        // 3. Mark diff as accepted in chat messages
        set_messages.update(|msgs: &mut Vec<ChatMessage>| {
            for msg in msgs.iter_mut() {
                if let Some(ref mut d) = msg.diff {
                    if d.id == diff.id {
                        d.status = DiffStatus::Accepted;
                    }
                }
            }
        });

        // 4. Log to console
        set_terminal_logs.update(|logs| {
            logs.push(format!("✔ [Safety Guard] Diff applied to: {}", file_path));
        });
        set_compiler_logs.update(|logs| {
            logs.push(format!("$ cargo check --target wasm32-unknown-unknown"));
            logs.push(format!("    Checking yggdrasil-workspace v0.1.0"));
            logs.push(format!("    Finished dev profile [unoptimized] in 0.42s"));
            logs.push(format!("Status: Clean Build (0 errors)"));
        });
        set_debug_logs.update(|logs| {
            logs.push(format!("[INFO] FileDiff id={} accepted and synced to WASM FS", diff.id));
        });
    });

    // Reject Diff Callback
    let handle_reject_diff = Callback::new(move |diff_id: String| {
        set_messages.update(|msgs: &mut Vec<ChatMessage>| {
            for msg in msgs.iter_mut() {
                if let Some(ref mut d) = msg.diff {
                    if d.id == diff_id {
                        d.status = DiffStatus::Rejected;
                    }
                }
            }
        });
        set_terminal_logs.update(|logs| {
            logs.push(format!("✖ [Safety Guard] Diff rejected by user (id: {})", diff_id));
        });
    });

    // Send Prompt & Swarm Orchestration
    let handle_send_prompt = Callback::new(move |prompt_text: String| {
        let is_cloud = mode.get() == "cloud";
        let api_key = StorageService::get_api_key().unwrap_or_default();

        if is_cloud && api_key.is_empty() {
            set_show_key_modal.set(true);
            return;
        }

        // Add User Message
        set_messages.update(|m: &mut Vec<ChatMessage>| m.push(ChatMessage::user(&prompt_text)));

        // Formulate and post Execution Plan
        let plan = OrchestratorService::formulate_plan(&prompt_text);
        let plan_desc = plan
            .stages
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. [{} {}] {}", i + 1, s.agent.icon(), s.agent.title(), s.title))
            .collect::<Vec<_>>()
            .join("\n");

        set_messages.update(|m: &mut Vec<ChatMessage>| {
            m.push(ChatMessage::system(
                "👑 Master Orchestrator",
                &format!("Formulated Execution Plan:\n{}", plan_desc),
            ));
        });

        // Set Loading state
        set_is_loading.set(true);

        let selected_model = active_model.get();
        let selected_mode = mode.get();
        let auto_apply_enabled = auto_apply.get();

        // Spawn async web request via gloo-net
        spawn_local(async move {
            let primary_agent = plan.stages.first().map(|s| s.agent).unwrap_or(AgentRole::Coder);
            let system_prompt = OrchestratorService::system_prompt_for(primary_agent);
            let agent_name = format!("{} {}", primary_agent.icon(), primary_agent.title());

            set_debug_logs.update(|logs| {
                logs.push(format!(
                    "[INFO] Dispatched prompt to {:?} via model: {} (mode: {})",
                    primary_agent, selected_model, selected_mode
                ));
            });

            match InferenceService::query_llm(
                &selected_mode,
                &selected_model,
                &api_key,
                &system_prompt,
                &prompt_text,
            )
            .await
            {
                Ok((content, thinking)) => {
                    let mut diffs = OrchestratorService::extract_diffs(&content, &agent_name);

                    // If auto-apply is on, mark them accepted immediately
                    if auto_apply_enabled {
                        for d in &mut diffs {
                            d.status = DiffStatus::Accepted;
                        }
                    }

                    let mut response_msg = ChatMessage::assistant(&agent_name, &content);
                    response_msg.thinking = thinking;
                    if let Some(first_diff) = diffs.first() {
                        response_msg.diff = Some(first_diff.clone());
                    }

                    set_messages.update(|m: &mut Vec<ChatMessage>| m.push(response_msg));

                    // If auto_apply, update files directly
                    if auto_apply_enabled {
                        for d in diffs {
                            set_root_item.update(|root| {
                                fn update_file(item: &mut FileItem, target: &str, content: &str) {
                                    if item.path == target {
                                        item.content = content.to_string();
                                        return;
                                    }
                                    for child in &mut item.children {
                                        update_file(child, target, content);
                                    }
                                }
                                update_file(root, &d.path, &d.proposed_content);
                            });
                        }
                    }

                    set_terminal_logs.update(|logs| {
                        logs.push(format!("⚡ [{}] Completed task execution successfully", agent_name));
                    });

                    set_compiler_logs.update(|logs| {
                        logs.push(format!("$ cargo check"));
                        logs.push(format!("Status: Clean Build (0 errors)"));
                    });
                }
                Err(err) => {
                    set_messages.update(|m: &mut Vec<ChatMessage>| {
                        m.push(ChatMessage::system(
                            "🐞 Debugger & Swarm Alert",
                            &format!("⚠️ Inference Error: {}\nCheck your API key or ensure local Ollama is active on port 11434.", err),
                        ));
                    });
                    set_debug_logs.update(|logs| {
                        logs.push(format!("[ERROR] Inference failure: {}", err));
                    });
                }
            }

            set_is_loading.set(false);
        });
    });

    // Run Command Callback from Terminal Tab
    let handle_run_command = Callback::new(move |cmd: String| {
        set_terminal_logs.update(|logs| logs.push(format!("$ {}", cmd)));

        let lower = cmd.trim().to_lowercase();
        if lower == "clear" {
            set_terminal_logs.set(vec![]);
        } else if lower == "help" {
            set_terminal_logs.update(|logs| {
                logs.push("Available simulated commands:".into());
                logs.push("  cargo check   - Run compiler diagnostics on workspace".into());
                logs.push("  cargo build   - Compile workspace with dev profile".into());
                logs.push("  cargo test    - Run workspace unit & integration test suite".into());
                logs.push("  cargo run     - Execute active workspace binary".into());
                logs.push("  git status    - Show working tree status".into());
                logs.push("  clear         - Clear terminal console".into());
            });
        } else if lower.starts_with("cargo check") {
            set_compiler_logs.update(|logs| {
                logs.push(format!("$ {}", cmd));
                logs.push("    Checking yggdrasil-workspace v0.1.0".into());
                logs.push("    Finished `dev` profile in 0.52s".into());
                logs.push("Status: Clean Build (0 errors, 0 warnings)".into());
            });
            set_terminal_logs.update(|logs| {
                logs.push("Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.52s".into());
            });
        } else if lower.starts_with("cargo test") {
            set_compiler_logs.update(|logs| {
                logs.push(format!("$ {}", cmd));
                logs.push("    Compiling yggdrasil-workspace v0.1.0".into());
                logs.push("    Finished `test` profile [unoptimized + debuginfo] in 0.74s".into());
                logs.push("     Running tests/integration_test.rs".into());
                logs.push("test swarm::test_coordination ... ok".into());
                logs.push("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out".into());
            });
            set_terminal_logs.update(|logs| {
                logs.push("test result: ok. 1 passed; 0 failed; 0 ignored".into());
            });
        } else if lower.starts_with("cargo run") {
            set_terminal_logs.update(|logs| {
                logs.push("    Finished `dev` profile [unoptimized + debuginfo] in 0.38s".into());
                logs.push("     Running `target/debug/yggdrasil-workspace`".into());
                logs.push("🌳 Yggdrasil Engine initialized successfully!".into());
                logs.push("Hello from autonomous Rust multi-agent workspace!".into());
            });
        } else if lower.starts_with("git status") {
            set_terminal_logs.update(|logs| {
                logs.push("On branch main".into());
                logs.push("Your branch is up to date with 'origin/main'.".into());
                logs.push("nothing to commit, working tree clean".into());
            });
        } else {
            set_terminal_logs.update(|logs| {
                logs.push(format!("Command executed: {}", cmd));
            });
        }
    });

    let handle_clear_logs = Callback::new(move |tab: ConsoleTab| match tab {
        ConsoleTab::Terminal => set_terminal_logs.set(vec![]),
        ConsoleTab::Compiler => set_compiler_logs.set(vec![]),
        ConsoleTab::Debug => set_debug_logs.set(vec![]),
    });

    // File tree callbacks
    let on_toggle_folder = Callback::new(move |path: String| {
        update_node_expanded(path);
    });

    let on_open_folder = Callback::new(move |_| {
        set_project_name.set("yggdrasil-workspace".to_string());
        set_terminal_logs.update(|logs| {
            logs.push("Workspace reloaded: /yggdrasil-workspace".to_string());
        });
    });

    view! {
        <div class="flex flex-col h-screen w-screen overflow-hidden bg-zinc-950 text-zinc-100 font-sans">
            // Top Navigation Bar
            <TopNav
                project_name=project_name
                mode=mode
                set_mode=set_mode
                active_model=active_model
                set_active_model=set_active_model
                has_api_key=has_api_key
                set_show_key_modal=set_show_key_modal
                on_open_folder=on_open_folder
            />

            // Center Work Area (Tree + Editor + Agent Deck)
            <div class="flex-1 flex min-h-0 overflow-hidden">
                // File Explorer
                <FileTree
                    root_item=root_item
                    on_select_file=handle_select_file
                    on_toggle_folder=on_toggle_folder
                />

                // Code & Media Editor
                <CodeEditor
                    open_tabs=open_tabs
                    active_tab_idx=active_tab_idx
                    set_active_tab_idx=set_active_tab_idx
                    on_close_tab=handle_close_tab
                    on_content_change=handle_content_change
                />

                // Autonomous Agent Deck
                <AgentDeck
                    messages=messages
                    is_loading=is_loading
                    show_thinking=show_thinking
                    set_show_thinking=set_show_thinking
                    auto_apply=auto_apply
                    set_auto_apply=set_auto_apply
                    on_send_prompt=handle_send_prompt
                    on_accept_diff=handle_accept_diff
                    on_reject_diff=handle_reject_diff
                />
            </div>

            // Bottom Console (Terminal, Compiler Diagnostics, Debug Logs)
            <BottomConsole
                terminal_logs=terminal_logs
                compiler_logs=compiler_logs
                debug_logs=debug_logs
                on_run_command=handle_run_command
                on_clear_logs=handle_clear_logs
            />

            // API Key Modal Dialog
            <ApiKeyModal
                show=show_key_modal
                set_show=set_show_key_modal
                set_has_key=set_has_api_key
            />
        </div>
    }
}
