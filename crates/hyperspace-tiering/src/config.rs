//! S3/Cloud tiering configuration parsed from environment variables.

use std::path::PathBuf;

/// All S3 tiering configuration, parsed from env vars.
#[derive(Debug, Clone)]
pub struct TieringConfig {
    /// Storage backend: "local" or "s3"
    pub backend: String,
    /// S3 bucket name
    pub bucket: String,
    /// AWS region (e.g. "us-east-1")
    pub region: String,
    /// S3 endpoint override (for `MinIO` / localstack)
    pub endpoint: Option<String>,
    /// AWS access key (or `MinIO` user)
    pub access_key: Option<String>,
    /// AWS secret key (or `MinIO` password)
    pub secret_key: Option<String>,
    /// Object key prefix inside the bucket (e.g. "v1/chunks")
    pub prefix: String,
    /// Local cache size limit in GB
    pub max_local_cache_gb: u64,
    /// Max retries for transient S3 errors
    pub max_retries: u32,
    /// Parallel upload/download slots
    pub upload_concurrency: usize,
    /// Local data directory (for chunk storage)
    pub data_dir: PathBuf,
}

impl TieringConfig {
    /// Parses configuration from environment variables.
    /// Uses sensible defaults for all optional fields.
    #[must_use]
    pub fn from_env(data_dir: PathBuf) -> Self {
        Self {
            backend: std::env::var("HS_STORAGE_BACKEND")
                .unwrap_or_else(|_| "local".to_string())
                .to_lowercase(),
            bucket: std::env::var("HS_S3_BUCKET")
                .unwrap_or_else(|_| "hyperspace-chunks".to_string()),
            region: std::env::var("HS_S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            endpoint: std::env::var("HS_S3_ENDPOINT").ok(),
            access_key: std::env::var("HS_S3_ACCESS_KEY").ok(),
            secret_key: std::env::var("HS_S3_SECRET_KEY").ok(),
            prefix: std::env::var("HS_S3_PREFIX").unwrap_or_else(|_| "v1/chunks".to_string()),
            max_local_cache_gb: std::env::var("HS_MAX_LOCAL_CACHE_GB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            max_retries: std::env::var("HS_S3_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            upload_concurrency: std::env::var("HS_S3_UPLOAD_CONCURRENCY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4),
            data_dir,
        }
    }

    /// Returns the S3 object key for a given chunk ID.
    #[must_use]
    pub fn object_key(&self, chunk_id: &str) -> String {
        if self.prefix.is_empty() {
            chunk_id.to_string()
        } else {
            format!("{}/{}", self.prefix.trim_end_matches('/'), chunk_id)
        }
    }

    /// Returns the local cache directory for a chunk.
    #[must_use]
    pub fn local_chunk_path(&self, chunk_id: &str) -> PathBuf {
        self.data_dir.join(chunk_id)
    }

    /// Whether S3 backend is requested.
    #[must_use]
    pub fn is_s3(&self) -> bool {
        self.backend == "s3"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_key() {
        let cfg = TieringConfig {
            backend: "s3".to_string(),
            bucket: "test".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
            access_key: None,
            secret_key: None,
            prefix: "v1/chunks".to_string(),
            max_local_cache_gb: 10,
            max_retries: 3,
            upload_concurrency: 4,
            data_dir: PathBuf::from("/data"),
        };
        assert_eq!(cfg.object_key("chunk_abc.hyp"), "v1/chunks/chunk_abc.hyp");
    }

    #[test]
    fn test_object_key_empty_prefix() {
        let cfg = TieringConfig {
            backend: "s3".to_string(),
            bucket: "test".to_string(),
            region: "us-east-1".to_string(),
            endpoint: None,
            access_key: None,
            secret_key: None,
            prefix: String::new(),
            max_local_cache_gb: 10,
            max_retries: 3,
            upload_concurrency: 4,
            data_dir: PathBuf::from("/data"),
        };
        assert_eq!(cfg.object_key("chunk_abc.hyp"), "chunk_abc.hyp");
    }
}
