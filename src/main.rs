mod layout;
mod message;
mod neural_engines;
mod chat;

use ai_core::ai::Models;
use message::Message;
use std::collections::HashSet;

use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use layout::Layout;
use neural_engines::NeuralEnginesPage;
use chat::ChatPage;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    ChatPage {},

    #[route("/settings")]
    NeuralEnginesPage {},
}

const LOGO: Asset = asset!("/assets/logo.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dotenvy::dotenv().ok();

    #[allow(unused_mut)]
    let mut window =
        WindowBuilder::new().with_min_inner_size(dioxus::desktop::LogicalSize::new(1000.0, 700.0));

    #[cfg(target_os = "macos")]
    {
        window = window
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);
    }

    #[cfg(not(target_os = "macos"))]
    {
        window = window.with_decorations(false);
    }

    let cfg = Config::new().with_window(window).with_menu(None);

    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    let mut models_registry = use_context_provider(|| Signal::new(Models::new()));
    let _ = use_context_provider(|| Signal::new(Vec::<Message>::new()));
    let _ = use_context_provider(|| Signal::new(Option::<HashSet<String>>::None));
    // Chat history: each entry is (session_name, messages)
    let _ = use_context_provider(|| Signal::new(Vec::<(String, Vec<Message>)>::new()));
    // Index of the currently-loaded history session (None = fresh/new session)
    let _ = use_context_provider(|| Signal::new(Option::<usize>::None));

    use_effect(move || {
        spawn(async move {
            let loaded_models = Models::load_models().await.unwrap();
            models_registry.set(loaded_models);
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "true",
        }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Manrope:wght@300;400;500;600;700&display=swap",
            rel: "stylesheet",
        }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap",
            rel: "stylesheet",
        }
        document::Title { "ModelCheck | AI Model Verifier" }
        Router::<Route> {}
    }
}
