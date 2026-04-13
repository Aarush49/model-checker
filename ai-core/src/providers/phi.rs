use std::sync::RwLock;

use anyhow::Result;
use async_trait::async_trait;

use crate::{
    ai::{ComputeMode, ModelProvider, ProviderStatus},
    local::{LocalModel, LocalStatus},
};

const ID: &'static str = "model.phi-4-mini";

pub struct Phi {
    http_client: reqwest::Client,
    handler: LocalModel,
    status: RwLock<ProviderStatus>,
    temperature: RwLock<f32>,
}

impl Phi {
    pub async fn new(http_client: &reqwest::Client) -> Result<Self> {
        let handler = LocalModel::new(ID.to_string())?;

        let status = match handler.status().await {
            LocalStatus::Installed => ProviderStatus::Ready,
            LocalStatus::NotInstalled | LocalStatus::PartiallyInstalled => {
                ProviderStatus::RequiresInstallation
            }
        };

        // Read the default temperature from genai_config.json if the model is installed
        let default_temp = if status == ProviderStatus::Ready {
            handler
                .read_genai_config()
                .await
                .map(|c| c.temperature)
                .unwrap_or(0.7)
        } else {
            0.7
        };

        Ok(Self {
            http_client: http_client.clone(),
            handler,
            status: RwLock::new(status),
            temperature: RwLock::new(default_temp),
        })
    }
}

#[async_trait]
impl ModelProvider for Phi {
    fn id(&self) -> &'static str {
        ID
    }
    fn name(&self) -> &'static str {
        "Phi 4"
    }

    fn supported_modes(&self) -> Vec<ComputeMode> {
        vec![ComputeMode::Local]
    }

    fn current_mode(&self) -> ComputeMode {
        ComputeMode::Local
    }

    fn status(&self) -> ProviderStatus {
        *self.status.read().unwrap()
    }

    fn temperature(&self) -> f32 {
        *self.temperature.read().unwrap()
    }

    fn set_temperature(&self, temperature: f32) {
        *self.temperature.write().unwrap() = temperature;
    }

    async fn setup(
        &self,
        progress_tx: Option<tokio::sync::mpsc::UnboundedSender<(u64, u64)>>,
    ) -> Result<()> {
        let download_urls = vec![
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/model.onnx".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/model.onnx.data".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/genai_config.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/config.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/tokenizer.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/tokenizer_config.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/special_tokens_map.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/added_tokens.json".to_string(),
        ];

        self.handler
            .setup(&self.http_client, download_urls, progress_tx)
            .await?;

        // After successful install, load the default temperature from the config
        if let Ok(config) = self.handler.read_genai_config().await {
            *self.temperature.write().unwrap() = config.temperature;
        }

        *self.status.write().unwrap() = ProviderStatus::Ready;

        Ok(())
    }

    async fn ask(
        &self,
        prompt: &String,
        tx: tokio::sync::mpsc::UnboundedSender<anyhow::Result<String>>,
    ) -> Result<()> {
        self.handler.load().await?;

        // Read the full generation config and override temperature with the user's current setting
        let mut config = self.handler.read_genai_config().await.unwrap_or_default();
        config.temperature = self.temperature();

        self.handler.ask(
            format!("<|system|>\nYou are a helpful AI assistant. Keep answers short and concise.<|end|>\n<|user|>\n{prompt}<|end|>\n<|assistant|>\n"),
            config,
            tx,
        ).await
    }
}
