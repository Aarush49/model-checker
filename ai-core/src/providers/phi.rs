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
    pub async fn new(http_client: &reqwest::Client) -> Self {
        let handler = LocalModel::new(ID.to_string());

        let status = match handler.status().await {
            LocalStatus::Installed => ProviderStatus::Ready,
            LocalStatus::NotInstalled => ProviderStatus::RequiresInstallation,
        };

        Self {
            http_client: http_client.clone(),
            handler,
            status: RwLock::new(status),
            temperature: RwLock::new(0.7),
        }
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

    async fn setup(&self) -> Result<()> {
        let download_urls = vec![
            // 1. The Core Graph (The "Brain" structure)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/model.onnx".to_string(),

            // 2. The Heavyweight Weights (The ~2.5GB file - this will take a few minutes!)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/model.onnx.data".to_string(),

            // 3. The Generation Config (Tells ORT the default Temperature, Top-P, etc.)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/genai_config.json".to_string(),

            // 4. Standard Model Config
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/config.json".to_string(),

            // 5. The Tokenizer Files (Converts your text into NPU integers)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/tokenizer.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/tokenizer_config.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/special_tokens_map.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/cpu_and_mobile/cpu-int4-rtn-block-32-acc-level-4/added_tokens.json".to_string(),
        ];

        self.handler.setup(&self.http_client, download_urls).await?;

        *self.status.write().unwrap() = ProviderStatus::Ready;

        Ok(())
    }

    async fn ask(&self, prompt: &String) -> Result<String> {
        self.handler.load().await?;

        self.handler.ask(format!("<|system|>\nYou are a helpful AI assistant.<|end|>\n<|user|>\n{prompt}<|end|>\n<|assistant|>\n"), 1024).await
    }
}
