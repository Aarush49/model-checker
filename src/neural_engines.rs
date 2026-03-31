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



        }
    }
}
