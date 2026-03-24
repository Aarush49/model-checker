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
                        to: Route::BenchmarkingPage {},
                        class: "px-4 py-3 mx-2 flex items-center gap-3 transition-all duration-200 active:scale-[0.98] rounded-lg group text-[#dee5ff]/60 hover:text-[#dee5ff] hover:bg-[#141f38]",
                        active_class: "text-[#5D3FD3] bg-[#141f38] font-bold",
                        span {
                            class: "material-symbols-outlined",
                            "compare_arrows"
                        }
                        span {
                            class: "font-body font-medium",
                            "Benchmarking"
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
                        href: "#",
                        class: "text-[#dee5ff]/60 hover:text-[#dee5ff] px-6 py-2 flex items-center gap-3 hover:bg-[#141f38] transition-colors",
                        span {
                            class: "material-symbols-outlined text-sm",
                            "description"
                        }
                        span {
                            class: "text-sm font-label",
                            "Documentation"
                        }
                    }
                    a {
                        href: "#",
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
            // TopNavBar
            header {
                class: "fixed top-0 right-0 w-[calc(100%-16rem)] h-16 z-40 bg-[#060e20]/80 backdrop-blur-xl flex justify-between items-center px-8 shadow-xl shadow-[#060e20]/40",
                div {
                    class: "flex items-center gap-6",
                    div {
                        class: "flex items-center gap-2 bg-surface-container-highest/50 px-4 py-1.5 rounded-full border border-outline-variant/20",
                        span {
                            class: "material-symbols-outlined text-on-surface-variant text-sm",
                            "search"
                        }
                        input {
                            class: "bg-transparent border-none focus:ring-0 text-sm w-48 font-body",
                            placeholder: "Search Workspace Alpha...",
                            "type": "text"
                        }
                    }
                    nav {
                        class: "flex items-center gap-6",
                        a {
                            href: "#",
                            class: "text-[#5D3FD3] font-bold border-b-2 border-[#5D3FD3] pb-1 text-sm font-label",
                            "Models"
                        }
                        a {
                            href: "#",
                            class: "text-[#dee5ff]/70 hover:text-[#dee5ff] transition-colors text-sm font-label",
                            "Datasets"
                        }
                        a {
                            href: "#",
                            class: "text-[#dee5ff]/70 hover:text-[#dee5ff] transition-colors text-sm font-label",
                            "Logs"
                        }
                    }
                }
                div {
                    class: "flex items-center gap-4",
                    button {
                        class: "w-10 h-10 flex items-center justify-center text-[#dee5ff]/70 hover:text-[#5D3FD3] transition-all",
                        span {
                            class: "material-symbols-outlined",
                            "notifications"
                        }
                    }
                    button {
                        class: "w-10 h-10 flex items-center justify-center text-[#dee5ff]/70 hover:text-[#5D3FD3] transition-all",
                        span {
                            class: "material-symbols-outlined",
                            "dns"
                        }
                    }
                    div {
                        class: "w-8 h-8 rounded-full bg-surface-container-highest overflow-hidden border border-primary/30",
                        img {
                            alt: "User Profile",
                            src: "https://lh3.googleusercontent.com/aida-public/AB6AXuCTLrvgNJStoH4a614OGFDTHE-rKQD3PaRIyzgDp7aRLwW1fgqBoBZP6sfCvdYNtuv2tzqYKKIOghhiWOwqoVC4pkcD4TEgFSlf3D7FPjhL2EdU9Pp4o0VkKKi62NU0H8rmbYsIS4JhWrPf_3yT1yPKhXy82Eyd_RXvkv4bkMEKZTWGx_uSU9X2mqs_VaZk8lUEujTsSVG5RrwExevKJFiz1umwrPV-yiUK7erlF7jRPxiD79cUMRni32g61h7F-vUmfxx04D1pF34"
                        }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}
