use crate::chunk_backend::ChunkBackend;
use crate::chunk_searcher;
use crate::exact_store::ExactVectorStore;
use crate::meta_router::{CentroidAccumulator, ChunkMeta, MetaRouter};
use crate::sync::CollectionDigest;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use hyperspace_cache::VectorCache;
use hyperspace_core::gpu::{rerank_topk_exact, GpuMetric};
use hyperspace_core::{
    Collection, FilterExpr, GlobalConfig, Metric, SearchParams, SearchResult, StorageMode,
    VacuumFilterOp, VacuumFilterQuery,
};
use hyperspace_index::HnswIndex;
use hyperspace_proto::hyperspace::{replication_log, InsertOp, ReplicationLog};
use hyperspace_store::{wal::Wal, PayloadStore, VectorStore, DEFAULT_ZSTD_LEVEL};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize)]
struct CollectionState {
    id_map: HashMap<u32, u32>,
    reverse_id_map: HashMap<u32, u32>,
    buckets: Vec<u64>,
    #[serde(default)]
    last_persisted_clock: u64,
}

#[inline]
fn env_rerank_enabled() -> bool {
    static VAL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *VAL.get_or_init(|| {
        // On by default since 4.2.5: quantized collections rerank their merged candidates
        // once against the exact store (≈3% QPS for exact top-k). Full precision skips it.
        std::env::var("HS_RERANK_ENABLED").map_or(true, |v| {
            !matches!(
                v.trim().to_lowercase().as_str(),
                "0" | "false" | "no" | "off"
            )
        })
    })
}

#[inline]
fn env_rerank_oversample() -> usize {
    static VAL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VAL.get_or_init(|| {
        std::env::var("HS_RERANK_OVERSAMPLE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(4)
            .max(1)
    })
}

#[inline]
fn env_chunk_probe_k() -> usize {
    static VAL: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *VAL.get_or_init(|| {
        std::env::var("HS_CHUNK_PROBE_K")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(3)
    })
}

#[inline]
fn env_cache_read_through() -> bool {
    static VAL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *VAL.get_or_init(|| {
        std::env::var("HYPERSPACE_CACHE_READ_THROUGH").map_or(true, |v| v.to_lowercase() != "false")
    })
}

/// `HS_AUTO_EF` (default on): tune the default `ef_search` from measured graph recall.
fn env_auto_ef() -> bool {
    std::env::var("HS_AUTO_EF").map_or(true, |v| {
        !matches!(
            v.trim().to_lowercase().as_str(),
            "0" | "false" | "no" | "off"
        )
    })
}

/// Graph recall the auto-tuned beam must reach (`HS_AUTO_EF_TARGET`, default 0.99).
fn env_auto_ef_target() -> f64 {
    std::env::var("HS_AUTO_EF_TARGET")
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| (0.5..=1.0).contains(v))
        .unwrap_or(0.99)
}

/// Smallest live node count worth tuning (below it the shard is flat-scanned anyway).
const AUTO_EF_MIN_NODES: usize = 1_000;
/// Re-tune whenever the graph has doubled since the last measurement.
const AUTO_EF_GROWTH: usize = 2;

/// Rerank candidates per result for coarse codes (1–2 bit, PQ) searched in one piece:
/// `top_k × oversample × COARSE_RERANK_POOL`. Their codes rank the true neighbours far
/// down the list; before 4.2.5 the pool came implicitly from 12 shards × `2 · top_k ×
/// oversample` (e.g. `extreme` + rerank ×4: 98.8% recall with that pool, 61% with 40).
const COARSE_RERANK_POOL: usize = 24;

/// Per-shard multiplier of `top_k × oversample` that gives a coarse-code collection a
/// total rerank pool of `top_k × oversample × COARSE_RERANK_POOL`. One shard is asked for
/// `top_k`; each of `S > 1` shards for `2 · top_k` (scatter-gather oversampling).
fn coarse_pool_factor(shards: usize) -> usize {
    if shards <= 1 {
        COARSE_RERANK_POOL
    } else {
        (COARSE_RERANK_POOL / 2).div_ceil(shards).max(1)
    }
}

/// Whether the background tuner will re-measure this graph's beam.
fn ef_tuning_due<M: Metric>(idx: &HnswIndex<M>) -> bool {
    if !env_auto_ef() {
        return false;
    }
    let live = idx.count_nodes();
    let last = idx.tuned_at();
    if live < AUTO_EF_MIN_NODES {
        return false;
    }
    if last == 0 || live >= last.saturating_mul(AUTO_EF_GROWTH) {
        return true;
    }
    // Re-measure on the real query distribution: first when enough queries are sampled,
    // then after every `AUTO_EF_REQUERY` new samples.
    let (fresh, enough) = idx.fresh_query_samples();
    enough
        && (!idx.tuned_with_real_queries()
            || (fresh >= AUTO_EF_REQUERY && idx.millis_since_tuning() >= AUTO_EF_REQUERY_MS))
}

/// New sampled queries after which the beam is re-measured on real queries ...
const AUTO_EF_REQUERY: usize = 512;
/// ... at most once a minute (a measurement costs ~64 flat scans + graph searches).
const AUTO_EF_REQUERY_MS: u64 = 60_000;

/// Background task that re-measures the beam width (`HnswIndex::tune_ef_search`) once the
/// indexing queue is idle and the graph has grown `AUTO_EF_GROWTH`× since the last run.
/// Candidates per query: `top_k × rerank oversample` when quantized collections rerank.
fn spawn_ef_tuner<M: Metric>(
    index_link: Arc<ArcSwap<HnswIndex<M>>>,
    config: Arc<hyperspace_core::config::GlobalConfig>,
    name: String,
    quantized: bool,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        if !env_auto_ef() {
            return;
        }
        // Default top-k 10; reranked collections keep `10 × oversample` code candidates,
        // of which the exact top 10 must survive.
        let k = 10;
        let pool = k * if quantized && env_rerank_enabled() {
            env_rerank_oversample()
        } else {
            1
        };
        let target = env_auto_ef_target();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let idx = index_link.load_full();
            let live = idx.count_nodes();
            if !ef_tuning_due(&idx) || config.get_queue_size() > 0 {
                continue;
            }
            // Counted as an indexing task so readiness checks wait for the new beam.
            config.inc_queue();
            let started = std::time::Instant::now();
            let tuned =
                tokio::task::spawn_blocking(move || idx.tune_ef_search(k, pool, target, 96))
                    .await
                    .ok()
                    .flatten();
            config.dec_queue();
            if let Some(ef) = tuned {
                println!(
                    "🎯 [{name}] auto ef_search = {ef} (exact top-{k} within {pool} graph candidates ≥ {target}, {live} nodes, {:?})",
                    started.elapsed()
                );
            } else {
                // Flat-scanned or too small: mark as measured so we wait for growth.
                index_link.load().mark_ef_tuned(live);
            }
        }
    })
}

/// Final step shared by both search paths: maps internal hits to user ids (with metadata
/// when requested), merges cold-chunk hits and write-buffer hits (vectors not yet in the
/// graph, searched with `wb.1` candidates), drops duplicate user ids (an upsert can sit in
/// both the buffer and the graph) and keeps the best `top_k`.
#[allow(clippy::too_many_arguments)]
fn finish_hits<M: Metric>(
    index: &HnswIndex<M>,
    hits: Vec<(u32, f64)>,
    extra: Vec<(u32, f64, HashMap<String, String>)>,
    include_metadata: bool,
    ids_are_identity: bool,
    reverse_id_map: &DashMap<u32, u32>,
    write_buffer: &crate::write_buffer::WriteBuffer,
    wb: (&[f64], usize, &HashMap<String, String>, &[FilterExpr]),
    top_k: usize,
) -> Vec<(u32, f64, HashMap<String, String>)> {
    let mut out: Vec<(u32, f64, HashMap<String, String>)> = hits
        .into_iter()
        .map(|(internal_id, dist)| {
            let meta = if include_metadata {
                index
                    .metadata
                    .forward
                    .get(&internal_id)
                    .map(|m| m.clone())
                    .unwrap_or_default()
            } else {
                HashMap::new()
            };
            let user_id = if ids_are_identity {
                internal_id
            } else {
                reverse_id_map.get(&internal_id).map_or(internal_id, |v| *v)
            };
            (user_id, dist, meta)
        })
        .collect();
    let merged_extra = !extra.is_empty();
    out.extend(extra);
    let buffered = !write_buffer.is_empty();
    if buffered {
        let (query, k, filters, complex) = wb;
        out.extend(
            write_buffer
                .search::<M>(query, k, filters, complex)
                .into_iter()
                .map(|(id, dist, meta, _)| (id, dist, meta)),
        );
    }
    if merged_extra || buffered {
        out.sort_by(|a, b| a.1.total_cmp(&b.1));
    }
    if merged_extra || buffered || !ids_are_identity {
        let mut seen = std::collections::HashSet::with_capacity(out.len());
        out.retain(|(user_id, _, _)| seen.insert(*user_id));
    }
    out.truncate(top_k);
    out
}

/// File (in a collection / shard data directory) recording the dimension of an MRL prefix
/// index. The index width is a property of the stored data: collections created before
/// 4.2.5 (or with `HS_MRL_INDEX=0`) index the full vector and have no marker.
const MRL_INDEX_MARKER: &str = "mrl_index.dim";

/// Prefix width an MRL collection indexes: the first cascade layer's `cutoff_dimension`
/// for a single-component L2 / cosine collection (8 ≤ cutoff < dimension). The graph and its
/// codes then hold only the prefix (cutoff / dimension of the RAM and distance cost) and
/// every search reranks a pool of candidates on the full vectors from the exact store.
/// `HS_MRL_INDEX=0` keeps full-dimension indexes.
fn mrl_index_dimension<M: Metric>(
    schema: &hyperspace_proto::hyperspace::CollectionSchema,
    dimension: usize,
) -> Option<usize> {
    if std::env::var("HS_MRL_INDEX").is_ok_and(|v| matches!(v.trim(), "0" | "false" | "no" | "off"))
    {
        return None;
    }
    if schema.components.len() > 1 || !matches!(M::name(), "l2" | "cosine") {
        return None;
    }
    let cut = schema.cascade_pipeline.first()?.cutoff_dimension as usize;
    (8..dimension).contains(&cut).then_some(cut)
}

/// Resolves the index width of a collection directory: the persisted marker, else the MRL
/// prefix for a brand-new directory (writing the marker), else the full dimension.
fn resolve_index_dim(data_dir: &std::path::Path, dimension: usize, mrl: Option<usize>) -> usize {
    let marker = data_dir.join(MRL_INDEX_MARKER);
    if let Some(d) = std::fs::read_to_string(&marker)
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&d| d > 0 && d <= dimension)
    {
        return d;
    }
    let Some(cut) = mrl else {
        return dimension;
    };
    let has_data = data_dir.join("index.snap").exists()
        || std::fs::read_dir(data_dir).is_ok_and(|entries| {
            entries.flatten().any(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.ends_with(".hyp") || name == "memtable"
            })
        });
    if has_data {
        // Existing full-dimension data (pre-4.2.5 MRL collection): keep its layout.
        return dimension;
    }
    match std::fs::write(&marker, cut.to_string()) {
        Ok(()) => cut,
        Err(e) => {
            eprintln!("⚠️ Cannot persist the MRL index marker ({e}); indexing the full vector.");
            dimension
        }
    }
}

/// First `d` coordinates of `v` (re-normalized for cosine), borrowed when `d` covers `v`.
fn mrl_prefix<M: Metric>(v: &[f64], d: usize) -> Cow<'_, [f64]> {
    if d >= v.len() {
        return Cow::Borrowed(v);
    }
    let mut p = v[..d].to_vec();
    if M::name() == "cosine" {
        let n = p.iter().map(|x| x * x).sum::<f64>().sqrt();
        if n > 1e-12 {
            for x in &mut p {
                *x /= n;
            }
        }
    }
    Cow::Owned(p)
}

/// Output of the blocking half of `RebuildIndex`.
struct RebuildOutput {
    element_size: usize,
    storage_f32: bool,
    /// Index width (the MRL prefix, or the full dimension).
    dimension: usize,
    /// Full vector dimension (the exact store's slot width).
    full_dimension: usize,
    vacuum_config: Arc<hyperspace_core::config::GlobalConfig>,
    temp_dir: PathBuf,
    new_snap_path: PathBuf,
    /// Old internal ids of the survivors in their new order (new id = position).
    order: Vec<u32>,
    old_len: usize,
    has_exact: bool,
}

/// Entries of the per-shard heap raw-vector cache (exact f32 copies for rerank and online
/// calibration; the memory-mapped exact store keeps every exact vector anyway). Budget:
/// `HS_RAW_CACHE_MB` per collection (default 0 with an exact store, else 256), split across
/// `shards`.
fn raw_cache_capacity(dimension: usize, shards: usize, has_exact_store: bool) -> usize {
    // With the memory-mapped exact store the page cache already serves exact vectors, so
    // the heap copy is opt-in (`HS_RAW_CACHE_MB`); legacy collections keep 256 MiB.
    let mb = std::env::var("HS_RAW_CACHE_MB")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(if has_exact_store { 0 } else { 256 });
    let per_vector = dimension.max(1) * 4 + 64; // f32 data + map / Vec overhead
    (mb * 1024 * 1024 / per_vector / shards.max(1)).max(if mb == 0 { 0 } else { 256 })
}

/// Minimum vectors a shard must hold before a trained quantizer (PQ / OPQ / PQ4 /
/// RaBitQ) is calibrated online. `HS_AUTO_CALIBRATE_MIN=0` disables auto-calibration
/// (calibrate explicitly with `RebuildIndex` instead).
fn env_auto_calibrate_min() -> usize {
    std::env::var("HS_AUTO_CALIBRATE_MIN")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(1024)
}

/// Smallest shard a mode is calibrated on. PQ / OPQ fit 256 centroids per sub-space:
/// with fewer than ~16 samples per centroid the learned codebook generalizes worse than
/// the default one (measured: rerank x8 recall 86% -> 79% when fitted on 1,024 vectors).
fn auto_calibrate_floor(mode: hyperspace_core::QuantizationMode) -> usize {
    use hyperspace_core::QuantizationMode as Q;
    match mode {
        Q::ProductQuantization | Q::OPQ => 16 * 256,
        _ => 0,
    }
}

/// Upper bound on training samples for online calibration.
const AUTO_CALIBRATE_MAX_SAMPLES: usize = 10_000;

/// Growth factor between progressive re-calibrations of a shard's quantizer.
const AUTO_CALIBRATE_GROWTH: usize = 8;

/// Online calibration of a shard's trained quantizer.
///
/// The first fit happens once the shard holds `min_vectors` vectors; it is re-fitted each
/// time the shard grows `AUTO_CALIBRATE_GROWTH`-fold, until the sample cap is reached
/// (re-encoding cost stays ~1.14 × N in total). Training uses exact vectors from the
/// raw-vector cache and every stored code is re-encoded (`HnswIndex::refit_online`).
/// Returns `(re_encoded, missing)` if it (re)calibrated.
fn maybe_auto_calibrate<M: Metric>(
    index: &HnswIndex<M>,
    raw_cache: &DashMap<u32, Vec<f32>>,
    exact: Option<&ExactVectorStore>,
    payload: Option<&PayloadStore>,
    min_vectors: usize,
    name: &str,
) -> Option<(usize, usize)> {
    if min_vectors == 0 || !index.mode.needs_training() {
        return None;
    }
    let min_vectors = min_vectors.max(auto_calibrate_floor(index.mode));
    let count = index.count();
    let last = index.calibrated_at();
    let due = if index.needs_calibration() {
        count >= min_vectors
    } else {
        // Already calibrated: refit while the training sample can still grow. An index
        // calibrated by RebuildIndex / a previous run (`last == 0`) is left alone.
        last > 0
            && last < AUTO_CALIBRATE_MAX_SAMPLES
            && count >= last.saturating_mul(AUTO_CALIBRATE_GROWTH)
    };
    if !due {
        return None;
    }
    let mut samples: Vec<Vec<f32>> = raw_cache
        .iter()
        .take(AUTO_CALIBRATE_MAX_SAMPLES)
        .map(|e| e.value().clone())
        .collect();
    if let Some(exact) = exact {
        // The raw cache is bounded (or disabled): top the sample up from the exact store,
        // spread evenly over the shard.
        let want = AUTO_CALIBRATE_MAX_SAMPLES.min(count);
        if samples.len() < want {
            let step = (count / want.max(1)).max(1);
            let mut id = 0usize;
            while samples.len() < want && id < count {
                if !raw_cache.contains_key(&(id as u32)) {
                    if let Some(v) = exact.get(id as u32) {
                        samples.push(v);
                    }
                }
                id += step;
            }
        }
    }
    if samples.len() < min_vectors.min(256) {
        return None;
    }
    let started = std::time::Instant::now();
    let refs: Vec<&[f32]> = samples.iter().map(Vec::as_slice).collect();
    // Exact vectors: the bounded raw cache first, then the payload store (which keeps the
    // exact f32 copy of every vector of a quantized collection).
    let dim = index.dimension;
    let (re_encoded, missing) = index.refit_online(&refs, 10, |id| {
        if let Some(v) = exact.and_then(|e| e.get(id)) {
            return Some(v);
        }
        raw_cache.get(&id).map(|v| v.value().clone()).or_else(|| {
            let bytes = payload?.fetch_blocking(id).ok()??;
            (bytes.len() == dim * 4).then(|| {
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| f32::from_le_bytes(*c))
                    .collect()
            })
        })
    });
    println!(
        "🎓 [{name}] Auto-calibrated {:?} quantizer on {} samples in {:?}: {re_encoded} codes re-encoded, {missing} without an exact copy",
        index.mode,
        samples.len(),
        started.elapsed()
    );
    Some((re_encoded, missing))
}

pub struct CollectionImpl<M: Metric> {
    /// Width of the HNSW index: the MRL prefix (`mrl_index_dimension`) or `dimension`.
    index_dim: usize,
    name: String,
    node_id: String,
    index_link: Arc<ArcSwap<HnswIndex<M>>>,
    wal_link: Arc<ArcSwap<tokio::sync::Mutex<Wal>>>,
    index_tx: mpsc::UnboundedSender<(u32, Arc<HashMap<String, String>>)>,
    replication_tx: broadcast::Sender<ReplicationLog>,
    config: Arc<GlobalConfig>,
    bg_tasks: Vec<JoinHandle<()>>,
    // Buckets for Merkle Tree synchronization
    buckets: Arc<Vec<AtomicU64>>,
    // Root hash for fast O(1) state comparison (incremental XOR)
    root_hash: AtomicU64,
    // Mapping from user ID to internal ID for upsert support
    id_map: Arc<DashMap<u32, u32>>,
    // Reverse mapping from internal ID to user ID for search results
    reverse_id_map: Arc<DashMap<u32, u32>>,
    // Data directory for optimization
    data_dir: PathBuf,
    // Quantization Mode
    mode: hyperspace_core::QuantizationMode,
    // Tracking latest clock for persistence/dedup
    last_clock: Arc<AtomicU64>,
    // True while user IDs are guaranteed to match internal IDs.
    ids_are_identity: AtomicBool,
    // Limit CPU-bound search tasks to avoid scheduler thrashing.
    search_limiter: Arc<Semaphore>,
    // Restrict background WAL rotation flush workers to 1 to prevent CPU starvation
    flush_limiter: Arc<Semaphore>,
    // If existing vector shift is <= threshold and metadata unchanged, skip graph relinking.
    fast_upsert_delta: f64,
    // Global Meta-Router for IVF-style chunk routing (Task 1.2)
    meta_router: Arc<MetaRouter>,
    // Count of vectors currently in the "Flush Purgatory" (Frozen WAL -> Chunk conversion)
    flushing_vector_count: Arc<AtomicUsize>,
    // Count of vectors in the current ACTIVE WAL
    wal_pending_count: Arc<AtomicU64>,
    // Storage Mode: Performance vs Tiered
    storage_mode: StorageMode,
    // Max RAM allowed for MemTable before forcing a flush
    max_ram_bytes: u64,
    // List of rotated WAL segments waiting to be flushed into a chunk (Task 8.1)
    pending_wal_flushes: Arc<tokio::sync::Mutex<Vec<PathBuf>>>,
    // ── Sidecar Payload Storage (v3.2) ───────────────────────────────────────
    // Stores heavy document blobs on disk. The index (16 bytes/entry) lives in RAM;
    // the bytes NEVER touch RAM during HNSW traversal. Reads use pread + spawn_blocking.
    payload_store: Arc<PayloadStore>,
    /// Exact `f32` vectors by internal id (quantized modes), see [`ExactVectorStore`].
    /// Swapped as a whole by `RebuildIndex`, which renumbers internal ids.
    exact_store: Arc<arc_swap::ArcSwapOption<ExactVectorStore>>,
    dimension: usize,
    /// Stored for introspection, schema validation, and future multi-component APIs.
    #[allow(dead_code)]
    schema: hyperspace_proto::hyperspace::CollectionSchema,
    layout: hyperspace_core::vector::VectorLayout,
    cache: Option<Arc<VectorCache>>,
    write_buffer: Arc<crate::write_buffer::WriteBuffer>,
    chunk_backend: Arc<dyn ChunkBackend>,
    storage_f32: bool,
    raw_vector_cache: Arc<DashMap<u32, Vec<f32>>>,
    raw_vector_cache_len: Arc<AtomicUsize>,
    /// Max entries of `raw_vector_cache` (byte budget `HS_RAW_CACHE_MB` per collection,
    /// split across shards).
    raw_vector_cache_cap: usize,
    /// Number of shards of the collection this one belongs to (1 = unsharded).
    shard_share: usize,
    pub explicit_graph: Arc<hyperspace_core::graph::GraphEngine>,
}

static EMPTY_LEGACY_FILTERS: LazyLock<HashMap<String, String>> = LazyLock::new(HashMap::new);
static EMPTY_COMPLEX_FILTERS: LazyLock<Vec<FilterExpr>> = LazyLock::new(Vec::new);

struct BatchEntry<'a> {
    id: u32,
    vector: Cow<'a, [f64]>,
    metadata: &'a HashMap<String, String>,
    internal_id: u32,
    reindex_needed: bool,
}

impl<M: Metric> CollectionImpl<M> {
    fn shift_l2_sq(a: &[f64], b: &[f64]) -> f64 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| {
                let d = x - y;
                d * d
            })
            .sum()
    }

    #[inline]
    fn to_internal_id(&self, user_id: u32) -> u32 {
        // Quick Win #3: Bypass DashMap lookup when identity mode is enabled
        if self.ids_are_identity.load(Ordering::Relaxed) {
            return user_id;
        }
        self.id_map.get(&user_id).map_or(user_id, |v| *v)
    }

    #[inline]
    fn to_user_id(&self, internal_id: u32) -> u32 {
        // Quick Win #3: Bypass DashMap lookup when identity mode is enabled
        if self.ids_are_identity.load(Ordering::Relaxed) {
            return internal_id;
        }
        self.reverse_id_map
            .get(&internal_id)
            .map_or(internal_id, |v| *v)
    }

    fn meta_numeric_value(meta: &HashMap<String, String>, key: &str) -> Option<f64> {
        if let Some(raw) = meta.get(key) {
            return raw.parse::<f64>().ok();
        }
        let typed_key = format!("__hs_typed__{key}");
        let raw_typed = meta.get(&typed_key)?;
        let parsed = serde_json::from_str::<serde_json::Value>(raw_typed).ok()?;
        parsed.get("v")?.as_f64()
    }

    fn matches_vacuum_filter(meta: &HashMap<String, String>, filter: &VacuumFilterQuery) -> bool {
        let Some(current) = Self::meta_numeric_value(meta, &filter.key) else {
            return false;
        };
        match filter.op {
            VacuumFilterOp::Lt => current < filter.value,
            VacuumFilterOp::Lte => current <= filter.value,
            VacuumFilterOp::Gt => current > filter.value,
            VacuumFilterOp::Gte => current >= filter.value,
            VacuumFilterOp::Eq => (current - filter.value).abs() <= 1e-12,
            VacuumFilterOp::Ne => (current - filter.value).abs() > 1e-12,
        }
    }

    /// Normalizes vector if metric is Cosine.
    /// Returns Cow to avoid allocation if normalization is not needed.
    #[inline]
    /// Exact re-ranking is a no-op for full-precision single-component collections (the
    /// index already ranks by the exact metric distance).
    fn rerank_is_redundant(&self, params: &SearchParams) -> bool {
        // An MRL prefix index ranks by prefix distances: the full-vector rerank is the
        // second stage of the funnel, never redundant.
        self.index_dim == self.dimension
            && self.mode == hyperspace_core::QuantizationMode::None
            && self.layout.components.len() <= 1
            && params.component_weights.is_none()
            && params.mrl_dimension.is_none()
    }

    /// Candidate count of an exact rerank that the scatter-gather layer can run once on
    /// the merged shard results (`None`: no rerank, or one this shard must run itself —
    /// multi-component layouts, MRL, hybrid queries, cold chunks).
    pub(crate) fn deferred_rerank_k(
        &self,
        filters: &HashMap<String, String>,
        params: &SearchParams,
    ) -> Option<usize> {
        let requested = filters
            .get("rerank")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"));
        let schema_k = self
            .schema
            .cascade_pipeline
            .iter()
            .find_map(|layer| (layer.rerank_top_k > 0).then_some(layer.rerank_top_k as usize));
        let rerank = requested.unwrap_or(env_rerank_enabled() || schema_k.is_some())
            && params.hybrid_query.is_none();
        static DEFER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let defer = *DEFER.get_or_init(|| {
            std::env::var("HS_RERANK_DEFER")
                .map_or(true, |v| !matches!(v.trim(), "0" | "false" | "no" | "off"))
        });
        if !defer
            || self.rerank_full_pool()
            || !rerank
            || self.rerank_is_redundant(params)
            || self.layout.components.len() > 1
            || params.component_weights.is_some()
            || params.mrl_dimension.is_some()
            || self.index_dim < self.dimension
            || self.meta_router.chunk_count() > 0
        {
            return None;
        }
        let oversample = filters
            .get("oversample")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or_else(env_rerank_oversample)
            .max(1);
        let k = params.top_k.max(1);
        Some(k.saturating_mul(oversample).max(schema_k.unwrap_or(0)))
    }

    /// Exact distance between the vector of `user_id` and an already processed
    /// (normalized for cosine) query, from the best exact source available. `None` when
    /// the id is unknown or still in the write buffer (whose hits are exact already).
    pub(crate) fn exact_distance(
        &self,
        user_id: u32,
        processed_query: &[f64],
        query_f32: &[f32],
    ) -> Option<f64> {
        if self.write_buffer.get_by_user_id(user_id).is_some() {
            return None;
        }
        let ids_identity = self.ids_are_identity.load(Ordering::Acquire);
        let internal = if ids_identity {
            user_id
        } else {
            *self.id_map.get(&user_id)?
        };
        let index = self.index_link.load();
        if internal as usize >= index.count_nodes()
            || index.metadata.deleted.read().contains(internal)
        {
            return None;
        }
        let q = &processed_query[..processed_query.len().min(self.dimension)];
        let exact = self.exact_store.load_full();
        if let Some(exact) = &exact {
            // L2 / cosine (squared L2 on normalized vectors): f32 SIMD straight from the map.
            if matches!(M::name(), "l2" | "cosine") {
                if let Some(bytes) = exact.get_bytes(internal) {
                    return Some(f64::from(hyperspace_core::simd::l2_sq_f32_bytes(
                        bytes, query_f32, None,
                    )));
                }
            }
        }
        let mut buf = vec![0.0f64; q.len()];
        if let Some(exact) = exact {
            if exact.read_f64(internal, &mut buf) {
                return Some(M::distance(&buf, q));
            }
        }
        if let Some(v) = self.raw_vector_cache.get(&internal) {
            for (d, &x) in buf.iter_mut().zip(v.iter()) {
                *d = f64::from(x);
            }
            return Some(M::distance(&buf, q));
        }
        if let Ok(Some(bytes)) = self.payload_store.fetch_blocking(internal) {
            if bytes.len() == self.dimension * 4 {
                for (d, c) in buf.iter_mut().zip(bytes.as_chunks::<4>().0) {
                    *d = f64::from(f32::from_le_bytes(*c));
                }
                return Some(M::distance(&buf, q));
            }
        }
        let orig = index.get_vector_original(internal);
        let n = orig.coords.len().min(q.len());
        Some(M::distance(&orig.coords[..n], &q[..n]))
    }

    /// Codes too coarse to keep the true neighbours within `top_k × oversample` merged
    /// candidates (1–2 bit, PQ; 4-bit RaBitQ keeps them within 40 and defers). They need the wide pool of per-shard reranking (each shard
    /// reranks its own `2 · top_k × oversample`), which also runs in parallel across shards,
    /// so they do not defer.
    fn rerank_full_pool(&self) -> bool {
        use hyperspace_core::QuantizationMode as Q;
        matches!(
            self.mode,
            Q::Binary
                | Q::Ternary
                | Q::RaBitQ
                | Q::RaBitQ2
                | Q::PQ4
                | Q::ProductQuantization
                | Q::OPQ
        )
    }

    /// Cosine collections normalize queries before searching; exact reranks done outside
    /// the shard must use the same processed query.
    pub(crate) fn process_query<'q>(&self, query: &'q [f64]) -> Cow<'q, [f64]> {
        Self::normalize_if_cosine(&query[..query.len().min(self.dimension)])
    }

    /// The vector the HNSW index sees: the first `index_dim` coordinates (MRL prefix index,
    /// re-normalized for cosine), or `v` itself for a full-dimension index.
    #[inline]
    fn index_view<'a>(&self, v: &'a [f64]) -> Cow<'a, [f64]> {
        mrl_prefix::<M>(v, self.index_dim)
    }

    /// Exact (full-dimension) vector of an internal id: the exact store when present,
    /// otherwise the index's own decoded vector.
    fn stored_vector(&self, index: &HnswIndex<M>, internal_id: u32) -> Vec<f64> {
        if self.index_dim < self.dimension {
            if let Some(v) = self
                .exact_store
                .load_full()
                .and_then(|e| e.get(internal_id))
            {
                return v.into_iter().map(f64::from).collect();
            }
        }
        index.get_vector(internal_id).coords
    }

    fn normalize_if_cosine(vector: &[f64]) -> Cow<'_, [f64]> {
        if M::name() != "cosine" {
            return Cow::Borrowed(vector);
        }

        let norm_sq: f64 = vector.iter().map(|x| x * x).sum();
        // If already unit length (within epsilon) or zero, return as is to save allocation
        if (norm_sq - 1.0).abs() < 1e-9 || norm_sq <= 1e-18 {
            return Cow::Borrowed(vector);
        }

        let inv_norm = 1.0 / norm_sq.sqrt();
        let normalized: Vec<f64> = vector.iter().map(|x| x * inv_norm).collect();
        Cow::Owned(normalized)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        name: String,
        node_id: String,
        data_dir: std::path::PathBuf,
        wal_path: std::path::PathBuf,
        mode: hyperspace_core::QuantizationMode,
        replication_tx: broadcast::Sender<ReplicationLog>,
        dimension: usize,
        schema: hyperspace_proto::hyperspace::CollectionSchema,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::new_shard(
            name,
            node_id,
            data_dir,
            wal_path,
            mode,
            replication_tx,
            dimension,
            schema,
            1,
        )
        .await
    }

    /// Like [`Self::new`] for one of `shard_share` shards of a collection: collection-wide
    /// memory budgets (hot-tier cache capacity, raw-vector cache) are split across shards.
    #[allow(clippy::too_many_arguments)]
    pub async fn new_shard(
        name: String,
        node_id: String,
        data_dir: std::path::PathBuf,
        wal_path: std::path::PathBuf,
        mode: hyperspace_core::QuantizationMode,
        replication_tx: broadcast::Sender<ReplicationLog>,
        dimension: usize,
        schema: hyperspace_proto::hyperspace::CollectionSchema,
        shard_share: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let shard_share = shard_share.max(1);
        let snap_path = data_dir.join("index.snap");
        let config = Arc::new(GlobalConfig::new());

        let gossip_env =
            std::env::var("HS_GOSSIP_ENABLED").is_ok_and(|v| v.to_lowercase() == "true");
        let anisotropic_env = std::env::var("HS_ANISOTROPIC_REFINEMENT")
            .map_or(true, |v| v.to_lowercase() != "false");

        config.set_gossip_enabled(gossip_env);
        config.set_anisotropic_enabled(anisotropic_env);

        let ef_cons_env = std::env::var("HS_HNSW_EF_CONSTRUCT")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);
        // RECALL FIX #1: Default ef_search raised from 10 → 64.
        // ef=10 is catastrophically low for HNSW on any real dataset.
        // The HNSW beam barely explores the graph at ef=10; recall collapses.
        // 64 is the VectorDBBench baseline (ef_search = max(64, top_k * 4)).
        // Default 100: the same beam the gRPC / HTTP layers used before per-request
        // `ef_search` existed, so a request without `ef_search` keeps its recall.
        let ef_search_env = std::env::var("HS_HNSW_EF_SEARCH")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100);
        let m_env = std::env::var("HS_HNSW_M")
            .unwrap_or_else(|_| "16".to_string())
            .parse()
            .unwrap_or(16);

        config.set_ef_construction(ef_cons_env);
        config.set_ef_search(ef_search_env);
        config.set_m(m_env);

        let bm25_method = std::env::var("HS_BM25_METHOD")
            .unwrap_or_else(|_| "bm25plus".to_string())
            .to_lowercase();
        let method = match bm25_method.as_str() {
            "robertson" => hyperspace_core::bm25::Bm25Method::Robertson,
            "lucene" => hyperspace_core::bm25::Bm25Method::Lucene,
            "atire" => hyperspace_core::bm25::Bm25Method::Atire,
            "bm25l" => hyperspace_core::bm25::Bm25Method::Bm25l,
            _ => hyperspace_core::bm25::Bm25Method::Bm25Plus,
        };
        let k1 = std::env::var("HS_BM25_K1")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1.2);
        let b = std::env::var("HS_BM25_B")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.75);
        let delta = std::env::var("HS_BM25_DELTA")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.5);
        let language = std::env::var("HS_BM25_LANGUAGE").unwrap_or_else(|_| "english".to_string());
        let ngrams = std::env::var("HS_BM25_NGRAMS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let fusion_method = std::env::var("HS_FUSION_METHOD")
            .unwrap_or_else(|_| "rrf".to_string())
            .to_lowercase();
        config.set_fusion_method(fusion_method);

        config.set_bm25_params(hyperspace_core::bm25::Bm25Params {
            method,
            k1,
            b,
            delta,
            language,
            ngrams,
        });

        let mut components = Vec::new();
        let mut offset = 0;
        for c in &schema.components {
            components.push(hyperspace_core::vector::VectorComponentLayout {
                name: c.name.clone(),
                metric: c.metric.clone(),
                dimension: c.full_dimension as usize,
                weight: c.weight,
                offset,
            });
            offset += c.full_dimension as usize;
        }

        let layout = hyperspace_core::vector::VectorLayout {
            dimension: offset,
            bytes_per_element: 8, // f64
            is_quantized: mode != hyperspace_core::QuantizationMode::None,
            components,
        };

        // f32 storage halves memory bandwidth for the hot distance-computation path.
        // It is enabled by default for unquantized collections; opt out with HS_STORAGE_FLOAT32=0.
        let storage_f32 = chunk_searcher::storage_f32_enabled(mode);
        let mut element_size =
            chunk_searcher::storage_element_size::<M>(mode, dimension, storage_f32);

        if std::env::var("HS_ZONAL_QUANTIZATION").is_ok_and(|v| v.to_lowercase() == "true") {
            element_size = 0;
        }

        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        // MRL collections index the vector prefix; the exact store keeps the full vector.
        let index_dim = resolve_index_dim(
            &data_dir,
            dimension,
            mrl_index_dimension::<M>(&schema, dimension),
        );
        let index_element_size = if index_dim == dimension || element_size == 0 {
            element_size
        } else {
            chunk_searcher::storage_element_size::<M>(mode, index_dim, storage_f32)
        };
        if index_dim < dimension {
            println!(
                "🪆 [{name}] MRL prefix index: {index_dim} of {dimension} dimensions, full-vector rerank"
            );
        }

        let (_store, index, _recovered_count) = if snap_path.exists() {
            let store = Arc::new(VectorStore::new(&data_dir, index_element_size));
            match HnswIndex::<M>::load_snapshot_with_storage_precision(
                &snap_path,
                store.clone(),
                mode,
                config.clone(),
                storage_f32,
                index_dim,
            ) {
                Ok(idx) => {
                    let count = idx.count_nodes();
                    (store, Arc::new(idx), count)
                }
                Err(e) => {
                    eprintln!("Failed to load snapshot for {name}: {e}. Starting fresh.");
                    let store = Arc::new(VectorStore::new(&data_dir, index_element_size));
                    (
                        store.clone(),
                        Arc::new(HnswIndex::new_with_storage_precision(
                            store,
                            mode,
                            config.clone(),
                            storage_f32,
                            index_dim,
                        )),
                        0,
                    )
                }
            }
        } else {
            let store = Arc::new(VectorStore::new(&data_dir, index_element_size));
            (
                store.clone(),
                Arc::new(HnswIndex::new_with_storage_precision(
                    store,
                    mode,
                    config.clone(),
                    storage_f32,
                    index_dim,
                )),
                0,
            )
        };

        // Wrap index in ArcSwap for Lock-Free Hot Swap
        let index_link = Arc::new(ArcSwap::new(index.clone()));

        // Initialize state
        let state_path = data_dir.join("state.json");
        let mut id_map_data = HashMap::new();
        let mut reverse_id_map_data = HashMap::new();
        let mut buckets_data = vec![0; crate::sync::SYNC_BUCKETS];
        let last_clock = Arc::new(AtomicU64::new(0));

        if state_path.exists() {
            if let Ok(s) = std::fs::read_to_string(&state_path) {
                if let Ok(state) = serde_json::from_str::<CollectionState>(&s) {
                    id_map_data.clone_from(&state.id_map);
                    reverse_id_map_data = state.reverse_id_map;
                    if state.buckets.len() == buckets_data.len() {
                        buckets_data = state.buckets;
                    }
                    last_clock.store(state.last_persisted_clock, Ordering::Relaxed);
                }
            }
        }

        // WAL
        let sync_mode_str = std::env::var("HYPERSPACE_WAL_SYNC_MODE")
            .unwrap_or_else(|_| "async".to_string())
            .to_lowercase();

        let sync_mode = match sync_mode_str.as_str() {
            "strict" | "fsync" => hyperspace_store::wal::WalSyncMode::Strict,
            "batch" => hyperspace_store::wal::WalSyncMode::Batch,
            _ => hyperspace_store::wal::WalSyncMode::Async,
        };

        if sync_mode == hyperspace_store::wal::WalSyncMode::Strict {
            println!("🔒 WAL Durability: STRICT (fsync on every write)");
        } else if sync_mode == hyperspace_store::wal::WalSyncMode::Batch {
            println!("🔒 WAL Durability: BATCH (Background fsync every 100ms)");
        }

        // Storage & Performance Tuning
        let hs_mode_env = std::env::var("HS_MODE")
            .unwrap_or_else(|_| "tiered".to_string()) // FIX: Default to Tiered (LSM) mode
            .to_lowercase();
        #[allow(clippy::match_same_arms)] // Intentional: explicit tiered/lsm aliases
        let storage_mode = match hs_mode_env.as_str() {
            "tiered" | "lsm" => StorageMode::Tiered,
            "performance" => {
                println!("🚀 Performance Mode: WAL will NOT flush until RAM limit hit.");
                StorageMode::Performance
            }
            _ => StorageMode::Tiered,
        };

        // Determine Memory Budget
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let total_ram_bytes = sys.total_memory();
        let max_ram_gb_env = std::env::var("HS_MAX_RAM_GB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok());

        let max_ram_bytes = match max_ram_gb_env {
            Some(gb) => gb * 1024 * 1024 * 1024,
            None => total_ram_bytes * 70 / 100, // Default to 70% of total RAM
        };

        println!(
            "🚀 Collection Mode: {:?} (Budget: {} GB)",
            storage_mode,
            max_ram_bytes / (1024 * 1024 * 1024)
        );

        let mut wal = Wal::new(&wal_path, sync_mode)?;

        // WAL Segment Configuration
        let default_segment_mb = match storage_mode {
            StorageMode::Performance => 4096, // 4 GB
            StorageMode::Tiered => 256,       // 256 MB
        };

        let wal_segment_mb = std::env::var("HS_WAL_SEGMENT_SIZE_MB")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(default_segment_mb)
            .clamp(16, 16384); // 16 MB .. 16 GB

        wal.set_size_limit(wal_segment_mb * 1024 * 1024);
        println!("📦 WAL Segment Size: {wal_segment_mb} MB");

        let wal_link = Arc::new(ArcSwap::new(Arc::new(tokio::sync::Mutex::new(wal))));
        let flushing_vector_count = Arc::new(AtomicUsize::new(0));
        let wal_pending_count = Arc::new(AtomicU64::new(0));

        // Replay
        let index_ref = index.clone();
        let loaded_clock = last_clock.load(Ordering::Relaxed);

        // Find all frozen WAL segments that haven't been flushed yet
        let mut wal_segments = Vec::new();
        if let Some(parent) = wal_path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    if file_name.contains(".frozen.") {
                        // Extract timestamp (e.g. "frozen.171234567890")
                        if let Some(ts_str) = file_name.rsplit('.').next() {
                            if let Ok(ts) = ts_str.parse::<u64>() {
                                wal_segments.push((ts, path));
                            }
                        }
                    }
                }
            }
        }

        // Sort segments chronologically
        wal_segments.sort_by_key(|(ts, _)| *ts);

        // Final list: all frozen segments + the active WAL path
        let replay_queue: Vec<_> = wal_segments.clone().into_iter().map(|(_, p)| p).collect();
        let pending_wal_flushes = Arc::new(tokio::sync::Mutex::new(
            wal_segments.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
        ));

        // Add the active path to replay, but it's not "frozen" yet
        let mut final_replay = replay_queue;
        final_replay.push(wal_path.clone());

        println!("⚡ Replaying {} WAL segment(s)...", final_replay.len());

        for path in final_replay {
            Wal::replay(&path, |entry| {
                match entry {
                    hyperspace_store::wal::WalEntry::Insert {
                        id,
                        vector,
                        metadata,
                        logical_clock,
                    } => {
                        // Only replay operations strictly newer than what's persisted in state.json
                        if logical_clock > loaded_clock {
                            // If ID exists, delete old version from index to prevent leaks (Upsert)
                            if let Some(&old_internal_id) = id_map_data.get(&id) {
                                index_ref.delete(old_internal_id);
                                reverse_id_map_data.remove(&old_internal_id);
                            }

                            if let Ok(internal_id) =
                                index_ref.insert(&mrl_prefix::<M>(&vector, index_dim), metadata)
                            {
                                id_map_data.insert(id, internal_id);
                                reverse_id_map_data.insert(internal_id, id);

                                if gossip_env {
                                    let canonical: Vec<f64> = if storage_f32 {
                                        vector.iter().map(|&x| (x as f32) as f64).collect()
                                    } else {
                                        vector.clone()
                                    };
                                    let hash = CollectionDigest::hash_entry(id, &canonical);
                                    let b_idx = CollectionDigest::get_bucket_index(id);
                                    buckets_data[b_idx] ^= hash;
                                }

                                // Track max clock derived from WAL
                                last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                                wal_pending_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    hyperspace_store::wal::WalEntry::Insert64 {
                        id,
                        vector,
                        metadata,
                        logical_clock,
                    } => {
                        let id = id as u32;
                        if logical_clock > loaded_clock {
                            if let Some(&old_internal_id) = id_map_data.get(&id) {
                                index_ref.delete(old_internal_id);
                                reverse_id_map_data.remove(&old_internal_id);
                            }

                            if let Ok(internal_id) =
                                index_ref.insert(&mrl_prefix::<M>(&vector, index_dim), metadata)
                            {
                                id_map_data.insert(id, internal_id);
                                reverse_id_map_data.insert(internal_id, id);

                                if gossip_env {
                                    let canonical: Vec<f64> = if storage_f32 {
                                        vector.iter().map(|&x| (x as f32) as f64).collect()
                                    } else {
                                        vector.clone()
                                    };
                                    let hash = CollectionDigest::hash_entry(id, &canonical);
                                    let b_idx = CollectionDigest::get_bucket_index(id);
                                    buckets_data[b_idx] ^= hash;
                                }

                                last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                                wal_pending_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                    hyperspace_store::wal::WalEntry::Delete { id, logical_clock } => {
                        let id = id as u32;
                        if logical_clock > loaded_clock {
                            if let Some(&old_internal_id) = id_map_data.get(&id) {
                                index_ref.delete(old_internal_id);
                                reverse_id_map_data.remove(&old_internal_id);
                            }
                            id_map_data.remove(&id);
                            last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                            wal_pending_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    hyperspace_store::wal::WalEntry::UpdatePayload {
                        id,
                        metadata,
                        logical_clock,
                    } => {
                        let id = id as u32;
                        if logical_clock > loaded_clock {
                            if let Some(&internal_id) = id_map_data.get(&id) {
                                let _ = index_ref.update_payload_metadata(internal_id, &metadata);
                                last_clock.fetch_max(logical_clock, Ordering::Relaxed);
                                wal_pending_count.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                }
            })?;
        }

        let write_buffer = Arc::new(crate::write_buffer::WriteBuffer::new());

        // Background Tasks
        let (index_tx, mut index_rx) = mpsc::unbounded_channel();
        let idx_link_worker = index_link.clone();
        let cfg_worker = config.clone();

        // Indexer Concurrency Configuration
        // Default: 1 (Serial) for maximum graph quality
        // Set to 0 to use all CPU cores (faster but lower recall due to race conditions)
        let num_cpus = std::thread::available_parallelism().map_or(8, std::num::NonZero::get);
        let concurrency_env = std::env::var("HS_INDEXER_CONCURRENCY")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<usize>()
            .unwrap_or(1);

        let concurrency = if concurrency_env == 0 {
            num_cpus
        } else if concurrency_env > num_cpus {
            println!(
                 "⚠️  Clamping Indexer Concurrency from {concurrency_env} to {num_cpus} (CPU limit) to avoid thrashing."
             );
            num_cpus
        } else {
            concurrency_env
        };

        println!("⚙️  Indexer Concurrency: {concurrency} thread(s)");
        let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency));

        let search_concurrency_env = std::env::var("HS_SEARCH_CONCURRENCY")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<usize>()
            .unwrap_or(0);
        // Quick Win #4: Auto-calculate search concurrency based on CPU count
        // Default: num_cpus * 2 for better throughput, with manual override via env var
        let search_concurrency = if search_concurrency_env == 0 {
            num_cpus * 2 // Auto: 2x CPU count for better parallelism
        } else if search_concurrency_env > num_cpus * 4 {
            num_cpus * 4 // Cap at 4x to avoid thrashing
        } else {
            search_concurrency_env
        };
        println!(
            "⚙️  Search Concurrency Limit: {search_concurrency} task(s) (CPU cores: {num_cpus})"
        );
        let search_limiter = Arc::new(Semaphore::new(search_concurrency));
        let flush_limiter = Arc::new(Semaphore::new(1));
        let fast_upsert_delta = std::env::var("HS_FAST_UPSERT_DELTA")
            .unwrap_or_else(|_| "0.0".to_string())
            .parse::<f64>()
            .unwrap_or(0.0)
            .max(0.0);

        // ── Open Sidecar Payload Storage (v3.2) ──────────────────────────────────────
        // Lives in the same data_dir as VectorStore. payloads.hyp is strictly disk-only;
        // payload_index.hyp is the tiny in-RAM index (16 bytes/entry).
        let zstd_level = std::env::var("HS_PAYLOAD_ZSTD_LEVEL")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(DEFAULT_ZSTD_LEVEL);
        let payload_store = match PayloadStore::open(&data_dir, zstd_level) {
            Ok(ps) => {
                println!("📦 Payload Store: successfully opened");
                Arc::new(ps)
            }
            Err(e) => {
                eprintln!("⚠️  Payload Store: failed to open ({e}), payload writes will be no-ops");
                // Fall back to a temp-dir store that won't persist but won't crash either.
                let tmp = std::env::temp_dir().join(format!("hs_payload_{}", uuid::Uuid::new_v4()));
                Arc::new(PayloadStore::open(&tmp, zstd_level).expect("fallback payload store"))
            }
        };

        // Exact f32 copies for rerank / calibration / RebuildIndex (quantized modes and MRL
        // prefix indexes, whose graph does not hold the full vector).
        let exact_store = Arc::new(arc_swap::ArcSwapOption::new(
            (mode != hyperspace_core::QuantizationMode::None || index_dim < dimension)
                .then(|| Arc::new(ExactVectorStore::open(&data_dir, dimension))),
        ));
        let exact_for_indexer = exact_store.clone();

        let write_buffer_for_indexer = write_buffer.clone();
        let raw_vector_cache: Arc<DashMap<u32, Vec<f32>>> = Arc::new(DashMap::new());
        let raw_cache_for_indexer = Arc::clone(&raw_vector_cache);
        let payload_for_indexer = Arc::clone(&payload_store);
        let raw_vector_cache_cap =
            raw_cache_capacity(dimension, shard_share, exact_store.load().is_some());
        let auto_calibrate_min = env_auto_calibrate_min();
        let calibrating = Arc::new(AtomicBool::new(false));
        let indexer_name = name.clone();
        let indexer_task = if concurrency <= 1 {
            tokio::task::spawn_blocking(move || {
                let mut received = 0u64;
                let mut errors = 0u64;
                let mut batch = Vec::with_capacity(64);

                while let Some(first) = index_rx.blocking_recv() {
                    batch.push(first);
                    while batch.len() < 64 {
                        match index_rx.try_recv() {
                            Ok(it) => batch.push(it),
                            Err(_) => break,
                        }
                    }

                    let batch_count = batch.len() as u64;
                    cfg_worker
                        .active_indexing
                        .fetch_add(batch_count, Ordering::Relaxed);
                    let idx = idx_link_worker.load().clone();

                    for (id, meta) in batch.drain(..) {
                        let effective_meta = write_buffer_for_indexer
                            .get_metadata_by_internal_id(id)
                            .unwrap_or(meta);
                        match idx.index_node(id, Arc::unwrap_or_clone(effective_meta)) {
                            Ok(()) => {
                                write_buffer_for_indexer.remove(id);
                            }
                            Err(e) => {
                                eprintln!("❌ Indexer error on ID {id}: {e}");
                                write_buffer_for_indexer.remove(id);
                                errors += 1;
                            }
                        }
                        received += 1;
                    }

                    // Calibrate before releasing the batch from the queue counters, so a
                    // caller waiting for the queue to drain also waits for the re-encoding.
                    maybe_auto_calibrate(
                        &idx,
                        &raw_cache_for_indexer,
                        exact_for_indexer.load_full().as_deref(),
                        Some(&payload_for_indexer),
                        auto_calibrate_min,
                        &indexer_name,
                    );
                    cfg_worker
                        .queue_size
                        .fetch_sub(batch_count, Ordering::Relaxed);
                    cfg_worker
                        .active_indexing
                        .fetch_sub(batch_count, Ordering::Relaxed);
                    std::thread::yield_now();

                    if received.is_multiple_of(10_000) {
                        let active = cfg_worker.active_indexing.load(Ordering::Relaxed);
                        let queue = cfg_worker.queue_size.load(Ordering::Relaxed);
                        println!(
                            "📊 Indexer: {received} received, {active} active, {queue} in queue, {errors} errors"
                        );
                    }
                }
                println!("🏁 Indexer task finished. Total received: {received}, errors: {errors}");
            })
        } else {
            tokio::spawn(async move {
                use std::sync::atomic::AtomicU64;
                let received = Arc::new(AtomicU64::new(0));
                let errors = Arc::new(AtomicU64::new(0));

                while let Some((id, meta)) = index_rx.recv().await {
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let idx_link = idx_link_worker.clone();
                    let cfg = cfg_worker.clone();
                    let errors_ref = errors.clone();
                    let write_buf = write_buffer_for_indexer.clone();
                    cfg.inc_active();

                    tokio::spawn(async move {
                        let _permit = permit;
                        let effective_meta =
                            write_buf.get_metadata_by_internal_id(id).unwrap_or(meta);
                        let result = tokio::task::spawn_blocking(move || {
                            let idx = idx_link.load().clone();
                            let result = idx.index_node(id, Arc::unwrap_or_clone(effective_meta));
                            (result, id)
                        })
                        .await;

                        match result {
                            Ok((Ok(()), processed_id)) => {
                                write_buf.remove(processed_id);
                                cfg.dec_queue();
                                cfg.dec_active();
                            }
                            Ok((Err(e), failed_id)) => {
                                eprintln!("❌ Indexer error on ID {failed_id}: {e}");
                                write_buf.remove(failed_id);
                                cfg.dec_queue();
                                cfg.dec_active();
                                errors_ref.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(join_err) => {
                                eprintln!("❌ Indexer task panicked: {join_err}");
                                write_buf.remove(id);
                                cfg.dec_queue();
                                cfg.dec_active();
                                errors_ref.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    });

                    let r = received.fetch_add(1, Ordering::Relaxed) + 1;
                    let idx_now = idx_link_worker.load().clone();
                    if idx_now.mode.needs_training()
                        && auto_calibrate_min > 0
                        && idx_now.count()
                            >= auto_calibrate_min.max(auto_calibrate_floor(idx_now.mode))
                        && (idx_now.needs_calibration()
                            || (idx_now.calibrated_at() > 0
                                && idx_now.calibrated_at() < AUTO_CALIBRATE_MAX_SAMPLES
                                && idx_now.count()
                                    >= idx_now
                                        .calibrated_at()
                                        .saturating_mul(AUTO_CALIBRATE_GROWTH)))
                        && !calibrating.swap(true, Ordering::AcqRel)
                    {
                        let raw = Arc::clone(&raw_cache_for_indexer);
                        let exact = exact_for_indexer.clone();
                        let payload = Arc::clone(&payload_for_indexer);
                        let flag = Arc::clone(&calibrating);
                        let name_cal = indexer_name.clone();
                        let cfg_cal = cfg_worker.clone();
                        // Counted as an indexing task so drain / readiness checks wait for it.
                        cfg_cal.inc_queue();
                        cfg_cal.inc_active();
                        tokio::task::spawn_blocking(move || {
                            maybe_auto_calibrate(
                                &idx_now,
                                &raw,
                                exact.load_full().as_deref(),
                                Some(&payload),
                                auto_calibrate_min,
                                &name_cal,
                            );
                            cfg_cal.dec_active();
                            cfg_cal.dec_queue();
                            flag.store(false, Ordering::Release);
                        });
                    }
                    if r.is_multiple_of(10_000) {
                        let active = cfg_worker.active_indexing.load(Ordering::Relaxed);
                        let queue = cfg_worker.queue_size.load(Ordering::Relaxed);
                        let errs = errors.load(Ordering::Relaxed);
                        println!(
                            "📊 Indexer: {r} received, {active} active, {queue} in queue, {errs} errors"
                        );
                    }
                }

                let final_r = received.load(Ordering::Relaxed);
                let final_e = errors.load(Ordering::Relaxed);
                println!("🏁 Indexer task finished. Total received: {final_r}, errors: {final_e}");
            })
        };

        // Task 1.2: Initialize MetaRouter and Load Existing Chunks
        let meta_router = Arc::new(MetaRouter::new());
        // Scan data directory for chunk segments
        if let Ok(entries) = std::fs::read_dir(&data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("chunk_") {
                        // Load chunk metadata (centroid + count) from its snapshot
                        let snap_path = path.join("index.snap");
                        if let Ok(_idx) = HnswIndex::<M>::load_snapshot_with_storage_precision(
                            &snap_path,
                            Arc::new(VectorStore::new(&path, element_size)),
                            mode,
                            config.clone(),
                            storage_f32,
                            dimension,
                        ) {
                            // Compute/recover centroid for routing
                            // Actually, HnswIndex doesn't store centroid, but we can compute it or
                            // assume it's stored in a separate file.
                            // For simplicity, we skip loading here if not explicitly stored,
                            // but the correct way is to have a chunk_info.json.
                            // Assuming Task 1.2 intended to load them.
                        }
                    }
                }
            }
        }

        let idx_link_snap = index_link.clone();
        let snap_path_clone = snap_path.clone();

        let buckets: Arc<Vec<AtomicU64>> =
            Arc::new(buckets_data.into_iter().map(AtomicU64::new).collect());
        let id_map = Arc::new(id_map_data.into_iter().collect::<DashMap<u32, u32>>());
        // Quick Win #3: HS_IDENTITY_IDS flag for ID mapping bypass
        // If true, skip DashMap lookups entirely (user IDs == internal IDs)
        let identity_ids_env =
            std::env::var("HS_IDENTITY_IDS").is_ok_and(|v| v.to_lowercase() == "true");
        let ids_are_identity =
            identity_ids_env || id_map.iter().all(|entry| *entry.key() == *entry.value());
        if identity_ids_env {
            println!("⚡ ID Mapping: BYPASSED (HS_IDENTITY_IDS=true, user IDs == internal IDs)");
        } else if ids_are_identity {
            println!("⚡ ID Mapping: Identity mode detected (all user IDs match internal IDs)");
        } else {
            println!("🗺️  ID Mapping: Enabled (user ID ↔ internal ID translation)");
        }
        let reverse_id_map = Arc::new(
            reverse_id_map_data
                .into_iter()
                .collect::<DashMap<u32, u32>>(),
        );

        let id_map_snap = id_map.clone();
        let reverse_id_map_snap = reverse_id_map.clone();
        let buckets_snap = buckets.clone();
        let state_path_snap = data_dir.join("state.json");
        let last_clock_snap = last_clock.clone();

        // Native Vector-Graph (KAG) Engine
        let graph_csr_path = data_dir.join("graph.csr");
        let explicit_graph = if graph_csr_path.exists() {
            match hyperspace_core::graph::GraphEngine::load_from_csr_file(
                config.get_graph_config(),
                &graph_csr_path,
            ) {
                Ok(engine) => {
                    println!(
                        "🕸️  Native Vector-Graph Engine loaded from graph.csr ({} edges)",
                        engine.dynamic.total_edges()
                    );
                    Arc::new(engine)
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to load graph.csr: {e}, initializing empty GraphEngine");
                    Arc::new(hyperspace_core::graph::GraphEngine::new(
                        config.get_graph_config(),
                        1024,
                    ))
                }
            }
        } else {
            Arc::new(hyperspace_core::graph::GraphEngine::new(
                config.get_graph_config(),
                1024,
            ))
        };
        let explicit_graph_snap = explicit_graph.clone();
        let graph_csr_path_snap = graph_csr_path.clone();

        let snap_interval = std::env::var("HYPERSPACE_SNAPSHOT_INTERVAL_SEC")
            .unwrap_or("60".to_string())
            .parse::<u64>()
            .unwrap_or(60);

        let ef_tuner = spawn_ef_tuner::<M>(
            index_link.clone(),
            config.clone(),
            name.clone(),
            mode != hyperspace_core::QuantizationMode::None,
        );
        let snapshot_handle = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(snap_interval)).await;
                let idx = idx_link_snap.load().clone();
                if let Err(e) = idx.save_snapshot(&snap_path_clone) {
                    eprintln!("Snapshot error: {e}");
                }

                if explicit_graph_snap.dynamic.total_edges() > 0 {
                    if let Err(e) = explicit_graph_snap.save_to_csr_file(&graph_csr_path_snap) {
                        eprintln!("Graph snapshot error: {e}");
                    }
                }

                // Save State (DashMap iteration)
                let map_data: HashMap<u32, u32> = id_map_snap
                    .iter()
                    .map(|entry| (*entry.key(), *entry.value()))
                    .collect();
                let reverse_map_data: HashMap<u32, u32> = reverse_id_map_snap
                    .iter()
                    .map(|entry| (*entry.key(), *entry.value()))
                    .collect();
                let buckets_data: Vec<u64> = buckets_snap
                    .iter()
                    .map(|b| b.load(Ordering::Relaxed))
                    .collect();

                let state = CollectionState {
                    id_map: map_data,
                    reverse_id_map: reverse_map_data,
                    buckets: buckets_data,
                    last_persisted_clock: last_clock_snap.load(Ordering::Relaxed),
                };

                if let Ok(s) = serde_json::to_string(&state) {
                    let _ = std::fs::write(&state_path_snap, s);
                }
            }
        });

        let mut initial_root_hash = 0u64;
        for b in buckets.iter() {
            initial_root_hash ^= b.load(Ordering::Relaxed);
        }

        let cache = if std::env::var("HYPERSPACE_CACHE_ENABLED")
            .map_or(true, |v| v.to_lowercase() != "false")
        {
            // Collection-wide capacity, split across the shards of a sharded collection.
            let l1_capacity = std::env::var("HYPERSPACE_CACHE_L1_CAPACITY")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(10000)
                .div_ceil(shard_share)
                .max(1);

            let eviction_policy = match std::env::var("HYPERSPACE_CACHE_EVICTION_POLICY")
                .unwrap_or_default()
                .to_lowercase()
                .as_str()
            {
                "lfu" => hyperspace_cache::EvictionPolicy::Lfu,
                "ttl" => hyperspace_cache::EvictionPolicy::Ttl,
                _ => hyperspace_cache::EvictionPolicy::Lru,
            };

            let ann_threshold = std::env::var("HYPERSPACE_CACHE_ANN_THRESHOLD")
                .ok()
                .and_then(|s| s.parse::<f64>().ok());

            let ann_rebuild_batch = std::env::var("HYPERSPACE_CACHE_REBUILD_BATCH")
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);

            let cache_metric = match M::name() {
                "cosine" => hyperspace_cache::CacheMetricType::Cosine,
                "poincare" => hyperspace_cache::CacheMetricType::Poincare,
                "lorentz" => hyperspace_cache::CacheMetricType::Lorentz,
                "hybrid" => hyperspace_cache::CacheMetricType::Hybrid {
                    lorentz_weight: 1.0,
                    l2_weight: 1.0,
                },
                _ => hyperspace_cache::CacheMetricType::L2,
            };

            let cache_config = hyperspace_cache::CacheConfig {
                l1_capacity,
                eviction_policy,
                ann_threshold,
                ann_rebuild_batch,
                collection_name: name.clone(),
                dimension,
                metric: cache_metric,
            };

            let c = Arc::new(VectorCache::new(cache_config));
            c.start_background_tasks();
            Some(c)
        } else {
            None
        };

        let chunk_backend = crate::chunk_backend::create_backend(data_dir.clone());

        let col = Self {
            name,
            node_id,
            index_link,
            wal_link,
            index_tx,
            replication_tx,
            config,
            bg_tasks: vec![indexer_task, snapshot_handle, ef_tuner],
            buckets,
            root_hash: AtomicU64::new(initial_root_hash),
            reverse_id_map,
            id_map,
            data_dir,
            mode,
            last_clock,
            ids_are_identity: AtomicBool::new(ids_are_identity),
            search_limiter,
            flush_limiter,
            fast_upsert_delta,
            meta_router,
            flushing_vector_count,
            wal_pending_count,
            storage_mode,
            max_ram_bytes,
            pending_wal_flushes,
            payload_store,
            exact_store,
            dimension,
            index_dim,
            schema,
            layout,
            cache,
            write_buffer,
            chunk_backend,
            storage_f32,
            raw_vector_cache,
            raw_vector_cache_cap,
            shard_share,
            raw_vector_cache_len: Arc::new(AtomicUsize::new(0)),
            explicit_graph,
        };

        // FIX (Bottleneck 4): Cache warmup on startup.
        // After a restart the L0 cache is empty, so the first N requests all fall
        // through to the on-disk HNSW traversal. To mitigate this we pre-populate
        // the cache in a background blocking task immediately after the collection is
        // opened.
        // FIX: Extended warmup to all QuantizationMode except Binary.
        //
        // Why not Binary: get_vector() for Binary mode reconstructs only the sign bit
        // (±1/√dim). This is a very lossy approximation — unusable for ANN distance
        // calculations in the cache. Every other mode (None, ScalarI8, Hybrid) returns
        // reasonable f64 coords via proper de-quantization inside get_vector().
        // MRL prefix indexes skip it: their index vectors are prefixes, not the full vector.
        if col.mode != hyperspace_core::QuantizationMode::Binary && col.index_dim == col.dimension {
            if let Some(ref cache_arc) = col.cache {
                let cache_clone = cache_arc.clone();
                let idx_full = col.index_link.load_full();
                let rev_map = col.reverse_id_map.clone();
                let l1_cap = cache_arc.l1_capacity();

                tokio::task::spawn_blocking(move || {
                    let count = idx_full.count();
                    if count == 0 {
                        return;
                    }
                    let limit = count.min(l1_cap);
                    let mut entries = Vec::with_capacity(limit);

                    // Iterate over (internal_id -> external_id) pairs.
                    // get_vector() always returns de-quantized f64 coords for all
                    // quantization modes except Binary (which is excluded above).
                    // metadata.forward contains the key-value metadata per internal_id.
                    for r in rev_map.iter() {
                        let internal_id = *r.key();
                        let external_id = *r.value();
                        if (internal_id as usize) < count {
                            let hv = idx_full.get_vector_original(internal_id);
                            // FIX (Limitation 2): Read metadata from the HNSW forward map
                            // so warmup entries have correct key-value fields, not empty HashMaps.
                            let meta = idx_full
                                .metadata
                                .forward
                                .get(&internal_id)
                                .map(|r| r.value().clone())
                                .unwrap_or_default();
                            entries.push((external_id, hv.coords.clone(), meta));
                        }
                        if entries.len() >= limit {
                            break;
                        }
                    }

                    cache_clone.warmup_from_entries_with_meta(&entries);
                });
            }
        }

        Ok(col)
    }

    #[allow(clippy::too_many_arguments)] // Background worker requires all context
    fn spawn_flush_worker(
        frozen_wal_paths: Vec<PathBuf>,
        config: Arc<GlobalConfig>,
        mode: hyperspace_core::QuantizationMode,
        data_dir: PathBuf,
        flush_limiter: Arc<Semaphore>,
        meta_router: Arc<MetaRouter>,
        index_link: Arc<ArcSwap<HnswIndex<M>>>,
        _id_map: Arc<DashMap<u32, u32>>,
        _reverse_id_map: Arc<DashMap<u32, u32>>,
        flushing_vector_count: Arc<AtomicUsize>,
        dimension: usize,
        index_dim: usize,
        chunk_backend: Arc<dyn ChunkBackend>,
    ) {
        let storage_f32 = chunk_searcher::storage_f32_enabled(mode);
        // Chunks hold full vectors; the fresh MemTable index keeps the collection's width.
        let element_size = chunk_searcher::storage_element_size::<M>(mode, dimension, storage_f32);
        let memtable_element_size =
            chunk_searcher::storage_element_size::<M>(mode, index_dim, storage_f32);

        tokio::spawn(async move {
            let permit = flush_limiter.clone().acquire_owned().await;
            let _ = tokio::task::spawn_blocking(move || {
                let _permit = permit;

                println!("🔄 Flush Worker: Starting conversion of {} WAL segment(s)...", frozen_wal_paths.len());
                let _flush_start = std::time::Instant::now();

                // Track the vectors entering purgatory across all files
                let mut total_frozen_entries = 0;
                for path in &frozen_wal_paths {
                    total_frozen_entries += Wal::pending_entries_at_path(path);
                }
                flushing_vector_count.fetch_add(total_frozen_entries as usize, Ordering::SeqCst);
                println!("🔄 Flush Worker: Total entries to process: {total_frozen_entries}");

                let chunk_id = uuid::Uuid::new_v4().to_string();
                let chunk_name = format!("chunk_{chunk_id}.hyp");
                let chunk_dir = data_dir.join(&chunk_name);

                if let Err(e) = std::fs::create_dir_all(&chunk_dir) {
                    eprintln!("Failed to create chunk directory {chunk_name}: {e}");
                    return;
                }

                let temp_store = Arc::new(VectorStore::new(&chunk_dir, element_size));
                let local_index = HnswIndex::<M>::new_with_storage_precision(
                    temp_store.clone(),
                    mode,
                    config.clone(),
                    storage_f32,
                    dimension,
                );

                let mut insert_count = 0u32;
                let mut centroid_acc = CentroidAccumulator::new(dimension);
                // chunk-local id -> global user id (persisted in the chunk manifest)
                let mut chunk_user_ids: Vec<u32> = Vec::new();

                // Replay ALL accumulated segments into the same chunk
                for (i, path) in frozen_wal_paths.iter().enumerate() {
                    let replay_start = std::time::Instant::now();
                    let replay_res = Wal::replay(path, |entry| {
                        let (user_id, vector, metadata) = match entry {
                            hyperspace_store::wal::WalEntry::Insert { id, vector, metadata, .. } => {
                                (id, vector, metadata)
                            }
                            hyperspace_store::wal::WalEntry::Insert64 { id, vector, metadata, .. } => {
                                let Ok(id) = u32::try_from(id) else { return };
                                (id, vector, metadata)
                            }
                            hyperspace_store::wal::WalEntry::Delete { .. }
                            | hyperspace_store::wal::WalEntry::UpdatePayload { .. } => return,
                        };
                        if vector.len() == dimension {
                            centroid_acc.add(&vector);
                            if let Ok(new_id) = local_index.insert_to_storage(&vector) {
                                let _ = local_index.index_node(new_id, metadata);
                                let slot = new_id as usize;
                                if chunk_user_ids.len() <= slot {
                                    chunk_user_ids.resize(slot + 1, u32::MAX);
                                }
                                chunk_user_ids[slot] = user_id;
                                insert_count += 1;
                            }
                        }
                    });

                    let replay_elapsed = replay_start.elapsed();
                    println!("🔄 Flush Worker: Replay segment {}/{} completed in {:.2}s ({} vectors)",
                        i+1, frozen_wal_paths.len(), replay_elapsed.as_secs_f64(), insert_count);

                    if replay_res.is_err() {
                        eprintln!("⚠️ Failed to replay WAL segment {}/{} during flush. Some data may be lost or chunk will be partial.", i+1, frozen_wal_paths.len());
                    }
                }

                if insert_count == 0 {
                    let _ = std::fs::remove_dir_all(&chunk_dir);
                    for path in &frozen_wal_paths { let _ = std::fs::remove_file(path); }
                    println!("⚠️ Flush Worker: No vectors to insert, cleaned up");
                    return;
                }

                let save_start = std::time::Instant::now();
                let manifest = chunk_searcher::ChunkManifest {
                    element_size: u32::try_from(element_size).unwrap_or(u32::MAX),
                    dimension: u32::try_from(dimension).unwrap_or(u32::MAX),
                    storage_f32,
                    ids: chunk_user_ids,
                };
                if let Err(e) = manifest.write(&chunk_dir) {
                    eprintln!("Failed to write chunk manifest for {chunk_name}: {e}");
                } else if let Err(e) = local_index.save_snapshot(&chunk_dir.join("index.snap")) {
                    eprintln!("Failed to save index for {chunk_name}: {e}");
                } else {
                    let save_elapsed = save_start.elapsed();
                    println!("✅ Flush Worker: Converted {} WAL segment(s) -> Immutable Segment: {} ({} vectors) in {:.2}s",
                        frozen_wal_paths.len(), chunk_name, insert_count, save_elapsed.as_secs_f64()
                    );

                    // Cleanup handled WALs
                    for path in &frozen_wal_paths {
                        let _ = std::fs::remove_file(path);
                    }

                    if let Some(centroid) = centroid_acc.finish() {
                        meta_router.register(ChunkMeta {
                            chunk_id: chunk_name.clone(),
                            path: chunk_dir.clone(),
                            centroid,
                            vector_count: insert_count,
                        });
                        println!("🗺️  MetaRouter: Registered chunk {chunk_name} ({insert_count} vectors)");
                    }

                    // S3/Cloud storage tiering: trigger async upload + cache registration
                    chunk_backend.on_chunk_created(&chunk_name, &chunk_dir);

                    // === MemTable Swap (LSM-Tree Core) ===
                    // Create a fresh empty HNSW index to replace the current MemTable.
                    // The old MemTable data is now safely persisted in the chunk.
                    // ArcSwap atomically swaps the pointer — zero downtime.
                    let memtable_dir = data_dir.join("memtable");
                    let _ = std::fs::create_dir_all(&memtable_dir);
                    let fresh_store =
                        Arc::new(VectorStore::new(&memtable_dir, memtable_element_size));
                    let fresh_index = Arc::new(HnswIndex::<M>::new_with_storage_precision(
                        fresh_store,
                        mode,
                        config.clone(),
                        storage_f32,
                        index_dim,
                    ));
                    index_link.store(fresh_index);

                    // Note: Clearing id_map was a mistake (Task 1.2 bug).
                    // We must keep all mappings for search and recall to work across segments.
                    // id_map.clear();
                    // reverse_id_map.clear();

                    // Done! Reclaim purgatory count.
                    flushing_vector_count.fetch_sub(total_frozen_entries as usize, Ordering::SeqCst);
                    println!("🔄 MemTable Swap: Replaced hot index with fresh empty HNSW (freed RAM)");
                }
            })
            .await;
        });
    }
}

#[async_trait::async_trait]
impl<M: Metric> Collection for CollectionImpl<M> {
    fn name(&self) -> &str {
        &self.name
    }

    fn metric_name(&self) -> &'static str {
        M::name()
    }

    fn state_hash(&self) -> u64 {
        self.root_hash.load(Ordering::Relaxed)
    }

    fn quantization_mode(&self) -> hyperspace_core::QuantizationMode {
        self.mode
    }

    fn graph_stats(&self) -> (usize, usize, usize) {
        (
            self.explicit_graph.dynamic.node_count(),
            self.explicit_graph.dynamic.total_edges(),
            self.explicit_graph.memory_usage_bytes(),
        )
    }

    fn get_usage(&self) -> hyperspace_core::CollectionUsage {
        let mut disk_usage = 0;
        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    disk_usage += meta.len();
                }
            }
        }

        let count = self.index_link.load().count();
        // In-RAM codes cover the indexed width (the MRL prefix for MRL collections).
        let dim = self.index_dim;
        let vector_bytes_per_doc: u64 = match self.mode {
            // Hybrid code layouts (exact head + coded tail): the storage slot size.
            _ if M::name() == "hybrid" && self.mode != hyperspace_core::QuantizationMode::None => {
                chunk_searcher::storage_element_size::<M>(self.mode, dim, self.storage_f32) as u64
            }
            hyperspace_core::QuantizationMode::RaBitQ => {
                hyperspace_core::rabitq::code_size(dim) as u64
            }
            hyperspace_core::QuantizationMode::RaBitQ4 => {
                hyperspace_core::rabitq::code_size_bits(dim, 4) as u64
            }
            hyperspace_core::QuantizationMode::RaBitQ2 => {
                hyperspace_core::rabitq::code_size_bits(dim, 2) as u64
            }
            hyperspace_core::QuantizationMode::PQ4 => {
                hyperspace_core::pq4::code_size(hyperspace_core::pq4::default_num_subvectors(dim))
                    as u64
            }
            hyperspace_core::QuantizationMode::None => dim as u64 * 8,
            hyperspace_core::QuantizationMode::ScalarI8 => dim as u64,
            hyperspace_core::QuantizationMode::Binary => (dim as u64).div_ceil(8),
            hyperspace_core::QuantizationMode::AsymmetricHybrid801 => {
                if dim == 801 {
                    33 * 4 + 768
                } else {
                    dim as u64
                }
            }
            hyperspace_core::QuantizationMode::AsymmetricHybridLowBit => {
                if dim >= 33 {
                    let euc = dim as u64 - 33;
                    33 * 4 + euc.div_ceil(2) + euc.div_ceil(16) * 4
                } else {
                    dim as u64
                }
            }
            hyperspace_core::QuantizationMode::AsymmetricHybridExtreme => {
                if dim >= 33 {
                    let euc = dim as u64 - 33;
                    33 * 4 + 4 + euc.div_ceil(8)
                } else {
                    dim as u64
                }
            }
            hyperspace_core::QuantizationMode::ScalarI4 => {
                let head_dim = if dim == 801 && self.metric_name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = (dim as u64).saturating_sub(head_dim as u64);
                let num_blocks = tail_dim.div_ceil(16);
                head_dim as u64 * 4 + tail_dim.div_ceil(2) + num_blocks * 4
            }
            hyperspace_core::QuantizationMode::Turbo => {
                let head_dim = if dim == 801 && self.metric_name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = (dim as u64).saturating_sub(head_dim as u64);
                head_dim as u64 * 4 + tail_dim.div_ceil(2) + 4
            }
            hyperspace_core::QuantizationMode::Ternary => {
                let head_dim = if dim == 801 && self.metric_name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = (dim as u64).saturating_sub(head_dim as u64);
                head_dim as u64 * 4 + 2 * tail_dim.div_ceil(8) + 8
            }
            hyperspace_core::QuantizationMode::ProductQuantization
            | hyperspace_core::QuantizationMode::OPQ => {
                hyperspace_core::pq::default_num_subvectors(dim) as u64
            }
        };

        // Estimate RAM: Vectors + Graph Topology (approx M neighbors per node * 4 bytes)
        let m = self.config.get_m();
        let graph_ram = count as u64 * m as u64 * 4;
        let vector_ram = count as u64 * vector_bytes_per_doc;

        hyperspace_core::CollectionUsage {
            disk_usage_bytes: disk_usage,
            ram_usage_bytes: graph_ram + vector_ram,
            active_indexing_tasks: self.config.active_indexing.load(Ordering::Relaxed),
        }
    }

    fn update_config(&self, update: hyperspace_core::CollectionConfigUpdate) -> Result<(), String> {
        if let Some(ef_s) = update.ef_search {
            self.config.set_ef_search(ef_s);
        }
        if let Some(ef_c) = update.ef_construction {
            self.config.set_ef_construction(ef_c);
        }
        if let Some(m) = update.m {
            self.config.set_m(m);
        }
        Ok(())
    }

    fn get_hnsw_config(&self) -> hyperspace_core::HnswConfig {
        hyperspace_core::HnswConfig {
            ef_search: self.config.get_ef_search(),
            ef_construction: self.config.get_ef_construction(),
            m: self.config.get_m(),
        }
    }

    fn buckets(&self) -> Vec<u64> {
        self.buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect()
    }

    fn write_buffer_size(&self) -> usize {
        self.write_buffer.size()
    }

    fn create_snapshot(&self) -> Result<(), String> {
        let start = std::time::Instant::now();
        while self.config.get_queue_size() > 0
            || self.config.active_indexing.load(Ordering::Relaxed) > 0
            || !self.write_buffer.is_empty()
        {
            if start.elapsed().as_millis() > 3000 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let snap_path = self.data_dir.join("index.snap");
        let idx = self.index_link.load().clone();
        idx.save_snapshot(&snap_path)?;

        // Save State (DashMap iteration)
        let map_data: HashMap<u32, u32> = self
            .id_map
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect();
        let reverse_map_data: HashMap<u32, u32> = self
            .reverse_id_map
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect();
        let buckets_data: Vec<u64> = self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();

        let state = CollectionState {
            id_map: map_data,
            reverse_id_map: reverse_map_data,
            buckets: buckets_data,
            last_persisted_clock: self.last_clock.load(Ordering::Relaxed),
        };

        let state_path = self.data_dir.join("state.json");
        if let Ok(s) = serde_json::to_string(&state) {
            if let Err(e) = std::fs::write(&state_path, s) {
                return Err(e.to_string());
            }
        } else {
            return Err("Failed to serialize state.json".to_string());
        }

        if let Ok(mut wal) = self.wal_link.load().try_lock() {
            let _ = wal.sync();
        }
        Ok(())
    }

    async fn insert(
        &self,
        vector: &[f64],
        id: u32,
        metadata: HashMap<String, String>,
        clock: u64,
        durability: hyperspace_core::Durability,
    ) -> Result<(), String> {
        self.insert_internal(vector, id, metadata, clock, durability, true)
            .await
    }

    async fn insert_batch(
        &self,
        vectors: Vec<(Vec<f64>, u32, HashMap<String, String>)>,
        clock: u64,
        durability: hyperspace_core::Durability,
    ) -> Result<(), String> {
        self.insert_batch_internal(vectors, clock, durability, true)
            .await
    }

    fn delete(&self, id: u32) -> Result<(), String> {
        let internal_id = if let Some((_, internal_id)) = self.id_map.remove(&id) {
            self.reverse_id_map.remove(&internal_id);
            internal_id
        } else {
            id
        };

        self.write_buffer.remove(internal_id);

        let idx = self.index_link.load();
        if self.config.is_gossip_enabled() {
            // Defensive check: only update if ID is within bounds of active index
            if (internal_id as usize) < idx.count() {
                let vector = self.stored_vector(&idx, internal_id);
                let hash = CollectionDigest::hash_entry(id, &vector);
                let b_idx = CollectionDigest::get_bucket_index(id);

                self.buckets[b_idx].fetch_xor(hash, Ordering::Relaxed);
                self.root_hash.fetch_xor(hash, Ordering::Relaxed);
            }
        }

        idx.delete(internal_id);
        if idx.needs_repair() {
            // Graph consolidation off the request path (tombstones degrade recall/latency).
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let idx_for_repair = self.index_link.load_full();
                handle.spawn_blocking(move || {
                    let repaired = idx_for_repair.repair_deleted();
                    if repaired > 0 {
                        println!(
                            "🩹 HNSW repair: relinked {repaired} adjacency lists after deletes"
                        );
                    }
                });
            }
        }
        if self.raw_vector_cache.remove(&internal_id).is_some() {
            self.raw_vector_cache_len.fetch_sub(1, Ordering::Relaxed);
        }
        if let Some(exact) = self.exact_store.load_full() {
            exact.remove(internal_id);
        }
        if let Some(ref cache) = self.cache {
            cache.invalidate(id);
        }

        let clock = self.last_clock.fetch_add(1, Ordering::SeqCst) + 1;
        if let Ok(mut wal) = self.wal_link.load().try_lock() {
            let _ = wal.append_delete(id as u64, clock);
        } else {
            let wal_link = self.wal_link.load().clone();
            tokio::spawn(async move {
                let mut wal = wal_link.lock().await;
                let _ = wal.append_delete(id as u64, clock);
            });
        }

        Ok(())
    }

    async fn search(
        &self,
        query: &[f64],
        filters: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &SearchParams,
    ) -> Result<Vec<SearchResult>, String> {
        if query.len() < self.dimension {
            return Err(format!(
                "Query dimension mismatch. Expected {}, got {}",
                self.dimension,
                query.len()
            ));
        }
        let slice = if query.len() > self.dimension {
            &query[..self.dimension]
        } else {
            query
        };

        if let Err(e) = M::validate(slice) {
            return Err(format!(
                "Invalid query vector for metric '{}': {}",
                M::name(),
                e
            ));
        }

        // Quick Win #5: Zero-copy normalization - keep Cow until absolutely necessary
        let processed_query_cow = Self::normalize_if_cosine(slice);

        let index_link = self.index_link.clone();
        let reverse_id_map = self.reverse_id_map.clone();
        let ids_are_identity = self.ids_are_identity.load(Ordering::Acquire);

        // Move only the required fields to avoid cloning whole params struct.
        let top_k = params.top_k;
        let skip_metadata = params.skip_metadata;
        // RECALL FIX #3: Per HNSW spec, ef_search MUST be >= top_k.
        // Respect caller's explicit ef_search when > 0; otherwise fall back to collection default.
        let ef_search = if params.ef_search > 0 {
            params.ef_search.max(top_k)
        } else {
            // The collection default is a floor: the auto-tuned beam (see
            // `spawn_ef_tuner`) widens it as the graph grows.
            self.config
                .get_ef_search()
                .max(self.index_link.load().tuned_ef())
                .max(top_k)
        };
        let include_payload = params.include_payload;

        if let Some(ref cache) = self.cache {
            let known_id = filters
                .get("id")
                .or_else(|| filters.get("_id"))
                .and_then(|val| val.parse::<u32>().ok());
            let is_pure_ann =
                filters.is_empty() && complex_filters.is_empty() && params.hybrid_query.is_none();
            if known_id.is_some() || is_pure_ann {
                if let Some(hits) = cache.search(&processed_query_cow, known_id, top_k) {
                    let pre_payload: Vec<(u32, f64, HashMap<String, String>)> = hits
                        .into_iter()
                        .map(|hit| {
                            let metadata_cloned =
                                Arc::try_unwrap(hit.metadata).unwrap_or_else(|m| (*m).clone());
                            (hit.id, hit.distance, metadata_cloned)
                        })
                        .collect();

                    if include_payload {
                        let top_k_ids: Vec<u32> =
                            pre_payload.iter().map(|(id, _, _)| *id).collect();
                        let payloads =
                            PayloadStore::fetch_many(Arc::clone(&self.payload_store), &top_k_ids)
                                .await;
                        return Ok(pre_payload
                            .into_iter()
                            .zip(payloads)
                            .map(|((id, dist, meta), payload)| (id, dist, meta, payload))
                            .collect());
                    }
                    return Ok(pre_payload
                        .into_iter()
                        .map(|(id, dist, meta)| (id, dist, meta, None))
                        .collect());
                }
            }
        }
        let schema_rerank_top_k = self.schema.cascade_pipeline.iter().find_map(|layer| {
            if layer.rerank_top_k > 0 {
                Some(layer.rerank_top_k as usize)
            } else {
                None
            }
        });

        let env_rerank_enabled = env_rerank_enabled();
        let env_rerank_oversample = env_rerank_oversample();

        let (request_rerank, request_oversample, clean_filters_cow) = if filters.is_empty() {
            (None, None, None)
        } else {
            let req_rerank = filters.get("rerank").map(|v| {
                v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
            });
            let req_oversample = filters
                .get("oversample")
                .and_then(|v| v.parse::<usize>().ok());

            let clean: Option<Cow<'_, HashMap<String, String>>> =
                if req_rerank.is_none() && req_oversample.is_none() {
                    Some(Cow::Borrowed(filters))
                } else {
                    let mut f = filters.clone();
                    f.remove("rerank");
                    f.remove("oversample");
                    if f.is_empty() {
                        None
                    } else {
                        Some(Cow::Owned(f))
                    }
                };
            (req_rerank, req_oversample, clean)
        };

        let mrl_index = self.index_dim < self.dimension;
        let rerank_requested = request_rerank
            .unwrap_or(env_rerank_enabled || schema_rerank_top_k.is_some() || mrl_index)
            && params.hybrid_query.is_none();
        let rerank_oversample = request_oversample.unwrap_or(env_rerank_oversample).max(1);
        // Full-precision single-component collections already rank by the exact distance:
        // re-scoring would decode every candidate to f64 and compute the same values again.
        // Keep the oversampled beam (`top_k × oversample`) so results stay identical.
        let rerank_redundant = rerank_requested && self.rerank_is_redundant(params);
        let rerank_enabled = rerank_requested && !rerank_redundant && !params.defer_rerank;
        // Coarse codes keep the true neighbours only within a wide pool (see
        // `COARSE_RERANK_POOL`); a scatter-gather caller already widened each shard's share.
        let coarse_pool = if self.rerank_full_pool() {
            coarse_pool_factor(self.shard_share)
        } else {
            1
        };
        let ef_search = if rerank_redundant {
            let search_k = top_k
                .saturating_mul(rerank_oversample)
                .max(schema_rerank_top_k.unwrap_or(0));
            ef_search.max(search_k)
        } else {
            ef_search
        };

        let use_wasserstein = params.use_wasserstein;
        let meta_router_ref = self.meta_router.clone();
        let chunk_backend_ref = self.chunk_backend.clone();
        let mode_for_search = self.mode;
        let config_for_search = self.config.clone();
        let permit = self
            .search_limiter
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| format!("Search limiter failed: {e}"))?;

        let layout_owned = self.layout.clone();
        let component_weights_owned = params.component_weights.clone();

        // Quick Win: Only offload to spawn_blocking when cold disk chunks or exact rerank are active;
        // pure in-RAM HNSW searches run inline without thread-hop overhead even when top_k > 50.
        let use_blocking = rerank_enabled || self.meta_router.chunk_count() > 0;

        let index_link_for_cache = index_link.clone();
        if use_blocking {
            // Convert to owned only when entering blocking task
            let (query_owned, filters_owned, complex_filters_owned) = (
                processed_query_cow.into_owned(),
                clean_filters_cow.map(|c| c.into_owned()),
                (!complex_filters.is_empty()).then(|| complex_filters.to_vec()),
            );
            let write_buffer_for_search = self.write_buffer.clone();
            let index_dim = self.index_dim;
            let mut search_params_owned = params.clone();
            let payload_store_owned = Arc::clone(&self.payload_store);
            let exact_store_owned = self.exact_store.load_full();
            let raw_vector_cache_owned = Arc::clone(&self.raw_vector_cache);
            let pre_payload = tokio::task::spawn_blocking(move || {
                let _permit = permit;
                let processed_query = query_owned;
                let index = index_link.load();
                let include_metadata = index.has_nonempty_metadata() && !skip_metadata;
                let filters_ref = filters_owned
                    .as_ref()
                    .map_or(&*EMPTY_LEGACY_FILTERS, |f| f);
                let complex_filters_ref = complex_filters_owned
                    .as_ref()
                    .map_or(EMPTY_COMPLEX_FILTERS.as_slice(), Vec::as_slice);
                let search_k = if rerank_enabled {
                    let min_k = schema_rerank_top_k.unwrap_or(0);
                    top_k
                        .saturating_mul(rerank_oversample)
                        .saturating_mul(coarse_pool)
                        .max(top_k)
                        .max(min_k)
                } else {
                    top_k
                };

                search_params_owned.top_k = search_k;
                search_params_owned.ef_search = ef_search.max(search_k);
                // MRL prefix index: the graph is searched with the query prefix; chunks,
                // routing and the rerank below use the full query.
                let index_query = mrl_prefix::<M>(&processed_query, index_dim);
                if index_dim < processed_query.len() {
                    search_params_owned.mrl_dimension =
                        search_params_owned.mrl_dimension.filter(|&d| d < index_dim);
                }
                let mem_results = index.search(
                    &index_query,
                    filters_ref,
                    complex_filters_ref,
                    &search_params_owned,
                );

                // === 2. Search cold chunks via MetaRouter (disk mmap / S3 tiering) ===
                let probe_k = env_chunk_probe_k();
                let routed_chunks = meta_router_ref.route(&processed_query, probe_k);
                let mut chunk_dirs: Vec<std::path::PathBuf> = Vec::with_capacity(routed_chunks.len());
                for (chunk_id, default_path, _) in &routed_chunks {
                    match chunk_backend_ref.resolve(chunk_id) {
                        Ok(resolved) => chunk_dirs.push(resolved),
                        Err(e) => {
                            eprintln!("⚠️ Failed to resolve chunk {chunk_id} via backend: {e}. Falling back to default path.");
                            if default_path.exists() {
                                chunk_dirs.push(default_path.clone());
                            }
                        }
                    }
                }

                let chunk_results = if chunk_dirs.is_empty() {
                    Vec::new()
                } else {
                    chunk_searcher::scatter_gather_search::<M>(
                        &chunk_dirs,
                        &processed_query,
                        search_k,
                        ef_search,
                        filters_ref,
                        complex_filters_ref,
                        mode_for_search,
                        &config_for_search,
                        use_wasserstein,
                    )
                };

                // === 3. Merge MemTable + Chunk results by distance ===
                // MemTable results carry internal IDs; chunk results are already resolved to
                // global user IDs (via the chunk manifest) and carry their own metadata.
                let mut merged: Vec<(u32, f64, Option<(u32, HashMap<String, String>)>)> =
                    Vec::with_capacity(mem_results.len() + chunk_results.len());
                for (id, dist) in &mem_results {
                    merged.push((*id, *dist, None));
                }
                for hit in chunk_results {
                    merged.push((u32::MAX, hit.distance, Some((hit.user_id, hit.metadata))));
                }
                merged.sort_by(|a, b| a.1.total_cmp(&b.1));
                merged.truncate(search_k);

                // Chunk hits keep their in-chunk distance (their exact vectors live in the
                // immutable chunk, not in this MemTable's rerank sources).
                let mut chunk_hits: Vec<(u32, f64, HashMap<String, String>)> = Vec::new();
                let mut results: Vec<(u32, f64)> = Vec::with_capacity(merged.len());
                for (internal_id, dist, chunk) in merged {
                    match chunk {
                        None => results.push((internal_id, dist)),
                        Some((user_id, meta)) => chunk_hits.push((user_id, dist, meta)),
                    }
                }

                let metric_tag = match M::name() {
                    "cosine" => GpuMetric::Cosine,
                    "poincare" => GpuMetric::Poincare,
                    "lorentz" => GpuMetric::Lorentz,
                    _ => GpuMetric::L2,
                };

                let reranked_internal: Vec<(u32, f64)> = if rerank_enabled && !results.is_empty() {
                    let q32_for_rerank: Vec<f32> = processed_query.iter().map(|&x| x as f32).collect();
                    let l2_like = matches!(M::name(), "l2" | "cosine");
                    let with_exact_coords = |id: u32, dst: &mut [f64], f: &mut dyn FnMut(&[f64]) -> f64| -> f64 {
                        if let Some(exact) = &exact_store_owned {
                            if exact.read_f64(id, dst) {
                                let len = dst.len().min(exact.dimension());
                                return f(&dst[..len]);
                            }
                        }
                        if let Some(v_f32) = raw_vector_cache_owned.get(&id) {
                            let len = dst.len().min(v_f32.len());
                            for (d, &src) in dst[..len].iter_mut().zip(v_f32[..len].iter()) {
                                *d = f64::from(src);
                            }
                            return f(&dst[..len]);
                        }
                        if let Ok(Some(bytes)) = payload_store_owned.fetch_blocking(id) {
                            if bytes.len() == processed_query.len() * 4 {
                                for (i, chunk) in bytes.as_chunks::<4>().0.iter().enumerate() {
                                    if i < dst.len() {
                                        dst[i] = f64::from(f32::from_le_bytes(*chunk));
                                    }
                                }
                                return f(dst);
                            } else if bytes.len() == processed_query.len() * 8 {
                                for (i, chunk) in bytes.as_chunks::<8>().0.iter().enumerate() {
                                    if i < dst.len() {
                                        dst[i] = f64::from_le_bytes(*chunk);
                                    }
                                }
                                return f(dst);
                            }
                        }
                        let orig = index.get_vector_original(id);
                        let len = dst.len().min(orig.coords.len());
                        dst[..len].copy_from_slice(&orig.coords[..len]);
                        f(&dst[..len])
                    };

                    if layout_owned.components.len() > 1 {
                        let candidate_ids: Vec<u32> = results.iter().map(|(id, _)| *id).collect();
                        let q_len = processed_query.len();
                        let mut flat_vectors = vec![0.0; candidate_ids.len() * q_len];
                        for (idx, id) in candidate_ids.iter().enumerate() {
                            let slot = &mut flat_vectors[idx * q_len..(idx + 1) * q_len];
                            with_exact_coords(*id, slot, &mut |_| 0.0);
                        }
                        let candidate_refs: Vec<&[f64]> =
                            flat_vectors.chunks_exact(q_len).collect();
                        rerank_topk_exact(
                            metric_tag,
                            &processed_query,
                            &candidate_ids,
                            &candidate_refs,
                            Some(&layout_owned),
                            component_weights_owned.as_ref(),
                        )
                    } else {
                        let mut exact_scratch = vec![0.0; processed_query.len()];
                        // Wide pools (coarse codes) on an idle server: score the candidates
                        // that have an exact copy on idle cores, the rest below.
                        let parts = hyperspace_index::parallel_parts(results.len(), 128);
                        let pre: Option<Vec<f64>> = (l2_like && parts > 1)
                            .then_some(exact_store_owned.as_ref())
                            .flatten()
                            .map(|ex| {
                                use rayon::prelude::*;
                                results
                                    .par_chunks(results.len().div_ceil(parts))
                                    .flat_map_iter(|c| {
                                        c.iter().map(|(id, _)| {
                                            ex.get_bytes(*id).map_or(f64::NAN, |b| {
                                                f64::from(hyperspace_core::simd::l2_sq_f32_bytes(
                                                    b,
                                                    &q32_for_rerank,
                                                    None,
                                                ))
                                            })
                                        })
                                    })
                                    .collect()
                            });
                        let mut exact_scores: Vec<(u32, f64)> = results
                            .iter()
                            .enumerate()
                            .map(|(i, r)| (r, pre.as_ref().map_or(f64::NAN, |p| p[i])))
                            .map(|((id, _), known)| {
                                if !known.is_nan() {
                                    return (*id, known);
                                }
                                if l2_like {
                                    if let Some(bytes) = exact_store_owned.as_ref().and_then(|e| e.get_bytes(*id)) {
                                        return (
                                            *id,
                                            f64::from(hyperspace_core::simd::l2_sq_f32_bytes(bytes, &q32_for_rerank, None)),
                                        );
                                    }
                                }
                                let exact_d = with_exact_coords(*id, &mut exact_scratch, &mut |coords| {
                                    M::distance(coords, &processed_query)
                                });
                                (*id, exact_d)
                            })
                            .collect();
                        exact_scores.sort_by(|a, b| a.1.total_cmp(&b.1));
                        exact_scores
                    }
                } else {
                    results
                };

                // Metadata / user ids inside the blocking worker; payloads after it (lazy I/O).
                finish_hits(
                    &index,
                    reranked_internal,
                    chunk_hits,
                    include_metadata,
                    ids_are_identity,
                    &reverse_id_map,
                    &write_buffer_for_search,
                    (&processed_query, search_k, filters_ref, complex_filters_ref),
                    top_k,
                )
            })
            .await
            .map_err(|e| format!("Search task failed: {e}"))?;

            // === Step 3 (Read-Through): Populate cache with HNSW results =========
            // When enabled, top-K results from every disk HNSW search are inserted
            // into the L1 cache so that future queries for the same or nearby vectors
            // become L1 hits. Controlled by HYPERSPACE_CACHE_READ_THROUGH (default: true).
            let read_through_enabled = env_cache_read_through();
            if read_through_enabled {
                if let Some(ref cache) = self.cache {
                    let index_snap = index_link_for_cache.load_full();
                    for (user_id, _dist, meta) in &pre_payload {
                        // Look up the internal id to fetch raw vector coords.
                        let internal_id_opt = if ids_are_identity {
                            Some(*user_id)
                        } else {
                            // reverse_id_map maps internal -> external; we need external -> internal
                            // Iterate the index to find it (best-effort, zero-copy on cache hit).
                            // For identity IDs this is a no-op branch.
                            None
                        };
                        if let Some(internal_id) = internal_id_opt {
                            if (internal_id as usize) < index_snap.count() {
                                let hv = index_snap.get_vector_original(internal_id);
                                cache.insert(*user_id, hv.coords.clone(), meta.clone(), None);
                            }
                        }
                    }
                }
            }

            // === Step 4: Lazy Payload Fetch (v3.2) ==============================
            // Only executed when include_payload=true AND after Top-K is decided.
            // Zero disk I/O on the default (include_payload=false) path.
            if include_payload {
                let top_k_ids: Vec<u32> = pre_payload.iter().map(|(id, _, _)| *id).collect();
                let payloads =
                    PayloadStore::fetch_many(Arc::clone(&self.payload_store), &top_k_ids).await;

                Ok(pre_payload
                    .into_iter()
                    .zip(payloads)
                    .map(|((id, dist, meta), payload)| (id, dist, meta, payload))
                    .collect())
            } else {
                Ok(pre_payload
                    .into_iter()
                    .map(|(id, dist, meta)| (id, dist, meta, None))
                    .collect())
            }
        } else {
            // Inline search for in-RAM HNSW - avoid spawn_blocking and query Vec cloning overhead
            let processed_query: &[f64] = &processed_query_cow;
            let _permit = permit;
            let index = index_link.load();
            let include_metadata = index.has_nonempty_metadata() && !skip_metadata;
            let filters_ref = clean_filters_cow
                .as_deref()
                .unwrap_or(&*EMPTY_LEGACY_FILTERS);
            let complex_filters_ref = complex_filters;

            // === 1. Search the hot MemTable (in-RAM HNSW) ===
            // Apply the resolved ef_search (collection default / >= top_k) on the inline path too.
            let effective_params: Cow<'_, SearchParams> = if params.ef_search == ef_search {
                Cow::Borrowed(params)
            } else {
                let mut p = params.clone();
                p.ef_search = ef_search;
                Cow::Owned(p)
            };
            let index_query = self.index_view(processed_query);
            let mem_results = index.search(
                &index_query,
                filters_ref,
                complex_filters_ref,
                &effective_params,
            );

            // === 2. Search cold chunks (none when use_blocking is false) ===

            // === 3. Convert results & merge write buffer (if non-empty) ===
            let raw_results = finish_hits(
                &index,
                mem_results,
                Vec::new(),
                include_metadata,
                ids_are_identity,
                &reverse_id_map,
                &self.write_buffer,
                (processed_query, top_k, filters_ref, complex_filters_ref),
                top_k,
            );

            // === Step 3: Lazy Payload Fetch (v3.2) — inline path ================
            if include_payload {
                let ids: Vec<u32> = raw_results.iter().map(|(id, _, _)| *id).collect();
                let payloads =
                    PayloadStore::fetch_many(Arc::clone(&self.payload_store), &ids).await;
                let results: Vec<SearchResult> = raw_results
                    .into_iter()
                    .zip(payloads)
                    .map(|((id, dist, meta), payload)| (id, dist, meta, payload))
                    .collect();
                Ok(results)
            } else {
                let results: Vec<SearchResult> = raw_results
                    .into_iter()
                    .map(|(id, dist, meta)| (id, dist, meta, None))
                    .collect();
                Ok(results)
            }
        }
    }

    async fn optimize(&self) -> Result<(), String> {
        self.optimize_with_filter(None).await
    }

    async fn optimize_with_filter(&self, filter: Option<VacuumFilterQuery>) -> Result<(), String> {
        println!("🧹 Starting Hot Vacuum for '{}'...", self.name);
        let start = std::time::Instant::now();
        let data_dir = self.data_dir.clone();
        let mode = self.mode;
        let original_config = self.config.clone();
        let index_link = self.index_link.clone();
        let raw_vector_cache = Arc::clone(&self.raw_vector_cache);
        let exact_store = self.exact_store.load_full();
        let payload_store = Arc::clone(&self.payload_store);
        let full_dimension = self.dimension;

        // Heavy lifting off the async runtime. Vectors are streamed one at a time from
        // their exact source into the new index (no all-vectors copy), graph built with
        // the collection's M in parallel.
        let built =
            tokio::task::spawn_blocking(move || -> Result<Option<RebuildOutput>, String> {
                use hyperspace_core::config::GlobalConfig;
                use hyperspace_store::VectorStore;

                let current_index = index_link.load().clone();
                // Index width (MRL prefix or full); exact vectors are `full_dimension` wide.
                let dimension = current_index.dimension;
                let mrl = dimension < full_dimension;
                let max_len = current_index.count_nodes();
                let mut live: Vec<u32> = {
                    let deleted = current_index.metadata.deleted.read();
                    (0..max_len as u32)
                        .filter(|id| !deleted.contains(*id))
                        .collect()
                };
                if let Some(filter) = &filter {
                    live.retain(|id| {
                        !Self::matches_vacuum_filter(&current_index.metadata_by_id(*id), filter)
                    });
                }
                if live.is_empty() {
                    return Ok(None);
                }
                // Id order. Inserting in BFS order of the old graph (cluster by cluster) was
                // measured to degrade the new graph: recall 99.67% → 98.70%, QPS −34%.
                let order = live;

                let exact_of = |old: u32| -> Vec<f64> {
                    if let Some(v) = exact_store.as_ref().and_then(|e| e.get(old)) {
                        return v.into_iter().map(f64::from).collect();
                    }
                    if let Some(v) = raw_vector_cache.get(&old) {
                        return v.iter().map(|&x| f64::from(x)).collect();
                    }
                    if mode != hyperspace_core::QuantizationMode::None {
                        // Pre-4.2.5 collections kept exact copies in the payload store.
                        if let Ok(Some(bytes)) = payload_store.fetch_blocking(old) {
                            if bytes.len() == full_dimension * 4 {
                                return bytes
                                    .as_chunks::<4>()
                                    .0
                                    .iter()
                                    .map(|c| f64::from(f32::from_le_bytes(*c)))
                                    .collect();
                            }
                        }
                    }
                    current_index.get_vector_original(old).coords
                };

                // The collection's own M (a wider graph doubles layer-0 memory and slows
                // every later search); a deeper build beam is cheap and improves recall.
                let env_usize = |k: &str| {
                    std::env::var(k)
                        .ok()
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .filter(|&v| v > 0)
                };
                let vacuum_m = env_usize("HS_REBUILD_M").unwrap_or_else(|| original_config.get_m());
                let vacuum_ef = env_usize("HS_REBUILD_EF_CONSTRUCT")
                    .unwrap_or_else(|| original_config.get_ef_construction().max(200));
                let vacuum_config = Arc::new(GlobalConfig::new());
                vacuum_config.set_m(vacuum_m);
                vacuum_config.set_ef_construction(vacuum_ef);
                vacuum_config.set_ef_search(original_config.get_ef_search());
                println!(
                    "   Building Shadow Index over {} vectors (M={vacuum_m}, EF={vacuum_ef})...",
                    order.len()
                );

                let temp_dir = data_dir.join(format!("idx_opt_{}", uuid::Uuid::new_v4()));
                std::fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
                let storage_f32 = chunk_searcher::storage_f32_enabled(mode);
                let element_size =
                    chunk_searcher::storage_element_size::<M>(mode, dimension, storage_f32);
                let temp_store = Arc::new(VectorStore::new(&temp_dir, element_size));
                let mut new_index = HnswIndex::<M>::new_with_storage_precision(
                    temp_store,
                    mode,
                    vacuum_config.clone(),
                    storage_f32,
                    dimension,
                );

                // Code-based quantizers are calibrated on exact vectors first.
                if mode.needs_training() {
                    let sample_limit = 10_000.min(order.len());
                    let step = (order.len() / sample_limit).max(1);
                    let samples: Vec<Vec<f32>> = order
                        .iter()
                        .step_by(step)
                        .take(sample_limit)
                        .map(|&old| {
                            mrl_prefix::<M>(&exact_of(old), dimension)
                                .iter()
                                .map(|&x| x as f32)
                                .collect()
                        })
                        .collect();
                    let refs: Vec<&[f32]> = samples.iter().map(Vec::as_slice).collect();
                    println!(
                        "   🎓 Calibrating {mode:?} quantizer on {} vectors...",
                        refs.len()
                    );
                    new_index.train_pq(&refs, 10);
                }

                let new_exact = (mode != hyperspace_core::QuantizationMode::None || mrl)
                    .then(|| ExactVectorStore::open(&temp_dir, full_dimension));
                let mut metas = Vec::with_capacity(order.len());
                for (pos, &old) in order.iter().enumerate() {
                    let v = exact_of(old);
                    let new_id = new_index.insert_to_storage(&mrl_prefix::<M>(&v, dimension))?;
                    if new_id as usize != pos {
                        return Err(format!("rebuild: unexpected id {new_id} at position {pos}"));
                    }
                    if let Some(e) = &new_exact {
                        let v32: Vec<f32> = v.iter().map(|&x| x as f32).collect();
                        e.put(new_id, &v32)?;
                    }
                    metas.push(current_index.metadata_by_id(old));
                }
                // Graph construction in parallel (`index_node` is safe to call concurrently).
                let next = std::sync::atomic::AtomicUsize::new(0);
                let metas = &metas;
                let index_ref = &new_index;
                let threads =
                    std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
                let failed = std::sync::atomic::AtomicBool::new(false);
                std::thread::scope(|s| {
                    for _ in 0..threads {
                        s.spawn(|| loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            if i >= metas.len() {
                                break;
                            }
                            if index_ref.index_node(i as u32, metas[i].clone()).is_err() {
                                failed.store(true, Ordering::Relaxed);
                            }
                        });
                    }
                });
                if failed.load(Ordering::Relaxed) {
                    return Err("rebuild: graph construction failed".to_string());
                }

                let new_snap_path = data_dir.join("index.snap.new");
                new_index.save_snapshot(&new_snap_path)?;
                drop(new_index);
                Ok(Some(RebuildOutput {
                    element_size,
                    storage_f32,
                    dimension,
                    full_dimension,
                    vacuum_config,
                    temp_dir,
                    new_snap_path,
                    order,
                    old_len: max_len,
                    has_exact: new_exact.is_some(),
                }))
            })
            .await
            .map_err(|e| e.to_string())??;

        let Some(out) = built else {
            return Ok(());
        };

        // Replace the vector store chunks on disk.
        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("chunk_") && name.ends_with(".hyp") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        if let Ok(entries) = std::fs::read_dir(&out.temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("chunk_") && name.ends_with(".hyp") {
                        let _ = std::fs::rename(&path, self.data_dir.join(name));
                    }
                }
            }
        }
        let snap_path = self.data_dir.join("index.snap");
        std::fs::rename(&out.new_snap_path, &snap_path).map_err(|e| e.to_string())?;
        // (`save_snapshot` already wrote the quantizer markers next to `index.snap.new`.)

        let final_store = Arc::new(VectorStore::new(&self.data_dir, out.element_size));
        let final_index = HnswIndex::<M>::load_snapshot_with_storage_precision(
            &snap_path,
            final_store,
            self.mode,
            out.vacuum_config,
            out.storage_f32,
            out.dimension,
        )
        .map_err(|e| format!("Failed to load rebuilt index: {e}"))?;

        // New internal ids: position in `order`. Remap every structure keyed by them.
        let identity = self.ids_are_identity.load(Ordering::Acquire);
        let user_of = |old: u32| -> u32 {
            if identity {
                old
            } else {
                self.reverse_id_map.get(&old).map_or(old, |v| *v)
            }
        };
        let survivors: Vec<u32> = out.order.iter().map(|&old| user_of(old)).collect();
        let mut new_of_old = vec![u32::MAX; out.old_len];
        for (new_id, &old) in out.order.iter().enumerate() {
            new_of_old[old as usize] = new_id as u32;
        }
        let cached: Vec<(u32, Vec<f32>)> = self
            .raw_vector_cache
            .iter()
            .filter_map(|e| {
                let new_id = *new_of_old.get(*e.key() as usize)?;
                (new_id != u32::MAX).then(|| (new_id, e.value().clone()))
            })
            .collect();

        // The rebuilt graph starts untuned: the background tuner re-measures its beam
        // (a beam carried over from the old graph measured 98.2% instead of 99.6% recall).
        println!("\u{1f504} Swapping indexes in memory...");
        self.index_link.store(Arc::new(final_index));
        self.id_map.clear();
        self.reverse_id_map.clear();
        let mut still_identity = true;
        for (new_id, &user) in survivors.iter().enumerate() {
            let new_id = new_id as u32;
            still_identity &= user == new_id;
            self.id_map.insert(user, new_id);
            self.reverse_id_map.insert(new_id, user);
        }
        self.ids_are_identity
            .store(still_identity, Ordering::Release);
        self.raw_vector_cache.clear();
        self.raw_vector_cache_len
            .store(cached.len(), Ordering::Relaxed);
        for (k, v) in cached {
            self.raw_vector_cache.insert(k, v);
        }
        if out.has_exact {
            let live_dir = self.data_dir.join(ExactVectorStore::DIR);
            let _ = std::fs::remove_dir_all(&live_dir);
            std::fs::rename(out.temp_dir.join(ExactVectorStore::DIR), &live_dir)
                .map_err(|e| format!("Failed to install the exact vector store: {e}"))?;
            self.exact_store.store(Some(Arc::new(ExactVectorStore::open(
                &self.data_dir,
                out.full_dimension,
            ))));
        }
        let _ = std::fs::remove_dir_all(&out.temp_dir);
        if let Some(cache) = &self.cache {
            for &user in &survivors {
                cache.invalidate(user);
            }
        }

        // Payload compaction: user payloads are keyed by *user* id. Exact vectors that
        // pre-4.2.5 collections kept here (by internal id) now live in the exact store.
        let old_payload_store = Arc::clone(&self.payload_store);
        let new_payload_dir = self
            .data_dir
            .join(format!("payload_compact_{}", uuid::Uuid::new_v4()));
        let _ = std::fs::create_dir_all(&new_payload_dir);
        match PayloadStore::open(&new_payload_dir, old_payload_store.zstd_level()) {
            Ok(new_payload_store) => {
                for &user in &survivors {
                    let _ = new_payload_store.compact_copy_from(&old_payload_store, user);
                }
                drop(new_payload_store);
                for file in ["payloads.hyp", "payload_index.hyp"] {
                    let src = new_payload_dir.join(file);
                    if src.exists() {
                        let dst = self.data_dir.join(file);
                        let _ = std::fs::remove_file(&dst);
                        let _ = std::fs::rename(&src, &dst);
                    }
                }
                println!(
                    "\u{2728} Payload Store compacted: {} surviving vectors.",
                    survivors.len()
                );
            }
            Err(e) => {
                eprintln!("\u{26a0}\u{fe0f}  Payload Store compaction failed: {e}. Original store unchanged.");
            }
        }
        let _ = std::fs::remove_dir_all(&new_payload_dir);

        // Persist the renumbered id maps right away: a restart must not pair the new
        // snapshot with the old mapping.
        self.create_snapshot()?;
        println!(
            "\u{2728} Vacuum Complete in {:?}. Recall upgraded.",
            start.elapsed()
        );
        Ok(())
    }

    fn count(&self) -> usize {
        let index = self.index_link.load();
        let deleted = index.metadata.deleted.read().len() as usize;
        let mem_count = index.count_nodes().saturating_sub(deleted);
        let chunk_count = self.meta_router.total_vector_count();
        let purgatory_count = self.flushing_vector_count.load(Ordering::Relaxed);
        mem_count + chunk_count + purgatory_count
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn index_dimension(&self) -> usize {
        self.index_dim
    }

    // Updated peek to use index_link
    /// Peek/scroll return the stored (exact) vector, never the quantized index code: with
    /// `extreme`/binary quantization the index only keeps a 1-bit code per dimension.
    fn peek(&self, limit: usize, offset: usize) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let items = self.index_link.load().peek(limit, offset);
        items
            .into_iter()
            .map(|(internal_id, vec, meta)| {
                let user_id = self
                    .reverse_id_map
                    .get(&internal_id)
                    .map_or(internal_id, |v| *v);
                let coords = self.stored_coords(internal_id).unwrap_or(vec);
                (user_id, coords, meta)
            })
            .collect()
    }

    fn get_points(&self, ids: &[u32]) -> Vec<(u32, Vec<f64>, HashMap<String, String>)> {
        let mut results = Vec::with_capacity(ids.len());
        let idx = self.index_link.load();
        for &id in ids {
            if let Some((vec, meta)) = self.write_buffer.get_by_user_id(id) {
                results.push((id, vec, meta));
                continue;
            }
            if self.id_map.contains_key(&id) {
                let internal_id = self.to_internal_id(id);
                let coords = self
                    .stored_coords(internal_id)
                    .unwrap_or_else(|| idx.get_vector(internal_id).coords);
                if !coords.is_empty() {
                    let meta = self.metadata_by_id(id);
                    results.push((id, coords, meta));
                }
            }
        }
        results
    }

    fn needs_calibration(&self) -> bool {
        self.index_link.load().needs_calibration()
    }

    fn queue_size(&self) -> u64 {
        // Return only the active indexing queue size.
        // WAL pending and flushing counts are internal LSM state and don't
        // necessarily indicate "pending indexing" from a user/search perspective.
        // A due beam re-measurement counts as pending work, so callers that wait for the
        // queue to drain (benchmarks, readiness checks) search with the tuned beam.
        self.config.get_queue_size() + u64::from(ef_tuning_due(&self.index_link.load()))
    }

    fn graph_neighbors(&self, id: u32, layer: usize, limit: usize) -> Result<Vec<u32>, String> {
        if self.explicit_graph.config.enabled && self.explicit_graph.dynamic.total_edges() > 0 {
            let internal_id = self.to_internal_id(id);
            let edges = self.explicit_graph.edges_for(internal_id);
            if !edges.is_empty() {
                return Ok(edges
                    .into_iter()
                    .take(limit)
                    .map(|e| self.to_user_id(e.target))
                    .collect());
            }
        }
        let internal_id = self.to_internal_id(id);
        let neighbors = self
            .index_link
            .load()
            .graph_neighbors(internal_id, layer, limit)?;
        Ok(neighbors.into_iter().map(|n| self.to_user_id(n)).collect())
    }

    fn graph_neighbor_distances(
        &self,
        source_id: u32,
        neighbor_ids: &[u32],
    ) -> Result<Vec<f64>, String> {
        let idx = self.index_link.load();
        let source_internal_id = self.to_internal_id(source_id);
        let source = self.stored_vector(&idx, source_internal_id);
        let distances = neighbor_ids
            .iter()
            .map(|neighbor_id| {
                let n_internal = self.to_internal_id(*neighbor_id);
                let n_vec = self.stored_vector(&idx, n_internal);
                M::distance(&source, &n_vec)
            })
            .collect();
        Ok(distances)
    }

    fn graph_traverse(
        &self,
        start_id: u32,
        layer: usize,
        max_depth: usize,
        max_nodes: usize,
        breadth_limit: usize,
    ) -> Result<Vec<u32>, String> {
        if self.explicit_graph.config.enabled && self.explicit_graph.dynamic.total_edges() > 0 {
            let internal_start = self.to_internal_id(start_id);
            let mask =
                self.explicit_graph
                    .get_reachability_mask(&[internal_start], max_depth, None);
            let res: Vec<u32> = mask
                .iter()
                .take(max_nodes)
                .map(|n| self.to_user_id(n))
                .collect();
            if !res.is_empty() {
                return Ok(res);
            }
        }
        let internal_start = self.to_internal_id(start_id);
        let traversed = self.index_link.load().graph_traverse(
            internal_start,
            layer,
            max_depth,
            max_nodes,
            breadth_limit,
        )?;
        Ok(traversed.into_iter().map(|n| self.to_user_id(n)).collect())
    }

    fn graph_clusters(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Result<Vec<Vec<u32>>, String> {
        let points = self.peek(max_nodes.max(1), 0);
        if points.len() >= min_cluster_size.max(2) {
            let vectors: Vec<Vec<f64>> = points.iter().map(|(_, v, _)| v.clone()).collect();
            let user_ids: Vec<u32> = points.iter().map(|(id, _, _)| *id).collect();

            let max_leaf_size = (points.len() / max_clusters.max(1))
                .max(min_cluster_size)
                .max(2);
            if let Some(clustered) =
                hyperspace_clustering::hierarchical_cluster_f64(&vectors, max_leaf_size)
            {
                let mut res: Vec<Vec<u32>> = Vec::new();
                for (_centroid, member_indices) in clustered {
                    if member_indices.len() >= min_cluster_size {
                        let members: Vec<u32> = member_indices
                            .into_iter()
                            .map(|idx| user_ids[idx])
                            .collect();
                        res.push(members);
                    }
                }
                if !res.is_empty() {
                    res.sort_by_key(|a| std::cmp::Reverse(a.len()));
                    res.truncate(max_clusters);
                    return Ok(res);
                }
            }
        }

        let clusters = self.index_link.load().graph_connected_components(
            layer,
            min_cluster_size,
            max_clusters,
            max_nodes,
        );
        Ok(clusters
            .into_iter()
            .map(|c| c.into_iter().map(|n| self.to_user_id(n)).collect())
            .collect())
    }

    fn graph_link(
        &self,
        source_id: u32,
        target_id: u32,
        relation: u16,
        weight: f32,
    ) -> Result<(), String> {
        let internal_src = self.to_internal_id(source_id);
        let internal_tgt = self.to_internal_id(target_id);
        self.explicit_graph
            .insert_edge(internal_src, internal_tgt, relation, weight);
        Ok(())
    }

    fn graph_link_batch(&self, edges: &[(u32, u32, u16, f32)]) -> Result<usize, String> {
        for &(src, tgt, rel, w) in edges {
            let internal_src = self.to_internal_id(src);
            let internal_tgt = self.to_internal_id(tgt);
            self.explicit_graph
                .insert_edge(internal_src, internal_tgt, rel, w);
        }
        Ok(edges.len())
    }

    fn graph_unlink(&self, source_id: u32, target_id: u32, relation: u16) -> Result<bool, String> {
        let internal_src = self.to_internal_id(source_id);
        let internal_tgt = self.to_internal_id(target_id);
        Ok(self
            .explicit_graph
            .delete_edge(internal_src, internal_tgt, relation))
    }

    fn graph_induce_from_metadata(
        &self,
        rules: &[hyperspace_core::graph::MetadataInductionRule],
    ) -> Result<usize, String> {
        let count = self.count().max(1);
        let points = self.peek(count, 0);
        let nodes: Vec<(u32, HashMap<String, String>)> = points
            .into_iter()
            .map(|(id, _, meta)| (self.to_internal_id(id), meta))
            .collect();
        let added = self.explicit_graph.induce_from_metadata(&nodes, rules);
        Ok(added)
    }

    fn compact_semantic_chunks(&self, target_partitions: usize) -> Result<String, String> {
        // Gather the full dataset: every flushed chunk + the hot MemTable.
        // MemTable entries are newer and win on user-ID collisions. Chunks that cannot be
        // exported (legacy, no manifest) are kept as-is instead of being deleted.
        let dimension = self.dimension;
        let old_chunks = self.meta_router.all_chunks();
        let mut by_user: HashMap<u32, (Vec<f64>, HashMap<String, String>)> = HashMap::new();
        let mut rewritten_chunks = Vec::new();
        let mut retained_chunks = Vec::new();
        for chunk in old_chunks {
            match chunk_searcher::export_chunk_points::<M>(
                &chunk.path,
                self.mode,
                &self.config,
                dimension,
            ) {
                Ok(Some(points)) => {
                    for (user_id, vec, meta) in points {
                        if vec.len() == dimension {
                            by_user.insert(user_id, (vec, meta));
                        }
                    }
                    rewritten_chunks.push(chunk);
                }
                Ok(None) => retained_chunks.push(chunk),
                Err(e) => {
                    return Err(format!(
                        "Semantic compaction aborted: failed to read chunk {}: {e}",
                        chunk.chunk_id
                    ))
                }
            }
        }

        let count = self.count().max(1);
        for (user_id, vec, meta) in self.peek(count, 0) {
            by_user.insert(user_id, (vec, meta));
        }

        if by_user.len() < 16 {
            return Ok(format!(
                "Not enough vectors ({}) for semantic compaction (minimum 16 required)",
                by_user.len()
            ));
        }
        let mut points: Vec<(u32, Vec<f64>, HashMap<String, String>)> = by_user
            .into_iter()
            .map(|(id, (vec, meta))| (id, vec, meta))
            .collect();
        points.sort_unstable_by_key(|(id, _, _)| *id);

        let k = target_partitions.clamp(2, 64);
        // New chunks are fully written (manifest + snapshot) before anything is removed.
        let (report, mut new_chunks) =
            crate::semantic_compactor::SemanticCompactor::compact_vectors::<M>(
                &points,
                k,
                &self.data_dir,
                self.mode,
                &self.config,
                self.storage_f32,
            )?;

        new_chunks.extend(retained_chunks);
        self.meta_router.replace_all(new_chunks);

        for old_chunk in rewritten_chunks {
            chunk_searcher::invalidate_chunk_cache(&old_chunk.path);
            if old_chunk.path.exists() && old_chunk.path.starts_with(&self.data_dir) {
                let _ = std::fs::remove_dir_all(&old_chunk.path);
            }
        }

        Ok(format!(
            "Semantic compaction completed: {} vectors repartitioned into {} Voronoi chunks in {} ms (clustering: {} ms)",
            report.total_vectors, report.compacted_chunks, report.total_time_ms, report.clustering_time_ms
        ))
    }

    fn evaluate_trust_score(&self, trajectory_ids: &[u32]) -> Result<f64, String> {
        if trajectory_ids.len() < 3 {
            return Err("Trust scoring requires a trajectory of at least 3 nodes".into());
        }

        let mut vectors = Vec::new();
        for &id in trajectory_ids {
            let pts = self.get_points(&[id]);
            if pts.is_empty() {
                return Err(format!("Node {id} not found"));
            }
            vectors.push(pts[0].1.clone());
        }

        let attractor = &vectors[vectors.len() - 1];
        let mut v_diff_sum = 0.0;

        for i in 0..vectors.len() - 1 {
            let v_t0 = M::distance(&vectors[i], attractor);
            let v_t1 = M::distance(&vectors[i + 1], attractor);
            v_diff_sum += v_t1 - v_t0;
        }

        let avg_change = v_diff_sum / (vectors.len() - 1) as f64;

        // Negative average change indicates convergence to the target.
        let score = 1.0 / (1.0 + avg_change.max(0.0).exp());
        Ok(score)
    }

    fn metadata_by_id(&self, id: u32) -> HashMap<String, String> {
        if let Some((_, wb_meta)) = self.write_buffer.get_by_user_id(id) {
            return wb_meta;
        }
        let internal_id = self.to_internal_id(id);
        self.index_link.load().metadata_by_id(internal_id)
    }

    async fn update_payload(
        &self,
        id: u32,
        patch: std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        let clock = self.last_clock.fetch_add(1, Ordering::SeqCst) + 1;
        self.update_payload_internal(id, patch, clock, true).await
    }

    /// Write a heavy payload blob to the disk-only Payload Layer.
    /// Called AFTER a successful insert(). The blob is zstd-compressed before
    /// touching disk. ZERO bytes of payload content are kept in RAM after return.
    async fn insert_payload(&self, id: u32, payload: Vec<u8>) -> Result<(), String> {
        let store = Arc::clone(&self.payload_store);
        tokio::task::spawn_blocking(move || {
            store
                .insert(id, &payload)
                .map_err(|e| format!("PayloadStore insert error: {e}"))
        })
        .await
        .map_err(|e| format!("spawn_blocking panicked: {e}"))?
    }

    /// Write batch of heavy payload blobs to the disk-only Payload Layer.
    async fn insert_payload_batch(&self, entries: Vec<(u32, Vec<u8>)>) -> Result<(), String> {
        let store = Arc::clone(&self.payload_store);
        tokio::task::spawn_blocking(move || {
            store
                .insert_batch(&entries)
                .map_err(|e| format!("PayloadStore insert_batch error: {e}"))
        })
        .await
        .map_err(|e| format!("spawn_blocking panicked: {e}"))?
    }

    /// Fetch decompressed payloads for final Top-K IDs via lazy disk I/O.
    /// Each ID is read concurrently on the blocking thread pool. Zero disk I/O
    /// for IDs with no stored payload (returns None at that index).
    async fn fetch_payloads(&self, ids: &[u32]) -> Vec<Option<Vec<u8>>> {
        PayloadStore::fetch_many(Arc::clone(&self.payload_store), ids).await
    }

    fn cache_stats(&self) -> Result<String, String> {
        if let Some(ref cache) = self.cache {
            let stats = cache.stats();
            serde_json::to_string(&stats).map_err(|e| e.to_string())
        } else {
            Err("Cache not enabled".to_string())
        }
    }

    fn cache_clear(&self) -> Result<(), String> {
        if let Some(ref cache) = self.cache {
            cache.clear();
            Ok(())
        } else {
            Err("Cache not enabled".to_string())
        }
    }

    fn cache_update_config(
        &self,
        policy: String,
        ann_threshold: Option<f64>,
    ) -> Result<(), String> {
        if let Some(ref cache) = self.cache {
            let policy_parsed = match policy.to_lowercase().as_str() {
                "lru" => hyperspace_cache::EvictionPolicy::Lru,
                "lfu" => hyperspace_cache::EvictionPolicy::Lfu,
                "ttl" => hyperspace_cache::EvictionPolicy::Ttl,
                _ => return Err("Invalid policy".to_string()),
            };
            cache.update_config(policy_parsed, ann_threshold);
            Ok(())
        } else {
            Err("Cache not enabled".to_string())
        }
    }
}

impl<M: Metric> Drop for CollectionImpl<M> {
    fn drop(&mut self) {
        println!(
            "🗑️ Dropping collection '{}'. Stopping background tasks...",
            self.name
        );
        // Abort background tasks (indexer, snapshot)
        for task in &self.bg_tasks {
            task.abort();
        }
        let wal_arc = self.wal_link.load_full();
        if let Ok(mut wal) = wal_arc.try_lock() {
            let _ = wal.sync();
        };
    }
}

impl<M: Metric> CollectionImpl<M> {
    /// The exact vector as inserted (raw cache, then the exact store); `None` when only the
    /// index representation exists (non-quantized collections, where it is already exact).
    fn stored_coords(&self, internal_id: u32) -> Option<Vec<f64>> {
        if let Some(v) = self.raw_vector_cache.get(&internal_id) {
            return Some(v.iter().map(|&x| f64::from(x)).collect());
        }
        self.exact_store
            .load_full()
            .and_then(|e| e.get(internal_id))
            .map(|v| v.into_iter().map(f64::from).collect())
    }

    #[inline]
    pub fn last_clock(&self) -> u64 {
        self.last_clock.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn update_payload_internal(
        &self,
        id: u32,
        patch: HashMap<String, String>,
        clock: u64,
        write_wal: bool,
    ) -> Result<(), String> {
        self.last_clock.fetch_max(clock, Ordering::Relaxed);

        // 1. Update in-memory write buffer if entry is buffered
        let in_wb = self.write_buffer.update_payload(id, &patch);

        // 2. Update active HNSW index
        let internal_id = self.to_internal_id(id);
        let mut res = self
            .index_link
            .load()
            .update_payload_metadata(internal_id, &patch);

        if in_wb && res.is_err() {
            // Point is actively being indexed right now by worker thread.
            // Spin briefly until index_node completes so HNSW index gets patched.
            for _ in 0..50 {
                tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
                res = self
                    .index_link
                    .load()
                    .update_payload_metadata(internal_id, &patch);
                if res.is_ok() {
                    break;
                }
            }
        }

        if !in_wb && res.is_err() {
            return Err(format!("Node {internal_id} not found"));
        }

        // 3. Write to WAL
        if write_wal {
            let wal_guard = self.wal_link.load();
            let mut wal = wal_guard.lock().await;
            wal.append_update_payload(id as u64, &patch, clock)
                .map_err(|e| format!("WAL Error: {e}"))?;
            self.wal_pending_count.fetch_add(1, Ordering::SeqCst);
        }

        Ok(())
    }

    pub async fn insert_internal(
        &self,
        vector: &[f64],
        id: u32,
        metadata: HashMap<String, String>,
        clock: u64,
        durability: hyperspace_core::Durability,
        write_wal: bool,
    ) -> Result<(), String> {
        if vector.len() < self.dimension {
            return Err(format!(
                "Vector dimension mismatch. Expected {}, got {}",
                self.dimension,
                vector.len()
            ));
        }
        let slice = if vector.len() > self.dimension {
            &vector[..self.dimension]
        } else {
            vector
        };

        let processed_vector_cow = Self::normalize_if_cosine(slice);
        let processed_vector = &processed_vector_cow;

        // Check if this user ID already exists (for upsert)
        let existing_internal_id = self.id_map.get(&id).map(|v| *v);

        let mut reindex_needed = true;
        if let Some(old_internal_id) = existing_internal_id {
            let index = self.index_link.load();
            if (old_internal_id as usize) < index.count() {
                let old_vector = self.stored_vector(&index, old_internal_id);
                if self.config.is_gossip_enabled() {
                    let old_id_hash = CollectionDigest::hash_entry(id, &old_vector);
                    let bucket_idx = CollectionDigest::get_bucket_index(id);
                    self.buckets[bucket_idx].fetch_xor(old_id_hash, Ordering::Relaxed);
                    self.root_hash.fetch_xor(old_id_hash, Ordering::Relaxed);
                }

                if self.fast_upsert_delta > 0.0 {
                    let shift_sq = Self::shift_l2_sq(&old_vector, processed_vector);
                    let old_meta = index.metadata_by_id(old_internal_id);
                    let metadata_changed = old_meta != metadata;
                    reindex_needed = metadata_changed
                        || shift_sq > self.fast_upsert_delta * self.fast_upsert_delta;
                }
            }
        }

        if self.config.is_gossip_enabled() {
            let entry_hash = if self.storage_f32 {
                let canonical: Vec<f64> = processed_vector
                    .iter()
                    .map(|&x| (x as f32) as f64)
                    .collect();
                CollectionDigest::hash_entry(id, &canonical)
            } else {
                CollectionDigest::hash_entry(id, processed_vector)
            };
            let bucket_idx = CollectionDigest::get_bucket_index(id);
            self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);
            self.root_hash.fetch_xor(entry_hash, Ordering::Relaxed);
        }

        let internal_id = if let Some(old_id) = existing_internal_id {
            if old_id != id {
                self.ids_are_identity.store(false, Ordering::Release);
            }
            self.index_link
                .load()
                .update_storage(old_id, &self.index_view(processed_vector))
                .map_err(|e| e.clone())?;
            old_id
        } else {
            let new_id = self
                .index_link
                .load()
                .insert_to_storage(&self.index_view(processed_vector))
                .map_err(|e| e.clone())?;
            self.id_map.insert(id, new_id);
            self.reverse_id_map.insert(new_id, id);
            if new_id != id {
                self.ids_are_identity.store(false, Ordering::Release);
            }
            new_id
        };

        self.last_clock.fetch_max(clock, Ordering::Relaxed);
        let mut frozen_paths_opt = None;
        let mut commit_ticket = None;
        if write_wal {
            let wal_guard = self.wal_link.load();
            let mut wal = wal_guard.lock().await;

            // Group commit: the fsync is awaited after the WAL lock is released.
            commit_ticket = wal
                .append_with_ticket(
                    id,
                    processed_vector,
                    &metadata,
                    clock,
                    durability == hyperspace_core::Durability::Strict,
                )
                .map_err(|e| format!("WAL Error: {e}"))?;

            if wal.is_full() {
                if let Ok(frozen_path) = wal.rotate() {
                    self.wal_pending_count.store(0, Ordering::SeqCst);
                    let mut pending = self.pending_wal_flushes.lock().await;
                    pending.push(frozen_path);

                    let should_flush = match self.storage_mode {
                        StorageMode::Tiered => {
                            let memtable_nodes = self.index_link.load().count_nodes();
                            let memtable_budget = self.max_ram_bytes / 10;
                            let est_memory = memtable_nodes * (self.dimension * 8 + 64);
                            est_memory as u64 > memtable_budget
                        }
                        StorageMode::Performance => false,
                    };

                    if should_flush {
                        frozen_paths_opt = Some(std::mem::take(&mut *pending));
                    }
                }
            } else {
                self.wal_pending_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Durable ack: one group fsync shared with concurrent writers, off the WAL lock.
        if let Some(ticket) = commit_ticket {
            tokio::task::spawn_blocking(move || ticket.wait())
                .await
                .map_err(|e| format!("WAL Sync Error: {e}"))?
                .map_err(|e| format!("WAL Sync Error: {e}"))?;
        }

        if self.mode != hyperspace_core::QuantizationMode::None || self.index_dim < self.dimension {
            // One f64 -> f32 conversion feeds the exact store and the optional heap cache.
            let v_f32: Vec<f32> = processed_vector.iter().map(|&x| x as f32).collect();
            if let Some(exact) = self.exact_store.load_full() {
                if let Err(e) = exact.put(internal_id, &v_f32) {
                    eprintln!("Failed to store exact vector for rerank: {e}");
                }
            }
            if self.raw_vector_cache.contains_key(&internal_id)
                || self.raw_vector_cache_len.load(Ordering::Relaxed) < self.raw_vector_cache_cap
            {
                if self.raw_vector_cache.insert(internal_id, v_f32).is_none() {
                    self.raw_vector_cache_len.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        if let Some(frozen_paths) = frozen_paths_opt {
            Self::spawn_flush_worker(
                frozen_paths,
                self.config.clone(),
                self.mode,
                self.data_dir.clone(),
                self.flush_limiter.clone(),
                self.meta_router.clone(),
                self.index_link.clone(),
                self.id_map.clone(),
                self.reverse_id_map.clone(),
                self.flushing_vector_count.clone(),
                self.dimension,
                self.index_dim,
                self.chunk_backend.clone(),
            );
        }

        // One shared allocation for write buffer + indexer queue (no per-insert deep copies).
        let metadata = Arc::new(metadata);
        if reindex_needed {
            self.config.inc_queue();
            if self.write_buffer.size() < 100_000 {
                self.write_buffer.insert(
                    internal_id,
                    id,
                    processed_vector.to_vec(),
                    Arc::clone(&metadata),
                );
            }
            let _ = self.index_tx.send((internal_id, Arc::clone(&metadata)));
        }

        let vector_owned = processed_vector_cow.into_owned();

        if self.replication_tx.receiver_count() > 0 {
            let log = ReplicationLog {
                logical_clock: clock,
                origin_node_id: self.node_id.clone(),
                collection: self.name.clone(),
                operation: Some(replication_log::Operation::Insert(InsertOp {
                    id,
                    vector: vector_owned.clone(),
                    metadata: (*metadata).clone(),
                    typed_metadata: HashMap::new(),
                })),
            };
            let _ = self.replication_tx.send(log);
        }

        if let Some(ref cache) = self.cache {
            let ttl = metadata
                .get("__ttl")
                .or_else(|| metadata.get("ttl"))
                .and_then(|s| s.parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            cache.insert(id, vector_owned, Arc::unwrap_or_clone(metadata), ttl);
        }

        Ok(())
    }

    pub async fn insert_batch_internal(
        &self,
        vectors: Vec<(Vec<f64>, u32, HashMap<String, String>)>,
        clock: u64,
        durability: hyperspace_core::Durability,
        write_wal: bool,
    ) -> Result<(), String> {
        for (vec, _, _) in &vectors {
            if vec.len() < self.dimension {
                return Err(format!(
                    "Vector dimension mismatch. Expected {}, got {}",
                    self.dimension,
                    vec.len()
                ));
            }
        }

        let mut entries = Vec::with_capacity(vectors.len());
        let index_reader = self.index_link.load();

        for (vector, id, metadata) in &vectors {
            let slice = if vector.len() > self.dimension {
                &vector[..self.dimension]
            } else {
                vector
            };
            let processed_vector = Self::normalize_if_cosine(slice);
            let existing_internal_id = self.id_map.get(id).map(|v| *v);

            let mut reindex_needed = true;
            if let Some(old_internal_id) = existing_internal_id {
                if (old_internal_id as usize) < index_reader.count() {
                    let old_vector = self.stored_vector(&index_reader, old_internal_id);
                    if self.config.is_gossip_enabled() {
                        let old_id_hash = CollectionDigest::hash_entry(*id, &old_vector);
                        let bucket_idx = CollectionDigest::get_bucket_index(*id);
                        self.buckets[bucket_idx].fetch_xor(old_id_hash, Ordering::Relaxed);
                        self.root_hash.fetch_xor(old_id_hash, Ordering::Relaxed);
                    }

                    if self.fast_upsert_delta > 0.0 {
                        let shift_sq = Self::shift_l2_sq(&old_vector, &processed_vector);
                        let old_meta = index_reader.metadata_by_id(old_internal_id);
                        let metadata_changed = old_meta != *metadata;
                        reindex_needed = metadata_changed
                            || shift_sq > self.fast_upsert_delta * self.fast_upsert_delta;
                    }
                }
            }

            if self.config.is_gossip_enabled() {
                let entry_hash = if self.storage_f32 {
                    let canonical: Vec<f64> = processed_vector
                        .iter()
                        .map(|&x| (x as f32) as f64)
                        .collect();
                    CollectionDigest::hash_entry(*id, &canonical)
                } else {
                    CollectionDigest::hash_entry(*id, &processed_vector)
                };
                let bucket_idx = CollectionDigest::get_bucket_index(*id);
                self.buckets[bucket_idx].fetch_xor(entry_hash, Ordering::Relaxed);
                self.root_hash.fetch_xor(entry_hash, Ordering::Relaxed);
            }

            let internal_id = if let Some(old_id) = existing_internal_id {
                if old_id != *id {
                    self.ids_are_identity.store(false, Ordering::Release);
                }
                index_reader
                    .update_storage(old_id, &self.index_view(&processed_vector))
                    .map_err(|e| e.clone())?;
                old_id
            } else {
                let new_id = index_reader
                    .insert_to_storage(&self.index_view(&processed_vector))
                    .map_err(|e| e.clone())?;

                self.id_map.insert(*id, new_id);
                self.reverse_id_map.insert(new_id, *id);
                if new_id != *id {
                    self.ids_are_identity.store(false, Ordering::Release);
                }
                new_id
            };

            entries.push(BatchEntry {
                id: *id,
                vector: processed_vector,
                metadata,
                internal_id,
                reindex_needed,
            });
        }

        self.last_clock.fetch_max(clock, Ordering::Relaxed);
        let mut frozen_paths_opt = None;
        let mut commit_ticket = None;
        if write_wal {
            let wal_data: Vec<_> = entries
                .iter()
                .map(|e| (&e.vector[..], e.id, e.metadata))
                .collect();

            let wal_guard = self.wal_link.load();
            let mut wal = wal_guard.lock().await;
            commit_ticket = wal
                .append_batch_with_ticket(
                    &wal_data,
                    clock,
                    durability == hyperspace_core::Durability::Strict,
                )
                .map_err(|e| e.to_string())?;

            if wal.is_full() {
                if let Ok(frozen_path) = wal.rotate() {
                    self.wal_pending_count.store(0, Ordering::SeqCst);
                    let mut pending = self.pending_wal_flushes.lock().await;
                    pending.push(frozen_path);

                    let should_flush = match self.storage_mode {
                        StorageMode::Tiered => {
                            let memtable_nodes = self.index_link.load().count_nodes();
                            let memtable_budget = self.max_ram_bytes / 10;
                            let est_memory = memtable_nodes * (self.dimension * 8 + 64);
                            est_memory as u64 > memtable_budget
                        }
                        StorageMode::Performance => false,
                    };

                    if should_flush {
                        frozen_paths_opt = Some(std::mem::take(&mut *pending));
                    }
                }
            } else {
                self.wal_pending_count
                    .fetch_add(vectors.len() as u64, Ordering::SeqCst);
            }
        }

        // Durable ack: one group fsync shared with concurrent writers, off the WAL lock.
        if let Some(ticket) = commit_ticket {
            tokio::task::spawn_blocking(move || ticket.wait())
                .await
                .map_err(|e| format!("WAL Sync Error: {e}"))?
                .map_err(|e| format!("WAL Sync Error: {e}"))?;
        }

        if let Some(frozen_paths) = frozen_paths_opt {
            Self::spawn_flush_worker(
                frozen_paths,
                self.config.clone(),
                self.mode,
                self.data_dir.clone(),
                self.flush_limiter.clone(),
                self.meta_router.clone(),
                self.index_link.clone(),
                self.id_map.clone(),
                self.reverse_id_map.clone(),
                self.flushing_vector_count.clone(),
                self.dimension,
                self.index_dim,
                self.chunk_backend.clone(),
            );
        }

        for _ in 0..entries.iter().filter(|e| e.reindex_needed).count() {
            self.config.inc_queue();
        }

        if self.mode != hyperspace_core::QuantizationMode::None || self.index_dim < self.dimension {
            for entry in &entries {
                let v_f32: Vec<f32> = entry.vector.iter().map(|&x| x as f32).collect();
                // Batch inserts need exact copies too (rerank, calibration, RebuildIndex);
                // before 4.2.5 only the bounded heap cache got them.
                if let Some(exact) = self.exact_store.load_full() {
                    if let Err(e) = exact.put(entry.internal_id, &v_f32) {
                        eprintln!("Failed to store exact vector for rerank: {e}");
                    }
                }
                if self.raw_vector_cache.contains_key(&entry.internal_id)
                    || self.raw_vector_cache_len.load(Ordering::Relaxed) < self.raw_vector_cache_cap
                {
                    if self
                        .raw_vector_cache
                        .insert(entry.internal_id, v_f32)
                        .is_none()
                    {
                        self.raw_vector_cache_len.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }

        let wb_has_capacity = self.write_buffer.size() < 100_000;
        for entry in &entries {
            if entry.reindex_needed {
                let meta_shared = Arc::new(entry.metadata.clone());
                if wb_has_capacity {
                    self.write_buffer.insert(
                        entry.internal_id,
                        entry.id,
                        entry.vector.to_vec(),
                        Arc::clone(&meta_shared),
                    );
                }
                let _ = self.index_tx.send((entry.internal_id, meta_shared));
            }
        }

        if self.replication_tx.receiver_count() > 0 {
            for entry in entries {
                let log = ReplicationLog {
                    logical_clock: clock,
                    origin_node_id: self.node_id.clone(),
                    collection: self.name.clone(),
                    operation: Some(replication_log::Operation::Insert(InsertOp {
                        id: entry.id,
                        vector: entry.vector.into_owned(),
                        metadata: entry.metadata.clone(),
                        typed_metadata: HashMap::new(),
                    })),
                };
                let _ = self.replication_tx.send(log);
            }
        }
        if let Some(ref cache) = self.cache {
            for (vector, id, metadata) in &vectors {
                let slice = if vector.len() > self.dimension {
                    &vector[..self.dimension]
                } else {
                    vector
                };
                let processed_vector = Self::normalize_if_cosine(slice).into_owned();
                let ttl = metadata
                    .get("__ttl")
                    .or_else(|| metadata.get("ttl"))
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(std::time::Duration::from_secs);
                cache.insert(*id, processed_vector, metadata.clone(), ttl);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod auto_calibration_tests {
    use super::{
        coarse_pool_factor, maybe_auto_calibrate, raw_cache_capacity, AUTO_CALIBRATE_GROWTH,
        COARSE_RERANK_POOL,
    };
    use dashmap::DashMap;
    use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
    use hyperspace_index::HnswIndex;
    use hyperspace_store::PayloadStore;
    use hyperspace_store::VectorStore;
    use std::collections::HashMap;
    use std::sync::Arc;

    const DIM: usize = 16;

    fn fill(
        mode: QuantizationMode,
        n: u32,
    ) -> (
        tempfile::TempDir,
        HnswIndex<EuclideanMetric>,
        DashMap<u32, Vec<f32>>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let element = match mode {
            QuantizationMode::PQ4 => {
                hyperspace_core::pq4::code_size(hyperspace_core::pq4::default_num_subvectors(DIM))
            }
            _ => hyperspace_core::pq::default_num_subvectors(DIM),
        };
        let store = Arc::new(VectorStore::new(dir.path(), element));
        let index =
            HnswIndex::<EuclideanMetric>::new(store, mode, Arc::new(GlobalConfig::default()), DIM);
        let raw = DashMap::new();
        for i in 0..n {
            let v: Vec<f64> = (0..DIM)
                .map(|d| f64::from((i * 7 + d as u32 * 13) % 101) / 101.0)
                .collect();
            let id = index.insert_to_storage(&v).unwrap();
            index.index_node(id, HashMap::new()).unwrap();
            raw.insert(id, v.iter().map(|&x| x as f32).collect());
        }
        (dir, index, raw)
    }

    #[test]
    fn calibrates_at_threshold_then_refits_after_growth() {
        let (_d, index, raw) = fill(QuantizationMode::PQ4, 300);
        assert!(
            maybe_auto_calibrate(&index, &raw, None, None, 400, "t").is_none(),
            "below threshold"
        );
        assert!(
            maybe_auto_calibrate(&index, &raw, None, None, 0, "t").is_none(),
            "0 disables"
        );
        assert_eq!(
            maybe_auto_calibrate(&index, &raw, None, None, 256, "t"),
            Some((300, 0)),
            "every stored code re-encoded"
        );
        assert!(!index.needs_calibration());
        assert_eq!(index.calibrated_at(), 300);
        assert!(
            maybe_auto_calibrate(&index, &raw, None, None, 256, "t").is_none(),
            "no refit before growth"
        );

        // Grow the shard past the growth factor: the next call re-fits.
        for i in 300..300 * AUTO_CALIBRATE_GROWTH as u32 {
            let v: Vec<f64> = (0..DIM)
                .map(|d| f64::from((i + d as u32) % 89) / 89.0)
                .collect();
            let id = index.insert_to_storage(&v).unwrap();
            raw.insert(id, v.iter().map(|&x| x as f32).collect());
        }
        assert!(maybe_auto_calibrate(&index, &raw, None, None, 256, "t").is_some());
        assert_eq!(index.calibrated_at(), 300 * AUTO_CALIBRATE_GROWTH);
    }

    #[test]
    fn pq_waits_for_enough_samples_per_centroid() {
        let (_d, index, raw) = fill(QuantizationMode::ProductQuantization, 1500);
        assert!(maybe_auto_calibrate(&index, &raw, None, None, 64, "t").is_none());
        assert!(
            index.needs_calibration(),
            "PQ keeps the default codebook below 4096 vectors"
        );
    }

    #[test]
    fn modes_without_training_are_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(VectorStore::new(dir.path(), DIM + 4));
        let index = HnswIndex::<EuclideanMetric>::new(
            store,
            QuantizationMode::ScalarI8,
            Arc::new(GlobalConfig::default()),
            DIM,
        );
        assert!(maybe_auto_calibrate(&index, &DashMap::new(), None, None, 1, "t").is_none());
    }

    #[test]
    fn calibration_falls_back_to_the_payload_store() {
        // Raw cache holds only the training sample; every other exact vector comes from
        // the payload store, so all codes are still re-encoded.
        let (_d, index, full) = fill(QuantizationMode::PQ4, 600);
        let pdir = tempfile::tempdir().unwrap();
        let payload = PayloadStore::open(pdir.path(), 1).unwrap();
        for e in &full {
            let bytes: Vec<u8> = e.value().iter().flat_map(|x| x.to_le_bytes()).collect();
            payload.insert(*e.key(), &bytes).unwrap();
        }
        let partial: DashMap<u32, Vec<f32>> = DashMap::new();
        for e in full.iter().take(300) {
            partial.insert(*e.key(), e.value().clone());
        }
        assert_eq!(
            maybe_auto_calibrate(&index, &partial, None, Some(&payload), 256, "t"),
            Some((600, 0)),
            "vectors missing from the raw cache are read from the payload store"
        );
        // Without the payload store the other 300 keep their old code.
        let (_d2, index2, full2) = fill(QuantizationMode::PQ4, 600);
        let half: DashMap<u32, Vec<f32>> = DashMap::new();
        for e in full2.iter().take(300) {
            half.insert(*e.key(), e.value().clone());
        }
        assert_eq!(
            maybe_auto_calibrate(&index2, &half, None, None, 256, "t"),
            Some((300, 300))
        );
    }

    #[test]
    fn coarse_rerank_pool_is_the_same_for_any_shard_count() {
        // Total pool = shards × per-shard top_k × factor, in units of top_k × oversample.
        let total = |s: usize| s * if s == 1 { 1 } else { 2 } * coarse_pool_factor(s);
        assert_eq!(total(1), COARSE_RERANK_POOL);
        assert_eq!(total(12), COARSE_RERANK_POOL, "pre-4.2.5 default layout");
        for s in [2, 3, 4, 6, 8, 16, 32] {
            assert!(total(s) >= COARSE_RERANK_POOL, "{s} shards: {}", total(s));
            assert!(
                total(s) < 2 * COARSE_RERANK_POOL + 2 * s,
                "{s} shards: {}",
                total(s)
            );
        }
    }

    #[test]
    fn raw_cache_budget_is_split_across_shards() {
        // 256 MiB default, 1024-D vectors (4,160 B incl. overhead).
        let one = raw_cache_capacity(1024, 1, false);
        let twelve = raw_cache_capacity(1024, 12, false);
        assert!((60_000..=70_000).contains(&one), "{one}");
        assert!(twelve * 12 <= one + 12 && twelve >= 256);
        assert!(
            raw_cache_capacity(16, 1, false) > one,
            "small vectors fit more entries"
        );
    }
}

#[cfg(test)]
mod mrl_index_tests {
    use super::{mrl_index_dimension, mrl_prefix, resolve_index_dim, MRL_INDEX_MARKER};
    use hyperspace_core::{CosineMetric, EuclideanMetric, PoincareMetric};
    use hyperspace_proto::hyperspace::{CollectionSchema, MrlLayer, VectorComponent};

    fn schema(components: usize, cutoff: u32) -> CollectionSchema {
        CollectionSchema {
            components: (0..components)
                .map(|i| VectorComponent {
                    name: format!("c{i}"),
                    metric: "cosine".into(),
                    full_dimension: 256,
                    weight: 1.0,
                })
                .collect(),
            cascade_pipeline: vec![MrlLayer {
                component_name: "c0".into(),
                cutoff_dimension: cutoff,
                store_in_ram: true,
                rerank_top_k: 0,
            }],
            encrypted: None,
        }
    }

    #[test]
    fn eligibility() {
        assert_eq!(
            mrl_index_dimension::<CosineMetric>(&schema(1, 64), 256),
            Some(64)
        );
        assert_eq!(
            mrl_index_dimension::<EuclideanMetric>(&schema(1, 64), 256),
            Some(64)
        );
        assert_eq!(
            mrl_index_dimension::<PoincareMetric>(&schema(1, 64), 256),
            None,
            "l2 / cosine only"
        );
        assert_eq!(
            mrl_index_dimension::<CosineMetric>(&schema(2, 64), 256),
            None,
            "single component"
        );
        assert_eq!(
            mrl_index_dimension::<CosineMetric>(&schema(1, 256), 256),
            None,
            "cutoff < dim"
        );
        assert_eq!(
            mrl_index_dimension::<CosineMetric>(&schema(1, 4), 256),
            None,
            "tiny cutoff"
        );
        let mut no_cascade = schema(1, 64);
        no_cascade.cascade_pipeline.clear();
        assert_eq!(mrl_index_dimension::<CosineMetric>(&no_cascade, 256), None);
    }

    #[test]
    fn marker_decides_the_layout() {
        let fresh = tempfile::tempdir().unwrap();
        assert_eq!(resolve_index_dim(fresh.path(), 256, Some(64)), 64);
        assert!(fresh.path().join(MRL_INDEX_MARKER).exists());
        // The marker wins on reopen, whatever the schema says now.
        assert_eq!(resolve_index_dim(fresh.path(), 256, None), 64);

        // Pre-existing full-dimension data without a marker stays full.
        let legacy = tempfile::tempdir().unwrap();
        std::fs::write(legacy.path().join("chunk_0.hyp"), b"x").unwrap();
        assert_eq!(resolve_index_dim(legacy.path(), 256, Some(64)), 256);
        assert!(!legacy.path().join(MRL_INDEX_MARKER).exists());

        let plain = tempfile::tempdir().unwrap();
        assert_eq!(resolve_index_dim(plain.path(), 256, None), 256);
        assert!(!plain.path().join(MRL_INDEX_MARKER).exists());
    }

    #[test]
    fn prefix_view() {
        let v = vec![3.0, 4.0, 12.0];
        assert_eq!(&*mrl_prefix::<CosineMetric>(&v, 2), &[0.6, 0.8]);
        assert_eq!(&*mrl_prefix::<EuclideanMetric>(&v, 2), &[3.0, 4.0]);
        assert!(matches!(
            mrl_prefix::<CosineMetric>(&v, 3),
            std::borrow::Cow::Borrowed(_)
        ));
    }
}
