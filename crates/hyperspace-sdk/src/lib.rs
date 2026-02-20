pub use hyperspace_proto::hyperspace::database_client::DatabaseClient;
pub use hyperspace_proto::hyperspace::{
    BatchInsertRequest, BatchSearchRequest, DurabilityLevel, EventMessage,
    EventSubscriptionRequest, EventType, FindSemanticClustersRequest, FindSemanticClustersResponse,
    GetConceptParentsRequest, GetConceptParentsResponse, GetNeighborsRequest, GetNeighborsResponse,
    GetNodeRequest, GraphNode, InsertRequest, SearchRequest, SearchResponse, SearchResult,
    TraverseRequest, TraverseResponse, VectorData,
};
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

pub mod math;

#[cfg(feature = "embedders")]
mod embedder;
#[cfg(feature = "embedders")]
pub use embedder::*;

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

pub struct Client {
    inner: DatabaseClient<InterceptedService<Channel, AuthInterceptor>>,
    #[cfg(feature = "embedders")]
    embedder: Option<Box<dyn Embedder>>,
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
        dimension: u32,
        metric: String,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::CreateCollectionRequest {
            name,
            dimension,
            metric,
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

    /// Lists all collections.
    ///
    /// # Errors
    /// Returns error on network failure.
    pub async fn list_collections(&mut self) -> Result<Vec<String>, tonic::Status> {
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
        let req = InsertRequest {
            id,
            vector,
            metadata,
            typed_metadata: std::collections::HashMap::new(),
            collection: collection.unwrap_or_default(),
            origin_node_id: String::new(),
            logical_clock: 0,
            durability: 0,
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
    pub async fn search(
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
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }

    /// Searches using f32 query vector (converted to protocol f64 once).
    ///
    /// # Errors
    /// Returns error if search fails.
    pub async fn search_f32(
        &mut self,
        vector: &[f32],
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        self.search(Self::vec_f32_to_f64(vector), top_k, collection)
            .await
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
                collection: collection_name.clone(),
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

    /// Advanced search with filters and hybrid query.
    ///
    /// # Errors
    /// Returns error if search fails.
    pub async fn search_advanced(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        filters: Vec<hyperspace_proto::hyperspace::Filter>,
        hybrid: Option<(String, f32)>,
        collection: Option<String>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let (hybrid_query, hybrid_alpha) = match hybrid {
            Some((q, a)) => (Some(q), Some(a)),
            None => (None, None),
        };

        let req = SearchRequest {
            vector,
            top_k,
            filter: std::collections::HashMap::default(),
            filters,
            hybrid_query,
            hybrid_alpha,
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
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
}
