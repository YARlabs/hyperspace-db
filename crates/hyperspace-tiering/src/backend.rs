//! ChunkBackend trait + factory function.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::TieringConfig;
use crate::local::LocalBackend;
use crate::s3::S3Backend;

/// Abstract interface for chunk storage.
///
/// Implementations are responsible for:
/// 1. Resolving a `chunk_id` to a local filesystem path (may trigger S3 download).
/// 2. Notifying the backend that a new chunk was created (for S3 upload).
/// 3. Evicting cold chunks when storage limits are reached.
pub trait ChunkBackend: Send + Sync {
    /// Returns the local path to a chunk directory.
    ///
    /// For `LocalBackend`, this is always `data_dir / chunk_id`.
    /// For `S3Backend`, this may trigger a download if the chunk is not cached locally.
    fn resolve(&self, chunk_id: &str) -> Result<PathBuf, String>;

    /// Notifies the backend that a new chunk was created locally.
    /// `LocalBackend`: no-op.
    /// `S3Backend`: schedules async upload + registers in LRU cache.
    fn on_chunk_created(&self, chunk_id: &str, local_path: &Path);

    /// Requests eviction of a chunk from local storage.
    /// `LocalBackend`: no-op.
    /// `S3Backend`: ensures chunk is uploaded, then removes local copy.
    fn evict(&self, chunk_id: &str) -> Result<(), String>;

    /// Returns the backend name for logging/diagnostics.
    fn name(&self) -> &'static str;

    /// Total number of chunks managed by this backend.
    fn chunk_count(&self) -> usize;

    /// Approximate local disk usage in bytes.
    fn local_disk_usage_bytes(&self) -> u64;
}

/// Creates the appropriate `ChunkBackend` based on configuration.
///
/// - `HS_STORAGE_BACKEND=local` (default) → `LocalBackend` (zero overhead).
/// - `HS_STORAGE_BACKEND=s3` → `S3Backend` (LRU cache + async S3 I/O).
pub fn create_backend(config: TieringConfig) -> Arc<dyn ChunkBackend> {
    if config.is_s3() {
        println!("☁️  Storage Backend: S3 (bucket: {}, region: {})", config.bucket, config.region);
        if let Some(ref ep) = config.endpoint {
            println!("    Endpoint: {ep}");
        }
        println!("    Local Cache Limit: {} GB", config.max_local_cache_gb);
        println!("    Upload Concurrency: {}", config.upload_concurrency);
        println!("    Max Retries: {}", config.max_retries);
        Arc::new(S3Backend::new(config))
    } else {
        println!("💾 Storage Backend: Local (all chunks on NVMe/SSD)");
        Arc::new(LocalBackend::new(config.data_dir))
    }
}
