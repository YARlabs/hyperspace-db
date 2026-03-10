use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Trait for Client-Side Embedders
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>>;
}

/// Which geometry the embedding vectors are designed for.
/// Determines post-processing normalization applied by local embedders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedGeometry {
    /// Cosine / dot-product space — unit-normalize the vector.
    Cosine,
    /// Euclidean (L2) space — unit-normalize the vector.
    L2,
    /// Poincaré ball — clamp to open unit ball (||x|| < 1).
    Poincare,
    /// Lorentz hyperboloid — no client-side post-processing needed.
    Lorentz,
}

impl EmbedGeometry {
    /// Parse a geometry name from a string. Case-insensitive.
    pub fn from_metric_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "poincare" | "hyperbolic" => Self::Poincare,
            "lorentz" => Self::Lorentz,
            "l2" | "euclidean" => Self::L2,
            _ => Self::Cosine,
        }
    }

    fn normalize(&self, vec: &mut [f64]) {
        const EPS: f64 = 1e-5;
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        match self {
            Self::Cosine | Self::L2 => {
                if norm > 0.0 {
                    vec.iter_mut().for_each(|x| *x /= norm);
                }
            }
            Self::Poincare => {
                if norm >= 1.0 {
                    let scale = (1.0 - EPS) / (norm + 1e-12);
                    vec.iter_mut().for_each(|x| *x *= scale);
                }
            }
            Self::Lorentz => {} // hyperboloid constraint is maintained by the model head
        }
    }
}

// ==========================================
// Local ONNX Embedder
// Loads model.onnx + tokenizer.json from disk.
// Feature: `local-onnx`
// ==========================================

#[cfg(feature = "local-onnx")]
pub struct LocalOnnxEmbedder {
    tokenizer: tokenizers::Tokenizer,
    session: std::sync::Mutex<ort::session::Session>,
    geometry: EmbedGeometry,
}

#[cfg(feature = "local-onnx")]
impl LocalOnnxEmbedder {
    /// Load an ONNX embedding model from local filesystem paths.
    ///
    /// # Arguments
    /// - `model_path`     — path to `model.onnx`
    /// - `tokenizer_path` — path to `tokenizer.json` (HF tokenizer format)
    /// - `geometry`       — how to post-process the output vector
    ///
    /// # Errors
    /// Returns an error if the model or tokenizer fails to load.
    pub fn new(
        model_path: &str,
        tokenizer_path: &str,
        geometry: EmbedGeometry,
    ) -> Result<Self, Box<dyn Error>> {
        let tokenizer = tokenizers::Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("Tokenizer load error: {e}"))?;
        let session = ort::session::Session::builder()?
            .with_optimization_level(ort::session::builder::GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;
        Ok(Self {
            tokenizer,
            session: std::sync::Mutex::new(session),
            geometry,
        })
    }

    fn run_inference(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        use ndarray::Array;
        use ort::value::Value;

        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| format!("Encode error: {e}"))?;

        let ids: Vec<i64> = encoding.get_ids().iter().map(|&x| x as i64).collect();
        let mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&x| x as i64)
            .collect();
        let len = ids.len();

        let ids_arr = Array::from_shape_vec((1, len), ids)?;
        let mask_arr = Array::from_shape_vec((1, len), mask.clone())?;

        let inputs = ort::inputs![
            "input_ids"      => Value::from_array(ids_arr)?,
            "attention_mask" => Value::from_array(mask_arr)?
        ];

        let mut sess = self.session.lock().map_err(|_| "Session lock poisoned")?;
        let outputs = sess.run(inputs)?;
        let (shape, data) = outputs[0].try_extract_tensor::<f32>()?;
        let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
        let tensor = ndarray::ArrayViewD::from_shape(shape_usize, data)?;

        // mean-pool over sequence dimension (dim 1) if 3D [batch, seq, hidden]
        let vec: Vec<f64> = if tensor.ndim() == 3 {
            let mask_f: Vec<f64> = encoding
                .get_attention_mask()
                .iter()
                .map(|&x| x as f64)
                .collect();
            let count: f64 = mask_f.iter().sum::<f64>().max(1.0);
            let hidden = tensor.shape()[2];
            (0..hidden)
                .map(|k| {
                    let sum: f64 = (0..len)
                        .map(|j| f64::from(tensor[[0, j, k]]) * mask_f[j])
                        .sum();
                    sum / count
                })
                .collect()
        } else {
            // 2D [batch, hidden] — already pooled
            (0..tensor.shape()[1])
                .map(|k| f64::from(tensor[[0, k]]))
                .collect()
        };

        Ok(vec)
    }
}

#[cfg(feature = "local-onnx")]
#[async_trait]
impl Embedder for LocalOnnxEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let mut vec = tokio::task::block_in_place(|| self.run_inference(text))?;
        self.geometry.normalize(&mut vec);
        Ok(vec)
    }
}

// ==========================================
// HuggingFace Hub Embedder
// Downloads model.onnx + tokenizer.json from Hub automatically,
// then delegates to LocalOnnxEmbedder.
// Feature: `huggingface`
// ==========================================

#[cfg(feature = "huggingface")]
pub struct HuggingFaceEmbedder {
    inner: LocalOnnxEmbedder,
    model_id: String,
}

#[cfg(feature = "huggingface")]
impl HuggingFaceEmbedder {
    /// Download an embedding model from HuggingFace Hub and load it locally.
    ///
    /// The repo must contain `model.onnx` and `tokenizer.json`.
    ///
    /// # Arguments
    /// - `model_id` — HF repo, e.g. `"BAAI/bge-small-en-v1.5"`
    /// - `hf_token` — optional token for gated / private models
    /// - `geometry`  — geometry for post-processing
    ///
    /// # Errors
    /// Returns an error if download or model loading fails.
    pub fn new(
        model_id: &str,
        hf_token: Option<String>,
        geometry: EmbedGeometry,
    ) -> Result<Self, Box<dyn Error>> {
        let mut builder = hf_hub::api::sync::ApiBuilder::new().with_progress(true);
        if let Some(tok) = hf_token.filter(|t| !t.is_empty()) {
            builder = builder.with_token(Some(tok));
        }
        let api = builder.build().map_err(|e| format!("HF API init: {e}"))?;
        let repo = api.model(model_id.to_string());

        let model_path = repo
            .get("model.onnx")
            .map_err(|e| format!("model.onnx: {e}"))?;
        let tok_path = repo
            .get("tokenizer.json")
            .map_err(|e| format!("tokenizer.json: {e}"))?;

        let inner = LocalOnnxEmbedder::new(
            model_path.to_str().ok_or("Invalid model path")?,
            tok_path.to_str().ok_or("Invalid tokenizer path")?,
            geometry,
        )?;

        Ok(Self {
            inner,
            model_id: model_id.to_string(),
        })
    }

    /// The HuggingFace model ID this embedder was constructed from.
    pub fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[cfg(feature = "huggingface")]
#[async_trait]
impl Embedder for HuggingFaceEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        self.inner.encode(text).await
    }
}

// ==========================================
// OpenAI & OpenRouter (Compatible API)
// ==========================================

#[derive(Clone)]
pub struct OpenAIEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAIEmbedder {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }

    /// Create an embedder for OpenRouter or other OpenAI-compatible APIs
    pub fn new_compatible(api_key: String, model: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            base_url,
        }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    input: String,
    model: String,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f64>,
}

#[async_trait]
impl Embedder for OpenAIEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let req = OpenAIRequest {
            input: text.replace("\n", " "),
            model: self.model.clone(),
        };

        let url = format!("{}/embeddings", self.base_url);

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(format!("OpenAI/OpenRouter API Error: {}", error).into());
        }

        let body: OpenAIResponse = res.json().await?;
        if let Some(item) = body.data.first() {
            Ok(item.embedding.clone())
        } else {
            Err("No embedding returned".into())
        }
    }
}

// ==========================================
// Cohere
// ==========================================

#[derive(Clone)]
pub struct CohereEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
    input_type: String,
}

impl CohereEmbedder {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
            input_type: "search_document".to_string(),
        }
    }
}

#[derive(Serialize)]
struct CohereRequest {
    texts: Vec<String>,
    model: String,
    input_type: String,
    embedding_types: Vec<String>,
}

#[derive(Deserialize)]
struct CohereResponse {
    embeddings: CohereEmbeddings,
}

#[derive(Deserialize)]
struct CohereEmbeddings {
    float: Vec<Vec<f64>>,
}

#[async_trait]
impl Embedder for CohereEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let req = CohereRequest {
            texts: vec![text.to_string()],
            model: self.model.clone(),
            input_type: self.input_type.clone(),
            embedding_types: vec!["float".to_string()],
        };

        let res = self
            .client
            .post("https://api.cohere.ai/v1/embed")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("X-Client-Name", "hyperspace-rust-sdk")
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(format!("Cohere API Error: {}", error).into());
        }

        let body: CohereResponse = res.json().await?;
        if let Some(emb) = body.embeddings.float.first() {
            Ok(emb.clone())
        } else {
            Err("No embedding returned".into())
        }
    }
}

// ==========================================
// Voyage AI
// ==========================================

#[derive(Clone)]
pub struct VoyageEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl VoyageEmbedder {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct VoyageRequest {
    input: Vec<String>,
    model: String,
}

#[derive(Deserialize)]
struct VoyageResponse {
    data: Vec<VoyageEmbeddingData>,
}

#[derive(Deserialize)]
struct VoyageEmbeddingData {
    embedding: Vec<f64>,
}

#[async_trait]
impl Embedder for VoyageEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let req = VoyageRequest {
            input: vec![text.to_string()],
            model: self.model.clone(),
        };

        let res = self
            .client
            .post("https://api.voyageai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(format!("Voyage AI API Error: {}", error).into());
        }

        let body: VoyageResponse = res.json().await?;
        if let Some(item) = body.data.first() {
            Ok(item.embedding.clone())
        } else {
            Err("No embedding returned".into())
        }
    }
}

// ==========================================
// Google (Gemini)
// ==========================================

#[derive(Clone)]
pub struct GoogleEmbedder {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl GoogleEmbedder {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleRequest {
    content: GoogleContent,
    task_type: String,
}

#[derive(Serialize)]
struct GoogleContent {
    parts: Vec<GooglePart>,
}

#[derive(Serialize)]
struct GooglePart {
    text: String,
}

#[derive(Deserialize)]
struct GoogleResponse {
    embedding: GoogleEmbedding,
}

#[derive(Deserialize)]
struct GoogleEmbedding {
    values: Vec<f64>,
}

#[async_trait]
impl Embedder for GoogleEmbedder {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>> {
        let req = GoogleRequest {
            content: GoogleContent {
                parts: vec![GooglePart {
                    text: text.to_string(),
                }],
            },
            task_type: "RETRIEVAL_DOCUMENT".to_string(),
        };

        // models/embedding-001 -> v1beta/models/embedding-001:embedContent
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}:embedContent?key={}",
            self.model, self.api_key
        );

        let res = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&req)
            .send()
            .await?;

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(format!("Google API Error: {}", error).into());
        }

        let body: GoogleResponse = res.json().await?;
        Ok(body.embedding.values)
    }
}
