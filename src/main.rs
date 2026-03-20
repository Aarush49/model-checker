use ai_core::ai::{Models, ProviderStatus};
use ai_core::providers::gemini::Gemini;
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Header)]
    #[route("/")]
    ChatPage {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Debug)]
struct Message {
    sender: String,
    content: String,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Temporarily set models to be empty
    let mut models_registry = use_context_provider(|| Signal::new(Models::new()));

    // Actually load the models here
    use_effect(move || {
        spawn(async move {
            let loaded_models = Models::load_models().await.unwrap();
            models_registry.set(loaded_models);
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Title { "AI Chat Interface" }
        div { class: "flex flex-col h-screen", Router::<Route> {} }
    }
}

#[component]
fn Header() -> Element {
    let models_registry = use_context::<Signal<Models>>();

    rsx! {
        // 1. Made the header a flex container to push title left, buttons right
        header { class: "flex justify-between items-center px-6 py-4 border-b border-gray-200 dark:border-gray-800",

            // Adjusted the title to fit nicely in the top bar
            h1 { class: "text-2xl font-bold tracking-tight", "Model Checker" }

            // 2. Added a container for your buttons with some horizontal spacing (space-x-3)
            div { class: "flex items-center space-x-3",

                for (index , model) in models_registry.read().models.iter().enumerate() {
                    button {
                        key: "{index}",
                        disabled: match model.status() {
                            ProviderStatus::Ready => true,
                            _ => false,
                        },

                        // 3. The Magic! Dynamic Tailwind classes based on the model's status
                        class: match model.status() {
                            ProviderStatus::Ready => { // Active states: Nice blue/purple, hover effects, subtle shadow
                                "px-4 py-2 rounded-lg font-medium text-white bg-green-600/70 cursor-not-allowed border border-transparent"
                            }
                            ProviderStatus::RequiresAuth => {
                                "px-4 py-2 rounded-lg font-medium text-white bg-blue-600 hover:bg-blue-700 shadow-sm transition-all focus:ring-2 focus:ring-blue-500 focus:outline-none"
                            }
                            ProviderStatus::RequiresInstallation => {
                                "px-4 py-2 rounded-lg font-medium text-white bg-purple-600 hover:bg-purple-700 shadow-sm transition-all focus:ring-2 focus:ring-purple-500 focus:outline-none"
                            }
                        },

                        onclick: move |_| {
                            spawn(async move {
                                models_registry.read().setup(index).await.unwrap();
                            });
                        },

                        match model.status() {
                            ProviderStatus::Ready => format!("{} (Ready)", model.name()),
                            ProviderStatus::RequiresAuth => format!("Login to {}", model.name()),
                            ProviderStatus::RequiresInstallation => format!("Install {}", model.name()),
                        }
                    }
                }
            }
        }
        main { Outlet::<Route> {} }
    }
}

#[component]
fn ChatPage() -> Element {
    let mut messages = use_signal(Vec::<Message>::new);
    let mut prompt = use_signal(String::new);
    let models_registry = use_context::<Signal<Models>>();

    // 1. Remove the `|_|` argument so it can be called from anywhere
    let mut send_message = move || {
        // 2. CAPTURE THE TEXT NOW, before the async boundary
        let current_prompt = prompt().clone(); 

        if !current_prompt.is_empty() {
            let user_msg = Message {
                sender: "User".to_string(),
                content: current_prompt.clone(),
            };
            messages.write().push(user_msg);

            // 3. Clear the UI immediately so it feels snappy
            prompt.set(String::new());

            // 4. Move the ALREADY CLONED text into the background task
            spawn(async move {
                let responses = models_registry
                    .read()
                    .ask(current_prompt) // Use the captured string here!
                    .await
                    .unwrap();

                messages.write().push(Message {
                    sender: "AI".to_string(),
                    content: responses.iter().map(|model| model.1.clone()).collect(),
                });
            });
        }
    };

    rsx! {
        div {
            id: "chat-page",
            class: "flex flex-col h-full bg-white dark:bg-gray-900 max-w-4xl mx-auto w-full",

            // Messages area
            div { class: "flex-1 overflow-y-auto p-4 space-y-4",
                for msg in messages() {
                    div { class: if msg.sender == "User" { "flex justify-end" } else { "flex justify-start" },
                        div { class: if msg.sender == "User" { "bg-blue-500 text-white rounded-lg px-4 py-2 max-w-xs lg:max-w-md" } else { "bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-white rounded-lg px-4 py-2 max-w-xs lg:max-w-md shadow" },
                            "{msg.content}"
                        }
                    }
                }
            }

            // Input area
            div { class: "shrink-0 p-4 border-t border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-900",
                div { class: "flex space-x-2",
                    textarea {
                        class: "flex-1 border border-gray-300 dark:border-gray-600 rounded-lg p-2 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-white dark:bg-gray-800 text-black dark:text-white resize-none",
                        rows: "2",
                        placeholder: "Type your message...",
                        value: "{prompt}",
                        oninput: move |event| prompt.set(event.value()),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter && !event.modifiers().contains(Modifiers::SHIFT) {
                                event.prevent_default();
                                // 5. Now you can safely call this from the Enter key!
                                send_message();
                            }
                        },
                    }
                    button {
                        class: "bg-blue-500 text-white px-4 py-2 rounded-lg hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500",
                        // 6. Wrap in a dummy closure to ignore the MouseData event
                        onclick: move |_| send_message(),
                        "Send"
                    }
                }
            }
        }
    }
}