use crate::manager::CollectionManager;
use axum::{
    extract::{Path, State},
    http::{StatusCode, Uri},
    response::{Html, IntoResponse, Response},
    routing::{get, delete},
    Json, Router,
};
use rust_embed::{Embed, RustEmbed};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(RustEmbed)]
#[folder = "../../dashboard/dist"]
struct FrontendAssets;

pub async fn start_http_server(
    manager: Arc<CollectionManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = Router::new()
        .route("/api/collections", get(list_collections).post(create_collection))
        .route("/api/collections/{name}", delete(delete_collection))
        .route("/api/collections/{name}/stats", get(get_stats))
        .fallback(static_handler)
        .layer(CorsLayer::permissive())
        .with_state(manager);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("HTTP Dashboard listening on http://{}", addr);

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
            dimension: 0, // TODO extend trait
            metric: "unknown".to_string(),
        }).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}
