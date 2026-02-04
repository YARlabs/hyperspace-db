use hyperspace_proto::hyperspace::database_client::DatabaseClient;
use hyperspace_proto::hyperspace::{CreateCollectionRequest, InsertRequest, SearchRequest};
use tonic::transport::Channel;

const COLLECTION_NAME: &str = "test_l2_check";

use tonic::{Request, Status, service::Interceptor};

struct AuthInterceptor;
impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.metadata_mut().insert("x-api-key", "I_LOVE_HYPERSPACEDB".parse().unwrap());
        Ok(request)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting L2 Integrity Test");
    let channel = Channel::from_static("http://127.0.0.1:50051").connect().await?;
    let mut client = DatabaseClient::with_interceptor(channel, AuthInterceptor);

    // 1. Create Collection
    println!("✨ Creating collection '{}'...", COLLECTION_NAME);
    let _ = client.create_collection(CreateCollectionRequest {
        name: COLLECTION_NAME.to_string(),
        dimension: 1024,
        metric: "l2".to_string(),
    }).await.ok(); 

    // 2. Insert Vectors
    // Vec A: Origin [0, 0, ...]
    let vec_a = vec![0.0; 1024];
    client.insert(InsertRequest {
        vector: vec_a.clone(),
        id: 1,
        metadata: std::collections::HashMap::new(),
        collection: COLLECTION_NAME.to_string(),
    }).await?;

    // Vec B: [0.5, 0, ...] -> Distance L2 sq = 0.25
    let mut vec_b = vec![0.0; 1024];
    vec_b[0] = 0.5;
    client.insert(InsertRequest {
        vector: vec_b,
        id: 2,
        metadata: std::collections::HashMap::new(),
        collection: COLLECTION_NAME.to_string(),
    }).await?;

    // Vec C: [0.8, 0, ...] -> Distance from A sq = 0.64
    let mut vec_c = vec![0.0; 1024];
    vec_c[0] = 0.8;
    client.insert(InsertRequest {
        vector: vec_c,
        id: 3,
        metadata: std::collections::HashMap::new(),
        collection: COLLECTION_NAME.to_string(),
    }).await?;

    println!("📦 Vectors inserted.");

    // 3. Search
    println!("🔍 Searching for Origin...");
    let response = client.search(SearchRequest {
        vector: vec_a,
        top_k: 3,
        collection: COLLECTION_NAME.to_string(),
        filter: Default::default(),
        filters: vec![],
        hybrid_query: None,
        hybrid_alpha: None,
    }).await?;

    let results = response.into_inner().results;
    println!("Results: {:?}", results);

    // Validate (Note: Server uses internal auto-increment IDs starting at 0)
    
    // Internal ID 1 = Vec B (0.5) -> Dist 0.25
    let found_1 = results.iter().find(|r| r.id == 1).expect("Should find Internal ID 1");
    // Tolerance due to quantization (1/127 ~ 0.008)
    assert!((found_1.distance - 0.25).abs() < 0.02, "Internal ID 1 distance should be 0.25, got {}", found_1.distance);

    // Internal ID 2 = Vec C (0.8) -> Dist 0.64
    let found_2 = results.iter().find(|r| r.id == 2).expect("Should find Internal ID 2");
    assert!((found_2.distance - 0.64).abs() < 0.02, "Internal ID 2 distance should be 0.64, got {}", found_2.distance);

    println!("✅ L2 Squared Distance Verification PASSED!");
    Ok(())
}
