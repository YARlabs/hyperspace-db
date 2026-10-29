//! `hs-embed` (`embed`): Unified In-Engine Semantic Chunker, ONNX/HF/Remote Model Driver & Multi-Manifold Embedder.
//!
//! Unifies all HyperspaceDB embedding capabilities inside the `hs-embed` PDK plugin:
//! 1. **Production Model Inference (`MultiVectorizer`)**:
//!    - Local ONNX Runtime (`OnnxVectorizer` via `ort` + `tokenizers`).
//!    - Automatic HuggingFace Hub model downloading & caching (`hf-hub`).
//!    - Remote API providers (`OpenAI`, `Cohere`, `Voyage`, `Mistral`, `Gemini`, `OpenRouter`, `YarInk`, `Generic`) with MRL dimension hints.
//!    - Per-metric `.env` routing (`HYPERSPACE_EMBED`, `HS_EMBED_<METRIC>_*`).
//! 2. **Zero-Config Deterministic Manifold Vectorizer (`embed_for_manifold`)**:
//!    - Multi-scale token/n-gram feature hashing with Fast Walsh-Hadamard mixing (`turbo_rot`),
//!      projected directly onto any of the 12 supported metric manifolds (`l2`, `cosine`, `dot`,
//!      `poincare`, `lorentz`, `hybrid`, `l1`, `linf`, `angular`, `jaccard`, `hamming`, `wasserstein`).
//! 3. **Markdown Structural Chunker & KAG Graph Linker**:
//!    - Splits Markdown/text preserving heading breadcrumbs, stores text in `Sidecar Payload`,
//!      and links `NEXT_CHUNK` and `CONTAINS` edges in `GraphEngine`.

#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::items_after_statements)]
#![allow(unknown_lints)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::needless_bool)]

use crate::{EngineContext, HookStage, HsPlugin, InsertPoint, PluginManifest, ProcedureSchema};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use hyperspace_core::Durability;
use ndarray::{Array, Array2, ArrayD, ArrayViewD};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value as OrtValue,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokenizers::Tokenizer;

/// Relation ID for sequential chunk progression (`chunk_i -> chunk_{i+1}`).
pub const REL_NEXT_CHUNK: u16 = 1;
/// Relation ID for document/section containment (`section_root -> chunk_i`).
pub const REL_CONTAINS_CHUNK: u16 = 2;

// ============================================================================
// Core Embedding Engine Types (ONNX + HuggingFace Hub + Remote APIs)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    Poincare,
    Lorentz,
    L2,
    Cosine,
    Hybrid,
    None,
}

/// Chunking configuration for long document processing configured via `.env`.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub chunk_size: usize,
    pub overlap: f64,
}

impl ChunkingConfig {
    /// Loads per-metric chunking configuration from environment variables (`HS_EMBED_<METRIC>_CHUNK_SIZE`).
    #[must_use]
    pub fn from_env(metric: &str) -> Option<Self> {
        let chunk_size_key = format!("HS_EMBED_{}_CHUNK_SIZE", metric.to_uppercase());
        let overlap_key = format!("HS_EMBED_{}_OVERLAP", metric.to_uppercase());

        let chunk_size = std::env::var(&chunk_size_key).ok()?.parse::<usize>().ok()?;
        let overlap = std::env::var(&overlap_key)
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.10);

        Some(Self {
            chunk_size,
            overlap,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiProvider {
    OpenAI,
    Cohere,
    Voyage,
    Mistral,
    Gemini,
    OpenRouter,
    YarInk,
    Generic,
}

impl FromStr for ApiProvider {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(Self::OpenAI),
            "cohere" => Ok(Self::Cohere),
            "voyage" => Ok(Self::Voyage),
            "mistral" => Ok(Self::Mistral),
            "gemini" => Ok(Self::Gemini),
            "openrouter" => Ok(Self::OpenRouter),
            "yarink" => Ok(Self::YarInk),
            "generic" => Ok(Self::Generic),
            _ => Err(()),
        }
    }
}

#[async_trait]
pub trait Vectorizer: Send + Sync {
    async fn vectorize(&self, texts: Vec<String>) -> Result<Vec<Vec<f64>>>;

    /// Vectorize with a target dimension hint taken from the collection schema.
    async fn vectorize_with_dim(&self, texts: Vec<String>, dim: usize) -> Result<Vec<Vec<f64>>> {
        let mut vecs = self.vectorize(texts).await?;
        for v in &mut vecs {
            v.truncate(dim);
        }
        Ok(vecs)
    }

    fn dimension(&self) -> usize;
}

/// Multi-metric vectorizer routing each geometry (`l2`, `cosine`, `poincare`, `lorentz`, `hybrid`)
/// to its configured ONNX, HuggingFace Hub, or Remote API backend.
pub struct MultiVectorizer {
    pub models: HashMap<String, Arc<dyn Vectorizer>>,
}

impl MultiVectorizer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    pub fn add(&mut self, metric: &str, vectorizer: Arc<dyn Vectorizer>) {
        self.models.insert(metric.to_string(), vectorizer);
    }

    /// Initializes `MultiVectorizer` from `.env` / process environment variables
    /// (`HYPERSPACE_EMBED=true` and `HS_EMBED_<METRIC>_*`).
    #[must_use]
    pub fn from_env() -> Option<Arc<Self>> {
        let enabled_raw = std::env::var("HYPERSPACE_EMBED").unwrap_or_else(|_| "false".to_string());
        let enabled = enabled_raw.eq_ignore_ascii_case("true");
        if !enabled {
            return None;
        }

        let mut multi = Self::new();
        for metric_name in ["l2", "cosine", "poincare", "lorentz", "hybrid"] {
            let metric_upper = metric_name.to_uppercase();
            let provider_key = format!("HS_EMBED_{metric_upper}_PROVIDER");
            let provider_str = std::env::var(&provider_key)
                .or_else(|_| std::env::var("HYPERSPACE_EMBED_PROVIDER"))
                .unwrap_or_else(|_| "disabled".to_string())
                .to_lowercase();

            if provider_str == "disabled" || provider_str == "none" {
                continue;
            }

            let metric = match metric_name {
                "poincare" => Metric::Poincare,
                "lorentz" => Metric::Lorentz,
                "l2" => Metric::L2,
                "hybrid" => Metric::Hybrid,
                _ => Metric::Cosine,
            };

            if provider_str == "local" {
                let model_path = std::env::var(format!("HS_EMBED_{metric_upper}_MODEL_PATH"))
                    .or_else(|_| std::env::var("HYPERSPACE_MODEL_PATH"))
                    .ok();
                let tok_path = std::env::var(format!("HS_EMBED_{metric_upper}_TOKENIZER_PATH"))
                    .or_else(|_| std::env::var("HYPERSPACE_TOKENIZER_PATH"))
                    .ok();

                if let (Some(m), Some(t)) = (model_path, tok_path) {
                    let dim: usize = std::env::var(format!("HS_EMBED_{metric_upper}_DIM"))
                        .or_else(|_| std::env::var("HYPERSPACE_EMBED_DIM"))
                        .unwrap_or_else(|_| "128".to_string())
                        .parse()
                        .unwrap_or(128);
                    if let Ok(v) = OnnxVectorizer::new(&m, &t, dim, metric, &metric_upper) {
                        multi.add(metric_name, Arc::new(v));
                    }
                }
            } else if provider_str == "huggingface" || provider_str == "hf" {
                let hf_model_id = std::env::var(format!("HS_EMBED_{metric_upper}_HF_MODEL_ID"))
                    .or_else(|_| std::env::var("HYPERSPACE_HF_MODEL_ID"))
                    .ok();
                if let Some(model_id) = hf_model_id {
                    let hf_token = std::env::var("HF_TOKEN")
                        .or_else(|_| std::env::var("HUGGING_FACE_HUB_TOKEN"))
                        .ok();
                    let hf_filename =
                        std::env::var(format!("HS_EMBED_{metric_upper}_HF_FILENAME")).ok();
                    let dim: usize = std::env::var(format!("HS_EMBED_{metric_upper}_DIM"))
                        .or_else(|_| std::env::var("HYPERSPACE_EMBED_DIM"))
                        .unwrap_or_else(|_| "128".to_string())
                        .parse()
                        .unwrap_or(128);
                    if let Ok(v) = OnnxVectorizer::new_from_hf(
                        &model_id,
                        hf_token.as_deref(),
                        dim,
                        metric,
                        &metric_upper,
                        hf_filename,
                    ) {
                        multi.add(metric_name, Arc::new(v));
                    }
                }
            } else if let Ok(provider) = ApiProvider::from_str(&provider_str) {
                let api_key = std::env::var(format!("HS_EMBED_{metric_upper}_API_KEY"))
                    .or_else(|_| std::env::var("HYPERSPACE_API_KEY_EMBED"))
                    .or_else(|_| std::env::var("OPENAI_API_KEY"))
                    .unwrap_or_default();
                let model = std::env::var(format!("HS_EMBED_{metric_upper}_EMBED_MODEL"))
                    .or_else(|_| std::env::var("HYPERSPACE_EMBED_MODEL"))
                    .unwrap_or_else(|_| "text-embedding-3-small".to_string());
                let base_url = std::env::var(format!("HS_EMBED_{metric_upper}_API_BASE"))
                    .or_else(|_| std::env::var("HYPERSPACE_API_BASE"))
                    .ok();
                multi.add(
                    metric_name,
                    Arc::new(RemoteVectorizer::new(provider, api_key, model, base_url)),
                );
            }
        }

        if multi.models.is_empty() {
            None
        } else {
            Some(Arc::new(multi))
        }
    }

    pub async fn vectorize_for(&self, texts: Vec<String>, metric: &str) -> Result<Vec<Vec<f64>>> {
        let metric_key = Self::normalize_metric(metric);

        if let Some(v) = self.models.get(&metric_key) {
            v.vectorize(texts).await
        } else if let Some(v) = self
            .models
            .get("l2")
            .or_else(|| self.models.values().next())
        {
            v.vectorize(texts).await
        } else {
            Err(anyhow!("No vectorizer available"))
        }
    }

    pub async fn vectorize_for_dim(
        &self,
        texts: Vec<String>,
        metric: &str,
        dim: usize,
    ) -> Result<Vec<Vec<f64>>> {
        let metric_key = Self::normalize_metric(metric);

        if let Some(v) = self.models.get(&metric_key) {
            v.vectorize_with_dim(texts, dim).await
        } else if let Some(v) = self
            .models
            .get("l2")
            .or_else(|| self.models.values().next())
        {
            v.vectorize_with_dim(texts, dim).await
        } else {
            Err(anyhow!("No vectorizer available"))
        }
    }

    fn normalize_metric(metric: &str) -> String {
        match metric.to_lowercase().as_str() {
            "l2" | "euclidean" => "l2".to_string(),
            "cosine" => "cosine".to_string(),
            "poincare" => "poincare".to_string(),
            "lorentz" => "lorentz".to_string(),
            "hybrid" => "hybrid".to_string(),
            other => other.to_string(),
        }
    }
}

impl Default for MultiVectorizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Local ONNX Runtime vectorizer (`ort` + HuggingFace `tokenizers`).
pub struct OnnxVectorizer {
    tokenizer: Tokenizer,
    session: Mutex<Session>,
    dimension: usize,
    metric: Metric,
    chunking_config: Option<ChunkingConfig>,
    #[allow(dead_code)]
    model_id: String,
}

impl OnnxVectorizer {
    pub fn new(
        model_path: &str,
        tokenizer_path: &str,
        dimension: usize,
        metric: Metric,
        metric_name: &str,
    ) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {e}"))?;

        let session = Session::builder()
            .map_err(|e| anyhow!("Ort session builder failed: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow!("Ort optimization failure: {e}"))?
            .with_intra_threads(4)
            .map_err(|e| anyhow!("Ort thread configuration failure: {e}"))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow!("Ort session commit failed for path {model_path}: {e}"))?;

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

    pub fn new_from_hf(
        model_id: &str,
        hf_token: Option<&str>,
        dimension: usize,
        metric: Metric,
        metric_name: &str,
        model_file: Option<String>,
    ) -> Result<Self> {
        use hf_hub::api::sync::{ApiBuilder, ApiRepo};
        use std::path::PathBuf;

        let mut builder = ApiBuilder::new().with_progress(false);
        if let Some(token) = hf_token {
            if !token.is_empty() {
                builder = builder.with_token(Some(token.to_string()));
            }
        }

        let cache_dir = PathBuf::from(std::env::var("HF_HOME").unwrap_or_else(|_| {
            std::env::var("HOME").map_or_else(
                |_| "./.hf_cache".to_string(),
                |h| format!("{h}/.cache/huggingface"),
            )
        }));
        let model_path_cached = cache_dir.join("hub").join(model_id.replace('/', "--"));
        let model_onnx_path = model_path_cached.join("model.onnx");
        let tokenizer_path_cached = model_path_cached.join("tokenizer.json");

        let is_cached = model_onnx_path.exists() && tokenizer_path_cached.exists();
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

        let api = builder.build().map_err(|e| {
            eprintln!("❌ HF API error for {model_id}: {e}");
            anyhow!("HF API error: {e}")
        })?;
        let repo: ApiRepo = api.model(model_id.to_string());

        let filename = model_file.unwrap_or_else(|| "model.onnx".to_string());
        let model_path = repo.get(&filename).map_err(|e| {
            eprintln!("❌ Failed to download {filename} for {model_id}: {e}");
            anyhow!("Failed to download {filename}: {e}")
        })?;

        for suffix in &[".data", "_data"] {
            let data_filename = format!("{filename}{suffix}");
            let _ = repo.get(&data_filename);
        }
        let tokenizer_path = repo.get("tokenizer.json").map_err(|e| {
            eprintln!("❌ Failed to download tokenizer.json for {model_id}: {e}");
            anyhow!("Failed to download tokenizer.json: {e}")
        })?;

        let tokenizer = Tokenizer::from_file(
            tokenizer_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid tokenizer path"))?,
        )
        .map_err(|e| {
            eprintln!("❌ Failed to parse tokenizer.json for {model_id}: {e}");
            anyhow!("Failed to load tokenizer: {e}")
        })?;

        let session = Session::builder()
            .map_err(|e| {
                eprintln!("❌ Ort session builder failed: {e}");
                anyhow!("Ort session builder failed: {e}")
            })?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow!("Ort optimization failure: {e}"))?
            .with_intra_threads(4)
            .map_err(|e| anyhow!("Ort thread configuration failure: {e}"))?
            .commit_from_file(
                model_path
                    .to_str()
                    .ok_or_else(|| anyhow!("Invalid model path"))?,
            )
            .map_err(|e| {
                eprintln!(
                    "❌ Ort session commit failed for {model_id} path {}: {e}",
                    model_path.display()
                );
                anyhow!("Ort session commit failed: {e}")
            })?;

        let chunking_config = ChunkingConfig::from_env(metric_name);
        let chunk_info = chunking_config.as_ref().map_or_else(
            || "no chunking".to_string(),
            |c| {
                format!(
                    "chunk={} tokens, overlap={:.0}%",
                    c.chunk_size,
                    c.overlap * 100.0
                )
            },
        );

        eprintln!("🚀 Model activated: {model_id} ({dimension}d, {chunk_info}, metric={metric:?})");

        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
            dimension,
            metric,
            chunking_config,
            model_id: model_id.to_string(),
        })
    }

    pub fn normalize(&self, vec: &mut Vec<f64>) {
        const EPSILON: f64 = 1e-12;
        let mut norm_sq: f64 = vec.iter().map(|x| x * x).sum();
        let mut norm = norm_sq.sqrt();

        match self.metric {
            Metric::Poincare => {
                if vec.len() == self.dimension + 1 {
                    let x0 = vec[0];
                    let denom = (1.0 + x0).max(EPSILON);
                    let mut projected = Vec::with_capacity(self.dimension);
                    for &x_val in vec.iter().skip(1) {
                        projected.push(x_val / denom);
                    }
                    *vec = projected;
                    norm_sq = vec.iter().map(|x| x * x).sum();
                    norm = norm_sq.sqrt();
                }

                if norm >= 1.0 - EPSILON {
                    let scale = (1.0 - EPSILON) / (norm + EPSILON);
                    for x in vec.iter_mut() {
                        *x *= scale;
                    }
                }

                for x in vec.iter_mut() {
                    if x.is_nan() || x.is_infinite() {
                        *x = 0.0;
                    }
                }
            }
            Metric::Lorentz => {
                if vec.is_empty() {
                    return;
                }
                if vec.len() == self.dimension - 1 {
                    let spatial_norm_sq = norm_sq;
                    let x0 = (1.0 + spatial_norm_sq).sqrt();
                    vec.insert(0, x0);
                } else if vec.len() == self.dimension {
                    let spatial_norm_sq: f64 = vec[1..].iter().map(|x| x * x).sum();
                    vec[0] = (1.0 + spatial_norm_sq).sqrt();
                }

                for x in vec.iter_mut() {
                    if x.is_nan() || x.is_infinite() {
                        *x = 0.0;
                    }
                }
            }
            Metric::Hybrid => {
                let lorentz_dim = 33;
                if vec.len() >= lorentz_dim {
                    let spatial_norm_sq: f64 = vec[1..lorentz_dim].iter().map(|x| x * x).sum();
                    vec[0] = (1.0 + spatial_norm_sq).sqrt();

                    for x in vec.iter_mut().take(lorentz_dim) {
                        if x.is_nan() || x.is_infinite() {
                            *x = 0.0;
                        }
                    }

                    let euclidean_part = &mut vec[lorentz_dim..];
                    let euc_norm_sq: f64 = euclidean_part.iter().map(|x| x * x).sum();
                    let euc_norm = euc_norm_sq.sqrt();
                    if euc_norm > 0.0 {
                        for x in euclidean_part.iter_mut() {
                            *x /= euc_norm;
                        }
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

    fn split_into_chunks(text: &str, config: &ChunkingConfig) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let chunk_size = config.chunk_size;
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let overlap_size = (chunk_size as f64 * config.overlap) as usize;
        let step_size = chunk_size.saturating_sub(overlap_size);

        if words.len() <= chunk_size {
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

    fn aggregate_embeddings(chunk_embeddings: &[Vec<f64>]) -> Vec<f64> {
        if chunk_embeddings.is_empty() {
            return vec![];
        }

        let dim = chunk_embeddings[0].len();
        let mut aggregated = vec![0.0; dim];

        for emb in chunk_embeddings {
            for (i, &v) in emb.iter().enumerate() {
                aggregated[i] += v;
            }
        }

        #[allow(clippy::cast_precision_loss)]
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

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn vectorize_direct(&self, texts: &[String]) -> Result<Vec<Vec<f64>>> {
        if texts.is_empty() {
            return Ok(vec![]);
        }

        let encoding = self
            .tokenizer
            .encode_batch(texts.to_owned(), true)
            .map_err(|e| anyhow!("Tokenization failed: {e}"))?;

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

        let mut session_guard = self
            .session
            .lock()
            .map_err(|_| anyhow!("Session lock poisoned"))?;

        let mut inputs: Vec<(String, OrtValue)> = Vec::new();
        for input in session_guard.inputs() {
            let name = input.name();
            match name {
                "input_ids" => {
                    inputs.push((
                        name.to_string(),
                        OrtValue::from_array(input_ids_arr.clone())?.into(),
                    ));
                }
                "attention_mask" => {
                    inputs.push((
                        name.to_string(),
                        OrtValue::from_array(attention_mask_arr.clone())?.into(),
                    ));
                }
                "position_ids" => {
                    #[allow(clippy::cast_possible_wrap)]
                    let position_ids_arr =
                        Array::from_shape_fn((batch_size, seq_len), |(_, c)| c as i64);
                    inputs.push((
                        name.to_string(),
                        OrtValue::from_array(position_ids_arr)?.into(),
                    ));
                }
                name if name.contains("past_key_values") => {
                    if let ort::value::ValueType::Tensor { shape, .. } = input.dtype() {
                        let mut final_shape = Vec::new();
                        for (i, dim) in shape.iter().enumerate() {
                            let dim_str = format!("{dim:?}");
                            if let Ok(n) = dim_str.parse::<usize>() {
                                final_shape.push(n);
                            } else if dim_str.contains("Fixed") || dim_str.contains("Some(") {
                                let n = dim_str
                                    .chars()
                                    .filter(char::is_ascii_digit)
                                    .collect::<String>()
                                    .parse::<usize>()
                                    .unwrap_or(0);
                                final_shape.push(n);
                            } else if i == 0 {
                                final_shape.push(batch_size);
                            } else {
                                final_shape.push(0);
                            }
                        }

                        let dummy = ArrayD::<f32>::zeros(final_shape);
                        inputs.push((name.to_string(), OrtValue::from_array(dummy)?.into()));
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

    fn process_outputs(
        &self,
        batch_size: usize,
        shape: &[i64],
        data: &[f32],
        attention_mask: &Array2<i64>,
    ) -> Result<Vec<Vec<f64>>> {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
        let embeddings_tensor = ArrayViewD::from_shape(shape_usize, data)
            .map_err(|e| anyhow!("Failed to create view from tensor: {e}"))?;

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
            return Err(anyhow!(
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

        if let Some(config) = &self.chunking_config {
            let texts_len = texts.len();
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

            let batch_size = 16;
            let mut chunk_embeddings: Vec<Vec<Vec<f64>>> = vec![Vec::new(); texts_len];

            for chunk_batch in all_chunks.chunks(batch_size) {
                let chunk_texts: Vec<String> = chunk_batch.iter().map(|(_, c)| c.clone()).collect();
                let embeddings = self.vectorize_direct(&chunk_texts)?;

                for ((text_idx, _), embedding) in chunk_batch.iter().zip(embeddings) {
                    chunk_embeddings[*text_idx].push(embedding);
                }
            }

            let final_embeddings = chunk_embeddings
                .into_iter()
                .map(|chunks| {
                    if chunks.is_empty() {
                        vec![0.0; self.dimension]
                    } else {
                        OnnxVectorizer::aggregate_embeddings(&chunks)
                    }
                })
                .collect();

            return Ok(final_embeddings);
        }

        self.vectorize_direct(&texts)
    }

    async fn vectorize_with_dim(&self, texts: Vec<String>, dim: usize) -> Result<Vec<Vec<f64>>> {
        let mut vecs = self.vectorize(texts).await?;
        for v in &mut vecs {
            v.truncate(dim);
            self.normalize(v);
        }
        Ok(vecs)
    }
}

/// Remote API vectorizer (`OpenAI`, `Cohere`, `Voyage`, `Mistral`, `OpenRouter`, `YarInk`, `Generic`).
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
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimension: Option<u32>,
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

    async fn vectorize_with_dim(&self, texts: Vec<String>, dim: usize) -> Result<Vec<Vec<f64>>> {
        #[allow(clippy::cast_possible_truncation)]
        let dim_u32 = dim as u32;
        match self.provider {
            ApiProvider::OpenAI
            | ApiProvider::OpenRouter
            | ApiProvider::Generic
            | ApiProvider::YarInk => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| match self.provider {
                        ApiProvider::OpenRouter => {
                            "https://openrouter.ai/api/v1/embeddings".to_string()
                        }
                        ApiProvider::YarInk => "https://the.yar.ink/v1/embeddings".to_string(),
                        _ => "https://api.openai.com/v1/embeddings".to_string(),
                    });
                let req = OpenAIRequest {
                    input: texts,
                    model: self.model.clone(),
                    dimensions: Some(dim_u32),
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .header("User-Agent", "hyperspace-pdk/4.2.0")
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;
                let body: OpenAIResponse = res.json().await?;
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
                    output_dimension: Some(dim_u32),
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
            _ => {
                let mut vecs = self.vectorize(texts).await?;
                for v in &mut vecs {
                    v.truncate(dim);
                }
                Ok(vecs)
            }
        }
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
                    dimensions: None,
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
                    output_dimension: None,
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
                    format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:embedContent?key={}",
                        self.model, self.api_key
                    )
                });
                Err(anyhow!(
                    "Gemini embedding not yet implemented (use Generic if compatible)"
                ))
            }
            ApiProvider::YarInk => {
                let url = self
                    .base_url
                    .clone()
                    .unwrap_or_else(|| "https://the.yar.ink/v1/embeddings".to_string());

                let req = OpenAIRequest {
                    input: texts,
                    model: self.model.clone(),
                    dimensions: None,
                };
                let res = self
                    .client
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", self.api_key))
                    .header("Content-Type", "application/json")
                    .header("User-Agent", "hyperspace-pdk/4.2.0")
                    .json(&req)
                    .send()
                    .await?
                    .error_for_status()?;

                let body: OpenAIResponse = res.json().await?;
                Ok(body.data.into_iter().map(|d| d.embedding).collect())
            }
        }
    }
}

// ============================================================================
// `hs-embed` PDK Plugin Implementation
// ============================================================================

/// A single extracted semantic text chunk with structural provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextChunk {
    pub index: usize,
    pub heading: Option<String>,
    pub text: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct IngestTextArgs {
    pub text: String,
    #[serde(default)]
    pub base_id: Option<u32>,
    #[serde(default)]
    pub chunk_size: Option<usize>,
    #[serde(default)]
    pub chunk_overlap: Option<usize>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default = "default_true")]
    pub link_sequential: bool,
    #[serde(default = "default_true")]
    pub link_hierarchy: bool,
}

const fn default_true() -> bool {
    true
}

/// Official `hs-embed` plugin implementation unifying ONNX/HF/Remote `MultiVectorizer`
/// and zero-config deterministic manifold embeddings.
#[derive(Default)]
pub struct HsEmbedPlugin {
    vectorizer: parking_lot::RwLock<Option<Arc<MultiVectorizer>>>,
}

impl std::fmt::Debug for HsEmbedPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HsEmbedPlugin")
            .field("has_vectorizer", &self.vectorizer.read().is_some())
            .finish()
    }
}

impl HsEmbedPlugin {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vectorizer: parking_lot::RwLock::new(None),
        }
    }

    #[must_use]
    pub fn with_vectorizer(vectorizer: Option<Arc<MultiVectorizer>>) -> Self {
        Self {
            vectorizer: parking_lot::RwLock::new(vectorizer),
        }
    }

    pub fn set_vectorizer(&self, vectorizer: Option<Arc<MultiVectorizer>>) {
        *self.vectorizer.write() = vectorizer;
    }

    #[must_use]
    pub fn vectorizer(&self) -> Option<Arc<MultiVectorizer>> {
        self.vectorizer.read().clone()
    }

    /// Embeds a single text using the configured `MultiVectorizer` (ONNX / HF / Remote API)
    /// when active for `metric`, or falls back to the deterministic manifold vectorizer.
    pub async fn embed_text_smart(&self, text: &str, dimension: usize, metric: &str) -> Vec<f64> {
        let multi_opt = self.vectorizer.read().clone();
        if let Some(multi) = multi_opt {
            if let Ok(mut vecs) = multi
                .vectorize_for_dim(vec![text.to_string()], metric, dimension)
                .await
            {
                if let Some(mut v) = vecs.pop() {
                    if v.len() == dimension {
                        return v;
                    } else if v.len() > dimension {
                        v.truncate(dimension);
                        return v;
                    }
                }
            }
        }
        Self::embed_for_manifold(text, dimension, metric)
    }

    /// Splits raw text or Markdown into semantic chunks, preserving Markdown headings.
    #[must_use]
    pub fn chunk_markdown(
        text: &str,
        max_chars: usize,
        overlap_chars: usize,
    ) -> Vec<TextChunk> {
        let max_chars = max_chars.max(64);
        let overlap_chars = overlap_chars.min(max_chars / 2);

        let mut chunks = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut buffer = String::new();
        let mut buffer_start = 0usize;
        let mut cursor = 0usize;

        let flush_buffer = |buf: &mut String,
                            start: usize,
                            end: usize,
                            heading: &Option<String>,
                            out: &mut Vec<TextChunk>| {
            let trimmed = buf.trim();
            if trimmed.is_empty() {
                buf.clear();
                return;
            }
            if trimmed.len() <= max_chars {
                let idx = out.len();
                out.push(TextChunk {
                    index: idx,
                    heading: heading.clone(),
                    text: trimmed.to_string(),
                    char_start: start,
                    char_end: end,
                });
                buf.clear();
                return;
            }

            let words: Vec<&str> = trimmed.split_whitespace().collect();
            let mut w_idx = 0usize;
            while w_idx < words.len() {
                let mut slice = String::new();
                let mut end_w = w_idx;
                while end_w < words.len() {
                    let candidate_len = if slice.is_empty() {
                        words[end_w].len()
                    } else {
                        slice.len() + 1 + words[end_w].len()
                    };
                    if candidate_len > max_chars && !slice.is_empty() {
                        break;
                    }
                    if !slice.is_empty() {
                        slice.push(' ');
                    }
                    slice.push_str(words[end_w]);
                    end_w += 1;
                }
                let idx = out.len();
                out.push(TextChunk {
                    index: idx,
                    heading: heading.clone(),
                    text: slice,
                    char_start: start,
                    char_end: end,
                });
                if end_w >= words.len() {
                    break;
                }
                let mut overlap_len = 0usize;
                let mut back = end_w;
                while back > w_idx + 1 && overlap_len < overlap_chars {
                    back -= 1;
                    overlap_len += words[back].len() + 1;
                }
                w_idx = back.max(w_idx + 1);
            }
            buf.clear();
        };

        for line in text.split_inclusive('\n') {
            let line_len = line.len();
            let trimmed_line = line.trim();
            if trimmed_line.starts_with('#') {
                flush_buffer(
                    &mut buffer,
                    buffer_start,
                    cursor,
                    &current_heading,
                    &mut chunks,
                );
                let heading_text = trimmed_line.trim_start_matches('#').trim().to_string();
                if !heading_text.is_empty() {
                    current_heading = Some(heading_text);
                }
                buffer_start = cursor;
            }
            if buffer.is_empty() {
                buffer_start = cursor;
            }
            buffer.push_str(line);
            cursor += line_len;

            if buffer.len() >= max_chars * 2 {
                flush_buffer(
                    &mut buffer,
                    buffer_start,
                    cursor,
                    &current_heading,
                    &mut chunks,
                );
                buffer_start = cursor;
            }
        }

        flush_buffer(
            &mut buffer,
            buffer_start,
            cursor,
            &current_heading,
            &mut chunks,
        );

        chunks
    }

    /// Generates a deterministic, geometry-aware embedding vector in pure Rust.
    #[must_use]
    pub fn embed_for_manifold(text: &str, dimension: usize, metric: &str) -> Vec<f64> {
        let dim = dimension.max(1);
        let mut raw = vec![0.0f32; dim];

        let normalized = text.to_lowercase();
        let tokens: Vec<&str> = normalized
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| !s.is_empty())
            .collect();

        if tokens.is_empty() {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let digest = hasher.finalize();
            for i in 0..dim {
                let b = digest[i % digest.len()];
                raw[i] = (f32::from(b) - 127.5) / 127.5;
            }
        } else {
            for (pos, &tok) in tokens.iter().enumerate() {
                Self::accumulate_feature(&mut raw, tok.as_bytes(), 1.0);
                if pos + 1 < tokens.len() {
                    let mut bigram = String::with_capacity(tok.len() + 1 + tokens[pos + 1].len());
                    bigram.push_str(tok);
                    bigram.push(' ');
                    bigram.push_str(tokens[pos + 1]);
                    Self::accumulate_feature(&mut raw, bigram.as_bytes(), 0.75);
                }
                let bytes = tok.as_bytes();
                if bytes.len() >= 3 {
                    for window in bytes.windows(3) {
                        Self::accumulate_feature(&mut raw, window, 0.35);
                    }
                }
            }
        }

        hyperspace_core::turbo_rot::rotate_vector_in_place(&mut raw);

        let metric_lower = metric.trim().to_ascii_lowercase();
        match metric_lower.as_str() {
            "poincare" => {
                let norm_sq: f64 = raw.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
                let norm = norm_sq.sqrt().max(1e-9);
                let target_radius = 0.75;
                raw.iter()
                    .map(|&x| (f64::from(x) / norm) * target_radius)
                    .collect()
            }
            "lorentz" => {
                let mut out = vec![0.0f64; dim];
                if dim == 1 {
                    out[0] = 1.0;
                    return out;
                }
                let spatial_norm_sq: f64 =
                    raw[1..].iter().map(|&x| f64::from(x) * f64::from(x)).sum();
                let scale = if spatial_norm_sq > 1e-12 {
                    1.2 / spatial_norm_sq.sqrt()
                } else {
                    1.0
                };
                let mut scaled_spatial_sq = 0.0f64;
                for i in 1..dim {
                    let v = f64::from(raw[i]) * scale;
                    out[i] = v;
                    scaled_spatial_sq += v * v;
                }
                out[0] = (1.0 + scaled_spatial_sq).sqrt();
                out
            }
            "hybrid" if dim >= 33 => {
                let mut out = vec![0.0f64; dim];
                let head_spatial_sq: f64 = raw[1..33]
                    .iter()
                    .map(|&x| f64::from(x) * f64::from(x))
                    .sum();
                let head_scale = if head_spatial_sq > 1e-12 {
                    1.0 / head_spatial_sq.sqrt()
                } else {
                    1.0
                };
                let mut actual_head_sq = 0.0f64;
                for i in 1..33 {
                    let v = f64::from(raw[i]) * head_scale;
                    out[i] = v;
                    actual_head_sq += v * v;
                }
                out[0] = (1.0 + actual_head_sq).sqrt();

                let tail_norm_sq: f64 = raw[33..]
                    .iter()
                    .map(|&x| f64::from(x) * f64::from(x))
                    .sum();
                let tail_norm = tail_norm_sq.sqrt().max(1e-9);
                for i in 33..dim {
                    out[i] = f64::from(raw[i]) / tail_norm;
                }
                out
            }
            "wasserstein" | "emd" | "sinkhorn" => {
                let positives: Vec<f64> = raw.iter().map(|&x| f64::from(x).abs() + 1e-4).collect();
                let sum: f64 = positives.iter().sum::<f64>().max(1e-9);
                positives.into_iter().map(|x| x / sum).collect()
            }
            _ => {
                let norm_sq: f64 = raw.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
                let norm = norm_sq.sqrt().max(1e-9);
                raw.iter().map(|&x| f64::from(x) / norm).collect()
            }
        }
    }

    #[inline]
    fn accumulate_feature(buf: &mut [f32], feature: &[u8], weight: f32) {
        let dim = buf.len();
        let mut h1: u64 = 0xcbf2_9ce4_8422_2325;
        for &b in feature {
            h1 ^= u64::from(b);
            h1 = h1.wrapping_mul(0x0100_0000_01b3);
        }
        let h2 = h1.rotate_left(31) ^ 0x9e37_79b9_7f4a_7c15;

        let idx1 = (h1 as usize) % dim;
        let sign1 = if (h1 >> 63) == 0 { weight } else { -weight };
        buf[idx1] += sign1;

        let idx2 = (h2 as usize) % dim;
        let sign2 = if (h2 >> 63) == 0 {
            weight * 0.5
        } else {
            -weight * 0.5
        };
        buf[idx2] += sign2;
    }
}

#[async_trait]
impl HsPlugin for HsEmbedPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "embed".to_string(),
            version: "4.2.0".to_string(),
            author: "HyperspaceDB Core Team".to_string(),
            description: "Unified in-engine Markdown/text chunker, ONNX/HF/Remote model driver, multi-manifold vectorizer, and automatic KAG chunk graph linker.".to_string(),
            runtime: "native-rust".to_string(),
            hooks: vec![HookStage::BeforeInsert],
            procedures: vec![
                ProcedureSchema {
                    name: "ingest_text".to_string(),
                    description: "Chunks raw text/Markdown, embeds each chunk into the collection manifold, stores text in Sidecar Payload, and links sequential/hierarchical edges in GraphEngine.".to_string(),
                    requires_collection: true,
                    input_schema: json!({
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": { "type": "string", "description": "Raw Markdown or plain text document to chunk and ingest" },
                            "base_id": { "type": "integer", "description": "Starting vector ID (auto-generated if omitted)" },
                            "chunk_size": { "type": "integer", "default": 512 },
                            "chunk_overlap": { "type": "integer", "default": 64 },
                            "metadata": { "type": "object", "additionalProperties": { "type": "string" } },
                            "link_sequential": { "type": "boolean", "default": true },
                            "link_hierarchy": { "type": "boolean", "default": true }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "chunks_inserted": { "type": "integer" },
                            "edges_linked": { "type": "integer" },
                            "chunk_ids": { "type": "array", "items": { "type": "integer" } }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "chunk_text".to_string(),
                    description: "Splits Markdown or plain text into structural chunks without inserting into a collection.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": { "type": "string" },
                            "chunk_size": { "type": "integer", "default": 512 },
                            "chunk_overlap": { "type": "integer", "default": 64 }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "chunks": { "type": "array" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "embed_text".to_string(),
                    description: "Computes a manifold-aligned vector embedding for the given text, dimension, and metric.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "required": ["text"],
                        "properties": {
                            "text": { "type": "string" },
                            "dimension": { "type": "integer", "default": 128 },
                            "metric": { "type": "string", "default": "cosine" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "vector": { "type": "array", "items": { "type": "number" } },
                            "dimension": { "type": "integer" },
                            "metric": { "type": "string" }
                        }
                    }),
                },
            ],
            custom_metrics: vec![],
        }
    }

    async fn before_insert(
        &self,
        _collection: &str,
        metric: &str,
        dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        for pt in points.iter_mut() {
            if pt.vector.is_empty() {
                let text_opt = pt
                    .metadata
                    .get("_text")
                    .or_else(|| pt.metadata.get("text"))
                    .cloned()
                    .or_else(|| {
                        pt.payload
                            .as_ref()
                            .and_then(|b| std::str::from_utf8(b).ok())
                            .map(std::string::ToString::to_string)
                    });
                if let Some(text) = text_opt {
                    pt.vector = self.embed_text_smart(&text, dimension, metric).await;
                    if pt.payload.is_none() {
                        pt.payload = Some(text.into_bytes());
                    }
                }
            }
        }
        Ok(())
    }

    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        match method {
            "ingest_text" | "ingest_document" => {
                let parsed: IngestTextArgs = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid arguments for embed.{method}: {e}"))?;
                let col = ctx.collection()?;
                let dim = col.dimension();
                let metric = col.metric_name();

                let chunk_size = parsed.chunk_size.unwrap_or(512);
                let chunk_overlap = parsed.chunk_overlap.unwrap_or(64);
                let chunks = Self::chunk_markdown(&parsed.text, chunk_size, chunk_overlap);
                if chunks.is_empty() {
                    return Ok(json!({
                        "chunks_inserted": 0,
                        "edges_linked": 0,
                        "chunk_ids": []
                    }));
                }

                let base_id = parsed.base_id.unwrap_or_else(|| {
                    (col.count() as u32)
                        .saturating_add(1)
                        .saturating_mul(100)
                });

                let mut points = Vec::with_capacity(chunks.len());
                let mut chunk_ids = Vec::with_capacity(chunks.len());
                let mut edges = Vec::new();

                for chunk in &chunks {
                    let id = base_id.saturating_add(chunk.index as u32);
                    chunk_ids.push(id);
                    let vec = self.embed_text_smart(&chunk.text, dim, metric).await;

                    let mut meta = parsed.metadata.clone();
                    meta.insert("chunk_index".to_string(), chunk.index.to_string());
                    meta.insert("char_start".to_string(), chunk.char_start.to_string());
                    meta.insert("char_end".to_string(), chunk.char_end.to_string());
                    if let Some(ref h) = chunk.heading {
                        meta.insert("heading".to_string(), h.clone());
                    }

                    points.push(InsertPoint {
                        id,
                        vector: vec,
                        metadata: meta,
                        payload: Some(chunk.text.clone().into_bytes()),
                    });
                }

                if parsed.link_sequential && chunk_ids.len() > 1 {
                    for w in chunk_ids.windows(2) {
                        edges.push((w[0], w[1], REL_NEXT_CHUNK, 1.0f32));
                    }
                }
                if parsed.link_hierarchy && chunk_ids.len() > 1 {
                    let root_id = chunk_ids[0];
                    for &child_id in &chunk_ids[1..] {
                        edges.push((root_id, child_id, REL_CONTAINS_CHUNK, 0.8f32));
                    }
                }

                let inserted = ctx.insert_points(points, Durability::Default).await?;
                let linked = if !edges.is_empty() {
                    ctx.link_batch(&edges)?
                } else {
                    0
                };

                Ok(json!({
                    "chunks_inserted": inserted,
                    "edges_linked": linked,
                    "chunk_ids": chunk_ids
                }))
            }
            "chunk_text" => {
                let text = args
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Missing 'text' field".to_string())?;
                let chunk_size = args
                    .get("chunk_size")
                    .and_then(Value::as_u64)
                    .unwrap_or(512) as usize;
                let chunk_overlap = args
                    .get("chunk_overlap")
                    .and_then(Value::as_u64)
                    .unwrap_or(64) as usize;
                let chunks = Self::chunk_markdown(text, chunk_size, chunk_overlap);
                Ok(json!({ "chunks": chunks }))
            }
            "embed_text" | "vectorize" => {
                let text = args
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Missing 'text' field".to_string())?;
                let dim = args
                    .get("dimension")
                    .and_then(Value::as_u64)
                    .map(|d| d as usize)
                    .or_else(|| ctx.dimension().ok())
                    .unwrap_or(128);
                let metric = args
                    .get("metric")
                    .and_then(Value::as_str)
                    .or_else(|| ctx.metric_name().ok())
                    .unwrap_or("cosine");
                let vector = self.embed_text_smart(text, dim, metric).await;
                Ok(json!({
                    "vector": vector,
                    "dimension": dim,
                    "metric": metric
                }))
            }
            other => Err(format!("Unknown procedure 'embed.{other}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_mrl_normalization_and_truncation() {
        let mut vec = vec![0.0; 801];
        for val in &mut vec[1..33] {
            *val = 0.1;
        }
        for val in &mut vec[33..801] {
            *val = 1.0;
        }

        use std::mem::MaybeUninit;
        let mut vectorizer_uninit = MaybeUninit::<OnnxVectorizer>::uninit();
        let vectorizer = unsafe {
            let ptr = vectorizer_uninit.as_mut_ptr();
            std::ptr::addr_of_mut!((*ptr).metric).write(Metric::Hybrid);
            std::ptr::addr_of_mut!((*ptr).dimension).write(801);
            &*ptr
        };

        vectorizer.normalize(&mut vec);

        let spatial_sq_sum: f64 = vec[1..33].iter().map(|&x| x * x).sum();
        let expected_x0 = (1.0 + spatial_sq_sum).sqrt();
        assert!((vec[0] - expected_x0).abs() < 1e-9);

        let euc_sq_sum: f64 = vec[33..801].iter().map(|&x| x * x).sum();
        assert!((euc_sq_sum - 1.0).abs() < 1e-9);

        let target_dim = 161;
        vec.truncate(target_dim);

        let euc_sq_sum_after_trunc: f64 = vec[33..target_dim].iter().map(|&x| x * x).sum();
        assert!(euc_sq_sum_after_trunc < 1.0);

        vectorizer.normalize(&mut vec);

        let spatial_sq_sum_final: f64 = vec[1..33].iter().map(|&x| x * x).sum();
        let expected_x0_final = (1.0 + spatial_sq_sum_final).sqrt();
        assert!((vec[0] - expected_x0_final).abs() < 1e-9);

        let euc_sq_sum_final: f64 = vec[33..target_dim].iter().map(|&x| x * x).sum();
        assert!((euc_sq_sum_final - 1.0).abs() < 1e-9);
    }
}
