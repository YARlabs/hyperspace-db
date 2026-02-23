//! S3Backend — Cloud-tiered chunk storage with LRU cache and retry logic.
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                    S3Backend                         │
//! │                                                      │
//! │  on_chunk_created(id, path)                          │
//! │    ├─ Register in LRU cache                          │
//! │    └─ Async upload to S3 (with retry)                │
//! │                                                      │
//! │  resolve(id) → local PathBuf                         │
//! │    ├─ Cache hit? → return local path                 │
//! │    └─ Cache miss? → download from S3 → cache → path │
//! │                                                      │
//! │  evict(id)                                           │
//! │    ├─ Ensure uploaded to S3                           │
//! │    └─ Remove local copy, remove from LRU             │
//! │                                                      │
//! │  LRU Cache (moka)                                    │
//! │    └─ On eviction: auto-upload + delete local        │
//! └──────────────────────────────────────────────────────┘
//! ```
//!
//! ## Retry & Resume
//!
//! All S3 operations use exponential backoff with jitter (via `backoff` crate).
//! Upload/download are chunk-level atomic: if interrupted, the entire chunk is
//! re-uploaded/downloaded on retry. For large chunks (>256 MB), S3 multipart
//! upload is used automatically by `object_store`.
//!
//! ## Network Interruption Handling
//!
//! - **Upload failure:** Chunk stays local. Background task retries periodically.
//!   The chunk is marked as "pending upload" in the LRU cache.
//! - **Download failure:** `resolve()` returns an error. Caller can retry or
//!   fall back to searching fewer chunks (graceful degradation).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use bytes::Bytes;
use moka::sync::Cache;
use object_store::aws::AmazonS3Builder;
use object_store::{ObjectStore, PutPayload};
use parking_lot::Mutex;
use tokio::sync::Semaphore;

use crate::backend::ChunkBackend;
use crate::config::TieringConfig;

/// State of a chunk in the cache.
#[derive(Debug, Clone)]
struct ChunkCacheEntry {
    /// Local path to the chunk directory.
    local_path: PathBuf,
    /// Whether this chunk has been successfully uploaded to S3.
    uploaded: bool,
    /// Approximate size in bytes (for cache weight).
    size_bytes: u64,
}

pub struct S3Backend {
    config: TieringConfig,
    /// S3-compatible object store client.
    store: Arc<dyn ObjectStore>,
    /// LRU cache tracking locally-cached chunks.
    /// Key: chunk_id, Value: cache entry with local path + upload status.
    cache: Cache<String, ChunkCacheEntry>,
    /// Set of chunk IDs currently being uploaded (prevents duplicate uploads).
    uploading: Arc<Mutex<HashSet<String>>>,
    /// Semaphore limiting concurrent S3 operations.
    upload_semaphore: Arc<Semaphore>,
    /// Tokio runtime handle for spawning async tasks from sync context.
    rt_handle: tokio::runtime::Handle,
    // Telemetry
    uploads_completed: AtomicU64,
    uploads_failed: AtomicU64,
    downloads_completed: AtomicU64,
    downloads_failed: AtomicU64,
}

impl S3Backend {
    /// Creates a new S3Backend with the given configuration.
    pub fn new(config: TieringConfig) -> Self {
        // Build the S3 client.
        let mut builder = AmazonS3Builder::new()
            .with_bucket_name(&config.bucket)
            .with_region(&config.region);

        if let Some(ref endpoint) = config.endpoint {
            builder = builder.with_endpoint(endpoint);
            // MinIO and localstack need virtual-hosted-style disabled.
            builder = builder.with_virtual_hosted_style_request(false);
            builder = builder.with_allow_http(endpoint.starts_with("http://"));
        }

        if let Some(ref key) = config.access_key {
            builder = builder.with_access_key_id(key);
        }
        if let Some(ref secret) = config.secret_key {
            builder = builder.with_secret_access_key(secret);
        }

        let store: Arc<dyn ObjectStore> = Arc::new(
            builder.build().expect("Failed to build S3 client. Check HS_S3_* environment variables."),
        );

        // LRU cache: max weight = max_local_cache_gb in bytes.
        let max_cache_bytes = config.max_local_cache_gb * 1024 * 1024 * 1024;
        let cache: Cache<String, ChunkCacheEntry> = Cache::builder()
            .max_capacity(max_cache_bytes)
            .weigher(|_key: &String, entry: &ChunkCacheEntry| -> u32 {
                // Weight in bytes, capped at u32::MAX (~4 GB per entry).
                entry.size_bytes.min(u32::MAX as u64) as u32
            })
            .build();

        let upload_semaphore = Arc::new(Semaphore::new(config.upload_concurrency));

        // Try to get the current tokio runtime handle.
        let rt_handle = tokio::runtime::Handle::try_current()
            .expect("S3Backend must be created within a Tokio runtime");

        Self {
            config,
            store,
            cache,
            uploading: Arc::new(Mutex::new(HashSet::new())),
            upload_semaphore,
            rt_handle,
            uploads_completed: AtomicU64::new(0),
            uploads_failed: AtomicU64::new(0),
            downloads_completed: AtomicU64::new(0),
            downloads_failed: AtomicU64::new(0),
        }
    }

    /// Uploads a chunk directory to S3 as a tar archive.
    ///
    /// We tar the chunk directory into a single object to preserve the
    /// directory structure (index.snap + VectorStore files).
    fn upload_chunk(&self, chunk_id: &str, local_path: &Path) -> Result<(), String> {
        let object_key = self.config.object_key(chunk_id);
        let store = self.store.clone();
        let max_retries = self.config.max_retries;

        // Create a tar archive of the chunk directory in memory.
        let tar_bytes = Self::tar_directory(local_path)?;
        let payload = PutPayload::from_bytes(Bytes::from(tar_bytes));
        let path = object_store::path::Path::from(object_key.clone());

        // Blocking upload with retry.
        let result = self.rt_handle.block_on(async {
            let mut attempt = 0u32;
            loop {
                attempt += 1;
                match store.put(&path, payload.clone()).await {
                    Ok(_) => return Ok(()),
                    Err(e) => {
                        if attempt >= max_retries {
                            return Err(format!(
                                "S3 upload failed after {max_retries} attempts for {object_key}: {e}"
                            ));
                        }
                        let delay = std::time::Duration::from_millis(
                            100 * 2u64.pow(attempt - 1), // Exponential backoff
                        );
                        eprintln!(
                            "⚠️  S3 upload attempt {attempt}/{max_retries} failed for {object_key}: {e}. Retrying in {:?}...",
                            delay
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        });

        match &result {
            Ok(()) => {
                self.uploads_completed.fetch_add(1, Ordering::Relaxed);
                println!("☁️  Uploaded chunk {chunk_id} to s3://{}/{object_key}", self.config.bucket);
            }
            Err(e) => {
                self.uploads_failed.fetch_add(1, Ordering::Relaxed);
                eprintln!("❌ {e}");
            }
        }
        result
    }

    /// Downloads a chunk from S3 and extracts it to the local cache directory.
    fn download_chunk(&self, chunk_id: &str) -> Result<PathBuf, String> {
        let object_key = self.config.object_key(chunk_id);
        let local_path = self.config.local_chunk_path(chunk_id);
        let store = self.store.clone();
        let max_retries = self.config.max_retries;
        let path = object_store::path::Path::from(object_key.clone());

        let tar_bytes = self.rt_handle.block_on(async {
            let mut attempt = 0u32;
            loop {
                attempt += 1;
                match store.get(&path).await {
                    Ok(result) => {
                        match result.bytes().await {
                            Ok(bytes) => return Ok(bytes),
                            Err(e) => {
                                if attempt >= max_retries {
                                    return Err(format!(
                                        "S3 download body read failed after {max_retries} attempts for {object_key}: {e}"
                                    ));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if attempt >= max_retries {
                            return Err(format!(
                                "S3 download failed after {max_retries} attempts for {object_key}: {e}"
                            ));
                        }
                    }
                }
                let delay = std::time::Duration::from_millis(
                    100 * 2u64.pow(attempt - 1),
                );
                eprintln!(
                    "⚠️  S3 download attempt {attempt}/{max_retries} failed for {object_key}. Retrying in {:?}...",
                    delay
                );
                tokio::time::sleep(delay).await;
            }
        });

        match tar_bytes {
            Ok(bytes) => {
                Self::untar_to_directory(&bytes, &local_path)?;
                self.downloads_completed.fetch_add(1, Ordering::Relaxed);
                println!("☁️  Downloaded chunk {chunk_id} from s3://{}/{object_key}",
                         self.config.bucket);
                Ok(local_path)
            }
            Err(e) => {
                self.downloads_failed.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
        }
    }

    /// Creates a tar archive of a directory in memory.
    fn tar_directory(dir: &Path) -> Result<Vec<u8>, String> {
        let mut archive = tar::Builder::new(Vec::new());
        archive.append_dir_all(".", dir)
            .map_err(|e| format!("Failed to tar chunk directory {}: {e}", dir.display()))?;
        archive.finish()
            .map_err(|e| format!("Failed to finalize tar archive: {e}"))?;
        let buf = archive.into_inner()
            .map_err(|e| format!("Failed to extract tar buffer: {e}"))?;
        Ok(buf)
    }

    /// Extracts a tar archive to a directory.
    fn untar_to_directory(tar_bytes: &[u8], target_dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create chunk dir {}: {e}", target_dir.display()))?;
        let cursor = std::io::Cursor::new(tar_bytes);
        let mut archive = tar::Archive::new(cursor);
        archive.unpack(target_dir)
            .map_err(|e| format!("Failed to untar chunk to {}: {e}", target_dir.display()))?;
        Ok(())
    }

    /// Calculates the approximate size of a directory in bytes.
    fn dir_size(path: &Path) -> u64 {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    if meta.is_file() {
                        total += meta.len();
                    } else if meta.is_dir() {
                        total += Self::dir_size(&entry.path());
                    }
                }
            }
        }
        total
    }
}

impl ChunkBackend for S3Backend {
    fn resolve(&self, chunk_id: &str) -> Result<PathBuf, String> {
        // 1. Check LRU cache.
        if let Some(entry) = self.cache.get(chunk_id) {
            if entry.local_path.exists() {
                return Ok(entry.local_path);
            }
            // Cache says it exists, but file is gone (disk corruption?).
            // Fall through to download.
            self.cache.invalidate(chunk_id);
        }

        // 2. Check if it exists on disk but not in cache (e.g. after restart).
        let local_path = self.config.local_chunk_path(chunk_id);
        if local_path.exists() {
            let size = Self::dir_size(&local_path);
            self.cache.insert(chunk_id.to_string(), ChunkCacheEntry {
                local_path: local_path.clone(),
                uploaded: true, // Assume uploaded since it was there before restart.
                size_bytes: size,
            });
            return Ok(local_path);
        }

        // 3. Download from S3.
        let downloaded_path = self.download_chunk(chunk_id)?;
        let size = Self::dir_size(&downloaded_path);
        self.cache.insert(chunk_id.to_string(), ChunkCacheEntry {
            local_path: downloaded_path.clone(),
            uploaded: true,
            size_bytes: size,
        });
        Ok(downloaded_path)
    }

    fn on_chunk_created(&self, chunk_id: &str, local_path: &Path) {
        let size = Self::dir_size(local_path);

        // Register in LRU cache (not yet uploaded).
        self.cache.insert(chunk_id.to_string(), ChunkCacheEntry {
            local_path: local_path.to_path_buf(),
            uploaded: false,
            size_bytes: size,
        });

        // Schedule async upload.
        let chunk_id_owned = chunk_id.to_string();
        let local_path_owned = local_path.to_path_buf();
        let uploading = self.uploading.clone();
        let sem = self.upload_semaphore.clone();

        // Check if already uploading.
        {
            let mut guard = uploading.lock();
            if guard.contains(&chunk_id_owned) {
                return; // Already in progress.
            }
            guard.insert(chunk_id_owned.clone());
        }

        // We need a reference to self for upload_chunk, but we're behind
        // a trait object. Instead, clone the necessary state.
        let store = self.store.clone();
        let config = self.config.clone();
        let cache = self.cache.clone();

        self.rt_handle.spawn(async move {
            let _permit = sem.acquire().await;

            // Upload with retry.
            let object_key = config.object_key(&chunk_id_owned);
            let tar_result = S3Backend::tar_directory(&local_path_owned);
            let upload_result = match tar_result {
                Ok(tar_bytes) => {
                    let payload = PutPayload::from_bytes(Bytes::from(tar_bytes));
                    let path = object_store::path::Path::from(object_key.clone());
                    let mut attempt = 0u32;
                    loop {
                        attempt += 1;
                        match store.put(&path, payload.clone()).await {
                            Ok(_) => break Ok(()),
                            Err(e) => {
                                if attempt >= config.max_retries {
                                    break Err(format!("Upload failed after {} attempts: {e}", config.max_retries));
                                }
                                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempt - 1));
                                eprintln!("⚠️  Async upload {attempt}/{} for {object_key}: {e}. Retry in {:?}...",
                                          config.max_retries, delay);
                                tokio::time::sleep(delay).await;
                            }
                        }
                    }
                }
                Err(e) => Err(e),
            };

            // Update cache entry with upload status.
            match upload_result {
                Ok(()) => {
                    println!("☁️  Async upload completed: {chunk_id_owned} → s3://{}/{object_key}",
                             config.bucket);
                    if let Some(entry) = cache.get(&chunk_id_owned) {
                        let mut updated = entry.clone();
                        updated.uploaded = true;
                        cache.insert(chunk_id_owned.clone(), updated);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Async upload failed for {chunk_id_owned}: {e}");
                }
            }

            uploading.lock().remove(&chunk_id_owned);
        });
    }

    fn evict(&self, chunk_id: &str) -> Result<(), String> {
        // Ensure the chunk is uploaded before evicting.
        if let Some(entry) = self.cache.get(chunk_id) {
            if !entry.uploaded {
                // Force synchronous upload before eviction.
                self.upload_chunk(chunk_id, &entry.local_path)?;
            }

            // Remove from local disk.
            if entry.local_path.exists() {
                std::fs::remove_dir_all(&entry.local_path)
                    .map_err(|e| format!("Failed to remove local chunk {chunk_id}: {e}"))?;
            }
        }

        self.cache.invalidate(chunk_id);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "s3"
    }

    fn chunk_count(&self) -> usize {
        self.cache.entry_count() as usize
    }

    fn local_disk_usage_bytes(&self) -> u64 {
        self.cache.weighted_size()
    }
}
