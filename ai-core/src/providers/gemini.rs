use std::sync::RwLock;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::ai::{ModelProvider, ProviderStatus};

pub struct Gemini {
    client: Client,
    api_key: RwLock<String>,
    status: RwLock<ProviderStatus>,
    temperature: RwLock<f32>,
}

impl Gemini {
    pub async fn new(http_client: &Client) -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        let status = if api_key.is_empty() {
            ProviderStatus::RequiresAuth
        } else {
            ProviderStatus::Ready
        };

        Self {
            client: http_client.clone(),
            api_key: RwLock::new(api_key),
            status: RwLock::new(status),
            temperature: RwLock::new(0.7),
        }
    }
}

#[async_trait]
impl ModelProvider for Gemini {
    fn id(&self) -> &'static str {
        "model.gemini"
    }
    fn name(&self) -> &'static str {
        "Gemini"
    }

    fn status(&self) -> ProviderStatus {
        return *self.status.read().unwrap();
    }

    fn temperature(&self) -> f32 {
        *self.temperature.read().unwrap()
    }

    fn set_temperature(&self, temperature: f32) {
        *self.temperature.write().unwrap() = temperature;
    }

    async fn setup(
        &self,
        _progress_tx: Option<tokio::sync::mpsc::UnboundedSender<(u64, u64)>>,
    ) -> Result<()> {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            *self.api_key.write().unwrap() = api_key;
            *self.status.write().unwrap() = ProviderStatus::Ready;
        } else {
            *self.status.write().unwrap() = ProviderStatus::RequiresAuth;
        }
        Ok(())
    }

    async fn ask(
        &self,
        prompt: &String,
        tx: tokio::sync::mpsc::UnboundedSender<anyhow::Result<String>>,
    ) -> Result<()> {
        let api_key = self.api_key.read().unwrap().clone();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1/models/gemini-2.5-flash:generateContent?key={}",
            api_key
        );

        let payload = json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": prompt}]
            }],
            "generationConfig": {
                "temperature": *self.temperature.read().unwrap(),
            }
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
        }

        let response_data: serde_json::Value = response.json().await?;

        let answer = response_data
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|c| c.first())
            .and_then(|c| c.get("content"))
            .and_then(|c| c.get("parts"))
            .and_then(|p| p.as_array())
            .and_then(|p| p.first())
            .and_then(|p| p.get("text"))
            .and_then(|t| t.as_str())
            .map(|s| s.to_string());

        let final_text = match answer {
            Some(t) => t,
            None => {
                return Err(anyhow::anyhow!(
                    "Gemini API structure unexpected: {}",
                    response_data
                ));
            }
        };

        let _ = tx.send(Ok(final_text));

        Ok(())
    }
}
