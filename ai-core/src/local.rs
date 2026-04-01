use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Ok, Result, anyhow};
use futures_util::{StreamExt, future::join_all};
use ndarray::{Array2, ArrayD};
use ort::{
    ep::CPU,
    memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType},
    session::{Session, builder::GraphOptimizationLevel},
    value::{DynValue, Tensor},
};
use tokenizers::Tokenizer;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

pub struct LocalModel {
    model_dir: PathBuf,
    session: Arc<Mutex<Option<Session>>>,
    tokenizer: Arc<Mutex<Option<Tokenizer>>>,
}

pub enum LocalStatus {
    /// The model is installed and ready to use
    Installed,
    /// The model is not installed
    NotInstalled,
    /// The directory exists but the install didn't finish (e.g. interrupted download)
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
            tokenizer: Arc::new(Mutex::new(None)),
        }
    }
    pub async fn status(&self) -> LocalStatus {
        if !self.model_dir.exists() {
            return LocalStatus::NotInstalled;
        }

        // The marker file is written only after ALL downloads succeed.
        // If the directory exists but the marker is missing, the install was interrupted.
        let marker = self.model_dir.join(".install_complete");
        if marker.exists() {
            LocalStatus::Installed
        } else {
            LocalStatus::PartiallyInstalled
        }
    }

    pub async fn setup(
        &self,
        http_client: &reqwest::Client,
        download_urls: Vec<String>,
        progress_tx: Option<tokio::sync::mpsc::UnboundedSender<(u64, u64)>>,
    ) -> Result<()> {
        // If the directory exists but installation was incomplete, wipe it and start fresh.
        // This handles interrupted downloads, corrupted files, etc.
        let marker = self.model_dir.join(".install_complete");
        if self.model_dir.exists() && !marker.exists() {
            println!("Detected incomplete installation at {:?}, cleaning up...", self.model_dir);
            fs::remove_dir_all(&self.model_dir).await.ok();
        }

        fs::create_dir_all(&self.model_dir).await?;

        let mut downloads = vec![];

        let downloaded = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let total = Arc::new(std::sync::atomic::AtomicU64::new(0));

        for url in download_urls {
            let http_client = http_client.clone();
            let downloaded = Arc::clone(&downloaded);
            let total = Arc::clone(&total);
            let progress_tx = progress_tx.clone();

            let file_name = url.trim().split('/').last().unwrap_or("unknown_file");
            let file_path = self.model_dir.join(file_name);

            println!("Downloading: {:?}", file_path);

            downloads.push(tokio::spawn(async move {
                let mut file = File::create(&file_path)
                    .await
                    .context(format!("Failed to create file at {:?}", file_path))?;
                let response = http_client.get(url).send().await?.error_for_status()?;

                // Add this file's size to the total
                if let Some(content_length) = response.content_length() {
                    total.fetch_add(content_length, std::sync::atomic::Ordering::Relaxed);
                }

                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let bytes = chunk?;
                    let len = bytes.len() as u64;

                    file.write_all(&bytes).await?;

                    let new_downloaded =
                        downloaded.fetch_add(len, std::sync::atomic::Ordering::Relaxed) + len;
                    let current_total = total.load(std::sync::atomic::Ordering::Relaxed);

                    if let Some(tx) = &progress_tx {
                        let _ = tx.send((new_downloaded, current_total));
                    }
                }

                Ok(())
            }));
        }

        let results = join_all(downloads).await;

        for res in results {
            res??;
        }

        // All downloads succeeded — write the completion marker.
        File::create(&marker).await?;

        Ok(())
    }

    pub async fn load(&self) -> Result<()> {
        if self.session.lock().unwrap().is_some() && self.tokenizer.lock().unwrap().is_some() {
            return Ok(());
        }

        let tokenizer_path = self.model_dir.join("tokenizer.json");
        let model_path = self.model_dir.join("model.onnx");

        let session_arc = Arc::clone(&self.session);
        let tokenizer_arc = Arc::clone(&self.tokenizer);

        tokio::task::spawn_blocking(move || -> Result<()> {
            println!("Loading tokenizer into memory...");
            let tokenizer = Tokenizer::from_file(&tokenizer_path)
                .map_err(|e| anyhow::anyhow!("Failed to parse tokenizer.json: {}", e))?;

            let execution_providers = vec![CPU::default().build()];

            let session = Session::builder()?
                .with_optimization_level(GraphOptimizationLevel::Level3)
                .map_err(|e| anyhow::anyhow!("Failed to set optimization level: {}", e))?
                .with_intra_threads(std::thread::available_parallelism().unwrap().get())
                .map_err(|e| anyhow::anyhow!("Failed to set intra threads: {}", e))?
                .with_memory_pattern(true)
                .map_err(|e| anyhow!("Failed to set memory pattern: {e}"))?
                .with_parallel_execution(true)
                .map_err(|e| anyhow!("Failed to set parallel execution: {e}"))?
                .with_execution_providers(execution_providers)
                .map_err(|e| anyhow::anyhow!("Failed to set execution providers: {}", e))?
                .commit_from_file(&model_path)?;

            // 2. Lock the mutexes again just to insert the newly loaded engines!
            *tokenizer_arc.lock().unwrap() = Some(tokenizer);
            *session_arc.lock().unwrap() = Some(session);

            println!("Model loaded successfully!");
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn ask(
        &self,
        prompt: String,
        max_tokens: usize,
        tx: tokio::sync::mpsc::UnboundedSender<anyhow::Result<String>>,
    ) -> Result<()> {
        // Ensure prompt not empty
        if prompt.trim().is_empty() {
            return Ok(());
        }

        let session_arc = Arc::clone(&self.session);
        let tokenizer_arc = Arc::clone(&self.tokenizer);

        let _generated_text = tokio::task::spawn_blocking(move || -> Result<()> {
            let mut session_guard = session_arc.lock().unwrap();
            let session = session_guard.as_mut().context("Model not loaded!")?;

            let tokenizer_guard = tokenizer_arc.lock().unwrap();
            let tokenizer = tokenizer_guard.as_ref().context("Tokenizer not loaded")?;

            let encoding = tokenizer
                .encode(prompt, true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

            let stop_tokens: Vec<u32> =
                vec!["<|im_end|>", "<|end|>", "<|eot_id|>", "<|endoftext|>"]
                    .into_iter()
                    .filter_map(|t| tokenizer.token_to_id(t))
                    .collect();
            let mut current_input_ids: Vec<i64> =
                encoding.get_ids().iter().map(|&i| i as i64).collect();

            // =================================================================
            // PHASE 1: KV CACHE DISCOVERY & ALLOCATION
            // =================================================================
            // We use a HashMap to store the historical math states for every layer natively via ort Values.
            let mut past_kv_state: HashMap<String, DynValue> = HashMap::new();

            // We map the input names (past_) to the output names (present_)
            let mut kv_names: Vec<(String, String)> = Vec::new();

            // Fallback dimensions for Phi/Llama (Batch=1, Heads=32, SeqLen=0, HeadDim=96)
            let mut num_heads = 32;
            let mut head_dim = 96;

            for input in session.inputs().iter() {
                // If the ONNX file asks for past_key_values, we prepare to feed it
                if input.name().starts_with("past_key_values") {
                    let mut present_name = input.name().replace("past_key_values", "present");

                    let exists = session.outputs().iter().any(|o| o.name() == present_name);
                    if !exists {
                        // Fallback if the engineers used a different naming scheme
                        present_name = input.name().replace("past_", "present_");
                    }

                    kv_names.push((input.name().to_string(), present_name));

                    // Read the ONNX graph to dynamically find the correct head size
                    if let Some(dimensions) = input.dtype().tensor_shape() {
                        if dimensions.len() == 4 {
                            if dimensions[1] != -1 {
                                num_heads = dimensions[1] as usize;
                            }
                            if dimensions[3] != -1 {
                                head_dim = dimensions[3] as usize;
                            }
                        }
                    }

                    // Create the initial empty tensor (Sequence Length is 0)
                    let empty_kv =
                        ArrayD::<f32>::from_shape_vec(vec![1, num_heads, 0, head_dim], vec![])
                            .unwrap();
                    let empty_tensor = Tensor::from_array(empty_kv).unwrap().into_dyn();
                    past_kv_state.insert(input.name().to_string(), empty_tensor);
                }
            }

            let mut past_seq_len = 0;

            let mut all_historical_tokens = current_input_ids.clone();

            let out_mem_info = MemoryInfo::new(
                AllocationDevice::CPU,
                0,
                AllocatorType::Device,
                MemoryType::CPUOutput,
            )?;

            // =================================================================
            // PHASE 2: THE INFERENCE LOOP
            // =================================================================
            for _ in 0..max_tokens {
                let current_seq_len = current_input_ids.len();
                let total_seq_len = past_seq_len + current_seq_len;

                // 1. Build the primary tensors (Notice we use total_seq_len for the mask)
                let input_tensor =
                    Array2::from_shape_vec((1, current_seq_len), current_input_ids.clone())?;
                let attention_mask_tensor = Array2::<i64>::ones((1, total_seq_len));

                let pos_ids: Vec<i64> = (past_seq_len as i64..total_seq_len as i64).collect();
                let position_ids_tensor = Array2::from_shape_vec((1, current_seq_len), pos_ids)?;

                // 2. Package everything into a dynamic list of inputs via IoBinding
                let mut binding = session.create_binding()?;

                let input_tensor_dyn = Tensor::from_array(input_tensor)?.into_dyn();
                binding.bind_input("input_ids", &input_tensor_dyn)?;

                let attention_mask_dyn = Tensor::from_array(attention_mask_tensor)?.into_dyn();
                if session
                    .inputs()
                    .iter()
                    .any(|i| i.name() == "attention_mask")
                {
                    binding.bind_input("attention_mask", &attention_mask_dyn)?;
                }

                let position_ids_dyn = Tensor::from_array(position_ids_tensor)?.into_dyn();
                if session.inputs().iter().any(|i| i.name() == "position_ids") {
                    binding.bind_input("position_ids", &position_ids_dyn)?;
                }

                // Inject the 64 historical memory states!
                for (past_name, past_tensor) in &past_kv_state {
                    binding.bind_input(past_name.as_str(), past_tensor)?;
                }

                binding.bind_output_to_device("logits", &out_mem_info)?;
                for (_, present_name) in &kv_names {
                    binding.bind_output_to_device(present_name.as_str(), &out_mem_info)?;
                }

                // 3. RUN THE NPU/CPU
                let mut outputs = session.run_binding(&binding)?;

                // 4. Update our HashMap with the AI's newest memories natively
                for (past_name, present_name) in &kv_names {
                    let present_tensor = outputs.remove(present_name.as_str()).unwrap();
                    past_kv_state.insert(past_name.clone(), present_tensor);
                }

                // 5. Calculate the next word
                let (shape, data) = outputs["logits"].try_extract_tensor::<f32>()?;

                let seq_len = shape[1] as usize;
                let vocab_size = shape[2] as usize;

                let start_idx = (seq_len - 1) * vocab_size;
                let last_token_logits = &data[start_idx..start_idx + vocab_size];

                let mut next_token_id = 0;
                let mut highest_score = f32::NEG_INFINITY;

                let penalty = 1.05_f32;

                for (id, &score) in last_token_logits.iter().enumerate() {
                    let mut score = score;

                    // Apply repetition penalty, but DO NOT penalize our stop tokens!
                    // Otherwise the model will never be allowed to stop generating.
                    if all_historical_tokens.contains(&(id as i64))
                        && !stop_tokens.contains(&(id as u32))
                    {
                        if score > 0.0 {
                            score /= penalty;
                        } else {
                            score *= penalty; // For negative logits, multiplying makes them more negative
                        }
                    }

                    if score > highest_score {
                        highest_score = score;
                        next_token_id = id as i64;
                    }
                }

                if stop_tokens.contains(&(next_token_id as u32)) {
                    break;
                }

                let token_str = tokenizer
                    .decode(&[next_token_id as u32], true)
                    .map_err(|e| anyhow::anyhow!("Detokenization failed: {e}"))?;

                let _ = tx.send(Ok(token_str.clone()));

                // 6. THE CRITICAL PHASE SHIFT
                // We add the length of the tokens we just processed to the historical count.
                // Then, we clear our input array and ONLY pass the brand new token!
                past_seq_len += current_seq_len;
                current_input_ids = vec![next_token_id];

                all_historical_tokens.push(next_token_id);
            }

            Ok(())
        })
        .await??;

        Ok(())
    }
}
