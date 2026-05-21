use crate::gossip::PeerRegistry;
use crate::manager::CollectionManager;
use axum::{
    extract::{Extension, Path, Query, Request, State},
    http::{StatusCode, Uri},
    middleware::{self, Next},
    response::{sse::Event, sse::Sse, Html, IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use base64::prelude::*;
use hyperspace_core::SearchParams;
use hyperspace_proto::hyperspace::EventMessage;
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Instant;
use sysinfo::Pid;
use tikv_jemalloc_ctl::epoch;
use tokio::sync::broadcast;
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
    let mut ctx = RequestContext {
        user_id: "anonymous".to_string(),
        is_admin: false,
    };

    // 1. Extract User Identity (for Multi-tenancy)
    let user_id_header = request
        .headers()
        .get("x-hyperspace-user-id")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    if let Some(uid) = user_id_header {
        ctx.user_id = uid;
    }

    // 2. Validate API Key (for Administrative Access)
    if let Some(expected) = expected_hash {
        if let Some(key) = request.headers().get("x-api-key") {
            if let Ok(key_str) = key.to_str() {
                let mut hasher = Sha256::new();
                hasher.update(key_str.as_bytes());
                let hash = hex::encode(hasher.finalize());

                if hash == expected {
                    ctx.is_admin = true;
                    // If no explicit user ID, admin acts as "default_admin"
                    if ctx.user_id == "anonymous" {
                        ctx.user_id = "default_admin".to_string();
                    }
                }
            }
        }

        // 3. Enforce Auth for API endpoints
        let path = request.uri().path();
        if path.starts_with("/api/") || path == "/metrics" {
            // If neither valid API key nor valid x-hyperspace-user-id was provided
            if !ctx.is_admin && ctx.user_id == "anonymous" {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    } else {
        // No Auth configured in environment (Dev mode)
        ctx.is_admin = true;
        if ctx.user_id == "anonymous" {
            ctx.user_id = "anonymous".to_string();
        }
    }

    // Auth is skipped for static files by default unless handled above
    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}

#[derive(Clone, serde::Serialize)]
pub struct ModelStatus {
    pub enabled: bool,
    pub provider: String,
    pub model: String,
    pub dimension: usize,
}

#[derive(Clone, serde::Serialize)]
pub struct EmbeddingInfo {
    pub enabled: bool,
    pub models: HashMap<String, ModelStatus>,
}

pub async fn start_http_server(
    manager: Arc<CollectionManager>,
    port: u16,
    embedding_info: Option<EmbeddingInfo>,
    peer_registry: Option<PeerRegistry>,
    replication_tx: broadcast::Sender<EventMessage>,
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
        .route(
            "/api/collections/{name}/insert/batch",
            post(batch_insert_http),
        )
        .route("/api/collections/{name}/stats", get(get_stats))
        .route(
            "/api/collections/{name}/config",
            patch(update_collection_config),
        )
        .route(
            "/api/collections/{name}/exists",
            get(collection_exists_http),
        )
        .route(
            "/api/collections/{name}/freeze",
            post(freeze_collection_http),
        )
        .route(
            "/api/collections/{name}/unfreeze",
            post(unfreeze_collection_http),
        )
        .route("/api/collections/{name}/digest", get(get_collection_digest))
        .route("/api/collections/{name}/peek", get(peek_collection))
        .route("/api/collections/{name}/search", post(search_collection))
        .route(
            "/api/collections/{name}/search/batch",
            post(search_batch_http),
        )
        .route("/api/analyze/geometry", post(analyze_raw_geometry))
        .route(
            "/api/collections/{name}/analyze/geometry",
            get(analyze_collection_geometry),
        )
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
        .route(
            "/api/collections/{name}/graph/subsumption",
            get(graph_get_subsumption_tree),
        )
        .route(
            "/api/collections/{name}/graph/explore",
            get(graph_explore_http),
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
        .route(
            "/api/admin/migration/status",
            get(get_migration_service_status),
        )
        .route("/api/admin/migration/start", post(start_migration_service))
        // Delta Sync HTTP API (Task 2.1 — for WASM and REST clients)
        .route(
            "/api/collections/{name}/sync/handshake",
            post(sync_handshake_http),
        )
        .route("/api/collections/{name}/sync/pull", post(sync_pull_http))
        .route("/api/collections/{name}/points", get(get_points_http))
        .route("/api/collections/{name}/cache/stats", get(get_cache_stats_http))
        .route("/api/collections/{name}/cache/clear", post(clear_cache_http))
        .route("/api/collections/{name}/cache/config", post(update_cache_config_http))
        .route(
            "/api/collections/{name}/payload",
            post(update_payload_http).patch(update_payload_http),
        )
        .route("/api/collections/{name}/scroll", post(scroll_http))
        .route("/api/collections/{name}/count", post(count_http))
        .route("/api/search/multi", post(search_multi_http))
        .route("/api/health", get(health_check_http))
        // P2P Swarm API (Task 3.4) — Gossip peer registry
        .route("/api/swarm/peers", get(get_swarm_peers))
        .route("/api/admin/trajectory/stream", get(stream_trajectory_sse))
        .route(
            "/api/admin/trajectory/history",
            get(get_trajectory_history_http),
        )
        .layer(middleware::from_fn_with_state(
            api_key_hash.clone(),
            validate_api_key,
        ))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .layer(axum::Extension(Arc::new(peer_registry)))
        .layer(axum::Extension(replication_tx))
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
    status: String,
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
        if manager.is_active(&ctx.user_id, &name) {
            if let Some(col) = manager.get(&ctx.user_id, &name).await {
                summaries.push(CollectionSummary {
                    name: name.clone(),
                    count: col.count(),
                    dimension: col.dimension(),
                    metric: col.metric_name().to_string(),
                    indexing_queue: col.queue_size(),
                    status: "active".to_string(),
                });
            }
        } else {
            if let Some(meta) = manager.get_metadata_no_wake(&ctx.user_id, &name) {
                summaries.push(CollectionSummary {
                    name: name.clone(),
                    count: 0,
                    dimension: meta.dimension(),
                    metric: meta.metric_name(),
                    indexing_queue: 0,
                    status: "idle".to_string(),
                });
            }
        }
    }
    Json(summaries)
}
#[derive(serde::Deserialize)]
struct CreateCollectionRequest {
    name: String,
    dimension: u32,
    metric: String,
    mrl_cutoff_dimension: Option<u32>,
    mrl_rerank_top_k: Option<u32>,
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
    let mut cascade_pipeline = Vec::new();
    if let Some(cutoff) = payload.mrl_cutoff_dimension {
        cascade_pipeline.push(hyperspace_proto::hyperspace::MrlLayer {
            component_name: "default".to_string(),
            cutoff_dimension: cutoff,
            store_in_ram: true,
            rerank_top_k: payload.mrl_rerank_top_k.unwrap_or(100),
        });
    }

    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: payload.metric.clone(),
            full_dimension: payload.dimension,
            weight: 1.0,
        }],
        cascade_pipeline,
    };

    match manager
        .create_collection(&ctx.user_id, &payload.name, schema)
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

#[derive(serde::Deserialize)]
struct BatchInsertReq {
    vectors: Vec<InsertPayload>,
}

async fn batch_insert_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<BatchInsertReq>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let clock = manager.cluster_state.read().await.logical_clock;
        let vectors: Vec<_> = payload
            .vectors
            .into_iter()
            .map(|p| (p.vector, p.id, p.metadata.unwrap_or_default()))
            .collect();

        match col
            .insert_batch(vectors, clock, hyperspace_core::Durability::Default)
            .await
        {
            Ok(()) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn collection_exists_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if manager.get(&ctx.user_id, &name).await.is_some() {
        StatusCode::OK.into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
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

async fn freeze_collection_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    match manager.freeze_collection(&ctx.user_id, &name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": format!("Collection '{}' frozen.", name)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": e
            })),
        )
            .into_response(),
    }
}

async fn unfreeze_collection_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    match manager.unfreeze_collection(&ctx.user_id, &name).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": format!("Collection '{}' unfrozen.", name)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": e
            })),
        )
            .into_response(),
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
        let usage = col.get_usage();
        let hnsw = col.get_hnsw_config();
        Json(serde_json::json!({
            "count": col.count(),
            "dimension": col.dimension(),
            "metric": col.metric_name(),
            "quantization": format!("{:?}", col.quantization_mode()),
            "indexing_queue": col.queue_size(),
            "write_buffer_size": col.write_buffer_size(),
            "active_tasks": usage.active_indexing_tasks,
            // Real live values from GlobalConfig atomics:
            "ef_search": hnsw.ef_search,
            "ef_construction": hnsw.ef_construction,
            "m": hnsw.m,
            "usage": {
                "disk_bytes": usage.disk_usage_bytes,
                "ram_bytes": usage.ram_usage_bytes,
            }
        }))
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}


async fn update_collection_config(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(update): Json<hyperspace_core::CollectionConfigUpdate>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.update_config(update) {
            Ok(()) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
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
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": uptime_str,
        "config": {
            "dimension": dim,
            "metric": metric,
            "quantization": quantization,
            "mode": std::env::var("HS_MODE").unwrap_or("performance".to_string()),
            "max_ram_gb": std::env::var("HS_MAX_RAM_GB").unwrap_or("0".to_string()),
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
    if ctx.is_admin && ctx.user_id == "default_admin" {
        // --- Full Administrative Metrics ---
        let total_vecs = manager.total_vector_count();
        let disk_usage_bytes = calculate_dir_size("./data").unwrap_or(0);
        let disk_usage_mb = (disk_usage_bytes as f64 / 1_048_576.0).round() as u64;

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

        return Json(serde_json::json!({
            "total_vectors": total_vecs,
            "active_collections": active_count,
            "idle_collections": idle_count,
            "total_collections": active_count + idle_count,
            "ram_usage_mb": ram_usage_mb,
            "cpu_usage_percent": cpu_usage_percent,
            "disk_usage_mb": disk_usage_mb,
            "is_admin": true
        }))
        .into_response();
    }

    // --- Isolated SaaS User Metrics ---
    let usage = manager.get_user_usage(&ctx.user_id);
    let disk_usage_mb = (usage.disk_usage_bytes as f64 / 1_048_576.0).round() as u64;

    Json(serde_json::json!({
        "total_vectors": usage.vector_count,
        "total_collections": usage.collection_count,
        "active_collections": usage.collection_count, // For SaaS we treat all as active/available
        "disk_usage_mb": disk_usage_mb,
        "ram_usage_mb": null, // Hidden for SaaS
        "cpu_usage_percent": null, // Hidden for SaaS
        "is_admin": false
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

    let mut body = format!(
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

    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    if encoder.encode(&metric_families, &mut buffer).is_ok() {
        if let Ok(registry_metrics) = String::from_utf8(buffer) {
            body.push_str(&registry_metrics);
        }
    }

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
    offset: Option<usize>,
    /// Reserved: filter results to entries with logical_clock <= until_clock.
    #[allow(dead_code)]
    until_clock: Option<u64>,
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
    let limit = params.limit.unwrap_or(50).min(250);
    let offset = params.offset.unwrap_or(0);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let items = col.peek(limit, offset);
        Json(items).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn analyze_collection_geometry(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        // Peek up to 500 vectors for analysis
        let samples = col.peek(500, 0);
        let vectors: Vec<Vec<f64>> = samples.into_iter().map(|(_, v, _)| v).collect();

        let (delta, recommendation) =
            hyperspace_core::gromov::analyze_delta_hyperbolicity(&vectors, 1000);

        Json(serde_json::json!({
            "delta": delta,
            "recommendation": recommendation,
            "samples": vectors.len(),
            "metric": col.metric_name()
        }))
        .into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn analyze_raw_geometry(Json(req): Json<AnalyzeRawGeometryReq>) -> impl IntoResponse {
    if req.vectors.is_empty() {
        return (StatusCode::BAD_REQUEST, "No vectors provided").into_response();
    }

    // Default to 1000 samples for analysis
    let (delta, recommendation) =
        hyperspace_core::gromov::analyze_delta_hyperbolicity(&req.vectors, 1000);

    Json(serde_json::json!({
        "delta": delta,
        "recommendation": recommendation,
        "samples": req.vectors.len()
    }))
    .into_response()
}

#[derive(serde::Deserialize)]
pub struct AnalyzeRawGeometryReq {
    pub vectors: Vec<Vec<f64>>,
}

#[derive(serde::Deserialize)]
struct SearchReq {
    vector: Vec<f64>,
    top_k: Option<usize>,
    filter: Option<HashMap<String, String>>,
    filters: Option<Vec<HttpFilter>>,
    use_wasserstein: Option<bool>,
    mrl_dimension: Option<usize>,
    include_payload: Option<bool>,
    weights: Option<HashMap<String, f32>>,
}

#[derive(serde::Deserialize)]
struct HttpFilter {
    #[serde(rename = "type")]
    filter_type: String,
    key: Option<String>,
    value: Option<String>,
    prefix: Option<String>,
    gte: Option<f64>,
    lte: Option<f64>,
    axes: Option<Vec<f64>>,
    apertures: Option<Vec<f64>>,
    cen: Option<f64>,
    min_bounds: Option<Vec<f64>>,
    max_bounds: Option<Vec<f64>>,
    center: Option<Vec<f64>>,
    radius: Option<f64>,
    conditions: Option<Vec<HttpFilter>>,
    condition: Option<Box<HttpFilter>>,
}

#[derive(serde::Serialize)]
struct HttpGraphNode {
    id: u32,
    layer: usize,
    vector: Option<Vec<f64>>,
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
    raw.iter().map(convert_filter).collect()
}

fn convert_filter(f: &HttpFilter) -> hyperspace_core::FilterExpr {
    match f.filter_type.as_str() {
        "match" => hyperspace_core::FilterExpr::Match {
            key: f.key.clone().unwrap_or_default(),
            value: f.value.clone().unwrap_or_default(),
        },
        "prefix" => hyperspace_core::FilterExpr::Prefix {
            key: f.key.clone().unwrap_or_default(),
            prefix: f.prefix.clone().unwrap_or_default(),
        },
        "range" => hyperspace_core::FilterExpr::Range {
            key: f.key.clone().unwrap_or_default(),
            gte: f.gte,
            lte: f.lte,
        },
        "in_cone" => {
            if let (Some(axes), Some(apertures), Some(cen)) = (&f.axes, &f.apertures, f.cen) {
                hyperspace_core::FilterExpr::InCone {
                    axes: axes.clone(),
                    apertures: apertures.clone(),
                    cen,
                }
            } else {
                hyperspace_core::FilterExpr::And(vec![])
            }
        }
        "in_box" => {
            if let (Some(min), Some(max)) = (&f.min_bounds, &f.max_bounds) {
                hyperspace_core::FilterExpr::InBox {
                    min_bounds: min.clone(),
                    max_bounds: max.clone(),
                }
            } else {
                hyperspace_core::FilterExpr::And(vec![])
            }
        }
        "in_ball" => {
            if let (Some(center), Some(radius)) = (&f.center, f.radius) {
                hyperspace_core::FilterExpr::InBall {
                    center: center.clone(),
                    radius,
                }
            } else {
                hyperspace_core::FilterExpr::And(vec![])
            }
        }
        "and" => hyperspace_core::FilterExpr::And(
            f.conditions
                .as_ref()
                .map(|c| convert_filters(c))
                .unwrap_or_default(),
        ),
        "or" => hyperspace_core::FilterExpr::Or(
            f.conditions
                .as_ref()
                .map(|c| convert_filters(c))
                .unwrap_or_default(),
        ),
        "not" => hyperspace_core::FilterExpr::Not(Box::new(
            f.condition
                .as_ref()
                .map_or(hyperspace_core::FilterExpr::And(vec![]), |c| {
                    convert_filter(c)
                }),
        )),
        _ => hyperspace_core::FilterExpr::And(vec![]),
    }
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
    let vector = col.get_points(&[id]).first().map(|(_, v, _)| v.clone());

    Ok(HttpGraphNode {
        id,
        layer,
        vector,
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
            hyperspace_core::FilterExpr::Prefix { key, prefix } => match metadata.get(key) {
                Some(actual) if actual.starts_with(prefix) => {}
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
            hyperspace_core::FilterExpr::InCone { .. }
            | hyperspace_core::FilterExpr::InBox { .. }
            | hyperspace_core::FilterExpr::InBall { .. } => {
                // Geometric filters are skipped in purely metadata-based graph traversal matching
            }
            hyperspace_core::FilterExpr::And(_conds) | hyperspace_core::FilterExpr::Or(_conds) => {
                // Use check_meta for recursive logical filters (metadata-only context).
                if !f.check_meta(metadata) {
                    return false;
                }
            }
            hyperspace_core::FilterExpr::Not(_) => {
                if !f.check_meta(metadata) {
                    return false;
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
            use_wasserstein: payload.use_wasserstein.unwrap_or(false),
            bm25_options: None,
            fusion_method: None,
            mrl_dimension: payload.mrl_dimension,
            include_payload: payload.include_payload.unwrap_or(false),
            component_weights: payload.weights,
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
                    .map(|(id, dist, meta, payload)| {
                        let (metadata, typed_metadata) = parse_typed_metadata(meta);
                        let payload_b64 = payload.as_ref().map(|p| BASE64_STANDARD.encode(p));
                        serde_json::json!({
                            "id": id,
                            "distance": dist,
                            "metadata": metadata,
                            "typed_metadata": typed_metadata,
                            "payload": payload_b64
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
struct GetPointsParams {
    ids: String,
}

async fn get_points_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(params): Query<GetPointsParams>,
) -> impl IntoResponse {
    let ids: Vec<u32> = params
        .ids
        .split(',')
        .filter_map(|s| s.parse().ok())
        .collect();

    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let points = col.get_points(&ids);
        let mapped: Vec<serde_json::Value> = points
            .into_iter()
            .map(|(id, vec, meta)| {
                let (metadata, typed_metadata) = parse_typed_metadata(&meta);
                serde_json::json!({
                    "id": id,
                    "vector": vec,
                    "metadata": metadata,
                    "typed_metadata": typed_metadata
                })
            })
            .collect();
        Json(mapped).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct UpdatePayloadReq {
    id: u32,
    metadata: HashMap<String, String>,
}

async fn update_payload_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdatePayloadReq>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.update_payload(payload.id, payload.metadata) {
            Ok(()) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct ScrollReq {
    limit: Option<usize>,
    offset: Option<usize>,
    filters: Option<Vec<HttpFilter>>,
}

async fn scroll_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<ScrollReq>,
) -> impl IntoResponse {
    let limit = payload.limit.unwrap_or(50).min(1000);
    let offset = payload.offset.unwrap_or(0);
    let filters = payload
        .filters
        .as_ref()
        .map_or_else(Vec::new, |f| convert_filters(f));

    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let points = col.scroll(limit, offset, &filters);
        let mapped: Vec<serde_json::Value> = points
            .into_iter()
            .map(|(id, vec, meta)| {
                let (metadata, typed_metadata) = parse_typed_metadata(&meta);
                serde_json::json!({
                    "id": id,
                    "vector": vec,
                    "metadata": metadata,
                    "typed_metadata": typed_metadata
                })
            })
            .collect();
        Json(mapped).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct CountReq {
    filters: Option<Vec<HttpFilter>>,
}

async fn count_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<CountReq>,
) -> impl IntoResponse {
    let filters = payload
        .filters
        .as_ref()
        .map_or_else(Vec::new, |f| convert_filters(f));

    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let count = col.count_filtered(&filters);
        Json(serde_json::json!({ "count": count })).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct SearchMultiReq {
    collections: Vec<String>,
    vector: Vec<f64>,
    top_k: Option<usize>,
}

async fn search_multi_http(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SearchMultiReq>,
) -> impl IntoResponse {
    let k = payload.top_k.unwrap_or(10);
    let mut responses = HashMap::new();

    for name in payload.collections {
        if let Some(col) = manager.get(&ctx.user_id, &name).await {
            let params = SearchParams {
                top_k: k,
                ef_search: default_ef_search(),
                ..Default::default()
            };
            if let Ok(res) = col
                .search(&payload.vector, &HashMap::new(), &[], &params)
                .await
            {
                let mapped: Vec<serde_json::Value> = res
                    .iter()
                    .map(|(id, dist, meta, payload)| {
                        let (metadata, typed_metadata) = parse_typed_metadata(meta);
                        let payload_b64 = payload.as_ref().map(|p| BASE64_STANDARD.encode(p));
                        serde_json::json!({
                            "id": id,
                            "distance": dist,
                            "metadata": metadata,
                            "typed_metadata": typed_metadata,
                            "payload": payload_b64
                        })
                    })
                    .collect();
                responses.insert(name, mapped);
            }
        }
    }
    Json(responses).into_response()
}

async fn search_batch_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<SearchBatchReq>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        let mut results = Vec::new();
        for req in payload.searches {
            let k = req.top_k.unwrap_or(10);
            let params = SearchParams {
                top_k: k,
                ef_search: default_ef_search(),
                use_wasserstein: req.use_wasserstein.unwrap_or(false),
                mrl_dimension: req.mrl_dimension,
                ..Default::default()
            };
            let complex_filters = req
                .filters
                .as_ref()
                .map_or_else(Vec::new, |f| convert_filters(f));

            if let Ok(res) = col
                .search(
                    &req.vector,
                    &req.filter.unwrap_or_default(),
                    &complex_filters,
                    &params,
                )
                .await
            {
                let mapped: Vec<serde_json::Value> = res
                    .iter()
                    .map(|(id, dist, meta, payload)| {
                        let (metadata, typed_metadata) = parse_typed_metadata(meta);
                        let payload_b64 = payload.as_ref().map(|p| BASE64_STANDARD.encode(p));
                        serde_json::json!({
                            "id": id,
                            "distance": dist,
                            "metadata": metadata,
                            "typed_metadata": typed_metadata,
                            "payload": payload_b64
                        })
                    })
                    .collect();
                results.push(mapped);
            } else {
                results.push(Vec::new());
            }
        }
        Json(results).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct SearchBatchReq {
    searches: Vec<SearchReq>,
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
        match col.graph_traverse(payload.start_id, layer, max_depth, max_nodes, usize::MAX) {
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

#[derive(serde::Deserialize)]
struct GraphSubsumptionQuery {
    root_id: u32,
    max_depth: Option<u32>,
}

async fn graph_get_subsumption_tree(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<GraphSubsumptionQuery>,
) -> impl IntoResponse {
    let max_depth = q.max_depth.unwrap_or(3).min(10);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.graph_traverse(q.root_id, 0, max_depth as usize, 1000, 10) {
            Ok(ids) => {
                let nodes: Vec<HttpGraphNode> = ids
                    .into_iter()
                    .filter_map(|id| graph_node_from_collection(&col, id, 0, 64, 0).ok())
                    .collect();
                Json(nodes).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct GraphExploreQuery {
    start_id: u32,
    max_depth: Option<u32>,
    max_nodes: Option<u32>,
}

async fn graph_explore_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Query(q): Query<GraphExploreQuery>,
) -> impl IntoResponse {
    let max_depth = q.max_depth.unwrap_or(2).min(5);
    let max_nodes = q.max_nodes.unwrap_or(256).min(2000);
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.graph_traverse(q.start_id, 0, max_depth as usize, max_nodes as usize, 10) {
            Ok(ids) => {
                let nodes: Vec<HttpGraphNode> = ids
                    .into_iter()
                    .filter_map(|id| graph_node_from_collection(&col, id, 0, 64, 0).ok())
                    .collect();
                Json(nodes).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn get_logs() -> Json<Vec<String>> {
    Json(vec![
        "[SYSTEM] Hyperspace DB Online".into(),
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

// ─── Delta Sync HTTP Handlers (Task 2.1) ──────────────────────────────────

// The `client_` prefix on all fields mirrors the JSON API schema where all peer
// fields are named client_* to distinguish them from server_* counterparts.
#[allow(clippy::struct_field_names)]
#[derive(serde::Deserialize)]
struct SyncHandshakeHttpRequest {
    /// Raw bucket hashes from the client (256 entries).
    client_buckets: Vec<u64>,
    /// Client's Lamport clock at time of handshake.
    #[serde(default)]
    client_logical_clock: u64,
    /// Total vector count on client side (reserved for future quota checks).
    #[serde(default)]
    #[allow(dead_code)]
    client_count: u64,
}

#[derive(serde::Serialize)]
struct SyncDiffBucket {
    bucket_index: u32,
    server_hash: u64,
    client_hash: u64,
}

#[derive(serde::Serialize)]
struct SyncHandshakeHttpResponse {
    diff_buckets: Vec<SyncDiffBucket>,
    server_logical_clock: u64,
    server_count: u64,
    in_sync: bool,
}

async fn sync_handshake_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(body): Json<SyncHandshakeHttpRequest>,
) -> impl IntoResponse {
    let Some(col) = manager.get(&ctx.user_id, &name).await else {
        return (StatusCode::NOT_FOUND, "Collection not found").into_response();
    };

    let server_buckets = col.buckets();
    let server_clock = manager.cluster_state.read().await.logical_clock;
    let server_count = col.count() as u64;

    if body.client_buckets.len() != server_buckets.len() {
        return (
            StatusCode::BAD_REQUEST,
            format!(
                "Bucket count mismatch: client={}, server={}",
                body.client_buckets.len(),
                server_buckets.len()
            ),
        )
            .into_response();
    }

    let mut diff_buckets = Vec::new();
    for (i, (client_hash, server_hash)) in body
        .client_buckets
        .iter()
        .zip(server_buckets.iter())
        .enumerate()
    {
        if client_hash != server_hash {
            diff_buckets.push(SyncDiffBucket {
                bucket_index: i as u32,
                server_hash: *server_hash,
                client_hash: *client_hash,
            });
        }
    }

    let in_sync = diff_buckets.is_empty();
    if in_sync {
        println!(
            "🔄 HTTP SyncHandshake: '{name}' in sync (client_clock={})",
            body.client_logical_clock
        );
    } else {
        println!(
            "🔄 HTTP SyncHandshake: '{name}' {} dirty buckets",
            diff_buckets.len()
        );
    }

    Json(SyncHandshakeHttpResponse {
        diff_buckets,
        server_logical_clock: server_clock,
        server_count,
        in_sync,
    })
    .into_response()
}

#[derive(serde::Deserialize)]
struct SyncPullHttpRequest {
    bucket_indices: Vec<u32>,
}

#[derive(serde::Serialize)]
struct SyncVectorDataHttp {
    id: u32,
    vector: Vec<f64>,
    metadata: std::collections::HashMap<String, String>,
    bucket_index: u32,
}

async fn sync_pull_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(body): Json<SyncPullHttpRequest>,
) -> impl IntoResponse {
    let Some(col) = manager.get(&ctx.user_id, &name).await else {
        return (StatusCode::NOT_FOUND, "Collection not found").into_response();
    };

    if body.bucket_indices.is_empty() {
        return (StatusCode::BAD_REQUEST, "No bucket indices specified").into_response();
    }

    println!(
        "📥 HTTP SyncPull: '{name}' pulling {} buckets",
        body.bucket_indices.len()
    );

    let vectors = col.peek_buckets(&body.bucket_indices);
    let result: Vec<SyncVectorDataHttp> = vectors
        .into_iter()
        .map(|(id, vector, metadata)| {
            let bucket_index = (id as usize % crate::sync::SYNC_BUCKETS) as u32;
            SyncVectorDataHttp {
                id,
                vector,
                metadata,
                bucket_index,
            }
        })
        .collect();

    println!(
        "📥 HTTP SyncPull: '{name}' returning {} vectors",
        result.len()
    );
    Json(result).into_response()
}

// ─── P2P Swarm API (Task 3.4) ──────────────────────────────────────────────

/// GET /api/swarm/peers
///
/// Returns all currently known gossip peers with their sync status.
/// The list is derived from the live PeerRegistry (gossip engine).
/// Stale peers (not seen for >30s) are automatically excluded.
async fn get_swarm_peers(
    Extension(registry): Extension<Arc<Option<PeerRegistry>>>,
) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct SwarmStatus {
        peers: Vec<crate::gossip::PeerInfo>,
        peer_count: usize,
        gossip_enabled: bool,
        replication_enabled: bool,
    }

    let registry_opt = registry.as_ref();
    let (peers, gossip_enabled) = if let Some(reg) = registry_opt {
        let guard = reg.read().await;
        let peers: Vec<_> = guard.values().filter(|p| !p.is_stale()).cloned().collect();
        (peers, true)
    } else {
        (vec![], false)
    };

    // Need to get replication status from state
    let replication_enabled =
        std::env::var("HS_REPLICATION_ALLOWED").unwrap_or_else(|_| "true".to_string()) == "true";

    let count = peers.len();
    Json(SwarmStatus {
        peers,
        peer_count: count,
        gossip_enabled,
        replication_enabled,
    })
    .into_response()
}

async fn get_migration_service_status() -> impl IntoResponse {
    let is_alive = std::net::TcpStream::connect("127.0.0.1:3001").is_ok();
    Json(serde_json::json!({ "active": is_alive }))
}

async fn start_migration_service() -> impl IntoResponse {
    if std::net::TcpStream::connect("127.0.0.1:3001").is_ok() {
        return (StatusCode::OK, "Service already running").into_response();
    }

    let service_dir = "./dashboard/service";
    let node_modules = std::path::Path::new(service_dir).join("node_modules");

    // 1. If node_modules is missing, try to install
    if !node_modules.exists() {
        println!("📦 node_modules missing in migration service. Running npm install...");
        let install_status = std::process::Command::new("npm")
            .arg("install")
            .current_dir(service_dir)
            .status();

        if let Err(e) = install_status {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to run npm install: {e}"),
            )
                .into_response();
        }
    }

    // 2. Spawn node process
    let status = std::process::Command::new("npm")
        .args(["start"])
        .current_dir(service_dir)
        .spawn();

    match status {
        Ok(_) => (StatusCode::ACCEPTED, "Starting migration service...").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to start: {e}"),
        )
            .into_response(),
    }
}

static TRAJECTORY_HISTORY: OnceLock<tokio::sync::RwLock<Vec<serde_json::Value>>> = OnceLock::new();

fn get_trajectory_history() -> &'static tokio::sync::RwLock<Vec<serde_json::Value>> {
    TRAJECTORY_HISTORY.get_or_init(|| tokio::sync::RwLock::new(Vec::with_capacity(1000)))
}

async fn stream_trajectory_sse(
    Extension(tx): Extension<broadcast::Sender<EventMessage>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = tx.subscribe();

    let stream = async_stream::stream! {
        while let Ok(msg) = rx.recv().await {
            // Save to history as JSON Value
            if let Ok(json) = serde_json::to_value(&msg) {
                let mut history = get_trajectory_history().write().await;
                if history.len() >= 1000 { history.remove(0); }
                history.push(json);
            }

            let json_str = serde_json::to_string(&msg).unwrap_or_default();
            yield Ok(Event::default().data(json_str));
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn get_trajectory_history_http() -> impl IntoResponse {
    let history = get_trajectory_history().read().await;
    Json(history.clone())
}

async fn health_check_http() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ONLINE" }))
}

async fn get_cache_stats_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.cache_stats() {
            Ok(stats_json) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stats_json) {
                    Json(value).into_response()
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse cache stats").into_response()
                }
            }
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn clear_cache_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.cache_clear() {
            Ok(()) => Json(serde_json::json!({ "status": "success" })).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct UpdateCacheConfigReq {
    policy: String,
    ann_threshold: Option<f64>,
}

async fn update_cache_config_http(
    Path(name): Path<String>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<UpdateCacheConfigReq>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.cache_update_config(payload.policy, payload.ann_threshold) {
            Ok(()) => Json(serde_json::json!({ "status": "success" })).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}
