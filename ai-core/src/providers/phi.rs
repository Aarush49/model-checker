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

    async fn setup(&self) -> Result<()> {
        let download_urls = vec![
            // 1. The Core Graph (The "Brain" structure)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/model.onnx".to_string(),

            // 2. The Heavyweight Weights (The ~2.5GB file - this will take a few minutes!)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/model.onnx.data".to_string(),

            // 3. The Generation Config (Tells ORT the default Temperature, Top-P, etc.)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/genai_config.json".to_string(),

            // 4. Standard Model Config
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/config.json".to_string(),

            // 5. The Tokenizer Files (Converts your text into NPU integers)
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/tokenizer.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/tokenizer_config.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/special_tokens_map.json".to_string(),
            "https://huggingface.co/microsoft/Phi-4-mini-instruct-onnx/resolve/main/gpu/gpu-int4-rtn-block-32/added_tokens.json".to_string(),
        ];

        self.handler.setup(&self.http_client, download_urls).await?;

        *self.status.write().unwrap() = ProviderStatus::Ready;

        Ok(())
    }

    async fn ask(&self, prompt: &String) -> Result<String> {
        self.handler.load()?;

        self.handler.ask(format!("<|im_start|>system<|im_sep|>You are a helpful AI assistant.<|im_end|>\n<|im_start|>user<|im_sep|>{prompt}<|im_end|>\n<|im_start|>assistant<|im_sep|>"), 1024).await
    }
}
