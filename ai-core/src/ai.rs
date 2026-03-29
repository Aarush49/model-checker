use anyhow::{Ok, Result, anyhow};
use async_trait::async_trait;

use std::collections::HashSet;

use crate::providers::{chatgpt::ChatGPT, gemini::Gemini, phi::Phi};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeMode {
    Local,
    Cloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    /// Ready for messages to be sent to
    Ready,
    /// Cloud model needs user to complete OAuth
    RequiresAuth,
    /// Local model needs to be downloaded
    RequiresInstallation,
}

#[async_trait]
pub trait ModelProvider {
    /// Return the model localization identifier
    fn id(&self) -> &'static str;
    /// Return the english name of the model. This should only be use until the app is fully built.
    fn name(&self) -> &'static str;

    /// What modes the AI model supports. If not specified by model, defaults to Cloud only.
    fn supported_modes(&self) -> Vec<ComputeMode> {
        vec![ComputeMode::Cloud]
    }

    /// Get the current execution mode. If not specificed by model, defaults to Cloud.
    fn current_mode(&self) -> ComputeMode {
        ComputeMode::Cloud
    }
    /// Set the execution mode of the model. If the model only supports one mode, it will not change anything.
    fn set_mode(&self) {}

    /// Indicate if the model is ready to be used, or if the user needs to login or install the model
    fn status(&self) -> ProviderStatus;

    async fn setup(&self) -> Result<()>;

    /// Ask the model something
    async fn ask(&self, prompt: &String) -> Result<String>;
}

pub struct Models {
    pub models: Vec<Box<dyn ModelProvider>>,
}

impl Models {
    /// Initialize Models to be empty
    pub fn new() -> Self {
        Self { models: Vec::new() }
    }

    pub async fn load_models() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent("ModelChecker/1.0 (Rust reqwest)")
            .build()?;

        let mut models: Vec<Box<dyn ModelProvider>> = vec![
            Box::new(Phi::new(&http_client).await),
            Box::new(ChatGPT::new(&http_client).await),
        ];

        if let std::result::Result::Ok(gemini) = Gemini::new(&http_client).await {
            models.push(Box::new(gemini));
        }

        Ok(Self { models })
    }

    pub async fn setup(&self, index: usize) -> Result<()> {
        self.models
            .get(index)
            .ok_or_else(|| anyhow!("Model not found at index {}", index))?
            .setup()
            .await?;

        Ok(())
    }

    pub async fn ask(&self, prompt: String, selected_model_ids: HashSet<String>) -> Result<Vec<(String, String)>> {
        let mut responses = vec![];
        for model in self
            .models
            .iter()
            .filter(|model| model.status() == ProviderStatus::Ready && selected_model_ids.contains(model.id()))
        {
            responses.push((model.id().to_string(), model.ask(&prompt).await?));
        }

        Ok(responses)
    }
}
