use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Ok, Result, anyhow};
use futures_util::{StreamExt, future::join_all};
use ndarray::{Array2, ArrayD};
use ort::{
    ep::{CPU},
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
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
        if self.model_dir.exists() {
            LocalStatus::Installed
        } else {
            LocalStatus::NotInstalled
        }
    }

    pub async fn setup(
        &self,
        http_client: &reqwest::Client,
        download_urls: Vec<String>,
    ) -> Result<()> {
        let mut downloads = vec![];

        fs::create_dir_all(&self.model_dir).await?;

        for url in download_urls {
            let http_client = http_client.clone();

            let file_name = url.trim().split('/').last().unwrap_or("unknown_file");
            let file_path = self.model_dir.join(file_name);

            println!("Attempting to create: {:?}", file_path);

            downloads.push(tokio::spawn(async move {
                let mut file = File::create(&file_path)
                    .await
                    .context(format!("Failed to create file at {:?}", file_path))?;
                let response = http_client.get(url).send().await?.error_for_status()?;

                let mut stream = response.bytes_stream();
                while let Some(chunk) = stream.next().await {
                    let bytes = chunk?;

                    file.write_all(&bytes).await?;
                }

                Ok(())
            }));
        }

        let results = join_all(downloads).await;

        for res in results {
            res??;
        }

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

            let execution_providers = vec![
                CPU::default().build(),
            ];

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

    pub async fn ask(&self, prompt: String, max_tokens: usize) -> Result<String> {
        // Ensure prompt not empty
        if prompt.trim().is_empty() {
            return Ok("".to_string());
        }

        let session_arc = Arc::clone(&self.session);
        let tokenizer_arc = Arc::clone(&self.tokenizer);

        let generated_text = tokio::task::spawn_blocking(move || -> Result<String> {
            let mut session_guard = session_arc.lock().unwrap();
            let session = session_guard.as_mut().context("Model not loaded!")?;

            let tokenizer_guard = tokenizer_arc.lock().unwrap();
            let tokenizer = tokenizer_guard.as_ref().context("Tokenizer not loaded")?;

            let encoding = tokenizer
                .encode(prompt, true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

            let stop_tokens: Vec<u32> = vec!["<|im_end|>", "<|end|>", "<|eot_id|>", "<|endoftext|>"]
                .into_iter()
                .filter_map(|t| tokenizer.token_to_id(t))
                .collect();

            let mut current_input_ids: Vec<i64> =
                encoding.get_ids().iter().map(|&x| x as i64).collect();

            let mut generated_text = String::new();

            // =================================================================
            // PHASE 1: KV CACHE DISCOVERY & ALLOCATION
            // =================================================================
            // We use a HashMap to store the historical math states for every layer.
            let mut past_kv_state: HashMap<String, ArrayD<f32>> = HashMap::new();

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
                    past_kv_state.insert(input.name().to_string(), empty_kv);
                }
            }

            let mut past_seq_len = 0;

            let mut all_historical_tokens = current_input_ids.clone();

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

                // 2. Package everything into a dynamic list of inputs
                // We use a Vec because we don't know exactly how many KV layers the model has
                let mut run_inputs = Vec::new();
                run_inputs.push((
                    "input_ids".to_string(),
                    Tensor::from_array(input_tensor)?.into_dyn(),
                ));
                if session.inputs().iter().any(|i| i.name() == "attention_mask") {
                    run_inputs.push((
                        "attention_mask".to_string(),
                        Tensor::from_array(attention_mask_tensor)?.into_dyn(),
                    ));
                }
                if session.inputs().iter().any(|i| i.name() == "position_ids") {
                    run_inputs.push((
                        "position_ids".to_string(),
                        Tensor::from_array(position_ids_tensor)?.into_dyn(),
                    ));
                }

                // Inject the 64 historical memory states!
                for (past_name, past_tensor) in &past_kv_state {
                    run_inputs.push((
                        past_name.clone(),
                        Tensor::from_array(past_tensor.clone())?.into_dyn(),
                    ));
                }

                // 3. RUN THE NPU/CPU
                let outputs = session.run(run_inputs)?;

                // 4. Update our HashMap with the AI's newest memories
                for (past_name, present_name) in &kv_names {
                    let (shape, data) =
                        outputs[present_name.as_str()].try_extract_tensor::<f32>()?;

                    let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
                    let present_tensor = ArrayD::from_shape_vec(shape_usize, data.to_vec())?;

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
                    if all_historical_tokens.contains(&(id as i64)) && !stop_tokens.contains(&(id as u32)) {
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

                generated_text.push_str(&token_str);

                // Stream it live to the console so you can watch it go fast!
                print!("{}", token_str);
                use std::io::Write;
                std::io::stdout().flush().unwrap();

                // 6. THE CRITICAL PHASE SHIFT
                // We add the length of the tokens we just processed to the historical count.
                // Then, we clear our input array and ONLY pass the brand new token!
                past_seq_len += current_seq_len;
                current_input_ids = vec![next_token_id];

                all_historical_tokens.push(next_token_id);
            }

            Ok(generated_text)
        })
        .await??;

        Ok(generated_text)
    }
}
