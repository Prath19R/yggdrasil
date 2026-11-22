use leptos::ev;
use leptos::prelude::*;
use crate::services::TauriService;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConsoleTab {
    Terminal,
    Compiler,
    Debug,
}

#[component]
pub fn BottomConsole(
    terminal_logs: ReadSignal<Vec<String>>,
    compiler_logs: ReadSignal<Vec<String>>,
    debug_logs: ReadSignal<Vec<String>>,
    on_run_command: Callback<String>,
    on_clear_logs: Callback<ConsoleTab>,
    project_path: ReadSignal<String>,
    is_collapsed: ReadSignal<bool>,
    set_is_collapsed: WriteSignal<bool>,
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(ConsoleTab::Terminal);
    let (cmd_input, set_cmd_input) = signal(String::new());
    let (height, set_height) = signal(260i32);
    let (is_dragging, set_is_dragging) = signal(false);
    let (is_maximized, set_is_maximized) = signal(false);

    let handle_command_submit = move || {
        let cmd = cmd_input.get().trim().to_string();
        if !cmd.is_empty() {
            set_cmd_input.set(String::new());
            on_run_command.run(cmd);
        }
    };

    let handle_drag_start = move |ev: web_sys::MouseEvent| {
        ev.prevent_default();
        set_is_dragging.set(true);
    };

    let _ = window_event_listener(ev::mousemove, move |ev: web_sys::MouseEvent| {
        if is_dragging.get() {
            if let Some(win) = web_sys::window() {
                if let Ok(win_h) = win.inner_height() {
                    let win_height = win_h.as_f64().unwrap_or(800.0) as i32;
                    let new_h = (win_height - ev.client_y()).clamp(90, (win_height - 120).max(120));
                    set_height.set(new_h);
                    if is_collapsed.get() {
                        set_is_collapsed.set(false);
                    }
                    if is_maximized.get() {
                        set_is_maximized.set(false);
                    }
                }
            }
        }
    });

    let _ = window_event_listener(ev::mouseup, move |_| {
        if is_dragging.get() {
            set_is_dragging.set(false);
        }
    });

    view! {
        <div
            style=move || {
                if is_collapsed.get() {
                    "height: 36px;".to_string()
                } else if is_maximized.get() {
                    "height: calc(100vh - 48px);".to_string()
                } else {
                    format!("height: {}px;", height.get())
                }
            }
            class="bg-zinc-950 border-t border-zinc-800 flex flex-col select-none relative shrink-0 transition-[height] duration-75"
        >
            // Draggable Top Resize Handle
            <div
                on:mousedown=handle_drag_start
                class="h-1.5 w-full cursor-row-resize bg-zinc-800/60 hover:bg-emerald-500 active:bg-emerald-400 transition-colors shrink-0 flex items-center justify-center group z-10"
                title="Drag up or down to resize terminal"
            >
                <div class="w-12 h-0.5 rounded-full bg-zinc-600 group-hover:bg-white group-active:bg-white transition-colors" />
            </div>

            // Console Header Tabs Bar
            <div class="h-9 px-3 bg-zinc-950 flex items-center justify-between border-b border-zinc-800 shrink-0">
                <div class="flex items-center gap-1">
                    <button
                        on:click=move |_| {
                            set_active_tab.set(ConsoleTab::Terminal);
                            set_is_collapsed.set(false);
                        }
                        class=move || {
                            format!(
                                "flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md transition {}",
                                if active_tab.get() == ConsoleTab::Terminal && !is_collapsed.get() {
                                    "bg-zinc-800 text-zinc-100 font-semibold shadow-sm"
                                } else {
                                    "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900"
                                }
                            )
                        }
                    >
                        <span>"💻"</span>
                        <span>"Terminal"</span>
                    </button>

                    <button
                        on:click=move |_| {
                            set_active_tab.set(ConsoleTab::Compiler);
                            set_is_collapsed.set(false);
                        }
                        class=move || {
                            format!(
                                "flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md transition {}",
                                if active_tab.get() == ConsoleTab::Compiler && !is_collapsed.get() {
                                    "bg-zinc-800 text-zinc-100 font-semibold shadow-sm"
                                } else {
                                    "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900"
                                }
                            )
                        }
                    >
                        <span>"🐞"</span>
                        <span>"Compiler Output"</span>
                    </button>

                    <button
                        on:click=move |_| {
                            set_active_tab.set(ConsoleTab::Debug);
                            set_is_collapsed.set(false);
                        }
                        class=move || {
                            format!(
                                "flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-md transition {}",
                                if active_tab.get() == ConsoleTab::Debug && !is_collapsed.get() {
                                    "bg-zinc-800 text-zinc-100 font-semibold shadow-sm"
                                } else {
                                    "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900"
                                }
                            )
                        }
                    >
                        <span>"🛠️"</span>
                        <span>"Debug Logs"</span>
                    </button>
                </div>

                // Middle: Keyboard shortcut hints
                <div class="hidden md:flex items-center gap-3 text-[10px] text-zinc-500 font-mono">
                    <span><kbd class="px-1 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-400">"Ctrl+`"</kbd> " Toggle"</span>
                    <span><kbd class="px-1 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-400">"Ctrl+S"</kbd> " Save/Accept"</span>
                    <span><kbd class="px-1 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-400">"Ctrl+O"</kbd> " Open Folder"</span>
                    <span><kbd class="px-1 py-0.5 bg-zinc-900 border border-zinc-800 rounded text-zinc-400">"Ctrl+B"</kbd> " Sidebar"</span>
                </div>

                // Right controls: Clear, Maximize/Restore, Collapse/Expand
                <div class="flex items-center gap-1.5 text-zinc-400">
                    <button
                        title="Clear console output"
                        on:click=move |_| on_clear_logs.run(active_tab.get())
                        class="px-2 py-1 hover:text-zinc-200 hover:bg-zinc-800 rounded transition text-xs flex items-center gap-1"
                    >
                        <span>"🧹"</span>
                        <span class="hidden sm:inline">"Clear"</span>
                    </button>

                    <button
                        title=move || if is_maximized.get() { "Restore height (🗗)" } else { "Maximize terminal (🗖)" }
                        on:click=move |_| {
                            set_is_maximized.update(|m| *m = !*m);
                            if is_collapsed.get() {
                                set_is_collapsed.set(false);
                            }
                        }
                        class="p-1 hover:text-zinc-200 hover:bg-zinc-800 rounded transition text-xs font-mono"
                    >
                        {move || if is_maximized.get() { "🗗" } else { "🗖" }}
                    </button>

                    <button
                        title=move || if is_collapsed.get() { "Expand terminal (▲) [Ctrl+`]" } else { "Minimize terminal (▼) [Ctrl+`]" }
                        on:click=move |_| {
                            set_is_collapsed.update(|c| *c = !*c);
                            if is_maximized.get() {
                                set_is_maximized.set(false);
                            }
                        }
                        class="p-1 hover:text-zinc-200 hover:bg-zinc-800 rounded transition text-xs font-mono"
                    >
                        {move || if is_collapsed.get() { "▲" } else { "▼" }}
                    </button>
                </div>
            </div>

            // Console Content Area (hidden if collapsed)
            <div class=move || {
                format!(
                    "flex-1 flex flex-col min-h-0 bg-[#090a0f] font-mono text-xs overflow-hidden {}",
                    if is_collapsed.get() { "hidden" } else { "" }
                )
            }>
                // Scrollable log area
                <div class="flex-1 overflow-y-auto p-3 space-y-1 select-text scrollbar-thin">
                    {move || {
                        let lines = match active_tab.get() {
                            ConsoleTab::Terminal => terminal_logs.get(),
                            ConsoleTab::Compiler => compiler_logs.get(),
                            ConsoleTab::Debug => debug_logs.get(),
                        };

                        if lines.is_empty() {
                            view! {
                                <div class="text-zinc-600 italic py-2">
                                    {match active_tab.get() {
                                        ConsoleTab::Terminal => {
                                            if TauriService::is_available() {
                                                "⚡ Yggdrasil Native Terminal active. Run real host commands (cargo build, cargo test, git status, dir, etc.)."
                                            } else {
                                                "⚠️ Web Sandbox Terminal. Type 'cargo check', 'cargo run', or 'help'. Run yggdrasil-desktop.exe for native host execution."
                                            }
                                        },
                                        ConsoleTab::Compiler => "No active compilation errors or warnings. Compiler output is clean.",
                                        ConsoleTab::Debug => "Debug runtime logs will stream here during execution.",
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            lines.into_iter().enumerate().map(|(_idx, line)| {
                                let style_class = if line.starts_with("PS ") || line.starts_with("$") || line.starts_with(">") {
                                    "text-cyan-400 font-bold"
                                } else if line.contains("error:") || line.contains("Error") || line.contains("FAILED") {
                                    "text-rose-400"
                                } else if line.contains("warning:") || line.contains("WARN") {
                                    "text-amber-300"
                                } else if line.contains("Finished") || line.contains("success") || line.contains("Clean Build") || line.starts_with("💾") || line.starts_with("✔") {
                                    "text-emerald-400 font-semibold"
                                } else if line.starts_with("[DEBUG]") {
                                    "text-purple-400"
                                } else if line.starts_with("[INFO]") {
                                    "text-blue-400"
                                } else if line.contains("Web Browser Mode") || line.contains("simulated") {
                                    "text-amber-300 italic"
                                } else {
                                    "text-zinc-300"
                                };

                                view! {
                                    <div class=format!("leading-relaxed break-words {}", style_class)>
                                        {line}
                                    </div>
                                }
                            }).collect::<Vec<_>>().into_any()
                        }
                    }}
                </div>

                // Interactive Terminal Command Input Bar (only shown in Terminal tab)
                {move || {
                    if active_tab.get() == ConsoleTab::Terminal {
                        view! {
                            <div class="border-t border-zinc-800/80 bg-zinc-950/80 px-3 py-1.5 flex items-center gap-2">
                                <span class=move || {
                                    if TauriService::is_available() {
                                        "text-emerald-400 font-bold select-none text-[11px] font-mono shrink-0"
                                    } else {
                                        "text-amber-400 font-bold select-none text-[11px] font-mono shrink-0"
                                    }
                                }>
                                    {move || {
                                        if TauriService::is_available() {
                                            let p = project_path.get();
                                            let short_path = if p.len() > 30 {
                                                format!("...{}", &p[p.len() - 25..])
                                            } else if p.is_empty() {
                                                "~".to_string()
                                            } else {
                                                p
                                            };
                                            format!("PS {}>", short_path)
                                        } else {
                                            "[wasm-sandbox] ❯".to_string()
                                        }
                                    }}
                                </span>
                                <input
                                    type="text"
                                    placeholder=move || {
                                        if TauriService::is_available() {
                                            "Execute native command (e.g. cargo check, cargo build, cargo test, git status)..."
                                        } else {
                                            "Run simulated command in browser (e.g. cargo check, cargo run, help)..."
                                        }
                                    }
                                    prop:value=move || cmd_input.get()
                                    on:input=move |ev| set_cmd_input.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" {
                                            handle_command_submit();
                                        }
                                    }
                                    class="flex-1 bg-transparent text-zinc-100 placeholder-zinc-600 outline-none text-xs font-mono"
                                />
                                <button
                                    on:click=move |_| handle_command_submit()
                                    class="px-2.5 py-1 bg-emerald-600/80 hover:bg-emerald-500 text-white font-medium rounded text-[11px] transition cursor-pointer"
                                >
                                    "Run"
                                </button>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span /> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}
