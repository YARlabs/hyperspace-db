#![cfg_attr(feature = "nightly-simd", feature(portable_simd))]
#![allow(clippy::many_single_char_names)]

pub use hyperspace_proto::hyperspace::database_client::DatabaseClient;
pub use hyperspace_proto::hyperspace::{
    BatchInsertRequest, BatchSearchRequest, CollectionSchema, CollectionSummary, CountRequest,
    DurabilityLevel, EventMessage, EventSubscriptionRequest, EventType, Filter, FilterAnd,
    FilterNot, FilterOr, FindSemanticClustersRequest, FindSemanticClustersResponse,
    GetConceptParentsRequest, GetConceptParentsResponse, GetNeighborsRequest, GetNeighborsResponse,
    GetNodeRequest, GetPointsRequest, GetSubsumptionTreeRequest, GetSubsumptionTreeResponse,
    GraphNode, InBall, InBox, InCone, InsertRequest, InsertTextRequest, Match, MrlLayer, Prefix,
    Range, ScrollRequest, SearchRequest, SearchResponse, SearchResult, SearchResult as ResultItem,
    SearchTextRequest, TraverseRequest, TraverseResponse, UpdatePayloadRequest, VectorComponent,
    VectorData, VectorizeRequest, VectorizeResponse,
};
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

pub mod fuzzy;
pub mod gromov;
pub mod math;

#[cfg(feature = "embedders")]
mod embedder;
#[cfg(feature = "embedders")]
pub use embedder::*;

/// Lightweight ego-graph result returned by [`Client::explore_graph`].
#[derive(Debug, Clone)]
pub struct EgoGraphNode {
    pub id: u32,
    pub metadata: std::collections::HashMap<String, String>,
    pub edge_types: Vec<i32>,
}

/// Result of [`Client::explore_graph`]: a centre node id and a list of neighbouring nodes.
#[derive(Debug, Clone)]
pub struct EgoGraph {
    pub center_id: u32,
    pub nodes: Vec<EgoGraphNode>,
}

#[derive(Clone)]
pub struct AuthInterceptor {
    api_key: Option<String>,
    user_id: Option<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(key) = &self.api_key {
            let token = key
                .parse()
                .map_err(|_| Status::invalid_argument("Invalid API Key format"))?;
            request.metadata_mut().insert("x-api-key", token);
        }
        if let Some(uid) = &self.user_id {
            let token = uid
                .parse()
                .map_err(|_| Status::invalid_argument("Invalid User ID format"))?;
            request.metadata_mut().insert("x-hyperspace-user-id", token);
        }
        Ok(request)
    }
}

struct EncryptionContext {
    aes_key: Vec<u8>,
    hmac_key: Vec<u8>,
    projection_matrices: parking_lot::RwLock<std::collections::HashMap<String, Vec<Vec<f64>>>>,
}

pub struct Client {
    inner: DatabaseClient<InterceptedService<Channel, AuthInterceptor>>,
    #[cfg(feature = "embedders")]
    embedder: Option<Box<dyn Embedder>>,
    collection_keys: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, String>>>,
    encryption_contexts: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, std::sync::Arc<EncryptionContext>>>>,
    collection_metrics: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, String>>>,
    collection_noise_sigmas: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, f64>>>,
    collection_schemas: std::sync::Arc<parking_lot::RwLock<std::collections::HashMap<String, CollectionSchema>>>,
}

impl Client {
    #[inline]
    fn vec_f32_to_f64(vector: &[f32]) -> Vec<f64> {
        vector.iter().map(|&x| f64::from(x)).collect()
    }

    /// Connects to the `HyperspaceDB` server.
    ///
    /// # Errors
    /// Returns error if connection fails.
    pub async fn connect(
        dst: String,
        api_key: Option<String>,
        user_id: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(dst)?
            .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
            .tcp_nodelay(true)
            .keep_alive_while_idle(true)
            .connect_timeout(std::time::Duration::from_secs(10))
            .connect()
            .await?;

        let interceptor = AuthInterceptor { api_key, user_id };
        let client = DatabaseClient::with_interceptor(channel, interceptor)
            .max_decoding_message_size(64 * 1024 * 1024) // 64MB
            .max_encoding_message_size(64 * 1024 * 1024); // 64MB

        Ok(Self {
            inner: client,
            #[cfg(feature = "embedders")]
            embedder: None,
            collection_keys: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            encryption_contexts: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            collection_metrics: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            collection_noise_sigmas: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
            collection_schemas: std::sync::Arc::new(parking_lot::RwLock::new(std::collections::HashMap::new())),
        })
    }

    #[cfg(feature = "embedders")]
    pub fn set_embedder(&mut self, embedder: Box<dyn Embedder>) {
        self.embedder = Some(embedder);
    }

    /// Creates a new collection.
    ///
    /// # Errors
    /// Returns error if the collection already exists or if network fails.
    pub async fn create_collection(
        &mut self,
        name: String,
        schema: CollectionSchema,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::CreateCollectionRequest {
            name,
            schema: Some(schema),
        };
        let resp = self.inner.create_collection(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Deletes a collection.
    ///
    /// # Errors
    /// Returns error if the collection does not exist cancellation.
    pub async fn delete_collection(&mut self, name: String) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DeleteCollectionRequest { name };
        let resp = self.inner.delete_collection(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Freezes a collection manually (unloads HNSW from memory).
    ///
    /// # Errors
    /// Returns error if the collection does not exist or network fails.
    pub async fn freeze_collection(&mut self, name: String) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::FreezeCollectionRequest { name };
        let resp = self.inner.freeze_collection(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Unfreezes a collection manually (lazy-loads HNSW back into memory).
    ///
    /// # Errors
    /// Returns error if the collection does not exist or network fails.
    pub async fn unfreeze_collection(&mut self, name: String) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::UnfreezeCollectionRequest { name };
        let resp = self.inner.unfreeze_collection(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Lists all collections with their metadata (dimension, metric, count).
    ///
    /// # Errors
    /// Returns error on network failure.
    pub async fn list_collections(&mut self) -> Result<Vec<CollectionSummary>, tonic::Status> {
        let req = hyperspace_proto::hyperspace::Empty {};
        let resp = self.inner.list_collections(req).await?;
        Ok(resp.into_inner().collections)
    }

    /// Gets statistics for a collection.
    ///
    /// # Errors
    /// Returns error if the collection does not exist or network fails.
    pub async fn get_collection_stats(
        &mut self,
        name: String,
    ) -> Result<hyperspace_proto::hyperspace::CollectionStatsResponse, tonic::Status> {
        let req = hyperspace_proto::hyperspace::CollectionStatsRequest { name };
        let resp = self.inner.get_collection_stats(req).await?;
        Ok(resp.into_inner())
    }

    /// Rebuilds the index for a collection. This is a resource-intensive operation.
    ///
    /// # Errors
    /// Returns error if the collection does not exist or operation fails.
    pub async fn rebuild_index(&mut self, name: String) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::RebuildIndexRequest {
            name,
            filter_query: None,
        };
        let resp = self.inner.rebuild_index(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Rebuilds index with optional metadata-based pruning filter.
    ///
    /// # Errors
    /// Returns error if operation fails.
    pub async fn rebuild_index_with_filter(
        &mut self,
        name: String,
        key: String,
        op: String,
        value: f64,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::RebuildIndexRequest {
            name,
            filter_query: Some(hyperspace_proto::hyperspace::VacuumFilterQuery { key, op, value }),
        };
        let resp = self.inner.rebuild_index(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Triggers memory cleanup (Vacuum).
    ///
    /// # Errors
    /// Returns error if the operation fails.
    pub async fn trigger_vacuum(&mut self) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::Empty {};
        let resp = self.inner.trigger_vacuum(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Triggers background memory reconsolidation (AI Sleep Mode / Flow Matching SGD).
    ///
    /// # Errors
    /// Returns error if the operation fails.
    pub async fn trigger_reconsolidation(
        &mut self,
        collection: String,
        target_vector: Vec<f64>,
        learning_rate: f64,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::ReconsolidationRequest {
            collection,
            target_vector,
            learning_rate,
        };
        let resp = self.inner.trigger_reconsolidation(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Inserts a vector into the collection.
    ///
    /// # Errors
    /// Returns error if insertion fails.
    pub async fn insert(
        &mut self,
        id: u32,
        vector: Vec<f64>,
        metadata: std::collections::HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<bool, tonic::Status> {
        self.insert_secure(id, vector, metadata, None, collection).await
    }

    /// Inserts a vector with an optional payload, supporting client-side ZK-privacy.
    ///
    /// # Errors
    /// Returns error if insertion fails.
    pub async fn insert_secure(
        &mut self,
        id: u32,
        vector: Vec<f64>,
        metadata: std::collections::HashMap<String, String>,
        payload: Option<Vec<u8>>,
        collection: Option<String>,
    ) -> Result<bool, tonic::Status> {
        let coll = collection.unwrap_or_default();
        let metric = {
            let metrics = self.collection_metrics.read();
            metrics.get(&coll).cloned().unwrap_or_else(|| "l2".to_string())
        };

        let context = self.get_encryption_context(&coll, vector.len(), &metric).await;

        let mut final_vector = vector;
        let mut final_metadata = metadata;
        let mut final_payload = payload;

        if let Some(ref ctx) = context {
            // 1. Noise injection
            let sigma = {
                let sigmas = self.collection_noise_sigmas.read();
                *sigmas.get(&coll).unwrap_or(&0.02)
            };
            if sigma > 0.0 {
                final_vector = math::inject_anisotropic_noise(&final_vector, &ctx.hmac_key, sigma);
            }

            // 2. Vector projection
            final_vector = self.project_collection_vector(&coll, &final_vector, ctx, &metric);

            // 3. Encrypt payload
            if let Some(p) = final_payload {
                final_payload = Some(self.encrypt_payload(&p, &ctx.aes_key).map_err(tonic::Status::internal)?);
            }

            // 4. Hash metadata
            let mut hashed_meta = std::collections::HashMap::new();
            for (k, v) in final_metadata {
                let ek = self.hash_metadata_key(&k, &ctx.hmac_key);
                let ev = self.hash_metadata_value(&v, &ctx.hmac_key);
                hashed_meta.insert(ek, ev);
            }
            final_metadata = hashed_meta;
        }

        let req = InsertRequest {
            id,
            vector: final_vector,
            metadata: final_metadata,
            typed_metadata: std::collections::HashMap::new(),
            collection: coll,
            origin_node_id: String::new(),
            logical_clock: 0,
            durability: 0,
            payload: final_payload,
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
    }

    /// Inserts a vector from f32 input (client-side conversion to protocol f64).
    ///
    /// # Errors
    /// Returns error if insertion fails.
    pub async fn insert_f32(
        &mut self,
        id: u32,
        vector: &[f32],
        metadata: std::collections::HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<bool, tonic::Status> {
        self.insert(id, Self::vec_f32_to_f64(vector), metadata, collection)
            .await
    }

    /// Inserts text that will be vectorized on the server side.
    ///
    /// # Errors
    /// Returns error if insertion/vectorization fails.
    pub async fn insert_text(
        &mut self,
        id: u32,
        text: String,
        metadata: std::collections::HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<bool, tonic::Status> {
        let req = InsertTextRequest {
            id,
            text,
            metadata,
            collection: collection.unwrap_or_default(),
            durability: 0,
        };
        let resp = self.inner.insert_text(req).await?;
        Ok(resp.into_inner().success)
    }

    /// Vectoize text using the server-side embedding engine.
    ///
    /// # Errors
    /// Returns error if vectorization fails.
    pub async fn vectorize(
        &mut self,
        text: String,
        metric: String,
    ) -> Result<Vec<f64>, tonic::Status> {
        let req = VectorizeRequest { text, metric };
        let resp = self.inner.vectorize(req).await?;
        Ok(resp.into_inner().vector)
    }

    /// Batch inserts multiple vectors.
    ///
    /// # Errors
    /// Returns error if insertion fails.
    pub async fn batch_insert(
        &mut self,
        items: Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)>,
        collection: Option<String>,
        durability: DurabilityLevel,
    ) -> Result<bool, tonic::Status> {
        let vectors = items
            .into_iter()
            .map(|(id, vector, metadata)| VectorData {
                id,
                vector,
                metadata,
                typed_metadata: std::collections::HashMap::new(),
            })
            .collect();
        let req = BatchInsertRequest {
            collection: collection.unwrap_or_default(),
            vectors,
            origin_node_id: String::new(),
            logical_clock: 0,
            durability: durability as i32,
        };
        let resp = self.inner.batch_insert(req).await?;
        Ok(resp.into_inner().success)
    }

    /// Batch inserts multiple vectors from f32 input.
    ///
    /// # Errors
    /// Returns error if insertion fails.
    pub async fn batch_insert_f32(
        &mut self,
        items: Vec<(u32, Vec<f32>, std::collections::HashMap<String, String>)>,
        collection: Option<String>,
        durability: DurabilityLevel,
    ) -> Result<bool, tonic::Status> {
        let items_f64 = items
            .into_iter()
            .map(|(id, v, m)| (id, Self::vec_f32_to_f64(&v), m))
            .collect();
        self.batch_insert(items_f64, collection, durability).await
    }

    /// Searches for nearest neighbors.
    ///
    /// # Errors
    /// Returns error if search fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        collection: Option<String>,
        mrl_dimension: Option<u32>,
        use_wasserstein: bool,
        component_weights: std::collections::HashMap<String, f32>,
        use_wave: bool,
        restart_factor: Option<f32>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let coll = collection.unwrap_or_default();
        let metric = {
            let metrics = self.collection_metrics.read();
            metrics.get(&coll).cloned().unwrap_or_else(|| "l2".to_string())
        };

        let context = self.get_encryption_context(&coll, vector.len(), &metric).await;

        let mut final_vector = vector;
        let mut filter = std::collections::HashMap::default();
        if let Some(rf) = restart_factor {
            filter.insert("wave_restart_factor".to_string(), rf.to_string());
        }
        let mut final_include_payload = false;

        if let Some(ref ctx) = context {
            // 1. Noise injection
            let sigma = {
                let sigmas = self.collection_noise_sigmas.read();
                *sigmas.get(&coll).unwrap_or(&0.02)
            };
            if sigma > 0.0 {
                final_vector = math::inject_anisotropic_noise(&final_vector, &ctx.hmac_key, sigma);
            }

            // 2. Vector projection
            final_vector = self.project_collection_vector(&coll, &final_vector, ctx, &metric);

            // 3. Force payload inclusion so we can decrypt locally
            final_include_payload = true;
        }

        let req = SearchRequest {
            vector: final_vector,
            top_k,
            filter,
            filters: vec![],
            hybrid_query: None,
            hybrid_alpha: None,
            use_wasserstein,
            collection: coll,
            bm25_options: None,
            mrl_dimension,
            include_payload: final_include_payload,
            component_weights,
            use_wave,
        };
        let resp = self.inner.search(req).await?;
        let mut results = resp.into_inner().results;

        if let Some(ctx) = context {
            for r in &mut results {
                if let Some(p) = &r.payload {
                    if let Ok(dec) = self.decrypt_payload(p, &ctx.aes_key) {
                        r.payload = Some(dec);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Searches using f32 query vector (converted to protocol f64 once).
    ///
    /// # Errors
    /// Returns error if search fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn search_f32(
        &mut self,
        vector: &[f32],
        top_k: u32,
        collection: Option<String>,
        mrl_dimension: Option<u32>,
        use_wasserstein: bool,
        component_weights: std::collections::HashMap<String, f32>,
        use_wave: bool,
        restart_factor: Option<f32>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        self.search(
            Self::vec_f32_to_f64(vector),
            top_k,
            collection,
            mrl_dimension,
            use_wasserstein,
            component_weights,
            use_wave,
            restart_factor,
        )
        .await
    }

    /// Searches using text that will be vectorized on the server side.
    ///
    /// # Errors
    /// Returns error if search fails.
    pub async fn search_text(
        &mut self,
        text: String,
        top_k: u32,
        collection: Option<String>,
        bm25_options: Option<hyperspace_proto::hyperspace::Bm25Options>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let req = SearchTextRequest {
            text,
            top_k,
            collection: collection.unwrap_or_default(),
            filter: std::collections::HashMap::new(),
            filters: vec![],
            bm25_options,
            hybrid_alpha: None,
            include_payload: false,
            component_weights: std::collections::HashMap::new(),
        };
        let resp = self.inner.search_text(req).await?;
        Ok(resp.into_inner().results)
    }

    /// Performs search utilizing the Wasserstein distance (Cross-Feature Matching Metric).
    ///
    /// # Errors
    /// Returns error if search fails.
    pub async fn search_wasserstein(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let req = SearchRequest {
            vector,
            top_k,
            filter: std::collections::HashMap::default(),
            filters: vec![],
            hybrid_query: None,
            hybrid_alpha: None,
            use_wasserstein: true,
            collection: collection.unwrap_or_default(),
            bm25_options: None,
            mrl_dimension: None,
            include_payload: false,
            component_weights: std::collections::HashMap::new(),
            use_wave: false,
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }

    /// Batch search for multiple vectors in a single RPC.
    ///
    /// # Errors
    /// Returns error if the batch search fails.
    pub async fn search_batch(
        &mut self,
        vectors: Vec<Vec<f64>>,
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<Vec<SearchResult>>, tonic::Status> {
        let collection_name = collection.unwrap_or_default();
        let searches = vectors
            .into_iter()
            .map(|vector| SearchRequest {
                vector,
                top_k,
                filter: std::collections::HashMap::default(),
                filters: vec![],
                hybrid_query: None,
                hybrid_alpha: None,
                use_wasserstein: false,
                collection: collection_name.clone(),
                bm25_options: None,
                mrl_dimension: None,
                include_payload: false,
                component_weights: std::collections::HashMap::new(),
                use_wave: false,
            })
            .collect();

        let req = BatchSearchRequest { searches };
        let resp = self.inner.search_batch(req).await?;
        Ok(resp
            .into_inner()
            .responses
            .into_iter()
            .map(|SearchResponse { results }| results)
            .collect())
    }

    /// Batch search from f32 vectors (converted to protocol f64 once).
    ///
    /// # Errors
    /// Returns error if the batch search fails.
    pub async fn search_batch_f32(
        &mut self,
        vectors: &[Vec<f32>],
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<Vec<SearchResult>>, tonic::Status> {
        let vectors_f64 = vectors
            .iter()
            .map(|v| Self::vec_f32_to_f64(v))
            .collect::<Vec<_>>();
        self.search_batch(vectors_f64, top_k, collection).await
    }

    /// Multi-Geometry Benchmark Endpoint (10.3)
    /// Performs identical searches across multiple collections in parallel (via Batch Search).
    /// Typically used by the Frontend to compare L2, Cosine, Poincare and Lorentz results directly.
    ///
    /// # Errors
    /// Returns error if the batch search RPC fails.
    pub async fn search_multi_collection(
        &mut self,
        vector: Vec<f64>,
        collections: Vec<String>,
        top_k: u32,
    ) -> Result<std::collections::HashMap<String, Vec<SearchResult>>, tonic::Status> {
        let searches = collections
            .iter()
            .map(|col_name| SearchRequest {
                vector: vector.clone(),
                top_k,
                filter: std::collections::HashMap::default(),
                filters: vec![],
                hybrid_query: None,
                hybrid_alpha: None,
                use_wasserstein: false,
                collection: col_name.clone(),
                bm25_options: None,
                mrl_dimension: None,
                include_payload: false,
                component_weights: std::collections::HashMap::new(),
                use_wave: false,
            })
            .collect();

        let req = BatchSearchRequest { searches };
        let resp = self.inner.search_batch(req).await?;

        let mut result_map = std::collections::HashMap::new();
        for (col_name, response) in collections.into_iter().zip(resp.into_inner().responses) {
            result_map.insert(col_name, response.results);
        }

        Ok(result_map)
    }

    /// Advanced search with filters and hybrid query.
    ///
    /// # Errors
    /// Returns error if search fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn search_advanced(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        filters: Vec<hyperspace_proto::hyperspace::Filter>,
        hybrid: Option<(String, f32)>,
        bm25_options: Option<hyperspace_proto::hyperspace::Bm25Options>,
        collection: Option<String>,
        use_wave: bool,
        restart_factor: Option<f32>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let coll = collection.unwrap_or_default();
        let metric = {
            let metrics = self.collection_metrics.read();
            metrics.get(&coll).cloned().unwrap_or_else(|| "l2".to_string())
        };

        let context = self.get_encryption_context(&coll, vector.len(), &metric).await;

        let mut final_vector = vector;
        let mut filter = std::collections::HashMap::default();
        if let Some(rf) = restart_factor {
            filter.insert("wave_restart_factor".to_string(), rf.to_string());
        }
        let mut final_filters = filters;
        let mut final_include_payload = false;

        if let Some(ref ctx) = context {
            // 1. Noise injection
            let sigma = {
                let sigmas = self.collection_noise_sigmas.read();
                *sigmas.get(&coll).unwrap_or(&0.02)
            };
            if sigma > 0.0 {
                final_vector = math::inject_anisotropic_noise(&final_vector, &ctx.hmac_key, sigma);
            }

            // 2. Vector projection
            final_vector = self.project_collection_vector(&coll, &final_vector, ctx, &metric);

            // 3. Hash metadata filters
            let mut hashed_filter = std::collections::HashMap::new();
            for (k, v) in filter {
                let ek = self.hash_metadata_key(&k, &ctx.hmac_key);
                let ev = self.hash_metadata_value(&v, &ctx.hmac_key);
                hashed_filter.insert(ek, ev);
            }
            filter = hashed_filter;

            // 4. Hash filters list
            final_filters = self.encrypt_filters(final_filters, ctx);

            // 5. Force include payload
            final_include_payload = true;
        }

        let (hybrid_query, hybrid_alpha) = match hybrid {
            Some((q, a)) => (Some(q), Some(a)),
            None => (None, None),
        };

        let req = SearchRequest {
            vector: final_vector,
            top_k,
            filter,
            filters: final_filters,
            hybrid_query,
            hybrid_alpha,
            use_wasserstein: false,
            collection: coll,
            bm25_options,
            mrl_dimension: None,
            include_payload: final_include_payload,
            component_weights: std::collections::HashMap::new(),
            use_wave,
        };
        let resp = self.inner.search(req).await?;
        let mut results = resp.into_inner().results;

        if let Some(ctx) = context {
            for r in &mut results {
                if let Some(p) = &r.payload {
                    if let Ok(dec) = self.decrypt_payload(p, &ctx.aes_key) {
                        r.payload = Some(dec);
                    }
                }
            }
        }

        Ok(results)
    }

    /// High-level hybrid search combining vector (semantic) and lexical (BM25) ranking.
    ///
    /// # Errors
    /// Returns error if search fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn search_hybrid(
        &mut self,
        vector: Vec<f64>,
        text: String,
        alpha: f32, // 1.0 = Pure Vector, 0.0 = Pure BM25
        top_k: u32,
        collection: Option<String>,
        bm25_options: Option<hyperspace_proto::hyperspace::Bm25Options>,
        use_wave: bool,
        restart_factor: Option<f32>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        self.search_advanced(
            vector,
            top_k,
            vec![],
            Some((text, alpha)),
            bm25_options,
            collection,
            use_wave,
            restart_factor,
        )
        .await
    }

    /// Deletes a vector by ID.
    ///
    /// # Errors
    /// Returns error if deletion fails.
    pub async fn delete(
        &mut self,
        id: u32,
        collection: Option<String>,
    ) -> Result<bool, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DeleteRequest {
            id,
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.delete(req).await?;
        Ok(resp.into_inner().success)
    }

    /// Returns a graph node with adjacency on a specific layer.
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn get_node(
        &mut self,
        id: u32,
        layer: u32,
        collection: Option<String>,
    ) -> Result<GraphNode, tonic::Status> {
        let req = GetNodeRequest {
            collection: collection.unwrap_or_default(),
            id,
            layer,
        };
        let resp = self.inner.get_node(req).await?;
        Ok(resp.into_inner())
    }

    /// Returns neighbors for a node with pagination.
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn get_neighbors(
        &mut self,
        id: u32,
        layer: u32,
        limit: u32,
        offset: u32,
        collection: Option<String>,
    ) -> Result<GetNeighborsResponse, tonic::Status> {
        let req = GetNeighborsRequest {
            collection: collection.unwrap_or_default(),
            id,
            layer,
            limit,
            offset,
        };
        let resp = self.inner.get_neighbors(req).await?;
        Ok(resp.into_inner())
    }

    /// Returns neighbors with aligned edge weights (distance to source).
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn get_neighbors_with_weights(
        &mut self,
        id: u32,
        layer: u32,
        limit: u32,
        offset: u32,
        collection: Option<String>,
    ) -> Result<Vec<(GraphNode, f64)>, tonic::Status> {
        let resp = self
            .get_neighbors(id, layer, limit, offset, collection)
            .await?;
        let mut out = Vec::with_capacity(resp.neighbors.len());
        for (idx, node) in resp.neighbors.into_iter().enumerate() {
            let w = resp.edge_weights.get(idx).copied().unwrap_or_default();
            out.push((node, w));
        }
        Ok(out)
    }

    /// Traverses graph from a start node with depth and node guards.
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn traverse(
        &mut self,
        req: TraverseRequest,
    ) -> Result<TraverseResponse, tonic::Status> {
        let resp = self.inner.traverse(req).await?;
        Ok(resp.into_inner())
    }

    /// Finds connected components as semantic clusters.
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn find_semantic_clusters(
        &mut self,
        req: FindSemanticClustersRequest,
    ) -> Result<FindSemanticClustersResponse, tonic::Status> {
        let resp = self.inner.find_semantic_clusters(req).await?;
        Ok(resp.into_inner())
    }

    /// Returns parent-like neighbors for concept-style traversals.
    ///
    /// # Errors
    /// Returns error if request fails.
    pub async fn get_concept_parents(
        &mut self,
        id: u32,
        layer: u32,
        limit: u32,
        collection: Option<String>,
    ) -> Result<GetConceptParentsResponse, tonic::Status> {
        let req = GetConceptParentsRequest {
            collection: collection.unwrap_or_default(),
            id,
            layer,
            limit,
        };
        let resp = self.inner.get_concept_parents(req).await?;
        Ok(resp.into_inner())
    }

    /// Subscribes to CDC event stream (`VectorInserted`/`VectorDeleted`).
    ///
    /// # Errors
    /// Returns error if stream initialization fails.
    pub async fn subscribe_to_events(
        &mut self,
        types: Vec<EventType>,
        collection: Option<String>,
    ) -> Result<tonic::Streaming<EventMessage>, tonic::Status> {
        let req = EventSubscriptionRequest {
            types: types.into_iter().map(|t| t as i32).collect(),
            collection,
        };
        let resp = self.inner.subscribe_to_events(req).await?;
        Ok(resp.into_inner())
    }

    /// Configures collection parameters.
    ///
    /// # Errors
    /// Returns error if configuration fails.
    pub async fn configure(
        &mut self,
        ef_search: Option<u32>,
        ef_construction: Option<u32>,
        collection: Option<String>,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::ConfigUpdate {
            ef_search,
            ef_construction,
            collection: collection.unwrap_or_default(),
            m: None,
        };
        let resp = self.inner.configure(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Gets collection digest (hash and count).
    ///
    /// # Errors
    /// Returns error if retrieval fails.
    pub async fn get_digest(
        &mut self,
        collection: Option<String>,
    ) -> Result<hyperspace_proto::hyperspace::DigestResponse, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DigestRequest {
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.get_digest(req).await?;
        Ok(resp.into_inner())
    }

    /// Performs Delta Sync Handshake with the server.
    ///
    /// # Errors
    /// Returns error on network failure.
    pub async fn sync_handshake(
        &mut self,
        collection: String,
        client_buckets: Vec<u64>,
        client_logical_clock: u64,
        client_count: u64,
    ) -> Result<hyperspace_proto::hyperspace::SyncHandshakeResponse, tonic::Status> {
        if client_buckets.len() != 256 {
            return Err(tonic::Status::invalid_argument(
                "client_buckets must contain exactly 256 elements",
            ));
        }
        let req = hyperspace_proto::hyperspace::SyncHandshakeRequest {
            collection,
            client_buckets,
            client_logical_clock,
            client_count,
        };
        let resp = self.inner.sync_handshake(req).await?;
        Ok(resp.into_inner())
    }

    /// Returns the subsumption tree (directed hierarchy) starting from a root node.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn get_subsumption_tree(
        &mut self,
        root_id: u32,
        max_depth: u32,
        collection: Option<String>,
    ) -> Result<Vec<GraphNode>, tonic::Status> {
        let req = GetSubsumptionTreeRequest {
            collection: collection.unwrap_or_default(),
            root_id,
            max_depth,
        };
        let resp = self.inner.get_subsumption_tree(req).await?;
        Ok(resp.into_inner().nodes)
    }

    /// High-level wrapper that explores the graph and returns an ego-graph structure.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn explore_graph(
        &mut self,
        start_id: u32,
        max_depth: u32,
        max_nodes: u32,
        collection: Option<String>,
    ) -> Result<EgoGraph, tonic::Status> {
        let req = TraverseRequest {
            start_id,
            max_depth,
            max_nodes,
            layer: 0,
            collection: collection.unwrap_or_default(),
            traversal_mode: 1, // DIFFUSIVE
            breadth_limit: 10,
            filter: std::collections::HashMap::new(),
            filters: vec![],
        };
        let resp = self.traverse(req).await?;

        let nodes = resp
            .nodes
            .into_iter()
            .map(|n| EgoGraphNode {
                id: n.id,
                metadata: n.metadata,
                edge_types: n.edge_types,
            })
            .collect();

        Ok(EgoGraph {
            center_id: start_id,
            nodes,
        })
    }

    /// Pulls differing subset of vectors from server bucket indices.
    ///
    /// # Errors
    /// Returns error if stream initialization fails.
    pub async fn sync_pull(
        &mut self,
        collection: String,
        bucket_indices: Vec<u32>,
    ) -> Result<tonic::Streaming<hyperspace_proto::hyperspace::SyncVectorData>, tonic::Status> {
        let req = hyperspace_proto::hyperspace::SyncPullRequest {
            collection,
            bucket_indices,
        };
        let resp = self.inner.sync_pull(req).await?;
        Ok(resp.into_inner())
    }

    /// Pushes offline delta back to the server.
    ///
    /// # Errors
    /// Returns error if push stream initialization fails.
    pub async fn sync_push(
        &mut self,
        stream: impl tonic::IntoStreamingRequest<Message = hyperspace_proto::hyperspace::SyncVectorData>,
    ) -> Result<hyperspace_proto::hyperspace::SyncPushResponse, tonic::Status> {
        let resp = self.inner.sync_push(stream).await?;
        Ok(resp.into_inner())
    }

    /// Triggers a snapshot of the database.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn trigger_snapshot(&mut self) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::Empty {};
        let resp = self.inner.trigger_snapshot(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Checks if a collection exists.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn exists(&mut self, name: String) -> Result<bool, tonic::Status> {
        match self.get_collection_stats(name).await {
            Ok(_) => Ok(true),
            Err(s) if s.code() == tonic::Code::NotFound => Ok(false),
            Err(s) => Err(s),
        }
    }

    /// Bulk retrieval of vectors by ID.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn get_points(
        &mut self,
        ids: Vec<u32>,
        collection: String,
    ) -> Result<Vec<hyperspace_proto::hyperspace::VectorData>, tonic::Status> {
        let req = GetPointsRequest { ids, collection };
        let resp = self.inner.get_points(req).await?;
        Ok(resp.into_inner().points)
    }

    /// Thread-safe metadata patching without re-indexing.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn update_payload(
        &mut self,
        id: u32,
        metadata: std::collections::HashMap<String, String>,
        collection: String,
    ) -> Result<String, tonic::Status> {
        let req = UpdatePayloadRequest {
            id,
            metadata,
            collection,
            typed_metadata: std::collections::HashMap::new(),
        };
        let resp = self.inner.update_payload(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Pagination/iteration for database scanning.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn scroll(
        &mut self,
        limit: u32,
        offset: u32,
        filters: Vec<hyperspace_proto::hyperspace::Filter>,
        collection: String,
    ) -> Result<Vec<hyperspace_proto::hyperspace::VectorData>, tonic::Status> {
        let req = ScrollRequest {
            limit,
            offset,
            filters,
            collection,
        };
        let resp = self.inner.scroll(req).await?;
        Ok(resp.into_inner().points)
    }

    /// Filtered counting of points.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn count(
        &mut self,
        filters: Vec<hyperspace_proto::hyperspace::Filter>,
        collection: String,
    ) -> Result<u64, tonic::Status> {
        let req = CountRequest {
            filters,
            collection,
        };
        let resp = self.inner.count(req).await?;
        Ok(resp.into_inner().count)
    }

    /// Operational status monitoring.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn health_check(&mut self) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::Empty {};
        let resp = self.inner.health_check(req).await?;
        Ok(resp.into_inner().status)
    }

    /// Streams system metrics.
    ///
    /// # Errors
    /// Returns error if the RPC call fails.
    pub async fn monitor(
        &mut self,
    ) -> Result<tonic::Streaming<hyperspace_proto::hyperspace::SystemStats>, tonic::Status> {
        let req = hyperspace_proto::hyperspace::MonitorRequest {};
        let resp = self.inner.monitor(req).await?;
        Ok(resp.into_inner())
    }

    /// Evaluates a fuzzy logic search query locally by issuing batch search RPCs
    /// and scoring candidates with TNorms/TConorms.
    ///
    /// # Errors
    /// Returns error if the batch search fails.
    pub async fn search_fuzzy(
        &mut self,
        query: &fuzzy::FuzzyQuery,
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<(u32, f32)>, tonic::Status> {
        let mut vectors = Vec::new();
        query.extract_vectors(&mut vectors);

        if vectors.is_empty() {
            return Ok(Vec::new());
        }

        // Search with an oversample factor to try to collect all relevant overlapping candidates
        let batch_results = self.search_batch(vectors, top_k * 5, collection).await?;

        // Extract distances into a structured map: NodeId -> HashMap<QueryIdx, Distance>
        let mut node_dists: std::collections::HashMap<u32, std::collections::HashMap<usize, f32>> =
            std::collections::HashMap::new();
        #[allow(clippy::cast_possible_truncation)]
        for (q_idx, results) in batch_results.iter().enumerate() {
            for res in results {
                node_dists
                    .entry(res.id)
                    .or_default()
                    .insert(q_idx, res.distance as f32);
            }
        }

        let mut scored_nodes = Vec::with_capacity(node_dists.len());

        for (id, dists) in node_dists {
            let mut eval_idx: usize = 0;
            // Unretrieved elements are assumed infinitely far (producing minimum membership)
            let score = query.evaluate_indexed(&mut eval_idx, &|idx| {
                *dists.get(&idx).unwrap_or(&(1000.0)) // very large distance fallback
            });
            scored_nodes.push((id, score));
        }

        scored_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored_nodes.truncate(usize::try_from(top_k).unwrap_or(usize::MAX));
        Ok(scored_nodes)
    }

    pub fn register_collection_key(
        &self,
        collection_name: String,
        key: String,
        metric: String,
        noise_sigma: f64,
        schema: Option<CollectionSchema>,
    ) {
        self.collection_keys.write().insert(collection_name.clone(), key);
        self.collection_metrics.write().insert(collection_name.clone(), metric);
        self.collection_noise_sigmas.write().insert(collection_name.clone(), noise_sigma);
        if let Some(s) = schema {
            self.collection_schemas.write().insert(collection_name.clone(), s);
        }
        self.encryption_contexts.write().remove(&collection_name);
    }

    fn derive_keys(&self, password: &str, collection_name: &str) -> (Vec<u8>, Vec<u8>) {
        use sha2::{Sha256, Digest};
        use pbkdf2::pbkdf2;

        let salt_hash = Sha256::digest(collection_name.as_bytes());
        let salt = salt_hash.as_slice();

        let mut aes_key = vec![0u8; 32];
        pbkdf2::<hmac::Hmac<Sha256>>(password.as_bytes(), salt, 100000, &mut aes_key).unwrap();

        let mut hmac_key = vec![0u8; 32];
        pbkdf2::<hmac::Hmac<Sha256>>(password.as_bytes(), salt, 100000, &mut hmac_key).unwrap();

        (aes_key, hmac_key)
    }

    fn encrypt_payload(&self, plaintext: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, String> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
        use rand::RngExt;
        use sha2::Sha256;
        use pbkdf2::pbkdf2;

        let mut pbkdf2_salt = vec![0u8; 16];
        rand::rng().fill(&mut pbkdf2_salt);

        let mut derived_key = vec![0u8; 32];
        pbkdf2::<hmac::Hmac<Sha256>>(aes_key, &pbkdf2_salt, 100000, &mut derived_key).unwrap();

        let cipher = Aes256Gcm::new_from_slice(&derived_key).map_err(|e| e.to_string())?;

        let mut iv = vec![0u8; 12];
        rand::rng().fill(&mut iv);

        let ciphertext_and_tag = cipher
            .encrypt(aes_gcm::Nonce::from_slice(&iv), plaintext)
            .map_err(|e| e.to_string())?;

        let mut result = Vec::with_capacity(16 + 12 + ciphertext_and_tag.len());
        result.extend_from_slice(&pbkdf2_salt);
        result.extend_from_slice(&iv);
        result.extend_from_slice(&ciphertext_and_tag);
        Ok(result)
    }

    fn decrypt_payload(&self, data: &[u8], aes_key: &[u8]) -> Result<Vec<u8>, String> {
        use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
        use sha2::Sha256;
        use pbkdf2::pbkdf2;

        if data.len() < 16 + 12 + 16 {
            return Err("Invalid encrypted payload size".to_string());
        }

        let pbkdf2_salt = &data[0..16];
        let iv = &data[16..28];
        let ciphertext_and_tag = &data[28..];

        let mut derived_key = vec![0u8; 32];
        pbkdf2::<hmac::Hmac<Sha256>>(aes_key, pbkdf2_salt, 100000, &mut derived_key).unwrap();

        let cipher = Aes256Gcm::new_from_slice(&derived_key).map_err(|e| e.to_string())?;

        cipher
            .decrypt(aes_gcm::Nonce::from_slice(iv), ciphertext_and_tag)
            .map_err(|e| e.to_string())
    }

    fn hash_metadata_key(&self, key: &str, hmac_key: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(hmac_key).unwrap();
        mac.update(key.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());
        format!("tag_{}", &hash[0..16])
    }

    fn hash_metadata_value(&self, value: &str, hmac_key: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        let mut mac = Hmac::<Sha256>::new_from_slice(hmac_key).unwrap();
        mac.update(value.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());
        format!("val_{}", hash)
    }

    async fn get_encryption_context(&mut self, collection: &str, vector_dim: usize, metric: &str) -> Option<std::sync::Arc<EncryptionContext>> {
        if collection.is_empty() {
            return None;
        }

        let key = {
            let keys = self.collection_keys.read();
            keys.get(collection).cloned()?
        };

        let schema_needed = {
            let schemas = self.collection_schemas.read();
            !schemas.contains_key(collection)
        };
        if schema_needed {
            if let Ok(stats) = self.get_collection_stats(collection.to_string()).await {
                if let Some(s) = stats.schema {
                    self.collection_schemas.write().insert(collection.to_string(), s);
                }
            }
        }

        let context = {
            let mut contexts = self.encryption_contexts.write();
            if !contexts.contains_key(collection) {
                let (aes, hmac) = self.derive_keys(&key, collection);
                contexts.insert(collection.to_string(), std::sync::Arc::new(EncryptionContext {
                    aes_key: aes,
                    hmac_key: hmac,
                    projection_matrices: parking_lot::RwLock::new(std::collections::HashMap::new()),
                }));
            }
            contexts.get(collection).cloned().unwrap()
        };

        if vector_dim > 0 {
            let cache_key = vector_dim.to_string();
            let matrix_needed = {
                let matrices = context.projection_matrices.read();
                !matrices.contains_key(&cache_key)
            };
            if matrix_needed {
                let is_lorentz = metric.eq_ignore_ascii_case("lorentz") || metric.eq_ignore_ascii_case("poincare");
                let matrix_dim = if metric.eq_ignore_ascii_case("poincare") { vector_dim + 1 } else { vector_dim };
                let matrix = if is_lorentz {
                    math::generate_lorentz_matrix(matrix_dim, &context.hmac_key)
                } else {
                    math::generate_orthogonal_matrix(matrix_dim, &context.hmac_key)
                };
                context.projection_matrices.write().insert(cache_key, matrix);
            }
        }

        Some(context)
    }

    fn project_single_block(&self, sub_vec: &[f64], metric: &str, context: &EncryptionContext, block_id: Option<&str>) -> Vec<f64> {
        let dim = sub_vec.len();
        if dim == 0 {
            return Vec::new();
        }

        let cache_key = match block_id {
            Some(bid) => format!("{}_{}", dim, bid),
            None => dim.to_string(),
        };

        let matrix_needed = {
            let matrices = context.projection_matrices.read();
            !matrices.contains_key(&cache_key)
        };
        if matrix_needed {
            let is_lorentz = metric.eq_ignore_ascii_case("lorentz") || metric.eq_ignore_ascii_case("poincare");
            let matrix_dim = if metric.eq_ignore_ascii_case("poincare") { dim + 1 } else { dim };

            let seed = match block_id {
                Some(bid) => {
                    use sha2::{Sha256, Digest};
                    let mut hasher = Sha256::new();
                    hasher.update(&context.hmac_key);
                    hasher.update(bid.as_bytes());
                    hasher.finalize().to_vec()
                }
                None => context.hmac_key.clone(),
            };

            let matrix = if is_lorentz {
                math::generate_lorentz_matrix(matrix_dim, &seed)
            } else {
                math::generate_orthogonal_matrix(matrix_dim, &seed)
            };
            context.projection_matrices.write().insert(cache_key.clone(), matrix);
        }

        let matrices = context.projection_matrices.read();
        let matrix = matrices.get(&cache_key).unwrap();

        if metric.eq_ignore_ascii_case("poincare") {
            let lorentz = math::poincare_to_lorentz_f64(sub_vec);
            let proj_lorentz = math::project_vector(&lorentz, matrix);
            math::lorentz_to_poincare_f64(&proj_lorentz)
        } else {
            math::project_vector(sub_vec, matrix)
        }
    }

    fn project_collection_vector(&self, collection: &str, vector: &[f64], context: &EncryptionContext, metric: &str) -> Vec<f64> {
        let schemas = self.collection_schemas.read();
        let schema = match schemas.get(collection) {
            Some(s) => s,
            None => return self.project_single_block(vector, metric, context, None),
        };

        if schema.components.is_empty() {
            return self.project_single_block(vector, metric, context, None);
        }

        let mut component_cutoffs = std::collections::HashMap::<String, Vec<usize>>::new();
        for layer in &schema.cascade_pipeline {
            if !layer.component_name.is_empty() && layer.cutoff_dimension > 0 {
                component_cutoffs
                    .entry(layer.component_name.clone())
                    .or_default()
                    .push(layer.cutoff_dimension as usize);
            }
        }

        for cutoffs in component_cutoffs.values_mut() {
            cutoffs.sort_unstable();
            cutoffs.dedup();
        }

        let mut projected_parts = Vec::new();
        let mut current_offset = 0;

        for comp in &schema.components {
            let comp_name = &comp.name;
            let comp_metric = &comp.metric;
            let comp_dim = comp.full_dimension as usize;

            if current_offset >= vector.len() {
                break;
            }

            let mut end = current_offset + comp_dim;
            if end > vector.len() {
                end = vector.len();
            }
            let mut sub_vec = vector[current_offset..end].to_vec();
            if sub_vec.len() < comp_dim {
                sub_vec.resize(comp_dim, 0.0);
            }

            let cutoffs = component_cutoffs.get(comp_name);
            let valid_cutoffs: Vec<usize> = match cutoffs {
                Some(c) => c.iter().copied().filter(|&x| x < comp_dim).collect(),
                None => Vec::new(),
            };

            let proj_sub = if valid_cutoffs.is_empty() {
                self.project_single_block(&sub_vec, comp_metric, context, None)
            } else {
                let mut p_sub = Vec::new();
                let mut block_start = 0;
                for &cutoff in &valid_cutoffs {
                    let block_data = &sub_vec[block_start..cutoff];
                    let bid = format!("{}_block_{}_{}", comp_name, block_start, cutoff);
                    p_sub.extend(self.project_single_block(block_data, comp_metric, context, Some(&bid)));
                    block_start = cutoff;
                }
                if block_start < comp_dim {
                    let block_data = &sub_vec[block_start..comp_dim];
                    let bid = format!("{}_block_{}_{}", comp_name, block_start, comp_dim);
                    p_sub.extend(self.project_single_block(block_data, comp_metric, context, Some(&bid)));
                }
                p_sub
            };

            projected_parts.extend(proj_sub);
            current_offset += comp_dim;
        }

        if current_offset < vector.len() {
            projected_parts.extend_from_slice(&vector[current_offset..]);
        }

        projected_parts
    }

    fn encrypt_filters(&self, filters: Vec<Filter>, context: &EncryptionContext) -> Vec<Filter> {
        use hyperspace_proto::hyperspace::filter::Condition;

        filters
            .into_iter()
            .map(|f| {
                let mut nf = Filter::default();
                if let Some(cond) = f.condition {
                    match cond {
                        Condition::Match(m) => {
                            nf.condition = Some(Condition::Match(Match {
                                key: self.hash_metadata_key(&m.key, &context.hmac_key),
                                value: self.hash_metadata_value(&m.value, &context.hmac_key),
                            }));
                        }
                        Condition::Prefix(p) => {
                            nf.condition = Some(Condition::Prefix(Prefix {
                                key: self.hash_metadata_key(&p.key, &context.hmac_key),
                                prefix: self.hash_metadata_value(&p.prefix, &context.hmac_key),
                            }));
                        }
                        Condition::AndOp(a) => {
                            nf.condition = Some(Condition::AndOp(FilterAnd {
                                conditions: self.encrypt_filters(a.conditions, context),
                            }));
                        }
                        Condition::OrOp(o) => {
                            nf.condition = Some(Condition::OrOp(FilterOr {
                                conditions: self.encrypt_filters(o.conditions, context),
                            }));
                        }
                        Condition::NotOp(n) => {
                            let inner_filter = if let Some(cond_inner) = n.condition {
                                self.encrypt_filters(vec![*cond_inner], context).remove(0)
                            } else {
                                Filter::default()
                            };
                            nf.condition = Some(Condition::NotOp(Box::new(FilterNot {
                                condition: Some(Box::new(inner_filter)),
                            })));
                        }
                        _ => {
                            nf.condition = Some(cond);
                        }
                    }
                }
                nf
            })
            .collect()
    }
}
