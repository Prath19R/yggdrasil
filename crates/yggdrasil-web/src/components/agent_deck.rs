use leptos::prelude::*;
use crate::models::agent::AgentRole;
use crate::models::chat::ChatMessage;
use crate::models::diff::{DiffStatus, FileDiff};

#[component]
pub fn AgentDeck(
    messages: ReadSignal<Vec<ChatMessage>>,
    is_loading: ReadSignal<bool>,
    show_thinking: ReadSignal<bool>,
    set_show_thinking: WriteSignal<bool>,
    auto_apply: ReadSignal<bool>,
    set_auto_apply: WriteSignal<bool>,
    on_send_prompt: Callback<String>,
    on_accept_diff: Callback<FileDiff>,
    on_reject_diff: Callback<String>,
) -> impl IntoView {
    let (input_text, set_input_text) = signal(String::new());
    let (active_role, set_active_role) = signal(AgentRole::Orchestrator);

    let handle_submit = move || {
        let text = input_text.get().trim().to_string();
        if !text.is_empty() && !is_loading.get() {
            set_input_text.set(String::new());
            on_send_prompt.run(text);
        }
    };

    view! {
        <aside class="w-96 bg-zinc-950 border-l border-zinc-800 flex flex-col h-full select-none shrink-0">
            // Header
            <div class="h-10 px-3 border-b border-zinc-800 flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <span class="text-base">"💬"</span>
                    <h3 class="text-xs font-semibold text-zinc-200">"Agent Deck"</h3>
                </div>

                <div class="flex items-center gap-3 text-[11px]">
                    <label class="flex items-center gap-1.5 text-zinc-400 hover:text-zinc-200 cursor-pointer">
                        <input
                            type="checkbox"
                            prop:checked=move || show_thinking.get()
                            on:change=move |ev| set_show_thinking.set(event_target_checked(&ev))
                            class="rounded bg-zinc-900 border-zinc-700 text-emerald-500 focus:ring-0 cursor-pointer"
                        />
                        <span>"Thinking"</span>
                    </label>

                    <label class="flex items-center gap-1.5 text-zinc-400 hover:text-zinc-200 cursor-pointer">
                        <input
                            type="checkbox"
                            prop:checked=move || auto_apply.get()
                            on:change=move |ev| set_auto_apply.set(event_target_checked(&ev))
                            class="rounded bg-zinc-900 border-zinc-700 text-emerald-500 focus:ring-0 cursor-pointer"
                        />
                        <span class="text-emerald-400 font-medium">"⚡ Auto-apply"</span>
                    </label>
                </div>
            </div>

            // Role Selector Filter Pills
            <div class="px-2.5 py-1.5 border-b border-zinc-800/80 flex items-center gap-1.5 overflow-x-auto text-[11px] scrollbar-none">
                <button
                    on:click=move |_| set_active_role.set(AgentRole::Orchestrator)
                    class=move || {
                        if active_role.get() == AgentRole::Orchestrator {
                            "px-2.5 py-0.5 rounded-full bg-amber-500/20 text-amber-300 border border-amber-500/40 font-medium"
                        } else {
                            "px-2.5 py-0.5 rounded-full bg-zinc-900 text-zinc-400 hover:text-zinc-200 border border-zinc-800"
                        }
                    }
                >
                    "👑 All"
                </button>
                <button
                    on:click=move |_| set_active_role.set(AgentRole::Coder)
                    class=move || {
                        if active_role.get() == AgentRole::Coder {
                            "px-2.5 py-0.5 rounded-full bg-blue-500/20 text-blue-300 border border-blue-500/40 font-medium"
                        } else {
                            "px-2.5 py-0.5 rounded-full bg-zinc-900 text-zinc-400 hover:text-zinc-200 border border-zinc-800"
                        }
                    }
                >
                    "💻 Coder"
                </button>
                <button
                    on:click=move |_| set_active_role.set(AgentRole::Debugger)
                    class=move || {
                        if active_role.get() == AgentRole::Debugger {
                            "px-2.5 py-0.5 rounded-full bg-red-500/20 text-red-300 border border-red-500/40 font-medium"
                        } else {
                            "px-2.5 py-0.5 rounded-full bg-zinc-900 text-zinc-400 hover:text-zinc-200 border border-zinc-800"
                        }
                    }
                >
                    "🐞 Debugger"
                </button>
                <button
                    on:click=move |_| set_active_role.set(AgentRole::Deployer)
                    class=move || {
                        if active_role.get() == AgentRole::Deployer {
                            "px-2.5 py-0.5 rounded-full bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 font-medium"
                        } else {
                            "px-2.5 py-0.5 rounded-full bg-zinc-900 text-zinc-400 hover:text-zinc-200 border border-zinc-800"
                        }
                    }
                >
                    "🚀 Deployer"
                </button>
            </div>

            // Chat Message Scroll View
            <div class="flex-1 overflow-y-auto p-3 space-y-3.5 text-xs font-sans">
                {move || {
                    let msgs = messages.get();
                    let thinking_visible = show_thinking.get();

                    msgs.into_iter().map(|msg| {
                        let is_user = msg.is_user;
                        let sender = msg.sender.clone();
                        let text = msg.text.clone();
                        let time = msg.timestamp.clone();
                        let thinking = msg.thinking.clone();
                        let diff_opt = msg.diff.clone();

                        view! {
                            <div class=format!(
                                "p-3 rounded-xl border transition shadow-sm {}",
                                if is_user {
                                    "bg-zinc-900/90 border-blue-500/30 text-zinc-100 ml-4"
                                } else {
                                    "bg-[#111218] border-zinc-800/90 text-zinc-200 mr-2"
                                }
                            )>
                                // Sender & Timestamp
                                <div class="flex items-center justify-between pb-1.5 mb-1.5 border-b border-zinc-800/60 text-[11px]">
                                    <span class=if is_user { "font-semibold text-blue-400" } else { "font-semibold text-amber-400" }>
                                        {sender}
                                    </span>
                                    <span class="text-zinc-500 font-mono text-[10px]">{time}</span>
                                </div>

                                // Thinking trace
                                {
                                    if thinking_visible {
                                        if let Some(t) = thinking {
                                            view! {
                                                <details class="mb-2 p-2 rounded-lg bg-zinc-950/80 border border-zinc-800 text-[11px] font-mono text-zinc-400">
                                                    <summary class="cursor-pointer text-amber-400/90 hover:text-amber-300 font-medium select-none">
                                                        "💭 Reasoning / Thinking Trace"
                                                    </summary>
                                                    <pre class="mt-2 whitespace-pre-wrap leading-relaxed text-zinc-400 font-mono text-[10px] overflow-x-auto">
                                                        {t}
                                                    </pre>
                                                </details>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }

                                // Message Text
                                <div class="whitespace-pre-wrap leading-relaxed">{text}</div>

                                // High-Contrast Code Diff Proposal Card
                                {
                                    if let Some(diff) = diff_opt {
                                        let diff_for_accept = diff.clone();
                                        let diff_id_for_reject = diff.id.clone();
                                        let accept_cb = on_accept_diff;
                                        let reject_cb = on_reject_diff;

                                        view! {
                                            <div class="mt-3 p-3 rounded-xl bg-emerald-950/20 border-2 border-emerald-500/60 shadow-lg text-xs font-mono">
                                                <div class="flex items-center justify-between mb-2">
                                                    <div class="flex items-center gap-1.5 font-bold text-emerald-400 text-xs">
                                                        <span>"📝"</span>
                                                        <span>"PROPOSED CODE DIFF"</span>
                                                    </div>
                                                    <span class="text-[10px] px-2 py-0.5 rounded-full font-semibold border bg-emerald-500/10 text-emerald-400 border-emerald-500/30">
                                                        {match diff.status {
                                                            DiffStatus::Pending => "⏳ PENDING REVIEW",
                                                            DiffStatus::Accepted => "✔ APPLIED TO FILE",
                                                            DiffStatus::Rejected => "✖ REJECTED",
                                                        }}
                                                    </span>
                                                </div>

                                                <div class="text-[11px] text-zinc-300 mb-1">
                                                    <span class="text-zinc-500">"File: "</span>
                                                    <span class="text-white font-semibold">{diff.path}</span>
                                                </div>

                                                <p class="text-[10px] text-zinc-400 mb-2 italic">{diff.summary}</p>

                                                // Code preview
                                                <div class="max-h-36 overflow-y-auto p-2 rounded-lg bg-black/60 border border-zinc-800 text-[11px] text-zinc-300 whitespace-pre font-mono">
                                                    {diff.proposed_content}
                                                </div>

                                                // Buttons
                                                {
                                                    if diff.status == DiffStatus::Pending {
                                                        view! {
                                                            <div class="flex items-center gap-2 mt-3 pt-2 border-t border-zinc-800">
                                                                <button
                                                                    on:click=move |_| accept_cb.run(diff_for_accept.clone())
                                                                    class="flex-1 py-1.5 text-xs font-bold bg-emerald-500 hover:bg-emerald-400 text-zinc-950 rounded-lg transition shadow-md cursor-pointer text-center"
                                                                >
                                                                    "✔ Accept & Apply Code to Disk"
                                                                </button>
                                                                <button
                                                                    on:click=move |_| reject_cb.run(diff_id_for_reject.clone())
                                                                    class="px-3 py-1.5 text-xs font-medium bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-white rounded-lg border border-zinc-700/60 transition cursor-pointer"
                                                                >
                                                                    "✖ Reject"
                                                                </button>
                                                            </div>
                                                        }.into_any()
                                                    } else if diff.status == DiffStatus::Accepted {
                                                        view! {
                                                            <div class="mt-2 text-[11px] text-emerald-400 font-semibold flex items-center gap-1.5">
                                                                <span>"✔ Changes successfully applied and saved."</span>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <div></div> }.into_any()
                                                    }
                                                }
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <div></div> }.into_any()
                                    }
                                }
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}

                // Loading Indicator
                {move || {
                    if is_loading.get() {
                        view! {
                            <div class="p-3 rounded-xl bg-zinc-900/60 border border-zinc-800 text-xs text-zinc-400 flex items-center gap-2 animate-pulse font-mono">
                                <span class="animate-spin text-sm">"⚡"</span>
                                <span>"Master Orchestrator reasoning & executing..."</span>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>

            // Chat Input Area
            <div class="p-3 border-t border-zinc-800 bg-zinc-950 shrink-0">
                <div class="flex gap-2">
                    <textarea
                        prop:value=move || input_text.get()
                        on:input=move |ev| set_input_text.set(event_target_value(&ev))
                        on:keydown=move |ev| {
                            if ev.key() == "Enter" && !ev.shift_key() {
                                ev.prevent_default();
                                handle_submit();
                            }
                        }
                        placeholder="Instruct Orchestrator (e.g. 'implement JWT auth', 'fix compiler errors')..."
                        rows="2"
                        class="flex-1 p-2.5 bg-zinc-900 border border-zinc-800 rounded-xl text-xs text-zinc-100 placeholder-zinc-500 focus:outline-none focus:border-emerald-500/70 resize-none font-sans"
                    ></textarea>

                    <button
                        on:click=move |_| handle_submit()
                        disabled=move || is_loading.get()
                        class="px-4 bg-emerald-500 hover:bg-emerald-400 disabled:opacity-50 text-zinc-950 font-bold rounded-xl transition text-xs flex items-center justify-center shadow-md cursor-pointer"
                    >
                        <span>"Send ➔"</span>
                    </button>
                </div>
            </div>
        </aside>
    }
}
