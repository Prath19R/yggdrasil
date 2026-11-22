use leptos::prelude::*;
use crate::models::file::{FileItem, FileKind};

#[component]
pub fn CodeEditor(
    open_tabs: ReadSignal<Vec<FileItem>>,
    active_tab_idx: ReadSignal<usize>,
    set_active_tab_idx: WriteSignal<usize>,
    on_close_tab: Callback<usize>,
    on_content_change: Callback<(usize, String)>,
) -> impl IntoView {
    view! {
        <div class="flex-1 flex flex-col h-full bg-[#0d0e12] overflow-hidden min-w-0">
            // Tabs Bar
            <div class="h-9 bg-zinc-950 border-b border-zinc-800 flex items-center overflow-x-auto select-none shrink-0 scrollbar-none">
                {move || {
                    let tabs = open_tabs.get();
                    let current = active_tab_idx.get();

                    tabs.into_iter().enumerate().map(|(idx, tab)| {
                        let is_active = idx == current;
                        let close = on_close_tab;
                        view! {
                            <div
                                on:click=move |_| set_active_tab_idx.set(idx)
                                class=format!(
                                    "flex items-center gap-2 px-3.5 h-full text-xs border-r border-zinc-800/80 cursor-pointer transition font-mono border-t-2 {}",
                                    if is_active {
                                        "bg-[#0d0e12] text-zinc-100 border-t-emerald-500 font-medium"
                                    } else {
                                        "bg-zinc-950/80 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900 border-t-transparent"
                                    }
                                )
                            >
                                <span>{tab.icon()}</span>
                                <span class="truncate max-w-[140px]">{tab.name}</span>
                                <button
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        close.run(idx);
                                    }
                                    class="text-zinc-500 hover:text-zinc-200 hover:bg-zinc-800 rounded p-0.5 ml-1 transition"
                                >
                                    "×"
                                </button>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>

            // Editor or Media Viewer Body
            <div class="flex-1 overflow-auto relative">
                {move || {
                    let tabs = open_tabs.get();
                    let current = active_tab_idx.get();

                    if tabs.is_empty() || current >= tabs.len() {
                        view! {
                            <div class="h-full flex flex-col items-center justify-center text-center p-8 text-zinc-500 select-none">
                                <span class="text-5xl mb-4">"🌳"</span>
                                <h2 class="text-lg font-semibold text-zinc-300">"Yggdrasil IDE"</h2>
                                <p class="text-xs max-w-sm mt-1 text-zinc-500">
                                    "Pure Rust AI-Powered IDE. Select a file from the explorer on the left or prompt the Master Orchestrator."
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        let active = tabs[current].clone();
                        match active.kind {
                            FileKind::Text => {
                                let content = active.content.clone();
                                let line_count = content.lines().count().max(1);
                                let on_change = on_content_change;

                                view! {
                                    <div class="h-full flex font-mono text-xs leading-relaxed">
                                        // Line numbers gutter
                                        <div class="w-12 py-3 px-2 bg-zinc-950/60 border-r border-zinc-800/60 text-zinc-600 text-right select-none font-mono text-[11px] shrink-0">
                                            {(1..=line_count).map(|n| view! { <div>{n}</div> }).collect::<Vec<_>>()}
                                        </div>

                                        // Text edit textarea
                                        <textarea
                                            prop:value=content
                                            on:input=move |ev| {
                                                let val = event_target_value(&ev);
                                                on_change.run((current, val));
                                            }
                                            class="flex-1 p-3 bg-transparent text-zinc-200 outline-none resize-none overflow-auto font-mono text-xs leading-relaxed selection:bg-emerald-500/30 selection:text-white"
                                            spellcheck="false"
                                        ></textarea>
                                    </div>
                                }.into_any()
                            }
                            FileKind::Image => {
                                view! {
                                    <div class="h-full flex flex-col items-center justify-center p-8 text-center">
                                        <span class="text-6xl mb-3">"🖼️"</span>
                                        <h3 class="text-sm font-semibold text-zinc-200">{active.name}</h3>
                                        <p class="text-xs text-zinc-400 mt-1">
                                            {format!("Size: {:.2} KB", active.size_bytes as f64 / 1024.0)}
                                        </p>
                                        <span class="mt-4 px-3 py-1.5 text-xs bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 rounded-lg">
                                            "Supports Multimodal Vision in Agent Deck"
                                        </span>
                                    </div>
                                }.into_any()
                            }
                            FileKind::Video => {
                                view! {
                                    <div class="h-full flex flex-col items-center justify-center p-8 text-center">
                                        <span class="text-6xl mb-3">"🎬"</span>
                                        <h3 class="text-sm font-semibold text-zinc-200">{active.name}</h3>
                                        <p class="text-xs text-zinc-400 mt-1">
                                            {format!("Size: {:.2} MB", active.size_bytes as f64 / (1024.0 * 1024.0))}
                                        </p>
                                    </div>
                                }.into_any()
                            }
                            _ => {
                                view! {
                                    <div class="h-full flex flex-col items-center justify-center p-8 text-center">
                                        <span class="text-6xl mb-3">"📑"</span>
                                        <h3 class="text-sm font-semibold text-zinc-200">{active.name}</h3>
                                        <p class="text-xs text-zinc-400 mt-1">
                                            {format!("Document / Binary: {} bytes", active.size_bytes)}
                                        </p>
                                    </div>
                                }.into_any()
                            }
                        }
                    }
                }}
            </div>
        </div>
    }
}
