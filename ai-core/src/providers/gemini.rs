use std::sync::RwLock;

use anyhow::{Context, Ok, Result};

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{
    ai::{ModelProvider, ProviderStatus},
    providers::auth::{AuthStatus, OAuth, OAuthConfig},
};

pub struct Gemini {
    oauth: OAuth,
    status: RwLock<ProviderStatus>,
}

impl Gemini {
    pub async fn new(http_client: &Client) -> Result<Self> {
        let gemini_config = OAuthConfig {
            provider_name: "google".to_string(),
            client_id: env!("GOOGLE_CLIENT_ID").to_string(),
            client_secret: Some(env!("GOOGLE_CLIENT_SECRET").to_string()),
            auth_uri: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            scopes: vec![
                "https://www.googleapis.com/auth/cloud-platform".to_string(),
                "https://www.googleapis.com/auth/generative-language.retriever".to_string(),
                "https://www.googleapis.com/auth/generative-language.peruserquota".to_string(),
            ],
            // These parameters are required to get a referesh token
            extra_auth_params: vec![
                ("access_type".to_string(), "offline".to_string()),
                ("prompt".to_string(), "consent".to_string()),
            ],
        };

        let oauth = OAuth::new(gemini_config, &http_client).await?;

        let status = match oauth.status().await {
            std::result::Result::Ok(status) => match status {
                AuthStatus::Authenticated => ProviderStatus::Ready,
                AuthStatus::Unauthenticated => ProviderStatus::RequiresAuth,
            },
            Err(_) => ProviderStatus::RequiresAuth,
        };

        Ok(Self {
            oauth,
            status: RwLock::new(status),
        })
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

    async fn setup(&self) -> Result<()> {
        match self.oauth.setup().await {
            std::result::Result::Ok(_) => *self.status.write().unwrap() = ProviderStatus::Ready,
            Err(_) => *self.status.write().unwrap() = ProviderStatus::RequiresAuth,
        }

        Ok(())
    }

    async fn ask(&self, prompt: &String) -> Result<String> {
        const URL: &str =
            "https://generativelanguage.googleapis.com/v1/models/gemini-2.0-flash:generateContent";

        let payload = json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let request = self
            .oauth
            .post(URL)
            .json(&payload)
            .header("x-goog-user-project", env!("GOOGLE_PROJECT_ID"));
        let response = self.oauth.make_request::<GeminiResponse>(request).await?;

        let answer = response
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .context("Gemini API returned an empty or malformed response")?;

        Ok(answer)
    }
}

#[derive(Deserialize, Debug)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize, Debug)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize, Debug)]
struct GeminiPart {
    text: String,
}
