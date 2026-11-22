use leptos::prelude::*;
use crate::models::file::FileItem;
use crate::services::TauriService;

#[component]
pub fn FileTree(
    root_item: ReadSignal<FileItem>,
    on_select_file: Callback<FileItem>,
    on_toggle_folder: Callback<String>,
) -> impl IntoView {
    view! {
        <aside class="w-64 bg-zinc-950 border-r border-zinc-800 flex flex-col h-full select-none shrink-0">
            // Explorer Header
            <div class="h-9 px-3 border-b border-zinc-800 flex items-center justify-between text-xs font-semibold uppercase tracking-wider text-zinc-400">
                <span>"Explorer"</span>
                <span class=move || {
                    if TauriService::is_available() {
                        "text-[10px] bg-emerald-950/60 border border-emerald-700/60 px-1.5 py-0.5 rounded text-emerald-400 font-bold"
                    } else {
                        "text-[10px] bg-zinc-900 border border-zinc-800 px-1.5 py-0.5 rounded text-zinc-400"
                    }
                }>
                    {move || if TauriService::is_available() { "NATIVE FS" } else { "WASM FS" }}
                </span>
            </div>

            // Tree Scrollable Area
            <div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-0.5 font-mono text-xs">
                {move || {
                    let select = on_select_file;
                    let toggle = on_toggle_folder;
                    let root = root_item.get();
                    view! {
                        <FileNodeView item=root on_select=select on_toggle=toggle />
                    }
                }}
            </div>
        </aside>
    }
}

#[component]
pub fn FileNodeView(
    item: FileItem,
    on_select: Callback<FileItem>,
    on_toggle: Callback<String>,
) -> impl IntoView {
    let item_clone = item.clone();
    let path = item.path.clone();

    if item.is_dir {
        view! {
            <div>
                <div
                    on:click=move |_| on_toggle.run(path.clone())
                    class="flex items-center gap-1.5 px-2 py-1 rounded hover:bg-zinc-800/60 cursor-pointer text-zinc-300 hover:text-white transition group"
                >
                    <span class="text-xs">{item.icon()}</span>
                    <span class="font-medium truncate">{item.name}</span>
                </div>
                {
                    if item.is_expanded {
                        let select = on_select;
                        let toggle = on_toggle;
                        view! {
                            <div class="pl-3.5 border-l border-zinc-800/80 ml-2 space-y-0.5">
                                {item.children.into_iter().map(|child| {
                                    view! {
                                        <FileNodeView item=child on_select=select on_toggle=toggle />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }
            </div>
        }.into_any()
    } else {
        view! {
            <div
                on:click=move |_| on_select.run(item_clone.clone())
                class="flex items-center gap-1.5 px-2 py-1 rounded hover:bg-zinc-800/80 cursor-pointer text-zinc-400 hover:text-zinc-100 transition"
            >
                <span class="text-xs">{item.icon()}</span>
                <span class="truncate">{item.name}</span>
            </div>
        }.into_any()
    }
}
