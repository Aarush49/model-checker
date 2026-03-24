mod benchmarking;
mod layout;
mod neural_engines;
mod orchestrator;

use ai_core::ai::Models;
use benchmarking::BenchmarkingPage;
use dioxus::prelude::*;
use layout::Layout;
use neural_engines::NeuralEnginesPage;
use orchestrator::OrchestratorPage;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    OrchestratorPage {},
    #[route("/benchmarking")]
    BenchmarkingPage {},
    #[route("/settings")]
    NeuralEnginesPage {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Debug)]
pub enum Message {
    User { content: String },
    AI { responses: Vec<(String, String)> },
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut models_registry = use_context_provider(|| Signal::new(Models::new()));

    use_effect(move || {
        spawn(async move {
            let loaded_models = Models::load_models().await.unwrap();
            models_registry.set(loaded_models);
        });
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com", crossorigin: "true" }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Manrope:wght@300;400;500;600;700&display=swap",
            rel: "stylesheet"
        }
        document::Link {
            href: "https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap",
            rel: "stylesheet"
        }
        document::Title { "Lumina AI | Synthetic Curator" }
        Router::<Route> {}
    }
}