//! # ChunkBackend — Storage Backend Bridge (Task 1.3)
//!
//! Re-exports [ChunkBackend] trait from either:
//! - Local-only impl (default) — zero cloud deps.
//! - `hyperspace-tiering` crate (when `s3-tiering` feature is enabled).

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::Arc;

// ─── Without s3-tiering feature ────────────────────────────────────────────

#[cfg(not(feature = "s3-tiering"))]
mod inner {
    use super::*;
    use std::path::Path;

    /// Minimal ChunkBackend trait for local-only mode.
    pub trait ChunkBackend: Send + Sync {
        fn resolve(&self, chunk_id: &str) -> Result<PathBuf, String>;
        fn on_chunk_created(&self, chunk_id: &str, local_path: &Path);
        fn evict(&self, chunk_id: &str) -> Result<(), String>;
        fn name(&self) -> &'static str;
    }

    pub struct LocalBackend {
        data_dir: PathBuf,
    }

    impl LocalBackend {
        pub fn new(data_dir: PathBuf) -> Self {
            Self { data_dir }
        }
    }

    impl ChunkBackend for LocalBackend {
        fn resolve(&self, chunk_id: &str) -> Result<PathBuf, String> {
            let path = self.data_dir.join(chunk_id);
            if path.exists() {
                Ok(path)
            } else {
                Err(format!("Chunk not found locally: {chunk_id}"))
            }
        }

        fn on_chunk_created(&self, _chunk_id: &str, _local_path: &Path) {}
        fn evict(&self, _chunk_id: &str) -> Result<(), String> { Ok(()) }
        fn name(&self) -> &'static str { "local" }
    }

    pub fn create_backend(data_dir: PathBuf) -> Arc<dyn ChunkBackend> {
        let backend_str = std::env::var("HS_STORAGE_BACKEND")
            .unwrap_or_else(|_| "local".to_string())
            .to_lowercase();
        if backend_str == "s3" {
            eprintln!("⚠️  HS_STORAGE_BACKEND=s3 requested, but `s3-tiering` feature is not compiled.");
            eprintln!("    Rebuild with: cargo build --features s3-tiering");
            eprintln!("    Falling back to LocalBackend.");
        }
        println!("💾 Storage Backend: Local (all chunks on NVMe/SSD)");
        Arc::new(LocalBackend::new(data_dir))
    }
}

// ─── With s3-tiering feature ───────────────────────────────────────────────

#[cfg(feature = "s3-tiering")]
mod inner {
    use super::*;

    // Re-export everything from the tiering crate.
    pub use hyperspace_tiering::{ChunkBackend, create_backend as tiering_create_backend};
    pub use hyperspace_tiering::config::TieringConfig;

    pub fn create_backend(data_dir: PathBuf) -> Arc<dyn ChunkBackend> {
        let config = TieringConfig::from_env(data_dir);
        tiering_create_backend(config)
    }
}

// ─── Public API ────────────────────────────────────────────────────────────

#[allow(unused_imports)]
pub use inner::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_backend_is_local_by_default() {
        let backend = create_backend(std::env::temp_dir());
        assert_eq!(backend.name(), "local");
    }
}
