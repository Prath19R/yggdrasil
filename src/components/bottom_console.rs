use leptos::prelude::*;

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
) -> impl IntoView {
    let (active_tab, set_active_tab) = signal(ConsoleTab::Terminal);
    let (is_collapsed, set_is_collapsed) = signal(false);
    let (cmd_input, set_cmd_input) = signal(String::new());

    let handle_command_submit = move || {
        let cmd = cmd_input.get().trim().to_string();
        if !cmd.is_empty() {
            set_cmd_input.set(String::new());
            on_run_command.run(cmd);
        }
    };

    view! {
        <div class=move || {
            format!(
                "bg-zinc-950 border-t border-zinc-800 flex flex-col transition-all duration-150 select-none {}",
                if is_collapsed.get() { "h-9" } else { "h-64" }
            )
        }>
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

                // Right controls: Clear & Collapse/Expand
                <div class="flex items-center gap-2 text-zinc-400">
                    <button
                        title="Clear console"
                        on:click=move |_| on_clear_logs.run(active_tab.get())
                        class="p-1 hover:text-zinc-200 hover:bg-zinc-800 rounded transition text-xs"
                    >
                        "🧹 Clear"
                    </button>

                    <button
                        title=move || if is_collapsed.get() { "Expand" } else { "Minimize" }
                        on:click=move |_| set_is_collapsed.update(|c| *c = !*c)
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
                                        ConsoleTab::Terminal => "Yggdrasil Terminal ready. Type commands like 'cargo check', 'cargo test', 'cargo run', or 'help' below.",
                                        ConsoleTab::Compiler => "No active compilation errors or warnings. Compiler output is clean.",
                                        ConsoleTab::Debug => "Debug runtime logs will stream here during execution.",
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            lines.into_iter().enumerate().map(|(_idx, line)| {
                                let style_class = if line.starts_with("$") || line.starts_with(">") {
                                    "text-cyan-400 font-bold"
                                } else if line.contains("error:") || line.contains("Error") || line.contains("FAILED") {
                                    "text-rose-400"
                                } else if line.contains("warning:") || line.contains("WARN") {
                                    "text-amber-300"
                                } else if line.contains("Finished") || line.contains("success") || line.contains("Clean Build") {
                                    "text-emerald-400 font-semibold"
                                } else if line.starts_with("[DEBUG]") {
                                    "text-purple-400"
                                } else if line.starts_with("[INFO]") {
                                    "text-blue-400"
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
                                <span class="text-emerald-400 font-bold select-none">"❯"</span>
                                <input
                                    type="text"
                                    placeholder="Run terminal command (e.g. cargo check, cargo run, cargo test, help)..."
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
                                    class="px-2 py-0.5 bg-zinc-800 hover:bg-zinc-700 text-zinc-300 hover:text-white rounded text-[11px] font-medium transition"
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
