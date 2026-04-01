use crate::document::eval;
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    User {
        content: String,
    },
    AI {
        model_id: String,
        model_name: String,
        response: String,
        error: Option<String>,
        is_finished: bool,
    },
}

#[component]
pub fn ChatMessage(msg: Message) -> Element {
    match msg {
        Message::User { content } => rsx! {
            UserMessage { content }
        },
        Message::AI {
            model_name,
            response,
            error,
            is_finished,
            ..
        } => rsx! {
            AiMessage {
                model_name,
                response,
                error,
                is_finished,
            }
        },
    }
}

#[component]
pub fn UserMessage(content: String) -> Element {
    rsx! {
        div { class: "flex flex-col items-end gap-2 group",
            div { class: "max-w-[80%] bg-surface-container-highest px-6 py-4 rounded-2xl rounded-tr-none border border-outline-variant/20 shadow-lg",
                p { class: "text-on-surface font-body leading-relaxed whitespace-pre-wrap",
                    "{content}"
                }
            }
            span { class: "text-[10px] text-on-surface-variant font-label opacity-0 group-hover:opacity-100 transition-opacity",
                "Just now • Orchestrated Request"
            }
        }
    }
}

#[component]
pub fn AiMessage(
    model_name: String,
    response: String,
    error: Option<String>,
    is_finished: bool,
) -> Element {
    let mut copied = use_signal(|| false);

    rsx! {
        div { class: "bg-surface-container p-6 rounded-2xl border border-primary/20 relative overflow-hidden group",
            div { class: "absolute top-0 left-0 w-1 h-full bg-primary" }

            // Floating Copy Button
            button {
                class: "absolute top-3 right-3 flex items-center justify-center p-1.5 rounded-lg bg-surface-container-highest text-on-surface-variant hover:text-primary border border-outline-variant/20 shadow-sm opacity-0 group-hover:opacity-100 transition-all z-10",
                class: if copied() { "animate-pulse !text-primary !border-primary/40 bg-primary/10 opacity-100" },
                title: "Copy Response",
                onclick: move |_| {
                    eval(
                        &format!(
                            r#" (function() {{
                                        const text = {:?};
                                        try {{
                                            if (navigator.clipboard && navigator.clipboard.writeText) {{
                                                navigator.clipboard.writeText(text).catch(() => copyFallback(text));
                                            }} else {{
                                                copyFallback(text);
                                            }}
                                        }} catch (e) {{
                                            copyFallback(text);
                                        }}
                                        function copyFallback(t) {{
                                            const textArea = document.createElement("textarea");
                                            textArea.value = t;
                                            textArea.style.position = "fixed";
                                            textArea.style.left = "-9999px";
                                            textArea.style.top = "0";
                                            document.body.appendChild(textArea);
                                            textArea.focus();
                                            textArea.select();
                                            try {{
                                                document.execCommand('copy');
                                                console.log('Copied using fallback');
                                            }} catch (err) {{
                                                console.error('Copy failed completely', err);
                                            }}
                                            document.body.removeChild(textArea);
                                        }}
                                    }})(); "#,
                            response,
                        ),
                    );
                    copied.set(true);
                    spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                        copied.set(false);
                    });
                },
                if copied() {
                    span { class: "material-symbols-outlined text-[14px]", "check" }
                } else {
                    span { class: "material-symbols-outlined text-[14px]", "content_copy" }
                }
            }

            div { class: "flex items-center justify-between mb-6",
                div { class: "flex items-center gap-3",
                    span { class: "material-symbols-outlined text-primary", "bolt" }
                    span { class: "font-headline font-bold text-sm tracking-tight text-on-surface",
                        "{model_name}"
                    }
                    div { class: "h-1 w-1 rounded-full bg-outline-variant" }

                    if is_finished {
                        span { class: "text-[10px] text-on-surface-variant font-label uppercase",
                            "Reasoning Complete"
                        }
                    } else if response.is_empty() && error.is_none() {
                        span { class: "text-[10px] text-primary animate-pulse font-label uppercase",
                            "Loading Model..."
                        }
                    } else {
                        span { class: "text-[10px] text-primary animate-pulse font-label uppercase",
                            "Generating..."
                        }
                    }
                }
            }
            div { class: "space-y-4 font-body text-on-surface/90 leading-relaxed text-sm whitespace-pre-wrap pr-12",
                "{response}"
            }
            if let Some(err) = error {
                div { class: "mt-4 p-4 bg-error/10 border border-error/20 rounded-xl text-error text-sm font-mono whitespace-pre-wrap",
                    "[Execution Halted] {err}"
                }
            }
        }
    }
}
