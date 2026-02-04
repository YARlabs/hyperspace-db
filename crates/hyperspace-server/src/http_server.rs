use crate::manager::CollectionManager;
use axum::{
    extract::{Path, State, Request},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, delete},
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

    let app = Router::new()
        .route("/api/collections", get(list_collections).post(create_collection))
        .route("/api/collections/{name}", delete(delete_collection))
        .route("/api/collections/{name}/stats", get(get_stats))
        .route("/api/status", get(get_status))
        .route("/api/metrics", get(get_metrics))
        .layer(middleware::from_fn_with_state(api_key_hash.clone(), validate_api_key))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(manager);

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

async fn list_collections(State(manager): State<Arc<CollectionManager>>) -> Json<Vec<String>> {
    Json(manager.list())
}

#[derive(serde::Deserialize)]
struct CreateReq {
    name: String,
    dimension: u32,
    metric: String,
}

async fn create_collection(
    State(manager): State<Arc<CollectionManager>>,
    Json(payload): Json<CreateReq>,
) -> impl IntoResponse {
    match manager.create_collection(&payload.name, payload.dimension, &payload.metric).await {
        Ok(_) => (StatusCode::CREATED, "Created").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

async fn delete_collection(
    State(manager): State<Arc<CollectionManager>>,
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
    State(manager): State<Arc<CollectionManager>>,
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

async fn get_status() -> Json<serde_json::Value> {
    let dim = std::env::var("HS_DIMENSION").unwrap_or("1024".to_string());
    let metric = std::env::var("HS_METRIC").unwrap_or("l2".to_string());
    
    Json(serde_json::json!({
        "status": "ONLINE",
        "version": "1.2.0",
        "uptime": "Unknown", 
        "config": {
            "dimension": dim,
            "metric": metric,
            "quantization": "ScalarI8"
        }
    }))
}

async fn get_metrics(State(manager): State<Arc<CollectionManager>>) -> Json<serde_json::Value> {
    let cols = manager.list();
    let mut total_vecs = 0;
    for c in &cols {
        if let Some(col) = manager.get(c) {
            total_vecs += col.count();
        }
    }

    Json(serde_json::json!({
        "total_vectors": total_vecs,
        "total_collections": cols.len(),
        "ram_usage_mb": 256,
        "cpu_usage_percent": 5, 
    }))
}
