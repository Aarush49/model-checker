use crate::Route;
use crate::message::Message;
use dioxus::desktop::use_window;
use dioxus::prelude::*;

/// Derive a session name from the first user prompt in a message list.
fn session_name_from_messages(msgs: &[Message]) -> String {
    for msg in msgs {
        if let Message::User { content } = msg {
            let trimmed = content.trim();
            if trimmed.len() > 30 {
                return format!("{}...", &trimmed[..30]);
            }
            return trimmed.to_string();
        }
    }
    "New Chat".to_string()
}

#[component]
pub fn Layout() -> Element {
    let mut messages = use_context::<Signal<Vec<crate::message::Message>>>();
    let mut chat_history = use_context::<Signal<Vec<(String, Vec<crate::message::Message>)>>>();
    let mut active_session = use_context::<Signal<Option<usize>>>();

    rsx! {
        div { class: "bg-background text-on-surface font-body selection:bg-secondary/30 h-screen overflow-hidden flex flex-col",
            TitleBar {}

            div { class: "flex-1 flex overflow-hidden",
                // SideNavBar - Adjusting py-6 to account for title bar
                aside { class: "w-64 flex flex-col bg-[#091328] py-8 z-40 relative",
                    div { class: "px-6 mb-8 flex items-center gap-3",
                        div { class: "w-8 h-8 rounded bg-gradient-to-br from-primary to-primary-dim flex items-center justify-center",
                            span {
                                class: "material-symbols-outlined text-on-primary text-sm",
                                style: "font-variation-settings: 'FILL' 1;",
                                "insights"
                            }
                        }
                        h1 { class: "text-xl font-bold tracking-tight text-[#dee5ff] font-['Space_Grotesk']",
                                "ModelCheck"
                            }
                    }
                    div { class: "px-4 mb-8",
                        button {
                            class: "w-full py-3 px-4 bg-gradient-to-r from-primary to-primary-dim text-on-primary font-bold rounded-xl flex items-center justify-center gap-2 shadow-md hover:shadow-lg hover:scale-[1.02] active:scale-95 active:shadow-sm active:brightness-90 transition-all duration-200",
                            onclick: move |_| {
                                let current_msgs = messages.read().clone();
                                if !current_msgs.is_empty() {
                                    // If we are viewing a saved session, update it in-place
                                    if let Some(idx) = *active_session.read() {
                                        let mut history = chat_history.write();
                                        if let Some(entry) = history.get_mut(idx) {
                                            entry.1 = current_msgs;
                                        }
                                    } else {
                                        // Archive as a new session
                                        let name = session_name_from_messages(&current_msgs);
                                        chat_history.write().push((name, current_msgs));
                                    }
                                }
                                messages.write().clear();
                                active_session.set(None);
                            },
                            span { class: "material-symbols-outlined text-sm", "add" }
                            span { class: "font-label", "New Session" }
                        }
                    }
                    nav { class: "flex-1 flex flex-col min-h-0",
                        div { class: "space-y-1",
                            Link {
                                to: Route::ChatPage {},
                                class: "px-4 py-3 mx-2 flex items-center gap-3 transition-all duration-200 active:scale-[0.98] rounded-lg group text-[#dee5ff]/60 hover:text-[#dee5ff] hover:bg-[#141f38]",
                                active_class: "text-[#5D3FD3] bg-[#141f38] font-bold",
                                span { class: "material-symbols-outlined", "chat_bubble" }
                                span { class: "font-body font-medium", "Chat" }
                            }

                            Link {
                                to: Route::NeuralEnginesPage {},
                                class: "px-4 py-3 mx-2 flex items-center gap-3 transition-all duration-200 active:scale-[0.98] rounded-lg group text-[#dee5ff]/60 hover:text-[#dee5ff] hover:bg-[#141f38]",
                                active_class: "text-[#5D3FD3] bg-[#141f38] font-bold",
                                span { class: "material-symbols-outlined", "settings_input_component" }
                                span { class: "font-body font-medium", "Neural Engines" }
                            }
                        }

                        // ── Chat History ──────────────────────────────────
                        if !chat_history.read().is_empty() {
                            div { class: "mt-4 pt-4 mx-4 border-t border-outline-variant/10 flex flex-col min-h-0 flex-1",
                                p { class: "text-[10px] uppercase tracking-[0.15em] text-on-surface-variant/50 font-label px-2 mb-2",
                                    "History"
                                }
                                div { class: "overflow-y-auto flex-1 space-y-0.5 scrollbar-hide",
                                    for (index , (name , saved_msgs)) in chat_history.read().iter().enumerate().rev() {
                                        {
                                            let is_active = (*active_session.read()) == Some(index);
                                            let name_clone = name.clone();
                                            // Get the full first user prompt for tooltip
                                            let full_prompt = saved_msgs.iter().find_map(|m| {
                                                if let Message::User { content } = m { Some(content.clone()) } else { None }
                                            }).unwrap_or_default();
                                            rsx! {
                                                div {
                                                    key: "{index}",
                                                    title: "{full_prompt}",
                                                    class: format!(
                                                        "group/item w-full text-left px-3 py-2 rounded-lg flex items-center gap-2.5 text-xs transition-all duration-150 cursor-pointer {}",
                                                        if is_active {
                                                            "bg-[#141f38] text-[#dee5ff] font-semibold"
                                                        } else {
                                                            "text-[#dee5ff]/50 hover:text-[#dee5ff]/80 hover:bg-[#141f38]/50"
                                                        }
                                                    ),
                                                    onclick: move |_| {
                                                        // Save current unsaved chat before switching
                                                        let current = messages.read().clone();
                                                        if !current.is_empty() {
                                                            if let Some(prev_idx) = *active_session.read() {
                                                                // Update existing saved session
                                                                let mut history = chat_history.write();
                                                                if let Some(entry) = history.get_mut(prev_idx) {
                                                                    entry.1 = current;
                                                                }
                                                            } else {
                                                                // Archive the current unsaved chat as a new entry
                                                                let name = session_name_from_messages(&current);
                                                                chat_history.write().push((name, current));
                                                            }
                                                        }
                                                        // Load the clicked session
                                                        let loaded = chat_history.read()[index].1.clone();
                                                        messages.set(loaded);
                                                        active_session.set(Some(index));
                                                    },
                                                    span { class: "material-symbols-outlined text-[14px] opacity-60", "forum" }
                                                    span { class: "truncate flex-1", "{name_clone}" }
                                                    // Delete button
                                                    button {
                                                        class: "ml-auto p-0.5 rounded opacity-0 group-hover/item:opacity-100 hover:text-red-400 hover:bg-red-400/10 transition-all",
                                                        onclick: move |evt: Event<MouseData>| {
                                                            evt.stop_propagation();
                                                            let current_active = *active_session.read();
                                                            // If deleting the active session, clear the chat
                                                            if current_active == Some(index) {
                                                                messages.write().clear();
                                                                active_session.set(None);
                                                            } else if let Some(active_idx) = current_active {
                                                                // Adjust active index if it's after the deleted one
                                                                if active_idx > index {
                                                                    active_session.set(Some(active_idx - 1));
                                                                }
                                                            }
                                                            chat_history.write().remove(index);
                                                        },
                                                        span { class: "material-symbols-outlined text-[12px]", "close" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "mt-auto pt-6 border-t border-outline-variant/10 space-y-1",
                        a {
                            href: "https://github.com/neeleshpoli/model-checker",
                            target: "_blank",
                            class: "text-[#dee5ff]/60 hover:text-[#dee5ff] px-6 py-2 flex items-center gap-3 hover:bg-[#141f38] transition-colors",
                            span { class: "material-symbols-outlined text-sm", "help" }
                            span { class: "text-sm font-label", "Support" }
                        }
                    }
                }

                div { class: "flex-1 relative h-full bg-surface",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

#[component]
fn TitleBar() -> Element {
    let window = use_window();
    let mut is_maximized = use_signal(|| window.is_maximized());

    let onmousedown = {
        let window = window.clone();
        move |_| window.drag()
    };

    let ondoubleclick = {
        let window = window.clone();
        move |_| {
            let next = !window.is_maximized();
            window.set_maximized(next);
            is_maximized.set(next);
        }
    };

    let onmin = {
        let window = window.clone();
        move |_| window.set_minimized(true)
    };

    let onmax = {
        let window = window.clone();
        move |_| {
            let next = !window.is_maximized();
            window.set_maximized(next);
            is_maximized.set(next);
        }
    };

    let onclose = {
        let window = window.clone();
        move |_| window.close()
    };

    rsx! {
        header {
            class: "h-11 flex-shrink-0 flex items-center justify-between bg-[#091328] border-b border-outline-variant/10 z-50 select-none",
            onmousedown,
            ondoubleclick,

            // PLATFORM SPECIFIC: Left side spacer or native button area
            div { class: "flex items-center pl-4 gap-2",
                if cfg!(target_os = "macos") {
                    div { class: "w-20" } // Space for native traffic lights
                } else {
                    div { class: "flex items-center gap-2",
                        span { class: "material-symbols-outlined text-[14px] text-primary/70", "insights" }
                        span { class: "text-[10px] font-headline font-bold uppercase tracking-widest text-[#dee5ff]/40", "ModelCheck" }
                    }
                }
            }

            // Central Drag Area (invisible spacer)
            div { class: "flex-1 h-full cursor-default" }

            // PLATFORM SPECIFIC: Window Controls for Windows/Others
            if !cfg!(target_os = "macos") {
                div { class: "flex h-full",
                    WindowButton {
                        icon: "remove",
                        onclick: onmin,
                        class: "hover:bg-white/5 active:bg-white/10"
                    }
                    WindowButton {
                        icon: if is_maximized() { "tab_unselected" } else { "check_box_outline_blank" },
                        onclick: onmax,
                        class: "hover:bg-white/5 active:bg-white/10"
                    }
                    WindowButton {
                        icon: "close",
                        onclick: onclose,
                        class: "hover:bg-[#e81123] hover:text-white active:bg-[#f1707a]"
                    }
                }
            } else {
                div { class: "w-4" }
            }
        }
    }
}

#[component]
fn WindowButton(icon: String, onclick: EventHandler<MouseEvent>, class: String) -> Element {
    rsx! {
        button {
            class: "w-12 h-full flex items-center justify-center text-[#dee5ff]/80 transition-all duration-75 {class}",
            onmousedown: |e| e.stop_propagation(), // CRITICAL: Stop drag propagation
            onclick: move |e| onclick.call(e),
            span { class: "material-symbols-outlined text-[17px]", "{icon}" }
        }
    }
}
