use std::sync::RwLock;

use anyhow::{Context, Result, anyhow};
use async_trait::async_trait;
use reqwest::{Client, header};
use serde::Deserialize;
use serde_json::json;

use crate::ai::{ModelProvider, ProviderStatus};

pub struct ChatGPT {
    client: Client,
    api_key: RwLock<String>,
    status: RwLock<ProviderStatus>,
    temperature: RwLock<f32>,
}

impl ChatGPT {
    pub async fn new(http_client: &Client) -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
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
impl ModelProvider for ChatGPT {
    fn id(&self) -> &'static str {
        "model.chatgpt"
    }

    fn name(&self) -> &'static str {
        "ChatGPT"
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

    async fn setup(&self) -> Result<()> {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            *self.api_key.write().unwrap() = api_key;
            *self.status.write().unwrap() = ProviderStatus::Ready;
        } else {
            *self.status.write().unwrap() = ProviderStatus::RequiresAuth;
        }
        Ok(())
    }

    async fn ask(&self, prompt: &String, tx: tokio::sync::mpsc::UnboundedSender<anyhow::Result<String>>) -> Result<()> {
        let url = "https://api.openai.com/v1/chat/completions";

        let payload = json!({
            "model": "gpt-4o-mini",
            "temperature": *self.temperature.read().unwrap(),
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let api_key = self.api_key.read().unwrap().clone();

        let response = self.client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("OpenAI API error: {}", error_text));
        }

        let response_data: ChatGPTResponse = response.json().await?;

        let answer = response_data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .context("ChatGPT API returned an empty or malformed response")?;

        let _ = tx.send(Ok(answer));

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct ChatGPTResponse {
    choices: Vec<ChatGPTChoice>,
}

#[derive(Deserialize, Debug)]
struct ChatGPTChoice {
    message: ChatGPTMessage,
}

#[derive(Deserialize, Debug)]
struct ChatGPTMessage {
    content: String,
}
