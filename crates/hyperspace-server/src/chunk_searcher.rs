//! # ChunkSearcher — On-Demand Immutable Chunk Loader (Task 1.2 Part B)
//!
//! Loads frozen HNSW chunk segments from disk via `mmap`, runs search inside
//! them, and returns candidate results to be merged with the MemTable.
//!
//! ## Design
//! - Each chunk is a self-contained HNSW index persisted as `index.snap` inside
//!   a `chunk_<uuid>.hyp/` directory, alongside its `VectorStore` mmap files.
//! - Loading is delegated to `HnswIndex::load_snapshot_with_storage_precision`,
//!   which already uses `memmap2` for zero-copy access.
//! - The OS Page Cache handles caching: frequently-accessed chunks stay warm in
//!   RAM transparently. No application-level LRU is needed until S3 tiering
//!   (Task 1.3).
//!
//! ## Thread Safety
//! All operations are `Send + Sync`. Chunk loading happens inside `spawn_blocking`
//! (or Rayon) to avoid blocking the async executor.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use hyperspace_core::{FilterExpr, GlobalConfig, Metric, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;

/// Loads a chunk snapshot from disk and performs a search.
///
/// This is stateless by design — we load, search, drop. The OS page cache
/// keeps hot chunks in memory between calls. When we add S3 tiering (Task 1.3)
/// this will be replaced by an LRU cache of `Arc<HnswIndex>`.
///
/// # Parameters
/// - `chunk_dir`: Path to the chunk directory (e.g. `data/chunk_<uuid>.hyp/`).
/// - `query`: The search query vector (already normalized if cosine).
/// - `k`: Number of top results to retrieve from this chunk.
/// - `ef_search`: HNSW ef parameter for search quality.
/// - `filters`: Legacy key-value metadata filters.
/// - `complex_filters`: Rich filter expressions.
/// - `mode`: Quantization mode used by this collection.
/// - `config`: Global HNSW configuration (M, ef_construct, etc.).
///
/// # Returns
/// `Vec<(internal_id, distance)>` — raw results from the chunk's local ID space.
/// The caller must NOT map these IDs through the collection's id_map since chunk
/// segments use their own internal IDs starting from 0.
#[allow(clippy::too_many_arguments)]
pub fn search_chunk<const N: usize, M: Metric<N>>(
    chunk_dir: &Path,
    query: &[f64],
    k: usize,
    ef_search: usize,
    filters: &HashMap<String, String>,
    complex_filters: &[FilterExpr],
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    use_wasserstein: bool,
) -> Result<Vec<(u32, f64)>, String> {
    let snap_path = chunk_dir.join("index.snap");
    if !snap_path.exists() {
        return Err(format!("Chunk snapshot not found: {}", snap_path.display()));
    }

    let storage_f32_requested = std::env::var("HS_STORAGE_FLOAT32")
        .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"));
    let storage_f32 = storage_f32_requested && mode == QuantizationMode::None;

    let element_size = match mode {
        QuantizationMode::ScalarI8 => hyperspace_core::vector::QuantizedHyperVector::<N>::SIZE,
        QuantizationMode::Binary => hyperspace_core::vector::BinaryHyperVector::<N>::SIZE,
        QuantizationMode::None => {
            if storage_f32 {
                hyperspace_core::vector::HyperVectorF32::<N>::SIZE
            } else {
                hyperspace_core::vector::HyperVector::<N>::SIZE
            }
        }
    };

    let store = Arc::new(VectorStore::new(chunk_dir, element_size));
    let chunk_index = HnswIndex::<N, M>::load_snapshot_with_storage_precision(
        &snap_path,
        store,
        mode,
        Arc::clone(config),
        storage_f32,
    )?;

    // Search inside the loaded chunk.
    let results = chunk_index.search(
        query,
        k,
        ef_search,
        filters,
        complex_filters,
        None, // hybrid_query not supported on chunk level (applied only on MemTable)
        None, // hybrid_alpha
        use_wasserstein,
    );

    Ok(results)
}

/// Searches multiple chunks in parallel using tokio (P1: Parallel chunk search).
///
/// # Parameters
/// - `chunk_dirs`: Paths to chunk directories to search.
/// - `query`: The search query vector.
/// - `k`: Number of top results to retrieve (total, after merge).
/// - `ef_search`: HNSW ef parameter.
/// - `filters` / `complex_filters`: Metadata filters.
/// - `mode`: Quantization mode.
/// - `config`: Global HNSW config.
///
/// # Returns
/// Merged and sorted `Vec<(internal_id_within_chunk, distance)>`, truncated to `k`.
/// Note: IDs are chunk-local and cannot be used for metadata lookups in the main index.
/// The caller should use only the distances for ranking merge.
#[allow(clippy::too_many_arguments)]
pub async fn scatter_gather_search_async<const N: usize, M: Metric<N> + Send + Sync + 'static>(
    chunk_dirs: &[std::path::PathBuf],
    query: &[f64],
    k: usize,
    ef_search: usize,
    filters: &HashMap<String, String>,
    complex_filters: &[FilterExpr],
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    use_wasserstein: bool,
) -> Vec<(u32, f64, usize)> {
    // P1: Parallel chunk search - use tokio to search chunks concurrently
    // Chunk loading is mmap-based, so parallel loading is safe and efficient
    let chunk_futures: Vec<_> = chunk_dirs
        .iter()
        .enumerate()
        .map(|(_chunk_idx, dir)| {
            let dir = dir.clone();
            let query = query.to_vec();
            let filters = filters.clone();
            let complex_filters = complex_filters.to_vec();
            let config = config.clone();

            tokio::task::spawn_blocking(move || {
                search_chunk::<N, M>(
                    &dir,
                    &query,
                    k,
                    ef_search,
                    &filters,
                    &complex_filters,
                    mode,
                    &config,
                    use_wasserstein,
                )
            })
        })
        .collect();

    // Collect results from all parallel tasks
    let mut all_results: Vec<(u32, f64, usize)> = Vec::new();

    for (chunk_idx, future) in chunk_futures.into_iter().enumerate() {
        match future.await {
            Ok(Ok(results)) => {
                for (local_id, dist) in results {
                    all_results.push((local_id, dist, chunk_idx));
                }
            }
            Ok(Err(e)) => {
                eprintln!("⚠️ ChunkSearcher: Failed to search chunk: {e}");
            }
            Err(e) => {
                eprintln!("⚠️ ChunkSearcher: Task panicked: {e}");
            }
        }
    }

    // Sort by distance ascending and truncate to k.
    all_results.sort_by(|a, b| a.1.total_cmp(&b.1));
    all_results.truncate(k);
    all_results
}

/// Legacy synchronous wrapper for backwards compatibility.
/// Spawns an async task and blocks on it - use scatter_gather_search_async when possible.
#[allow(clippy::too_many_arguments)]
pub fn scatter_gather_search<const N: usize, M: Metric<N> + Send + Sync + 'static>(
    chunk_dirs: &[std::path::PathBuf],
    query: &[f64],
    k: usize,
    ef_search: usize,
    filters: &HashMap<String, String>,
    complex_filters: &[FilterExpr],
    mode: QuantizationMode,
    config: &Arc<GlobalConfig>,
    use_wasserstein: bool,
) -> Vec<(u32, f64, usize)> {
    // Use tokio runtime to run async function
    let rt = tokio::runtime::Handle::current();
    rt.block_on(scatter_gather_search_async::<N, M>(
        chunk_dirs,
        query,
        k,
        ef_search,
        filters,
        complex_filters,
        mode,
        config,
        use_wasserstein,
    ))
}
