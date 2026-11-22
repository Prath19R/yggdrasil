use leptos::prelude::*;
use crate::services::StorageService;

#[component]
pub fn ApiKeyModal(
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    set_has_key: WriteSignal<bool>,
) -> impl IntoView {
    let initial_key = StorageService::get_api_key().unwrap_or_default();
    let (key_input, set_key_input) = signal(initial_key);

    let handle_save = move |_| {
        let val = key_input.get().trim().to_string();
        if !val.is_empty() {
            StorageService::set_api_key(&val);
            set_has_key.set(true);
        }
        set_show.set(false);
    };

    view! {
        {move || {
            if show.get() {
                view! {
                    <div class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                        <div class="bg-zinc-900 border border-zinc-700/80 rounded-2xl w-full max-w-md p-6 shadow-2xl animate-in fade-in zoom-in-95 duration-150">
                            <div class="flex items-center justify-between pb-3 border-b border-zinc-800">
                                <div class="flex items-center gap-2">
                                    <span class="text-xl">"🔑"</span>
                                    <h3 class="font-semibold text-white text-base">"NVIDIA NIM API Key"</h3>
                                </div>
                                <button
                                    on:click=move |_| set_show.set(false)
                                    class="text-zinc-400 hover:text-white p-1 rounded-md transition"
                                >
                                    "✕"
                                </button>
                            </div>

                            <p class="text-xs text-zinc-400 mt-3 leading-relaxed">
                                "Enter your free NVIDIA NIM API key to enable Nemotron, Mistral Large, and DeepSeek R1 models."
                            </p>
                            <a
                                href="https://build.nvidia.com"
                                target="_blank"
                                rel="noreferrer"
                                class="text-xs text-emerald-400 hover:underline inline-block mt-1 mb-4"
                            >
                                "Get a free API key at build.nvidia.com ↗"
                            </a>

                            <div class="space-y-2">
                                <label class="text-xs font-medium text-zinc-300">"API Key (nvapi-...)"</label>
                                <input
                                    type="password"
                                    prop:value=move || key_input.get()
                                    on:input=move |ev| set_key_input.set(event_target_value(&ev))
                                    placeholder="nvapi-..."
                                    class="w-full bg-zinc-950 border border-zinc-800 rounded-lg px-3 py-2 text-sm text-zinc-100 placeholder-zinc-600 focus:outline-none focus:border-emerald-500 font-mono"
                                />
                            </div>

                            <div class="flex items-center justify-end gap-2.5 mt-6 pt-3 border-t border-zinc-800">
                                <button
                                    on:click=move |_| set_show.set(false)
                                    class="px-4 py-2 text-xs font-medium text-zinc-400 hover:text-white rounded-lg hover:bg-zinc-800 transition"
                                >
                                    "Cancel"
                                </button>
                                <button
                                    on:click=handle_save
                                    class="px-4 py-2 text-xs font-semibold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 rounded-lg transition shadow-md"
                                >
                                    "Save & Apply Key"
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <div></div> }.into_any()
            }
        }}
    }
}
