use ai_core::ai::{Models, ProviderStatus};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Shared progress state accessible by both parent and child components.
#[derive(Clone)]
struct InstallState {
    store: Arc<Mutex<HashMap<usize, (u64, u64)>>>,
    tick: Signal<u64>,
}

impl PartialEq for InstallState {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.store, &other.store) && self.tick == other.tick
    }
}

#[component]
pub fn NeuralEnginesPage() -> Element {
    let mut models_registry = use_context::<Signal<Models>>();

    let install_state = use_hook(|| {
        InstallState {
            store: Arc::new(Mutex::new(HashMap::new())),
            tick: Signal::new(0u64),
        }
    });

    // Extract model data before rsx
    let models_data: Vec<(String, ProviderStatus, f32)> = {
        let registry = models_registry.read();
        registry
            .models
            .iter()
            .map(|m| (m.name().to_string(), m.status(), m.temperature()))
            .collect()
    };

    // Read current progress snapshot — subscribe to tick so we re-render
    let progress_snapshot = {
        let _ = install_state.tick.read();
        install_state.store.lock().unwrap().clone()
    };

    let model_cards = models_data
        .into_iter()
        .enumerate()
        .map(|(index, (model_name, model_status, model_temp))| {
            let progress_entry = progress_snapshot.get(&index).copied();
            let install_state = install_state.clone();

            rsx! {
                ModelCard {
                    key: "{index}",
                    index,
                    model_name,
                    model_status,
                    model_temp,
                    progress_entry,
                    install_state,
                    models_registry,
                }
            }
        });

    rsx! {
        main { class: "ml-64 pt-12 pb-12 px-12 h-screen bg-background overflow-y-auto",
            div { class: "max-w-7xl mx-auto",
                // Editorial Header
                section { class: "mb-12",
                    p { class: "font-label text-xs uppercase tracking-[0.2em] text-secondary mb-2 font-semibold",
                        "Core Configuration"
                    }
                    h2 { class: "font-headline text-5xl font-bold tracking-tight text-on-surface mb-4",
                        "Neural Engines"
                    }
                    p { class: "text-on-surface-variant max-w-2xl text-lg leading-relaxed",
                        "Fine-tune the weights, context windows, and operational parameters for your synthetic workforce. Each engine represents a specialized cognitive layer."
                    }
                }
                // Model Grid (Bento Style)
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6 mb-12",
                    {model_cards}
                }
            }
        }
    }
}

#[component]
fn ModelCard(
    index: usize,
    model_name: String,
    model_status: ProviderStatus,
    model_temp: f32,
    progress_entry: Option<(u64, u64)>,
    install_state: InstallState,
    mut models_registry: Signal<Models>,
) -> Element {
    let (percent, progress_text) = match progress_entry {
        Some((downloaded, total)) => {
            let p = if total > 0 {
                ((downloaded as f64 / total as f64) * 100.0).min(100.0)
            } else {
                0.0
            };
            let dl_mb = downloaded as f64 / 1_048_576.0;
            let tot_mb = total as f64 / 1_048_576.0;
            let text = if total > 0 {
                format!("{:.1} / {:.1} MB ({:.0}%)", dl_mb, tot_mb, p)
            } else {
                format!("{:.1} MB", dl_mb)
            };
            (p, Some(text))
        }
        None => (0.0, None),
    };

    rsx! {
        div {
            class: "bg-surface-container p-8 rounded-xl flex flex-col gap-8 relative overflow-hidden group transition-all duration-300 hover:bg-surface-container-highest hover:shadow-2xl hover:border-primary/50 border border-outline-variant/10 hover:translate-y-[-4px]",
            div { class: "absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full -translate-y-1/2 translate-x-1/2 blur-3xl" }
            div { class: "flex justify-between items-start z-10",
                div {
                    h3 { class: "font-headline text-2xl font-bold truncate", "{model_name}" }
                    p { class: "text-xs text-on-surface-variant font-medium tracking-wide uppercase", "AI PROVIDER CONFIG" }
                }
                label { class: "group relative inline-flex items-center cursor-pointer",
                    input {
                        "type": "checkbox",
                        class: "sr-only peer",
                        checked: model_status == ProviderStatus::Ready,
                        onchange: {
                            let install_state = install_state.clone();
                            move |_| {
                                let mut tick = install_state.tick;

                                // Show installing state immediately
                                install_state.store.lock().unwrap().insert(index, (0, 0));
                                tick.with_mut(|v| *v += 1);

                                let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(u64, u64)>();

                                let store = install_state.store.clone();
                                spawn(async move {
                                    let mut last_update = std::time::Instant::now();
                                    let mut latest = (0u64, 0u64);

                                    while let Some((downloaded, total)) = rx.recv().await {
                                        latest = (downloaded, total);

                                        if last_update.elapsed() > std::time::Duration::from_millis(100) {
                                            store.lock().unwrap().insert(index, latest);
                                            tick.with_mut(|v| *v += 1);
                                            last_update = std::time::Instant::now();
                                        }
                                    }

                                    store.lock().unwrap().remove(&index);
                                    tick.with_mut(|v| *v += 1);
                                });

                                spawn(async move {
                                    let _ = models_registry.peek().setup(index, Some(tx)).await;
                                    models_registry.write();
                                });
                            }
                        }
                    }
                    div { class: "w-11 h-6 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-secondary ring-2 ring-primary/15 hover:ring-primary/30 transition-all" }
                }
            }

            // Installation Progress Bar
            if let Some(text) = progress_text {
                div { class: "z-10",
                    div { class: "flex justify-between items-center mb-2",
                        span { class: "text-[10px] font-bold uppercase tracking-widest text-primary animate-pulse", "Installing..." }
                        span { class: "text-[10px] font-mono text-on-surface-variant",
                            "{text}"
                        }
                    }
                    div { class: "w-full h-2 bg-surface-container-highest rounded-full overflow-hidden",
                        div {
                            class: "h-full bg-white rounded-full transition-all duration-300",
                            width: "{percent}%"
                        }
                    }
                }
            }

            div { class: format!("space-y-6 z-10 {}", if model_status != ProviderStatus::Ready { "opacity-50" } else { "" }),
                div {
                    label { class: "block text-[10px] font-bold uppercase tracking-widest text-outline mb-2", "API Configuration / Setup" }
                    div { class: "flex gap-2",
                        input {
                            "type": "text",
                            readonly: true,
                            class: "bg-surface-container-lowest border border-outline-variant/20 rounded-lg px-4 py-2 text-sm w-full text-on-surface-variant focus:border-secondary/40 outline-none transition-all cursor-not-allowed",
                            value: match model_status {
                                ProviderStatus::Ready => "Authenticated",
                                ProviderStatus::RequiresAuth => "Needs Login",
                                ProviderStatus::RequiresInstallation => "Needs Install",
                            }
                        }
                        button { class: "bg-surface-container-highest px-3 py-2 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright transition-colors", "REVEAL" }
                    }
                }
                div {
                    div { class: "flex justify-between items-center mb-2",
                        label { class: "text-[10px] font-bold uppercase tracking-widest text-outline", "Temperature" }
                        span { class: "text-xs font-mono text-secondary", "{model_temp:.2}" }
                    }
                    input {
                        class: "w-full accent-secondary",
                        "type": "range",
                        min: "0.0",
                        max: "2.0",
                        step: "0.01",
                        value: "{model_temp}",
                        oninput: move |e| {
                            if let Ok(temp) = e.value().parse::<f32>() {
                                models_registry.read().models[index].set_temperature(temp);
                                models_registry.write();
                            }
                        }
                    }
                }
                div {
                    label { class: "block text-[10px] font-bold uppercase tracking-widest text-outline mb-2", "Context Window" }
                    select { class: "w-full bg-surface-container-lowest border border-outline-variant/20 rounded-lg px-4 py-2 text-sm text-on-surface-variant focus:border-secondary/40 outline-none appearance-none cursor-pointer",
                        option { "128k Tokens (Standard)" }
                        option { "200k Tokens (Deep Scan)" }
                        option { "1M Tokens (Vast)" }
                    }
                }
            }
            if model_status == ProviderStatus::Ready {
                div { class: "mt-auto z-10 pt-4",
                    div { class: "p-4 rounded-lg bg-secondary/5 border border-secondary/10 flex items-center gap-3",
                        div { class: "w-2 h-2 rounded-full bg-[#00ff9d] animate-pulse shadow-[0_0_15px_rgba(0,255,157,0.7)]" }
                        span { class: "text-[10px] font-bold uppercase tracking-widest text-[#00ff9d]", "ACTIVE: READY" }
                    }
                }
            }
        }
    }
}
