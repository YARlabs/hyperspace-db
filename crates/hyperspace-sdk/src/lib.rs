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

    pub async fn insert(
        &mut self,
        id: u32,
        vector: Vec<f64>,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<bool, tonic::Status> {
        let req = InsertRequest {
            id,
            vector,
            metadata,
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
    }

    #[cfg(feature = "embedders")]
    pub async fn insert_document(
        &mut self,
        id: u32,
        document: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(embedder) = &self.embedder {
            let vector = embedder.encode(document).await?;
            self.insert(id, vector, metadata)
                .await
                .map_err(|e| e.into())
        } else {
            Err("No embedder configured".into())
        }
    }

    pub async fn search(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
    ) -> Result<Vec<SearchResult>, tonic::Status> {
        self.search_advanced(vector, top_k, vec![], None).await
    }

    #[cfg(feature = "embedders")]
    pub async fn search_document(
        &mut self,
        query_text: &str,
        top_k: u32,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        if let Some(embedder) = &self.embedder {
            let vector = embedder.encode(query_text).await?;
            self.search_advanced(vector, top_k, vec![], None)
                .await
                .map_err(|e| e.into())
        } else {
            Err("No embedder configured".into())
        }
    }

    pub async fn search_advanced(
        &mut self,
        vector: Vec<f64>,
        top_k: u32,
        filters: Vec<hyperspace_proto::hyperspace::Filter>,
        hybrid: Option<(String, f32)>,
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
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }
    pub async fn delete(&mut self, id: u32) -> Result<bool, tonic::Status> {
        let req = hyperspace_proto::hyperspace::DeleteRequest { id };
        let resp = self.inner.delete(req).await?;
        Ok(resp.into_inner().success)
    }

    pub async fn trigger_snapshot(&mut self) -> Result<String, tonic::Status> {
        let resp = self
            .inner
            .trigger_snapshot(hyperspace_proto::hyperspace::Empty {})
            .await?;
        Ok(resp.into_inner().status)
    }

    pub async fn configure(
        &mut self,
        ef_search: Option<u32>,
        ef_construction: Option<u32>,
    ) -> Result<String, tonic::Status> {
        let req = hyperspace_proto::hyperspace::ConfigUpdate {
            ef_search,
            ef_construction,
        };
        let resp = self.inner.configure(req).await?;
        Ok(resp.into_inner().status)
    }
}
