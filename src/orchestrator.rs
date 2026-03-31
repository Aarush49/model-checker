use crate::Message;
use ai_core::ai::{Models, ProviderStatus};
use dioxus::prelude::*;
use std::collections::HashSet;
#[component]
pub fn OrchestratorPage() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut prompt = use_signal(String::new);
    let models_registry = use_context::<Signal<Models>>();
    let mut selected_models = use_signal(|| HashSet::<String>::new());

    use_effect(move || {
        let registry = models_registry.read();
        if selected_models.read().is_empty() {
            let mut initial = HashSet::new();
            for m in registry.models.iter() {
                if m.status() == ProviderStatus::Ready {
                    initial.insert(m.id().to_string());
                }
            }
            if !initial.is_empty() {
                selected_models.set(initial);
            }
        }
    });

    let models_data = {
        let registry = models_registry.read();
        registry.models.iter().map(|m| {
            (m.id().to_string(), m.name().to_string(), m.status())
        }).collect::<Vec<_>>()
    };

    let mut send_message = move || {
        let current_prompt = prompt().clone();
        let selected = selected_models.read().clone();

        if !current_prompt.is_empty() {
            messages.write().push(Message::User {
                content: current_prompt.clone(),
            });

            prompt.set(String::new());

            spawn(async move {
                let result = models_registry
                    .read()
                    .ask(current_prompt, selected)
                    .await;

                println!("Result from ask: {:?}", result);

                let responses = result.unwrap_or_default();

                messages.write().push(Message::AI {
                    responses,
                });
            });
        }
    };

    rsx! {
        main { class: "ml-64 pt-0 h-screen flex flex-col bg-surface",
            // Model Selector Row
            section { class: "px-8 py-6 shrink-0 relative z-20",
                div { class: "flex items-center justify-between mb-4",
                    div { class: "flex flex-col",
                        span { class: "text-[10px] uppercase tracking-widest text-secondary font-bold font-label mb-1",
                            "Active Ensemble"
                        }
                        h2 { class: "text-2xl font-headline font-bold text-on-surface",
                            "Orchestrator"
                        }
                    }
                    div { class: "flex items-center gap-2",
                        span { class: "text-xs text-on-surface-variant font-label mr-2",
                            "{models_registry.read().models.len()} Models Loaded"
                        }
                    }
                }
                div { class: "flex gap-4 overflow-x-auto pt-2 pb-2 scrollbar-hide",
                    for (index, (model_id, model_name, model_status)) in models_data.into_iter().enumerate() {
                        div {
                            key: "{index}",
                            class: format!("select-none cursor-pointer flex-shrink-0 w-64 p-4 rounded-xl bg-surface-container border-2 flex flex-col gap-3 relative overflow-hidden group transition-all duration-300 hover:translate-y-[-4px] hover:shadow-xl hover:z-10 {}", if selected_models.read().contains(&model_id) { "border-primary/80 bg-primary/5 shadow-[0_0_15px_rgba(0,227,253,0.1)]" } else if model_status == ProviderStatus::Ready { "border-primary/20 opacity-70 hover:opacity-100 hover:border-primary/40 hover:bg-surface-container-highest" } else { "border-outline-variant/20 opacity-60 hover:opacity-90 hover:border-outline-variant/40" }),
                            onclick: move |_| {
                                let mut selected = selected_models.write();
                                if selected.contains(&model_id) {
                                    selected.remove(&model_id);
                                } else {
                                    selected.insert(model_id.clone());
                                }
                            },
                            if selected_models.read().contains(&model_id) {
                                div { class: "absolute top-0 right-0 p-2 transition-opacity",
                                    span { class: "material-symbols-outlined text-primary text-sm", style: "font-variation-settings: 'FILL' 1;", "check_circle" }
                                }
                            } else {
                                div { class: "absolute top-0 right-0 p-2 opacity-0 group-hover:opacity-100 transition-opacity",
                                    span { class: "material-symbols-outlined text-outline text-sm opacity-50 hover:opacity-100", "add_circle" }
                                }
                            }
                            div { class: "flex items-center gap-3",
                                div { class: "w-10 h-10 rounded-lg bg-surface-container-highest flex items-center justify-center",
                                    span { class: "material-symbols-outlined text-primary", "bolt" }
                                }
                                div {
                                    h3 { class: "font-bold text-sm font-headline truncate", "{model_name}" }
                                    p { class: "text-[10px] text-on-surface-variant truncate", "AI Provider" }
                                }
                            }
                            div { class: "flex items-center justify-between mt-2",
                                match model_status {
                                    ProviderStatus::Ready => rsx!{ span { class: "text-[10px] bg-primary/10 text-primary px-2 py-0.5 rounded uppercase font-bold tracking-tighter", "Ready" } },
                                    ProviderStatus::RequiresAuth => rsx!{ span { class: "text-[10px] bg-error/10 text-error px-2 py-0.5 rounded uppercase font-bold tracking-tighter", "Needs Auth" } },
                                    ProviderStatus::RequiresInstallation => rsx!{ span { class: "text-[10px] bg-tertiary/10 text-tertiary px-2 py-0.5 rounded uppercase font-bold tracking-tighter", "Install" } },
                                }
                                div { class: "flex gap-1",
                                    div { class: "w-1.5 h-1.5 rounded-full bg-secondary" }
                                    div { class: "w-1.5 h-1.5 rounded-full bg-secondary" }
                                    div { class: "w-1.5 h-1.5 rounded-full bg-secondary/30" }
                                }
                            }
                        }
                    }
                    // Add More Button
                    button { class: "flex-shrink-0 w-24 rounded-xl border-2 border-dashed border-outline-variant/30 flex flex-col items-center justify-center gap-2 hover:bg-surface-container-highest hover:border-primary/50 transition-all group",
                        span { class: "material-symbols-outlined text-on-surface-variant group-hover:text-primary transition-colors", "add_circle" }
                        span { class: "text-[10px] font-bold uppercase tracking-tighter text-on-surface-variant group-hover:text-on-surface", "Models" }
                    }
                }
            }
            // Chat Area
            div { class: "flex-1 overflow-y-auto px-8 pb-32 pt-6",
                div { class: "max-w-4xl mx-auto space-y-12",
                    for msg in messages() {
                        match msg {
                            Message::User { content } => rsx! {
                                div { class: "flex flex-col items-end gap-2 group",
                                    div { class: "max-w-[80%] bg-surface-container-highest px-6 py-4 rounded-2xl rounded-tr-none border border-outline-variant/20 shadow-lg",
                                        p { class: "text-on-surface font-body leading-relaxed whitespace-pre-wrap", "{content}" }
                                    }
                                    span { class: "text-[10px] text-on-surface-variant font-label opacity-0 group-hover:opacity-100 transition-opacity",
                                        "Just now • Orchestrated Request"
                                    }
                                }
                            },
                            Message::AI { responses } => rsx! {
                                div { class: "space-y-6",
                                    for (model_name, response_text) in responses {
                                        div { class: "bg-surface-container p-6 rounded-2xl border border-primary/20 relative overflow-hidden",
                                            div { class: "absolute top-0 left-0 w-1 h-full bg-primary" }
                                            div { class: "flex items-center justify-between mb-6",
                                                div { class: "flex items-center gap-3",
                                                    span { class: "material-symbols-outlined text-primary", "bolt" }
                                                    span { class: "font-headline font-bold text-sm tracking-tight text-on-surface", "{model_name}" }
                                                    div { class: "h-1 w-1 rounded-full bg-outline-variant" }
                                                    span { class: "text-[10px] text-on-surface-variant font-label uppercase", "Reasoning Complete" }
                                                }
                                            }
                                            div { class: "space-y-4 font-body text-on-surface/90 leading-relaxed text-sm whitespace-pre-wrap",
                                                "{response_text}"
                                            }
                                            div { class: "mt-6 flex items-center gap-4 border-t border-outline-variant/10 pt-4",
                                                button { class: "flex items-center gap-1.5 text-xs text-on-surface-variant hover:text-secondary transition-colors",
                                                    span { class: "material-symbols-outlined text-sm", "thumb_up" }
                                                }
                                                button { class: "flex items-center gap-1.5 text-xs text-on-surface-variant hover:text-tertiary transition-colors",
                                                    span { class: "material-symbols-outlined text-sm", "content_copy" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Bottom Message Bar
            footer { class: "fixed bottom-0 right-0 w-[calc(100%-16rem)] p-8",
                div { class: "max-w-4xl mx-auto",
                    div { class: "glass-panel bg-surface-container-highest/60 backdrop-blur-xl border border-outline-variant/30 rounded-2xl p-2 shadow-2xl",
                        div { class: "flex items-center gap-2 px-4 py-2 mb-1",
                            span { class: "text-[10px] font-bold text-on-surface-variant uppercase tracking-tighter",
                                "Shift + Enter for new line"
                            }
                        }
                        div { class: "flex items-end gap-3 px-2",
                            textarea {
                                class: "flex-1 bg-transparent border-none focus:ring-0 text-on-surface placeholder:text-on-surface-variant/50 font-body py-4 px-4 resize-none min-h-[56px] outline-none",
                                placeholder: "Prompt all active models...",
                                rows: "1",
                                value: "{prompt}",
                                oninput: move |event| prompt.set(event.value()),
                                onkeydown: move |event| {
                                    if event.key() == Key::Enter && !event.modifiers().contains(Modifiers::SHIFT) {
                                        event.prevent_default();
                                        send_message();
                                    }
                                }
                            }
                            button {
                                class: "mb-2 mr-2 w-12 h-12 rounded-xl bg-gradient-to-br from-primary to-primary-dim flex items-center justify-center text-on-primary shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all outline-none",
                                onclick: move |_| send_message(),
                                span { class: "material-symbols-outlined", "send" }
                            }
                        }
                    }

                }
            }
        }
    }
}
