//! # hyperspace-tiering — Cloud Storage Tiering for HyperspaceDB
//!
//! This crate provides a `ChunkBackend` abstraction for managing immutable HNSW
//! chunk segments across local disk and cloud object storage (S3/MinIO).
//!
//! ## Architecture
//!
//! ```text
//!                    ┌────────────────────────┐
//!                    │   ChunkBackend trait    │
//!                    └────────┬───────────────┘
//!                    ┌────────┴───────────────┐
//!              ┌─────┴─────┐           ┌──────┴──────┐
//!              │LocalBackend│           │  S3Backend  │
//!              │(default)   │           │(s3-tiering) │
//!              │Zero-cost   │           │LRU + Upload │
//!              │passthrough │           │+ Retry      │
//!              └────────────┘           └─────────────┘
//! ```
//!
//! ## Configuration (.env)
//!
//! ```env
//! # Backend selection: "local" (default) or "s3"
//! HS_STORAGE_BACKEND=local
//!
//! # S3-specific settings (only used when HS_STORAGE_BACKEND=s3)
//! HS_S3_BUCKET=hyperspace-chunks
//! HS_S3_REGION=us-east-1
//! HS_S3_ENDPOINT=http://localhost:9000   # For MinIO
//! HS_S3_ACCESS_KEY=minioadmin
//! HS_S3_SECRET_KEY=minioadmin
//! HS_S3_PREFIX=v1/chunks                 # Object key prefix
//! HS_MAX_LOCAL_CACHE_GB=10               # LRU cache limit (GB)
//! HS_S3_MAX_RETRIES=3                    # Max retry attempts
//! HS_S3_UPLOAD_CONCURRENCY=4             # Parallel upload slots
//! ```

pub mod backend;
pub mod config;
pub mod local;
pub mod s3;

pub use backend::{create_backend, ChunkBackend};
pub use config::TieringConfig;
pub use local::LocalBackend;
pub use s3::S3Backend;
