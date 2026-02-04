use crate::manager::CollectionManager;
use hyperspace_core::SearchParams;
use std::collections::HashMap;
use std::time::Instant;
use sysinfo::{System, Pid};
use axum::{
    extract::{Path, State, Request, Query},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, delete, post},
    Json, Router,
    middleware::{self, Next},
};
use rust_embed::RustEmbed;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use sha2::{Digest, Sha256};

#[derive(RustEmbed)]
#[folder = "../../dashboard/dist"]
struct FrontendAssets;

// API Key validation middleware
async fn validate_api_key(
    State(expected_hash): State<Option<String>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for static files
    if !request.uri().path().starts_with("/api/") {
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
                        return Ok(next.run(request).await);
                    }
                }
                Err(StatusCode::UNAUTHORIZED)
            }
            None => Err(StatusCode::UNAUTHORIZED),
        }
    } else {
        Ok(next.run(request).await)
    }
}

pub async fn start_http_server(
    manager: Arc<CollectionManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get API key hash if set
    let api_key_hash = std::env::var("HYPERSPACE_API_KEY").ok().map(|key| {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    });

    let start_time = Arc::new(Instant::now());

    let app = Router::new()
        .route("/api/collections", get(list_collections).post(create_collection))
        .route("/api/collections/{name}", delete(delete_collection))
        .route("/api/collections/{name}/stats", get(get_stats))
        .route("/api/collections/{name}/peek", get(peek_collection))
        .route("/api/collections/{name}/search", post(search_collection))
        .route("/api/status", get(get_status))
        .route("/api/cluster/status", get(get_cluster_status))
        .route("/api/metrics", get(get_metrics))
        .route("/api/logs", get(get_logs))
        .layer(middleware::from_fn_with_state(api_key_hash.clone(), validate_api_key))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state((manager, start_time));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("HTTP Dashboard listening on http://{}", addr);
    if api_key_hash.is_some() {
        println!("🔒 Dashboard API Key Auth Enabled");
    } else {
        println!("⚠️  Dashboard API Key Auth Disabled");
    }

    let listener = tokio::net::TcpListener::bind(addr).await
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
            ([(axum::http::header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
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
        None => (StatusCode::NOT_FOUND, "Dashboard not built. Run `npm run build` in dashboard/").into_response(),
    }
}

// Handlers

#[derive(serde::Serialize)]
struct CollectionSummary {
    name: String,
    count: usize,
    dimension: usize,
    metric: String,
}

async fn get_cluster_status(State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>) -> Json<crate::manager::ClusterState> {
    let state = manager.cluster_state.read().await;
    Json(state.clone())
}

async fn list_collections(State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>) -> Json<Vec<CollectionSummary>> {
    let names = manager.list();
    let mut summaries = Vec::new();
    for name in names {
         if let Some(col) = manager.get(&name) {
             summaries.push(CollectionSummary {
                 name: name.clone(),
                 count: col.count(),
                 dimension: col.dimension(),
                 metric: col.metric_name().to_string()
             });
         }
    }
    Json(summaries)
}

#[derive(serde::Deserialize)]
struct CreateReq {
    name: String,
    dimension: u32,
    metric: String,
}

async fn create_collection(
    State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>,
    Json(payload): Json<CreateReq>,
) -> impl IntoResponse {
    match manager.create_collection(&payload.name, payload.dimension, &payload.metric).await {
        Ok(_) => (StatusCode::CREATED, "Created").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn delete_collection(
    State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match manager.delete_collection(&name) {
        Ok(_) => (StatusCode::OK, "Deleted").into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e).into_response(),
    }
}

#[derive(serde::Serialize)]
struct StatsRes {
    count: usize,
    dimension: u32,
    metric: String,
}

async fn get_stats(
    State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&name) {
        Json(StatsRes {
            count: col.count(),
            dimension: col.dimension() as u32,
            metric: col.metric_name().to_string(),
        }).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

async fn get_status(State((_, start_time)): State<(Arc<CollectionManager>, Arc<Instant>)>) -> Json<serde_json::Value> {
    let dim = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
    let metric = std::env::var("HS_METRIC").unwrap_or("l2".to_string());
    
    let uptime_secs = start_time.elapsed().as_secs();
    let uptime_str = if uptime_secs < 60 {
        format!("{}s", uptime_secs)
    } else if uptime_secs < 3600 {
        format!("{}m {}s", uptime_secs / 60, uptime_secs % 60)
    } else {
        format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60)
    };
    
    Json(serde_json::json!({
        "status": "ONLINE",
        "version": "1.2.0",
        "uptime": uptime_str, 
        "config": {
            "dimension": dim,
            "metric": metric,
            "quantization": "ScalarI8"
        }
    }))
}

async fn get_metrics(State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>) -> Json<serde_json::Value> {
    let cols = manager.list();
    let mut total_vecs = 0;
    for c in &cols {
        if let Some(col) = manager.get(c) {
            total_vecs += col.count();
        }
    }

    // Calculate disk usage from data directory
    let disk_usage_bytes = calculate_dir_size("./data").unwrap_or(0);
    let disk_usage_mb = (disk_usage_bytes as f64 / 1_048_576.0).round() as u64;

    // Get real system metrics
    let sys = System::new_all();
    
    // Get current process memory and CPU usage
    let current_pid = Pid::from_u32(std::process::id());
    
    let (ram_usage_mb, cpu_usage_percent) = if let Some(process) = sys.process(current_pid) {
        let ram = (process.memory() as f64 / 1_048_576.0).round() as u64;
        let cpu = process.cpu_usage().round() as u64;
        (ram, cpu)
    } else {
        (0, 0)
    };

    Json(serde_json::json!({
        "total_vectors": total_vecs,
        "total_collections": cols.len(),
        "ram_usage_mb": ram_usage_mb,
        "cpu_usage_percent": cpu_usage_percent,
        "disk_usage_mb": disk_usage_mb,
    }))
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
    State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>,
    Path(name): Path<String>,
    Query(params): Query<PeekParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(50).min(100);
    if let Some(col) = manager.get(&name) {
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
}

async fn search_collection(
    State((manager, _)): State<(Arc<CollectionManager>, Arc<Instant>)>,
    Path(name): Path<String>,
    Json(payload): Json<SearchReq>,
) -> impl IntoResponse {
    let k = payload.top_k.unwrap_or(10);
    if let Some(col) = manager.get(&name) {
        let dummy_params = SearchParams { top_k: k, ef_search: 100, hybrid_query: None, hybrid_alpha: None };
        match col.search(&payload.vector, &HashMap::new(), &[], &dummy_params) {
             Ok(res) => {
                 let mapped: Vec<serde_json::Value> = res.iter().map(|c| serde_json::json!({
                     "id": c.0,
                     "distance": c.1
                 })).collect();
                 Json(mapped).into_response()
             },
             Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
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
