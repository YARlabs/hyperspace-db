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

mod chunk_backend;
mod chunk_searcher;
mod collection;
mod gossip;
mod http_server;
mod manager;
mod meta_router;
mod sync;
#[cfg(test)]
mod tests;
use manager::CollectionManager;

#[cfg(feature = "embed")]
use hyperspace_embed::{ApiProvider, Metric, MultiVectorizer, OnnxVectorizer, RemoteVectorizer};
use hyperspace_proto::hyperspace::database_server::{Database, DatabaseServer};
use hyperspace_proto::hyperspace::{
    metadata_value, BatchInsertRequest, BatchSearchRequest, BatchSearchResponse,
    CollectionStatsRequest, CollectionStatsResponse, ConfigUpdate, CreateCollectionRequest,
    DeleteCollectionRequest, DeleteRequest, DeleteResponse, DiffBucket, DigestRequest,
    DigestResponse, EventMessage, EventSubscriptionRequest, EventType, Filter,
    FindSemanticClustersRequest, FindSemanticClustersResponse, GetConceptParentsRequest,
    GetConceptParentsResponse, GetNeighborsRequest, GetNeighborsResponse, GetNodeRequest,
    GraphCluster, GraphNode, InsertRequest, InsertResponse, InsertTextRequest,
    ListCollectionsResponse, MetadataValue, MonitorRequest, SearchMultiCollectionRequest,
    SearchMultiCollectionResponse, SearchRequest, SearchResponse, SearchResult, SearchTextRequest,
    SyncHandshakeRequest, SyncHandshakeResponse, SyncPullRequest, SyncPushResponse, SyncVectorData,
    SystemStats, TraverseRequest, TraverseResponse, VectorDeletedEvent, VectorInsertedEvent,
    VectorizeRequest, VectorizeResponse,
};
use hyperspace_proto::hyperspace::{replication_log, Empty, ReplicationLog};
use tonic::Streaming;

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

    /// User ID for multi-tenant replication (if follower)
    #[arg(long)]
    user_id: Option<String>,

    /// Unique Node ID for this instance
    #[arg(long)]
    node_id: Option<String>,

    /// Allow outgoing replication streams?
    #[arg(long, default_value = "false", env = "HS_REPLICATION_ALLOWED")]
    replication_allowed: bool,
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
    user_id: Option<String>,
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

fn search_batch_inner_concurrency() -> usize {
    static INNER_CONCURRENCY: OnceLock<usize> = OnceLock::new();
    *INNER_CONCURRENCY.get_or_init(|| {
        std::env::var("HS_SEARCH_BATCH_INNER_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(1)
            .min(128)
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
                hyperspace_proto::hyperspace::filter::Condition::InCone(c) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InCone {
                        axes: c.axes,
                        apertures: c.apertures,
                        cen: c.cen,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::InBox(b) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InBox {
                        min_bounds: b.min_bounds,
                        max_bounds: b.max_bounds,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::InBall(b) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InBall {
                        center: b.center,
                        radius: b.radius,
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
        use_wasserstein: req.use_wasserstein,
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
            hyperspace_core::FilterExpr::InCone { .. }
            | hyperspace_core::FilterExpr::InBox { .. }
            | hyperspace_core::FilterExpr::InBall { .. } => {
                // Vector-based filters are evaluated during search index traversal,
                // so we can't evaluate them purely on metadata. We assume match here.
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
                hyperspace_proto::hyperspace::filter::Condition::InCone(c) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InCone {
                        axes: c.axes,
                        apertures: c.apertures,
                        cen: c.cen,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::InBox(b) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InBox {
                        min_bounds: b.min_bounds,
                        max_bounds: b.max_bounds,
                    });
                }
                hyperspace_proto::hyperspace::filter::Condition::InBall(b) => {
                    complex_filters.push(hyperspace_core::FilterExpr::InBall {
                        center: b.center,
                        radius: b.radius,
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

        if let Some(uid) = &self.user_id {
            if let Ok(uid_meta) = uid.parse() {
                request.metadata_mut().insert("x-hyperspace-user-id", uid_meta);
            }
        }
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
    replication_allowed: bool,
    #[cfg(feature = "embed")]
    vectorizer: Option<Arc<MultiVectorizer>>,
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
        let collections = self.manager.list_detailed(&user_id).await;
        Ok(Response::new(ListCollectionsResponse { collections }))
    }

    async fn get_collection_stats(
        &self,
        request: Request<CollectionStatsRequest>,
    ) -> Result<Response<CollectionStatsResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        if let Some(col) = self.manager.get(&user_id, &req.name).await {
            Ok(Response::new(CollectionStatsResponse {
                count: col.count() as u64,
                dimension: col.dimension() as u32,
                metric: col.metric_name().to_string(),
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

            if let Some(multi) = &self.vectorizer {
                let col_name = if req.collection.is_empty() {
                    "default".to_string()
                } else {
                    req.collection.clone()
                };

                // Discover metric from collection to route to correct model
                let metric = if let Some(col) = self.manager.get(&user_id, &col_name).await {
                    col.metric_name().to_string()
                } else {
                    "l2".to_string()
                };

                let vectors = multi
                    .vectorize_for(vec![req.text], &metric)
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

    async fn vectorize(
        &self,
        request: Request<VectorizeRequest>,
    ) -> Result<Response<VectorizeResponse>, Status> {
        #[cfg(feature = "embed")]
        {
            let req = request.into_inner();
            if let Some(multi) = &self.vectorizer {
                let vectors = multi
                    .vectorize_for(vec![req.text], &req.metric)
                    .await
                    .map_err(|e| Status::internal(format!("Embedding failed: {e}")))?;
                if vectors.is_empty() {
                    return Err(Status::internal("Empty vector result"));
                }
                Ok(Response::new(VectorizeResponse {
                    vector: vectors[0].clone(),
                }))
            } else {
                Err(Status::failed_precondition("Embedding engine disabled"))
            }
        }
        #[cfg(not(feature = "embed"))]
        {
            let _ = request;
            Err(Status::unimplemented("Embedding feature not compiled"))
        }
    }

    async fn search_text(
        &self,
        request: Request<SearchTextRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        #[cfg(feature = "embed")]
        {
            let user_id = get_user_id(&request);
            let req = request.into_inner();

            if let Some(multi) = &self.vectorizer {
                let col_name = if req.collection.is_empty() {
                    "default".to_string()
                } else {
                    req.collection.clone()
                };

                // Discover metric from collection to route to correct model
                let metric = if let Some(col) = self.manager.get(&user_id, &col_name).await {
                    col.metric_name().to_string()
                } else {
                    "l2".to_string()
                };

                let vectors = multi
                    .vectorize_for(vec![req.text], &metric)
                    .await
                    .map_err(|e| Status::internal(format!("Embedding failed: {e}")))?;

                if vectors.is_empty() {
                    return Err(Status::internal("Empty vector result"));
                }
                let vector = vectors[0].clone();

                // Build filters and search parameters
                let exact_filter = req.filter.clone().into_iter().collect();
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
                            hyperspace_proto::hyperspace::filter::Condition::InCone(c) => {
                                complex_filters.push(hyperspace_core::FilterExpr::InCone {
                                    axes: c.axes,
                                    apertures: c.apertures,
                                    cen: c.cen,
                                });
                            }
                            hyperspace_proto::hyperspace::filter::Condition::InBox(b) => {
                                complex_filters.push(hyperspace_core::FilterExpr::InBox {
                                    min_bounds: b.min_bounds,
                                    max_bounds: b.max_bounds,
                                });
                            }
                            hyperspace_proto::hyperspace::filter::Condition::InBall(b) => {
                                complex_filters.push(hyperspace_core::FilterExpr::InBall {
                                    center: b.center,
                                    radius: b.radius,
                                });
                            }
                        }
                    }
                }

                let params = hyperspace_core::SearchParams {
                    top_k: req.top_k as usize,
                    ef_search: default_ef_search(),
                    hybrid_query: None,
                    hybrid_alpha: None,
                    use_wasserstein: false,
                };

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
            } else {
                Err(Status::failed_precondition("Embedding engine disabled"))
            }
        }
        #[cfg(not(feature = "embed"))]
        {
            let _ = request;
            Err(Status::unimplemented("Embedding feature not compiled"))
        }
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
        let inner_concurrency = search_batch_inner_concurrency();

        if inner_concurrency <= 1 {
            let mut responses = Vec::with_capacity(req.searches.len());
            for search_req in req.searches {
                let (col_name, vector, exact_filter, complex_filters, params) =
                    build_filters(search_req);
                let col = self.manager.get(&user_id, &col_name).await.ok_or_else(|| {
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
            return Ok(Response::new(BatchSearchResponse { responses }));
        }

        let total = req.searches.len();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(inner_concurrency));
        let mut tasks = tokio::task::JoinSet::new();
        for (idx, search_req) in req.searches.into_iter().enumerate() {
            let (col_name, vector, exact_filter, complex_filters, params) =
                build_filters(search_req);
            let col =
                self.manager.get(&user_id, &col_name).await.ok_or_else(|| {
                    Status::not_found(format!("Collection '{col_name}' not found"))
                })?;
            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| Status::internal(format!("search_batch semaphore error: {e}")))?;
            tasks.spawn(async move {
                let _permit = permit;
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
                Ok::<(usize, SearchResponse), Status>((idx, SearchResponse { results }))
            });
        }

        let mut ordered: Vec<Option<SearchResponse>> = vec![None; total];
        while let Some(join_res) = tasks.join_next().await {
            let task_res =
                join_res.map_err(|e| Status::internal(format!("search_batch join error: {e}")))?;
            let (idx, response) = task_res?;
            ordered[idx] = Some(response);
        }

        let mut responses = Vec::with_capacity(total);
        for item in ordered {
            let response =
                item.ok_or_else(|| Status::internal("search_batch internal ordering error"))?;
            responses.push(response);
        }

        Ok(Response::new(BatchSearchResponse { responses }))
    }

    async fn search_multi_collection(
        &self,
        request: Request<SearchMultiCollectionRequest>,
    ) -> Result<Response<SearchMultiCollectionResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let inner_concurrency = search_batch_inner_concurrency();

        let mut responses = std::collections::HashMap::new();

        if inner_concurrency <= 1 {
            for col_name in req.collections {
                let col = self.manager.get(&user_id, &col_name).await.ok_or_else(|| {
                    Status::not_found(format!("Collection '{col_name}' not found"))
                })?;
                let params = hyperspace_core::SearchParams {
                    top_k: req.top_k as usize,
                    ef_search: default_ef_search(),
                    hybrid_query: None,
                    hybrid_alpha: None,
                    use_wasserstein: false,
                };
                let exact_filter = std::collections::HashMap::new();
                let complex_filters = Vec::new();
                let res = col
                    .search(&req.vector, &exact_filter, &complex_filters, &params)
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
                responses.insert(col_name, SearchResponse { results });
            }
            return Ok(Response::new(SearchMultiCollectionResponse { responses }));
        }

        let semaphore = Arc::new(tokio::sync::Semaphore::new(inner_concurrency));
        let mut tasks = tokio::task::JoinSet::new();

        for col_name in req.collections {
            let col =
                self.manager.get(&user_id, &col_name).await.ok_or_else(|| {
                    Status::not_found(format!("Collection '{col_name}' not found"))
                })?;
            let vector = req.vector.clone();
            let top_k = req.top_k;
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                Status::internal(format!("search_multi_collection semaphore error: {e}"))
            })?;

            tasks.spawn(async move {
                let _permit = permit;
                let params = hyperspace_core::SearchParams {
                    top_k: top_k as usize,
                    ef_search: default_ef_search(),
                    hybrid_query: None,
                    hybrid_alpha: None,
                    use_wasserstein: false,
                };
                let exact_filter = std::collections::HashMap::new();
                let complex_filters = Vec::new();
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
                Ok::<_, Status>((col_name, SearchResponse { results }))
            });
        }

        while let Some(join_res) = tasks.join_next().await {
            let res = join_res.map_err(|e| {
                Status::internal(format!("search_multi_collection join error: {e}"))
            })?;
            let (col_name, response) = res?;
            responses.insert(col_name, response);
        }

        Ok(Response::new(SearchMultiCollectionResponse { responses }))
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
    type SyncPullStream = ReceiverStream<Result<SyncVectorData, Status>>;

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
        if !self.replication_allowed {
            return Err(Status::permission_denied("Replication export is disabled on this node. Set HS_REPLICATION_ALLOWED=true to enable."));
        }

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

    async fn trigger_reconsolidation(
        &self,
        request: Request<hyperspace_proto::hyperspace::ReconsolidationRequest>,
    ) -> Result<Response<hyperspace_proto::hyperspace::StatusResponse>, Status> {
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection
        };

        let lr = req.learning_rate;
        println!("🧠 Memory Reconsolidation Triggered for {col_name} (lr={lr})");

        // Future: spawn background Tokio task using hyperspace_core::optim::MemoryReconsolidator
        Ok(Response::new(
            hyperspace_proto::hyperspace::StatusResponse {
                status: "Memory reconsolidation background task scheduled".to_string(),
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

    // ─── Delta Sync RPCs (Task 2.1) ─────────────────────────────────────────

    async fn sync_handshake(
        &self,
        request: Request<SyncHandshakeRequest>,
    ) -> Result<Response<SyncHandshakeResponse>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default"
        } else {
            &req.collection
        };

        let col = self
            .manager
            .get(&user_id, col_name)
            .await
            .ok_or_else(|| Status::not_found(format!("Collection '{col_name}' not found")))?;

        let server_buckets = col.buckets();
        let server_clock = self.manager.cluster_state.read().await.logical_clock;
        let server_count = col.count() as u64;

        // Compare bucket hashes to find diffs
        let client_buckets = &req.client_buckets;
        if client_buckets.len() != server_buckets.len() {
            return Err(Status::invalid_argument(format!(
                "Bucket count mismatch: client={}, server={}",
                client_buckets.len(),
                server_buckets.len()
            )));
        }

        let mut diff_buckets = Vec::new();
        for (i, (client_hash, server_hash)) in
            client_buckets.iter().zip(server_buckets.iter()).enumerate()
        {
            if client_hash != server_hash {
                diff_buckets.push(DiffBucket {
                    bucket_index: i as u32,
                    server_hash: *server_hash,
                    client_hash: *client_hash,
                });
            }
        }

        let in_sync = diff_buckets.is_empty();
        if in_sync {
            println!("🔄 SyncHandshake: '{col_name}' is in sync (client clock={}, server clock={server_clock})", req.client_logical_clock);
        } else {
            println!(
                "🔄 SyncHandshake: '{col_name}' has {} dirty buckets (client clock={}, server clock={server_clock})",
                diff_buckets.len(),
                req.client_logical_clock
            );
        }

        Ok(Response::new(SyncHandshakeResponse {
            diff_buckets,
            server_logical_clock: server_clock,
            server_count,
            in_sync,
        }))
    }

    async fn sync_pull(
        &self,
        request: Request<SyncPullRequest>,
    ) -> Result<Response<Self::SyncPullStream>, Status> {
        let user_id = get_user_id(&request);
        let req = request.into_inner();
        let col_name = if req.collection.is_empty() {
            "default".to_string()
        } else {
            req.collection.clone()
        };

        let col = self
            .manager
            .get(&user_id, &col_name)
            .await
            .ok_or_else(|| Status::not_found(format!("Collection '{col_name}' not found")))?;

        let bucket_indices: Vec<u32> = req.bucket_indices;
        if bucket_indices.is_empty() {
            return Err(Status::invalid_argument("No bucket indices specified"));
        }

        println!(
            "📥 SyncPull: '{col_name}' pulling {} buckets: {:?}",
            bucket_indices.len(),
            &bucket_indices[..bucket_indices.len().min(10)]
        );

        // Extract vectors for the requested buckets
        let vectors = col.peek_buckets(&bucket_indices);
        let total = vectors.len();

        let (tx, rx) = mpsc::channel(256);
        let col_name_clone = col_name.clone();

        tokio::spawn(async move {
            for (id, vector, metadata) in vectors {
                let bucket_index = (id as usize % crate::sync::SYNC_BUCKETS) as u32;
                let data = SyncVectorData {
                    collection: col_name_clone.clone(),
                    id,
                    vector,
                    metadata,
                    bucket_index,
                };
                if tx.send(Ok(data)).await.is_err() {
                    break;
                }
            }
            println!("📥 SyncPull: Streamed {total} vectors for '{col_name}'");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn sync_push(
        &self,
        request: Request<Streaming<SyncVectorData>>,
    ) -> Result<Response<SyncPushResponse>, Status> {
        let user_id = get_user_id(&request);
        let mut stream = request.into_inner();

        let mut accepted = 0u32;
        let mut rejected = 0u32;
        let mut duplicates = 0u32;
        let mut target_collection: Option<String> = None;

        let clock = self.manager.tick_cluster_clock().await;

        while let Some(data) = stream.message().await? {
            let col_name = if data.collection.is_empty() {
                "default"
            } else {
                &data.collection
            };

            if target_collection.is_none() {
                target_collection = Some(col_name.to_string());
            }

            let Some(col) = self.manager.get(&user_id, col_name).await else {
                rejected += 1;
                continue;
            };

            // Check dimension match
            if data.vector.len() != col.dimension() {
                rejected += 1;
                continue;
            }

            match col
                .insert(
                    &data.vector,
                    data.id,
                    data.metadata,
                    clock,
                    hyperspace_core::Durability::Batch,
                )
                .await
            {
                Ok(()) => accepted += 1,
                Err(_) => duplicates += 1,
            }
        }

        let col_display = target_collection.as_deref().unwrap_or("unknown");
        println!(
            "📤 SyncPush: '{col_display}' accepted={accepted}, rejected={rejected}, duplicates={duplicates}"
        );

        Ok(Response::new(SyncPushResponse {
            accepted,
            rejected,
            duplicates,
        }))
    }
}

async fn start_server(args: Args) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = format!("0.0.0.0:{}", args.port).parse()?;

    // Setup Manager
    let data_dir = std::path::PathBuf::from(
        std::env::var("HS_DATA_DIR").unwrap_or_else(|_| "data".to_string()),
    );
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
                                user_id: args.user_id.clone(),
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
    //
    // Per-Metric Embedding Architecture:
    // Each geometry (l2, cosine, poincare, lorentz) can have its own embedding backend.
    // Priority per metric:
    //   1. HS_EMBED_<METRIC>_PROVIDER=local   → HYPERSPACE_<METRIC>_MODEL_PATH + TOKENIZER_PATH (filesystem ONNX)
    //   2. HS_EMBED_<METRIC>_PROVIDER=huggingface → HS_EMBED_<METRIC>_HF_MODEL_ID (downloads from HF Hub)
    //   3. HS_EMBED_<METRIC>_PROVIDER=<api>   → HYPERSPACE_API_KEY / HYPERSPACE_EMBED_MODEL (cloud API)
    //
    // Fallback: if no per-metric config exists, use the global HYPERSPACE_EMBED_PROVIDER config.
    #[cfg(feature = "embed")]
    let vectorizer: Option<Arc<MultiVectorizer>> = {
        let enabled_raw = std::env::var("HYPERSPACE_EMBED").unwrap_or_else(|_| "false".to_string());
        let enabled = enabled_raw.to_lowercase() == "true";
        println!(
            "🔍 Embedding Global Status: [HYPERSPACE_EMBED={enabled_raw}] -> enabled={enabled}"
        );

        if enabled {
            let mut multi = MultiVectorizer::new();
            for metric_name in ["l2", "cosine", "poincare", "lorentz"] {
                let metric_upper = metric_name.to_uppercase();
                let provider_key = format!("HS_EMBED_{metric_upper}_PROVIDER");
                let provider_str = std::env::var(&provider_key)
                    .or_else(|_| std::env::var("HYPERSPACE_EMBED_PROVIDER"))
                    .unwrap_or_else(|_| "disabled".to_string())
                    .to_lowercase();

                println!("   ⚙️  Checking metric: {metric_name} | Provider: {provider_str}");

                if provider_str == "disabled" || provider_str == "none" {
                    continue;
                }

                let metric = match metric_name {
                    "poincare" => Metric::Poincare,
                    "lorentz" => Metric::Lorentz,
                    "l2" => Metric::L2,
                    _ => Metric::Cosine,
                };

                if provider_str == "local" {
                    let model_path = std::env::var(format!("HS_EMBED_{metric_upper}_MODEL_PATH"))
                        .or_else(|_| std::env::var("HYPERSPACE_MODEL_PATH"))
                        .ok();
                    let tok_path = std::env::var(format!("HS_EMBED_{metric_upper}_TOKENIZER_PATH"))
                        .or_else(|_| std::env::var("HYPERSPACE_TOKENIZER_PATH"))
                        .ok();

                    if let (Some(m), Some(t)) = (model_path, tok_path) {
                        let dim: usize = std::env::var(format!("HS_EMBED_{metric_upper}_DIM"))
                            .or_else(|_| std::env::var("HYPERSPACE_EMBED_DIM"))
                            .unwrap_or_else(|_| "128".to_string())
                            .parse()
                            .unwrap_or(128);
                        println!("🧠 [{metric_upper}] Loading local ONNX model: {m} (dim={dim})");
                        if let Ok(v) = OnnxVectorizer::new(&m, &t, dim, metric, &metric_upper) {
                            multi.add(metric_name, Arc::new(v));
                        }
                    }
                } else if provider_str == "huggingface" || provider_str == "hf" {
                    let hf_model_id = std::env::var(format!("HS_EMBED_{metric_upper}_HF_MODEL_ID"))
                        .or_else(|_| std::env::var("HYPERSPACE_HF_MODEL_ID"))
                        .ok();
                    if let Some(model_id) = hf_model_id {
                        let hf_token = std::env::var("HF_TOKEN")
                            .or_else(|_| std::env::var("HUGGING_FACE_HUB_TOKEN"))
                            .ok();
                        let hf_filename =
                            std::env::var(format!("HS_EMBED_{metric_upper}_HF_FILENAME")).ok();
                        let dim: usize = std::env::var(format!("HS_EMBED_{metric_upper}_DIM"))
                            .or_else(|_| std::env::var("HYPERSPACE_EMBED_DIM"))
                            .unwrap_or_else(|_| "128".to_string())
                            .parse()
                            .unwrap_or(128);
                        println!(
                            "🤗 [{metric_upper}] Downloading HF model: {model_id} (dim={dim})"
                        );
                        if let Ok(v) = OnnxVectorizer::new_from_hf(
                            &model_id,
                            hf_token.as_deref(),
                            dim,
                            metric,
                            &metric_upper,
                            hf_filename,
                        ) {
                            multi.add(metric_name, Arc::new(v));
                        }
                    }
                } else if let Ok(provider) = ApiProvider::from_str(&provider_str) {
                    let api_key = std::env::var(format!("HS_EMBED_{metric_upper}_API_KEY"))
                        .or_else(|_| std::env::var("HYPERSPACE_API_KEY_EMBED"))
                        .or_else(|_| std::env::var("OPENAI_API_KEY"))
                        .unwrap_or_default();
                    let model = std::env::var(format!("HS_EMBED_{metric_upper}_EMBED_MODEL"))
                        .or_else(|_| std::env::var("HYPERSPACE_EMBED_MODEL"))
                        .unwrap_or_else(|_| "text-embedding-3-small".to_string());
                    let base_url = std::env::var(format!("HS_EMBED_{metric_upper}_API_BASE"))
                        .or_else(|_| std::env::var("HYPERSPACE_API_BASE"))
                        .ok();
                    println!("☁️  [{metric_upper}] Remote embedding: {provider:?} | model={model}");
                    multi.add(
                        metric_name,
                        Arc::new(RemoteVectorizer::new(provider, api_key, model, base_url)),
                    );
                }
            }
            let count = multi.models.len();
            if count == 0 {
                println!("⚠️  All configured models failed to load - Embedding Pipeline DISABLED");
                None
            } else {
                println!("✅ Embedding Pipeline ACTIVE with {count} model(s)");
                Some(Arc::new(multi))
            }
        } else {
            println!("⚠️  Embedding Pipeline is COMPRESSED into NULL (HYPERSPACE_EMBED=false or not found)");
            None
        }
    };

    // 2. Prepare Embedding Info for Dashboard
    let embedding_info = {
        #[cfg(feature = "embed")]
        {
            let mut models_map = std::collections::HashMap::new();
            let metrics = ["l2", "cosine", "poincare", "lorentz"];

            for metric in metrics {
                let status = if let Some(multi) = &vectorizer {
                    if let Some(v) = multi.models.get(metric) {
                        let m_u = metric.to_uppercase();
                        let provider = std::env::var(format!("HS_EMBED_{m_u}_PROVIDER"))
                            .or_else(|_| std::env::var("HYPERSPACE_EMBED_PROVIDER"))
                            .unwrap_or("local".to_string());
                        let model_id = std::env::var(format!("HS_EMBED_{m_u}_HF_MODEL_ID"))
                            .or_else(|_| std::env::var(format!("HS_EMBED_{m_u}_EMBED_MODEL")))
                            .or_else(|_| std::env::var("HYPERSPACE_EMBED_MODEL"))
                            .unwrap_or("default".to_string());

                        http_server::ModelStatus {
                            enabled: true,
                            provider,
                            model: model_id,
                            dimension: v.dimension(),
                        }
                    } else {
                        http_server::ModelStatus {
                            enabled: false,
                            provider: "-".into(),
                            model: "-".into(),
                            dimension: 0,
                        }
                    }
                } else {
                    http_server::ModelStatus {
                        enabled: false,
                        provider: "-".into(),
                        model: "-".into(),
                        dimension: 0,
                    }
                };
                models_map.insert(metric.to_string(), status);
            }

            Some(http_server::EmbeddingInfo {
                enabled: vectorizer.is_some(),
                models: models_map,
            })
        }
        #[cfg(not(feature = "embed"))]
        {
            println!("❌ Embedding Engine NOT COMPILED (missing --features embed)");
            None
        }
    };

    // 3. Start Gossip Engine (Task 3.4) — optional, driven by HS_GOSSIP_PEERS env var
    let http_port = args.http_port;
    let gossip_enabled =
        std::env::var("HS_GOSSIP_ENABLED").is_ok_and(|v| v.to_lowercase() == "true");

    let peer_registry: Option<gossip::PeerRegistry> = if gossip_enabled {
        let (node_id, role, logical_clock) = {
            let state = manager.cluster_state.read().await;
            (
                state.node_id.clone(),
                state.role.clone(),
                state.logical_clock,
            )
        };
        // Digests ref: empty initially, populated as insert/search ops update bucket hashes
        let digests_ref = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        // Logical clock ref: mirrors the cluster Lamport clock
        let clock_ref = Arc::new(tokio::sync::RwLock::new(logical_clock));
        let registry = gossip::start_gossip(node_id, role, http_port, clock_ref, digests_ref).await;
        Some(registry)
    } else {
        println!("ℹ️  Gossip disabled — set HS_GOSSIP_PEERS=<ip:port,...> to enable swarm mode");
        None
    };

    // 4. Start HTTP Dashboard
    let http_mgr = manager.clone();
    tokio::spawn(async move {
        if let Err(e) =
            http_server::start_http_server(http_mgr, http_port, embedding_info, peer_registry).await
        {
            eprintln!("HTTP Server panicked: {e}");
        }
    });

    let node_id = args.node_id.clone().unwrap_or_else(|| {
        uuid::Uuid::new_v4().to_string()
    });

    println!("⚡ Local Node ID: {node_id}");
    if args.replication_allowed {
        println!("📡 Replication: [ENABLED] (Accepting outgoing streams)");
    } else {
        println!("🔒 Replication: [DISABLED] (Outgoing streams blocked)");
    }

    let service = HyperspaceService {
        manager,
        replication_tx,
        role: args.role,
        replication_allowed: args.replication_allowed,
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
    println!("[H] HyperspaceDB Server v3");
    hyperspace_core::check_simd();

    dotenv::dotenv().ok();
    let args = Args::parse();
    start_server(args).await
}
