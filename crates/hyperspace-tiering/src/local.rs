//! LocalBackend — default zero-overhead chunk storage.
//!
//! All chunks live on local disk forever. No LRU, no async I/O,
//! no cloud dependencies. Optimal for edge devices and performance-tuned setups.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::backend::ChunkBackend;

pub struct LocalBackend {
    data_dir: PathBuf,
    chunk_count: AtomicU64,
    disk_usage: AtomicU64,
}

impl LocalBackend {
    pub fn new(data_dir: PathBuf) -> Self {
        // Scan existing chunks on startup.
        let (count, bytes) = Self::scan_chunks(&data_dir);
        if count > 0 {
            println!(
                "📂 LocalBackend: Found {count} existing chunk(s) ({} MB on disk)",
                bytes / (1024 * 1024)
            );
        }
        Self {
            data_dir,
            chunk_count: AtomicU64::new(count),
            disk_usage: AtomicU64::new(bytes),
        }
    }

    fn scan_chunks(data_dir: &Path) -> (u64, u64) {
        let mut count = 0u64;
        let mut bytes = 0u64;
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("chunk_") && name_str.ends_with(".hyp") {
                    count += 1;
                    if let Ok(meta) = entry.metadata() {
                        bytes += meta.len();
                        // Also account for sub-files in chunk directory
                        if meta.is_dir() {
                            if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                                for sub in sub_entries.flatten() {
                                    if let Ok(sub_meta) = sub.metadata() {
                                        bytes += sub_meta.len();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        (count, bytes)
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

    fn on_chunk_created(&self, _chunk_id: &str, local_path: &Path) {
        self.chunk_count.fetch_add(1, Ordering::Relaxed);
        // Estimate disk usage from the chunk directory.
        if let Ok(meta) = std::fs::metadata(local_path) {
            self.disk_usage.fetch_add(meta.len(), Ordering::Relaxed);
        }
    }

    fn evict(&self, _chunk_id: &str) -> Result<(), String> {
        // No-op — local backend never evicts chunks.
        Ok(())
    }

    fn name(&self) -> &'static str {
        "local"
    }

    fn chunk_count(&self) -> usize {
        self.chunk_count.load(Ordering::Relaxed) as usize
    }

    fn local_disk_usage_bytes(&self) -> u64 {
        self.disk_usage.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_resolve_existing() {
        let tmp = std::env::temp_dir().join("hs_tiering_local_test");
        let chunk = tmp.join("chunk_test.hyp");
        let _ = fs::create_dir_all(&chunk);

        let backend = LocalBackend::new(tmp.clone());
        assert!(backend.resolve("chunk_test.hyp").is_ok());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_resolve_missing() {
        let tmp = std::env::temp_dir().join("hs_tiering_local_miss");
        let _ = fs::create_dir_all(&tmp);

        let backend = LocalBackend::new(tmp.clone());
        assert!(backend.resolve("nonexistent.hyp").is_err());

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_evict_noop() {
        let backend = LocalBackend::new(PathBuf::from("/tmp"));
        assert!(backend.evict("any").is_ok());
    }

    #[test]
    fn test_name() {
        let backend = LocalBackend::new(PathBuf::from("/tmp"));
        assert_eq!(backend.name(), "local");
    }
}
