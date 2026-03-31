use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Layout() -> Element {
    rsx! {
        div {
            class: "bg-background text-on-surface font-body selection:bg-secondary/30 h-screen overflow-hidden",
            // SideNavBar
            aside {
                class: "h-screen w-64 fixed left-0 top-0 flex flex-col bg-[#091328] py-6 z-50",
                div {
                    class: "px-6 mb-8 flex items-center gap-3",
                    div {
                        class: "w-8 h-8 rounded bg-gradient-to-br from-primary to-primary-dim flex items-center justify-center",
                        span {
                            class: "material-symbols-outlined text-on-primary text-sm",
                            style: "font-variation-settings: 'FILL' 1;",
                            "insights"
                        }
                    }
                    div {
                        h1 {
                            class: "text-xl font-bold tracking-tight text-[#dee5ff] font-['Space_Grotesk']",
                            "Lumina AI"
                        }
                        p {
                            class: "text-[10px] uppercase tracking-[0.2em] text-on-surface-variant font-medium",
                            "Synthetic Curator"
                        }
                    }
                }
                div {
                    class: "px-4 mb-8",
                    button {
                        class: "w-full py-3 px-4 bg-gradient-to-r from-primary to-primary-dim text-on-primary font-bold rounded-xl flex items-center justify-center gap-2 transition-all active:scale-[0.98]",
                        span {
                            class: "material-symbols-outlined text-sm",
                            "add"
                        }
                        span {
                            class: "font-label",
                            "New Session"
                        }
                    }
                }
                nav {
                    class: "flex-1 space-y-1",
                    Link {
                        to: Route::OrchestratorPage {},
                        class: "px-4 py-3 mx-2 flex items-center gap-3 transition-all duration-200 active:scale-[0.98] rounded-lg group text-[#dee5ff]/60 hover:text-[#dee5ff] hover:bg-[#141f38]",
                        active_class: "text-[#5D3FD3] bg-[#141f38] font-bold",
                        span {
                            class: "material-symbols-outlined",
                            "chat_bubble"
                        }
                        span {
                            class: "font-body font-medium",
                            "Orchestrator"
                        }
                    }

                    Link {
                        to: Route::NeuralEnginesPage {},
                        class: "px-4 py-3 mx-2 flex items-center gap-3 transition-all duration-200 active:scale-[0.98] rounded-lg group text-[#dee5ff]/60 hover:text-[#dee5ff] hover:bg-[#141f38]",
                        active_class: "text-[#5D3FD3] bg-[#141f38] font-bold",
                        span {
                            class: "material-symbols-outlined",
                            "settings_input_component"
                        }
                        span {
                            class: "font-body font-medium",
                            "Neural Engines"
                        }
                    }
                }
                div {
                    class: "mt-auto pt-6 border-t border-outline-variant/10 space-y-1",
                    a {
                        href: "https://github.com/neeleshpoli/model-checker",
                        target: "_blank",
                        class: "text-[#dee5ff]/60 hover:text-[#dee5ff] px-6 py-2 flex items-center gap-3 hover:bg-[#141f38] transition-colors",
                        span {
                            class: "material-symbols-outlined text-sm",
                            "help"
                        }
                        span {
                            class: "text-sm font-label",
                            "Support"
                        }
                    }
                }
            }

            Outlet::<Route> {}
        }
    }
}
