pub use hyperspace_proto::hyperspace::database_client::DatabaseClient;
pub use hyperspace_proto::hyperspace::{InsertRequest, SearchRequest, SearchResult};
use tonic::transport::Channel;

pub struct Client {
    inner: DatabaseClient<Channel>,
}

impl Client {
    pub async fn connect(dst: String) -> Result<Self, tonic::transport::Error> {
        let client = DatabaseClient::connect(dst).await?;
        Ok(Self { inner: client })
    }

    pub async fn insert(&mut self, id: u32, vector: Vec<f64>, metadata: std::collections::HashMap<String, String>) -> Result<bool, tonic::Status> {
        let req = InsertRequest {
            id,
            vector,
            metadata,
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
    }

    pub async fn search(&mut self, vector: Vec<f64>, top_k: u32) -> Result<Vec<SearchResult>, tonic::Status> {
        self.search_advanced(vector, top_k, vec![], None).await
    }

    pub async fn search_advanced(
        &mut self, 
        vector: Vec<f64>, 
        top_k: u32, 
        filters: Vec<hyperspace_proto::hyperspace::Filter>, 
        hybrid: Option<(String, f32)>
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
}
