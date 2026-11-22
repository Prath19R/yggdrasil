use leptos::prelude::*;
use crate::services::StorageService;

#[component]
pub fn TopNav(
    project_name: ReadSignal<String>,
    mode: ReadSignal<String>,
    set_mode: WriteSignal<String>,
    active_model: ReadSignal<String>,
    set_active_model: WriteSignal<String>,
    has_api_key: ReadSignal<bool>,
    set_show_key_modal: WriteSignal<bool>,
    on_open_folder: Callback<()>,
) -> impl IntoView {
    let toggle_fullscreen = move |_| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.document_element() {
                let _ = el.request_fullscreen();
            }
        }
    };

    view! {
        <header class="h-12 bg-zinc-950 border-b border-zinc-800 flex items-center justify-between px-4 select-none shrink-0 z-20">
            // Left Branding & Workspace Status
            <div class="flex items-center gap-3">
                <div class="flex items-center gap-2 font-bold tracking-tight text-white text-base">
                    <span class="text-xl">"🌳"</span>
                    <span class="bg-gradient-to-r from-emerald-400 to-cyan-400 bg-clip-text text-transparent">"Yggdrasil"</span>
                </div>
                <div class="h-4 w-px bg-zinc-800"></div>

                <button
                    on:click=move |_| on_open_folder.run(())
                    class="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium bg-zinc-900 hover:bg-zinc-800 text-zinc-300 hover:text-white rounded-md border border-zinc-700/60 transition shadow-sm"
                >
                    <span>"📂"</span>
                    <span>"Open Folder..."</span>
                </button>

                <div class="flex items-center gap-2 px-2.5 py-1 bg-zinc-900/60 rounded-md border border-zinc-800 text-xs">
                    <span class="text-zinc-500">"Workspace:"</span>
                    <span class="font-mono text-zinc-200">{move || project_name.get()}</span>
                    <span class="px-1.5 py-0.5 text-[10px] bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 rounded font-semibold">"🦀 Rust"</span>
                </div>
            </div>

            // Right Controls: Mode, Model, Key, Fullscreen
            <div class="flex items-center gap-2.5">
                // Mode Toggle (Cloud NIM vs Local Ollama)
                <div class="flex items-center bg-zinc-900 border border-zinc-800 rounded-lg p-0.5 text-xs font-medium">
                    <button
                        on:click=move |_| {
                            StorageService::set_mode("cloud");
                            set_mode.set("cloud".to_string());
                        }
                        class=move || {
                            let is_cloud = mode.get() == "cloud";
                            if is_cloud {
                                "px-2.5 py-1 rounded-md bg-emerald-500 text-zinc-950 font-semibold shadow-sm transition"
                            } else {
                                "px-2.5 py-1 rounded-md text-zinc-400 hover:text-zinc-200 transition"
                            }
                        }
                    >
                        "☁️ NVIDIA NIM"
                    </button>
                    <button
                        on:click=move |_| {
                            StorageService::set_mode("local");
                            set_mode.set("local".to_string());
                        }
                        class=move || {
                            let is_local = mode.get() == "local";
                            if is_local {
                                "px-2.5 py-1 rounded-md bg-cyan-500 text-zinc-950 font-semibold shadow-sm transition"
                            } else {
                                "px-2.5 py-1 rounded-md text-zinc-400 hover:text-zinc-200 transition"
                            }
                        }
                    >
                        "💻 Local Ollama"
                    </button>
                </div>

                // Model Selector
                <select
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        set_active_model.set(val);
                    }
                    class="bg-zinc-900 border border-zinc-800 text-zinc-300 hover:text-white text-xs rounded-lg px-2.5 py-1.5 outline-none focus:border-zinc-700 cursor-pointer font-mono"
                >
                    <option value="nvidia/nemotron-3.5-lightning-30b-a3b" selected=move || active_model.get().contains("nemotron")>
                        "Nemotron 3.5 (Reasoning)"
                    </option>
                    <option value="mistralai/mistral-large-4-1t-a52b" selected=move || active_model.get().contains("mistral")>
                        "Mistral Large (Multimodal)"
                    </option>
                    <option value="qwen/qwen2.5-coder-32b-instruct" selected=move || active_model.get().contains("qwen")>
                        "Qwen 2.5 Coder (Fast)"
                    </option>
                    <option value="deepseek-ai/deepseek-r1" selected=move || active_model.get().contains("deepseek")>
                        "DeepSeek R1 (Logic)"
                    </option>
                </select>

                // API Key Status Button
                <button
                    on:click=move |_| set_show_key_modal.set(true)
                    class=move || {
                        format!(
                            "flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium rounded-lg border transition shadow-sm cursor-pointer {}",
                            if has_api_key.get() {
                                "bg-emerald-500/10 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20"
                            } else {
                                "bg-red-500/10 text-red-400 border-red-500/30 hover:bg-red-500/20 animate-pulse"
                            }
                        )
                    }
                >
                    <span>{move || if has_api_key.get() { "🟢" } else { "🔴" }}</span>
                    <span>{move || if has_api_key.get() { "API Key Set" } else { "Configure Key" }}</span>
                </button>

                // Fullscreen Button
                <button
                    on:click=toggle_fullscreen
                    class="p-1.5 text-zinc-400 hover:text-white bg-zinc-900 hover:bg-zinc-800 rounded-lg border border-zinc-800 transition"
                    title="Toggle Fullscreen"
                >
                    <span class="text-sm">"⛶"</span>
                </button>
            </div>
        </header>
    }
}
