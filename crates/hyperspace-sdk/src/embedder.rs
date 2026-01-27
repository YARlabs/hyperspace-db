use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;

/// Trait for Client-Side Embedders
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn encode(&self, text: &str) -> Result<Vec<f64>, Box<dyn Error>>;
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
