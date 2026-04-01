use crate::message::{ChatMessage, Message};
use ai_core::ai::{Models, ProviderStatus};
use dioxus::prelude::*;
use std::collections::HashSet;

#[component]
pub fn OrchestratorPage() -> Element {
    let mut messages = use_context::<Signal<Vec<Message>>>();
    let mut prompt = use_signal(String::new);
    let models_registry = use_context::<Signal<Models>>();
    let mut selected_models = use_context::<Signal<Option<HashSet<String>>>>();

    use_effect(move || {
        if selected_models.read().is_none() {
            selected_models.set(Some(HashSet::new()));
        }
    });

    let models_data = {
        let registry = models_registry.read();
        registry
            .models
            .iter()
            .map(|m| (m.id().to_string(), m.name().to_string(), m.status()))
            .collect::<Vec<_>>()
    };

    let mut send_message = move || {
        let current_prompt = prompt().clone();
        let selected = selected_models.read().clone().unwrap_or_default();

        if !current_prompt.is_empty() {
            messages.write().push(Message::User {
                content: current_prompt.clone(),
            });

            prompt.set(String::new());

            if selected.is_empty() {
                messages.write().push(Message::AI {
                    model_id: "system".to_string(),
                    model_name: "System".to_string(),
                    response: String::new(),
                    error: Some("No models are currently selected. Please select at least one model before sending a prompt.".to_string()),
                    is_finished: true,
                });
                return;
            }

            let (tx_ui, mut rx_ui) =
                tokio::sync::mpsc::unbounded_channel::<(String, anyhow::Result<String>)>();

            // Track which message index belongs to which model
            let mut model_indices = std::collections::HashMap::new();

            for model_id in selected.iter() {
                let registry = models_registry.read();
                let model = registry.models.iter().find(|m| m.id() == model_id);

                let model_name = model
                    .map(|m| m.name().to_string())
                    .unwrap_or_else(|| "Unknown Model".to_string());
                let is_ready = model
                    .map(|m| m.status() == ProviderStatus::Ready)
                    .unwrap_or(false);

                messages.write().push(Message::AI {
                    model_id: model_id.clone(),
                    model_name,
                    response: String::new(),
                    error: if is_ready {
                        None
                    } else {
                        Some(
                            "Model requires authentication or setup in Neural Engines before use."
                                .to_string(),
                        )
                    },
                    is_finished: !is_ready,
                });

                let msg_index = messages.read().len() - 1;
                model_indices.insert(model_id.clone(), msg_index);
            }

            // Listener task: routes tokens from the collective stream to per-model message slots
            spawn(async move {
                while let Some((model_id, result)) = rx_ui.recv().await {
                    if let Some(&msg_index) = model_indices.get(&model_id) {
                        let mut msgs = messages.write();
                        if let Some(Message::AI {
                            response, error, ..
                        }) = msgs.get_mut(msg_index)
                        {
                            match result {
                                Ok(token) => response.push_str(&token),
                                Err(e) => *error = Some(e.to_string()),
                            }
                        }
                    }
                }

                // Channel closed — mark all models as finished
                let mut msgs = messages.write();
                for &msg_index in model_indices.values() {
                    if let Some(Message::AI { is_finished, .. }) = msgs.get_mut(msg_index) {
                        *is_finished = true;
                    }
                }
            });

            // Execution Loop
            spawn(async move {
                models_registry
                    .read()
                    .ask(current_prompt, selected, tx_ui)
                    .await;
            });
        }
    };

    rsx! {
        main { class: "ml-64 pt-0 h-screen flex flex-col bg-surface",
            // Model Selector Row
            section { class: "shrink-0 relative z-20 bg-surface-container-low/50 border-b border-outline-variant/10",
                div { class: "max-w-7xl mx-auto px-8 py-3",
                    div { class: "flex items-center justify-between",
                        h2 { class: "text-2xl font-headline font-bold text-on-surface",
                            "Orchestrator"
                        }
                        div { class: "flex items-center gap-2",
                            span { class: "text-xs text-on-surface-variant font-label mr-2",
                                "{models_registry.read().models.len()} Models Loaded"
                            }
                        }
                    }
                }
                div { class: "max-w-7xl mx-auto px-8 pb-3",
                    div { class: "flex gap-2 overflow-x-auto pb-1 scrollbar-hide flex-wrap",
                        for (index , (model_id , model_name , model_status)) in models_data.into_iter().enumerate() {
                            button {
                                key: "{index}",
                                class: format!(
                                    "select-none cursor-pointer flex-shrink-0 px-3 py-1.5 rounded-lg border flex items-center gap-2 text-xs font-headline transition-all duration-200 {}",
                                    if selected_models
                                        .read()
                                        .as_ref()
                                        .map(|s| s.contains(&model_id))
                                        .unwrap_or(false)
                                    {
                                        "border-primary/80 bg-primary/10 text-primary font-bold shadow-[0_0_10px_rgba(0,227,253,0.08)]"
                                    } else if model_status == ProviderStatus::Ready {
                                        "border-transparent text-on-surface-variant font-normal hover:border-outline-variant/50 hover:bg-surface-container-highest hover:text-on-surface"
                                    } else {
                                        "border-transparent text-on-surface-variant font-normal hover:border-outline-variant/50 hover:bg-surface-container-highest hover:text-on-surface"
                                    },
                                ),
                                onclick: move |_| {
                                    let mut selected_opt = selected_models.write();
                                    let mut selected = selected_opt.clone().unwrap_or_default();
                                    if selected.contains(&model_id) {
                                        selected.remove(&model_id);
                                    } else {
                                        selected.insert(model_id.clone());
                                    }
                                    *selected_opt = Some(selected);
                                },
                                if selected_models.read().as_ref().map(|s| s.contains(&model_id)).unwrap_or(false) {
                                    span {
                                        class: "material-symbols-outlined text-[14px] text-primary",
                                        style: "font-variation-settings: 'FILL' 1;",
                                        "check_circle"
                                    }
                                } else {
                                    match model_status {
                                        ProviderStatus::Ready => rsx! {
                                            span { class: "w-1.5 h-1.5 rounded-full bg-primary/50" }
                                        },
                                        ProviderStatus::RequiresAuth => rsx! {
                                            span { class: "w-1.5 h-1.5 rounded-full bg-error/50" }
                                        },
                                        ProviderStatus::RequiresInstallation => rsx! {
                                            span { class: "w-1.5 h-1.5 rounded-full bg-tertiary/50" }
                                        },
                                    }
                                }
                                "{model_name}"
                            }
                        }
                        // Add More Button
                        button { class: "flex-shrink-0 px-3 py-1.5 rounded-lg border border-dashed border-outline-variant/30 flex items-center gap-1.5 text-xs text-on-surface-variant hover:border-primary/50 hover:text-primary transition-all",
                            span { class: "material-symbols-outlined text-[14px]", "add" }
                            "Add"
                        }
                    }
                }
            }
            // Chat Area
            div { class: "flex-1 overflow-y-auto px-8 pb-44 pt-6",
                div { class: "max-w-4xl mx-auto space-y-8",
                    for msg in messages() {
                        ChatMessage { msg }
                    }
                }
            }
            // Bottom Message Bar
            footer { class: "fixed bottom-0 right-0 w-[calc(100%-16rem)] px-8 pb-4 pt-2",
                div { class: "max-w-4xl mx-auto",
                    div { class: "glass-panel bg-surface-container-highest/60 backdrop-blur-xl border border-outline-variant/30 rounded-2xl p-2 shadow-2xl",
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
                                },
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
