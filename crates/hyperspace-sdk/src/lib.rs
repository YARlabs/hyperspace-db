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

    pub async fn insert(&mut self, id: u32, vector: Vec<f64>) -> Result<bool, tonic::Status> {
        let req = InsertRequest {
            id,
            vector,
            metadata: Default::default(),
        };
        let resp = self.inner.insert(req).await?;
        Ok(resp.into_inner().success)
    }

    pub async fn search(&mut self, vector: Vec<f64>, top_k: u32) -> Result<Vec<SearchResult>, tonic::Status> {
        let req = SearchRequest {
            vector,
            top_k,
            filter: Default::default(),
            filters: Default::default(),
        };
        let resp = self.inner.search(req).await?;
        Ok(resp.into_inner().results)
    }
}
