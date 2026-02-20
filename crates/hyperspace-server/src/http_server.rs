use crate::manager::CollectionManager;
use axum::{
    extract::{Extension, Path, Query, Request, State},
    http::{StatusCode, Uri},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use hyperspace_core::SearchParams;
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;
use sysinfo::Pid;
use tikv_jemalloc_ctl::epoch;
use tower_http::cors::CorsLayer;

const TYPED_META_PREFIX: &str = "__hs_typed__";

#[derive(RustEmbed)]
#[folder = "../../dashboard/dist"]
struct FrontendAssets;

// API Key validation middleware
#[derive(Clone)]
pub struct RequestContext {
    pub user_id: String,
    pub is_admin: bool,
}

async fn validate_api_key(
    State(expected_hash): State<Option<String>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Check Trusted Header (SaaS / Kong)
    // Clone header value to avoid holding immutable borrow
    let user_id_header = request
        .headers()
        .get("x-hyperspace-user-id")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    if let Some(uid) = user_id_header {
        request.extensions_mut().insert(RequestContext {
            user_id: uid,
            is_admin: false,
        });
        return Ok(next.run(request).await);
    }

    // Auth is skipped for static files (except if we want to enforce user context?)
    if !request.uri().path().starts_with("/api/") && request.uri().path() != "/metrics" {
        return Ok(next.run(request).await);
    }

    if let Some(expected) = expected_hash {
        match request.headers().get("x-api-key") {
            Some(key) => {
                if let Ok(key_str) = key.to_str() {
                    let mut hasher = Sha256::new();
                    hasher.update(key_str.as_bytes());
                    let hash = hex::encode(hasher.finalize());

                    if hash == expected {
                        request.extensions_mut().insert(RequestContext {
                            user_id: "default_admin".to_string(),
                            is_admin: true,
                        });
                        return Ok(next.run(request).await);
                    }
                }
                Err(StatusCode::UNAUTHORIZED)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        // No Auth configured (Dev mode)
        request.extensions_mut().insert(RequestContext {
            user_id: "anonymous".to_string(),
            is_admin: true,
        });
        Ok(next.run(request).await)
    }
}

#[derive(Clone, serde::Serialize)]
pub struct EmbeddingInfo {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    pub dimension: usize,
}

pub async fn start_http_server(
    manager: Arc<CollectionManager>,
    port: u16,
    embedding_info: Option<EmbeddingInfo>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get API key hash if set
    let api_key_hash = std::env::var("HYPERSPACE_API_KEY").ok().map(|key| {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    });

    let start_time = Arc::new(Instant::now());
    let embedding_state = Arc::new(embedding_info);

    let app = Router::new()
        .route(
            "/api/collections",
            get(list_collections).post(create_collection),
        )
        .route(
            "/api/collections/{name}",
            get(get_collection_digest).delete(delete_collection),
        )
        .route("/api/collections/{name}/insert", post(insert_vector))
        .route("/api/collections/{name}/stats", get(get_stats))
        .route("/api/collections/{name}/digest", get(get_collection_digest))
        .route("/api/collections/{name}/peek", get(peek_collection))
        .route("/api/collections/{name}/search", post(search_collection))
        .route("/api/collections/{name}/graph/node", get(graph_get_node))
        .route(
            "/api/collections/{name}/graph/neighbors",
            get(graph_get_neighbors),
        )
        .route(
            "/api/collections/{name}/graph/parents",
            get(graph_get_parents),
        )
        .route(
            "/api/collections/{name}/graph/traverse",
            post(graph_traverse),
        )
        .route(
            "/api/collections/{name}/graph/clusters",
            post(graph_clusters),
        )
        .route("/api/status", get(get_status))
        .route("/api/cluster/status", get(get_cluster_status))
        .route("/api/metrics", get(get_metrics))
        .route("/metrics", get(get_prometheus_metrics))
        .route("/api/logs", get(get_logs))
        .route(
            "/api/collections/{name}/rebuild",
            post(rebuild_collection_http),
        )
        .route("/api/admin/vacuum", post(trigger_vacuum_http))
        .route("/api/admin/usage", get(get_usage_report_http))
        .layer(middleware::from_fn_with_state(
            api_key_hash.clone(),
            validate_api_key,
        ))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state((manager, start_time, embedding_state));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("HTTP Dashboard listening on http://{addr}");
    if api_key_hash.is_some() {
        println!("🔒 Dashboard API Key Auth Enabled");
    } else {
        println!("⚠️  Dashboard API Key Auth Disabled");
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    axum::serve(listener, app)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    Ok(())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == "index.html" {
        return index_html().await;
    }

    match FrontendAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data,
            )
                .into_response()
        }
        None => {
            if path.starts_with("api") {
                (StatusCode::NOT_FOUND, "API Route Not Found").into_response()
            } else {
                // SPA fallback
                index_html().await
            }
        }
    }
}

async fn index_html() -> Response {
    match FrontendAssets::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "Dashboard not built. Run `npm run build` in dashboard/",
        )
            .into_response(),
    }
}

// Handlers

#[derive(serde::Serialize)]
struct CollectionSummary {
    name: String,
    count: usize,
    dimension: usize,
    metric: String,
    indexing_queue: u64,
}

async fn get_cluster_status(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
) -> Json<crate::manager::ClusterState> {
    let state = manager.cluster_state.read().await;
    Json(state.clone())
}

async fn list_collections(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> Json<Vec<CollectionSummary>> {
    let names = manager.list(&ctx.user_id);
    let mut summaries = Vec::new();
    for name in names {
        if let Some(col) = manager.get(&ctx.user_id, &name).await {
            summaries.push(CollectionSummary {
                name: name.clone(),
                count: col.count(),
                dimension: col.dimension(),
                metric: col.metric_name().to_string(),
                indexing_queue: col.queue_size(),
            });
        }
    }
    Json(summaries)
}
#[derive(serde::Deserialize)]
struct CreateCollectionRequest {
    name: String,
    dimension: u32,
    metric: String,
}

#[derive(serde::Deserialize)]
struct InsertPayload {
    vector: Vec<f64>,
    id: u32,
    metadata: Option<HashMap<String, String>>,
}

async fn create_collection(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CreateCollectionRequest>,
) -> impl IntoResponse {
    match manager
        .create_collection(
            &ctx.user_id,
            &payload.name,
            payload.dimension,
            &payload.metric,
        )
        .await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn insert_vector(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<InsertPayload>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let clock = manager.cluster_state.read().await.logical_clock;
        let meta = payload.metadata.unwrap_or_default();

        match col
            .insert(
                &payload.vector,
                payload.id,
                meta,
                clock,
                hyperspace_core::Durability::Default,
            )
            .await
        {
            Ok(()) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn delete_collection(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    match manager.delete_collection(&ctx.user_id, &name).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

async fn get_stats(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        Json(serde_json::json!({
            "count": col.count(),
            "dimension": col.dimension(),
            "metric": col.metric_name(),
            "quantization": format!("{:?}", col.quantization_mode()),
            "indexing_queue": col.queue_size(),
        }))
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn get_collection_digest(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let clock = manager.cluster_state.read().await.logical_clock;
        let digest =
            crate::sync::CollectionDigest::new(name.clone(), clock, col.count(), col.buckets());
        Json(digest).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn get_status(
    State((_, start_time, embedding)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
) -> Json<serde_json::Value> {
    let dim = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
    let metric = std::env::var("HS_METRIC").unwrap_or("l2".to_string());
    let quantization = std::env::var("HS_QUANTIZATION_LEVEL").unwrap_or("scalar".to_string());
    let uptime_secs = start_time.elapsed().as_secs();
    let uptime_str = if uptime_secs < 60 {
        format!("{uptime_secs}s")
    } else if uptime_secs < 3600 {
        format!("{}m {}s", uptime_secs / 60, uptime_secs % 60)
    } else {
        format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60)
    };

    Json(serde_json::json!({
        "status": "ONLINE",
        "version": "2.0.0",
        "uptime": uptime_str,
        "config": {
            "dimension": dim,
            "metric": metric,
            "quantization": quantization,
        },
        "embedding": embedding.as_ref()
    }))
}

async fn get_metrics(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if !ctx.is_admin {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let total_vecs = manager.total_vector_count();

    // Calculate disk usage from data directory
    let disk_usage_bytes = calculate_dir_size("./data").unwrap_or(0);
    let disk_usage_mb = (disk_usage_bytes as f64 / 1_048_576.0).round() as u64;

    // Get real system metrics from persistent manager state
    let sys = manager.system.lock();
    let current_pid = Pid::from_u32(std::process::id());

    let (ram_usage_mb, cpu_usage_percent) = if let Some(process) = sys.process(current_pid) {
        let ram = (process.memory() as f64 / 1_048_576.0).round() as u64;
        let cpu = process.cpu_usage().round() as u64;
        (ram, cpu)
    } else {
        (0, 0)
    };

    let (active_count, idle_count) = manager.get_collection_counts();

    Json(serde_json::json!({
        "total_vectors": total_vecs,
        "active_collections": active_count,
        "idle_collections": idle_count,
        "total_collections": active_count + idle_count,
        "ram_usage_mb": ram_usage_mb,
        "cpu_usage_percent": cpu_usage_percent,
        "disk_usage_mb": disk_usage_mb,
    }))
    .into_response()
}

async fn get_prometheus_metrics(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if !ctx.is_admin {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    let (active, idle) = manager.get_collection_counts();
    let total_vecs = manager.total_vector_count();

    // System stats from persistent manager
    let sys = manager.system.lock();
    let pid = Pid::from_u32(std::process::id());
    let (ram_mb, cpu_percent) = if let Some(proc) = sys.process(pid) {
        (
            (proc.memory() as f64 / 1_048_576.0).round() as u64,
            proc.cpu_usage().round() as u64,
        )
    } else {
        (0, 0)
    };

    let disk_mb = calculate_dir_size("./data").unwrap_or(0) / 1_048_576;

    let body = format!(
        "# HELP hyperspace_active_collections Number of collections in memory\n\
         # TYPE hyperspace_active_collections gauge\n\
         hyperspace_active_collections {active}\n\
         # HELP hyperspace_idle_collections Number of collections unloaded to disk\n\
         # TYPE hyperspace_idle_collections gauge\n\
         hyperspace_idle_collections {idle}\n\
         # HELP hyperspace_total_vectors Total number of vectors in active collections\n\
         # TYPE hyperspace_total_vectors gauge\n\
         hyperspace_total_vectors {total_vecs}\n\
         # HELP hyperspace_ram_usage_mb Memory usage in MB\n\
         # TYPE hyperspace_ram_usage_mb gauge\n\
         hyperspace_ram_usage_mb {ram_mb}\n\
         # HELP hyperspace_disk_usage_mb Disk usage in MB\n\
         # TYPE hyperspace_disk_usage_mb gauge\n\
         hyperspace_disk_usage_mb {disk_mb}\n\
         # HELP hyperspace_cpu_usage_percent CPU usage percent\n\
         # TYPE hyperspace_cpu_usage_percent gauge\n\
         hyperspace_cpu_usage_percent {cpu_percent}\n"
    );

    (
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4",
        )],
        body,
    )
        .into_response()
}

fn calculate_dir_size(path: &str) -> std::io::Result<u64> {
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let metadata = entry.metadata()?;
            if metadata.is_file() {
                total_size += metadata.len();
            } else if metadata.is_dir() {
                total_size += calculate_dir_size(&entry.path().to_string_lossy())?;
            }
        }
    }

    Ok(total_size)
}

#[derive(serde::Deserialize)]
struct PeekParams {
    limit: Option<usize>,
}

async fn peek_collection(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(params): Query<PeekParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let items = col.peek(limit);
        Json(items).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct SearchReq {
    vector: Vec<f64>,
    top_k: Option<usize>,
    filter: Option<HashMap<String, String>>,
    filters: Option<Vec<HttpFilter>>,
}

#[derive(serde::Deserialize)]
struct HttpFilter {
    #[serde(rename = "type")]
    filter_type: String,
    key: String,
    value: Option<String>,
    gte: Option<f64>,
    lte: Option<f64>,
}

#[derive(serde::Serialize)]
struct HttpGraphNode {
    id: u32,
    layer: usize,
    neighbors: Vec<u32>,
    metadata: HashMap<String, String>,
    typed_metadata: HashMap<String, serde_json::Value>,
}

fn parse_typed_metadata(
    metadata: &HashMap<String, String>,
) -> (HashMap<String, String>, HashMap<String, serde_json::Value>) {
    let mut plain = HashMap::new();
    let mut typed = HashMap::new();
    for (k, v) in metadata {
        if let Some(raw_key) = k.strip_prefix(TYPED_META_PREFIX) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(v) {
                if let Some(val) = json.get("v") {
                    typed.insert(raw_key.to_string(), val.clone());
                }
            }
            continue;
        }
        plain.insert(k.clone(), v.clone());
    }
    (plain, typed)
}

fn convert_filters(raw: &[HttpFilter]) -> Vec<hyperspace_core::FilterExpr> {
    let mut filters = Vec::new();
    for f in raw {
        match f.filter_type.as_str() {
            "match" => {
                if let Some(value) = &f.value {
                    filters.push(hyperspace_core::FilterExpr::Match {
                        key: f.key.clone(),
                        value: value.clone(),
                    });
                }
            }
            "range" => {
                filters.push(hyperspace_core::FilterExpr::Range {
                    key: f.key.clone(),
                    gte: f.gte,
                    lte: f.lte,
                });
            }
            _ => {}
        }
    }
    filters
}

fn graph_node_from_collection(
    col: &Arc<dyn hyperspace_core::Collection>,
    id: u32,
    layer: usize,
    limit: usize,
    offset: usize,
) -> Result<HttpGraphNode, String> {
    let neighbors = col
        .graph_neighbors(id, layer, limit.saturating_add(offset))?
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let meta = col.metadata_by_id(id);
    let (metadata, typed_metadata) = parse_typed_metadata(&meta);
    Ok(HttpGraphNode {
        id,
        layer,
        neighbors,
        metadata,
        typed_metadata,
    })
}

fn graph_match_filters(
    metadata: &HashMap<String, String>,
    exact_filter: &HashMap<String, String>,
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
    for f in complex_filters {
        match f {
            hyperspace_core::FilterExpr::Match { key, value } => match metadata.get(key) {
                Some(actual) if actual == value => {}
                _ => return false,
            },
            hyperspace_core::FilterExpr::Range { key, gte, lte } => {
                let Some(val) = meta_numeric(key) else {
                    return false;
                };
                if let Some(min) = gte {
                    if val < *min {
                        return false;
                    }
                }
                if let Some(max) = lte {
                    if val > *max {
                        return false;
                    }
                }
            }
        }
    }
    true
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

async fn search_collection(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SearchReq>,
) -> impl IntoResponse {
    let k = payload.top_k.unwrap_or(10);
    let exact_filter = payload.filter.unwrap_or_default();
    let complex_filters = payload
        .filters
        .as_ref()
        .map_or_else(Vec::new, |f| convert_filters(f));
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let dummy_params = SearchParams {
            top_k: k,
            ef_search: default_ef_search(),
            hybrid_query: None,
            hybrid_alpha: None,
        };
        match col
            .search(
                &payload.vector,
                &exact_filter,
                &complex_filters,
                &dummy_params,
            )
            .await
        {
            Ok(res) => {
                let mapped: Vec<serde_json::Value> = res
                    .iter()
                    .map(|(id, dist, meta)| {
                        let (metadata, typed_metadata) = parse_typed_metadata(meta);
                        serde_json::json!({
                            "id": id,
                            "distance": dist,
                            "metadata": metadata,
                            "typed_metadata": typed_metadata
                        })
                    })
                    .collect();
                Json(mapped).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct GraphNodeQuery {
    id: u32,
    layer: Option<usize>,
}

#[derive(serde::Deserialize)]
struct GraphNeighborsQuery {
    id: u32,
    layer: Option<usize>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(serde::Deserialize)]
struct GraphTraverseReq {
    start_id: u32,
    layer: Option<usize>,
    max_depth: Option<usize>,
    max_nodes: Option<usize>,
    filter: Option<HashMap<String, String>>,
    filters: Option<Vec<HttpFilter>>,
}

#[derive(serde::Deserialize)]
struct GraphClustersReq {
    layer: Option<usize>,
    min_cluster_size: Option<usize>,
    max_clusters: Option<usize>,
    max_nodes: Option<usize>,
}

async fn graph_get_node(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<GraphNodeQuery>,
) -> impl IntoResponse {
    let layer = q.layer.unwrap_or(0);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match graph_node_from_collection(&col, q.id, layer, 128, 0) {
            Ok(node) => Json(node).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn graph_get_neighbors(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<GraphNeighborsQuery>,
) -> impl IntoResponse {
    let layer = q.layer.unwrap_or(0);
    let limit = q.limit.unwrap_or(64).min(512);
    let offset = q.offset.unwrap_or(0);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.graph_neighbors(q.id, layer, limit.saturating_add(offset)) {
            Ok(neighbors) => {
                let nodes: Vec<HttpGraphNode> = neighbors
                    .into_iter()
                    .skip(offset)
                    .take(limit)
                    .filter_map(|id| graph_node_from_collection(&col, id, layer, 64, 0).ok())
                    .collect();
                Json(nodes).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn graph_get_parents(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<GraphNeighborsQuery>,
) -> impl IntoResponse {
    let layer = q.layer.unwrap_or(0).saturating_add(1);
    let limit = q.limit.unwrap_or(32).min(256);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let ids = col
            .graph_neighbors(q.id, layer, limit)
            .or_else(|_| col.graph_neighbors(q.id, 0, limit));
        match ids {
            Ok(ids) => {
                let nodes: Vec<HttpGraphNode> = ids
                    .into_iter()
                    .filter_map(|id| graph_node_from_collection(&col, id, layer, 64, 0).ok())
                    .collect();
                Json(nodes).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn graph_traverse(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GraphTraverseReq>,
) -> impl IntoResponse {
    let layer = payload.layer.unwrap_or(0);
    let max_depth = payload.max_depth.unwrap_or(2).min(8);
    let max_nodes = payload.max_nodes.unwrap_or(256).min(10_000);
    let exact_filter = payload.filter.unwrap_or_default();
    let complex_filters = payload
        .filters
        .as_ref()
        .map_or_else(Vec::new, |f| convert_filters(f));
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.graph_traverse(payload.start_id, layer, max_depth, max_nodes) {
            Ok(ids) => {
                let nodes: Vec<HttpGraphNode> = ids
                    .into_iter()
                    .filter(|id| {
                        if exact_filter.is_empty() && complex_filters.is_empty() {
                            return true;
                        }
                        let meta = col.metadata_by_id(*id);
                        graph_match_filters(&meta, &exact_filter, &complex_filters)
                    })
                    .filter_map(|id| graph_node_from_collection(&col, id, layer, 64, 0).ok())
                    .collect();
                Json(nodes).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn graph_clusters(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GraphClustersReq>,
) -> impl IntoResponse {
    let layer = payload.layer.unwrap_or(0);
    let min_cluster_size = payload.min_cluster_size.unwrap_or(3).max(1);
    let max_clusters = payload.max_clusters.unwrap_or(32).min(256);
    let max_nodes = payload.max_nodes.unwrap_or(10_000).min(200_000);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.graph_clusters(layer, min_cluster_size, max_clusters, max_nodes) {
            Ok(clusters) => Json(clusters).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn get_logs() -> Json<Vec<String>> {
    Json(vec![
        "[SYSTEM] Hyperspace DB v1.2.0 Online".into(),
        "[INFO] Control Plane: HTTP :50050".into(),
        "[INFO] Data Plane: gRPC :50051".into(),
    ])
}

async fn rebuild_collection_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    payload: Option<Json<RebuildPayload>>,
) -> impl IntoResponse {
    let filter = payload.and_then(|Json(p)| p.filter_query).and_then(|f| {
        let op = match f.op.to_lowercase().as_str() {
            "lt" => hyperspace_core::VacuumFilterOp::Lt,
            "lte" => hyperspace_core::VacuumFilterOp::Lte,
            "gt" => hyperspace_core::VacuumFilterOp::Gt,
            "gte" => hyperspace_core::VacuumFilterOp::Gte,
            "eq" => hyperspace_core::VacuumFilterOp::Eq,
            "ne" => hyperspace_core::VacuumFilterOp::Ne,
            _ => return None,
        };
        Some(hyperspace_core::VacuumFilterQuery {
            key: f.key,
            op,
            value: f.value,
        })
    });

    match manager
        .rebuild_collection_with_filter(&ctx.user_id, &name, filter)
        .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct RebuildPayload {
    filter_query: Option<RebuildFilterQuery>,
}

#[derive(serde::Deserialize)]
struct RebuildFilterQuery {
    key: String,
    op: String,
    value: f64,
}

async fn trigger_vacuum_http(
    State((_manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if !ctx.is_admin {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }

    // 1. Refresh jemalloc statistics
    if let Err(e) = epoch::advance() {
        eprintln!("Failed to advance jemalloc epoch: {e}");
    }

    // 2. Perform global purge via mallctl
    // In jemalloc 5.x, "arena.4096.purge" purges all arenas.
    // SAFETY: Calling jemalloc purge is safe here as it only triggers memory return to OS.
    if let Err(e) = unsafe { tikv_jemalloc_ctl::raw::update(b"arena.4096.purge\0", ()) } {
        eprintln!("Failed to purge jemalloc arenas: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"status": "Error", "message": format!("Purge failed: {e}")})),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "status": "Success",
        "message": "System memory purged and returned to OS"
    }))
    .into_response()
}

async fn get_usage_report_http(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if !ctx.is_admin {
        return (StatusCode::FORBIDDEN, "Admin access required").into_response();
    }
    let report = manager.get_usage_report();
    Json(report).into_response()
}
