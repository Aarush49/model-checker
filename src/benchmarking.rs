use dioxus::prelude::*;

#[component]
pub fn BenchmarkingPage() -> Element {
    rsx! {
        main { class: "ml-64 pt-24 pb-12 px-12 min-h-screen bg-transparent overflow-y-auto",
            // Hero Editorial Header
            section { class: "mb-12 flex justify-between items-end",
                div { class: "max-w-2xl",
                    span { class: "font-label text-xs uppercase tracking-[0.3em] text-secondary font-bold mb-4 block",
                        "Engine Comparison Matrix"
                    }
                    h2 { class: "font-headline text-5xl font-bold tracking-tight text-on-background mb-4 leading-none",
                        "Benchmarking Analysis"
                    }
                    p { class: "text-on-surface-variant text-lg font-light leading-relaxed",
                        "Evaluating synthetic reasoning across "
                        span { class: "text-on-background font-medium", "Neural Engines" }
                        ". Identifying latent semantic drift and architectural efficiency in real-time."
                    }
                }
                div { class: "flex gap-4",
                    button { class: "px-6 py-3 rounded-lg bg-surface-container-highest text-on-surface hover:bg-surface-bright transition-all flex items-center gap-2 font-medium",
                        span { class: "material-symbols-outlined", "filter_list" }
                        "Filters"
                    }
                    button { class: "px-6 py-3 rounded-lg bg-gradient-to-r from-primary to-primary-dim text-on-primary-fixed font-bold flex items-center gap-2 shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all",
                        span { class: "material-symbols-outlined", "refresh" }
                        "Re-run Prompt"
                    }
                }
            }

            // Prompt Overview
            section { class: "mb-12",
                div { class: "p-8 rounded-xl bg-surface-container-low border border-outline-variant/5",
                    div { class: "flex items-center gap-3 mb-4",
                        span { class: "material-symbols-outlined text-secondary", style: "font-variation-settings: 'FILL' 1;", "terminal" }
                        span { class: "font-label text-xs font-bold uppercase tracking-widest text-on-surface-variant",
                            "Active Prompt Context"
                        }
                    }
                    p { class: "font-headline text-2xl text-on-surface italic opacity-90",
                        "\"Synthesize the socio-economic implications of decentralized compute networks on late-stage manufacturing paradigms, focusing on the intersection of AI-governance and carbon neutrality.\""
                    }
                }
            }

            // Comparison Grid
            div { class: "grid grid-cols-3 gap-8 mb-12",
                // Engine 1
                div { class: "flex flex-col gap-6",
                    div { class: "p-6 rounded-xl bg-surface-container relative overflow-hidden group border border-primary/20",
                        div { class: "absolute top-0 left-0 w-1 h-full bg-primary/40" }
                        div { class: "flex justify-between items-start mb-6",
                            div {
                                h3 { class: "font-headline text-xl font-bold text-on-background", "Nebula-9" }
                                p { class: "text-xs text-on-surface-variant uppercase tracking-tighter", "Large Language Cluster" }
                            }
                            div { class: "px-3 py-1 bg-primary/10 rounded-full border border-primary/20 text-primary text-[10px] font-bold uppercase tracking-widest flex items-center gap-1 shadow-[0_0_20px_rgba(0,227,253,0.15)]",
                                span { class: "material-symbols-outlined text-[12px]", style: "font-variation-settings: 'FILL' 1;", "star" }
                                "Best Tone Match"
                            }
                        }
                        div { class: "space-y-4 text-on-surface-variant text-sm leading-relaxed",
                            p { "Decentralized compute shifts the gravity of manufacturing from centralized hubs to edge-optimized nodes. This transition mandates a recalibration of carbon credit distribution, where AI acts as the primary orchestrator of sustainable output..." }
                            p { "The convergence of governance and green-energy protocols suggests a 22% reduction in logistical overhead when neural steering is implemented at the factory level." }
                        }
                        div { class: "mt-8 grid grid-cols-2 gap-4 pt-6 border-t border-outline-variant/10",
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Inference Time" }
                                p { class: "text-lg font-headline font-bold text-on-background", "1.4s" }
                            }
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Confidence" }
                                p { class: "text-lg font-headline font-bold text-on-background", "98.2%" }
                            }
                        }
                    }
                }
                // Engine 2
                div { class: "flex flex-col gap-6",
                    div { class: "p-6 rounded-xl bg-surface-container relative overflow-hidden group border border-secondary/20",
                        div { class: "absolute top-0 left-0 w-1 h-full bg-secondary/40" }
                        div { class: "flex justify-between items-start mb-6",
                            div {
                                h3 { class: "font-headline text-xl font-bold text-on-background", "Zenith-X" }
                                p { class: "text-xs text-on-surface-variant uppercase tracking-tighter", "Multi-Modal Logic Hub" }
                            }
                        }
                        div { class: "space-y-4 text-on-surface-variant text-sm leading-relaxed",
                            p { "The manufacturing paradigm is undergoing a 'silent' revolution. By decoupling physical hardware from compute constraints, decentralized networks allow for localized production that mirrors natural ecosystem cycles." }
                            p { "Zenith-X identifies three key risk factors: Latency in decision-making, regulatory fragmentation, and the energy-intensity of distributed ledger validation for supply chains." }
                        }
                        div { class: "mt-8 grid grid-cols-2 gap-4 pt-6 border-t border-outline-variant/10",
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Inference Time" }
                                p { class: "text-lg font-headline font-bold text-on-background", "2.1s" }
                            }
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Confidence" }
                                p { class: "text-lg font-headline font-bold text-on-background", "94.5%" }
                            }
                        }
                    }
                }
                // Engine 3
                div { class: "flex flex-col gap-6",
                    div { class: "p-6 rounded-xl bg-surface-container relative overflow-hidden group border border-tertiary/20",
                        div { class: "absolute top-0 left-0 w-1 h-full bg-tertiary/40" }
                        div { class: "flex justify-between items-start mb-6",
                            div {
                                h3 { class: "font-headline text-xl font-bold text-on-background", "Flux Core" }
                                p { class: "text-xs text-on-surface-variant uppercase tracking-tighter", "Sub-Latent Reasoning Engine" }
                            }
                            div { class: "px-3 py-1 bg-secondary/10 rounded-full border border-secondary/20 text-secondary text-[10px] font-bold uppercase tracking-widest flex items-center gap-1",
                                span { class: "material-symbols-outlined text-[12px]", style: "font-variation-settings: 'FILL' 1;", "bolt" }
                                "Efficiency Lead"
                            }
                        }
                        div { class: "space-y-4 text-on-surface-variant text-sm leading-relaxed",
                            p { "Decentralization = Efficiency. Compute networks will automate carbon neutrality by pricing externalities in real-time. Governance becomes code. Late-stage manufacturing is no longer a physical constraint but a data-routing problem." }
                            p { "Key Outcome: Neutrality is achieved through 0.4ms micro-adjustments in factory thermal outputs directed by the edge network." }
                        }
                        div { class: "mt-8 grid grid-cols-2 gap-4 pt-6 border-t border-outline-variant/10",
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Inference Time" }
                                p { class: "text-lg font-headline font-bold text-secondary", "0.6s" }
                            }
                            div {
                                p { class: "text-[10px] uppercase text-on-surface-variant/60 font-bold tracking-widest", "Confidence" }
                                p { class: "text-lg font-headline font-bold text-on-background", "89.1%" }
                            }
                        }
                    }
                }
            }

            // Semantic Drift Chart Section
            section { class: "mt-16",
                div { class: "flex items-center justify-between mb-8",
                    div {
                        h3 { class: "font-headline text-2xl font-bold text-on-background", "Comparative Semantic Drift" }
                        p { class: "text-on-surface-variant text-sm", "Quantifying the deviation between synthetic responses and source intent." }
                    }
                    div { class: "flex items-center gap-6",
                        div { class: "flex items-center gap-2",
                            div { class: "w-3 h-3 rounded-full bg-primary" }
                            span { class: "text-xs font-label uppercase font-bold text-on-surface-variant/60", "Nebula" }
                        }
                        div { class: "flex items-center gap-2",
                            div { class: "w-3 h-3 rounded-full bg-secondary" }
                            span { class: "text-xs font-label uppercase font-bold text-on-surface-variant/60", "Zenith" }
                        }
                        div { class: "flex items-center gap-2",
                            div { class: "w-3 h-3 rounded-full bg-tertiary" }
                            span { class: "text-xs font-label uppercase font-bold text-on-surface-variant/60", "Flux" }
                        }
                    }
                }
                div { class: "glass-panel bg-surface-container-highest/60 backdrop-blur-xl rounded-2xl p-8 border border-outline-variant/10",
                    div { class: "flex items-end justify-between h-64 gap-8",
                        // Iteration Columns
                        div { class: "flex-1 flex flex-col justify-end gap-1 group relative",
                            div { class: "flex items-end gap-2 h-full",
                                div { class: "w-full bg-primary/20 hover:bg-primary/40 transition-all rounded-t-lg h-[40%]" }
                                div { class: "w-full bg-secondary/20 hover:bg-secondary/40 transition-all rounded-t-lg h-[65%]" }
                                div { class: "w-full bg-tertiary/20 hover:bg-tertiary/40 transition-all rounded-t-lg h-[30%]" }
                            }
                            span { class: "text-[10px] text-center uppercase tracking-widest font-bold text-on-surface-variant mt-4", "Structural" }
                        }
                        div { class: "flex-1 flex flex-col justify-end gap-1 group relative",
                            div { class: "flex items-end gap-2 h-full",
                                div { class: "w-full bg-primary/20 hover:bg-primary/40 transition-all rounded-t-lg h-[80%]" }
                                div { class: "w-full bg-secondary/20 hover:bg-secondary/40 transition-all rounded-t-lg h-[50%]" }
                                div { class: "w-full bg-tertiary/20 hover:bg-tertiary/40 transition-all rounded-t-lg h-[90%]" }
                            }
                            span { class: "text-[10px] text-center uppercase tracking-widest font-bold text-on-surface-variant mt-4", "Technical" }
                        }
                        div { class: "flex-1 flex flex-col justify-end gap-1 group relative",
                            div { class: "flex items-end gap-2 h-full",
                                div { class: "w-full bg-primary/20 hover:bg-primary/40 transition-all rounded-t-lg h-[30%]" }
                                div { class: "w-full bg-secondary/20 hover:bg-secondary/40 transition-all rounded-t-lg h-[45%]" }
                                div { class: "w-full bg-tertiary/20 hover:bg-tertiary/40 transition-all rounded-t-lg h-[25%]" }
                            }
                            span { class: "text-[10px] text-center uppercase tracking-widest font-bold text-on-surface-variant mt-4", "Philosophical" }
                        }
                        div { class: "flex-1 flex flex-col justify-end gap-1 group relative",
                            div { class: "flex items-end gap-2 h-full",
                                div { class: "w-full bg-primary/20 hover:bg-primary/40 transition-all rounded-t-lg h-[60%]" }
                                div { class: "w-full bg-secondary/20 hover:bg-secondary/40 transition-all rounded-t-lg h-[35%]" }
                                div { class: "w-full bg-tertiary/20 hover:bg-tertiary/40 transition-all rounded-t-lg h-[55%]" }
                            }
                            span { class: "text-[10px] text-center uppercase tracking-widest font-bold text-on-surface-variant mt-4", "Ethical" }
                        }
                        div { class: "flex-1 flex flex-col justify-end gap-1 group relative",
                            div { class: "flex items-end gap-2 h-full",
                                div { class: "w-full bg-primary/20 hover:bg-primary/40 transition-all rounded-t-lg h-[45%]" }
                                div { class: "w-full bg-secondary/20 hover:bg-secondary/40 transition-all rounded-t-lg h-[75%]" }
                                div { class: "w-full bg-tertiary/20 hover:bg-tertiary/40 transition-all rounded-t-lg h-[40%]" }
                            }
                            span { class: "text-[10px] text-center uppercase tracking-widest font-bold text-on-surface-variant mt-4", "Syntactic" }
                        }
                    }
                }
            }

            // Final Summary Stats
            section { class: "mt-16 grid grid-cols-4 gap-6",
                div { class: "col-span-1 p-6 rounded-2xl bg-surface-container-low flex flex-col justify-center",
                    span { class: "text-[10px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1", "Avg Response Complexity" }
                    p { class: "text-4xl font-headline font-bold text-on-background",
                        "84"
                        span { class: "text-primary", ".2" }
                    }
                    div { class: "mt-4 flex items-center gap-2 text-secondary text-xs font-bold",
                        span { class: "material-symbols-outlined text-sm", "trending_up" }
                        "+12% from baseline"
                    }
                }
                div { class: "col-span-1 p-6 rounded-2xl bg-surface-container-low flex flex-col justify-center border-l border-primary/20",
                    span { class: "text-[10px] font-bold text-on-surface-variant/60 uppercase tracking-widest mb-1", "Token Efficiency" }
                    p { class: "text-4xl font-headline font-bold text-on-background", "91.4%" }
                    p { class: "text-xs text-on-surface-variant mt-2", "Optimal compression" }
                }
                div { class: "col-span-2 p-6 rounded-2xl bg-surface-container-highest relative overflow-hidden flex items-center gap-8",
                    div { class: "z-10",
                        h4 { class: "font-headline text-lg font-bold mb-2", "Architectural Verdict" }
                        p { class: "text-sm text-on-surface-variant", "Nebula-9 consistently outperformed peers in narrative coherence, though Flux Core maintained a significant lead in computational velocity." }
                    }
                    div { class: "w-32 h-32 rounded-full border-4 border-primary/20 flex items-center justify-center shrink-0",
                        span { class: "material-symbols-outlined text-4xl text-primary", style: "font-variation-settings: 'FILL' 1;", "analytics" }
                    }
                    div { class: "absolute top-0 right-0 w-32 h-32 bg-primary/10 rounded-full blur-3xl -mr-16 -mt-16" }
                }
            }
        }
    }
}
