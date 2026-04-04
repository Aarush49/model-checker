use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result, anyhow};
use futures_util::{StreamExt, future::join_all};
use ndarray::{Array1, Array2, Array3, ArrayD};
use ort::{
    ep::CPU,
    memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType},
    value::{DynValue, Tensor},
};
use serde::Deserialize;
use tokenizers::Tokenizer;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

/// Represents the `search` section of a genai_config.json file.
#[derive(Debug, Clone, Deserialize)]
pub struct GenaiSearchConfig {
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_repetition_penalty")]
    pub repetition_penalty: f32,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub max_length: usize,
}

const fn default_temperature() -> f32 { 1.0 }
const fn default_repetition_penalty() -> f32 { 1.0 }
const fn default_top_k() -> usize { 1 }
const fn default_top_p() -> f32 { 1.0 }

impl Default for GenaiSearchConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            repetition_penalty: default_repetition_penalty(),
            top_k: default_top_k(),
            top_p: default_top_p(),
            max_length: 512,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GenaiConfig {
    search: GenaiSearchConfig,
}

pub struct LocalModel {
    model_dir: PathBuf,
    session: Arc<Mutex<Option<ort::session::Session>>>,
    embed_session: Arc<Mutex<Option<ort::session::Session>>>,
    tokenizer: Arc<Mutex<Option<Tokenizer>>>,
}

pub enum LocalStatus {
    Installed,
    NotInstalled,
    PartiallyInstalled,
}

impl LocalModel {
    pub fn new(model_id: String) -> Self {
        let mut path = dirs::data_local_dir().unwrap();
        path.push("ModelChecker");
        path.push("models");
        path.push(&model_id);

        Self {
            model_dir: path,
            session: Arc::new(Mutex::new(None)),
            embed_session: Arc::new(Mutex::new(None)),
            tokenizer: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn read_genai_config(&self) -> Result<GenaiSearchConfig> {
        let config_path = self.model_dir.join("genai_config.json");
        let contents = fs::read_to_string(&config_path)
            .await
            .context(format!("Failed to read {:?}", config_path))?;
        let config: GenaiConfig =
            serde_json::from_str(&contents).context("Failed to parse genai_config.json")?;
        Ok(config.search)
    }

    pub async fn status(&self) -> LocalStatus {
        if !self.model_dir.exists() {
            return LocalStatus::NotInstalled;
        }
        if self.model_dir.join(".install_complete").exists() {
            return LocalStatus::Installed;
        }
        let alternatives = ["model.onnx", "decoder_model_merged_q4.onnx", "model_q4.onnx"];
        for alt in alternatives {
            if self.model_dir.join(alt).exists() {
                return LocalStatus::Installed;
            }
        }
        LocalStatus::PartiallyInstalled
    }

    pub async fn setup(
        &self,
        http_client: &reqwest::Client,
        download_urls: Vec<String>,
        progress_tx: Option<tokio::sync::mpsc::UnboundedSender<(u64, u64)>>,
    ) -> Result<()> {
        if !self.model_dir.exists() {
            fs::create_dir_all(&self.model_dir).await?;
        }
        let mut downloads: Vec<tokio::task::JoinHandle<Result<()>>> = vec![];
        let downloaded = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let total = Arc::new(std::sync::atomic::AtomicU64::new(0));

        for url in download_urls {
            let http_client = http_client.clone();
            let downloaded = Arc::clone(&downloaded);
            let total = Arc::clone(&total);
            let progress_tx = progress_tx.clone();
            let file_path = self.model_dir.join(url.trim().split('/').last().unwrap_or("file.onnx"));

            downloads.push(tokio::spawn(async move {
                let mut file = File::create(&file_path).await?;
                let response = http_client.get(url).send().await?.error_for_status()?;
                if let Some(cl) = response.content_length() { total.fetch_add(cl, std::sync::atomic::Ordering::Relaxed); }
                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let bytes = chunk?;
                    file.write_all(&bytes).await?;
                    let current = downloaded.fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed) + bytes.len() as u64;
                    if let Some(tx) = &progress_tx { let _ = tx.send((current, total.load(std::sync::atomic::Ordering::Relaxed))); }
                }
                Ok(())
            }));
        }
        for res in join_all(downloads).await { res??; }
        let _ = File::create(self.model_dir.join(".install_complete")).await;
        Ok(())
    }

    pub async fn load(&self) -> Result<()> {
        if self.session.lock().unwrap().is_some() && self.tokenizer.lock().unwrap().is_some() {
            return Ok(());
        }

        let model_path = [
            "model.onnx",
            "decoder_model_merged_q4.onnx",
            "model_q4.onnx",
            "decoder_model.onnx",
        ].iter().map(|&alt| self.model_dir.join(alt)).find(|p| p.exists())
         .ok_or_else(|| anyhow!("No model weight found in {:?}", self.model_dir))?;

        let embed_path = ["embed_tokens.onnx", "embed_tokens_q4.onnx", "embeddings.onnx", "embed.onnx"]
            .iter().map(|&alt| self.model_dir.join(alt)).find(|p| p.exists());

        let tokenizer_path = self.model_dir.join("tokenizer.json");
        let session_arc = Arc::clone(&self.session);
        let embed_arc = Arc::clone(&self.embed_session);
        let tokenizer_arc = Arc::clone(&self.tokenizer);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let tokenizer = Tokenizer::from_file(&tokenizer_path).map_err(|e| anyhow!("Tokenizer fail: {e}"))?;
            let execution_providers = vec![CPU::default().build()];

            let session = ort::session::Session::builder()
                .map_err(|e| anyhow!("Builder error: {e}"))?
                .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
                .map_err(|e| anyhow!("Opt error: {e}"))?
                .commit_from_file(&model_path)
                .map_err(|e| anyhow!("Load decoder error: {e}"))?;

            let mut embed_session = None;
            if let Some(ep_path) = embed_path {
                embed_session = Some(ort::session::Session::builder()
                    .map_err(|e| anyhow!("Builder error: {e}"))?
                    .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)
                    .map_err(|e| anyhow!("Opt error: {e}"))?
                    .commit_from_file(ep_path)
                    .map_err(|e| anyhow!("Load embed error: {e}"))?);
            }

            *tokenizer_arc.lock().unwrap() = Some(tokenizer);
            *session_arc.lock().unwrap() = Some(session);
            *embed_arc.lock().unwrap() = embed_session;
            Ok(())
        }).await??;
        Ok(())
    }

    pub async fn ask(
        &self,
        prompt: String,
        config: GenaiSearchConfig,
        tx: tokio::sync::mpsc::UnboundedSender<Result<String>>,
    ) -> Result<()> {
        if prompt.trim().is_empty() { return Ok(()); }
        let session_arc = Arc::clone(&self.session);
        let embed_arc = Arc::clone(&self.embed_session);
        let tokenizer_arc = Arc::clone(&self.tokenizer);

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut session_guard = session_arc.lock().unwrap();
            let session = session_guard.as_mut().context("No session")?;
            let mut embed_guard = embed_arc.lock().unwrap();
            let mut embed_session = embed_guard.as_mut();
            let tokenizer_guard = tokenizer_arc.lock().unwrap();
            let tokenizer = tokenizer_guard.as_ref().context("No tokenizer")?;

            let encoding = tokenizer.encode(prompt, true).map_err(|e| anyhow!("Tokenize fail: {e}"))?;
            let mut current_input_ids: Vec<i64> = encoding.get_ids().iter().map(|&i| i as i64).collect();
            let stop_tokens: Vec<u32> = vec!["<|im_end|>", "<|end|>", "<|eot_id|>", "<|endoftext|>"]
                .into_iter().filter_map(|t| tokenizer.token_to_id(t)).collect();

            let mut is_thinking = false;
            let mut has_started = false;
            let mut past_kv_state: HashMap<String, DynValue> = HashMap::new();
            let mut kv_names: Vec<(String, String)> = Vec::new();

            for input in session.inputs().iter() {
                if input.name().starts_with("past_") {
                    let input_name = input.name().to_string();
                    let candidates = [
                        input_name.replace("past_", "present_"),
                        input_name.replace("past_key_values", "present"),
                        input_name.replace("past_", "present."),
                    ];
                    let mut found_present = None;
                    for cand in candidates {
                        if session.outputs().iter().any(|o| o.name() == cand) {
                            found_present = Some(cand);
                            break;
                        }
                    }
                    if let Some(present_name) = found_present {
                        kv_names.push((input_name.clone(), present_name));
                        let mut real_shape = Vec::new();
                        if let Some(s) = input.dtype().tensor_shape() {
                            for (i, &v) in s.iter().enumerate() {
                                if v == -1 {
                                    real_shape.push(if i == 0 { 1 } else { 0 });
                                } else {
                                    real_shape.push(v as usize);
                                }
                            }
                        } else {
                            real_shape = vec![1, 32, 0, 96];
                        }
                        let empty_kv = ArrayD::<f32>::zeros(real_shape);
                        past_kv_state.insert(input_name, Tensor::from_array(empty_kv).unwrap().into_dyn());
                    }
                }
            }

            let mut past_seq_len = 0;
            let mut all_historical_tokens = current_input_ids.clone();
            let out_mem = MemoryInfo::new(AllocationDevice::CPU, 0, AllocatorType::Device, MemoryType::CPUOutput)?;
            let limit = if config.max_length > 0 { config.max_length } else { 512 };

            for _ in 0..limit {
                let cur_len = current_input_ids.len();
                let tot_len = past_seq_len + cur_len;
                let mut binding = session.create_binding()?;
                let mut keep_alive = Vec::new();

                if let Some(ref mut es) = embed_session {
                    let input_e = Array2::from_shape_vec((1, cur_len), current_input_ids.clone())?;
                    let input_d = Tensor::from_array(input_e).unwrap().into_dyn();
                    let mut eb = es.create_binding()?;
                    eb.bind_input("input_ids", &input_d)?;
                    eb.bind_output_to_device("inputs_embeds", &out_mem)?;
                    let out = es.run_binding(&eb)?;
                    for (n, v) in out { if n == "inputs_embeds" { keep_alive.push(v); } }
                    binding.bind_input("inputs_embeds", &keep_alive[0])?;
                } else {
                    let input_i = Array2::from_shape_vec((1, cur_len), current_input_ids.clone())?;
                    let input_d = Tensor::from_array(input_i).unwrap().into_dyn();
                    keep_alive.push(input_d);
                    binding.bind_input("input_ids", &keep_alive[0])?;
                }

                let mask_d = Tensor::from_array(Array2::<i64>::ones((1, tot_len))).unwrap().into_dyn();
                if session.inputs().iter().any(|i| i.name() == "attention_mask") {
                    binding.bind_input("attention_mask", &mask_d)?;
                }
                let pos_ids_base: Vec<i64> = (past_seq_len as i64..tot_len as i64).collect();
                let mut pos_ids_triple = Vec::with_capacity(pos_ids_base.len() * 3);
                for _ in 0..3 {
                    pos_ids_triple.extend_from_slice(&pos_ids_base);
                }
                let pos_d = Tensor::from_array(Array3::from_shape_vec((3, 1, cur_len), pos_ids_triple)?).unwrap().into_dyn();
                if session.inputs().iter().any(|i| i.name() == "position_ids") {
                    binding.bind_input("position_ids", &pos_d)?;
                }

                for (pn, pv) in &past_kv_state { binding.bind_input(pn.as_str(), pv)?; }
                binding.bind_output_to_device("logits", &out_mem)?;
                for (_, prn) in &kv_names { binding.bind_output_to_device(prn.as_str(), &out_mem)?; }

                let mut outputs = session.run_binding(&binding)?;
                for (pn, prn) in &kv_names {
                    past_kv_state.insert(pn.clone(), outputs.remove(prn.as_str()).unwrap());
                }

                let (shape, data) = outputs["logits"].try_extract_tensor::<f32>()?;
                let start_idx = (shape[1] as usize - 1) * shape[2] as usize;
                let mut logits = data[start_idx..start_idx + shape[2] as usize].to_vec();

                if config.repetition_penalty != 1.0 {
                    for (id, score) in logits.iter_mut().enumerate() {
                        if all_historical_tokens.contains(&(id as i64)) && !stop_tokens.contains(&(id as u32)) {
                            if *score > 0.0 { *score /= config.repetition_penalty; } else { *score *= config.repetition_penalty; }
                        }
                    }
                }

                let next_token_id = if config.temperature <= 1e-8 {
                    logits.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).map(|(id, _)| id as i64).unwrap()
                } else {
                    for l in logits.iter_mut() { *l /= config.temperature; }
                    let max_l = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                    let mut probs_vec: Vec<f32> = logits.iter().map(|&l| (l - max_l).exp()).collect();
                    let sum_p: f32 = probs_vec.iter().sum();
                    for p in probs_vec.iter_mut() { *p /= sum_p; }
                    
                    let mut indexed: Vec<(usize, f32)> = probs_vec.iter().enumerate().map(|(i, &p)| (i, p)).collect();
                    indexed.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                    if config.top_k > 0 && config.top_k < indexed.len() { indexed.truncate(config.top_k); }
                    if config.top_p < 1.0 && config.top_p > 0.0 {
                        let mut cum = 0.0;
                        let mut cut = indexed.len();
                        for (i, (_, p)) in indexed.iter().enumerate() { cum += p; if cum >= config.top_p { cut = i + 1; break; } }
                        indexed.truncate(cut);
                    }
                    let f_sum: f32 = indexed.iter().map(|(_, p)| p).sum();
                    let r = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos() as f32 % 1_000_000.0) / 1_000_000.0;
                    let mut cum_p = 0.0;
                    let mut chosen = indexed[0].0 as i64;
                    for (id, p) in &indexed { cum_p += p / f_sum; if cum_p >= r { chosen = *id as i64; break; } }
                    chosen
                };

                if stop_tokens.contains(&(next_token_id as u32)) { break; }
                let token_str = tokenizer.decode(&[next_token_id as u32], true).unwrap();
                
                // --- THINKING FILTER ---
                if token_str.contains("<think>") {
                    is_thinking = true;
                    past_seq_len += cur_len;
                    current_input_ids = vec![next_token_id];
                    all_historical_tokens.push(next_token_id);
                    continue;
                }
                if token_str.contains("</think>") {
                    is_thinking = false;
                    let parts: Vec<&str> = token_str.split("</think>").collect();
                    if parts.len() > 1 && !parts[1].is_empty() {
                        let trimmed = parts[1].trim_start();
                        if !trimmed.is_empty() {
                            let _ = tx.send(Ok(trimmed.to_string()));
                            has_started = true;
                        }
                    }
                    past_seq_len += cur_len;
                    current_input_ids = vec![next_token_id];
                    all_historical_tokens.push(next_token_id);
                    continue;
                }

                if !is_thinking {
                    if !has_started {
                        let trimmed = token_str.trim_start();
                        if !trimmed.is_empty() {
                            let _ = tx.send(Ok(trimmed.to_string()));
                            has_started = true;
                        }
                    } else {
                        let _ = tx.send(Ok(token_str));
                    }
                }
                
                past_seq_len += cur_len;
                current_input_ids = vec![next_token_id];
                all_historical_tokens.push(next_token_id);
            }
            Ok(())
        }).await??;
        Ok(())
    }
}
