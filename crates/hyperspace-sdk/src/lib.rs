pub use hyperspace_proto::hyperspace::database_client::DatabaseClient;
pub use hyperspace_proto::hyperspace::{InsertRequest, SearchRequest, SearchResult};
use tonic::codegen::InterceptedService;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

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
    /// Connects to the `HyperspaceDB` server.
    ///
    /// # Errors
    /// Returns error if connection fails.
    pub async fn connect(
        dst: String,
        api_key: Option<String>,
        user_id: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(dst)?.connect().await?;
        let interceptor = AuthInterceptor { api_key, user_id };
        let client = DatabaseClient::with_interceptor(channel, interceptor);
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
        let req = hyperspace_proto::hyperspace::RebuildIndexRequest { name };
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
            collection: collection.unwrap_or_default(),
            origin_node_id: String::new(),
            logical_clock: 0,
            durability: 0,
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
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
