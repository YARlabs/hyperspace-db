//! # ChunkSearcher — Cached Immutable Chunk Loader (Task 1.2 Part B)
//!
//! Loads frozen HNSW chunk segments from disk via `mmap`, runs search inside
//! them, and returns candidate results to be merged with the MemTable.
//!
//! ## Design
//! - Each chunk is a self-contained HNSW index persisted as `index.snap` inside
//!   a `chunk_<uuid>.hyp/` directory, alongside its `VectorStore` mmap files and a
//!   [`ChunkManifest`] (`chunk.manifest`).
//! - The manifest records the exact storage layout (`element_size`, `storage_f32`,
//!   `dimension`) the chunk was written with, plus the chunk-local → global user ID
//!   mapping. Readers never re-derive the layout from environment variables, so a
//!   chunk is always decoded with the precision it was encoded with.
//! - Loaded chunk indexes are kept in a small process-wide LRU (`HS_CHUNK_CACHE_SIZE`,
//!   default 64 chunks). Chunks are immutable, so an entry only needs to be dropped
//!   when its directory is deleted ([`invalidate_chunk_cache`]).
//!
//! ## Thread Safety
//! All operations are `Send + Sync`. Chunk loading and search must run on a
//! blocking/Rayon thread (never directly on the async executor).

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use hyperspace_core::{FilterExpr, GlobalConfig, Metric, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use parking_lot::Mutex;
use rayon::prelude::*;

/// File name of the per-chunk layout + ID manifest.
pub const CHUNK_MANIFEST_FILE: &str = "chunk.manifest";
const CHUNK_MANIFEST_MAGIC: &[u8; 8] = b"HSCHKMF1";
const MANIFEST_FLAG_STORAGE_F32: u32 = 1;

/// Returns `true` when unquantized collections should store vectors as `f32`.
/// Enabled by default; opt out with `HS_STORAGE_FLOAT32=0|false|no|off`.
pub fn storage_f32_enabled(mode: QuantizationMode) -> bool {
    let disabled = std::env::var("HS_STORAGE_FLOAT32")
        .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "0" | "false" | "no" | "off"));
    !disabled && mode == QuantizationMode::None
}

/// Single source of truth for the per-vector byte size in `VectorStore`
/// (MemTable, flushed chunks and compacted chunks all share this layout).
pub fn storage_element_size<M: Metric>(
    mode: QuantizationMode,
    dimension: usize,
    storage_f32: bool,
) -> usize {
    let hybrid_head = if dimension == 801 && M::name() == "hybrid" {
        33
    } else {
        0
    };
    match mode {
        QuantizationMode::ScalarI8 => dimension + 4,
        QuantizationMode::Binary => dimension.div_ceil(8) + 4,
        QuantizationMode::AsymmetricHybrid801 => {
            if dimension > 33 {
                33 * 4 + (dimension - 33) + 4
            } else {
                dimension * 4 + 4
            }
        }
        QuantizationMode::AsymmetricHybridLowBit => {
            if dimension > 33 {
                let euc_dim = dimension - 33;
                33 * 4 + 4 + euc_dim.div_ceil(16) * 12
            } else {
                dimension * 4 + 4
            }
        }
        QuantizationMode::AsymmetricHybridExtreme => {
            if dimension > 33 {
                33 * 4 + 4 + (dimension - 33).div_ceil(8)
            } else {
                dimension * 4 + 4
            }
        }
        QuantizationMode::ScalarI4 => {
            let tail_dim = dimension.saturating_sub(hybrid_head);
            hybrid_head * 4 + 4 + tail_dim.div_ceil(16) * 12
        }
        QuantizationMode::Turbo => {
            let tail_dim = dimension.saturating_sub(hybrid_head);
            hybrid_head * 4 + 4 + tail_dim.div_ceil(2) + 4
        }
        QuantizationMode::Ternary => {
            let tail_dim = dimension.saturating_sub(hybrid_head);
            hybrid_head * 4 + 2 * tail_dim.div_ceil(8) + 8
        }
        QuantizationMode::ProductQuantization | QuantizationMode::OPQ => {
            hyperspace_core::pq::default_num_subvectors(dimension)
        }
        QuantizationMode::RaBitQ => hyperspace_core::rabitq::code_size(dimension),
        QuantizationMode::PQ4 => {
            hyperspace_core::pq4::code_size(hyperspace_core::pq4::default_num_subvectors(dimension))
        }
        QuantizationMode::None => {
            if storage_f32 {
                dimension * 4 + 4
            } else {
                dimension * 8 + 8
            }
        }
    }
}

/// Storage layout + chunk-local → global user ID mapping of an immutable chunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkManifest {
    pub element_size: u32,
    pub dimension: u32,
    pub storage_f32: bool,
    /// `ids[local_id] = user_id`.
    pub ids: Vec<u32>,
}

impl ChunkManifest {
    /// Atomically writes the manifest into `dir` (write to temp file, then rename).
    pub fn write(&self, dir: &Path) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(24 + self.ids.len() * 4);
        buf.extend_from_slice(CHUNK_MANIFEST_MAGIC);
        buf.extend_from_slice(&self.element_size.to_le_bytes());
        buf.extend_from_slice(&self.dimension.to_le_bytes());
        let flags = if self.storage_f32 {
            MANIFEST_FLAG_STORAGE_F32
        } else {
            0
        };
        buf.extend_from_slice(&flags.to_le_bytes());
        let count = u32::try_from(self.ids.len())
            .map_err(|_| std::io::Error::other("chunk manifest: too many ids"))?;
        buf.extend_from_slice(&count.to_le_bytes());
        for id in &self.ids {
            buf.extend_from_slice(&id.to_le_bytes());
        }

        let final_path = dir.join(CHUNK_MANIFEST_FILE);
        let tmp_path = dir.join(format!("{CHUNK_MANIFEST_FILE}.tmp"));
        {
            let mut f = std::fs::File::create(&tmp_path)?;
            f.write_all(&buf)?;
            f.sync_all()?;
        }
        std::fs::rename(tmp_path, final_path)
    }

    /// Reads the manifest from `dir`.
    pub fn read(dir: &Path) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        std::fs::File::open(dir.join(CHUNK_MANIFEST_FILE))?.read_to_end(&mut buf)?;
        let invalid =
            |msg: &str| std::io::Error::new(std::io::ErrorKind::InvalidData, msg.to_string());
        if buf.len() < 24 || &buf[..8] != CHUNK_MANIFEST_MAGIC {
            return Err(invalid("chunk manifest: bad header"));
        }
        let read_u32 =
            |off: usize| u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
        let element_size = read_u32(8);
        let dimension = read_u32(12);
        let flags = read_u32(16);
        let count = read_u32(20) as usize;
        if buf.len() != 24 + count * 4 {
            return Err(invalid("chunk manifest: truncated id table"));
        }
        let ids = buf[24..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| u32::from_le_bytes(*c))
            .collect();
        Ok(Self {
            element_size,
            dimension,
            storage_f32: flags & MANIFEST_FLAG_STORAGE_F32 != 0,
            ids,
        })
    }
}

/// A single result produced by an immutable chunk, already resolved to the global user ID.
#[derive(Debug, Clone)]
pub struct ChunkHit {
    pub user_id: u32,
    pub distance: f64,
    pub metadata: HashMap<String, String>,
}

struct LoadedChunk<M: Metric> {
    index: HnswIndex<M>,
    /// `None` for legacy chunks written before manifests existed (IDs not resolvable).
    ids: Option<Vec<u32>>,
}

struct CacheEntry {
    chunk: Arc<dyn Any + Send + Sync>,
    last_used: u64,
}

#[derive(Default)]
struct ChunkCache {
    entries: HashMap<(PathBuf, TypeId), CacheEntry>,
    tick: u64,
}

fn chunk_cache() -> &'static Mutex<ChunkCache> {
    static CACHE: OnceLock<Mutex<ChunkCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(ChunkCache::default()))
}

fn chunk_cache_capacity() -> usize {
    static CAP: OnceLock<usize> = OnceLock::new();
    *CAP.get_or_init(|| {
        std::env::var("HS_CHUNK_CACHE_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(64)
            .max(1)
    })
}

/// Drops every cached chunk whose directory lives under `prefix`.
/// Must be called before/after deleting chunk directories from disk.
pub fn invalidate_chunk_cache(prefix: &Path) {
    chunk_cache()
        .lock()
        .entries
        .retain(|(path, _), _| !path.starts_with(prefix));
}

fn load_chunk<M: Metric>(
    chunk_dir: &Path,
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    dimension: usize,
) -> Result<Arc<LoadedChunk<M>>, String> {
    let key = (chunk_dir.to_path_buf(), TypeId::of::<M>());
    {
        let mut cache = chunk_cache().lock();
        cache.tick += 1;
        let tick = cache.tick;
        if let Some(entry) = cache.entries.get_mut(&key) {
            entry.last_used = tick;
            if let Ok(chunk) = Arc::clone(&entry.chunk).downcast::<LoadedChunk<M>>() {
                return Ok(chunk);
            }
        }
    }

    let snap_path = chunk_dir.join("index.snap");
    if !snap_path.exists() {
        return Err(format!("Chunk snapshot not found: {}", snap_path.display()));
    }

    let (element_size, storage_f32, ids) = if let Ok(manifest) = ChunkManifest::read(chunk_dir) {
        if manifest.dimension as usize != dimension {
            return Err(format!(
                "Chunk {} dimension mismatch: manifest {} vs query {dimension}",
                chunk_dir.display(),
                manifest.dimension
            ));
        }
        (
            manifest.element_size as usize,
            manifest.storage_f32,
            Some(manifest.ids),
        )
    } else {
        eprintln!(
            "⚠️ ChunkSearcher: {} has no {CHUNK_MANIFEST_FILE}; legacy chunk results cannot be mapped to user IDs and are skipped.",
            chunk_dir.display()
        );
        let storage_f32 = storage_f32_enabled(mode);
        (
            storage_element_size::<M>(mode, dimension, storage_f32),
            storage_f32,
            None,
        )
    };

    let store = Arc::new(VectorStore::new(chunk_dir, element_size));
    let index = HnswIndex::<M>::load_snapshot_with_storage_precision(
        &snap_path,
        store,
        mode,
        Arc::clone(config),
        storage_f32,
        dimension,
    )?;
    let loaded = Arc::new(LoadedChunk { index, ids });

    let mut cache = chunk_cache().lock();
    cache.tick += 1;
    let tick = cache.tick;
    let capacity = chunk_cache_capacity();
    while cache.entries.len() >= capacity {
        let Some(oldest) = cache
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| k.clone())
        else {
            break;
        };
        cache.entries.remove(&oldest);
    }
    cache.entries.insert(
        key,
        CacheEntry {
            chunk: Arc::clone(&loaded) as Arc<dyn Any + Send + Sync>,
            last_used: tick,
        },
    );
    Ok(loaded)
}

/// Exports every live point of a chunk as `(user_id, vector, metadata)`.
///
/// Returns `Ok(None)` for legacy chunks without a manifest (IDs not resolvable):
/// callers must keep such chunks instead of rewriting them.
#[allow(clippy::type_complexity)]
pub fn export_chunk_points<M: Metric>(
    chunk_dir: &Path,
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    dimension: usize,
) -> Result<Option<Vec<(u32, Vec<f64>, HashMap<String, String>)>>, String> {
    let chunk = load_chunk::<M>(chunk_dir, mode, config, dimension)?;
    let Some(ids) = chunk.ids.as_ref() else {
        return Ok(None);
    };
    Ok(Some(
        chunk
            .index
            .peek_all()
            .into_iter()
            .filter_map(|(local_id, vec, meta)| {
                let user_id = *ids.get(local_id as usize)?;
                (user_id != u32::MAX).then_some((user_id, vec, meta))
            })
            .collect(),
    ))
}

/// Searches a single chunk (loading it into the chunk cache on first use).
///
/// Returns hits resolved to global user IDs. Legacy chunks without a manifest
/// yield no hits because their local IDs cannot be mapped back.
#[allow(clippy::too_many_arguments, clippy::implicit_hasher)]
pub fn search_chunk<M: Metric>(
    chunk_dir: &Path,
    query: &[f64],
    k: usize,
    ef_search: usize,
    filters: &HashMap<String, String>,
    complex_filters: &[FilterExpr],
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    use_wasserstein: bool,
) -> Result<Vec<ChunkHit>, String> {
    let chunk = load_chunk::<M>(chunk_dir, mode, config, query.len())?;
    let Some(ids) = chunk.ids.as_ref() else {
        return Ok(Vec::new());
    };

    let params = hyperspace_core::SearchParams {
        top_k: k,
        ef_search,
        hybrid_query: None,
        hybrid_alpha: None,
        component_weights: None,
        use_wasserstein,
        bm25_options: None,
        fusion_method: None,
        mrl_dimension: None,
        include_payload: false,
        use_wave: false,
    };

    let include_metadata = chunk.index.has_nonempty_metadata();
    let results = chunk.index.search(query, filters, complex_filters, &params);
    Ok(results
        .into_iter()
        .filter_map(|(local_id, distance)| {
            let user_id = *ids.get(local_id as usize)?;
            let metadata = if include_metadata {
                chunk
                    .index
                    .metadata
                    .forward
                    .get(&local_id)
                    .map(|m| m.clone())
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };
            Some(ChunkHit {
                user_id,
                distance,
                metadata,
            })
        })
        .collect())
}

/// Searches multiple chunks in parallel on the Rayon pool and merges the top-`k`.
///
/// Must be called from a blocking context (e.g. inside `spawn_blocking`).
#[allow(clippy::too_many_arguments, clippy::implicit_hasher)]
pub fn scatter_gather_search<M: Metric>(
    chunk_dirs: &[PathBuf],
    query: &[f64],
    k: usize,
    ef_search: usize,
    filters: &HashMap<String, String>,
    complex_filters: &[FilterExpr],
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    use_wasserstein: bool,
) -> Vec<ChunkHit> {
    let mut all: Vec<ChunkHit> = chunk_dirs
        .par_iter()
        .flat_map_iter(|dir| {
            match search_chunk::<M>(
                dir,
                query,
                k,
                ef_search,
                filters,
                complex_filters,
                mode,
                config,
                use_wasserstein,
            ) {
                Ok(hits) => hits,
                Err(e) => {
                    eprintln!("⚠️ ChunkSearcher: Failed to search chunk: {e}");
                    Vec::new()
                }
            }
        })
        .collect();
    all.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    all.truncate(k);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = ChunkManifest {
            element_size: 516,
            dimension: 128,
            storage_f32: true,
            ids: vec![7, 42, 1_000_000],
        };
        manifest.write(dir.path()).unwrap();
        assert_eq!(ChunkManifest::read(dir.path()).unwrap(), manifest);
    }

    #[test]
    fn manifest_rejects_garbage() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CHUNK_MANIFEST_FILE), b"nope").unwrap();
        assert!(ChunkManifest::read(dir.path()).is_err());
    }
}
