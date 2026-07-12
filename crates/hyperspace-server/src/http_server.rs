#![allow(clippy::type_complexity)]

use crate::gossip::PeerRegistry;
use crate::manager::CollectionManager;
use axum::{
    extract::{Extension, Path, Query, Request, State},
    http::{StatusCode, Uri},
    middleware::{self, Next},
    response::{sse::Event, sse::Sse, Html, IntoResponse, Response},
    routing::{delete, get, patch, post},
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
    pub client_ip: String,
    pub role: crate::security::UserRole,
}

async fn validate_api_key(
    State(expected_hash): State<Option<String>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let client_ip = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|axum::extract::ConnectInfo(addr)| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut ctx = RequestContext {
        user_id: "anonymous".to_string(),
        is_admin: false,
        client_ip,
        role: crate::security::UserRole::ReadOnly,
    };

    let user_id_header = request
        .headers()
        .get("x-hyperspace-user-id")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string);

    if let Some(uid) = user_id_header {
        ctx.user_id = uid;
    }

    let mut provided_key = None;
    if let Some(key) = request.headers().get("x-api-key") {
        if let Ok(key_str) = key.to_str() {
            provided_key = Some(key_str.to_string());
        }
    } else if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.to_lowercase().starts_with("bearer ") {
                provided_key = Some(auth_str[7..].to_string());
            }
        }
    }

    let mut key_role = None;
    if let Some(key_str) = provided_key {
        let mut hasher = Sha256::new();
        hasher.update(key_str.as_bytes());
        let hash = hex::encode(hasher.finalize());

        if let Some((uid, role)) = crate::security::validate_key(&hash) {
            ctx.user_id = uid;
            ctx.is_admin = role == crate::security::UserRole::Admin;
            ctx.role = role;
            key_role = Some(role);
        } else {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    if key_role.is_none() {
        if expected_hash.is_none() {
            ctx.is_admin = true;
            if ctx.user_id == "anonymous" {
                ctx.user_id = "anonymous".to_string();
            }
            ctx.role = crate::security::UserRole::Admin;
            key_role = Some(crate::security::UserRole::Admin);
        } else {
            let path = request.uri().path();
            if (path.starts_with("/api/") || path == "/metrics")
                && !ctx.is_admin
                && ctx.user_id == "anonymous"
            {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    let path = request.uri().path();
    if path.starts_with("/api/") {
        let is_write = request.method() == axum::http::Method::POST
            || request.method() == axum::http::Method::DELETE
            || request.method() == axum::http::Method::PATCH;

        if let Some(role) = key_role {
            if is_write && role == crate::security::UserRole::ReadOnly {
                return Err(StatusCode::FORBIDDEN);
            }
        }
    }

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

fn load_certs(path: &str) -> std::io::Result<Vec<rustls_pki_types::CertificateDer<'static>>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(rustls_pki_types::CertificateDer::from)
        .collect();
    Ok(certs)
}

fn load_key(path: &str) -> std::io::Result<rustls_pki_types::PrivateKeyDer<'static>> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    // Try pkcs8 first
    if let Some(key) = rustls_pemfile::pkcs8_private_keys(&mut reader)?
        .into_iter()
        .next()
    {
        return Ok(rustls_pki_types::PrivateKeyDer::Pkcs8(key.into()));
    }
    // Try rsa
    reader = std::io::BufReader::new(std::fs::File::open(path)?);
    if let Some(key) = rustls_pemfile::rsa_private_keys(&mut reader)?
        .into_iter()
        .next()
    {
        return Ok(rustls_pki_types::PrivateKeyDer::Pkcs1(key.into()));
    }
    // Try sec1
    reader = std::io::BufReader::new(std::fs::File::open(path)?);
    if let Some(key) = rustls_pemfile::ec_private_keys(&mut reader)?
        .into_iter()
        .next()
    {
        return Ok(rustls_pki_types::PrivateKeyDer::Sec1(key.into()));
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No private key found",
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn start_http_server(
    manager: Arc<CollectionManager>,
    port: u16,
    embedding_info: Option<EmbeddingInfo>,
    peer_registry: Option<PeerRegistry>,
    replication_tx: broadcast::Sender<EventMessage>,
    tls_cert: Option<String>,
    tls_key: Option<String>,
    tls_ca: Option<String>,
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
        .route(
            "/api/collections/{name}/grant",
            post(grant_collection_access),
        )
        .route(
            "/api/collections/{name}/revoke",
            post(revoke_collection_access),
        )
        .route(
            "/api/collections/{name}/grants",
            get(list_collection_grants),
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
        .route("/api/eco/metrics", get(get_eco_metrics))
        .route("/api/eco/esg-report", get(get_esg_report))
        .route("/metrics", get(get_prometheus_metrics))
        .route("/api/logs", get(get_logs))
        .route(
            "/api/collections/{name}/rebuild",
            post(rebuild_collection_http),
        )
        .route("/api/admin/vacuum", post(trigger_vacuum_http))
        .route("/api/admin/snapshot", post(trigger_snapshot_http))
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
        .route(
            "/api/collections/{name}/points/{id}",
            delete(delete_point_http),
        )
        .route(
            "/api/collections/{name}/cache/stats",
            get(get_cache_stats_http),
        )
        .route(
            "/api/collections/{name}/cache/clear",
            post(clear_cache_http),
        )
        .route(
            "/api/collections/{name}/cache/config",
            post(update_cache_config_http),
        )
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

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    if let (Some(cert_path), Some(key_path)) = (tls_cert, tls_key) {
        println!("🔒 HTTP Dashboard listening on https://{addr}");
        if api_key_hash.is_some() {
            println!("🔒 Dashboard API Key Auth Enabled");
        } else {
            println!("⚠️  Dashboard API Key Auth Disabled");
        }

        let certs = load_certs(&cert_path)?;
        let key = load_key(&key_path)?;

        let mut server_config = tokio_rustls::rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        if let Some(ca_path) = tls_ca {
            println!("🔒 mTLS Client Certificate Verification Enabled");
            let ca_certs = load_certs(&ca_path)?;
            let mut roots = tokio_rustls::rustls::RootCertStore::empty();
            for ca in ca_certs {
                roots
                    .add(ca)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            }
            let client_verifier =
                tokio_rustls::rustls::server::WebPkiClientVerifier::builder(roots.into())
                    .build()
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            server_config = tokio_rustls::rustls::ServerConfig::builder()
                .with_client_cert_verifier(client_verifier)
                .with_single_cert(load_certs(&cert_path)?, load_key(&key_path)?)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        }

        let acceptor = tokio_rustls::TlsAcceptor::from(std::sync::Arc::new(server_config));
        let make_service = app.into_make_service_with_connect_info::<std::net::SocketAddr>();

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to accept connection: {e}");
                    continue;
                }
            };

            let acceptor = acceptor.clone();
            use tower_service::Service;
            let mut make_service_clone = make_service.clone();

            tokio::spawn(async move {
                let tower_service = match make_service_clone.call(peer_addr).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to create service: {e}");
                        return;
                    }
                };

                let tls_stream = match acceptor.accept(stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("TLS handshake failed: {e}");
                        return;
                    }
                };

                let hyper_service = hyper_util::service::TowerToHyperService::new(tower_service);
                let _ = hyper_util::server::conn::auto::Builder::new(
                    hyper_util::rt::TokioExecutor::new(),
                )
                .serve_connection(hyper_util::rt::TokioIo::new(tls_stream), hyper_service)
                .await;
            });
        }
    } else {
        println!("HTTP Dashboard listening on http://{addr}");
        if api_key_hash.is_some() {
            println!("🔒 Dashboard API Key Auth Enabled");
        } else {
            println!("⚠️  Dashboard API Key Auth Disabled");
        }

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
    }

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
    #[serde(skip_serializing_if = "Option::is_none")]
    eco_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_eco_certified: Option<bool>,
    privilege: String,
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
                #[cfg(feature = "eco-monitor")]
                let (eco_tier, is_eco_certified) = {
                    let full_dim =
                        if let Some(meta) = manager.get_metadata_no_wake(&ctx.user_id, &name) {
                            meta.dimension()
                        } else {
                            col.dimension()
                        };
                    let eco_schema = hyperspace_eco::CollectionEcoSchema {
                        collection_name: name.clone(),
                        vector_count: col.count() as u64,
                        full_dimension: full_dim as u32,
                        mrl_dimension: col.dimension() as u32,
                        quantization: col.quantization_mode(),
                    };
                    let tier = eco_schema.calculate_eco_tier();
                    (Some(format!("{:?}", tier)), Some(tier.is_eco_certified()))
                };
                #[cfg(not(feature = "eco-monitor"))]
                let (eco_tier, is_eco_certified) = (None, None);

                summaries.push(CollectionSummary {
                    name: name.clone(),
                    count: col.count(),
                    dimension: col.dimension(),
                    metric: col.metric_name().to_string(),
                    indexing_queue: col.queue_size(),
                    status: "active".to_string(),
                    eco_tier,
                    is_eco_certified,
                    privilege: "Admin".to_string(),
                });
            }
        } else if let Some(meta) = manager.get_metadata_no_wake(&ctx.user_id, &name) {
            #[cfg(feature = "eco-monitor")]
            let (eco_tier, is_eco_certified) = {
                let eco_schema = hyperspace_eco::CollectionEcoSchema {
                    collection_name: name.clone(),
                    vector_count: 0,
                    full_dimension: meta.dimension() as u32,
                    mrl_dimension: meta.dimension() as u32,
                    quantization: meta.quantization_mode(),
                };
                let tier = eco_schema.calculate_eco_tier();
                (Some(format!("{:?}", tier)), Some(tier.is_eco_certified()))
            };
            #[cfg(not(feature = "eco-monitor"))]
            let (eco_tier, is_eco_certified) = (None, None);

            summaries.push(CollectionSummary {
                name: name.clone(),
                count: 0,
                dimension: meta.dimension(),
                metric: meta.metric_name(),
                indexing_queue: 0,
                status: "idle".to_string(),
                eco_tier,
                is_eco_certified,
                privilege: "Admin".to_string(),
            });
        }
    }

    // Append shared collections
    let shared = crate::security::list_shared_collections(&ctx.user_id);
    for (owner, name, role) in shared {
        if manager.is_active(&owner, &name) {
            if let Some(col) = manager.get(&owner, &name).await {
                #[cfg(feature = "eco-monitor")]
                let (eco_tier, is_eco_certified) = {
                    let full_dim = if let Some(meta) = manager.get_metadata_no_wake(&owner, &name) {
                        meta.dimension()
                    } else {
                        col.dimension()
                    };
                    let eco_schema = hyperspace_eco::CollectionEcoSchema {
                        collection_name: name.clone(),
                        vector_count: col.count() as u64,
                        full_dimension: full_dim as u32,
                        mrl_dimension: col.dimension() as u32,
                        quantization: col.quantization_mode(),
                    };
                    let tier = eco_schema.calculate_eco_tier();
                    (Some(format!("{:?}", tier)), Some(tier.is_eco_certified()))
                };
                #[cfg(not(feature = "eco-monitor"))]
                let (eco_tier, is_eco_certified) = (None, None);

                summaries.push(CollectionSummary {
                    name: format!("{owner}:{name}"),
                    count: col.count(),
                    dimension: col.dimension(),
                    metric: col.metric_name().to_string(),
                    indexing_queue: col.queue_size(),
                    status: "active".to_string(),
                    eco_tier,
                    is_eco_certified,
                    privilege: role.as_str().to_string(),
                });
            }
        } else if let Some(meta) = manager.get_metadata_no_wake(&owner, &name) {
            #[cfg(feature = "eco-monitor")]
            let (eco_tier, is_eco_certified) = {
                let eco_schema = hyperspace_eco::CollectionEcoSchema {
                    collection_name: name.clone(),
                    vector_count: 0,
                    full_dimension: meta.dimension() as u32,
                    mrl_dimension: meta.dimension() as u32,
                    quantization: meta.quantization_mode(),
                };
                let tier = eco_schema.calculate_eco_tier();
                (Some(format!("{:?}", tier)), Some(tier.is_eco_certified()))
            };
            #[cfg(not(feature = "eco-monitor"))]
            let (eco_tier, is_eco_certified) = (None, None);

            summaries.push(CollectionSummary {
                name: format!("{owner}:{name}"),
                count: 0,
                dimension: meta.dimension(),
                metric: meta.metric_name(),
                indexing_queue: 0,
                status: "idle".to_string(),
                eco_tier,
                is_eco_certified,
                privilege: role.as_str().to_string(),
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

    let result = manager
        .create_collection(&ctx.user_id, &payload.name, schema)
        .await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.as_str()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Low,
        &ctx.user_id,
        "CREATE_COLLECTION",
        Some(&payload.name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
    }
}

fn resolve_http_collection(
    ctx: &RequestContext,
    req_name: &str,
    required: crate::security::UserRole,
) -> Result<(String, String), StatusCode> {
    let (owner, col_name) = if let Some((o, c)) = req_name.split_once(':') {
        (o.to_string(), c.to_string())
    } else if let Some((o, c)) = req_name.split_once('/') {
        (o.to_string(), c.to_string())
    } else {
        (ctx.user_id.clone(), req_name.to_string())
    };

    if !crate::security::check_access(&ctx.user_id, &owner, &col_name, required) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok((owner, col_name))
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    let result = async {
        let col = manager
            .get(&owner, &col_name)
            .await
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Collection not found".to_string()))?;

        let clock = manager.cluster_state.read().await.logical_clock;
        let meta = payload.metadata.unwrap_or_default();

        col.insert(
            &payload.vector,
            payload.id,
            meta,
            clock,
            hyperspace_core::Durability::Default,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        Ok(StatusCode::OK)
    }
    .await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err((_, e)) => Err(e.as_ref()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Medium,
        &ctx.user_id,
        "INSERT",
        Some(&col_name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
        Ok(code) => code.into_response(),
        Err((code, err)) => (code, err).into_response(),
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    let result = async {
        let col = manager
            .get(&owner, &col_name)
            .await
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Collection not found".to_string()))?;

        let clock = manager.cluster_state.read().await.logical_clock;
        let vectors: Vec<_> = payload
            .vectors
            .into_iter()
            .map(|p| (p.vector, p.id, p.metadata.unwrap_or_default()))
            .collect();

        col.insert_batch(vectors, clock, hyperspace_core::Durability::Default)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

        Ok(StatusCode::OK)
    }
    .await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err((_, e)) => Err(e.as_ref()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Medium,
        &ctx.user_id,
        "BATCH_INSERT",
        Some(&col_name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
        Ok(code) => code.into_response(),
        Err((code, err)) => (code, err).into_response(),
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if manager.get(&owner, &col_name).await.is_some() {
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::Admin) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    let result = manager.delete_collection(&owner, &col_name).await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.as_str()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Low,
        &ctx.user_id,
        "DELETE_COLLECTION",
        Some(&col_name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::Admin) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    let result = manager.freeze_collection(&owner, &col_name).await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.as_str()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Low,
        &ctx.user_id,
        "FREEZE_COLLECTION",
        Some(&col_name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": format!("Collection '{}' frozen.", col_name)
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::Admin) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    let result = manager.unfreeze_collection(&owner, &col_name).await;

    let audit_status = match &result {
        Ok(_) => Ok(()),
        Err(e) => Err(e.as_str()),
    };
    crate::audit_log::log_action(
        crate::audit_log::AuditLogLevel::Low,
        &ctx.user_id,
        "UNFREEZE_COLLECTION",
        Some(&col_name),
        audit_status,
        &ctx.client_ip,
    );

    match result {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": format!("Collection '{}' unfrozen.", col_name)
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
        let clock = manager.cluster_state.read().await.logical_clock;
        let digest =
            crate::sync::CollectionDigest::new(col_name, clock, col.count(), col.buckets());
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

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct EcoParams {
    range: Option<String>,
}

async fn get_eco_metrics(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Query(params): Query<EcoParams>,
) -> impl IntoResponse {
    #[cfg(feature = "eco-monitor")]
    {
        let range = params.range.as_deref().unwrap_or("all");
        let db_path = manager.base_path().join("eco_telemetry.bin");
        match hyperspace_eco::get_carbon_metrics(db_path, range).await {
            Ok(metrics) => Json(serde_json::json!({
                "status": "success",
                "metrics": metrics
            }))
            .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "status": "error", "message": e })),
            )
                .into_response(),
        }
    }
    #[cfg(not(feature = "eco-monitor"))]
    {
        let _ = manager;
        let _ = params;
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "error",
                "message": "EcoMonitor feature is not enabled in this build"
            })),
        )
            .into_response()
    }
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct EsgParams {
    range: Option<String>,
    format: Option<String>,
}

async fn get_esg_report(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Query(params): Query<EsgParams>,
) -> impl IntoResponse {
    #[cfg(feature = "eco-monitor")]
    {
        let range = params.range.as_deref().unwrap_or("all");
        let format = params.format.as_deref().unwrap_or("json");
        let db_path = manager.base_path().join("eco_telemetry.bin");
        match hyperspace_eco::generate_esg_report(db_path, range, format).await {
            Ok(report_data) => {
                let content_type = if format == "csv" {
                    "text/csv"
                } else {
                    "application/json"
                };
                (
                    [(axum::http::header::CONTENT_TYPE, content_type)],
                    report_data,
                )
                    .into_response()
            }
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "status": "error", "message": e })),
            )
                .into_response(),
        }
    }
    #[cfg(not(feature = "eco-monitor"))]
    {
        let _ = manager;
        let _ = params;
        (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "status": "error",
                "message": "EcoMonitor feature is not enabled in this build"
            })),
        )
            .into_response()
    }
}

async fn get_prometheus_metrics(
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    use prometheus::Encoder;
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

    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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

    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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
            use_wave: false,
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

async fn delete_point_http(
    Path((name, id)): Path<(String, u32)>,
    State((manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    if let Some(col) = manager.get(&ctx.user_id, &name).await {
        match col.delete(id) {
            Ok(()) => StatusCode::OK.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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

async fn get_logs(Extension(ctx): Extension<RequestContext>) -> Json<Vec<String>> {
    let logs = crate::audit_log::get_tenant_logs(&ctx.user_id, ctx.is_admin);
    Json(logs)
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

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
        .rebuild_collection_with_filter(&owner, &col_name, filter)
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

async fn trigger_snapshot_http(
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

    let active_collections = manager.list_active_collections();
    let mut successes = std::collections::HashMap::new();
    let mut failures = std::collections::HashMap::new();

    for (name, collection) in active_collections {
        match collection.create_snapshot() {
            Ok(()) => {
                successes.insert(name, "Ok".to_string());
            }
            Err(e) => {
                failures.insert(name, e);
            }
        }
    }

    if !failures.is_empty() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "status": "Error",
                "message": "Failed to create snapshots for some collections.",
                "errors": failures,
                "successes": successes
            })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "status": "Success",
        "message": "System snapshots created successfully.",
        "collections": successes
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadOnly) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
        match col.cache_stats() {
            Ok(stats_json) => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&stats_json) {
                    Json(value).into_response()
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to parse cache stats",
                    )
                        .into_response()
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
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
    let (owner, col_name) =
        match resolve_http_collection(&ctx, &name, crate::security::UserRole::ReadWrite) {
            Ok(res) => res,
            Err(status) => return status.into_response(),
        };

    if let Some(col) = manager.get(&owner, &col_name).await {
        match col.cache_update_config(payload.policy, payload.ann_threshold) {
            Ok(()) => Json(serde_json::json!({ "status": "success" })).into_response(),
            Err(e) => (StatusCode::BAD_REQUEST, e).into_response(),
        }
    } else {
        (StatusCode::NOT_FOUND, "Collection not found").into_response()
    }
}

#[derive(serde::Deserialize)]
struct GrantAccessPayload {
    grantee_id: String,
    privilege: String,
}

async fn grant_collection_access(
    Path(name): Path<String>,
    State((_manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<GrantAccessPayload>,
) -> impl IntoResponse {
    let (owner, col_name) = if let Some((o, c)) = name.split_once('/') {
        (o, c)
    } else {
        (ctx.user_id.as_str(), name.as_str())
    };

    if owner != ctx.user_id && ctx.user_id != "default_admin" {
        return (
            StatusCode::FORBIDDEN,
            "Only collection owner can grant access",
        )
            .into_response();
    }

    let role = crate::security::UserRole::from_str(&payload.privilege);
    match crate::security::grant_access(owner, col_name, &payload.grantee_id, role) {
        Ok(_) => (StatusCode::OK, "Access granted successfully").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

#[derive(serde::Deserialize)]
struct RevokeAccessPayload {
    grantee_id: String,
}

async fn revoke_collection_access(
    Path(name): Path<String>,
    State((_manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
    Json(payload): Json<RevokeAccessPayload>,
) -> impl IntoResponse {
    let (owner, col_name) = if let Some((o, c)) = name.split_once('/') {
        (o, c)
    } else {
        (ctx.user_id.as_str(), name.as_str())
    };

    if owner != ctx.user_id && ctx.user_id != "default_admin" {
        return (
            StatusCode::FORBIDDEN,
            "Only collection owner can revoke access",
        )
            .into_response();
    }

    match crate::security::revoke_access(owner, col_name, &payload.grantee_id) {
        Ok(_) => (StatusCode::OK, "Access revoked successfully").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

async fn list_collection_grants(
    Path(name): Path<String>,
    State((_manager, _, _)): State<(
        Arc<CollectionManager>,
        Arc<Instant>,
        Arc<Option<EmbeddingInfo>>,
    )>,
    Extension(ctx): Extension<RequestContext>,
) -> impl IntoResponse {
    let (owner, col_name) = if let Some((o, c)) = name.split_once('/') {
        (o, c)
    } else {
        (ctx.user_id.as_str(), name.as_str())
    };

    if owner != ctx.user_id && ctx.user_id != "default_admin" {
        return (StatusCode::FORBIDDEN, "Only owner can list grants").into_response();
    }

    let grants = crate::security::list_grants(owner, col_name);
    let list: Vec<serde_json::Value> = grants
        .into_iter()
        .map(|(grantee, role)| {
            serde_json::json!({
                "grantee_id": grantee,
                "privilege": role.as_str()
            })
        })
        .collect();

    Json(list).into_response()
}
