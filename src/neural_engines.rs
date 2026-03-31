use ai_core::ai::{Models, ProviderStatus};
use dioxus::prelude::*;

#[component]
pub fn NeuralEnginesPage() -> Element {
    let mut models_registry = use_context::<Signal<Models>>();

    rsx! {
        main { class: "ml-64 pt-12 pb-12 px-12 h-screen bg-background overflow-y-auto",
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
                for (index, model) in models_registry.read().models.iter().enumerate() {
                    div {
                        key: "{index}",
                        class: "bg-surface-container p-8 rounded-xl flex flex-col gap-8 relative overflow-hidden group transition-all duration-300 hover:bg-surface-container-highest hover:shadow-2xl hover:border-primary/50 border border-outline-variant/10 hover:translate-y-[-4px]",
                        div { class: "absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full -translate-y-1/2 translate-x-1/2 blur-3xl" }
                        div { class: "flex justify-between items-start z-10",
                            div {
                                h3 { class: "font-headline text-2xl font-bold truncate", "{model.name()}" }
                                p { class: "text-xs text-on-surface-variant font-medium tracking-wide uppercase", "AI PROVIDER CONFIG" }
                            }
                                label { class: "group relative inline-flex items-center cursor-pointer",
                                input {
                                    "type": "checkbox",
                                    class: "sr-only peer",
                                    checked: model.status() == ProviderStatus::Ready,
                                    onchange: move |_| {
                                        spawn(async move {
                                            models_registry.read().setup(index).await.unwrap();
                                            // Trigger re-render to reflect new status
                                            models_registry.write();
                                        });
                                    }
                                }
                                div { class: "w-11 h-6 bg-surface-container-highest peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-secondary group-hover:ring-4 group-hover:ring-primary/10 transition-all" }
                            }
                        }
                        div { class: format!("space-y-6 z-10 {}", if model.status() != ProviderStatus::Ready { "opacity-50" } else { "" }),
                            div {
                                label { class: "block text-[10px] font-bold uppercase tracking-widest text-outline mb-2", "API Configuration / Setup" }
                                div { class: "flex gap-2",
                                    input {
                                        "type": "text",
                                        readonly: true,
                                        class: "bg-surface-container-lowest border border-outline-variant/20 rounded-lg px-4 py-2 text-sm w-full text-on-surface-variant focus:border-secondary/40 outline-none transition-all cursor-not-allowed",
                                        value: match model.status() {
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
                                    span { class: "text-xs font-mono text-secondary", "0.72" }
                                }
                                input { class: "w-full", "type": "range" }
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
                        if model.status() == ProviderStatus::Ready {
                            div { class: "mt-auto z-10 pt-4",
                                div { class: "p-4 rounded-lg bg-secondary/5 border border-secondary/10 flex items-center gap-3",
                                    div { class: "w-2 h-2 rounded-full bg-secondary animate-pulse shadow-[0_0_10px_rgba(0,227,253,0.5)]" }
                                    span { class: "text-[10px] font-bold uppercase tracking-widest text-secondary", "ACTIVE: READY" }
                                }
                            }
                        }
                    }
                }
            }

            // Global Security Settings
            section { class: "mt-20",
                div { class: "flex items-center gap-4 mb-8",
                    div { class: "h-px flex-1 bg-outline-variant/20" }
                    h3 { class: "font-headline text-xl font-bold tracking-tight text-on-surface-variant",
                        "Global Security Protocol"
                    }
                    div { class: "h-px flex-1 bg-outline-variant/20" }
                }
                div { class: "grid grid-cols-1 lg:grid-cols-4 gap-6",
                    // Fallback Config
                    div { class: "lg:col-span-2 glass-panel p-8 rounded-xl border border-outline-variant/10",
                        div { class: "flex items-center gap-4 mb-6",
                            span { class: "material-symbols-outlined text-secondary", "shuffle" }
                            h4 { class: "font-headline text-lg font-bold", "Dynamic Fallback Pipeline" }
                        }
                        p { class: "text-sm text-on-surface-variant mb-6 leading-relaxed",
                            "Automatically reroute requests when the primary engine latency exceeds 2500ms or encounters rate limits."
                        }
                        div { class: "space-y-4",
                            div { class: "flex items-center justify-between p-4 bg-surface-container-highest/30 rounded-lg",
                                span { class: "text-sm font-medium", "Primary: Curator-X1" }
                                span { class: "material-symbols-outlined text-outline", "arrow_forward" }
                                span { class: "text-sm font-medium text-secondary", "Backup: Muse-Ultra" }
                            }
                            button { class: "w-full py-2 text-xs font-bold uppercase tracking-widest border border-outline-variant/30 rounded-lg hover:bg-surface-container-highest transition-colors",
                                "Configure Routing Chain"
                            }
                        }
                    }
                    // Token Limits
                    div { class: "bg-surface-container-low p-8 rounded-xl border border-outline-variant/5",
                        div { class: "flex items-center gap-4 mb-6",
                            span { class: "material-symbols-outlined text-primary", "toll" }
                            h4 { class: "font-headline text-lg font-bold", "Hard Limits" }
                        }
                        div { class: "space-y-4",
                            div {
                                label { class: "block text-[10px] font-bold uppercase tracking-widest text-outline mb-1", "Monthly Token Cap" }
                                div { class: "text-2xl font-mono text-on-surface",
                                    "50.0M "
                                    span { class: "text-xs text-on-surface-variant", "/ 75.0M" }
                                }
                                div { class: "w-full bg-surface-container-highest h-1.5 rounded-full mt-2 overflow-hidden",
                                    div { class: "bg-primary h-full w-[66%]" }
                                }
                            }
                            div { class: "flex items-center gap-2",
                                span { class: "material-symbols-outlined text-xs text-error", "warning" }
                                span { class: "text-[10px] text-error font-bold uppercase tracking-widest", "Auto-pause at 95% usage" }
                            }
                        }
                    }
                    // Logging
                    div { class: "bg-surface-container-low p-8 rounded-xl border border-outline-variant/5",
                        div { class: "flex items-center gap-4 mb-6",
                            span { class: "material-symbols-outlined text-on-surface-variant", "terminal" }
                            h4 { class: "font-headline text-lg font-bold", "Request Logs" }
                        }
                        div { class: "space-y-3",
                            div { class: "flex items-center justify-between",
                                span { class: "text-xs text-on-surface-variant", "Detailed Debugging" }
                                div { class: "w-8 h-4 bg-secondary/20 rounded-full relative cursor-pointer",
                                    div { class: "absolute right-0.5 top-0.5 w-3 h-3 bg-secondary rounded-full" }
                                }
                            }
                            div { class: "flex items-center justify-between",
                                span { class: "text-xs text-on-surface-variant", "PII Scrubbing" }
                                div { class: "w-8 h-4 bg-surface-container-highest rounded-full relative cursor-pointer",
                                    div { class: "absolute left-0.5 top-0.5 w-3 h-3 bg-outline rounded-full" }
                                }
                            }
                            div { class: "flex items-center justify-between",
                                span { class: "text-xs text-on-surface-variant", "Log Retention" }
                                span { class: "text-xs font-mono font-bold", "30D" }
                            }
                            button { class: "w-full mt-4 py-2 text-xs font-bold uppercase tracking-widest bg-surface-container-highest rounded-lg hover:bg-surface-bright transition-colors",
                                "Export Logs"
                            }
                        }
                    }
                }
            }

        }
    }
}
