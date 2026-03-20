use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ndarray::{Array, Array2, ArrayD, ArrayViewD};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;

// --- Config Types ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    Poincare,
    Lorentz,
    L2,
    Cosine,
    None,
}

/// Chunking configuration for long document processing
/// Model-agnostic: enabled via env vars if needed
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub chunk_size: usize, // tokens per chunk (e.g., 512, 4096)
    pub overlap: f64,      // overlap ratio (e.g., 0.10 = 10%)
}

impl ChunkingConfig {
    /// Load from env vars for a specific metric
    /// Returns None if chunking is not configured
    pub fn from_env(metric: &str) -> Option<Self> {
        let chunk_size_key = format!("HS_EMBED_{}_CHUNK_SIZE", metric.to_uppercase());
        let overlap_key = format!("HS_EMBED_{}_OVERLAP", metric.to_uppercase());

        let chunk_size = std::env::var(&chunk_size_key).ok()?.parse::<usize>().ok()?;

        let overlap = std::env::var(&overlap_key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.10); // Default 10% overlap

        Some(Self {
            chunk_size,
            overlap,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ApiProvider {
    OpenAI,
    Cohere,
    Voyage,
    Mistral,
    Gemini,
    OpenRouter,
    Generic,
}

impl std::str::FromStr for ApiProvider {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "cohere" => Ok(Self::Cohere),
            "voyage" => Ok(Self::Voyage),
            "mistral" => Ok(Self::Mistral),
            "gemini" => Ok(Self::Gemini),
            "openrouter" => Ok(Self::OpenRouter),
            "generic" => Ok(Self::Generic),
            _ => Err(()),
        }
    }
}

// --- Trait ---

#[async_trait]
pub trait Vectorizer: Send + Sync {
    async fn vectorize(&self, texts: Vec<String>) -> Result<Vec<Vec<f64>>>;
    fn dimension(&self) -> usize;
}

// --- Multi-Vectorizer (Routes by Metric) ---

pub struct MultiVectorizer {
    pub models: HashMap<String, Arc<dyn Vectorizer>>,
}

impl MultiVectorizer {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }
}

impl Default for MultiVectorizer {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiVectorizer {
    pub fn add(&mut self, metric: &str, vectorizer: Arc<dyn Vectorizer>) {
        self.models.insert(metric.to_string(), vectorizer);
    }

    pub async fn vectorize_for(&self, texts: Vec<String>, metric: &str) -> Result<Vec<Vec<f64>>> {
        let metric_key = match metric.to_lowercase().as_str() {
            "l2" | "euclidean" => "l2",
            "cosine" => "cosine",
            "poincare" => "poincare",
            "lorentz" => "lorentz",
            _ => metric,
        };

        if let Some(v) = self.models.get(metric_key) {
            v.vectorize(texts).await
        } else {
            // Fallback to primary if exists
            if let Some(v) = self
                .models
                .get("l2")
                .or_else(|| self.models.values().next())
            {
                v.vectorize(texts).await
            } else {
                Err(anyhow!("No vectorizer available"))
            }
        }
    }
}

// --- Local ONNX Vectorizer ---

pub struct OnnxVectorizer {
    tokenizer: Tokenizer,
    session: Mutex<Session>,
    dimension: usize,
    metric: Metric,
    chunking_config: Option<ChunkingConfig>, // Optional chunking (model-agnostic)
    #[allow(dead_code)] // Kept for future debugging/logging
    model_id: String,
}

impl OnnxVectorizer {
    /// Creates a new `OnnxVectorizer`.
    ///
    /// # Errors
    /// Returns error if model loading or tokenizer loading fails.
    pub fn new(
        model_path: &str,
        tokenizer_path: &str,
        dimension: usize,
        metric: Metric,
        metric_name: &str, // For env var lookup (e.g., "L2", "COSINE", "LORENTZ", "POINCARE")
    ) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

        let session = Session::builder()
            .map_err(|e| anyhow::anyhow!("Ort session builder failed: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Ort optimization failure: {e}"))?
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("Ort thread configuration failure: {e}"))?
            .commit_from_file(model_path)
            .map_err(|e| {
                anyhow::anyhow!("Ort session commit failed for path {}: {}", model_path, e)
            })?;

        // Load chunking config from env (model-agnostic)
        let chunking_config = ChunkingConfig::from_env(metric_name);

        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
            dimension,
            metric,
            chunking_config,
            model_id: "local".to_string(),
        })
    }

    ///
    /// # Errors
    /// Returns error if API key is invalid, model download fails, or model parsing fails.
    pub fn new_from_hf(
        model_id: &str,
        hf_token: Option<String>,
        dimension: usize,
        metric: Metric,
        metric_name: &str, // For env var lookup (e.g., "L2", "COSINE", "LORENTZ", "POINCARE")
        model_file: Option<String>,
    ) -> Result<Self> {
        use hf_hub::api::sync::{ApiBuilder, ApiRepo};
        use std::path::PathBuf;

        // 1. Setup HF hub with progress disabled but cache enabled
        let mut builder = ApiBuilder::new().with_progress(false); // Disable progress bar for cleaner logs

        if let Some(token) = hf_token.clone() {
            if !token.is_empty() {
                builder = builder.with_token(Some(token));
            }
        }

        // 2. Get cache directory for logging
        let cache_dir = PathBuf::from(std::env::var("HF_HOME").unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| format!("{}/.cache/huggingface", h))
                .unwrap_or_else(|_| "./.hf_cache".to_string())
        }));
        let model_path_cached = cache_dir.join("hub").join(model_id.replace('/', "--"));
        let model_onnx_path = model_path_cached.join("model.onnx");
        let tokenizer_path_cached = model_path_cached.join("tokenizer.json");

        // 3. Check if already cached
        let is_cached = model_onnx_path.exists() && tokenizer_path_cached.exists();

        // 4. Log appropriately
        if is_cached {
            eprintln!(
                "✅ Using cached model: {} ({})",
                model_id,
                model_path_cached.display()
            );
        } else {
            eprintln!(
                "📥 Loading model from HF Hub: {} → {}",
                model_id,
                model_path_cached.display()
            );
        }

        // 5. Download/load model (hf_hub handles caching automatically)
        let api = builder.build().map_err(|e| {
            eprintln!("❌ HF API error for {}: {}", model_id, e);
            anyhow::anyhow!("HF API error: {e}")
        })?;
        let repo: ApiRepo = api.model(model_id.to_string());

        let filename = model_file.unwrap_or_else(|| "model.onnx".to_string());
        let model_path = repo.get(&filename).map_err(|e| {
            eprintln!("❌ Failed to download {} for {}: {}", filename, model_id, e);
            anyhow::anyhow!("Failed to download {}: {e}", filename)
        })?;

        // Try to download external data for large models (.onnx.data or .onnx_data)
        for suffix in &[".data", "_data"] {
            let data_filename = format!("{}{}", filename, suffix);
            let _ = repo.get(&data_filename);
        }
        let tokenizer_path = repo.get("tokenizer.json").map_err(|e| {
            eprintln!(
                "❌ Failed to download tokenizer.json for {}: {}",
                model_id, e
            );
            anyhow::anyhow!("Failed to download tokenizer.json: {e}")
        })?;

        // 6. Load tokenizer
        let tokenizer = Tokenizer::from_file(
            tokenizer_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid tokenizer path"))?,
        )
        .map_err(|e| {
            eprintln!("❌ Failed to parse tokenizer.json for {}: {}", model_id, e);
            anyhow::anyhow!("Failed to load tokenizer: {e}")
        })?;

        // 7. Load session
        let session = Session::builder()
            .map_err(|e| {
                eprintln!("❌ Ort session builder failed: {}", e);
                anyhow::anyhow!("Ort session builder failed: {e}")
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("Ort optimization failure: {e}"))?
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("Ort thread configuration failure: {e}"))?
            .commit_from_file(
                model_path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid model path"))?,
            )
            .map_err(|e| {
                eprintln!(
                    "❌ Ort session commit failed for {} path {:?}: {}",
                    model_id, model_path, e
                );
                anyhow::anyhow!("Ort session commit failed: {e}")
            })?;

        // 8. Load chunking config from env (model-agnostic)
        let chunking_config = ChunkingConfig::from_env(metric_name);

        // 9. Log activation
        let chunk_info = chunking_config
            .as_ref()
            .map(|c| {
                format!(
                    "chunk={} tokens, overlap={:.0}%",
                    c.chunk_size,
                    c.overlap * 100.0
                )
            })
            .unwrap_or_else(|| "no chunking".to_string());

        eprintln!(
            "🚀 Model activated: {} ({}d, {}, metric={:?})",
            model_id, dimension, chunk_info, metric
        );

        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
            dimension,
            metric,
            chunking_config,
            model_id: model_id.to_string(),
        })
    }

    fn normalize(&self, vec: &mut Vec<f64>) {
        const EPSILON: f64 = 1e-12; // Use stricter epsilon for hyperbolic geometry
        let mut norm_sq: f64 = vec.iter().map(|x| x * x).sum();
        let mut norm = norm_sq.sqrt();

        match self.metric {
            Metric::Poincare => {
                // Task: Project to Poincare Ball (Dimension N)
                // Case 1: Model outputs N+1 dims (Lorentz Point/Hyperboloid)
                // Result: Dimensionality reduction (e.g. 129 -> 128)
                if vec.len() == self.dimension + 1 {
                    let x0 = vec[0];
                    let denom = (1.0 + x0).max(EPSILON);
                    let mut projected = Vec::with_capacity(self.dimension);
                    for &x_val in vec.iter().skip(1) {
                        projected.push(x_val / denom);
                    }
                    *vec = projected;

                    // Re-calculate norm for clamping
                    norm_sq = vec.iter().map(|x| x * x).sum();
                    norm = norm_sq.sqrt();
                }

                // Case 2: Model outputs N dims (Tangent Space or raw Ball coordinates)
                // Standard stereographic projection or scaling to unit ball
                if norm >= 1.0 - EPSILON {
                    let scale = (1.0 - EPSILON) / (norm + EPSILON);
                    for x in vec.iter_mut() {
                        *x *= scale;
                    }
                }

                // NaN/Inf Guard
                for x in vec.iter_mut() {
                    if x.is_nan() || x.is_infinite() {
                        *x = 0.0;
                    }
                }
            }
            Metric::Lorentz => {
                // Task: Project to Lorentz Hyperboloid (Dimension N)
                // In HyperspaceDB, Lorentz metric REQUIRES N+1 format (e.g. 129)

                if vec.is_empty() {
                    return;
                }

                // Case 1: Model outputs N-1 dims (Spatial/Tangent vector)
                // Result: Dimension expansion (e.g. 128 -> 129)
                if vec.len() == self.dimension - 1 {
                    let spatial_norm_sq = norm_sq;
                    let x0 = (1.0 + spatial_norm_sq).sqrt();
                    vec.insert(0, x0);
                }
                // Case 2: Model outputs N dims (already Hyperboloid format)
                else if vec.len() == self.dimension {
                    // Constraint: -x0^2 + |x|^2 = -1  =>  x0 = sqrt(1 + |x|^2)
                    let spatial_norm_sq: f64 = vec[1..].iter().map(|x| x * x).sum();
                    vec[0] = (1.0 + spatial_norm_sq).sqrt(); // Enforce upper sheet constraint
                }

                // NaN/Infinity protection
                for x in vec.iter_mut() {
                    if x.is_nan() || x.is_infinite() {
                        *x = 0.0;
                    }
                }
            }
            Metric::None => {}
            Metric::L2 | Metric::Cosine => {
                if norm > 0.0 {
                    for x in vec.iter_mut() {
                        *x /= norm;
                    }
                }
            }
        }
    }

    /// Split text into chunks with overlap (word-based for simplicity)
    fn split_into_chunks(text: &str, config: &ChunkingConfig) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = config.chunk_size;
        let overlap_size = (chunk_size as f64 * config.overlap) as usize;
        let step_size = chunk_size.saturating_sub(overlap_size);

        if words.len() <= chunk_size {
            // No chunking needed
            return vec![text.to_string()];
        }

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < words.len() {
            let end = (start + chunk_size).min(words.len());
            let chunk = words[start..end].join(" ");
            chunks.push(chunk);

            if end == words.len() {
                break;
            }

            start += step_size;
        }

        chunks
    }

    /// Aggregate chunk embeddings via mean pooling
    fn aggregate_embeddings(chunk_embeddings: Vec<Vec<f64>>) -> Vec<f64> {
        if chunk_embeddings.is_empty() {
            return vec![];
        }

        let dim = chunk_embeddings[0].len();
        let mut aggregated = vec![0.0; dim];

        for emb in &chunk_embeddings {
            for (i, &v) in emb.iter().enumerate() {
                aggregated[i] += v;
            }
        }

        let count = chunk_embeddings.len() as f64;
        for v in &mut aggregated {
            *v /= count;
        }

        aggregated
    }

    #[allow(clippy::unused_self)]
    fn mean_pooling(
        &self,
        last_hidden_state: &ArrayViewD<f32>,
        attention_mask: &ArrayViewD<i64>,
    ) -> Vec<Vec<f64>> {
        let shape = last_hidden_state.shape();
        let batch_size = shape[0];
        let seq_len = shape[1];
        let hidden_dim = shape[2];
        let mut output = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let mut sum_vec = vec![0.0f64; hidden_dim];
            let mut count = 0.0f64;
            for j in 0..seq_len {
                if attention_mask[[i, j]] == 1 {
                    for (k, item) in sum_vec.iter_mut().enumerate().take(hidden_dim) {
                        *item += f64::from(last_hidden_state[[i, j, k]]);
                    }
                    count += 1.0;
                }
            }
            if count > 0.0 {
                for item in sum_vec.iter_mut().take(hidden_dim) {
                    *item /= count;
                }
            }
            output.push(sum_vec);
        }
        output
    }
}

#[async_trait]
impl Vectorizer for OnnxVectorizer {
    fn dimension(&self) -> usize {
        self.dimension
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn vectorize(&self, texts: Vec<String>) -> Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        // Check if chunking is enabled
        if let Some(config) = &self.chunking_config {
            let texts_len = texts.len();

            // Split all texts into chunks
            let all_chunks: Vec<(usize, String)> = texts
                .into_iter()
                .enumerate()
                .flat_map(|(text_idx, text)| {
                    OnnxVectorizer::split_into_chunks(&text, config)
                        .into_iter()
                        .map(move |chunk| (text_idx, chunk))
                })
                .collect();

            if all_chunks.is_empty() {
                return Ok(vec![vec![0.0; self.dimension]; texts_len]);
            }

            // Process chunks in batches (batch_size=16 for efficiency)
            let batch_size = 16;
            let mut chunk_embeddings: Vec<Vec<Vec<f64>>> = vec![Vec::new(); texts_len];

            for chunk_batch in all_chunks.chunks(batch_size) {
                let chunk_texts: Vec<String> = chunk_batch.iter().map(|(_, c)| c.clone()).collect();
                let embeddings = self.vectorize_direct(chunk_texts).await?;

                for ((text_idx, _), embedding) in chunk_batch.iter().zip(embeddings) {
                    chunk_embeddings[*text_idx].push(embedding);
                }
            }

            // Aggregate chunk embeddings per text via mean pooling
            let final_embeddings = chunk_embeddings
                .into_iter()
                .map(|chunks| {
                    if chunks.is_empty() {
                        vec![0.0; self.dimension]
                    } else {
                        OnnxVectorizer::aggregate_embeddings(chunks)
                    }
                })
                .collect();

            return Ok(final_embeddings);
        }

        // No chunking - direct vectorization
        self.vectorize_direct(texts).await
    }
}

impl OnnxVectorizer {
    /// Direct vectorization without chunking (internal helper)
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    async fn vectorize_direct(&self, texts: Vec<String>) -> Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let encoding = self
            .tokenizer
            .encode_batch(texts.clone(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

        let batch_size = encoding.len();
        if batch_size == 0 {
            return Ok(vec![]);
        }
        let seq_len = encoding[0].len();

        let mut input_ids = Vec::with_capacity(batch_size * seq_len);
        let mut attention_mask = Vec::with_capacity(batch_size * seq_len);

        for enc in &encoding {
            input_ids.extend(enc.get_ids().iter().map(|&x| i64::from(x)));
            attention_mask.extend(enc.get_attention_mask().iter().map(|&x| i64::from(x)));
        }

        let input_ids_arr = Array::from_shape_vec((batch_size, seq_len), input_ids)?;
        let attention_mask_arr = Array::from_shape_vec((batch_size, seq_len), attention_mask)?;

        let attention_mask_clone = attention_mask_arr.clone();

        // 6. Prepare inputs
        let mut session_guard = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("Session lock poisoned"))?;

        // Detect required inputs
        let mut inputs: Vec<(String, Value)> = Vec::new();
        for input in session_guard.inputs() {
            let name = input.name();
            match name {
                "input_ids" => {
                    inputs.push((
                        name.to_string(),
                        Value::from_array(input_ids_arr.clone())?.into(),
                    ));
                }
                "attention_mask" => {
                    inputs.push((
                        name.to_string(),
                        Value::from_array(attention_mask_arr.clone())?.into(),
                    ));
                }
                "position_ids" => {
                    let position_ids_arr =
                        Array::from_shape_fn((batch_size, seq_len), |(_, c)| c as i64);
                    inputs.push((
                        name.to_string(),
                        Value::from_array(position_ids_arr)?.into(),
                    ));
                }
                name if name.contains("past_key_values") => {
                    if let ort::value::ValueType::Tensor { shape, .. } = input.dtype() {
                        let mut final_shape = Vec::new();
                        // Generic dimension extraction from debug print or direct parse
                        for (i, dim) in shape.iter().enumerate() {
                            let dim_str = format!("{:?}", dim);

                            // 1. Try direct parse (e.g. "8", "128")
                            if let Ok(n) = dim_str.parse::<usize>() {
                                final_shape.push(n);
                            }
                            // 2. Try extract from Fixed(N) or Some(N)
                            else if dim_str.contains("Fixed") || dim_str.contains("Some(") {
                                let n = dim_str
                                    .chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .collect::<String>()
                                    .parse::<usize>()
                                    .unwrap_or(0);
                                final_shape.push(n);
                            }
                            // 3. Handle Dynamic/Batch
                            else if i == 0 {
                                final_shape.push(batch_size);
                            } else {
                                final_shape.push(0); // Dynamic seq_len_past (e.g. initial pass)
                            }
                        }

                        let dummy = ArrayD::<f32>::zeros(final_shape);
                        inputs.push((name.to_string(), Value::from_array(dummy)?.into()));
                    }
                }
                _ => {}
            }
        }

        let outputs = session_guard.run(inputs)?;
        let output_tensor = &outputs[0];
        let (shape, data) = output_tensor.try_extract_tensor::<f32>()?;

        self.process_outputs(batch_size, shape, data, &attention_mask_clone)
    }

    /// Internal helper to process raw tensor outputs into normalized vectors
    fn process_outputs(
        &self,
        batch_size: usize,
        shape: &[i64],
        data: &[f32],
        attention_mask: &Array2<i64>,
    ) -> Result<Vec<Vec<f64>>> {
        let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
        let embeddings_tensor = ArrayViewD::from_shape(shape_usize, data)
            .map_err(|e| anyhow::anyhow!("Failed to create view from tensor: {e}"))?;

        let mut final_vectors = Vec::with_capacity(batch_size);

        if embeddings_tensor.ndim() == 3 {
            final_vectors =
                self.mean_pooling(&embeddings_tensor.view(), &attention_mask.view().into_dyn());
        } else if embeddings_tensor.ndim() == 2 {
            for i in 0..batch_size {
                let mut vec = Vec::with_capacity(self.dimension);
                for k in 0..embeddings_tensor.shape()[1] {
                    vec.push(f64::from(embeddings_tensor[[i, k]]));
                }
                final_vectors.push(vec);
            }
        } else {
            return Err(anyhow::anyhow!(
                "Unexpected output dimension: {}",
                embeddings_tensor.ndim()
            ));
        }

        for vec in &mut final_vectors {
            self.normalize(vec);
        }

        Ok(final_vectors)
    }
}

// --- Remote API Vectorizer ---

pub struct RemoteVectorizer {
    client: Client,
    provider: ApiProvider,
    api_key: String,
    model: String,
    base_url: Option<String>,
}

impl RemoteVectorizer {
    #[must_use]
    pub fn new(
        provider: ApiProvider,
        api_key: String,
        model: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            provider,
            api_key,
            model,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    input: Vec<String>,
    model: String,
}
#[derive(Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIEmbedding>,
}
#[derive(Deserialize)]
struct OpenAIEmbedding {
    embedding: Vec<f64>,
}

#[derive(Serialize)]
struct CohereRequest {
    texts: Vec<String>,
    model: String,
    input_type: String,
}
#[derive(Deserialize)]
struct CohereResponse {
    embeddings: Vec<Vec<f64>>,
}

#[derive(Serialize)]
struct VoyageRequest {
    input: Vec<String>,
    model: String,
}
#[derive(Deserialize)]
struct VoyageResponse {
    data: Vec<VoyageEmbedding>,
}
#[derive(Deserialize)]
struct VoyageEmbedding {
    embedding: Vec<f64>,
}

#[derive(Serialize)]
struct MistralRequest {
    input: Vec<String>,
    model: String,
}
#[derive(Deserialize)]
struct MistralResponse {
    data: Vec<MistralEmbedding>,
}
#[derive(Deserialize)]
struct MistralEmbedding {
    embedding: Vec<f64>,
}

#[async_trait]
impl Vectorizer for RemoteVectorizer {
    fn dimension(&self) -> usize {
        0
    }

    async fn vectorize(&self, texts: Vec<String>) -> Result<Vec<Vec<f64>>> {
        match self.provider {
            ApiProvider::OpenAI | ApiProvider::OpenRouter | ApiProvider::Generic => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| match self.provider {
                        ApiProvider::OpenRouter => {
                            "https://openrouter.ai/api/v1/embeddings".to_string()
                        }
                        _ => "https://api.openai.com/v1/embeddings".to_string(),
                    });

                let req = OpenAIRequest {
                    input: texts,
                    model: self.model.clone(),
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;

                let body: OpenAIResponse = res.json().await?;
                Ok(body.data.into_iter().map(|d| d.embedding).collect())
            }
            ApiProvider::Mistral => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.mistral.ai/v1/embeddings".to_string());
                let req = MistralRequest {
                    input: texts,
                    model: self.model.clone(),
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;
                let body: MistralResponse = res.json().await?;
                Ok(body.data.into_iter().map(|d| d.embedding).collect())
            }
            ApiProvider::Voyage => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.voyageai.com/v1/embeddings".to_string());
                let req = VoyageRequest {
                    input: texts,
                    model: self.model.clone(),
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;
                let body: VoyageResponse = res.json().await?;
                Ok(body.data.into_iter().map(|d| d.embedding).collect())
            }
            ApiProvider::Cohere => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://api.cohere.ai/v1/embed".to_string());
                let req = CohereRequest {
                    texts,
                    model: self.model.clone(),
                    input_type: "search_document".to_string(),
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("accept", "application/json")
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;
                let body: CohereResponse = res.json().await?;
                Ok(body.embeddings)
            }
            ApiProvider::Gemini => {
                let _url = self.base_url.clone().unwrap_or_else(|| {
                   format!("https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}", self.model, self.api_key)
                });
                Err(anyhow::anyhow!(
                    "Gemini embedding not yet implemented (use Generic if compatible)"
                ))
            }
        }
    }
}
