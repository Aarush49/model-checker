use dioxus::prelude::*;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag};

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
    let response_for_copy = response.clone();

    rsx! {
        div { class: "bg-surface-container p-6 rounded-2xl border border-primary/20 relative overflow-hidden group",
            div { class: "absolute top-0 left-0 w-1 h-full bg-primary" }

            // Floating Copy Button
            button {
                class: "absolute top-3 right-3 flex items-center justify-center p-1.5 rounded-lg bg-surface-container-highest text-on-surface-variant hover:text-primary border border-outline-variant/20 shadow-sm opacity-0 group-hover:opacity-100 transition-all z-10",
                class: if copied() { "animate-pulse !text-primary !border-primary/40 bg-primary/10 opacity-100" },
                title: "Copy Response",
                onclick: move |_| {
                    document::eval(
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
                            response_for_copy,
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
            div { class: "font-body text-on-surface/90 leading-relaxed text-sm pr-12",
                Markdown { content: response }
            }
            if let Some(err) = error {
                div { class: "mt-4 p-4 bg-error/10 border border-error/20 rounded-xl text-error text-sm font-mono whitespace-pre-wrap",
                    "[Execution Halted] {err}"
                }
            }
        }
    }
}

#[component]
pub fn Markdown(content: String) -> Element {
    let nodes = parse_markdown(&content);

    rsx! {
        div { class: "markdown-body text-on-surface/90 leading-relaxed space-y-1",
            for node in nodes {
                RenderNode { node }
            }
        }
    }
}

#[component]
fn RenderNode(node: MarkdownNode) -> Element {
    match node {
        MarkdownNode::Text(text) => rsx! { "{text}" },
        MarkdownNode::InlineCode(code) => rsx! {
            code { class: "bg-surface-container-highest px-1.5 py-0.5 rounded text-primary font-mono text-xs font-semibold",
                "{code}"
            }
        },
        MarkdownNode::Break => rsx! { br {} },
        MarkdownNode::Heading(level, children) => {
            let class = match level {
                pulldown_cmark::HeadingLevel::H1 => {
                    "text-2xl font-bold mt-8 mb-4 font-headline text-on-surface border-b border-outline-variant/30 pb-2"
                }
                pulldown_cmark::HeadingLevel::H2 => {
                    "text-xl font-bold mt-6 mb-3 font-headline text-on-surface border-b border-outline-variant/20 pb-1"
                }
                pulldown_cmark::HeadingLevel::H3 => {
                    "text-lg font-bold mt-5 mb-2 font-headline text-on-surface"
                }
                _ => "text-base font-bold mt-4 mb-2 font-headline text-on-surface/80",
            };
            rsx! { div { class,
                for child in children {
                    RenderNode { node: child }
                }
            } }
        }
        MarkdownNode::Paragraph(children) => rsx! {
            p { class: "mb-4 last:mb-0",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::Strong(children) => rsx! {
            strong { class: "font-bold text-on-surface",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::Emphasis(children) => rsx! {
            em { class: "italic",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::Strikethrough(children) => rsx! {
            del { class: "line-through opacity-70",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::Link(url, title, children) => rsx! {
            a {
                href: "{url}",
                title: "{title}",
                class: "text-primary hover:underline underline-offset-4 decoration-primary/30 transition-all",
                target: "_blank",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::CodeBlock(lang, children) => rsx! {
            pre { class: "bg-surface-container-highest/50 p-4 rounded-xl font-mono text-xs overflow-x-auto my-6 border border-outline-variant/20 shadow-inner group/code relative",
                if !lang.is_empty() {
                    span { class: "absolute top-2 right-3 text-[10px] font-mono text-on-surface-variant/50 uppercase tracking-wider",
                        "{lang}"
                    }
                }
                code { class: "block leading-relaxed",
                    for child in children {
                        RenderNode { node: child }
                    }
                }
            }
        },
        MarkdownNode::List(start, children) => {
            if start.is_some() {
                rsx! { ol { class: "list-decimal ml-6 space-y-2 my-4",
                    for child in children {
                        RenderNode { node: child }
                    }
                } }
            } else {
                rsx! { ul { class: "list-disc ml-6 space-y-2 my-4",
                    for child in children {
                        RenderNode { node: child }
                    }
                } }
            }
        }
        MarkdownNode::Item(children) => rsx! {
            li { class: "pl-1",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::BlockQuote(children) => rsx! {
            blockquote { class: "border-l-4 border-primary/40 pl-6 py-2 italic bg-primary/5 rounded-r-xl my-6 text-on-surface/80",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::Table(children) => rsx! {
            div { class: "overflow-x-auto my-6 rounded-xl border border-outline-variant/20",
                table { class: "w-full text-sm border-collapse",
                    for child in children {
                        RenderNode { node: child }
                    }
                }
            }
        },
        MarkdownNode::TableHead(children) => rsx! {
            thead { class: "bg-surface-container-highest/30",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::TableRow(children) => rsx! {
            tr { class: "border-b border-outline-variant/10 last:border-0 hover:bg-surface-container-highest/20 transition-colors",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
        MarkdownNode::TableCell(children) => rsx! {
            td { class: "px-4 py-3",
                for child in children {
                    RenderNode { node: child }
                }
            }
        },
    }
}

fn parse_markdown(content: &str) -> Vec<MarkdownNode> {
    let parser = Parser::new(content);
    let mut root = Vec::new();
    let mut stack = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => {
                stack.push((tag, Vec::new()));
            }
            Event::End(_) => {
                if let Some((tag, children)) = stack.pop() {
                    let node = convert_tag(tag, children);
                    if let Some((_, parent_children)) = stack.last_mut() {
                        parent_children.push(node);
                    } else {
                        root.push(node);
                    }
                }
            }
            Event::Text(text) => {
                let node = MarkdownNode::Text(text.to_string());
                if let Some((_, children)) = stack.last_mut() {
                    children.push(node);
                } else {
                    root.push(node);
                }
            }
            Event::Code(code) => {
                let node = MarkdownNode::InlineCode(code.to_string());
                if let Some((_, children)) = stack.last_mut() {
                    children.push(node);
                } else {
                    root.push(node);
                }
            }
            Event::SoftBreak => {
                let node = MarkdownNode::Text(" ".to_string());
                if let Some((_, children)) = stack.last_mut() {
                    children.push(node);
                } else {
                    root.push(node);
                }
            }
            Event::HardBreak => {
                let node = MarkdownNode::Break;
                if let Some((_, children)) = stack.last_mut() {
                    children.push(node);
                } else {
                    root.push(node);
                }
            }
            _ => {}
        }
    }

    while let Some((tag, children)) = stack.pop() {
        let node = convert_tag(tag, children);
        if let Some((_, parent_children)) = stack.last_mut() {
            parent_children.push(node);
        } else {
            root.push(node);
        }
    }

    root
}

fn convert_tag(tag: Tag<'_>, children: Vec<MarkdownNode>) -> MarkdownNode {
    match tag {
        Tag::Heading { level, .. } => MarkdownNode::Heading(level, children),
        Tag::Paragraph => MarkdownNode::Paragraph(children),
        Tag::Strong => MarkdownNode::Strong(children),
        Tag::Emphasis => MarkdownNode::Emphasis(children),
        Tag::Strikethrough => MarkdownNode::Strikethrough(children),
        Tag::Link {
            dest_url, title, ..
        } => MarkdownNode::Link(dest_url.to_string(), title.to_string(), children),
        Tag::CodeBlock(kind) => {
            let lang = match kind {
                CodeBlockKind::Fenced(l) => l.to_string(),
                _ => "".to_string(),
            };
            MarkdownNode::CodeBlock(lang, children)
        }
        Tag::List(start) => MarkdownNode::List(start, children),
        Tag::Item => MarkdownNode::Item(children),
        Tag::BlockQuote(_) => MarkdownNode::BlockQuote(children),
        Tag::Table(_) => MarkdownNode::Table(children),
        Tag::TableHead => MarkdownNode::TableHead(children),
        Tag::TableRow => MarkdownNode::TableRow(children),
        Tag::TableCell => MarkdownNode::TableCell(children),
        _ => MarkdownNode::Paragraph(children),
    }
}

#[derive(Clone, PartialEq)]
enum MarkdownNode {
    Text(String),
    InlineCode(String),
    Break,
    Heading(pulldown_cmark::HeadingLevel, Vec<MarkdownNode>),
    Paragraph(Vec<MarkdownNode>),
    Strong(Vec<MarkdownNode>),
    Emphasis(Vec<MarkdownNode>),
    Strikethrough(Vec<MarkdownNode>),
    Link(String, String, Vec<MarkdownNode>),
    CodeBlock(String, Vec<MarkdownNode>),
    List(Option<u64>, Vec<MarkdownNode>),
    Item(Vec<MarkdownNode>),
    BlockQuote(Vec<MarkdownNode>),
    Table(Vec<MarkdownNode>),
    TableHead(Vec<MarkdownNode>),
    TableRow(Vec<MarkdownNode>),
    TableCell(Vec<MarkdownNode>),
}
