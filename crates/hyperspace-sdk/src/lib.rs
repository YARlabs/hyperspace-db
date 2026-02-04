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
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(key) = &self.api_key {
            let token = key
                .parse()
                .map_err(|_| Status::invalid_argument("Invalid API Key format"))?;
            request.metadata_mut().insert("x-api-key", token);
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
    pub async fn connect(
        dst: String,
        api_key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(dst)?.connect().await?;
        let interceptor = AuthInterceptor { api_key };
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

    pub async fn delete_collection(&mut self, name: String) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DeleteCollectionRequest { name };
        let resp = self.inner.delete_collection(req).await?;
        Ok(resp.into_inner().status)
    }

    pub async fn list_collections(&mut self) -> Result<Vec<String>, tonic::Status> {
        let req = hyperspace_proto::hyperspace::Empty {};
        let resp = self.inner.list_collections(req).await?;
        Ok(resp.into_inner().collections)
    }

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
            origin_node_id: "".to_string(),
            logical_clock: 0,
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
    }

    pub async fn search(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        collection: Option<String>,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        let req = SearchRequest {
            vector,
            top_k,
            filter: Default::default(),
            filters: vec![],
            hybrid_query: None,
            hybrid_alpha: None,
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }

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
            filter: Default::default(),
            filters,
            hybrid_query,
            hybrid_alpha,
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }

    pub async fn delete(&mut self, id: u32, collection: Option<String>) -> Result<bool, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DeleteRequest { 
            id,
            collection: collection.unwrap_or_default(),
        };
        let resp = self.inner.delete(req).await?;
        Ok(resp.into_inner().success)
    }

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
}
