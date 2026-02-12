use anyhow::Result;
use async_trait::async_trait;
use ndarray::{Array, ArrayViewD};
use ort::{
    session::{builder::GraphOptimizationLevel, Session},
    value::Value,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokenizers::Tokenizer;

// --- Config Types ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Metric {
    Poincare,
    L2,
    Cosine,
    None,
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

// --- Local ONNX Vectorizer ---

pub struct OnnxVectorizer {
    tokenizer: Tokenizer,
    session: Mutex<Session>,
    dimension: usize,
    metric: Metric,
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
    ) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {e}"))?;

        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self {
            tokenizer,
            session: Mutex::new(session),
            dimension,
            metric,
        })
    }

    fn normalize(&self, vec: &mut [f64]) {
        const EPSILON: f64 = 1e-5;
        let norm_sq: f64 = vec.iter().map(|x| x * x).sum();
        let norm = norm_sq.sqrt();

        match self.metric {
            Metric::Poincare => {
                if norm >= 1.0 {
                    let scale = (1.0 - EPSILON) / (norm + 1e-12);
                    for x in vec.iter_mut() {
                        *x *= scale;
                    }
                }
            }
            Metric::L2 | Metric::Cosine => {
                if norm > 0.0 {
                    for x in vec.iter_mut() {
                        *x /= norm;
                    }
                }
            }
            Metric::None => {}
        }
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

        let input_ids_val = Value::from_array(input_ids_arr)?;
        let attention_mask_val = Value::from_array(attention_mask_arr)?;

        let inputs = ort::inputs![
            "input_ids" => input_ids_val,
            "attention_mask" => attention_mask_val
        ];

        let mut session_guard = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("Session lock poisoned"))?;
        let outputs = session_guard.run(inputs)?;

        let output_tensor = &outputs[0];
        let (shape, data) = output_tensor.try_extract_tensor::<f32>()?;
        let shape_usize: Vec<usize> = shape.iter().map(|&x| x as usize).collect();
        let embeddings_tensor = ArrayViewD::from_shape(shape_usize, data)
            .map_err(|e| anyhow::anyhow!("Failed to create view from tensor: {e}"))?;

        let mut final_vectors = Vec::with_capacity(batch_size);

        if embeddings_tensor.ndim() == 3 {
            final_vectors = self.mean_pooling(
                &embeddings_tensor.view(),
                &attention_mask_clone.view().into_dyn(),
            );
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
