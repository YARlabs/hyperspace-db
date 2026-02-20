#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::future_not_send)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::type_complexity)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unused_async)]

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use clap::Parser;
// Access index via CollectionManager.
// use hyperspace_index::HnswIndex;

mod collection;
mod http_server;
mod manager;
mod sync;
#[cfg(test)]
mod tests;
use manager::CollectionManager;

#[cfg(feature = "embed")]
use hyperspace_embed::{ApiProvider, Metric, OnnxVectorizer, RemoteVectorizer, Vectorizer};
use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{
    metadata_value, BatchInsertRequest, BatchSearchRequest, BatchSearchResponse,
    CollectionStatsRequest, CollectionStatsResponse, ConfigUpdate, CreateCollectionRequest,
    DeleteCollectionRequest, DeleteRequest, DeleteResponse, DigestRequest, DigestResponse,
    EventMessage, EventSubscriptionRequest, EventType, Filter, FindSemanticClustersRequest,
    FindSemanticClustersResponse, GetConceptParentsRequest, GetConceptParentsResponse,
    GetNeighborsRequest, GetNeighborsResponse, GetNodeRequest, GraphCluster, GraphNode,
    InsertRequest, InsertResponse, InsertTextRequest, ListCollectionsResponse, MetadataValue,
    MonitorRequest, SearchRequest, SearchResponse, SearchResult, SystemStats, TraverseRequest,
    TraverseResponse, VectorDeletedEvent, VectorInsertedEvent,
};
use hyperspace_proto::hyperspace::{replication_log, Empty, ReplicationLog};

use sha2::{Digest, Sha256};
use std::collections::HashSet;
#[cfg(feature = "embed")]
use std::str::FromStr;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{service::Interceptor, transport::Server, Request, Response, Status};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on (gRPC)
    #[arg(short, long, default_value = "50051")]
    port: u16,

    /// HTTP Dashboard Port
    #[arg(long, default_value = "50050")]
    http_port: u16,

    /// Role: leader or follower
    #[arg(long, default_value = "leader")]
    role: String,

    /// Leader address (if follower)
    #[arg(long)]
    leader: Option<String>,
}

#[derive(Clone)]
struct AuthInterceptor {
    expected_hash: Option<String>,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(expected) = &self.expected_hash {
            match request.metadata().get("x-api-key") {
                Some(t) => {
                    // Hash the received token
                    if let Ok(token_str) = t.to_str() {
                        let mut hasher = Sha256::new();
                        hasher.update(token_str.as_bytes());
                        let result = hasher.finalize();
                        let request_hash = hex::encode(result);

                        // Constant-time comparison to prevent timing attacks
                        if constant_time_eq(request_hash.as_bytes(), expected.as_bytes()) {
                            return Ok(request);
                        }
                    }
                    Err(Status::unauthenticated("Invalid API Key"))
                }
                None => Err(Status::unauthenticated("Missing x-api-key header")),
            }
        } else {
            Ok(request)
        }
    }
}

/// Constant-time comparison for byte slices
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut res = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        res |= x ^ y;
    }
    res == 0
}

// Client-side interceptor (for Follower connecting to Leader)
#[derive(Clone)]
struct ClientAuthInterceptor {
    api_key: String,
}

fn default_ef_search() -> usize {
    static DEFAULT_EF_SEARCH: OnceLock<usize> = OnceLock::new();
    *DEFAULT_EF_SEARCH.get_or_init(|| {
        std::env::var("HS_HNSW_EF_SEARCH")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100)
    })
}

fn range_bounds_f64(r: &hyperspace_proto::hyperspace::Range) -> (Option<f64>, Option<f64>) {
    let gte = r.gte_f64.or(r.gte.map(|v| v as f64));
    let lte = r.lte_f64.or(r.lte.map(|v| v as f64));
    (gte, lte)
}

fn build_filters(
    req: SearchRequest,
) -> (
    String,
    Vec<f64>,
    std::collections::HashMap<String, String>,
    Vec<hyperspace_core::FilterExpr>,
    hyperspace_core::SearchParams,
) {
    let col_name = if req.collection.is_empty() {
        "default".to_string()
    } else {
        req.collection
    };

    let exact_filter = req.filter.into_iter().collect();
    let mut complex_filters = Vec::new();
    for f in req.filters {
        if let Some(cond) = f.condition {
            match cond {
                hyperspace_proto::hyperspace::filter::Condition::Match(m) => {
                    complex_filters.push(hyperspace_core::FilterExpr::Match {
                        key: m.key,
                        value: m.value,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::Range(r) => {
                    let (gte, lte) = range_bounds_f64(&r);
                    complex_filters.push(hyperspace_core::FilterExpr::Range {
                        key: r.key,
                        gte,
                        lte,
                    });
                }
            }
        }
    }

    let params = hyperspace_core::SearchParams {
        top_k: req.top_k as usize,
        ef_search: default_ef_search(),
        hybrid_query: req.hybrid_query,
        hybrid_alpha: req.hybrid_alpha,
    };

    (col_name, req.vector, exact_filter, complex_filters, params)
}

const TYPED_META_PREFIX: &str = "__hs_typed__";

fn metadata_value_to_shadow_json(v: &MetadataValue) -> Option<String> {
    match &v.kind {
        Some(metadata_value::Kind::StringValue(x)) => {
            Some(serde_json::json!({"t":"s","v":x}).to_string())
        }
        Some(metadata_value::Kind::IntValue(x)) => {
            Some(serde_json::json!({"t":"i","v":x}).to_string())
        }
        Some(metadata_value::Kind::DoubleValue(x)) => {
            Some(serde_json::json!({"t":"f","v":x}).to_string())
        }
        Some(metadata_value::Kind::BoolValue(x)) => {
            Some(serde_json::json!({"t":"b","v":x}).to_string())
        }
        None => None,
    }
}

fn shadow_json_to_metadata_value(s: &str) -> Option<MetadataValue> {
    let json: serde_json::Value = serde_json::from_str(s).ok()?;
    let kind = json.get("t")?.as_str()?;
    let value = json.get("v")?;
    let out = match kind {
        "s" => MetadataValue {
            kind: Some(metadata_value::Kind::StringValue(
                value.as_str()?.to_string(),
            )),
        },
        "i" => MetadataValue {
            kind: Some(metadata_value::Kind::IntValue(value.as_i64()?)),
        },
        "f" => MetadataValue {
            kind: Some(metadata_value::Kind::DoubleValue(value.as_f64()?)),
        },
        "b" => MetadataValue {
            kind: Some(metadata_value::Kind::BoolValue(value.as_bool()?)),
        },
        _ => return None,
    };
    Some(out)
}

fn merge_metadata(
    mut base: std::collections::HashMap<String, String>,
    typed: std::collections::HashMap<String, MetadataValue>,
) -> std::collections::HashMap<String, String> {
    for (key, value) in typed {
        if let Some(shadow) = metadata_value_to_shadow_json(&value) {
            base.insert(format!("{TYPED_META_PREFIX}{key}"), shadow);
        }
        match value.kind {
            Some(metadata_value::Kind::StringValue(v)) => {
                base.insert(key, v);
            }
            Some(metadata_value::Kind::IntValue(v)) => {
                base.insert(key, v.to_string());
            }
            Some(metadata_value::Kind::DoubleValue(v)) => {
                base.insert(key, v.to_string());
            }
            Some(metadata_value::Kind::BoolValue(v)) => {
                base.insert(key, v.to_string());
            }
            None => {}
        }
    }
    base
}

fn strip_internal_metadata(
    metadata: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    metadata
        .iter()
        .filter(|(k, _)| !k.starts_with(TYPED_META_PREFIX))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn extract_typed_metadata(
    metadata: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, MetadataValue> {
    let mut typed = std::collections::HashMap::new();
    for (k, v) in metadata {
        if let Some(raw_key) = k.strip_prefix(TYPED_META_PREFIX) {
            if let Some(parsed) = shadow_json_to_metadata_value(v) {
                typed.insert(raw_key.to_string(), parsed);
            }
        }
    }
    typed
}

fn build_graph_node(
    col: &Arc<dyn hyperspace_core::Collection>,
    id: u32,
    layer: usize,
) -> GraphNode {
    let metadata = col.metadata_by_id(id);
    let typed_metadata = extract_typed_metadata(&metadata);
    let plain_metadata = strip_internal_metadata(&metadata);
    let neighbors = col
        .graph_neighbors(id, layer, usize::MAX)
        .unwrap_or_default();
    GraphNode {
        id,
        layer: layer as u32,
        neighbors,
        metadata: plain_metadata,
        typed_metadata,
    }
}

fn matches_filter_exprs(
    metadata: &std::collections::HashMap<String, String>,
    exact_filter: &std::collections::HashMap<String, String>,
    complex_filters: &[hyperspace_core::FilterExpr],
) -> bool {
    let meta_numeric = |key: &str| -> Option<f64> {
        if let Some(raw) = metadata.get(key) {
            return raw.parse::<f64>().ok();
        }
        let typed_key = format!("{TYPED_META_PREFIX}{key}");
        let raw_typed = metadata.get(&typed_key)?;
        let parsed = serde_json::from_str::<serde_json::Value>(raw_typed).ok()?;
        parsed.get("v")?.as_f64()
    };

    for (k, v) in exact_filter {
        match metadata.get(k) {
            Some(actual) if actual == v => {}
            _ => return false,
        }
    }

    for expr in complex_filters {
        match expr {
            hyperspace_core::FilterExpr::Match { key, value } => match metadata.get(key) {
                Some(actual) if actual == value => {}
                _ => return false,
            },
            hyperspace_core::FilterExpr::Range { key, gte, lte } => {
                let Some(num) = meta_numeric(key) else {
                    return false;
                };
                if let Some(min) = gte {
                    if num < *min {
                        return false;
                    }
                }
                if let Some(max) = lte {
                    if num > *max {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn parse_graph_filters(
    exact_filter: std::collections::HashMap<String, String>,
    filters: Vec<Filter>,
) -> (
    std::collections::HashMap<String, String>,
    Vec<hyperspace_core::FilterExpr>,
) {
    let mut complex_filters = Vec::new();
    for f in filters {
        if let Some(cond) = f.condition {
            match cond {
                hyperspace_proto::hyperspace::filter::Condition::Match(m) => {
                    complex_filters.push(hyperspace_core::FilterExpr::Match {
                        key: m.key,
                        value: m.value,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::Range(r) => {
                    let (gte, lte) = range_bounds_f64(&r);
                    complex_filters.push(hyperspace_core::FilterExpr::Range {
                        key: r.key,
                        gte,
                        lte,
                    });
                }
            }
        }
    }
    (exact_filter, complex_filters)
}

#[allow(clippy::result_large_err)]
fn parse_vacuum_filter(
    filter: Option<hyperspace_proto::hyperspace::VacuumFilterQuery>,
) -> Result<Option<hyperspace_core::VacuumFilterQuery>, Status> {
    let Some(filter) = filter else {
        return Ok(None);
    };
    let op = match filter.op.to_lowercase().as_str() {
        "lt" => hyperspace_core::VacuumFilterOp::Lt,
        "lte" => hyperspace_core::VacuumFilterOp::Lte,
        "gt" => hyperspace_core::VacuumFilterOp::Gt,
        "gte" => hyperspace_core::VacuumFilterOp::Gte,
        "eq" => hyperspace_core::VacuumFilterOp::Eq,
        "ne" => hyperspace_core::VacuumFilterOp::Ne,
        other => {
            return Err(Status::invalid_argument(format!(
                "Unsupported vacuum filter op '{other}', use lt/lte/gt/gte/eq/ne"
            )))
        }
    };
    Ok(Some(hyperspace_core::VacuumFilterQuery {
        key: filter.key,
        op,
        value: filter.value,
    }))
}

impl Interceptor for ClientAuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = self
            .api_key
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid API Key format"))?;
        request.metadata_mut().insert("x-api-key", token);
        Ok(request)
    }
}

fn get_user_id<T>(req: &Request<T>) -> String {
    req.metadata()
        .get("x-hyperspace-user-id")
        .and_then(|v| v.to_str().ok())
        .map_or_else(
            || "default_admin".to_string(),
            std::string::ToString::to_string,
        )
}

pub struct HyperspaceService {
    manager: Arc<CollectionManager>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    role: String,
    #[cfg(feature = "embed")]
    vectorizer: Option<Arc<dyn Vectorizer>>,
}

#[tonic::async_trait]
impl Database for HyperspaceService {
    // --- Collection Management ---

    async fn create_collection(
        &self,
        request: Request<CreateCollectionRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        if req.name.is_empty() {
            return Err(Status::invalid_argument("Collection name cannot be empty"));
        }

        // Map string metric to internal
        // Manager accepts string metric.
        match self
            .manager
            .create_collection(&user_id, &req.name, req.dimension, &req.metric)
            .await
        {
            Ok(()) => Ok(Response::new(
                hyperspace_proto::hyperspace::StatusResponse {
                    status: format!("Collection '{}' created.", req.name),
                },
            )),
            Err(e) => Err(Status::already_exists(e)),
        }
    }

    async fn delete_collection(
        &self,
        request: Request<DeleteCollectionRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        match self.manager.delete_collection(&user_id, &req.name).await {
            Ok(()) => Ok(Response::new(
                hyperspace_proto::hyperspace::StatusResponse {
                    status: format!("Collection '{}' deleted.", req.name),
                },
            )),
            Err(e) => Err(Status::not_found(e)),
        }
    }

    async fn list_collections(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<ListCollectionsResponse>, Status> {
        let user_id = get_user_id(&_request);
        let list = self.manager.list(&user_id);
        Ok(Response::new(ListCollectionsResponse { collections: list }))
    }

    async fn get_collection_stats(
        &self,
        request: Request<CollectionStatsRequest>,
    ) -> Result<Response<CollectionStatsResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        if let Some(col) = self.manager.get(&user_id, &req.name).await {
            // TODO: Extend Collection trait to expose dimension and metric.
            // For now return dummy or count.
            Ok(Response::new(CollectionStatsResponse {
                count: col.count() as u64,
                dimension: 0, // TODO: Expose from trait
                metric: "unknown".into(),
                indexing_queue: col.queue_size(),
            }))
        } else {
            Err(Status::not_found("Collection not found"))
        }
    }

    // --- Data Plane ---

    async fn insert(
        &self,
        request: Request<InsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        if self.role == "follower" {
            return Err(Status::permission_denied("Followers are read-only"));
        }
        let user_id = get_user_id(&request);
        let req = request.into_inner();

        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        if let Some(col) = self.manager.get(&user_id, &col_name).await {
            let meta = merge_metadata(
                req.metadata.into_iter().collect(),
                req.typed_metadata.into_iter().collect(),
            );
            // Tick clock
            let clock = self.manager.tick_cluster_clock().await;

            // Durability mapping
            let durability = match hyperspace_proto::hyperspace::DurabilityLevel::try_from(
                req.durability,
            )
            .ok()
            {
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Strict) => {
                    hyperspace_core::Durability::Strict
                }
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Async) => {
                    hyperspace_core::Durability::Async
                }
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Batch) => {
                    hyperspace_core::Durability::Batch
                }
                _ => hyperspace_core::Durability::Default,
            };

            // id is u32 in proto.
            if let Err(e) = col
                .insert(&req.vector, req.id, meta, clock, durability)
                .await
            {
                return Err(Status::internal(e));
            }
            Ok(Response::new(InsertResponse { success: true }))
        } else {
            Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )))
        }
    }

    async fn batch_insert(
        &self,
        request: Request<BatchInsertRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        if self.role == "follower" {
            return Err(Status::permission_denied("Followers are read-only"));
        }
        let user_id = get_user_id(&request);
        let req = request.into_inner();

        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if let Some(col) = self.manager.get(&user_id, &col_name).await {
            // Convert protos to internal types
            let vectors: Vec<(Vec<f64>, u32, std::collections::HashMap<String, String>)> = req
                .vectors
                .into_iter()
                .map(|v| {
                    (
                        v.vector,
                        v.id,
                        merge_metadata(v.metadata.into_iter().collect(), v.typed_metadata),
                    )
                })
                .collect();

            // Tick clock
            let clock = self.manager.tick_cluster_clock().await;

            // Durability mapping
            let durability = match hyperspace_proto::hyperspace::DurabilityLevel::try_from(
                req.durability,
            )
            .ok()
            {
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Strict) => {
                    hyperspace_core::Durability::Strict
                }
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Async) => {
                    hyperspace_core::Durability::Async
                }
                Some(hyperspace_proto::hyperspace::DurabilityLevel::Batch) => {
                    hyperspace_core::Durability::Batch
                }
                _ => hyperspace_core::Durability::Default,
            };

            if let Err(e) = col.insert_batch(vectors, clock, durability).await {
                return Err(Status::internal(e));
            }
            Ok(Response::new(InsertResponse { success: true }))
        } else {
            Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )))
        }
    }

    #[allow(unused_variables)]
    async fn insert_text(
        &self,
        request: Request<InsertTextRequest>,
    ) -> Result<Response<InsertResponse>, Status> {
        #[cfg(feature = "embed")]
        {
            if self.role == "follower" {
                return Err(Status::permission_denied("Followers are read-only"));
            }
            let user_id = get_user_id(&request);
            let req = request.into_inner();

            if let Some(vectorizer) = &self.vectorizer {
                let vectors = vectorizer
                    .vectorize(vec![req.text])
                    .await
                    .map_err(|e| Status::internal(format!("Embedding failed: {e}")))?;

                if vectors.is_empty() {
                    return Err(Status::internal("Empty vector result"));
                }
                let vector = vectors[0].clone();

                let col_name = if req.collection.is_empty() {
                    "default".to_string()
                } else {
                    req.collection
                };

                if let Some(col) = self.manager.get(&user_id, &col_name).await {
                    let meta: std::collections::HashMap<String, String> =
                        req.metadata.into_iter().collect();
                    let clock = self.manager.tick_cluster_clock().await;

                    // Durability mapping
                    let durability = match hyperspace_proto::hyperspace::DurabilityLevel::try_from(
                        req.durability,
                    )
                    .ok()
                    {
                        Some(hyperspace_proto::hyperspace::DurabilityLevel::Strict) => {
                            hyperspace_core::Durability::Strict
                        }
                        Some(hyperspace_proto::hyperspace::DurabilityLevel::Async) => {
                            hyperspace_core::Durability::Async
                        }
                        Some(hyperspace_proto::hyperspace::DurabilityLevel::Batch) => {
                            hyperspace_core::Durability::Batch
                        }
                        _ => hyperspace_core::Durability::Default,
                    };

                    if let Err(e) = col.insert(&vector, req.id, meta, clock, durability).await {
                        return Err(Status::internal(e));
                    }
                    return Ok(Response::new(InsertResponse { success: true }));
                }

                return Err(Status::not_found(format!(
                    "Collection '{col_name}' not found"
                )));
            }

            return Err(Status::unimplemented(
                "Server configured without embedding model",
            ));
        }
        #[cfg(not(feature = "embed"))]
        return Err(Status::unimplemented("Embedding feature not compiled"));
    }

    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<DeleteResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if let Some(col) = self.manager.get(&user_id, &col_name).await {
            if let Err(e) = col.delete(req.id) {
                return Err(Status::internal(e));
            }
            if self.replication_tx.receiver_count() > 0 {
                let clock = self.manager.tick_cluster_clock().await;
                let log = ReplicationLog {
                    logical_clock: clock,
                    origin_node_id: self.manager.cluster_state.read().await.node_id.clone(),
                    collection: col_name.clone(),
                    operation: Some(replication_log::Operation::Delete(
                        hyperspace_proto::hyperspace::DeleteOp { id: req.id },
                    )),
                };
                let _ = self.replication_tx.send(log);
            }
            Ok(Response::new(DeleteResponse { success: true }))
        } else {
            Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )))
        }
    }

    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let user_id = get_user_id(&request);
        let (col_name, vector, exact_filter, complex_filters, params) =
            build_filters(request.into_inner());

        if let Some(col) = self.manager.get(&user_id, &col_name).await {
            match col
                .search(&vector, &exact_filter, &complex_filters, &params)
                .await
            {
                Ok(res) => {
                    let output = res
                        .into_iter()
                        .map(|(id, dist, meta)| {
                            let typed_metadata = extract_typed_metadata(&meta);
                            let metadata = strip_internal_metadata(&meta);
                            SearchResult {
                                id,
                                distance: dist,
                                metadata,
                                typed_metadata,
                            }
                        })
                        .collect();
                    Ok(Response::new(SearchResponse { results: output }))
                }
                Err(e) => Err(Status::internal(e)),
            }
        } else {
            Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )))
        }
    }

    async fn search_batch(
        &self,
        request: Request<BatchSearchRequest>,
    ) -> Result<Response<BatchSearchResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();

        let mut responses = Vec::with_capacity(req.searches.len());
        for search_req in req.searches {
            let (col_name, vector, exact_filter, complex_filters, params) =
                build_filters(search_req);
            let col =
                self.manager.get(&user_id, &col_name).await.ok_or_else(|| {
                    Status::not_found(format!("Collection '{col_name}' not found"))
                })?;

            let res = col
                .search(&vector, &exact_filter, &complex_filters, &params)
                .await
                .map_err(Status::internal)?;

            let results = res
                .into_iter()
                .map(|(id, dist, meta)| {
                    let typed_metadata = extract_typed_metadata(&meta);
                    let metadata = strip_internal_metadata(&meta);
                    SearchResult {
                        id,
                        distance: dist,
                        metadata,
                        typed_metadata,
                    }
                })
                .collect();
            responses.push(SearchResponse { results });
        }

        Ok(Response::new(BatchSearchResponse { responses }))
    }

    async fn get_node(
        &self,
        request: Request<GetNodeRequest>,
    ) -> Result<Response<GraphNode>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        let layer = req.layer as usize;
        let Some(col) = self.manager.get(&user_id, &col_name).await else {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        };
        let node = build_graph_node(&col, req.id, layer);
        Ok(Response::new(node))
    }

    async fn get_neighbors(
        &self,
        request: Request<GetNeighborsRequest>,
    ) -> Result<Response<GetNeighborsResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        let layer = req.layer as usize;
        let limit = if req.limit == 0 {
            64
        } else {
            req.limit as usize
        };
        let offset = req.offset as usize;
        let Some(col) = self.manager.get(&user_id, &col_name).await else {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        };
        let fetch_limit = limit.saturating_add(offset);
        let mut ids = col
            .graph_neighbors(req.id, layer, fetch_limit)
            .map_err(Status::invalid_argument)?;
        if offset > 0 {
            ids = ids.into_iter().skip(offset).collect();
        }
        if ids.len() > limit {
            ids.truncate(limit);
        }
        let edge_weights = col
            .graph_neighbor_distances(req.id, &ids)
            .map_err(Status::internal)?;
        let neighbors = ids
            .into_iter()
            .map(|id| build_graph_node(&col, id, layer))
            .collect();
        Ok(Response::new(GetNeighborsResponse {
            neighbors,
            edge_weights,
        }))
    }

    async fn get_concept_parents(
        &self,
        request: Request<GetConceptParentsRequest>,
    ) -> Result<Response<GetConceptParentsResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        let layer = req.layer as usize;
        let limit = if req.limit == 0 {
            32
        } else {
            req.limit as usize
        };
        let Some(col) = self.manager.get(&user_id, &col_name).await else {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        };
        let upper_layer = layer.saturating_add(1);
        let (ids, resolved_layer) = match col.graph_neighbors(req.id, upper_layer, limit) {
            Ok(ids) => (ids, upper_layer),
            Err(_) => (
                col.graph_neighbors(req.id, layer, limit)
                    .map_err(Status::invalid_argument)?,
                layer,
            ),
        };
        let parents = ids
            .into_iter()
            .map(|id| build_graph_node(&col, id, resolved_layer))
            .collect();
        Ok(Response::new(GetConceptParentsResponse { parents }))
    }

    async fn traverse(
        &self,
        request: Request<TraverseRequest>,
    ) -> Result<Response<TraverseResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        let layer = req.layer as usize;
        let max_depth = if req.max_depth == 0 {
            2
        } else {
            req.max_depth as usize
        };
        let max_nodes = if req.max_nodes == 0 {
            256
        } else {
            req.max_nodes as usize
        };
        let (exact_filter, complex_filters) =
            parse_graph_filters(req.filter.into_iter().collect(), req.filters);
        let Some(col) = self.manager.get(&user_id, &col_name).await else {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        };
        let mut ids = col
            .graph_traverse(req.start_id, layer, max_depth, max_nodes)
            .map_err(Status::invalid_argument)?;
        if !exact_filter.is_empty() || !complex_filters.is_empty() {
            ids.retain(|id| {
                let meta = col.metadata_by_id(*id);
                matches_filter_exprs(&meta, &exact_filter, &complex_filters)
            });
        }
        let nodes = ids
            .into_iter()
            .map(|id| build_graph_node(&col, id, layer))
            .collect();
        Ok(Response::new(TraverseResponse { nodes }))
    }

    async fn find_semantic_clusters(
        &self,
        request: Request<FindSemanticClustersRequest>,
    ) -> Result<Response<FindSemanticClustersResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };
        let layer = req.layer as usize;
        let min_cluster_size = if req.min_cluster_size == 0 {
            3
        } else {
            req.min_cluster_size as usize
        };
        let max_clusters = if req.max_clusters == 0 {
            32
        } else {
            req.max_clusters as usize
        };
        let max_nodes = if req.max_nodes == 0 {
            10_000
        } else {
            req.max_nodes as usize
        };
        let Some(col) = self.manager.get(&user_id, &col_name).await else {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        };
        let clusters = col
            .graph_clusters(layer, min_cluster_size, max_clusters, max_nodes)
            .map_err(Status::internal)?
            .into_iter()
            .map(|node_ids| GraphCluster { node_ids })
            .collect();
        Ok(Response::new(FindSemanticClustersResponse { clusters }))
    }

    type MonitorStream = ReceiverStream<Result<SystemStats, Status>>;

    async fn monitor(
        &self,
        _request: Request<MonitorRequest>,
    ) -> Result<Response<Self::MonitorStream>, Status> {
        // Monitor aggregates global statistics.
        let (tx, rx) = mpsc::channel(4);
        let manager = self.manager.clone();

        tokio::spawn(async move {
            loop {
                // Aggregate stats
                // Use list_all for global stats
                let collections = manager.list_all();
                let mut total_count = 0;

                for name in &collections {
                    if let Some(c) = manager.get_internal(name).await {
                        total_count += c.count();
                    }
                }

                let stats = SystemStats {
                    total_collections: collections.len() as u64,
                    total_vectors: total_count as u64,
                    total_memory_mb: 0.0, // TODO
                    qps: 0.0,             // TODO
                };

                if tx.send(Ok(stats)).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ReplicateStream = ReceiverStream<Result<ReplicationLog, Status>>;
    type SubscribeToEventsStream = ReceiverStream<Result<EventMessage, Status>>;

    async fn get_digest(
        &self,
        request: Request<DigestRequest>,
    ) -> Result<Response<DigestResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let name = if req.collection.is_empty() {
            "default"
        } else {
            &req.collection
        };

        if let Some(col) = self.manager.get(&user_id, name).await {
            let clock = self.manager.cluster_state.read().await.logical_clock;
            Ok(Response::new(DigestResponse {
                logical_clock: clock,
                state_hash: col.state_hash(),
                buckets: col.buckets(),
                count: col.count() as u64,
            }))
        } else {
            Err(Status::not_found("Collection not found"))
        }
    }

    async fn replicate(
        &self,
        request: Request<hyperspace_proto::hyperspace::ReplicationRequest>,
    ) -> Result<Response<Self::ReplicateStream>, Status> {
        if self.role == "follower" {
            return Err(Status::failed_precondition(
                "I am a follower, cannot replicate from me",
            ));
        }

        // Extract peer address
        let peer_addr = request
            .remote_addr()
            .map_or_else(|| "unknown".to_string(), |addr| addr.to_string());

        let req = request.into_inner();
        println!(
            "📡 Follower connected: {peer_addr} (Last clock: {})",
            req.last_logical_clock
        );

        // Register follower
        {
            let mut state = self.manager.cluster_state.write().await;
            if !state.downstream_peers.contains(&peer_addr) {
                state.downstream_peers.push(peer_addr.clone());
            }
        }

        let mut rx = self.replication_tx.subscribe();
        let (tx, out_rx) = mpsc::channel(100);
        let manager = self.manager.clone();
        let peer_addr_clone = peer_addr.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(log) => {
                        if tx.send(Ok(log)).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("⚠️ Replication stream lagged, skipped {skipped} messages");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            // Unregister on disconnect
            let mut state = manager.cluster_state.write().await;
            state.downstream_peers.retain(|p| p != &peer_addr_clone);
            println!("📡 Follower disconnected: {peer_addr_clone}");
        });

        Ok(Response::new(ReceiverStream::new(out_rx)))
    }

    async fn subscribe_to_events(
        &self,
        request: Request<EventSubscriptionRequest>,
    ) -> Result<Response<Self::SubscribeToEventsStream>, Status> {
        let req = request.into_inner();
        let wanted: HashSet<i32> = req.types.into_iter().collect();
        let filter_collection = req.collection.unwrap_or_default();
        let mut rx = self.replication_tx.subscribe();
        let (tx, out_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            loop {
                let log = match rx.recv().await {
                    Ok(log) => log,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("⚠️ Event stream lagged, skipped {skipped} messages");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };

                if !filter_collection.is_empty() && filter_collection != log.collection {
                    continue;
                }

                let event = match log.operation {
                    Some(replication_log::Operation::Insert(op)) => {
                        let ty = EventType::VectorInserted as i32;
                        if !wanted.is_empty() && !wanted.contains(&ty) {
                            continue;
                        }
                        let typed_metadata = if op.typed_metadata.is_empty() {
                            extract_typed_metadata(&op.metadata)
                        } else {
                            op.typed_metadata
                        };
                        let metadata = strip_internal_metadata(&op.metadata);
                        EventMessage {
                            r#type: ty,
                            payload: Some(hyperspace_proto::hyperspace::event_message::Payload::VectorInserted(
                                VectorInsertedEvent {
                                    id: op.id,
                                    collection: log.collection.clone(),
                                    logical_clock: log.logical_clock,
                                    origin_node_id: log.origin_node_id.clone(),
                                    metadata,
                                    typed_metadata,
                                },
                            )),
                        }
                    }
                    Some(replication_log::Operation::Delete(op)) => {
                        let ty = EventType::VectorDeleted as i32;
                        if !wanted.is_empty() && !wanted.contains(&ty) {
                            continue;
                        }
                        EventMessage {
                            r#type: ty,
                            payload: Some(
                                hyperspace_proto::hyperspace::event_message::Payload::VectorDeleted(
                                    VectorDeletedEvent {
                                        id: op.id,
                                        collection: log.collection.clone(),
                                        logical_clock: log.logical_clock,
                                        origin_node_id: log.origin_node_id.clone(),
                                    },
                                ),
                            ),
                        }
                    }
                    _ => continue,
                };

                if tx.send(Ok(event)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(out_rx)))
    }

    async fn trigger_snapshot(
        &self,
        _request: Request<hyperspace_proto::hyperspace::Empty>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        // Individual collections manage snapshots via background tasks.
        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse {
                status: "Snapshots are handled automatically by background tasks.".into(),
            },
        ))
    }

    async fn trigger_vacuum(
        &self,
        _request: Request<hyperspace_proto::hyperspace::Empty>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        // Trigger manual GC/Vacuum
        println!("🧹 Manual Vacuum Triggered: Memory cleanup initiated.");
        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse {
                status: "Memory cleanup triggered".to_string(),
            },
        ))
    }

    async fn rebuild_index(
        &self,
        request: Request<hyperspace_proto::hyperspace::RebuildIndexRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        println!("🔧 Rebuild Index Request for: '{}'", req.name);
        let vacuum_filter = parse_vacuum_filter(req.filter_query)?;

        let rebuild_res = if let Some(filter) = vacuum_filter {
            self.manager
                .rebuild_collection_with_filter(&user_id, &req.name, Some(filter))
                .await
        } else {
            self.manager.rebuild_collection(&user_id, &req.name).await
        };

        match rebuild_res {
            Ok(()) => Ok(Response::new(
                hyperspace_proto::hyperspace::StatusResponse {
                    status: "Index rebuilt and reloaded successfully".to_string(),
                },
            )),
            Err(e) => Err(Status::internal(e)),
        }
    }

    async fn configure(
        &self,
        request: Request<ConfigUpdate>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        // TODO: Update config for specific collection
        // ConfigUpdate now has `collection` field.
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        if self.manager.get(&user_id, &col_name).await.is_none() {
            return Err(Status::not_found(format!(
                "Collection '{col_name}' not found"
            )));
        }
        // Not implemented on trait yet.
        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse {
                status: "Dynamic config not yet implemented for collections".into(),
            },
        ))
    }
}

async fn start_server(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", args.port).parse()?;

    // Setup Manager
    let data_dir = std::path::PathBuf::from("data");
    let event_buffer = std::env::var("HS_EVENT_STREAM_BUFFER")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1024)
        .max(64);
    println!("⚙️ Event Stream Buffer: {event_buffer}");
    let (replication_tx, _) = broadcast::channel(event_buffer);

    let manager = Arc::new(CollectionManager::new(data_dir, replication_tx.clone()));

    // Load existing
    println!("Loading collections...");
    manager.load_existing().await?;

    // Use env vars for default
    let dim_str = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
    let dim: u32 = dim_str.parse().unwrap_or(1024);

    // Support HS_METRIC and HS_DISTANCE_METRIC (compatibility alias)
    let metric = std::env::var("HS_METRIC")
        .or_else(|_| std::env::var("HS_DISTANCE_METRIC"))
        .unwrap_or("poincare".to_string())
        .to_lowercase();

    println!("🚀 Booting HyperspaceDB | Dim: {dim} | Metric: {metric}");

    // Create default collection if not exists
    if manager.get("default_admin", "default").await.is_none() {
        println!("Creating default collection...");
        manager
            .create_collection("default_admin", "default", dim, &metric)
            .await?;
    }

    // Follower Logic
    if args.role == "follower" {
        if let Some(leader) = args.leader.clone() {
            println!("🚀 Starting as FOLLOWER of: {leader}");
            let manager_weak = Arc::downgrade(&manager);
            let api_key_for_client = std::env::var("HYPERSPACE_API_KEY").ok();

            tokio::spawn(async move {
                use hyperspace_proto::hyperspace::database_client::DatabaseClient;
                use tonic::transport::Channel;

                loop {
                    println!("Connecting to leader {leader}...");
                    match Channel::from_shared(leader.clone())
                        .expect("Invalid leader URL")
                        .connect()
                        .await
                    {
                        Ok(channel) => {
                            let interceptor = ClientAuthInterceptor {
                                api_key: api_key_for_client.clone().unwrap_or_default(),
                            };
                            let mut client = DatabaseClient::with_interceptor(channel, interceptor);

                            println!("Connected! Requesting replication stream...");
                            let current_clock = manager_weak.upgrade().map_or(0, |m| {
                                futures::executor::block_on(async {
                                    m.cluster_state.read().await.logical_clock
                                })
                            });

                            let req = hyperspace_proto::hyperspace::ReplicationRequest {
                                last_logical_clock: current_clock,
                            };

                            match client.replicate(req).await {
                                Ok(resp) => {
                                    let mut stream = resp.into_inner();
                                    while let Ok(Some(log)) = stream.message().await {
                                        if let Some(mgr) = manager_weak.upgrade() {
                                            let col_name = if log.collection.is_empty() {
                                                "default"
                                            } else {
                                                &log.collection
                                            };

                                            // Merge clock
                                            mgr.merge_cluster_clock(log.logical_clock).await;

                                            match log.operation {
                                                Some(replication_log::Operation::Insert(op)) => {
                                                    // Use get_internal for replication
                                                    if let Some(col) =
                                                        mgr.get_internal(col_name).await
                                                    {
                                                        let merged_meta = merge_metadata(
                                                            op.metadata.into_iter().collect(),
                                                            op.typed_metadata,
                                                        );
                                                        if let Err(e) = col.insert(
                                                            &op.vector,
                                                            op.id,
                                                            merged_meta,
                                                            log.logical_clock,
                                                            hyperspace_core::Durability::Default,
                                                        ).await {
                                                            eprintln!("Rep Error: {e}");
                                                        }
                                                    } else {
                                                        eprintln!("Unknown collection for insert: {col_name}");
                                                    }
                                                }
                                                Some(
                                                    replication_log::Operation::CreateCollection(
                                                        op,
                                                    ),
                                                ) => {
                                                    println!("Rep: Creating collection {col_name}");
                                                    if let Err(e) = mgr
                                                        .create_collection_from_replication(
                                                            col_name,
                                                            op.dimension,
                                                            &op.metric,
                                                        )
                                                        .await
                                                    {
                                                        eprintln!("Rep Error (Create): {e}");
                                                    }
                                                }
                                                Some(
                                                    replication_log::Operation::DeleteCollection(_),
                                                ) => {
                                                    println!("Rep: Deleting collection {col_name}");
                                                    if let Err(e) = mgr
                                                        .delete_collection_from_replication(
                                                            col_name,
                                                        )
                                                        .await
                                                    {
                                                        eprintln!("Rep Error (Delete): {e}");
                                                    }
                                                }
                                                Some(replication_log::Operation::Delete(op)) => {
                                                    if let Some(col) =
                                                        mgr.get_internal(col_name).await
                                                    {
                                                        let _ = col.delete(op.id);
                                                    }
                                                }
                                                None => {}
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed: {e}"),
                            }
                        }
                        Err(e) => eprintln!("Conn failed: {e}"),
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            });
        }
    } else {
        println!("🚀 Starting as LEADER");
    }

    // 1. Initialize Vectorizer (Moved before HTTP Server)
    #[cfg(feature = "embed")]
    let vectorizer: Option<Arc<dyn Vectorizer>> = {
        // 1. Check explicit enable/disable
        let enabled = std::env::var("HYPERSPACE_EMBED")
            .unwrap_or_else(|_| "true".to_string())
            .to_lowercase()
            == "true";

        if enabled {
            let provider_str = std::env::var("HYPERSPACE_EMBED_PROVIDER")
                .unwrap_or_else(|_| "local".to_string())
                .to_lowercase();

            let metric_str = std::env::var("HS_METRIC")
                .or_else(|_| std::env::var("HS_DISTANCE_METRIC"))
                .unwrap_or("poincare".to_string())
                .to_lowercase();

            let metric = match metric_str.as_str() {
                "poincare" | "hyperbolic" => Metric::Poincare,
                "cosine" | "l2" | "euclidean" => Metric::L2,
                _ => Metric::None,
            };

            // 2. Validate Configuration
            if provider_str == "local" && metric != Metric::Poincare {
                eprintln!("⚠️ CRITICAL CONFIG CONFLICT:");
                eprintln!("   Provider 'local' (Hyperbolic ONNX) requires HS_METRIC='poincare'.");
                eprintln!("   Found HS_METRIC='{metric_str}'.");
                eprintln!("🛑 Embedding Service Disabled to prevent mathematical errors.");
                None
            } else if provider_str == "local" {
                let model_path = std::env::var("HYPERSPACE_MODEL_PATH").ok();
                let tok_path = std::env::var("HYPERSPACE_TOKENIZER_PATH").ok();

                if let (Some(m), Some(t)) = (model_path, tok_path) {
                    println!("🧠 Loading Local Embedding Model: {m}");
                    let dim: usize = std::env::var("HYPERSPACE_EMBED_DIM")
                        .unwrap_or("128".to_string())
                        .parse()
                        .unwrap_or(128);

                    match OnnxVectorizer::new(&m, &t, dim, metric) {
                        Ok(v) => Some(Arc::new(v)),
                        Err(e) => {
                            eprintln!("❌ Failed to load local vectorizer: {e}");
                            None
                        }
                    }
                } else {
                    // If defaulting to local but no model path provided
                    println!(
                        "⚠️ Embedding Service needs HYPERSPACE_MODEL_PATH for 'local' provider."
                    );
                    None
                }
            } else {
                // Remote
                if let Ok(provider) = ApiProvider::from_str(&provider_str) {
                    let api_key = std::env::var("HYPERSPACE_API_KEY_EMBED")
                        .or_else(|_| std::env::var("OPENAI_API_KEY"))
                        .unwrap_or_default();

                    let model = std::env::var("HYPERSPACE_EMBED_MODEL")
                        .unwrap_or("text-embedding-3-small".to_string());

                    let base_url = std::env::var("HYPERSPACE_API_BASE").ok();

                    println!("☁️ Using Remote Embeddings: {provider:?} | Model: {model}");
                    Some(Arc::new(RemoteVectorizer::new(
                        provider, api_key, model, base_url,
                    )))
                } else {
                    eprintln!("❌ Unknown embedding provider: {provider_str}");
                    None
                }
            }
        } else {
            println!("🛑 Embedding Service Disabled (HYPERSPACE_EMBED=false)");
            None
        }
    };

    // 2. Prepare Embedding Info for Dashboard
    let embedding_info = {
        #[cfg(feature = "embed")]
        {
            if let Some(v) = &vectorizer {
                let provider =
                    std::env::var("HYPERSPACE_EMBED_PROVIDER").unwrap_or("local".to_string());
                let model =
                    std::env::var("HYPERSPACE_EMBED_MODEL").unwrap_or("default".to_string());
                Some(http_server::EmbeddingInfo {
                    enabled: true,
                    provider,
                    model,
                    dimension: v.dimension(), // Requires Vectorizer trait to expose dimension()
                })
            } else {
                // Check if it was explicitly disabled or failed
                let enabled_flag = std::env::var("HYPERSPACE_EMBED")
                    .unwrap_or("true".to_string())
                    .to_lowercase()
                    == "true";

                if enabled_flag {
                    None // Failed to load
                } else {
                    Some(http_server::EmbeddingInfo {
                        enabled: false,
                        provider: "-".into(),
                        model: "-".into(),
                        dimension: 0,
                    })
                }
            }
        }
        #[cfg(not(feature = "embed"))]
        None
    };

    // 3. Start HTTP Dashboard
    let http_mgr = manager.clone();
    let http_port = args.http_port;
    tokio::spawn(async move {
        if let Err(e) = http_server::start_http_server(http_mgr, http_port, embedding_info).await {
            eprintln!("HTTP Server panicked: {e}");
        }
    });

    let service = HyperspaceService {
        manager,
        replication_tx,
        role: args.role,
        #[cfg(feature = "embed")]
        vectorizer,
    };

    println!("HyperspaceDB listening on {addr}");

    // Setup Auth
    let api_key = std::env::var("HYPERSPACE_API_KEY").ok();
    let interceptor = if let Some(key) = api_key {
        println!("🔒 API Auth Enabled");
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        AuthInterceptor {
            expected_hash: Some(hash),
        }
    } else {
        println!("⚠️ API Auth Disabled");
        AuthInterceptor {
            expected_hash: None,
        }
    };

    // Limit GRPC message size
    let max_msg_size = 64 * 1024 * 1024; // 64 MB

    let db_service = DatabaseServer::new(service)
        .max_decoding_message_size(max_msg_size)
        .max_encoding_message_size(max_msg_size);

    let service_with_auth =
        tonic::service::interceptor::InterceptedService::new(db_service, interceptor);

    Server::builder()
        .add_service(service_with_auth)
        .serve_with_shutdown(addr, async {
            tokio::signal::ctrl_c().await.ok();
            println!("\n🛑 Received Ctrl+C. Initiating graceful shutdown...");
        })
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[H] HyperspaceDB Server v2.0.0");
    hyperspace_core::check_simd();

    dotenv::dotenv().ok();
    let args = Args::parse();
    start_server(args).await
}
