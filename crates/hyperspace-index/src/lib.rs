#![allow(unused_features)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::inline_always)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::ref_as_ptr)]
#![allow(unknown_lints)]
#![allow(clippy::chunks_exact_to_as_chunks)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::needless_bool)]

pub mod flat_layer0;
mod layer0;
pub mod spatial_aabb;
pub mod stopwords;
pub mod tokenizer;

pub use flat_layer0::FlatLayer0Segment;

use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use rand::Rng;
use rayon::prelude::*;
#[cfg(feature = "persistence")]
use rkyv::ser::Serializer;
use rkyv::{Archive, Deserialize, Serialize};
use roaring::RoaringBitmap;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BinaryHeap, HashMap};
#[cfg(feature = "persistence")]
use std::fs::File;
#[cfg(feature = "persistence")]
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

// Imports
use hyperspace_core::hybrid::{
    HybridExtremeQuantizedVector, HybridLowBitQuantizedVector, HybridQuantizedVector,
    ScalarI4Vector,
};
use hyperspace_core::vector::{
    BinaryHyperVector, HyperVector, HyperVectorF32, QuantizedHyperVector,
};
use hyperspace_core::QuantizationMode;
use hyperspace_core::{GlobalConfig, LorentzMetric, Metric, PoincareMetric};
use hyperspace_store::VectorStore;
use std::marker::PhantomData;

#[cfg(not(target_arch = "wasm32"))]
static TMP_SNAP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct SnapshotData {
    pub max_layer: u32,
    pub entry_point: u32,
    pub nodes: Vec<SnapshotNode>,
    pub metadata: SnapshotMetadata,
}

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct SnapshotNode {
    pub id: u32,
    pub layers: Vec<Vec<u32>>,
}

pub type KeyedBitmaps = Vec<(i64, Vec<u8>)>;

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct SnapshotMetadata {
    // Key -> Serialized RoaringBitmap
    pub inverted: Vec<(String, Vec<u8>)>,
    // Key -> [(Value, Serialized RoaringBitmap)]
    pub numeric: Vec<(String, KeyedBitmaps)>,
    // Serialized RoaringBitmap for deleted items
    pub deleted: Vec<u8>,
    // Mapping ID -> Metadata Map
    pub forward: Vec<(u32, Vec<(String, String)>)>,
}

// Constants are defined later in the file.

/// Side files persisting code-quantizer state next to `index.snap`.
const RABITQ_FILE: &str = "rabitq_centroid.bin";
const PQ4_FILE: &str = "pq4_codebook.bin";
/// Written next to the snapshot once a trained quantizer has been fitted to data.
const CALIBRATED_MARKER: &str = "quantizer.calibrated";
/// Written next to the snapshot of a ScalarI8 L2 / cosine index whose codes use a
/// per-vector scale (`alpha = max |x|`); indexes without it keep the legacy `x · 127` codes.
const SQ8_SCALED_MARKER: &str = "sq8.scaled";

/// Metadata key prefix for typed values (`{"v": ...}` JSON payloads).
const TYPED_META_PREFIX: &str = "__hs_typed__";

/// Maps an `f64` onto a `u64` whose unsigned ordering equals the numeric ordering
/// (sign bit flipped for non-negatives, all bits inverted for negatives; `-0.0 == 0.0`).
#[inline]
#[must_use]
pub fn ordered_f64_key(v: f64) -> u64 {
    let v = if v == 0.0 { 0.0 } else { v };
    let bits = v.to_bits();
    if bits >> 63 == 1 {
        !bits
    } else {
        bits | (1u64 << 63)
    }
}

use hyperspace_core::FilterExpr;

#[derive(Debug)]
pub struct MetadataIndex {
    pub inverted: DashMap<String, RoaringBitmap>,
    pub prefix_index: DashMap<String, crossbeam_skiplist::SkipSet<String>>,
    /// Numeric range index: field -> (order-preserving `f64` key -> ids).
    /// Keys come from [`ordered_f64_key`]; rebuilt from `forward` on snapshot load.
    pub numeric: DashMap<String, crossbeam_skiplist::SkipMap<u64, RwLock<RoaringBitmap>>>,
    pub deleted: RwLock<RoaringBitmap>,
    pub has_deletions: AtomicBool,
    pub forward: DashMap<u32, std::collections::HashMap<String, String>>,
    pub token_df: DashMap<String, u32>,
    pub doc_token_len: boxcar::Vec<AtomicU32>,
    pub term_doc_freq: DashMap<String, Vec<(u32, u16)>>,
    pub total_token_len: AtomicU64,
    pub has_lexical_index: AtomicBool,
}

impl Default for MetadataIndex {
    fn default() -> Self {
        Self {
            inverted: DashMap::new(),
            prefix_index: DashMap::new(),
            numeric: DashMap::new(),
            deleted: RwLock::new(RoaringBitmap::new()),
            has_deletions: AtomicBool::new(false),
            forward: DashMap::new(),
            token_df: DashMap::new(),
            doc_token_len: boxcar::Vec::new(),
            term_doc_freq: DashMap::new(),
            total_token_len: AtomicU64::new(0),
            has_lexical_index: AtomicBool::new(false),
        }
    }
}

impl<M: Metric> HnswIndex<M> {
    /// Extracts the numeric range-index entry for a metadata `(key, value)` pair.
    /// Plain values are parsed as `f64`; typed values (`__hs_typed__<key>` = `{"v": ...}`)
    /// are indexed under `<key>`. Parsing happens once at write time, never at query time.
    fn numeric_entry<'a>(key: &'a str, val: &str) -> Option<(&'a str, f64)> {
        let (field, num) = if let Some(field) = key.strip_prefix(TYPED_META_PREFIX) {
            let parsed = serde_json::from_str::<serde_json::Value>(val).ok()?;
            (field, parsed.get("v")?.as_f64()?)
        } else {
            (key, val.trim().parse::<f64>().ok()?)
        };
        num.is_finite().then_some((field, num))
    }

    fn numeric_insert(&self, field: &str, num: f64, id: NodeId) {
        let key = ordered_f64_key(num);
        let tree = self.metadata.numeric.entry(field.to_string()).or_default();
        tree.get_or_insert_with(key, || RwLock::new(RoaringBitmap::new()))
            .value()
            .write()
            .insert(id);
    }

    fn numeric_remove(&self, field: &str, num: f64, id: NodeId) {
        if let Some(tree) = self.metadata.numeric.get(field) {
            if let Some(entry) = tree.get(&ordered_f64_key(num)) {
                entry.value().write().remove(id);
            }
        }
    }

    /// Rebuilds the numeric range index from the forward metadata store.
    fn rebuild_numeric_index(&self) {
        self.metadata.numeric.clear();
        for item in &self.metadata.forward {
            let id = *item.key();
            for (k, v) in item.value() {
                if let Some((field, num)) = Self::numeric_entry(k, v) {
                    self.numeric_insert(field, num, id);
                }
            }
        }
    }

    /// Freezes Layer-0 into a cache-aligned, continuous strided FlatLayer0Segment.
    /// This eliminates pointer chasing and heap allocations for Layer-0 lookups.
    pub fn freeze_flat_layer0(&self) -> Arc<FlatLayer0Segment> {
        let node_count = self.nodes.count();
        let max_m = self.config.get_m() * 2;
        let vector_len = if node_count > 0 {
            self.storage.get(0).len()
        } else {
            0
        };
        let mut segment = FlatLayer0Segment::new(node_count, max_m, vector_len);
        let l0 = self.layer0.view();
        let mut neighbors = layer0::Links::new();
        for id in 0..node_count as u32 {
            l0.read_into(id, &mut neighbors);
            segment.set_node(id, &neighbors, self.storage.get(id));
        }
        let seg_arc = Arc::new(segment);
        self.flat_l0.store(Some(seg_arc.clone()));
        seg_arc
    }

    /// Sets an already prepared FlatLayer0Segment into this index.
    pub fn set_flat_layer0(&self, segment: Arc<FlatLayer0Segment>) {
        self.flat_l0.store(Some(segment));
    }

    /// Checks if a FlatLayer0Segment is currently active.
    #[must_use]
    pub fn has_flat_layer0(&self) -> bool {
        self.flat_l0.load().is_some()
    }

    /// Freezes Layer-0 into a cache-aligned, continuous strided FlatLayer0Segment if not
    /// already active (explicit request: unlike the automatic freeze on snapshot / load it
    /// ignores the small-index skip).
    pub fn ensure_flat_layer0(&self) {
        if self.flat_l0.load().is_none() && self.nodes.count() > 0 {
            self.freeze_flat_layer0();
        }
    }

    /// The frozen segment copies every vector next to its adjacency. Indexes small enough
    /// for an exact flat scan at the configured beam never walk it, so they skip the copy.
    fn flat_segment_worthwhile(&self) -> bool {
        let n = self.nodes.count();
        n > 0
            && n > self.flat_scan_limit(&hyperspace_core::SearchParams {
                top_k: 10,
                ..Default::default()
            })
    }

    /// Mirrors a layer-0 neighbor update into the frozen segment. A memory-mapped segment
    /// (loaded from `flat_l0.bin`) is read-only: drop it instead of serving stale links.
    fn sync_flat_neighbors(&self, id: NodeId, links: &[NodeId]) {
        if let Some(ref flat) = *self.flat_l0.load() {
            if !flat.update_neighbors_in_place(id, links) && (id as usize) < flat.node_count() {
                self.flat_l0.store(None);
            }
        }
    }

    #[cfg(feature = "persistence")]
    pub fn save_snapshot(&self, path: &std::path::Path) -> Result<(), String> {
        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        let nodes_count = self.nodes.count();
        let mut snapshot_nodes = Vec::with_capacity(nodes_count);

        for (_, node) in &self.nodes {
            snapshot_nodes.push(SnapshotNode {
                id: node.id,
                layers: self.snapshot_layers(node),
            });
        }

        let mut inverted_vec = Vec::new();
        for item in &self.metadata.inverted {
            let mut buf = Vec::new();
            item.value()
                .serialize_into(&mut buf)
                .map_err(|e| e.to_string())?;
            inverted_vec.push((item.key().clone(), buf));
        }

        // The numeric range index is derived data: it is rebuilt from `forward` on load
        // (order-preserving f64 keys), so the legacy i64 section is written empty.
        let numeric_vec: Vec<(String, KeyedBitmaps)> = Vec::new();

        let mut deleted_buf = Vec::new();
        self.metadata
            .deleted
            .read()
            .serialize_into(&mut deleted_buf)
            .map_err(|e| e.to_string())?;

        let mut forward_vec = Vec::new();
        for item in &self.metadata.forward {
            let mut map_vec = Vec::new();
            for (k, v) in item.value() {
                map_vec.push((k.clone(), v.clone()));
            }
            forward_vec.push((*item.key(), map_vec));
        }

        let data = SnapshotData {
            max_layer,
            entry_point,
            nodes: snapshot_nodes,
            metadata: SnapshotMetadata {
                inverted: inverted_vec,
                numeric: numeric_vec,
                deleted: deleted_buf,
                forward: forward_vec,
            },
        };

        // Serialize
        let mut serializer = rkyv::ser::serializers::AllocSerializer::<256>::default();
        serializer.serialize_value(&data).map_err(
            |e: rkyv::ser::serializers::CompositeSerializerError<_, _, _>| e.to_string(),
        )?;
        drop(data);
        let bytes = serializer.into_serializer().into_inner();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
        }

        let count = TMP_SNAP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp_path = path.with_extension(format!("snap.tmp.{}.{}", std::process::id(), count));
        {
            let mut file = File::create(&tmp_path).map_err(|e| e.to_string())?;
            file.write_all(&bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
        }
        drop(bytes);
        std::fs::rename(&tmp_path, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp_path);
            e.to_string()
        })?;
        if let Some(parent) = path.parent() {
            if let Ok(dir) = std::fs::File::open(parent) {
                let _ = dir.sync_all();
            }
        }

        if let Some(parent) = path.parent() {
            if self.flat_l0.load().is_none() && self.flat_segment_worthwhile() {
                self.freeze_flat_layer0();
            }
            let flat_guard = self.flat_l0.load();
            if let Some(ref seg) = *flat_guard {
                let flat_path = parent.join("flat_l0.bin");
                let _ = seg.save_to_file(&flat_path);
            }
        }

        if let Some(ref pq) = *self.pq.load() {
            if let Some(parent) = path.parent() {
                let codebook_path = parent.join("pq_codebook.bin");
                let _ = pq.save_to_file(&codebook_path);
            }
        }
        if let Some(parent) = path.parent() {
            if let Some(ref rq) = *self.rabitq.load() {
                let _ = rq.save_to_file(&parent.join(RABITQ_FILE));
            }
            if let Some(ref pq4) = *self.pq4.load() {
                let _ = pq4.save_to_file(&parent.join(PQ4_FILE));
            }
            if self.mode.needs_training() && self.is_calibrated() {
                let _ = std::fs::write(parent.join(CALIBRATED_MARKER), b"1");
            }
            if self.sq8_scaled {
                let _ = std::fs::write(parent.join(SQ8_SCALED_MARKER), b"1");
            }
        }

        Ok(())
    }

    #[cfg(feature = "persistence")]
    pub fn load_snapshot(
        path: &std::path::Path,
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        dimension: usize,
    ) -> Result<Self, String> {
        Self::load_snapshot_with_storage_precision(path, storage, mode, config, false, dimension)
    }

    #[cfg(feature = "persistence")]
    pub fn load_snapshot_with_storage_precision(
        path: &std::path::Path,
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        storage_f32: bool,
        dimension: usize,
    ) -> Result<Self, String> {
        use std::time::Instant;
        let start = Instant::now();

        println!("📂 Loading snapshot: {}", path.display());

        if let Some(parent) = path.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let ep = entry.path();
                    if let Some(name) = ep.file_name().and_then(|n| n.to_str()) {
                        if name.contains(".tmp.") {
                            let _ = std::fs::remove_file(ep);
                        }
                    }
                }
            }
        }

        // Memory-map the snapshot file for zero-copy access.
        let file = File::open(path).map_err(|e| format!("Failed to open snapshot: {e}"))?;
        let file_size = file.metadata().map_err(|e| e.to_string())?.len();
        println!("   File size: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| format!("Failed to mmap snapshot: {e}"))?
        };
        let mmap_time = start.elapsed();
        println!("   ✓ Memory-mapped in {:.3}s", mmap_time.as_secs_f64());

        // 2. Validate archived data
        let archived = rkyv::check_archived_root::<SnapshotData>(&mmap)
            .map_err(|e| format!("Snapshot corruption: {e}"))?;
        let validate_time = start.elapsed();
        println!("   ✓ Validated in {:.3}s", validate_time.as_secs_f64());

        // 3. Deserialize
        let deserialized: SnapshotData = archived.deserialize(&mut rkyv::Infallible).unwrap();
        let deserialize_time = start.elapsed();
        println!(
            "   ✓ Deserialized in {:.3}s",
            deserialize_time.as_secs_f64()
        );

        // 4. Reconstruct Graph with progress
        let total_nodes = deserialized.nodes.len();
        let nodes_bc: boxcar::Vec<Node> = boxcar::Vec::with_capacity(total_nodes);
        let mut layer0_lists: Vec<Vec<NodeId>> = Vec::with_capacity(total_nodes);

        println!("   ⏳ Reconstructing HNSW graph: {total_nodes} nodes...");

        let progress_interval = if total_nodes > 100_000 {
            50_000
        } else {
            10_000
        };

        for (i, s_node) in deserialized.nodes.into_iter().enumerate() {
            // Progress reporting
            if i > 0 && i % progress_interval == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                let progress_pct = (i as f64 / total_nodes as f64) * 100.0;
                let nodes_per_sec = i as f64 / elapsed;
                let eta = (total_nodes - i) as f64 / nodes_per_sec;
                println!(
                    "      Progress: {i}/{total_nodes} ({progress_pct:.1}%) | {nodes_per_sec:.0} nodes/s | ETA: {eta:.1}s"
                );
            }

            // Reconstruct node; boxcar push order == s_node.id (sequential snapshot)
            let mut s_layers = s_node.layers.into_iter();
            layer0_lists.push(s_layers.next().unwrap_or_default());
            nodes_bc.push(Node {
                id: s_node.id,
                upper: s_layers
                    .map(|l| RwLock::new(SmallVec::from_vec(l)))
                    .collect(),
            });
        }
        let layer0 = Self::build_layer0(&config, layer0_lists);

        // Sync storage count
        storage.set_count(nodes_bc.count());

        let total_time = start.elapsed();
        println!(
            "   ✅ Loaded {} nodes in {:.3}s ({:.0} nodes/s)",
            total_nodes,
            total_time.as_secs_f64(),
            total_nodes as f64 / total_time.as_secs_f64()
        );

        println!("   📦 Restoring Metadata Index...");

        let inverted = DashMap::new();
        let prefix_index: DashMap<String, crossbeam_skiplist::SkipSet<String>> = DashMap::new();
        deserialized
            .metadata
            .inverted
            .into_par_iter()
            .for_each(|(k, v)| {
                let bitmap = RoaringBitmap::deserialize_from(&v[..]).unwrap_or_default();
                if let Some((field, val)) = k.split_once(':') {
                    prefix_index
                        .entry(field.to_string())
                        .or_default()
                        .insert(val.to_string());
                }
                inverted.insert(k, bitmap);
            });

        // Legacy i64 numeric sections are ignored; the f64 range index is rebuilt from `forward`.
        let numeric = DashMap::new();

        let deleted =
            RoaringBitmap::deserialize_from(&deserialized.metadata.deleted[..]).unwrap_or_default();

        let forward = DashMap::new();
        let has_nonempty = AtomicBool::new(false);
        deserialized
            .metadata
            .forward
            .into_par_iter()
            .for_each(|(k, v)| {
                let mut attributes = std::collections::HashMap::with_capacity(v.len());
                for (mk, mv) in v {
                    attributes.insert(mk, mv);
                }
                if !attributes.is_empty() {
                    has_nonempty.store(true, Ordering::Relaxed);
                }
                forward.insert(k, attributes);
            });
        let has_nonempty_metadata = has_nonempty.load(Ordering::Relaxed);

        let fast_routing = std::env::var("HS_FAST_ROUTING")
            .is_ok_and(|v| v.to_lowercase() == "true")
            && matches!(M::name(), "poincare" | "lorentz" | "hybrid")
            && matches!(mode, QuantizationMode::None);

        let density_pruning =
            std::env::var("HS_DENSITY_PRUNING").is_ok_and(|v| v.to_lowercase() == "true");
        let zonal =
            std::env::var("HS_ZONAL_QUANTIZATION").is_ok_and(|v| v.to_lowercase() == "true");

        let node_count = storage.count();
        let pq = match mode {
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ => {
                let codebook_path = path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join("pq_codebook.bin");
                if codebook_path.exists() {
                    if let Ok(loaded) =
                        hyperspace_core::pq::ProductQuantizer::load_from_file(&codebook_path)
                    {
                        Some(Arc::new(loaded))
                    } else {
                        let is_opq = mode == QuantizationMode::OPQ;
                        let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                        Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                            dimension, num_sub, is_opq,
                        )))
                    }
                } else {
                    let is_opq = mode == QuantizationMode::OPQ;
                    let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                    Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                        dimension, num_sub, is_opq,
                    )))
                }
            }
            _ => None,
        };

        let (rabitq, pq4) = Self::code_quantizers(mode, dimension, path.parent());
        let has_del = !deleted.is_empty();
        let index = Self {
            nodes: nodes_bc,
            layer0,
            append_lock: Mutex::new(()),
            metadata: MetadataIndex {
                inverted,
                prefix_index,
                numeric,
                deleted: RwLock::new(deleted),
                has_deletions: AtomicBool::new(has_del),
                forward,
                token_df: DashMap::new(),
                doc_token_len: boxcar::Vec::new(),
                term_doc_freq: DashMap::new(),
                total_token_len: AtomicU64::new(0),
                has_lexical_index: AtomicBool::new(false),
            },
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            storage_f32,
            config,
            has_nonempty_metadata: AtomicBool::new(has_nonempty_metadata),
            fast_routing,
            density_pruning,
            zonal,
            zonal_storage: dashmap::DashMap::new(),
            node_counter: AtomicU32::new(node_count as u32),
            spatial_index: Arc::new(spatial_aabb::SpatialAabbIndex::new(dimension)),
            pq: arc_swap::ArcSwapOption::from(pq),
            flat_l0: arc_swap::ArcSwapOption::empty(),
            deletes_since_repair: AtomicUsize::new(0),
            rabitq: arc_swap::ArcSwapOption::from(rabitq),
            pq4: arc_swap::ArcSwapOption::from(pq4),
            calibrated: AtomicBool::new(
                !mode.needs_training()
                    || path
                        .parent()
                        .is_some_and(|d| d.join(CALIBRATED_MARKER).exists()),
            ),
            calibration_gate: RwLock::new(()),
            calibrated_at: AtomicUsize::new(0),
            tuned_ef: AtomicUsize::new(0),
            tuned_at: AtomicUsize::new(0),
            query_samples: Mutex::new(std::collections::VecDeque::new()),
            queries_seen: AtomicUsize::new(0),
            tuned_queries_seen: AtomicUsize::new(usize::MAX),
            tuned_at_ms: AtomicU64::new(0),
            sq8_scaled: Self::sq8_scaled_supported(mode)
                && path
                    .parent()
                    .is_some_and(|d| d.join(SQ8_SCALED_MARKER).exists()),
            repair_running: AtomicBool::new(false),
            graph_engine: None,
            _marker: PhantomData,
            dimension,
        };
        index.rebuild_lexical_stats();
        index.rebuild_numeric_index();
        let flat_path = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("flat_l0.bin");
        if !index.flat_segment_worthwhile() {
            // Small index: searches scan it flat; keep neither the copy nor the file mapping.
        } else if flat_path.exists() {
            if let Ok(seg) = FlatLayer0Segment::load_from_file(&flat_path) {
                index.flat_l0.store(Some(Arc::new(seg)));
            }
        } else {
            index.freeze_flat_layer0();
        }
        Ok(index)
    }
    pub fn save_to_bytes(&self) -> Result<Vec<u8>, String> {
        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        let nodes_count = self.nodes.count();
        let mut snapshot_nodes = Vec::with_capacity(nodes_count);
        for (_, node) in &self.nodes {
            snapshot_nodes.push(SnapshotNode {
                id: node.id,
                layers: self.snapshot_layers(node),
            });
        }

        let mut inverted_vec = Vec::new();
        for item in &self.metadata.inverted {
            let mut buf = Vec::new();
            item.value()
                .serialize_into(&mut buf)
                .map_err(|e| e.to_string())?;
            inverted_vec.push((item.key().clone(), buf));
        }

        // The numeric range index is derived data: it is rebuilt from `forward` on load
        // (order-preserving f64 keys), so the legacy i64 section is written empty.
        let numeric_vec: Vec<(String, KeyedBitmaps)> = Vec::new();

        let mut deleted_buf = Vec::new();
        self.metadata
            .deleted
            .read()
            .serialize_into(&mut deleted_buf)
            .map_err(|e| e.to_string())?;

        let mut forward_vec = Vec::new();
        for item in &self.metadata.forward {
            let mut map_vec = Vec::new();
            for (k, v) in item.value() {
                map_vec.push((k.clone(), v.clone()));
            }
            forward_vec.push((*item.key(), map_vec));
        }

        let snapshot = SnapshotData {
            max_layer,
            entry_point,
            nodes: snapshot_nodes,
            metadata: SnapshotMetadata {
                inverted: inverted_vec,
                numeric: numeric_vec,
                deleted: deleted_buf,
                forward: forward_vec,
            },
        };

        let bytes = rkyv::to_bytes::<_, 1024>(&snapshot)
            .map_err(|e| format!("Serialization error: {e}"))?;

        Ok(bytes.into_vec())
    }

    pub fn load_from_bytes(
        data: &[u8],
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        dimension: usize,
    ) -> Result<Self, String> {
        let archived = unsafe { rkyv::archived_root::<SnapshotData>(data) };

        let deserialized: SnapshotData = archived
            .deserialize(&mut rkyv::Infallible)
            .map_err(|e| format!("Deserialization error: {e}"))?;

        let nodes_bc: boxcar::Vec<Node> = boxcar::Vec::with_capacity(deserialized.nodes.len());
        let mut layer0_lists: Vec<Vec<NodeId>> = Vec::with_capacity(deserialized.nodes.len());
        for s_node in deserialized.nodes {
            let mut s_layers = s_node.layers.into_iter();
            layer0_lists.push(s_layers.next().unwrap_or_default());
            nodes_bc.push(Node {
                id: s_node.id,
                upper: s_layers
                    .map(|l| RwLock::new(SmallVec::from_vec(l)))
                    .collect(),
            });
        }
        let layer0 = Self::build_layer0(&config, layer0_lists);

        storage.set_count(nodes_bc.count());

        let inverted = DashMap::new();
        let prefix_index: DashMap<String, crossbeam_skiplist::SkipSet<String>> = DashMap::new();
        for (k, v) in deserialized.metadata.inverted {
            let bitmap = RoaringBitmap::deserialize_from(&v[..]).unwrap_or_default();
            if let Some((field, val)) = k.split_once(':') {
                prefix_index
                    .entry(field.to_string())
                    .or_default()
                    .insert(val.to_string());
            }
            inverted.insert(k, bitmap);
        }

        // Legacy i64 numeric sections are ignored; the f64 range index is rebuilt from `forward`.
        let numeric = DashMap::new();

        let deleted =
            RoaringBitmap::deserialize_from(&deserialized.metadata.deleted[..]).unwrap_or_default();

        let forward = DashMap::new();
        let mut has_nonempty_metadata = false;
        for (k, v) in deserialized.metadata.forward {
            let mut attributes = std::collections::HashMap::new();
            for (mk, mv) in v {
                attributes.insert(mk, mv);
            }
            if !attributes.is_empty() {
                has_nonempty_metadata = true;
            }
            forward.insert(k, attributes);
        }

        let fast_routing = std::env::var("HS_FAST_ROUTING")
            .is_ok_and(|v| v.to_lowercase() == "true")
            && matches!(M::name(), "poincare" | "lorentz" | "hybrid")
            && matches!(mode, QuantizationMode::None);

        let density_pruning =
            std::env::var("HS_DENSITY_PRUNING").is_ok_and(|v| v.to_lowercase() == "true");
        let zonal =
            std::env::var("HS_ZONAL_QUANTIZATION").is_ok_and(|v| v.to_lowercase() == "true");

        let node_count = storage.count();
        let pq = match mode {
            QuantizationMode::ProductQuantization => {
                let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                    dimension, num_sub, false,
                )))
            }
            QuantizationMode::OPQ => {
                let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                    dimension, num_sub, true,
                )))
            }
            _ => None,
        };

        let (rabitq, pq4) = Self::code_quantizers(mode, dimension, None);
        let has_del = !deleted.is_empty();
        let index = Self {
            nodes: nodes_bc,
            layer0,
            append_lock: Mutex::new(()),
            metadata: MetadataIndex {
                inverted,
                prefix_index,
                numeric,
                deleted: RwLock::new(deleted),
                has_deletions: AtomicBool::new(has_del),
                forward,
                token_df: DashMap::new(),
                doc_token_len: boxcar::Vec::new(),
                term_doc_freq: DashMap::new(),
                total_token_len: AtomicU64::new(0),
                has_lexical_index: AtomicBool::new(false),
            },
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            storage_f32: false,
            config,
            has_nonempty_metadata: AtomicBool::new(has_nonempty_metadata),
            fast_routing,
            density_pruning,
            zonal,
            zonal_storage: dashmap::DashMap::new(),
            node_counter: AtomicU32::new(node_count as u32),
            spatial_index: Arc::new(spatial_aabb::SpatialAabbIndex::new(dimension)),
            pq: arc_swap::ArcSwapOption::from(pq),
            flat_l0: arc_swap::ArcSwapOption::empty(),
            deletes_since_repair: AtomicUsize::new(0),
            rabitq: arc_swap::ArcSwapOption::from(rabitq),
            pq4: arc_swap::ArcSwapOption::from(pq4),
            // A byte snapshot carries no calibration marker; never re-calibrate it online.
            calibrated: AtomicBool::new(true),
            calibration_gate: RwLock::new(()),
            calibrated_at: AtomicUsize::new(0),
            tuned_ef: AtomicUsize::new(0),
            tuned_at: AtomicUsize::new(0),
            query_samples: Mutex::new(std::collections::VecDeque::new()),
            queries_seen: AtomicUsize::new(0),
            tuned_queries_seen: AtomicUsize::new(usize::MAX),
            tuned_at_ms: AtomicU64::new(0),
            // Byte snapshots are produced and read by the same build (wasm).
            sq8_scaled: Self::sq8_scaled_supported(mode),
            repair_running: AtomicBool::new(false),
            graph_engine: None,
            _marker: PhantomData,
            dimension,
        };
        index.rebuild_lexical_stats();
        index.rebuild_numeric_index();
        Ok(index)
    }

    pub fn get_storage(&self) -> Arc<VectorStore> {
        self.storage.clone()
    }
}

/// Node Identifier (index in VectorStore)
pub type NodeId = u32;

const MAX_LAYERS: usize = 16;

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct HnswIndex<M: Metric> {
    // Topology storage. Lock-free concurrent Vec: index == NodeId.
    // Node structs (with their inner per-layer RwLocks) are created in
    // insert_to_storage() where IDs are sequential, then graph links are
    // wired in index_node(). Reads are lock-free O(1).
    nodes: boxcar::Vec<Node>,

    // Contiguous seqlocked layer-0 adjacency (index == NodeId).
    layer0: layer0::Layer0Arena,

    // Guard for sequential pushes in both VectorStore and boxcar
    append_lock: Mutex<()>,

    // Metadata storage
    pub metadata: MetadataIndex,

    // Graph entry point (top level)
    entry_point: AtomicU32,

    // Current max layer
    max_layer: AtomicU32,

    // Reference to data (raw vectors)
    storage: Arc<VectorStore>,

    // Quantization
    pub mode: QuantizationMode,
    // If true and mode=None, vectors are stored as f32 in mmap.
    storage_f32: bool,

    // Runtime configuration
    pub config: Arc<GlobalConfig>,
    has_nonempty_metadata: AtomicBool,

    // Fast multi-geometric routing (Klein Euclidean chords on upper layers)
    pub fast_routing: bool,
    pub dimension: usize,

    // Task 6.4: Density-based HNSW graph pruning in syntactic voids
    pub density_pruning: bool,

    // Task 6.3: Zonal storage to compress items and bypass mmap
    pub zonal: bool,
    pub zonal_storage: dashmap::DashMap<NodeId, hyperspace_core::vector::ZonalVector>,
    pub node_counter: AtomicU32,
    pub spatial_index: Arc<spatial_aabb::SpatialAabbIndex>,
    // Trained quantizers. Swappable so online auto-calibration can replace a default
    // codebook while the index keeps serving reads (see `calibrate_online`).
    pub pq: arc_swap::ArcSwapOption<hyperspace_core::pq::ProductQuantizer>,
    pub rabitq: arc_swap::ArcSwapOption<hyperspace_core::rabitq::RaBitQ>,
    pub pq4: arc_swap::ArcSwapOption<hyperspace_core::pq4::Pq4Quantizer>,
    /// `true` once a trained mode's quantizer was fitted to data (always `true` for
    /// modes without training).
    calibrated: AtomicBool,
    /// Vectors stored when the quantizer was last fitted online (0 = never / unknown).
    calibrated_at: AtomicUsize,
    /// Beam width found by [`Self::tune_ef_search`] (0 = not tuned) and the live node count
    /// it was measured at.
    tuned_ef: AtomicUsize,
    tuned_at: AtomicUsize,
    /// Every `QUERY_SAMPLE_EVERY`-th unfiltered search query (last `QUERY_SAMPLES`), used by
    /// [`Self::tune_ef_search`] once enough are collected.
    query_samples: Mutex<std::collections::VecDeque<Vec<f64>>>,
    queries_seen: AtomicUsize,
    /// `queries_seen` at the last tuning that used real queries (`usize::MAX` = none yet).
    tuned_queries_seen: AtomicUsize,
    /// Milliseconds since process start at the last tuning (rate limit for re-measuring on
    /// new queries; growth-triggered re-measurements ignore it).
    tuned_at_ms: AtomicU64,
    /// Held shared while a vector is encoded and appended / updated, exclusively while
    /// the quantizer is swapped, so no code is written with a stale codebook after the
    /// swap without being re-encoded.
    calibration_gate: RwLock<()>,
    /// ScalarI8 codes carry a per-vector scale in `alpha` (see [`SQ8_SCALED_MARKER`]).
    sq8_scaled: bool,
    pub flat_l0: arc_swap::ArcSwapOption<FlatLayer0Segment>,
    /// Deletes recorded since the last graph repair (see [`HnswIndex::repair_deleted`]).
    deletes_since_repair: AtomicUsize,
    repair_running: AtomicBool,
    pub graph_engine: Option<Arc<hyperspace_core::graph::GraphEngine>>,

    _marker: PhantomData<M>,
}

#[inline(always)]
fn prefetch_l1<T>(ptr: *const T) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_prefetch(ptr.cast::<i8>(), core::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!("prfm pldl1keep, [{0}]", in(reg) ptr, options(nostack, readonly));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = ptr;
    }
}

/// Prefetches the leading cache lines of a stored vector (up to 256 bytes / 4 lines).
/// A 768-D `f32` vector spans 48 lines; touching the first few lets the hardware
/// streamer pick up the rest, instead of stalling on the first line only.
#[inline(always)]
fn prefetch_bytes(bytes: &[u8]) {
    const LINE: usize = 64;
    const MAX_LINES: usize = 4;
    let ptr = bytes.as_ptr();
    let lines = bytes.len().div_ceil(LINE).clamp(1, MAX_LINES);
    for i in 0..lines {
        prefetch_l1(ptr.wrapping_add(i * LINE));
    }
}

/// Upper-layer (level >= 1) neighbor list; layer-0 lists live in [`layer0::Layer0Arena`].
type UpperLinks = RwLock<SmallVec<[NodeId; 16]>>;

#[derive(Debug)]
struct Node {
    id: NodeId,
    /// `upper[l - 1]` = neighbors on level `l`. Empty (no allocation) for level-0 nodes,
    /// which are `1 - 1/M` of the graph (~94% with `M = 16`).
    upper: Box<[UpperLinks]>,
}

impl Node {
    fn new(id: NodeId, level: usize) -> Self {
        Self {
            id,
            upper: (0..level).map(|_| RwLock::new(SmallVec::new())).collect(),
        }
    }

    /// Top level this node participates in (0 = layer-0 only).
    #[inline(always)]
    fn level(&self) -> usize {
        self.upper.len()
    }
}

#[derive(Default)]
struct VisitedScratch {
    epochs: Vec<u16>,
    epoch: u16,
    candidates_l0: BinaryHeap<Candidate>,
    results_l0: BinaryHeap<std::cmp::Reverse<Candidate>>,
    candidates_layer: BinaryHeap<Candidate>,
    results_layer: BinaryHeap<std::cmp::Reverse<Candidate>>,
    query_f32: Vec<f32>,
}

impl VisitedScratch {
    #[inline]
    fn prepare(&mut self, len: usize) {
        if self.epochs.len() < len {
            self.epochs.resize(len, 0);
        }
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.epochs.fill(0);
            self.epoch = 1;
        }
    }

    #[inline(always)]
    fn mark_visited(&mut self, id: u32) -> bool {
        let idx = id as usize;
        if idx < self.epochs.len() {
            let slot = &mut self.epochs[idx];
            if *slot == self.epoch {
                false
            } else {
                *slot = self.epoch;
                true
            }
        } else {
            false
        }
    }
}

#[derive(Default)]
struct SelectScratch {
    result_coords: Vec<f32>,
    result_alphas: Vec<f64>,
    pruned: Vec<NodeId>,
}

thread_local! {
    static VISITED_SCRATCH: RefCell<VisitedScratch> = RefCell::new(VisitedScratch::default());
    static QUERY_F32_SCRATCH: RefCell<Vec<f32>> = const { RefCell::new(Vec::new()) };
    static QUERY_F64_SCRATCH: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
    static SELECT_SCRATCH: RefCell<SelectScratch> = RefCell::new(SelectScratch::default());
    static BYTES_SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Maps an `f32` onto a `u32` whose unsigned ordering equals the numeric ordering
/// (works for negative distances such as `1 - dot` on unnormalized vectors).
#[inline(always)]
fn ordered_f32_key(d: f32) -> u32 {
    let bits = d.to_bits();
    if bits & 0x8000_0000 == 0 {
        bits | 0x8000_0000
    } else {
        !bits
    }
}

/// 8-byte packed nearest-neighbor candidate `{ id: u32, distance: f32 }`.
///
/// Heap comparisons use a single `u64` compare of `(ordered_f32_key(distance) << 32) | id`,
/// which is correct for negative distances as well. Distances may be stored with a
/// `bias` subtracted (see [`HnswIndex::dist_bias`]): hyperbolic metrics produce
/// `cosh(d) ≈ 1 + d²/2`, and storing `cosh(d) - 1` keeps full `f32` relative precision
/// for close neighbors instead of collapsing them onto `1.0f32`.
#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, PartialEq)]
struct Candidate {
    id: NodeId,
    distance: f32,
}

impl Candidate {
    #[inline(always)]
    fn biased(id: NodeId, distance: f64, bias: f64) -> Self {
        let d32 = (distance - bias) as f32;
        Self {
            id,
            distance: if d32.is_nan() { f32::MAX } else { d32 },
        }
    }

    #[cfg(test)]
    #[inline(always)]
    fn new(id: NodeId, distance: f64) -> Self {
        Self::biased(id, distance, 0.0)
    }

    #[inline(always)]
    fn id(self) -> NodeId {
        self.id
    }

    /// Returns the distance with `bias` added back.
    #[inline(always)]
    fn unbiased(self, bias: f64) -> f64 {
        f64::from(self.distance) + bias
    }

    #[inline(always)]
    fn as_u64(self) -> u64 {
        (u64::from(ordered_f32_key(self.distance)) << 32) | u64::from(self.id)
    }
}

impl Eq for Candidate {}
impl Ord for Candidate {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> CmpOrdering {
        other.as_u64().cmp(&self.as_u64())
    }
}
impl PartialOrd for Candidate {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

#[repr(C, align(8))]
#[derive(Debug, Copy, Clone, PartialEq)]
struct MaxCandidate {
    id: NodeId,
    distance: f32,
}

impl MaxCandidate {
    #[inline(always)]
    fn biased(id: NodeId, distance: f64, bias: f64) -> Self {
        let d32 = (distance - bias) as f32;
        Self {
            id,
            distance: if d32.is_nan() { f32::MAX } else { d32 },
        }
    }

    #[inline(always)]
    fn id(self) -> NodeId {
        self.id
    }

    #[inline(always)]
    fn unbiased(self, bias: f64) -> f64 {
        f64::from(self.distance) + bias
    }

    #[inline(always)]
    fn as_u64(self) -> u64 {
        (u64::from(ordered_f32_key(self.distance)) << 32) | u64::from(self.id)
    }
}

impl Eq for MaxCandidate {}
impl Ord for MaxCandidate {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.as_u64().cmp(&other.as_u64())
    }
}
impl PartialOrd for MaxCandidate {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

/// Process-wide override of the flat-scan node limit (`-1` = `HS_FLAT_SCAN` / auto).
static FLAT_SCAN_OVERRIDE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);

/// Forces the flat-scan node limit (`Some(0)` = always walk the graph, `None` = default).
/// For A/B tests and benchmarks.
#[doc(hidden)]
pub fn set_flat_scan_limit(limit: Option<usize>) {
    FLAT_SCAN_OVERRIDE.store(
        limit.map_or(-1, |l| i64::try_from(l).unwrap_or(i64::MAX)),
        Ordering::Relaxed,
    );
}

/// Global switch for the per-query prepared kernels of `extreme`, `ternary` (nibble
/// tables) and `medium_plus` (de-interleaved ScalarI4), on by default. Only for A/B tests
/// and benchmarks: results are the same either way, the prepared kernels are just faster.
static BIT_LUT_KERNELS: AtomicBool = AtomicBool::new(true);

/// Enables / disables the `extreme` / `ternary` nibble-table kernels process-wide.
#[doc(hidden)]
pub fn set_bit_lut_kernels(enabled: bool) {
    BIT_LUT_KERNELS.store(enabled, Ordering::Relaxed);
}

/// Integer (SDOT / VNNI-style) kernel for per-vector scaled ScalarI8 codes, on by default.
/// Off falls back to the `f32`-query kernel over the same codes (A/B tests only).
static SQ8_INT_KERNEL: AtomicBool = AtomicBool::new(true);

/// Whether *new* ScalarI8 L2 / cosine indexes use per-vector scaled codes (default) or the
/// legacy fixed `x · 127` codes. Existing indexes keep the format they were written with.
static SQ8_SCALED_CODES: AtomicBool = AtomicBool::new(true);

/// Enables / disables the integer ScalarI8 kernel process-wide.
#[doc(hidden)]
pub fn set_sq8_int_kernel(enabled: bool) {
    SQ8_INT_KERNEL.store(enabled, Ordering::Relaxed);
}

/// Chooses the ScalarI8 code format for indexes created afterwards (A/B tests only).
#[doc(hidden)]
pub fn set_sq8_scaled_codes(enabled: bool) {
    SQ8_SCALED_CODES.store(enabled, Ordering::Relaxed);
}

/// One in `QUERY_SAMPLE_EVERY` searches is kept for beam tuning, up to `QUERY_SAMPLES`.
const QUERY_SAMPLE_EVERY: usize = 8;
const QUERY_SAMPLES: usize = 64;
/// Stored vectors averaged into one synthetic tuning query (a local centroid).
const TUNE_NEIGHBOURHOOD: usize = 5;
/// Real queries needed before tuning uses them instead of synthetic ones.
const MIN_REAL_QUERIES: usize = 16;

/// Milliseconds since the first call (monotonic, never 0).
fn process_millis() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    let start = *START.get_or_init(std::time::Instant::now);
    u64::try_from(start.elapsed().as_millis())
        .unwrap_or(u64::MAX)
        .max(1)
}

/// Index searches in flight process-wide (drives [`flat_scan_parallelism`]).
static SEARCHES_ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// Rows per parallel flat-scan part (smaller parts cost more in scheduling than they save).
const FLAT_PART_ROWS: usize = 1024;

/// Decrements [`SEARCHES_ACTIVE`] on drop.
struct ActiveSearch;

impl ActiveSearch {
    fn enter() -> Self {
        SEARCHES_ACTIVE.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for ActiveSearch {
    fn drop(&mut self) {
        SEARCHES_ACTIVE.fetch_sub(1, Ordering::Relaxed);
    }
}

/// Parts a flat scan of `rows` is split into: one per idle core (searches in flight leave
/// the rest busy), at least `FLAT_PART_ROWS` rows each; 1 = scan on the calling thread.
fn flat_scan_parallelism(rows: usize) -> usize {
    parallel_parts(rows, FLAT_PART_ROWS)
}

/// Parts (≥ `min_rows` each) that per-query work over `rows` items can be split into on
/// idle cores: 1 when at least half the cores already run searches (the rayon pool would
/// only add scheduling overhead then). Used by flat scans and large reranks.
#[must_use]
pub fn parallel_parts(rows: usize, min_rows: usize) -> usize {
    let cores = rayon::current_num_threads().max(1);
    let active = SEARCHES_ACTIVE.load(Ordering::Relaxed).max(1);
    if active * 2 > cores {
        return 1;
    }
    (cores / active).min(rows / min_rows.max(1)).max(1)
}

/// Whether a search may pick the exact flat scan (`Auto`) or is forced to one path.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    Auto,
    Graph,
    Flat,
}

/// Nibble-table kernel family for bit-packed quantizers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BitLutKind {
    Binary,
    TernaryL2,
    TernaryCos,
}

/// Distance kernel resolved **once per query** from (quantization mode, metric, storage
/// precision). Hot loops dispatch on this small loop-invariant enum instead of re-walking
/// the full `mode × metric` decision tree in [`HnswIndex::dist_bytes_opt`] per candidate,
/// which lets LLVM unswitch the loop and inline the SIMD kernel.
#[derive(Clone, Copy)]
enum DistKernel<'a> {
    /// `f32` storage, squared L2 (also cosine on normalized vectors).
    F32L2(&'a [f32]),
    /// `f32` storage, Poincaré `cosh(d)` via stored conformal factor.
    F32Poincare(&'a [f32], f64),
    /// `f32` storage, Lorentz `cosh(d)` (difference identity).
    F32Lorentz(&'a [f32]),
    /// `f32` storage, squared hybrid product distance.
    F32HybridSq(&'a [f32]),
    /// `f32` storage, `1 - dot`.
    F32Dot(&'a [f32]),
    /// `ScalarI8` storage, squared L2 against an `f32` query.
    Sq8L2(&'a [f32]),
    /// Legacy `f64` storage, squared L2.
    F64L2(&'a [f64]),
    /// Legacy `f64` storage, Lorentz `cosh(d)`.
    F64Lorentz(&'a [f64]),
    /// RaBitQ codes (popcount estimator).
    RaBitQ(&'a hyperspace_core::rabitq::RaBitQQuery),
    /// PQ4 codes (L1-resident u8 LUT).
    Pq4(&'a hyperspace_core::pq4::Pq4Lut),
    /// 1-bit `extreme` codes, ADC through a per-query nibble table.
    BinaryLut(&'a hyperspace_core::vector::BitPlaneLut),
    /// Ternary codes, squared L2 through a per-query nibble table.
    TernaryL2Lut(&'a hyperspace_core::vector::BitPlaneLut),
    /// Ternary codes, cosine through a per-query nibble table.
    TernaryCosLut(&'a hyperspace_core::vector::BitPlaneLut),
    /// ScalarI4 codes, squared L2 against a de-interleaved query (NEON on aarch64).
    I4L2(&'a [f32], &'a [f32], f64),
    /// Per-vector scaled ScalarI8 codes, integer (SDOT) squared L2.
    Sq8Int(&'a hyperspace_core::simd::Sq8Query),
    /// Everything else: full `dist_bytes_opt` dispatch.
    Generic,
}

/// Per-query precomputation for code-based (ADC) quantization modes.
#[derive(Debug, Clone)]
pub enum QueryPrep {
    /// 8-bit PQ / OPQ lookup table.
    Pq(hyperspace_core::pq::PQLookupTable),
    /// RaBitQ rotated, bit-plane quantized query.
    RaBitQ(hyperspace_core::rabitq::RaBitQQuery),
    /// 4-bit PQ fast-scan table.
    Pq4(hyperspace_core::pq4::Pq4Lut),
    /// Nibble table for bit-packed codes (`extreme` 1-bit ADC, ternary).
    Bits(hyperspace_core::vector::BitPlaneLut),
    /// Int8-quantized query for per-vector scaled ScalarI8 (`medium`) on L2 / cosine.
    Sq8(hyperspace_core::simd::Sq8Query),
    /// De-interleaved `f32` query for ScalarI4 (`medium_plus`) squared L2.
    I4 {
        even: Vec<f32>,
        odd: Vec<f32>,
        norm_sq: f64,
    },
}

/// Per-query layer-0 traversal policy.
#[derive(Clone, Copy, Default)]
struct TraversalPolicy {
    /// ACORN-style filtered traversal: filtered-out neighbors are never scored; instead
    /// their own neighbors (2-hop) are explored, keeping the predicate subgraph connected
    /// while spending distance evaluations only on matching nodes.
    acorn: bool,
    /// Adaptive-ef early termination: stop after this many consecutive expansions that
    /// did not improve the result set (0 = disabled, classic HNSW termination only).
    patience: usize,
}

/// Unified precomputed query context for HNSW upper-layer routing and Layer-0 evaluation.
#[derive(Copy, Clone)]
struct QueryContext<'a> {
    query: &'a HyperVector,
    query_f32: Option<&'a [f32]>,
    query_klein: Option<&'a HyperVector>,
    query_klein_f32: Option<&'a [f32]>,
    pq_lut: Option<&'a QueryPrep>,
    mrl_dim: Option<usize>,
}

impl<M: Metric> HnswIndex<M> {
    pub fn new(
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        dimension: usize,
    ) -> Self {
        Self::new_with_storage_precision(storage, mode, config, false, dimension)
    }

    pub fn new_with_storage_precision(
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        storage_f32: bool,
        dimension: usize,
    ) -> Self {
        let fast_routing = std::env::var("HS_FAST_ROUTING")
            .is_ok_and(|v| v.to_lowercase() == "true")
            && matches!(M::name(), "poincare" | "lorentz" | "hybrid")
            && matches!(mode, QuantizationMode::None);

        let density_pruning =
            std::env::var("HS_DENSITY_PRUNING").is_ok_and(|v| v.to_lowercase() == "true");
        let zonal =
            std::env::var("HS_ZONAL_QUANTIZATION").is_ok_and(|v| v.to_lowercase() == "true");

        let pq = match mode {
            QuantizationMode::ProductQuantization => {
                let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                    dimension, num_sub, false,
                )))
            }
            QuantizationMode::OPQ => {
                let num_sub = hyperspace_core::pq::default_num_subvectors(dimension);
                Some(Arc::new(hyperspace_core::pq::ProductQuantizer::new(
                    dimension, num_sub, true,
                )))
            }
            _ => None,
        };

        let layer0 = layer0::Layer0Arena::new(Self::layer0_capacity(&config, 0));
        let (rabitq, pq4) = Self::code_quantizers(mode, dimension, None);
        Self {
            nodes: boxcar::Vec::new(),
            layer0,
            dimension,
            append_lock: Mutex::new(()),
            metadata: MetadataIndex::default(),
            entry_point: AtomicU32::new(0),
            max_layer: AtomicU32::new(0),
            storage,
            mode,
            storage_f32,
            config,
            has_nonempty_metadata: AtomicBool::new(false),
            fast_routing,
            density_pruning,
            zonal,
            zonal_storage: dashmap::DashMap::new(),
            node_counter: AtomicU32::new(0),
            spatial_index: Arc::new(spatial_aabb::SpatialAabbIndex::new(dimension)),
            pq: arc_swap::ArcSwapOption::from(pq),
            flat_l0: arc_swap::ArcSwapOption::empty(),
            deletes_since_repair: AtomicUsize::new(0),
            rabitq: arc_swap::ArcSwapOption::from(rabitq),
            pq4: arc_swap::ArcSwapOption::from(pq4),
            calibrated: AtomicBool::new(!mode.needs_training()),
            calibration_gate: RwLock::new(()),
            calibrated_at: AtomicUsize::new(0),
            tuned_ef: AtomicUsize::new(0),
            tuned_at: AtomicUsize::new(0),
            query_samples: Mutex::new(std::collections::VecDeque::new()),
            queries_seen: AtomicUsize::new(0),
            tuned_queries_seen: AtomicUsize::new(usize::MAX),
            tuned_at_ms: AtomicU64::new(0),
            sq8_scaled: Self::sq8_scaled_supported(mode)
                && SQ8_SCALED_CODES.load(Ordering::Relaxed),
            repair_running: AtomicBool::new(false),
            graph_engine: None,
            _marker: PhantomData,
        }
    }

    /// Attaches a Knowledge-Augmented Graph (`GraphEngine`) for single-pass KAG-guided HNSW traversal.
    pub fn set_graph_engine(&mut self, graph: Arc<hyperspace_core::graph::GraphEngine>) {
        self.graph_engine = Some(graph);
    }

    /// Builder-style attachment of a Knowledge-Augmented Graph (`GraphEngine`).
    #[must_use]
    pub fn with_graph_engine(mut self, graph: Arc<hyperspace_core::graph::GraphEngine>) -> Self {
        self.graph_engine = Some(graph);
        self
    }

    #[inline]
    fn compute_query_klein(&self, q_vec: &HyperVector) -> Option<HyperVector> {
        if !self.fast_routing || self.storage_f32 {
            return None;
        }
        match M::name() {
            "poincare" => Some(q_vec.to_klein()),
            "lorentz" => Some(q_vec.lorentz_to_klein()),
            "hybrid" => Some(q_vec.hybrid_to_klein()),
            _ => None,
        }
    }

    #[inline]
    fn compute_query_klein_f32(&self, q_vec: &HyperVector) -> Option<SmallVec<[f32; 256]>> {
        if !self.fast_routing || !self.storage_f32 {
            return None;
        }
        match M::name() {
            "poincare" => {
                let sq_norm = 1.0 - (1.0 / q_vec.alpha);
                let factor = (2.0 / (1.0 + sq_norm)) as f32;
                Some(q_vec.coords.iter().map(|&p| (p as f32) * factor).collect())
            }
            "lorentz" => {
                if q_vec.coords.is_empty() {
                    return None;
                }
                let inv_t = (1.0 / q_vec.coords[0].max(1.0)) as f32;
                let mut out = SmallVec::with_capacity(q_vec.coords.len());
                out.push(1.0);
                for &x in &q_vec.coords[1..] {
                    out.push((x as f32) * inv_t);
                }
                Some(out)
            }
            "hybrid" => {
                if q_vec.coords.len() < 33 {
                    return None;
                }
                let inv_t = (1.0 / q_vec.coords[0].max(1.0)) as f32;
                let mut out = SmallVec::with_capacity(33);
                out.push(1.0);
                for &x in &q_vec.coords[1..33] {
                    out.push((x as f32) * inv_t);
                }
                Some(out)
            }
            _ => None,
        }
    }

    /// Fine-tune the Product Quantizer codebooks on training samples.
    ///
    /// Also fits the RaBitQ centroid and trains the PQ4 codebook for those modes.
    /// Fits the trained quantizer to `samples` before any vector is stored (vacuum /
    /// rebuild path). For an index that already holds vectors use [`Self::calibrate_online`].
    pub fn train_pq(&mut self, samples: &[&[f32]], iterations: usize) {
        self.fit_quantizer(samples, iterations);
        self.calibrated.store(true, Ordering::Release);
    }

    /// Trains a copy of the current quantizer and swaps it in.
    fn fit_quantizer(&self, samples: &[&[f32]], iterations: usize) {
        if samples.is_empty() {
            return;
        }
        if let Some(pq) = self.pq.load_full() {
            let mut new_pq = (*pq).clone();
            new_pq.train(samples, iterations);
            self.pq.store(Some(Arc::new(new_pq)));
        }
        if let Some(rq) = self.rabitq.load_full() {
            let mut new_rq = (*rq).clone();
            new_rq.fit_centroid(samples);
            self.rabitq.store(Some(Arc::new(new_rq)));
        }
        if let Some(pq4) = self.pq4.load_full() {
            let mut new_pq4 = (*pq4).clone();
            new_pq4.train(samples, iterations);
            self.pq4.store(Some(Arc::new(new_pq4)));
        }
    }

    /// `true` for a trained mode (PQ / OPQ / PQ4 / RaBitQ) whose quantizer still uses
    /// the data-independent default codebook.
    #[inline]
    pub fn needs_calibration(&self) -> bool {
        self.mode.needs_training() && !self.calibrated.load(Ordering::Acquire)
    }

    #[inline]
    pub fn is_calibrated(&self) -> bool {
        self.calibrated.load(Ordering::Acquire)
    }

    /// Online auto-calibration of a trained quantizer on a live index.
    ///
    /// Trains on `samples`, swaps the quantizer while holding the calibration gate
    /// exclusively (so every vector stored afterwards is encoded with the new codebook),
    /// then re-encodes the vectors stored before the swap from their exact values
    /// (`raw(id)`, which must return the same pre-processed vector that was passed to
    /// `insert_to_storage`). Vectors without an exact copy keep their old code.
    /// Returns `(re_encoded, missing_raw)`.
    pub fn calibrate_online<F>(
        &self,
        samples: &[&[f32]],
        iterations: usize,
        raw: F,
    ) -> (usize, usize)
    where
        F: Fn(u32) -> Option<Vec<f32>> + Sync,
    {
        if !self.needs_calibration() {
            return (0, 0);
        }
        self.refit_online(samples, iterations, raw)
    }

    /// Vectors stored when the quantizer was last fitted online (0 if never).
    #[inline]
    pub fn calibrated_at(&self) -> usize {
        self.calibrated_at.load(Ordering::Acquire)
    }

    /// Re-fits an already calibrated quantizer on a larger sample (progressive
    /// calibration) with the same swap-and-re-encode protocol as
    /// [`Self::calibrate_online`]. No-op for modes without training.
    pub fn refit_online<F>(&self, samples: &[&[f32]], iterations: usize, raw: F) -> (usize, usize)
    where
        F: Fn(u32) -> Option<Vec<f32>> + Sync,
    {
        use rayon::prelude::*;
        if !self.mode.needs_training() || samples.is_empty() {
            return (0, 0);
        }
        let stored_before_swap = {
            let _g = self.calibration_gate.write();
            self.fit_quantizer(samples, iterations);
            self.calibrated.store(true, Ordering::Release);
            let n = self.storage.count();
            self.calibrated_at.store(n.max(1), Ordering::Release);
            n
        };
        // Re-encode on all cores: each code is independent and the storage slots already
        // exist. Small blocks keep the shared gate's read sections short so a concurrent
        // swap is never starved.
        (0..stored_before_swap as u32)
            .into_par_iter()
            .with_min_len(256)
            .map(|id| match raw(id) {
                Some(v) => {
                    let code = {
                        let _g = self.calibration_gate.read();
                        self.encode_code_f32(&v)
                    };
                    let ok = !code.is_empty() && self.storage.update(id, &code).is_ok();
                    (usize::from(ok), 0usize)
                }
                None => (0, 1),
            })
            .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
    }

    /// Encodes a vector with the current trained quantizer (PQ / OPQ / RaBitQ / PQ4).
    fn encode_code_f32(&self, v: &[f32]) -> Vec<u8> {
        if let Some(ref pq) = *self.pq.load() {
            pq.encode(v).to_bytes()
        } else if let Some(ref rq) = *self.rabitq.load() {
            rq.encode(v)
        } else if let Some(ref pq4) = *self.pq4.load() {
            pq4.encode(v)
        } else {
            Vec::new()
        }
    }

    /// Encodes a vector for the RaBitQ / PQ4 code modes.
    fn encode_code(&self, coords: &[f64]) -> Vec<u8> {
        let v: Vec<f32> = coords.iter().map(|&x| x as f32).collect();
        self.encode_code_f32(&v)
    }

    #[inline]
    pub fn has_nonempty_metadata(&self) -> bool {
        self.has_nonempty_metadata.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.storage.count()
    }

    fn build_allowed_bitmap(
        &self,
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
    ) -> Option<RoaringBitmap> {
        fn apply_mask(bitmap: &mut Option<RoaringBitmap>, mask: &RoaringBitmap) {
            if let Some(ref mut bm) = bitmap {
                *bm &= mask;
            } else {
                *bitmap = Some(mask.clone());
            }
        }

        // PERF: Only clone the deleted bitmap when geometric filters are present.
        // Geometric filters do an O(self.dimension) scan and hold no lock during it (snapshot approach).
        // For plain metadata/range filters we keep the read guard alive (cheap shared ptr).
        // Cloning on every unfiltered search was a regression from the previous fix.
        let has_geometric = complex_filters.iter().any(|f| {
            matches!(
                f,
                FilterExpr::InBall { .. }
                    | FilterExpr::InBox { .. }
                    | FilterExpr::InCone { .. }
                    | FilterExpr::And(_)
                    | FilterExpr::Or(_)
                    | FilterExpr::Not(_)
            )
        });
        // If geometric: snapshot + release lock immediately to unblock concurrent deletes.
        // If not: hold the read guard for the duration (zero-copy, just an atomic ref-count).
        let (deleted_owned, deleted_guard) = if has_geometric {
            (Some(self.metadata.deleted.read().clone()), None)
        } else {
            (None, Some(self.metadata.deleted.read()))
        };
        let deleted = if let Some(ref owned) = deleted_owned {
            owned
        } else {
            deleted_guard.as_deref().unwrap()
        };

        let mut bitmap: Option<RoaringBitmap> = None;

        if !filter.is_empty() {
            for (key, val) in filter {
                let tag = format!("{key}:{val}");
                if let Some(tag_bitmap) = self.metadata.inverted.get(&tag) {
                    apply_mask(&mut bitmap, &tag_bitmap);
                } else {
                    return Some(RoaringBitmap::new());
                }
            }
        }

        for expr in complex_filters {
            match expr {
                FilterExpr::Match { key, value } => {
                    let tag = format!("{key}:{value}");
                    if let Some(tag_bitmap) = self.metadata.inverted.get(&tag) {
                        apply_mask(&mut bitmap, &tag_bitmap);
                    } else {
                        return Some(RoaringBitmap::new());
                    }
                }
                FilterExpr::Prefix { key, prefix } => {
                    let mut prefix_union = RoaringBitmap::new();
                    if let Some(set) = self.metadata.prefix_index.get(key) {
                        for val in set.range(prefix.clone()..) {
                            if !val.value().starts_with(prefix) {
                                break;
                            }
                            let tag = format!("{key}:{}", val.value());
                            if let Some(tag_bitmap) = self.metadata.inverted.get(&tag) {
                                prefix_union |= &*tag_bitmap;
                            }
                        }
                    }
                    if prefix_union.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &prefix_union);
                }
                FilterExpr::Range { key, gte, lte } => {
                    // Exact f64 range scan over the order-preserving numeric index:
                    // O(log N + |matches|), no per-document parsing at query time.
                    let mut range_union = RoaringBitmap::new();
                    if let Some(tree) = self.metadata.numeric.get(key) {
                        let start = gte.filter(|x| !x.is_nan()).map_or(0, ordered_f64_key);
                        let end = lte
                            .filter(|x| !x.is_nan())
                            .map_or(u64::MAX, ordered_f64_key);
                        if start <= end {
                            for entry in tree.range(start..=end) {
                                range_union |= &*entry.value().read();
                            }
                        }
                    }
                    if range_union.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &range_union);
                }
                FilterExpr::InBox {
                    min_bounds,
                    max_bounds,
                } => {
                    self.ensure_spatial_index();
                    let count = self.count_nodes() as u32;
                    let region = hyperspace_core::region::BoxRegion::new(
                        min_bounds.clone(),
                        max_bounds.clone(),
                    );
                    let candidate_bm = self.spatial_index.query_candidates_box(
                        min_bounds,
                        max_bounds,
                        bitmap.as_ref(),
                        deleted,
                        count,
                    );
                    let mut box_match = RoaringBitmap::new();
                    let mut scratch = Vec::with_capacity(self.dimension);
                    for i in candidate_bm {
                        if self.with_vector_coords(i, &mut scratch, |coords| {
                            region.contains_coords(coords)
                        }) {
                            box_match.insert(i);
                        }
                    }
                    if box_match.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &box_match);
                }
                FilterExpr::InCone {
                    axes,
                    apertures,
                    cen,
                } => {
                    let count = self.count_nodes() as u32;
                    let region = hyperspace_core::region::ConeRegion::new(
                        axes.clone(),
                        apertures.clone(),
                        *cen,
                    );
                    let mut cone_match = RoaringBitmap::new();
                    let mut scratch = Vec::with_capacity(self.dimension);
                    if let Some(ref bm) = bitmap {
                        for i in bm {
                            if !deleted.contains(i)
                                && self.with_vector_coords(i, &mut scratch, |coords| {
                                    region.contains_coords(coords)
                                })
                            {
                                cone_match.insert(i);
                            }
                        }
                    } else {
                        for i in 0..count {
                            if !deleted.contains(i)
                                && self.with_vector_coords(i, &mut scratch, |coords| {
                                    region.contains_coords(coords)
                                })
                            {
                                cone_match.insert(i);
                            }
                        }
                    }
                    if cone_match.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &cone_match);
                }
                FilterExpr::InBall { center, radius } => {
                    self.ensure_spatial_index();
                    let count = self.count_nodes() as u32;
                    let region = hyperspace_core::region::BallRegion::new(center.clone(), *radius);
                    let candidate_bm = self.spatial_index.query_candidates_ball(
                        center,
                        *radius,
                        bitmap.as_ref(),
                        deleted,
                        count,
                    );
                    let mut ball_match = RoaringBitmap::new();
                    let mut scratch = Vec::with_capacity(self.dimension);
                    for i in candidate_bm {
                        if self.with_vector_coords(i, &mut scratch, |coords| {
                            region.contains_coords(coords)
                        }) {
                            ball_match.insert(i);
                        }
                    }
                    if ball_match.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &ball_match);
                }
                FilterExpr::And(conditions) => {
                    // Evaluate each sub-condition independently and AND (intersect) the bitmaps.
                    let empty_exact = std::collections::HashMap::new();
                    for cond in conditions {
                        if let Some(sub_bm) =
                            self.build_allowed_bitmap(&empty_exact, std::slice::from_ref(cond))
                        {
                            apply_mask(&mut bitmap, &sub_bm);
                        } else {
                            return Some(RoaringBitmap::new());
                        }
                    }
                }
                FilterExpr::Or(conditions) => {
                    // Evaluate each sub-condition independently and OR (union) the bitmaps.
                    let empty_exact = std::collections::HashMap::new();
                    let mut union = RoaringBitmap::new();
                    for cond in conditions {
                        if let Some(sub_bm) =
                            self.build_allowed_bitmap(&empty_exact, std::slice::from_ref(cond))
                        {
                            union |= sub_bm;
                        }
                    }
                    if union.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &union);
                }
                FilterExpr::Not(cond) => {
                    // NOT = all active nodes MINUS nodes matching the inner condition.
                    let count = self.count_nodes() as u32;
                    let mut all_active = RoaringBitmap::new();
                    all_active.insert_range(0..count);
                    all_active -= deleted;
                    let empty_exact = std::collections::HashMap::new();
                    let excluded = self
                        .build_allowed_bitmap(&empty_exact, std::slice::from_ref(cond))
                        .unwrap_or_default();
                    let not_bm = all_active - excluded;
                    if not_bm.is_empty() {
                        return Some(RoaringBitmap::new());
                    }
                    apply_mask(&mut bitmap, &not_bm);
                }
            }
        }

        match bitmap {
            Some(mut bm) => {
                bm -= deleted;
                Some(bm)
            }
            None => None,
        }
    }

    // Support Soft Delete and reclaim metadata memory immediately
    pub fn delete(&self, id: NodeId) {
        self.deletes_since_repair.fetch_add(1, Ordering::Relaxed);
        {
            let mut del = self.metadata.deleted.write();
            del.insert(id);
            self.metadata.has_deletions.store(true, Ordering::Release);
        }
        if let Some((_, old_meta)) = self.metadata.forward.remove(&id) {
            for (k, v) in old_meta {
                if let Some((field, num)) = Self::numeric_entry(&k, &v) {
                    self.numeric_remove(field, num, id);
                }
                let tag = format!("{k}:{v}");
                if let Some(mut bm) = self.metadata.inverted.get_mut(&tag) {
                    bm.remove(id);
                }
            }
        }
    }

    pub fn search_momentum(
        &self,
        trajectory: &[Vec<f64>],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &hyperspace_core::SearchParams,
    ) -> Result<Vec<(NodeId, f64)>, String> {
        if trajectory.len() < 2 {
            return Err("Momentum traversal requires at least 2 points in trajectory".into());
        }
        let past = &trajectory[trajectory.len() - 2];
        let current = &trajectory[trajectory.len() - 1];

        // 1. Extrapolate to find the semantic destination using metric-specific geometry
        let predicted = M::extrapolate_momentum(past, current, 1.0)?;

        // 2. Perform greedy search around the predicted destination
        Ok(self.search(&predicted, filter, complex_filters, params))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        query: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        params: &hyperspace_core::SearchParams,
    ) -> Vec<(NodeId, f64)> {
        // If hybrid query is present, we use RRF Fusion
        if let Some(text) = params.hybrid_query.as_deref() {
            return self.search_hybrid(query, filter, complex_filters, text, params);
        }

        let allowed_bitmap = if filter.is_empty() && complex_filters.is_empty() {
            None
        } else {
            let bm = self.build_allowed_bitmap(filter, complex_filters);
            if bm.as_ref().is_some_and(roaring::RoaringBitmap::is_empty) {
                return Vec::new();
            }
            bm
        };

        self.search_with_allowed(query, allowed_bitmap.as_ref(), params)
    }

    fn ensure_spatial_index(&self) {
        let count = self.count_nodes() as u32;
        let start = self.spatial_index.indexed_count();
        if start >= count {
            return;
        }
        let mut scratch = Vec::with_capacity(self.dimension);
        for id in start..count {
            self.with_vector_coords(id, &mut scratch, |coords| {
                self.spatial_index.insert_vector(id, coords);
            });
        }
    }

    #[inline(always)]
    fn build_hyper_vector(coords: Vec<f64>) -> HyperVector {
        let alpha = if M::name() == "poincare" {
            let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
            1.0 / (1.0 - sq_norm)
        } else {
            1.0
        };
        HyperVector { coords, alpha }
    }

    pub fn search_with_allowed(
        &self,
        query: &[f64],
        allowed_bitmap: Option<&roaring::RoaringBitmap>,
        params: &hyperspace_core::SearchParams,
    ) -> Vec<(NodeId, f64)> {
        self.search_impl(query, allowed_bitmap, params, ScanMode::Auto)
    }

    fn search_impl(
        &self,
        query: &[f64],
        allowed_bitmap: Option<&roaring::RoaringBitmap>,
        params: &hyperspace_core::SearchParams,
        mode: ScanMode,
    ) -> Vec<(NodeId, f64)> {
        let _active = ActiveSearch::enter();
        if allowed_bitmap.is_some_and(roaring::RoaringBitmap::is_empty) {
            return Vec::new();
        }

        // 1. Validate query.
        if query.len() != self.dimension || M::validate(query).is_err() {
            return Vec::new();
        }
        let needs_rotation = matches!(
            self.mode,
            QuantizationMode::Turbo | QuantizationMode::Binary | QuantizationMode::Ternary
        );
        let mut q_coords_scratch =
            QUERY_F64_SCRATCH.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
        q_coords_scratch.clear();
        q_coords_scratch.extend_from_slice(query);
        let mut q_vec = Self::build_hyper_vector(q_coords_scratch);

        if needs_rotation {
            if self.mode == QuantizationMode::Turbo {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = self.dimension.saturating_sub(head_dim);
                QUERY_F32_SCRATCH.with(|buf| {
                    let mut b = buf.borrow_mut();
                    b.clear();
                    b.reserve(tail_dim);
                    b.extend(q_vec.coords[head_dim..].iter().map(|&x| x as f32));
                    hyperspace_core::turbo_rot::rotate_vector_in_place(&mut b);
                    for i in 0..tail_dim {
                        q_vec.coords[head_dim + i] = b[i] as f64;
                    }
                });
            } else if self.mode == QuantizationMode::Binary {
                QUERY_F32_SCRATCH.with(|buf| {
                    let mut b = buf.borrow_mut();
                    b.clear();
                    b.reserve(q_vec.coords.len());
                    b.extend(q_vec.coords.iter().map(|&x| x as f32));
                    hyperspace_core::turbo_rot::rotate_vector_in_place(&mut b);
                    for i in 0..q_vec.coords.len() {
                        q_vec.coords[i] = b[i] as f64;
                    }
                });
            } else if self.mode == QuantizationMode::Ternary {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = self.dimension.saturating_sub(head_dim);
                QUERY_F32_SCRATCH.with(|buf| {
                    let mut b = buf.borrow_mut();
                    b.clear();
                    b.reserve(tail_dim);
                    b.extend(q_vec.coords[head_dim..].iter().map(|&x| x as f32));
                    hyperspace_core::turbo_rot::rotate_vector_in_place(&mut b);
                    for i in 0..tail_dim {
                        q_vec.coords[head_dim + i] = b[i] as f64;
                    }
                });
            }
        }

        let entry_node = self.entry_point.load(Ordering::Relaxed);

        // FIX #3: Merge both checks into a single lock-free call.
        let start_layer = {
            let nodes_count = self.nodes.count();
            if nodes_count == 0 || (entry_node as usize) >= nodes_count {
                QUERY_F64_SCRATCH.with(|cell| *cell.borrow_mut() = q_vec.coords);
                return vec![];
            }
            self.nodes.get(entry_node as usize).map_or(0, Node::level)
        };

        let query_klein = if start_layer > 0 {
            self.compute_query_klein(&q_vec)
        } else {
            None
        };
        let query_klein_f32: Option<SmallVec<[f32; 256]>> = if start_layer > 0 {
            self.compute_query_klein_f32(&q_vec)
        } else {
            None
        };
        let query_klein_f32_ref = query_klein_f32.as_deref();

        let needs_f32 = self.needs_query_f32();
        let mut query_f32_scratch = if needs_f32 {
            let mut v =
                VISITED_SCRATCH.with(|cell| std::mem::take(&mut cell.borrow_mut().query_f32));
            v.clear();
            v.extend(q_vec.coords.iter().map(|&x| x as f32));
            Some(v)
        } else {
            None
        };
        let q_f32_ref = query_f32_scratch.as_deref();

        let pq_lut_buf = self.prepare_query(&q_vec, q_f32_ref);
        let pq_lut_ref = pq_lut_buf.as_ref();
        let qctx = QueryContext {
            query: &q_vec,
            query_f32: q_f32_ref,
            query_klein: query_klein.as_ref(),
            query_klein_f32: query_klein_f32_ref,
            pq_lut: pq_lut_ref,
            mrl_dim: params.mrl_dimension,
        };

        if mode == ScanMode::Auto
            && allowed_bitmap.is_none()
            && params.mrl_dimension.is_none()
            && params.hybrid_query.is_none()
        {
            self.sample_query(query);
        }
        let flat = match mode {
            ScanMode::Auto => self.flat_scan_applies(params),
            ScanMode::Graph => false,
            ScanMode::Flat => true,
        };
        let mut candidates = if allowed_bitmap.is_none() && flat {
            // Small index: an exact sequential scan touches fewer cache lines than a
            // graph walk that would visit most nodes anyway (and has 100% recall).
            self.search_flat_all(
                qctx.query,
                qctx.query_f32,
                params.top_k,
                qctx.pq_lut,
                qctx.mrl_dim,
            )
        } else {
            let storage_view = self.storage.view();
            let mut curr_dist = if (entry_node as usize) < storage_view.count() {
                self.dist_upper_bytes(
                    storage_view.get(entry_node),
                    qctx.query,
                    qctx.query_f32,
                    qctx.query_klein,
                    qctx.query_klein_f32,
                    qctx.pq_lut,
                    qctx.mrl_dim,
                )
            } else {
                f64::MAX
            };
            let mut curr_node = entry_node;

            // 1. Zoom-in phase: Greedy search from top to layer 1.
            {
                let nodes_count = self.nodes.count().min(storage_view.count());
                for level in (1..=start_layer).rev() {
                    let mut changed = true;
                    while changed {
                        changed = false;
                        if (curr_node as usize) >= nodes_count {
                            break;
                        }
                        let Some(node) = self.nodes.get(curr_node as usize) else {
                            break;
                        };
                        if node.level() < level {
                            break;
                        }
                        let neighbor_ids: SmallVec<[NodeId; 16]> =
                            node.upper[level - 1].read().iter().copied().collect();
                        for neighbor in neighbor_ids {
                            if (neighbor as usize) >= nodes_count {
                                continue;
                            }
                            let d = self.dist_upper_bytes(
                                storage_view.get(neighbor),
                                qctx.query,
                                qctx.query_f32,
                                qctx.query_klein,
                                qctx.query_klein_f32,
                                qctx.pq_lut,
                                qctx.mrl_dim,
                            );
                            if d < curr_dist {
                                curr_dist = d;
                                curr_node = neighbor;
                                changed = true;
                            }
                        }
                    }
                }
            }

            // 2. Local search phase: Layer 0 with Filter
            // RECALL FIX #4: Enforce ef >= top_k. Per HNSW spec the beam size must
            // be at least as large as the number of results requested, or the
            // result set is truncated to ef < top_k silently.
            let ef_for_search = params.ef_search.max(params.top_k);
            self.search_layer0(
                curr_node,
                qctx.query,
                qctx.query_f32,
                params.top_k,
                ef_for_search,
                allowed_bitmap,
                qctx.pq_lut,
                qctx.mrl_dim,
            )
        };

        // MRL Reranking: If we used truncated search, we must recalculate full distance for the results.
        if params.mrl_dimension.is_some() {
            for cand in &mut candidates {
                cand.1 = self.dist_mrl_opt(cand.0, &q_vec, q_f32_ref, pq_lut_ref, None, None);
            }
            candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
        }

        if let Some(v) = query_f32_scratch.take() {
            VISITED_SCRATCH.with(|cell| cell.borrow_mut().query_f32 = v);
        }
        QUERY_F64_SCRATCH.with(|cell| *cell.borrow_mut() = q_vec.coords);

        if params.use_wasserstein {
            for cand in &mut candidates {
                let vec = self.get_vector_original(cand.0);
                cand.1 =
                    hyperspace_core::wasserstein::WassersteinDistance::compute(query, &vec.coords);
            }
            candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
            // Ensure we keep only top k
            candidates.truncate(params.top_k);
        } else {
            for cand in &mut candidates {
                cand.1 = Self::finalize_distance(cand.1);
            }
        }

        candidates
    }

    pub fn peek(
        &self,
        limit: usize,
        offset: usize,
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        // FIX #6: Hold a single lock-free count (boxcar) for the entire loop.
        let max_len = self.nodes.count();

        let mut result = Vec::with_capacity(limit);
        let mut skipped = 0;

        // FIX #6: Hold a single read-lock for the entire loop instead of acquiring it per-iteration.
        let deleted = self.metadata.deleted.read();
        for id in (0..max_len).rev() {
            if result.len() >= limit {
                break;
            }
            let id = id as u32;

            if deleted.contains(id) {
                continue;
            }

            if skipped < offset {
                skipped += 1;
                continue;
            }

            let vec = self.get_vector_original(id).coords;
            let meta = self
                .metadata
                .forward
                .get(&id)
                .map_or_else(std::collections::HashMap::new, |m| m.clone());
            result.push((id, vec, meta));
        }
        result
    }

    pub fn peek_all(&self) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        // FIX #6: Lock-free count via boxcar.
        let max_len = self.nodes.count();

        let mut result = Vec::with_capacity(max_len);

        // FIX #6: Hold a single read-lock for the entire loop instead of acquiring it per-iteration.
        let deleted = self.metadata.deleted.read();
        for id in 0..max_len {
            let id = id as u32;
            if deleted.contains(id) {
                continue;
            }
            let vec = self.get_vector_original(id).coords;
            let meta = self
                .metadata
                .forward
                .get(&id)
                .map_or_else(std::collections::HashMap::new, |m| m.clone());
            result.push((id, vec, meta));
        }
        result
    }

    /// Offset subtracted from distances before packing them into `f32` candidates.
    /// Hyperbolic metrics (Poincaré / Lorentz) return `cosh(d) >= 1`; storing
    /// `cosh(d) - 1` preserves precision for near neighbors (`cosh(d) - 1 ≈ d²/2`).
    #[inline(always)]
    fn dist_bias() -> f64 {
        if matches!(M::name(), "poincare" | "lorentz") {
            1.0
        } else {
            0.0
        }
    }

    /// ACORN-style filtered traversal (`HS_FILTER_ACORN`, default on). Only engaged for
    /// filters that keep < 50% of the graph and are too large for the brute-force path.
    fn acorn_enabled() -> bool {
        static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *ON.get_or_init(|| {
            std::env::var("HS_FILTER_ACORN").map_or(true, |v| {
                !matches!(v.to_lowercase().as_str(), "0" | "false" | "no" | "off")
            })
        })
    }

    /// Adaptive-ef patience (`HS_SEARCH_PATIENCE`, default 0 = disabled): layer-0 search
    /// stops after this many consecutive non-improving expansions once `k` results exist.
    fn search_patience() -> usize {
        static P: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *P.get_or_init(|| {
            std::env::var("HS_SEARCH_PATIENCE")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
        })
    }

    /// Vamana-style α for the RNG neighbor heuristic (`HS_HNSW_ALPHA`, default 1.0 =
    /// classic HNSW). α > 1 keeps more long-range edges, improving recall on hard data.
    fn rng_alpha() -> f64 {
        static ALPHA: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
        *ALPHA.get_or_init(|| {
            std::env::var("HS_HNSW_ALPHA")
                .ok()
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|a| a.is_finite() && *a >= 1.0)
                .unwrap_or(1.0)
        })
    }

    /// RNG domination test: `true` when the candidate is "covered" by an already
    /// selected neighbor (`α · d(cand, neighbor) < d(cand, query)`). Squared-type
    /// distances (L2², cosh−1 ≈ d²/2) scale with α².
    #[inline(always)]
    fn rng_dominated(dist_to_neighbor: f64, cand_dist: f64, alpha: f64) -> bool {
        if alpha <= 1.0 {
            return dist_to_neighbor < cand_dist;
        }
        let scale = if matches!(
            M::name(),
            "l2" | "cosine" | "poincare" | "lorentz" | "hybrid"
        ) {
            alpha * alpha
        } else {
            alpha
        };
        let bias = Self::dist_bias();
        (dist_to_neighbor - bias) * scale < cand_dist - bias
    }

    #[inline(always)]
    fn needs_query_f32(&self) -> bool {
        self.storage_f32
            || matches!(
                self.mode,
                QuantizationMode::ScalarI8
                    | QuantizationMode::AsymmetricHybrid801
                    | QuantizationMode::ProductQuantization
                    | QuantizationMode::OPQ
                    | QuantizationMode::RaBitQ
                    | QuantizationMode::RaBitQ4
                    | QuantizationMode::RaBitQ2
                    | QuantizationMode::PQ4
            )
    }

    #[inline(always)]
    #[allow(clippy::many_single_char_names)]
    fn dist_mrl_opt(
        &self,
        node_id: NodeId,
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
        threshold: Option<f64>,
    ) -> f64 {
        // Defensive: Check bounds to avoid casting fallback/misaligned bytes during swaps
        if node_id as usize >= self.storage.count() {
            return f64::MAX;
        }

        let bytes = self.storage.get(node_id);
        self.dist_bytes_opt(bytes, query, query_f32, pq_lut, mrl_dim, threshold)
    }

    /// Internal ranking distance used by every HNSW traversal / heap.
    ///
    /// Identical to the metric distance except for the hybrid product space, which ranks by
    /// the **squared** distance `d_lor² + ‖Δe‖²` (monotone in the distance): no final `sqrt`,
    /// and the f32 fast path needs no `ln`/`sqrt` for near Lorentz heads. `threshold` is in the
    /// same internal units. Convert to user-facing distances with [`Self::finalize_distance`].
    #[inline]
    pub fn dist_bytes_opt(
        &self,
        bytes: &[u8],
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
        threshold: Option<f64>,
    ) -> f64 {
        if M::name() == "hybrid" {
            if self.mode == QuantizationMode::None && self.storage_f32 {
                if let Some(q) = query_f32 {
                    let cb = &bytes[..bytes.len().saturating_sub(4)];
                    return hyperspace_core::simd::hybrid_sq_f32_bytes(cb, q, threshold);
                }
            }
            let d = self.dist_bytes_raw(
                bytes,
                query,
                query_f32,
                pq_lut,
                mrl_dim,
                threshold.map(|t| t.max(0.0).sqrt()),
            );
            return d * d;
        }
        self.dist_bytes_raw(bytes, query, query_f32, pq_lut, mrl_dim, threshold)
    }

    /// Layer-0 slot capacity: the largest list the prune policy can leave behind
    /// (`m_max` plus the 25% slack used by decode-heavy modes, plus the link being added),
    /// and never less than the longest list being restored from a snapshot.
    fn layer0_capacity(config: &GlobalConfig, longest_loaded: usize) -> usize {
        let m_max = config.get_m() * 2;
        ((m_max * 5 / 4).max(m_max + 2) + 1)
            .max(longest_loaded)
            .max(8)
    }

    fn build_layer0(config: &GlobalConfig, lists: Vec<Vec<NodeId>>) -> layer0::Layer0Arena {
        let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
        let arena = layer0::Layer0Arena::new(Self::layer0_capacity(config, longest));
        for (id, list) in lists.into_iter().enumerate() {
            arena.set(id as u32, &list);
        }
        arena
    }

    /// All levels of a node's adjacency, level 0 first (snapshot format).
    fn snapshot_layers(&self, node: &Node) -> Vec<Vec<NodeId>> {
        let mut out = Vec::with_capacity(node.level() + 1);
        let mut l0 = layer0::Links::new();
        self.layer0.read_into(node.id, &mut l0);
        out.push(l0.to_vec());
        for upper in &*node.upper {
            out.push(upper.read().to_vec());
        }
        out
    }

    /// Snapshot of `id`'s neighbors on `level` into `out`. Returns `false` if the node
    /// does not exist or does not reach `level`.
    fn links_into(&self, id: NodeId, level: usize, out: &mut layer0::Links) -> bool {
        out.clear();
        let Some(node) = self.nodes.get(id as usize) else {
            return false;
        };
        if level == 0 {
            self.layer0.read_into(id, out);
            return true;
        }
        match node.upper.get(level - 1) {
            Some(l) => {
                out.extend_from_slice(&l.read());
                true
            }
            None => false,
        }
    }

    /// Mutates `id`'s neighbor list on `level` under its writer lock; returns the new length
    /// (0 if the node does not reach `level`).
    fn links_update(&self, id: NodeId, level: usize, f: impl FnOnce(&mut layer0::Links)) -> usize {
        if level == 0 {
            if (id as usize) >= self.nodes.count() {
                return 0;
            }
            return self.layer0.update(id, f);
        }
        let Some(node) = self.nodes.get(id as usize) else {
            return 0;
        };
        let Some(lock) = node.upper.get(level - 1) else {
            return 0;
        };
        let mut guard = lock.write();
        let mut buf = layer0::Links::from_slice(&guard);
        f(&mut buf);
        *guard = SmallVec::from_slice(&buf);
        guard.len()
    }

    fn links_set(&self, id: NodeId, level: usize, links: &[NodeId]) {
        self.links_update(id, level, |buf| {
            buf.clear();
            buf.extend_from_slice(links);
        });
    }

    /// Builds the RaBitQ / PQ4 quantizers for `mode`, loading persisted state from `dir`.
    #[allow(clippy::type_complexity)]
    fn code_quantizers(
        mode: QuantizationMode,
        dimension: usize,
        dir: Option<&std::path::Path>,
    ) -> (
        Option<Arc<hyperspace_core::rabitq::RaBitQ>>,
        Option<Arc<hyperspace_core::pq4::Pq4Quantizer>>,
    ) {
        match mode {
            QuantizationMode::RaBitQ | QuantizationMode::RaBitQ4 | QuantizationMode::RaBitQ2 => {
                let bits = mode.rabitq_bits().unwrap_or(1);
                // The centroid file holds no width: the collection's mode decides it.
                let rq = dir
                    .and_then(|d| {
                        hyperspace_core::rabitq::RaBitQ::load_from_file(&d.join(RABITQ_FILE)).ok()
                    })
                    .filter(|r| r.dim == dimension)
                    .map_or_else(
                        || hyperspace_core::rabitq::RaBitQ::with_bits(dimension, bits),
                        |r| hyperspace_core::rabitq::RaBitQ { bits, ..r },
                    );
                (Some(Arc::new(rq)), None)
            }
            QuantizationMode::PQ4 => {
                let pq4 = dir
                    .and_then(|d| {
                        hyperspace_core::pq4::Pq4Quantizer::load_from_file(&d.join(PQ4_FILE)).ok()
                    })
                    .filter(|q| q.dimension == dimension)
                    .unwrap_or_else(|| {
                        hyperspace_core::pq4::Pq4Quantizer::new(
                            dimension,
                            hyperspace_core::pq4::default_num_subvectors(dimension),
                        )
                    });
                (None, Some(Arc::new(pq4)))
            }
            _ => (None, None),
        }
    }

    /// Per-query ADC precomputation for code-based modes (PQ / OPQ / RaBitQ / PQ4).
    fn prepare_query(&self, query: &HyperVector, query_f32: Option<&[f32]>) -> Option<QueryPrep> {
        if self.bit_lut_kind().is_some() {
            return Some(QueryPrep::Bits(hyperspace_core::vector::BitPlaneLut::new(
                &query.coords,
            )));
        }
        if self.sq8_scaled && SQ8_INT_KERNEL.load(Ordering::Relaxed) {
            let owned;
            let q: &[f32] = if let Some(q) = query_f32 {
                q
            } else {
                owned = query.coords.iter().map(|&x| x as f32).collect::<Vec<f32>>();
                &owned
            };
            let n = q.len().min(self.dimension);
            return Some(QueryPrep::Sq8(hyperspace_core::simd::Sq8Query::new(
                &q[..n],
            )));
        }
        if self.i4_kernel_applies() {
            let (even, odd) = ScalarI4Vector::deinterleave_query(&query.coords, self.dimension);
            let norm_sq = query
                .coords
                .iter()
                .take(self.dimension)
                .map(|&x| f64::from(x as f32).powi(2))
                .sum();
            return Some(QueryPrep::I4 { even, odd, norm_sq });
        }
        let owned;
        let q: &[f32] = if let Some(q) = query_f32 {
            q
        } else {
            owned = query.coords.iter().map(|&x| x as f32).collect::<Vec<f32>>();
            &owned
        };
        match self.mode {
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ => self
                .pq
                .load()
                .as_ref()
                .map(|pq| QueryPrep::Pq(pq.compute_lut(q))),
            QuantizationMode::RaBitQ | QuantizationMode::RaBitQ4 | QuantizationMode::RaBitQ2 => {
                self.rabitq
                    .load()
                    .as_ref()
                    .map(|r| QueryPrep::RaBitQ(r.query(q)))
            }
            QuantizationMode::PQ4 => self
                .pq4
                .load()
                .as_ref()
                .map(|p| QueryPrep::Pq4(p.compute_lut(q))),
            _ => None,
        }
    }

    /// Kernel for a hot loop given an optional ADC precomputation.
    fn kernel_for<'a>(
        &self,
        query: &'a HyperVector,
        query_f32: Option<&'a [f32]>,
        prep: Option<&'a QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> DistKernel<'a> {
        match prep {
            Some(QueryPrep::RaBitQ(q)) => DistKernel::RaBitQ(q),
            Some(QueryPrep::Pq4(l)) => DistKernel::Pq4(l),
            Some(QueryPrep::Bits(l)) if mrl_dim.is_none() => match self.bit_lut_kind() {
                Some(BitLutKind::Binary) => DistKernel::BinaryLut(l),
                Some(BitLutKind::TernaryL2) => DistKernel::TernaryL2Lut(l),
                Some(BitLutKind::TernaryCos) => DistKernel::TernaryCosLut(l),
                None => DistKernel::Generic,
            },
            Some(QueryPrep::I4 { even, odd, norm_sq }) if mrl_dim.is_none() => {
                DistKernel::I4L2(even, odd, *norm_sq)
            }
            Some(QueryPrep::Sq8(q)) if mrl_dim.is_none() => DistKernel::Sq8Int(q),
            // 8-bit PQ scores through `dist_bytes_opt`; MRL truncation needs the generic path.
            Some(
                QueryPrep::Pq(_) | QueryPrep::Bits(_) | QueryPrep::I4 { .. } | QueryPrep::Sq8(_),
            ) => DistKernel::Generic,
            None => self.resolve_kernel(query, query_f32, mrl_dim),
        }
    }

    /// New ScalarI8 L2 / cosine indexes store per-vector scaled codes.
    fn sq8_scaled_supported(mode: QuantizationMode) -> bool {
        mode == QuantizationMode::ScalarI8 && matches!(M::name(), "l2" | "cosine")
    }

    /// Dequantization step of a ScalarI8 code (`x_i ≈ code_i · step`).
    #[inline(always)]
    fn sq8_step(&self, alpha: f32) -> f32 {
        if self.sq8_scaled {
            alpha / 127.0
        } else {
            1.0 / 127.0
        }
    }

    /// Squared L2 over per-vector scaled ScalarI8 codes: integer kernel when a prepared
    /// query is available, `f32` query kernel otherwise (MRL truncation included).
    #[inline]
    fn sq8_scaled_dist(
        bytes: &[u8],
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        prep: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let split = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[split..].try_into().unwrap_or([0; 4]));
        if let (Some(QueryPrep::Sq8(q)), None) = (prep, mrl_dim) {
            return f64::from(hyperspace_core::simd::sq8_scaled_l2_int(
                &bytes[..split],
                alpha,
                q,
            ));
        }
        let owned;
        let q32: &[f32] = if let Some(q) = query_f32 {
            q
        } else {
            owned = query.coords.iter().map(|&x| x as f32).collect::<Vec<f32>>();
            &owned
        };
        let n = split.min(q32.len()).min(mrl_dim.unwrap_or(usize::MAX));
        f64::from(
            hyperspace_core::simd::sq8_scaled_l2_and_norm_sq(&bytes[..n], alpha / 127.0, &q32[..n])
                .0,
        )
    }

    /// Decodes a stored ScalarI8 code (`codes ‖ alpha`) into `out` without allocating and
    /// returns the `alpha` a `HyperVector` of this metric carries: 0 for Lorentz (the time
    /// coordinate is recomputed on the hyperboloid from the scaled spatial part), 1 for
    /// per-vector scaled codes, the stored `1 / (1 − ‖x‖²)` for legacy codes.
    fn decode_sq8(&self, bytes: &[u8], out: &mut [f64]) -> f64 {
        let split = bytes.len().saturating_sub(4);
        let stored = if bytes.len() >= 4 {
            f32::from_le_bytes(bytes[split..].try_into().unwrap_or([0; 4]))
        } else {
            1.0
        };
        let codes = &bytes[..split];
        if M::name() == "lorentz" {
            let scale = f64::from(stored) / 127.0;
            let mut norm_sq = 0.0;
            for (o, &c) in out.iter_mut().zip(codes).skip(1) {
                let v = f64::from(c.cast_signed()) * scale;
                *o = v;
                norm_sq += v * v;
            }
            if let Some(t) = out.first_mut() {
                *t = (1.0 + norm_sq).sqrt();
            }
            return 0.0;
        }
        let step = f64::from(self.sq8_step(stored));
        for (o, &c) in out.iter_mut().zip(codes) {
            *o = f64::from(c.cast_signed()) * step;
        }
        if self.sq8_scaled {
            1.0
        } else {
            f64::from(stored)
        }
    }

    /// Whether this index stores per-vector scaled ScalarI8 codes.
    #[must_use]
    pub fn sq8_scaled(&self) -> bool {
        self.sq8_scaled
    }

    /// ScalarI4 indexes whose distance is plain squared L2 over the whole vector (mirrors
    /// the `else` branch of the `ScalarI4` arm of `dist_bytes_raw`).
    #[inline]
    fn i4_kernel_applies(&self) -> bool {
        self.mode == QuantizationMode::ScalarI4
            && BIT_LUT_KERNELS.load(Ordering::Relaxed)
            && !matches!(M::name(), "poincare" | "lorentz")
            && !(M::name() == "hybrid" && self.dimension == 801)
    }

    /// Which nibble-table kernel serves this index (mirrors the metric dispatch of the
    /// `Binary` / `Ternary` arms of `dist_bytes_raw`; MRL-head hybrid ternary excluded).
    #[inline]
    fn bit_lut_kind(&self) -> Option<BitLutKind> {
        if !BIT_LUT_KERNELS.load(Ordering::Relaxed) {
            return None;
        }
        let name = M::name();
        match self.mode {
            QuantizationMode::Binary if name != "poincare" => Some(BitLutKind::Binary),
            QuantizationMode::Ternary => match name {
                "poincare" | "lorentz" => None,
                "hybrid" if self.dimension == 801 => None,
                "cosine" => Some(BitLutKind::TernaryCos),
                _ => Some(BitLutKind::TernaryL2),
            },
            _ => None,
        }
    }

    /// Resolves the per-query distance kernel (see [`DistKernel`]).
    fn resolve_kernel<'a>(
        &self,
        query: &'a HyperVector,
        query_f32: Option<&'a [f32]>,
        mrl_dim: Option<usize>,
    ) -> DistKernel<'a> {
        if mrl_dim.is_some() {
            return DistKernel::Generic;
        }
        let name = M::name();
        match (self.mode, self.storage_f32, query_f32) {
            (QuantizationMode::None, true, Some(q)) => match name {
                "l2" | "cosine" => DistKernel::F32L2(q),
                "poincare" => DistKernel::F32Poincare(q, query.alpha),
                "lorentz" => DistKernel::F32Lorentz(q),
                "hybrid" => DistKernel::F32HybridSq(q),
                "dot" => DistKernel::F32Dot(q),
                _ => DistKernel::Generic,
            },
            (QuantizationMode::None, false, _) if !self.zonal => match name {
                "l2" | "cosine" => DistKernel::F64L2(&query.coords),
                "lorentz" => DistKernel::F64Lorentz(&query.coords),
                _ => DistKernel::Generic,
            },
            (QuantizationMode::ScalarI8, _, Some(q))
                if matches!(name, "l2" | "cosine") && !self.sq8_scaled =>
            {
                DistKernel::Sq8L2(q)
            }
            _ => DistKernel::Generic,
        }
    }

    /// Hot-loop distance with a pre-resolved kernel. Semantically identical to
    /// [`Self::dist_bytes_opt`] (same internal units and threshold semantics).
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    fn dist_k(
        &self,
        kernel: DistKernel<'_>,
        bytes: &[u8],
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
        threshold: Option<f64>,
    ) -> f64 {
        use hyperspace_core::simd;
        match kernel {
            DistKernel::F32L2(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(4)];
                f64::from(simd::l2_sq_f32_bytes_adsampling(
                    cb,
                    q,
                    threshold.map(|t| t as f32),
                    simd::EXACT_EARLY_ABANDON,
                ))
            }
            DistKernel::F32Poincare(q, query_alpha) => {
                let split = bytes.len().saturating_sub(4);
                let alpha_u = if bytes.len() >= 4 {
                    f64::from(f32::from_le_bytes(
                        bytes[split..].try_into().unwrap_or([0; 4]),
                    ))
                } else {
                    1.0
                };
                let alpha_prod = (alpha_u * query_alpha).max(1e-9);
                let diff_thresh =
                    threshold.map(|t| ((t - 1.0) / (2.0 * alpha_prod)).max(0.0) as f32);
                let diff_sq = f64::from(simd::l2_sq_f32_bytes_adsampling(
                    &bytes[..split],
                    q,
                    diff_thresh,
                    simd::EXACT_EARLY_ABANDON,
                ));
                1.0 + 2.0 * diff_sq * alpha_prod
            }
            DistKernel::F32Lorentz(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(4)];
                simd::lorentz_f32_bytes_thresh(cb, q, threshold)
            }
            DistKernel::F32HybridSq(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(4)];
                simd::hybrid_sq_f32_bytes(cb, q, threshold)
            }
            DistKernel::F32Dot(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(4)];
                let n = (cb.len() / 4).min(q.len());
                1.0 - f64::from(simd::dot_f32_bytes(&cb[..n * 4], &q[..n]))
            }
            DistKernel::Sq8L2(q) => QuantizedHyperVector::distance_bytes_euclidean_sq_f32(bytes, q),
            DistKernel::Sq8Int(q) => {
                let split = bytes.len().saturating_sub(4);
                let alpha = f32::from_le_bytes(bytes[split..].try_into().unwrap_or([0; 4]));
                f64::from(simd::sq8_scaled_l2_int(&bytes[..split], alpha, q))
            }
            DistKernel::F64L2(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(8)];
                let n = (cb.len() / 8).min(q.len());
                simd::l2_sq_f64_bytes(&cb[..n * 8], &q[..n])
            }
            DistKernel::F64Lorentz(q) => {
                let cb = &bytes[..bytes.len().saturating_sub(8)];
                let n = (cb.len() / 8).min(q.len());
                simd::lorentz_cosh_f64_bytes(&cb[..n * 8], &q[..n])
            }
            DistKernel::RaBitQ(q) => f64::from(q.distance_sq(bytes)),
            DistKernel::Pq4(l) => f64::from(l.distance(bytes)),
            DistKernel::BinaryLut(l) => BinaryHyperVector::adc_distance_lut(bytes, l),
            DistKernel::TernaryL2Lut(l) => {
                hyperspace_core::vector::TernaryHyperVector::distance_l2_sq_lut(
                    bytes,
                    l,
                    self.dimension,
                )
            }
            DistKernel::TernaryCosLut(l) => {
                hyperspace_core::vector::TernaryHyperVector::distance_cosine_lut(
                    bytes,
                    l,
                    self.dimension,
                )
            }
            DistKernel::I4L2(even, odd, norm_sq) => ScalarI4Vector::distance_l2_sq_deinterleaved(
                bytes,
                even,
                odd,
                norm_sq,
                self.dimension,
            ),
            DistKernel::Generic => {
                self.dist_bytes_opt(bytes, query, query_f32, pq_lut, mrl_dim, threshold)
            }
        }
    }

    /// Converts an internal ranking distance into the user-facing metric distance
    /// (`acosh` of the `cosh` proxy for Poincaré/Lorentz, `sqrt` for the hybrid space).
    #[inline(always)]
    fn finalize_distance(d: f64) -> f64 {
        match M::name() {
            "poincare" | "lorentz" => hyperspace_core::acosh_from_cosh(d),
            "hybrid" => d.max(0.0).sqrt(),
            _ => d,
        }
    }

    #[allow(clippy::many_single_char_names)]
    fn dist_bytes_raw(
        &self,
        bytes: &[u8],
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
        threshold: Option<f64>,
    ) -> f64 {
        match self.mode {
            QuantizationMode::RaBitQ
            | QuantizationMode::RaBitQ4
            | QuantizationMode::RaBitQ2
            | QuantizationMode::PQ4 => match pq_lut {
                Some(QueryPrep::RaBitQ(q)) => f64::from(q.distance_sq(bytes)),
                Some(QueryPrep::Pq4(l)) => f64::from(l.distance(bytes)),
                _ => match self.prepare_query(query, query_f32) {
                    Some(QueryPrep::RaBitQ(q)) => f64::from(q.distance_sq(bytes)),
                    Some(QueryPrep::Pq4(l)) => f64::from(l.distance(bytes)),
                    _ => f64::MAX,
                },
            },
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ => {
                let thresh_f32 = threshold.map(|t| t as f32);
                if let Some(QueryPrep::Pq(lut)) = pq_lut {
                    f64::from(lut.distance_bytes_thresh(bytes, thresh_f32))
                } else if let Some(ref pq) = *self.pq.load() {
                    if let Some(buf) = query_f32 {
                        let lut = pq.compute_lut(buf);
                        f64::from(lut.distance_bytes_thresh(bytes, thresh_f32))
                    } else {
                        let q_f32: Vec<f32> = query.coords.iter().map(|&x| x as f32).collect();
                        let lut = pq.compute_lut(&q_f32);
                        f64::from(lut.distance_bytes_thresh(bytes, thresh_f32))
                    }
                } else {
                    0.0
                }
            }
            QuantizationMode::ScalarI8 => {
                let name = M::name();
                if self.sq8_scaled {
                    Self::sq8_scaled_dist(bytes, query, query_f32, pq_lut, mrl_dim)
                } else if name == "l2" || name == "cosine" {
                    if let Some(q_f32) = query_f32 {
                        QuantizedHyperVector::distance_bytes_euclidean_sq_f32(bytes, q_f32)
                    } else {
                        QuantizedHyperVector::distance_bytes_euclidean_sq(bytes, query)
                    }
                } else if name == "poincare" {
                    if let Some(q_f32) = query_f32 {
                        QuantizedHyperVector::distance_bytes_poincare_sq_f32(
                            bytes,
                            q_f32,
                            query.alpha,
                        )
                    } else {
                        QuantizedHyperVector::distance_bytes_poincare_sq(bytes, query)
                    }
                } else if name == "lorentz" {
                    if let Some(q_f32) = query_f32 {
                        QuantizedHyperVector::distance_bytes_lorentz_f32(bytes, q_f32)
                    } else {
                        QuantizedHyperVector::distance_bytes_lorentz(bytes, query)
                    }
                } else {
                    let q = QuantizedHyperVector::from_bytes(bytes);
                    M::distance_quantized(&q, query)
                }
            }
            QuantizationMode::AsymmetricHybrid801 => {
                // Specialized hybrid path
                assert!(self.dimension == 801, "AsymmetricHybrid801 quantization mode is only supported for 801-dimensional vectors");
                if let Some(q_f32) = query_f32 {
                    HybridQuantizedVector::distance_bytes_mrl_f32(bytes, q_f32, mrl_dim, threshold)
                } else {
                    HybridQuantizedVector::distance_bytes_mrl(bytes, query, mrl_dim)
                }
            }
            QuantizationMode::AsymmetricHybridLowBit => {
                assert!(self.dimension == 801, "AsymmetricHybridLowBit quantization mode is only supported for 801-dimensional vectors");
                let euc_dim = mrl_dim.unwrap_or(768);
                HybridLowBitQuantizedVector::distance_bytes_mrl(bytes, query, euc_dim)
            }
            QuantizationMode::AsymmetricHybridExtreme => {
                assert!(self.dimension == 801, "AsymmetricHybridExtreme quantization mode is only supported for 801-dimensional vectors");
                let euc_cutoff = if let Some(dim) = mrl_dim {
                    dim.saturating_sub(33)
                } else {
                    768
                };
                HybridExtremeQuantizedVector::distance_bytes_mrl(bytes, query, euc_cutoff)
            }
            QuantizationMode::ScalarI4 => {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let active_dim = mrl_dim.unwrap_or(self.dimension);
                let name = M::name();
                if name == "hybrid" && self.dimension == 801 {
                    let d_lor = QUERY_F64_SCRATCH.with(|buf| {
                        let mut b = buf.borrow_mut();
                        b.clear();
                        b.reserve(33);
                        for i in 0..33 {
                            let offset = i * 4;
                            let h = f32::from_le_bytes(
                                bytes[offset..offset + 4].try_into().unwrap_or([0; 4]),
                            );
                            b.push(f64::from(h));
                        }
                        <LorentzMetric as Metric>::distance(&b, &query.coords[..33])
                    });
                    let d_euc =
                        ScalarI4Vector::distance_bytes_l2_sq_mrl(bytes, query, active_dim, 33);
                    hyperspace_core::hybrid::combine_hybrid_distance(d_lor, d_euc)
                } else if name == "poincare" {
                    QUERY_F64_SCRATCH.with(|buf| {
                        let mut b = buf.borrow_mut();
                        ScalarI4Vector::reconstruct_bytes_into(bytes, active_dim, head_dim, &mut b);
                        PoincareMetric::cosh_distance(&b, &query.coords[..active_dim])
                    })
                } else if name == "lorentz" {
                    QUERY_F64_SCRATCH.with(|buf| {
                        let mut b = buf.borrow_mut();
                        ScalarI4Vector::reconstruct_bytes_into(bytes, active_dim, head_dim, &mut b);
                        LorentzMetric::cosh_distance(&b, &query.coords[..active_dim])
                    })
                } else if let (Some(QueryPrep::I4 { even, odd, norm_sq }), None) = (pq_lut, mrl_dim)
                {
                    ScalarI4Vector::distance_l2_sq_deinterleaved(
                        bytes,
                        even,
                        odd,
                        *norm_sq,
                        self.dimension,
                    )
                } else {
                    ScalarI4Vector::distance_bytes_l2_sq_mrl(bytes, query, active_dim, head_dim)
                }
            }
            QuantizationMode::Turbo => {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = self.dimension.saturating_sub(head_dim);
                let active_dim = mrl_dim.unwrap_or(self.dimension);
                if M::name() == "cosine" {
                    hyperspace_core::hybrid::TurboQuantVector::distance_bytes_cosine(
                        bytes,
                        &query.coords[..active_dim],
                        tail_dim,
                    )
                } else {
                    hyperspace_core::hybrid::TurboQuantVector::distance_bytes_l2_sq(
                        bytes,
                        &query.coords[..active_dim],
                        tail_dim,
                    )
                }
            }
            QuantizationMode::Binary => {
                let name = M::name();
                if name == "poincare" {
                    BinaryHyperVector::poincare_distance_sq_bytes(bytes, query)
                } else if let (Some(QueryPrep::Bits(l)), None) = (pq_lut, mrl_dim) {
                    BinaryHyperVector::adc_distance_lut(bytes, l)
                } else {
                    BinaryHyperVector::adc_distance_bytes(bytes, query)
                }
            }
            QuantizationMode::Ternary
                if mrl_dim.is_none() && matches!(pq_lut, Some(QueryPrep::Bits(_))) =>
            {
                let Some(QueryPrep::Bits(l)) = pq_lut else {
                    unreachable!()
                };
                match self.bit_lut_kind() {
                    Some(BitLutKind::TernaryCos) => {
                        hyperspace_core::vector::TernaryHyperVector::distance_cosine_lut(
                            bytes,
                            l,
                            self.dimension,
                        )
                    }
                    Some(_) => hyperspace_core::vector::TernaryHyperVector::distance_l2_sq_lut(
                        bytes,
                        l,
                        self.dimension,
                    ),
                    None => self.dist_bytes_raw(bytes, query, query_f32, None, mrl_dim, threshold),
                }
            }
            QuantizationMode::Ternary => {
                let name = M::name();
                if name == "hybrid" && self.dimension == 801 {
                    let active_dim = mrl_dim.unwrap_or(801);
                    hyperspace_core::vector::TernaryHyperVector::hybrid_distance_bytes(
                        bytes, query, active_dim,
                    )
                } else if name == "poincare" {
                    hyperspace_core::vector::TernaryHyperVector::poincare_distance_sq_bytes(
                        bytes,
                        query,
                        self.dimension,
                    )
                } else if name == "lorentz" {
                    hyperspace_core::vector::TernaryHyperVector::lorentz_distance_bytes(
                        bytes,
                        query,
                        self.dimension,
                        0,
                    )
                } else if name == "cosine" {
                    hyperspace_core::vector::TernaryHyperVector::distance_bytes_cosine(
                        bytes,
                        query,
                        self.dimension,
                        0,
                    )
                } else {
                    hyperspace_core::vector::TernaryHyperVector::distance_bytes_l2_sq(
                        bytes,
                        query,
                        self.dimension,
                        0,
                    )
                }
            }
            QuantizationMode::None => {
                let name = M::name();
                if self.storage_f32 {
                    let coord_bytes = &bytes[..bytes.len().saturating_sub(4)];
                    let eval_f32 = |b: &[f32]| -> Option<f64> {
                        let n = (coord_bytes.len() / 4).min(b.len());
                        let cb = &coord_bytes[..n * 4];
                        let qb = &b[..n];
                        let thresh_f32 = threshold.map(|t| t as f32);
                        match name {
                            "l2" | "cosine" => Some(f64::from(
                                hyperspace_core::simd::l2_sq_f32_bytes_adsampling(
                                    cb,
                                    qb,
                                    thresh_f32,
                                    hyperspace_core::simd::EXACT_EARLY_ABANDON,
                                ),
                            )),
                            "dot" => {
                                Some(1.0 - f64::from(hyperspace_core::simd::dot_f32_bytes(cb, qb)))
                            }
                            "l1" => Some(f64::from(hyperspace_core::simd::l1_f32_bytes(
                                cb, qb, thresh_f32,
                            ))),
                            "linf" => Some(f64::from(hyperspace_core::simd::linf_f32_bytes(
                                cb, qb, thresh_f32,
                            ))),
                            "poincare" => {
                                let alpha_offset = bytes.len().saturating_sub(4);
                                let alpha_u = if bytes.len() >= 4 {
                                    let ab: [u8; 4] =
                                        bytes[alpha_offset..].try_into().unwrap_or([0; 4]);
                                    f64::from(f32::from_le_bytes(ab))
                                } else {
                                    1.0
                                };
                                let alpha_prod = (alpha_u * query.alpha).max(1e-9);
                                let diff_thresh = thresh_f32
                                    .map(|t| ((t - 1.0) / (2.0 * alpha_prod as f32)).max(0.0));
                                let diff_sq =
                                    f64::from(hyperspace_core::simd::l2_sq_f32_bytes_adsampling(
                                        cb,
                                        qb,
                                        diff_thresh,
                                        hyperspace_core::simd::EXACT_EARLY_ABANDON,
                                    ));
                                Some(1.0 + 2.0 * diff_sq * alpha_prod)
                            }
                            "lorentz" => Some(hyperspace_core::simd::lorentz_f32_bytes_thresh(
                                cb, qb, threshold,
                            )),
                            "hybrid" => Some(
                                hyperspace_core::simd::hybrid_sq_f32_bytes(
                                    cb,
                                    qb,
                                    threshold.map(|t| t * t),
                                )
                                .sqrt(),
                            ),
                            "angular" => {
                                let cos_dist = hyperspace_core::simd::cosine_f32_bytes(cb, qb);
                                let cos = (1.0 - cos_dist).clamp(-1.0, 1.0);
                                Some(cos.acos() / std::f64::consts::PI)
                            }
                            "jaccard" => Some(hyperspace_core::simd::jaccard_f32_bytes(cb, qb)),
                            "hamming" => Some(hyperspace_core::simd::hamming_f32_bytes(cb, qb)),
                            _ => None,
                        }
                    };

                    if let Some(b) = query_f32 {
                        if let Some(d) = eval_f32(b) {
                            return d;
                        }
                    } else {
                        let maybe_d = QUERY_F32_SCRATCH.with(|buf| {
                            let mut b = buf.borrow_mut();
                            b.clear();
                            b.extend(query.coords.iter().map(|&x| x as f32));
                            eval_f32(&b)
                        });
                        if let Some(d) = maybe_d {
                            return d;
                        }
                    }

                    // Fallback for composite/custom metrics (e.g., hybrid, wasserstein) using thread-local scratch
                    QUERY_F64_SCRATCH.with(|buf| {
                        let mut b = buf.borrow_mut();
                        b.clear();
                        let n = coord_bytes.len() / 4;
                        b.reserve(n);
                        for chunk in coord_bytes[..n * 4].chunks_exact(4) {
                            b.push(f64::from(f32::from_le_bytes([
                                chunk[0], chunk[1], chunk[2], chunk[3],
                            ])));
                        }
                        M::distance(&b, &query.coords)
                    })
                } else {
                    let coord_bytes = &bytes[..bytes.len().saturating_sub(8)];
                    let n = (coord_bytes.len() / 8).min(query.coords.len());
                    // Zero-copy fast paths: read the f64 storage in place.
                    match name {
                        "l2" | "cosine" => {
                            return hyperspace_core::simd::l2_sq_f64_bytes(
                                &coord_bytes[..n * 8],
                                &query.coords[..n],
                            );
                        }
                        "poincare" if bytes.len() >= 8 => {
                            let ab: [u8; 8] = bytes[bytes.len() - 8..].try_into().unwrap_or([0; 8]);
                            let alpha_u = f64::from_le_bytes(ab);
                            if alpha_u.is_finite() && alpha_u > 0.0 {
                                let diff_sq = hyperspace_core::simd::l2_sq_f64_bytes(
                                    &coord_bytes[..n * 8],
                                    &query.coords[..n],
                                );
                                return 1.0 + 2.0 * diff_sq * alpha_u * query.alpha;
                            }
                        }
                        "lorentz" => {
                            return hyperspace_core::simd::lorentz_cosh_f64_bytes(
                                &coord_bytes[..n * 8],
                                &query.coords[..n],
                            );
                        }
                        _ => {}
                    }
                    QUERY_F64_SCRATCH.with(|buf| {
                        let mut b = buf.borrow_mut();
                        b.clear();
                        let n = (coord_bytes.len() / 8).min(query.coords.len());
                        b.reserve(n);
                        for chunk in coord_bytes[..n * 8].chunks_exact(8) {
                            b.push(f64::from_le_bytes([
                                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5],
                                chunk[6], chunk[7],
                            ]));
                        }
                        if name == "poincare" {
                            PoincareMetric::cosh_distance(&b, &query.coords[..n])
                        } else if name == "lorentz" {
                            LorentzMetric::cosh_distance(&b, &query.coords[..n])
                        } else {
                            M::distance(&b, &query.coords[..n])
                        }
                    })
                }
            }
        }
    }

    #[allow(clippy::chunks_exact_to_as_chunks, clippy::too_many_arguments)]
    fn dist_upper(
        &self,
        node_id: NodeId,
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        query_klein: Option<&HyperVector>,
        query_klein_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> f64 {
        if node_id as usize >= self.storage.count() {
            return f64::MAX;
        }
        let bytes = self.storage.get(node_id);
        self.dist_upper_bytes(
            bytes,
            query,
            query_f32,
            query_klein,
            query_klein_f32,
            pq_lut,
            mrl_dim,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn dist_upper_bytes(
        &self,
        bytes: &[u8],
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        query_klein: Option<&HyperVector>,
        query_klein_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> f64 {
        if let Some(qk_f32) = query_klein_f32 {
            if self.storage_f32 {
                let m_name = M::name();
                if m_name == "lorentz" {
                    let n = (bytes.len().saturating_sub(4) / 4).min(qk_f32.len());
                    if n <= 1 || bytes.len() < 4 {
                        return 0.0;
                    }
                    let t_u = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).max(1.0);
                    return f64::from(hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                        &bytes[4..n * 4],
                        1.0 / t_u,
                        &qk_f32[1..n],
                    ));
                } else if m_name == "hybrid" {
                    let n = (bytes.len().saturating_sub(4) / 4).min(self.dimension);
                    if n < 33 || bytes.len() < 132 || qk_f32.len() < 33 {
                        return 0.0;
                    }
                    let t_u = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).max(1.0);
                    let d_k = hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                        &bytes[4..132],
                        1.0 / t_u,
                        &qk_f32[1..33],
                    );
                    let d_e = if let (true, Some(qf32)) = (n > 33, query_f32) {
                        hyperspace_core::simd::l2_sq_f32_bytes(
                            &bytes[132..n * 4],
                            &qf32[33..n.min(qf32.len())],
                            None,
                        )
                    } else {
                        0.0
                    };
                    return f64::from(d_k + d_e);
                } else if m_name == "poincare" {
                    let n = (bytes.len().saturating_sub(4) / 4).min(qk_f32.len());
                    let alpha_bytes: [u8; 4] = match bytes
                        .get(bytes.len().saturating_sub(4)..)
                        .and_then(|b| b.try_into().ok())
                    {
                        Some(ab) => ab,
                        None => return f64::MAX,
                    };
                    let alpha = f64::from(f32::from_le_bytes(alpha_bytes));
                    let factor = if alpha == 0.0 {
                        1.0
                    } else {
                        let denom = 2.0 - (1.0 / alpha);
                        if denom.abs() > 1e-12 {
                            2.0 / denom
                        } else {
                            0.0
                        }
                    };
                    return f64::from(hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                        &bytes[..n * 4],
                        factor as f32,
                        &qk_f32[..n],
                    ));
                }
            }
        }
        if let Some(qk) = query_klein {
            let m_name = M::name();
            if m_name == "lorentz" {
                if self.storage_f32 {
                    let n = (bytes.len().saturating_sub(4) / 4).min(qk.coords.len());
                    if n <= 1 || bytes.len() < 4 {
                        return 0.0;
                    }
                    let t_u = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).max(1.0);
                    let scale = 1.0 / t_u;
                    if let Some(qk_f32) = query_klein_f32 {
                        return f64::from(hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                            &bytes[4..n * 4],
                            scale,
                            &qk_f32[1..n],
                        ));
                    }
                    let scale64 = f64::from(scale);
                    let mut sum = 0.0;
                    for (c, &qk_val) in bytes[4..n * 4].chunks_exact(4).zip(qk.coords[1..n].iter())
                    {
                        let x_i = f64::from(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
                        let diff = x_i * scale64 - qk_val;
                        sum += diff * diff;
                    }
                    return sum;
                }
                let n = (bytes.len().saturating_sub(8) / 8).min(qk.coords.len());
                if n <= 1 || bytes.len() < 8 {
                    return 0.0;
                }
                let t_u = f64::from_le_bytes(bytes[0..8].try_into().unwrap_or([0; 8])).max(1.0);
                let scale = 1.0 / t_u;
                let mut sum = 0.0;
                for (c, &qk_val) in bytes[8..n * 8].chunks_exact(8).zip(qk.coords[1..n].iter()) {
                    let x_i = f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
                    let diff = x_i * scale - qk_val;
                    sum += diff * diff;
                }
                return sum;
            } else if m_name == "hybrid" && self.storage_f32 {
                let n = (bytes.len().saturating_sub(4) / 4).min(qk.coords.len());
                if n < 33 || bytes.len() < 132 {
                    return 0.0;
                }
                let t_u = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).max(1.0);
                let scale = 1.0 / t_u;
                if let Some(qk_f32) = query_klein_f32 {
                    let d_k = hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                        &bytes[4..132],
                        scale,
                        &qk_f32[1..33],
                    );
                    let d_e = if n > 33 {
                        hyperspace_core::simd::l2_sq_f32_bytes(
                            &bytes[132..n * 4],
                            &qk_f32[33..n],
                            None,
                        )
                    } else {
                        0.0
                    };
                    return f64::from(d_k + d_e);
                }
            }
            if self.storage_f32 {
                let n = (bytes.len().saturating_sub(4) / 4).min(qk.coords.len());
                let alpha_bytes: [u8; 4] = match bytes
                    .get(bytes.len().saturating_sub(4)..)
                    .and_then(|b| b.try_into().ok())
                {
                    Some(ab) => ab,
                    None => return f64::MAX,
                };
                let alpha = f64::from(f32::from_le_bytes(alpha_bytes));
                let factor = if alpha == 0.0 {
                    1.0
                } else {
                    let denom = 2.0 - (1.0 / alpha);
                    if denom.abs() > 1e-12 {
                        2.0 / denom
                    } else {
                        0.0
                    }
                };

                if let Some(qk_f32) = query_klein_f32 {
                    return f64::from(hyperspace_core::simd::scaled_l2_sq_f32_bytes(
                        &bytes[..n * 4],
                        factor as f32,
                        &qk_f32[..n],
                    ));
                }

                let mut sum = 0.0;
                for (c, &qk_val) in bytes[..n * 4].chunks_exact(4).zip(qk.coords[..n].iter()) {
                    let p_i = f64::from(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
                    let k_i = p_i * factor;
                    let diff = k_i - qk_val;
                    sum += diff * diff;
                }
                return sum;
            }
            let n = (bytes.len().saturating_sub(8) / 8).min(qk.coords.len());
            let alpha_bytes: [u8; 8] = match bytes
                .get(bytes.len().saturating_sub(8)..)
                .and_then(|b| b.try_into().ok())
            {
                Some(ab) => ab,
                None => return f64::MAX,
            };
            let alpha = f64::from_le_bytes(alpha_bytes);
            let factor = if alpha == 0.0 {
                1.0
            } else {
                let denom = 2.0 - (1.0 / alpha);
                if denom.abs() > 1e-12 {
                    2.0 / denom
                } else {
                    0.0
                }
            };

            let mut sum = 0.0;
            for (c, &qk_val) in bytes[..n * 8].chunks_exact(8).zip(qk.coords[..n].iter()) {
                let p_i = f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
                let k_i = p_i * factor;
                let diff = k_i - qk_val;
                sum += diff * diff;
            }
            return sum;
        }
        self.dist_bytes_opt(bytes, query, query_f32, pq_lut, mrl_dim, None)
    }

    #[allow(clippy::cast_sign_loss)]
    fn filtered_bruteforce_threshold(&self, nodes_count: usize, ef: usize) -> u64 {
        // FIX #7: Cache via OnceLock — env::var() is a syscall with a global mutex.
        // Without caching this is called on every filtered search.
        static ENV_THRESHOLD: std::sync::OnceLock<Option<u64>> = std::sync::OnceLock::new();
        let env_override = *ENV_THRESHOLD.get_or_init(|| {
            std::env::var("HS_FILTER_BRUTEFORCE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
        });

        if let Some(t) = env_override {
            if t > 0 {
                return t;
            }
        }

        if nodes_count == 0 {
            return 0;
        }

        // Small collection heuristic: for small graphs (<= 2000 nodes),
        // flat SIMD scan is universally faster than graph traversal.
        if nodes_count <= 2000 {
            return nodes_count as u64;
        }

        // Adaptive cost model:
        // HNSW distance evaluations with selectivity s = |allowed| / N scale as O(ef * M / s).
        // Flat parallel scan scales as O(|allowed|).
        // Equating costs: |allowed|^2 ~ C * N * ef * M  =>  |allowed| ~ sqrt(N * ef * M) * factor.
        // Parallel linear scan with SIMD early-abort and prefetch is ~1.5x-2.0x cheaper per vector
        // than random pointer-chasing in graph nodes.
        let m = (self
            .config
            .m
            .load(std::sync::atomic::Ordering::Relaxed)
            .max(4)) as f64;
        let ef_f = (ef.max(16)) as f64;
        let n_f = nodes_count as f64;

        let base = (n_f * ef_f * m).sqrt() * 1.75;
        let max_threshold = (nodes_count / 2).max(2000) as u64;
        let min_threshold = 1000u64.min(nodes_count as u64);

        (base as u64).clamp(min_threshold, max_threshold)
    }

    /// Upper bound on live nodes for which an unfiltered search scans the whole index
    /// instead of walking the graph (`HS_FLAT_SCAN`: `auto` (default), `off`, or a node
    /// count). `auto` uses `ef · 2M`: a layer-0 beam of width `ef` over nodes with up to
    /// `2M` neighbors evaluates on the order of that many distances before it converges,
    /// so below it the graph walk visits most of the index anyway — with random instead of
    /// sequential memory access. Capped at 32,768 nodes.
    fn flat_scan_limit(&self, params: &hyperspace_core::SearchParams) -> usize {
        static MODE: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
        const CAP: usize = 32_768;
        let fixed = *MODE.get_or_init(|| match std::env::var("HS_FLAT_SCAN") {
            Ok(v) => match v.trim().to_ascii_lowercase().as_str() {
                "" | "auto" | "on" | "true" => None,
                "off" | "false" | "0" | "no" => Some(0),
                n => n.parse::<usize>().ok().or(None),
            },
            Err(_) => None,
        });
        let forced = FLAT_SCAN_OVERRIDE.load(Ordering::Relaxed);
        if forced >= 0 {
            return usize::try_from(forced).unwrap_or(0);
        }
        if let Some(limit) = fixed {
            return limit;
        }
        let ef = if params.ef_search > 0 {
            params.ef_search
        } else {
            self.config.get_ef_search()
        }
        .max(params.top_k);
        ef.saturating_mul(2 * self.config.get_m().max(1)).min(CAP)
    }

    /// Whether an unfiltered search should scan the whole index (see [`Self::flat_scan_limit`]).
    /// Keeps every `QUERY_SAMPLE_EVERY`-th query for beam tuning (never blocks a search).
    fn sample_query(&self, query: &[f64]) {
        let seen = self.queries_seen.fetch_add(1, Ordering::Relaxed);
        if !seen.is_multiple_of(QUERY_SAMPLE_EVERY) {
            return;
        }
        if let Some(mut q) = self.query_samples.try_lock() {
            if q.len() >= QUERY_SAMPLES {
                q.pop_front();
            }
            q.push_back(query.to_vec());
        }
    }

    /// Sampled real queries collected since the last tuning that used them (all sampled
    /// queries when none did), and whether enough exist to tune with.
    #[must_use]
    pub fn fresh_query_samples(&self) -> (usize, bool) {
        let seen = self.queries_seen.load(Ordering::Relaxed);
        let last = self.tuned_queries_seen.load(Ordering::Relaxed);
        let fresh = if last == usize::MAX {
            seen
        } else {
            seen.saturating_sub(last)
        };
        (
            fresh / QUERY_SAMPLE_EVERY,
            self.query_samples.lock().len() >= MIN_REAL_QUERIES,
        )
    }

    /// Milliseconds since the last [`Self::tune_ef_search`] (`u64::MAX` if never).
    #[must_use]
    pub fn millis_since_tuning(&self) -> u64 {
        match self.tuned_at_ms.load(Ordering::Relaxed) {
            0 => u64::MAX,
            t => process_millis().saturating_sub(t),
        }
    }

    /// Whether the last tuning used real (sampled) queries.
    #[must_use]
    pub fn tuned_with_real_queries(&self) -> bool {
        self.tuned_queries_seen.load(Ordering::Relaxed) != usize::MAX
    }

    /// Beam width found by the last [`Self::tune_ef_search`] (0 = never tuned).
    #[must_use]
    pub fn tuned_ef(&self) -> usize {
        self.tuned_ef.load(Ordering::Relaxed)
    }

    /// Records that the beam was evaluated at `live` nodes without a result (flat-scanned
    /// or too small), so the caller waits for growth before measuring again.
    pub fn mark_ef_tuned(&self, live: usize) {
        self.tuned_at.store(live.max(1), Ordering::Relaxed);
    }

    /// Live node count at the last [`Self::tune_ef_search`].
    #[must_use]
    pub fn tuned_at(&self) -> usize {
        self.tuned_at.load(Ordering::Relaxed)
    }

    fn live_count(&self) -> usize {
        let nodes = self.nodes.count().min(self.storage.count());
        let deleted = if self.metadata.has_deletions.load(Ordering::Relaxed) {
            self.metadata.deleted.read().len() as usize
        } else {
            0
        };
        nodes.saturating_sub(deleted)
    }

    /// Measures, on recent real queries (once `MIN_REAL_QUERIES` are sampled) or else on
    /// `samples` held-out-like queries (local centroids of stored vectors), which share of
    /// the exact top-`pool` (flat scan of the same codes) the graph search returns among
    /// its top-`pool` candidates, and stores the smallest beam from a fixed ladder that
    /// reaches `target`. `pool > k` models a reranked search: the true top `k` can sit
    /// anywhere in the code ranking's first `top_k × oversample`, so the graph must find
    /// that whole candidate list, not just its first `k`. A graph over more nodes
    /// needs a wider beam for the same recall, and how much depends on the data, so the
    /// beam is measured instead of guessed. Returns `None` when the index is small enough
    /// to be flat-scanned (no beam involved) or has too few nodes to sample.
    pub fn tune_ef_search(
        &self,
        k: usize,
        pool: usize,
        target: f64,
        samples: usize,
    ) -> Option<usize> {
        const LADDER: [usize; 14] = [
            16, 24, 32, 48, 64, 96, 128, 160, 192, 256, 384, 512, 768, 1024,
        ];
        let k = k.max(1);
        let pool = pool.max(k);
        let live = self.live_count();
        if live < (pool + 1) * 8 || samples == 0 {
            return None;
        }
        let probe = hyperspace_core::SearchParams {
            top_k: pool + 1,
            ef_search: LADDER[0].max(pool + 1),
            ..Default::default()
        };
        if self.flat_scan_applies(&probe) {
            return None;
        }
        let nodes = self.nodes.count().min(self.storage.count());
        let step = (nodes / samples).max(1);
        let ids: Vec<NodeId> = {
            let deleted = self.metadata.deleted.read();
            (0..nodes)
                .step_by(step)
                .map(|i| i as NodeId)
                .filter(|id| !deleted.contains(*id))
                .take(samples)
                .collect()
        };
        // Held-out-like queries: the centroid of a stored vector and its nearest stored
        // neighbours. A stored vector itself is a graph node linked to its neighbours (an
        // easy query that overstates recall); a local centroid is not a node and, on
        // diffuse clusters, approaches the cluster centre — where real queries often land
        // and where the graph is weakest.
        let nn_probe = hyperspace_core::SearchParams {
            top_k: TUNE_NEIGHBOURHOOD,
            ef_search: TUNE_NEIGHBOURHOOD,
            ..Default::default()
        };
        let real: Vec<Vec<f64>> = self.query_samples.lock().iter().cloned().collect();
        let seen_now = self.queries_seen.load(Ordering::Relaxed);
        let use_real = real.len() >= MIN_REAL_QUERIES;
        let mut queries: Vec<Vec<f64>> = if use_real {
            real
        } else {
            Vec::with_capacity(ids.len())
        };
        for id in ids.into_iter().filter(|_| !use_real) {
            let x = self.get_vector(id).coords;
            if M::validate(&x).is_err() {
                continue;
            }
            let near = self.search_impl(&x, None, &nn_probe, ScanMode::Flat);
            let mut q = x.clone();
            let mut members = 1.0;
            for r in near.iter().filter(|r| r.0 != id) {
                for (a, b) in q.iter_mut().zip(self.get_vector(r.0).coords) {
                    *a += b;
                }
                members += 1.0;
            }
            for v in &mut q {
                *v /= members;
            }
            if M::name() == "cosine" {
                let n = q.iter().map(|v| v * v).sum::<f64>().sqrt().max(1e-12);
                for v in &mut q {
                    *v /= n;
                }
            }
            if M::validate(&q).is_ok() {
                queries.push(q);
            }
        }
        if queries.is_empty() {
            return None;
        }
        let top = |res: Vec<(NodeId, f64)>, n: usize| -> Vec<NodeId> {
            res.into_iter().map(|r| r.0).take(n).collect()
        };
        let truth: Vec<std::collections::HashSet<NodeId>> = queries
            .iter()
            .map(|q| {
                top(self.search_impl(q, None, &probe, ScanMode::Flat), pool)
                    .into_iter()
                    .collect()
            })
            .collect();
        let mut chosen = *LADDER.last().unwrap_or(&1024);
        for &ef in LADDER.iter().filter(|&&ef| ef > pool) {
            let p = hyperspace_core::SearchParams {
                ef_search: ef,
                ..probe.clone()
            };
            let (mut hit, mut total) = (0usize, 0usize);
            for (q, t) in queries.iter().zip(&truth) {
                let got = top(self.search_impl(q, None, &p, ScanMode::Graph), pool);
                hit += got.iter().filter(|x| t.contains(x)).count();
                total += t.len();
            }
            if total == 0 || hit as f64 >= target * total as f64 {
                chosen = ef;
                break;
            }
        }
        self.tuned_ef.store(chosen, Ordering::Relaxed);
        self.tuned_at.store(live, Ordering::Relaxed);
        self.tuned_at_ms.store(process_millis(), Ordering::Relaxed);
        if use_real {
            self.tuned_queries_seen.store(seen_now, Ordering::Relaxed);
        }
        Some(chosen)
    }

    fn flat_scan_applies(&self, params: &hyperspace_core::SearchParams) -> bool {
        if self.zonal
            || params.hybrid_query.is_some()
            // KAG-guided traversal expands knowledge-graph edges during the graph walk.
            || self.graph_engine.as_ref().is_some_and(|g| g.config.enabled)
        {
            return false;
        }
        let nodes = self.nodes.count().min(self.storage.count());
        let deleted = if self.metadata.has_deletions.load(Ordering::Relaxed) {
            self.metadata.deleted.read().len() as usize
        } else {
            0
        };
        let live = nodes.saturating_sub(deleted);
        live > 0 && live <= self.flat_scan_limit(params)
    }

    /// Exact top-`k` over every live node (sequential scan, early-abandon threshold).
    /// Returns internal distances, ascending.
    fn search_flat_all(
        &self,
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        k: usize,
        prep: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> Vec<(NodeId, f64)> {
        if k == 0 {
            return Vec::new();
        }
        let storage_view = self.storage.view();
        let n = self.nodes.count().min(storage_view.count()) as u32;
        let deleted = self
            .metadata
            .has_deletions
            .load(Ordering::Relaxed)
            .then(|| self.metadata.deleted.read());
        let deleted = deleted.as_deref().filter(|d| !d.is_empty());
        let kernel = self.kernel_for(query, query_f32, prep, mrl_dim);
        let bias = Self::dist_bias();
        let scan = |lo: u32, hi: u32| -> BinaryHeap<MaxCandidate> {
            let mut heap: BinaryHeap<MaxCandidate> = BinaryHeap::with_capacity(k + 1);
            let mut worst: Option<f64> = None;
            for id in lo..hi {
                if deleted.is_some_and(|d| d.contains(id)) {
                    continue;
                }
                if id + 4 < hi {
                    prefetch_bytes(storage_view.get(id + 4));
                }
                let d = self.dist_k(
                    kernel,
                    storage_view.get(id),
                    query,
                    query_f32,
                    prep,
                    mrl_dim,
                    worst,
                );
                if heap.len() < k {
                    heap.push(MaxCandidate::biased(id, d, bias));
                    if heap.len() == k {
                        worst = heap.peek().map(|w| w.unbiased(bias));
                    }
                } else if worst.is_some_and(|w| d < w) {
                    heap.pop();
                    heap.push(MaxCandidate::biased(id, d, bias));
                    worst = heap.peek().map(|w| w.unbiased(bias));
                }
            }
            heap
        };
        // Idle server: split the scan across cores (a scan is memory-bound per core, so a
        // single query gets the latency of a sharded scan). Under load every core already
        // serves another query and the scan stays on the calling thread.
        let parts = flat_scan_parallelism(n as usize);
        let heap = if parts > 1 {
            let chunk = n.div_ceil(parts as u32);
            let heaps: Vec<BinaryHeap<MaxCandidate>> = (0..parts as u32)
                .into_par_iter()
                .map(|p| scan(p * chunk, ((p + 1) * chunk).min(n)))
                .collect();
            let mut merged: BinaryHeap<MaxCandidate> = BinaryHeap::with_capacity(k + 1);
            for c in heaps.into_iter().flat_map(BinaryHeap::into_vec) {
                if merged.len() < k {
                    merged.push(c);
                } else if merged.peek().is_some_and(|w| c.distance < w.distance) {
                    merged.pop();
                    merged.push(c);
                }
            }
            merged
        } else {
            scan(0, n)
        };
        let mut out: Vec<(NodeId, f64)> = heap
            .into_iter()
            .map(|c| (c.id(), c.unbiased(bias)))
            .collect();
        out.sort_by(|a, b| a.1.total_cmp(&b.1));
        out
    }

    fn search_bruteforce_bitmap(
        &self,
        query: &HyperVector,
        k: usize,
        allowed: &RoaringBitmap,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> Vec<(NodeId, f64)> {
        const PARALLEL_THRESHOLD: usize = 4096;

        if k == 0 || allowed.is_empty() {
            return Vec::new();
        }
        let nodes_len = self.nodes.count() as u32;

        let local_lut = if pq_lut.is_none() {
            self.prepare_query(query, None)
        } else {
            None
        };
        let effective_lut = pq_lut.or(local_lut.as_ref());

        let q_f32_vec = if self.needs_query_f32() {
            Some(query.coords.iter().map(|&x| x as f32).collect::<Vec<_>>())
        } else {
            None
        };
        let q_f32_ref = q_f32_vec.as_deref();
        let kernel = self.kernel_for(query, q_f32_ref, effective_lut, mrl_dim);

        let allowed_len = allowed.len() as usize;
        let mut final_heap: BinaryHeap<MaxCandidate>;

        if allowed_len <= k * 2 {
            let storage_view = self.storage.view();
            let mut out = Vec::with_capacity(allowed_len);
            for id in allowed {
                if id >= nodes_len {
                    continue;
                }
                let bytes = storage_view.get(id);
                let d = self.dist_k(
                    kernel,
                    bytes,
                    query,
                    q_f32_ref,
                    effective_lut,
                    mrl_dim,
                    None,
                );
                out.push((id, d));
            }
            out.sort_by(|a, b| a.1.total_cmp(&b.1));
            out.truncate(k);
            // Internal distances: `search_with_allowed` finalizes them once.
            return out;
        }

        if allowed_len < PARALLEL_THRESHOLD {
            // Fast single-threaded sequential scan with prefetch and early-abort threshold
            let storage_view = self.storage.view();
            let mut heap = BinaryHeap::with_capacity(k + 1);
            let ids: Vec<u32> = allowed.iter().collect();
            let n_ids = ids.len();

            for (idx, &id) in ids.iter().enumerate() {
                if id >= nodes_len {
                    continue;
                }
                if idx + 4 < n_ids {
                    let pf_id = ids[idx + 4];
                    if pf_id < nodes_len {
                        prefetch_bytes(storage_view.get(pf_id));
                    }
                }
                let thresh = if heap.len() >= k {
                    heap.peek()
                        .map(|w: &MaxCandidate| w.unbiased(Self::dist_bias()))
                } else {
                    None
                };
                let bytes = storage_view.get(id);
                let d = self.dist_k(
                    kernel,
                    bytes,
                    query,
                    q_f32_ref,
                    effective_lut,
                    mrl_dim,
                    thresh,
                );

                if heap.len() < k {
                    heap.push(MaxCandidate::biased(id, d, Self::dist_bias()));
                } else if let Some(worst) = heap.peek() {
                    if d < worst.unbiased(Self::dist_bias()) {
                        heap.pop();
                        heap.push(MaxCandidate::biased(id, d, Self::dist_bias()));
                    }
                }
            }
            final_heap = heap;
        } else {
            // Parallel chunked scan across CPU cores using Rayon
            let ids: Vec<u32> = allowed.iter().collect();
            let num_threads = rayon::current_num_threads().max(1);
            let chunk_size = (ids.len() / (num_threads * 2)).max(1024);

            final_heap = ids
                .par_chunks(chunk_size)
                .map(|chunk| {
                    let storage_view = self.storage.view();
                    let mut local_heap = BinaryHeap::with_capacity(k + 1);
                    let n_chunk = chunk.len();

                    for (idx, &id) in chunk.iter().enumerate() {
                        if id >= nodes_len {
                            continue;
                        }
                        if idx + 4 < n_chunk {
                            let pf_id = chunk[idx + 4];
                            if pf_id < nodes_len {
                                prefetch_bytes(storage_view.get(pf_id));
                            }
                        }
                        let thresh = if local_heap.len() >= k {
                            local_heap
                                .peek()
                                .map(|w: &MaxCandidate| w.unbiased(Self::dist_bias()))
                        } else {
                            None
                        };
                        let bytes = storage_view.get(id);
                        let d = self.dist_k(
                            kernel,
                            bytes,
                            query,
                            q_f32_ref,
                            effective_lut,
                            mrl_dim,
                            thresh,
                        );

                        if local_heap.len() < k {
                            local_heap.push(MaxCandidate::biased(id, d, Self::dist_bias()));
                        } else if let Some(worst) = local_heap.peek() {
                            if d < worst.unbiased(Self::dist_bias()) {
                                local_heap.pop();
                                local_heap.push(MaxCandidate::biased(id, d, Self::dist_bias()));
                            }
                        }
                    }
                    local_heap
                })
                .reduce(
                    || BinaryHeap::with_capacity(k + 1),
                    |mut h1, h2| {
                        for c in h2 {
                            if h1.len() < k {
                                h1.push(c);
                            } else if let Some(worst) = h1.peek() {
                                if c.unbiased(Self::dist_bias()) < worst.unbiased(Self::dist_bias())
                                {
                                    h1.pop();
                                    h1.push(c);
                                }
                            }
                        }
                        h1
                    },
                );
        }

        let mut out = Vec::with_capacity(final_heap.len());
        while let Some(c) = final_heap.pop() {
            out.push((c.id(), c.unbiased(Self::dist_bias())));
        }
        out.reverse();
        // Internal distances: `search_with_allowed` finalizes them once.
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn search_layer0_flat<F>(
        &self,
        flat_l0: &FlatLayer0Segment,
        start_node: NodeId,
        query: &HyperVector,
        q_f32_ref: Option<&[f32]>,
        k: usize,
        ef: usize,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
        is_valid: &F,
        policy: TraversalPolicy,
        scratch: &mut VisitedScratch,
    ) -> Vec<(NodeId, f64)>
    where
        F: Fn(u32) -> bool,
    {
        let nodes_count = flat_l0.node_count();
        scratch.prepare(nodes_count);

        let ef_capacity = ef.max(k).max(16);
        let mut candidates = std::mem::take(&mut scratch.candidates_l0);
        let mut results = std::mem::take(&mut scratch.results_l0);

        candidates.clear();
        results.clear();
        if candidates.capacity() < ef_capacity {
            candidates.reserve(ef_capacity - candidates.capacity());
        }
        if results.capacity() < ef_capacity {
            results.reserve(ef_capacity - results.capacity());
        }

        let kernel = self.kernel_for(query, q_f32_ref, pq_lut, mrl_dim);
        let start_vec = flat_l0.get_vector(start_node);
        let d = self.dist_k(kernel, start_vec, query, q_f32_ref, pq_lut, mrl_dim, None);
        let first = Candidate::biased(start_node, d, Self::dist_bias());

        candidates.push(first);
        if is_valid(start_node) {
            results.push(std::cmp::Reverse(first));
        }
        let _ = scratch.mark_visited(start_node);

        let mut unvisited = SmallVec::<[NodeId; 64]>::new();
        let mut neighbors = SmallVec::<[NodeId; 64]>::new();
        let mut hop = SmallVec::<[NodeId; 64]>::new();
        let mut stale = 0usize;
        let mut thresh = if results.len() >= ef {
            results
                .peek()
                .map(|worst| worst.0.unbiased(Self::dist_bias()))
        } else {
            None
        };
        while let Some(cand) = candidates.pop() {
            if let Some(worst_dist) = thresh {
                if cand.unbiased(Self::dist_bias()) > worst_dist {
                    break;
                }
            }

            let cand_id = cand.id();
            if (cand_id as usize) >= nodes_count {
                continue;
            }

            // Seqlock-validated snapshot (no torn reads while add_link/prune update in place).
            flat_l0.read_neighbors_into(cand_id, &mut neighbors);
            unvisited.clear();
            for &neighbor in &neighbors {
                if (neighbor as usize) >= nodes_count || !scratch.mark_visited(neighbor) {
                    continue;
                }
                if policy.acorn && !is_valid(neighbor) {
                    flat_l0.read_neighbors_into(neighbor, &mut hop);
                    for &h in &hop {
                        if (h as usize) < nodes_count && is_valid(h) && scratch.mark_visited(h) {
                            unvisited.push(h);
                        }
                    }
                    continue;
                }
                unvisited.push(neighbor);
            }
            let mut improved = false;

            let u_len = unvisited.len();
            // Prefetch initial unvisited slots
            for &pf_id in unvisited.iter().take(3) {
                prefetch_bytes(flat_l0.get_vector(pf_id));
            }

            for (idx, &neighbor) in unvisited.iter().enumerate() {
                if idx + 3 < u_len {
                    prefetch_bytes(flat_l0.get_vector(unvisited[idx + 3]));
                }

                let neighbor_vec = flat_l0.get_vector(neighbor);
                let dist = self.dist_k(
                    kernel,
                    neighbor_vec,
                    query,
                    q_f32_ref,
                    pq_lut,
                    mrl_dim,
                    thresh,
                );

                if thresh.is_none_or(|worst_dist| dist <= worst_dist) {
                    let c = Candidate::biased(neighbor, dist, Self::dist_bias());
                    candidates.push(c);

                    if is_valid(neighbor) {
                        results.push(std::cmp::Reverse(c));
                        improved = true;
                        if results.len() > ef {
                            results.pop();
                        }
                        if results.len() >= ef {
                            thresh = results
                                .peek()
                                .map(|worst| worst.0.unbiased(Self::dist_bias()));
                        }
                    }
                }
            }
            if improved {
                stale = 0;
            } else {
                stale += 1;
                if policy.patience > 0 && stale >= policy.patience && results.len() >= k {
                    break;
                }
            }
        }

        let mut output = Vec::with_capacity(k.min(results.len()));
        while let Some(std::cmp::Reverse(c)) = results.pop() {
            output.push((c.id(), c.unbiased(Self::dist_bias())));
        }
        output.reverse();
        output.truncate(k);

        candidates.clear();
        results.clear();
        scratch.candidates_l0 = candidates;
        scratch.results_l0 = results;
        output
    }

    #[allow(clippy::too_many_arguments)]
    fn search_layer0(
        &self,
        start_node: NodeId,
        query: &HyperVector,
        caller_q_f32: Option<&[f32]>,
        k: usize,
        ef: usize,
        allowed: Option<&RoaringBitmap>,
        pq_lut: Option<&QueryPrep>,
        mrl_dim: Option<usize>,
    ) -> Vec<(NodeId, f64)> {
        // LOCK-FREE: boxcar::Vec — no global read lock needed.
        let nodes_count = self.nodes.count();

        // Fast-path: Only acquire `deleted.read()` when no explicit allow-list is given
        // AND deletions have actually occurred (or bitmap is non-empty).
        let deleted_guard =
            if allowed.is_none() && self.metadata.has_deletions.load(Ordering::Relaxed) {
                let guard = self.metadata.deleted.read();
                if guard.is_empty() {
                    None
                } else {
                    Some(guard)
                }
            } else {
                None
            };

        let policy = TraversalPolicy {
            acorn: Self::acorn_enabled()
                && allowed.is_some_and(|bm| (bm.len() as f64) < 0.5 * nodes_count as f64),
            patience: Self::search_patience(),
        };

        let is_valid = |id: u32| -> bool {
            if let Some(bm) = allowed {
                bm.contains(id)
            } else if let Some(ref del) = deleted_guard {
                !del.contains(id)
            } else {
                true
            }
        };

        if let Some(allowed_bitmap) = allowed {
            if allowed_bitmap.len() <= self.filtered_bruteforce_threshold(nodes_count, ef) {
                return self.search_bruteforce_bitmap(query, k, allowed_bitmap, pq_lut, mrl_dim);
            }
        }

        // Safety check start_node
        if (start_node as usize) >= nodes_count {
            return vec![];
        }

        VISITED_SCRATCH.with(|scratch_cell| {
            let mut scratch = scratch_cell.borrow_mut();

            let ef_capacity = ef.max(k).max(16);
            let mut candidates = std::mem::take(&mut scratch.candidates_l0);
            let mut results = std::mem::take(&mut scratch.results_l0);

            candidates.clear();
            results.clear();
            if candidates.capacity() < ef_capacity {
                candidates.reserve(ef_capacity - candidates.capacity());
            }
            if results.capacity() < ef_capacity {
                results.reserve(ef_capacity - results.capacity());
            }

            let needs_f32 = self.needs_query_f32();
            let mut query_f32 = std::mem::take(&mut scratch.query_f32);
            if needs_f32 && caller_q_f32.is_none() {
                query_f32.clear();
                query_f32.extend(query.coords.iter().map(|&x| x as f32));
            }
            let q_f32_ref = caller_q_f32.or({
                if needs_f32 {
                    Some(query_f32.as_slice())
                } else {
                    None
                }
            });

            let flat_guard = self.flat_l0.load();
            if let Some(ref flat_l0) = *flat_guard {
                // Only use the frozen segment while it covers every live node; nodes
                // appended after the freeze would otherwise be unreachable (recall loss).
                if (start_node as usize) < flat_l0.node_count()
                    && flat_l0.node_count() >= nodes_count
                {
                    scratch.candidates_l0 = candidates;
                    scratch.results_l0 = results;
                    let out = self.search_layer0_flat(
                        flat_l0,
                        start_node,
                        query,
                        q_f32_ref,
                        k,
                        ef,
                        pq_lut,
                        mrl_dim,
                        &is_valid,
                        policy,
                        &mut scratch,
                    );
                    scratch.query_f32 = query_f32;
                    return out;
                }
            }

            scratch.prepare(nodes_count);
            let d = self.dist_mrl_opt(start_node, query, q_f32_ref, pq_lut, mrl_dim, None);
            let first = Candidate::biased(start_node, d, Self::dist_bias());

            candidates.push(first);
            if is_valid(start_node) {
                results.push(std::cmp::Reverse(first));
            }
            let _ = scratch.mark_visited(start_node);

            let storage_view = self.storage.view();
            let kernel = self.kernel_for(query, q_f32_ref, pq_lut, mrl_dim);
            let csr_snap = self
                .graph_engine
                .as_ref()
                .filter(|ge| ge.config.enabled)
                .map(|ge| ge.csr_snapshot.read().clone());
            let mut unvisited = SmallVec::<[NodeId; 64]>::new();
            let l0_view = self.layer0.view();
            let mut neighbor_buf = layer0::Links::new();
            let mut hop_buf = layer0::Links::new();
            let mut stale = 0usize;
            let mut kag_targets = SmallVec::<[(NodeId, f32); 8]>::new();
            let mut thresh = if results.len() >= ef {
                results
                    .peek()
                    .map(|worst| worst.0.unbiased(Self::dist_bias()))
            } else {
                None
            };
            while let Some(cand) = candidates.pop() {
                // Lower Bound Pruning:
                if let Some(worst_dist) = thresh {
                    if cand.unbiased(Self::dist_bias()) > worst_dist {
                        break;
                    }
                }

                let cand_id = cand.id();
                if (cand_id as usize) >= nodes_count {
                    continue;
                }

                // LOCK-FREE: seqlocked flat layer-0 slot (no per-node lock / pointer chase)
                l0_view.read_into(cand_id, &mut neighbor_buf);
                // Warm the adjacency slot of the next beam candidate while we compute distances.
                if let Some(next) = candidates.peek() {
                    prefetch_l1(l0_view.slot_ptr(next.id()));
                }
                unvisited.clear();
                for &neighbor in &neighbor_buf {
                    if (neighbor as usize) >= nodes_count || !scratch.mark_visited(neighbor) {
                        continue;
                    }
                    if policy.acorn && !is_valid(neighbor) {
                        // ACORN 2-hop: route through the filtered-out node without scoring it.
                        l0_view.read_into(neighbor, &mut hop_buf);
                        for &h in &hop_buf {
                            if (h as usize) < nodes_count && is_valid(h) && scratch.mark_visited(h)
                            {
                                unvisited.push(h);
                            }
                        }
                        continue;
                    }
                    unvisited.push(neighbor);
                }
                let mut improved = false;

                let u_len = unvisited.len();
                for &pf_id in unvisited.iter().take(3) {
                    prefetch_bytes(storage_view.get(pf_id));
                }

                for (idx, &neighbor) in unvisited.iter().enumerate() {
                    if idx + 3 < u_len {
                        let lookahead_n = unvisited[idx + 3];
                        prefetch_bytes(storage_view.get(lookahead_n));
                    }

                    let neighbor_bytes = storage_view.get(neighbor);
                    let dist = self.dist_k(
                        kernel,
                        neighbor_bytes,
                        query,
                        q_f32_ref,
                        pq_lut,
                        mrl_dim,
                        thresh,
                    );

                    if thresh.is_none_or(|worst_dist| dist <= worst_dist) {
                        let c = Candidate::biased(neighbor, dist, Self::dist_bias());
                        candidates.push(c);

                        if is_valid(neighbor) {
                            results.push(std::cmp::Reverse(c));
                            improved = true;
                            if results.len() > ef {
                                results.pop();
                            }
                            if results.len() >= ef {
                                thresh = results
                                    .peek()
                                    .map(|worst| worst.0.unbiased(Self::dist_bias()));
                            }
                        }
                    }
                }

                // KAG-Guided HNSW Traversal: expand high-affinity knowledge graph edges of strong beam nodes
                if let Some(ref csr) = csr_snap {
                    kag_targets.clear();
                    for edge in csr.neighbors(cand_id) {
                        if edge.is_active()
                            && (edge.target as usize) < nodes_count
                            && scratch.mark_visited(edge.target)
                        {
                            kag_targets.push((edge.target, edge.weight.max(0.1)));
                        }
                    }
                    for &(target, weight) in &kag_targets {
                        let raw_dist = self.dist_k(
                            kernel,
                            storage_view.get(target),
                            query,
                            q_f32_ref,
                            pq_lut,
                            mrl_dim,
                            None,
                        );
                        // Topological affinity bonus bridges the semantic gap in a single pass.
                        // The boost only steers expansion order; results keep the true distance
                        // so returned scores and the pruning threshold stay metric-consistent.
                        let bias = Self::dist_bias();
                        let boosted_dist =
                            bias + (raw_dist - bias) / (1.0 + f64::from(weight) * 4.0);
                        candidates.push(Candidate::biased(target, boosted_dist, bias));
                        if is_valid(target)
                            && thresh.is_none_or(|worst_dist| raw_dist <= worst_dist)
                        {
                            results
                                .push(std::cmp::Reverse(Candidate::biased(target, raw_dist, bias)));
                            improved = true;
                            if results.len() > ef {
                                results.pop();
                            }
                            if results.len() >= ef {
                                thresh = results
                                    .peek()
                                    .map(|worst| worst.0.unbiased(Self::dist_bias()));
                            }
                        }
                    }
                }

                // Adaptive ef: stop once the beam has stopped producing better results.
                if improved {
                    stale = 0;
                } else {
                    stale += 1;
                    if policy.patience > 0 && stale >= policy.patience && results.len() >= k {
                        break;
                    }
                }
            }

            let mut output = Vec::with_capacity(k.min(results.len()));
            while let Some(std::cmp::Reverse(c)) = results.pop() {
                output.push((c.id(), c.unbiased(Self::dist_bias())));
            }
            output.reverse();
            output.truncate(k);

            candidates.clear();
            results.clear();
            scratch.candidates_l0 = candidates;
            scratch.results_l0 = results;
            scratch.query_f32 = query_f32;
            output
        })
    }

    // Search candidates on a layer (returns Heap instead of sorted vec)
    #[allow(clippy::too_many_arguments)]
    fn search_layer_candidates(
        &self,
        start_node: NodeId,
        query: &HyperVector,
        query_f32: Option<&[f32]>,
        query_klein: Option<&HyperVector>,
        query_klein_f32: Option<&[f32]>,
        pq_lut: Option<&QueryPrep>,
        level: usize,
        ef: usize,
    ) -> BinaryHeap<Candidate> {
        // LOCK-FREE: boxcar::Vec
        let nodes_count = self.nodes.count();

        if (start_node as usize) >= nodes_count {
            return BinaryHeap::new();
        }
        let Some(start_node_ref) = self.nodes.get(start_node as usize) else {
            return BinaryHeap::new();
        };
        if start_node_ref.level() < level {
            return BinaryHeap::new();
        }

        VISITED_SCRATCH.with(|scratch_cell| {
            let mut scratch = scratch_cell.borrow_mut();
            scratch.prepare(nodes_count);

            let ef_capacity = ef.max(16);
            let mut candidates = std::mem::take(&mut scratch.candidates_layer);
            let mut results = std::mem::take(&mut scratch.results_layer);

            candidates.clear();
            results.clear();
            if candidates.capacity() < ef_capacity {
                candidates.reserve(ef_capacity - candidates.capacity());
            }
            if results.capacity() < ef_capacity {
                results.reserve(ef_capacity - results.capacity());
            }

            let d = self.dist_upper(
                start_node,
                query,
                query_f32,
                query_klein,
                query_klein_f32,
                pq_lut,
                None,
            );
            let first = Candidate::biased(start_node, d, Self::dist_bias());

            candidates.push(first);
            results.push(std::cmp::Reverse(first));
            let _ = scratch.mark_visited(start_node);

            let storage_view = self.storage.view();
            let mut unvisited = SmallVec::<[NodeId; 64]>::new();
            let mut neighbor_buf = layer0::Links::new();
            while let Some(cand) = candidates.pop() {
                let mut curr_worst = results.peek().unwrap().0.unbiased(Self::dist_bias());
                if cand.unbiased(Self::dist_bias()) > curr_worst && results.len() >= ef {
                    break;
                }

                let cand_id = cand.id();
                if (cand_id as usize) >= nodes_count {
                    continue;
                }

                if !self.links_into(cand_id, level, &mut neighbor_buf) {
                    continue;
                }
                unvisited.clear();
                for &neighbor in &neighbor_buf {
                    if (neighbor as usize) < nodes_count && scratch.mark_visited(neighbor) {
                        unvisited.push(neighbor);
                    }
                }

                let u_len = unvisited.len();
                for &pf_id in unvisited.iter().take(3) {
                    prefetch_bytes(storage_view.get(pf_id));
                }

                let has_klein = query_klein.is_some() || query_klein_f32.is_some();
                let kernel = if has_klein {
                    DistKernel::Generic
                } else {
                    self.kernel_for(query, query_f32, pq_lut, None)
                };
                for (idx, &neighbor) in unvisited.iter().enumerate() {
                    if idx + 3 < u_len {
                        let lookahead_n = unvisited[idx + 3];
                        prefetch_bytes(storage_view.get(lookahead_n));
                    }

                    let neighbor_bytes = storage_view.get(neighbor);
                    let thresh = if results.len() >= ef && !has_klein {
                        Some(curr_worst)
                    } else {
                        None
                    };
                    let dist = if has_klein {
                        self.dist_upper_bytes(
                            neighbor_bytes,
                            query,
                            query_f32,
                            query_klein,
                            query_klein_f32,
                            pq_lut,
                            None,
                        )
                    } else {
                        self.dist_k(
                            kernel,
                            neighbor_bytes,
                            query,
                            query_f32,
                            pq_lut,
                            None,
                            thresh,
                        )
                    };

                    if results.len() < ef || dist < curr_worst {
                        let c = Candidate::biased(neighbor, dist, Self::dist_bias());
                        candidates.push(c);
                        results.push(std::cmp::Reverse(c));

                        if results.len() > ef {
                            results.pop();
                        }
                        if results.len() >= ef {
                            curr_worst = results.peek().unwrap().0.unbiased(Self::dist_bias());
                        }
                    }
                }
            }
            candidates.clear();
            scratch.candidates_layer = candidates;
            let mut out = BinaryHeap::with_capacity(results.len());
            for r in results.drain() {
                out.push(r.0);
            }
            results.clear();
            scratch.results_layer = results;
            out
        })
    }

    /// Decodes a stored node for pairwise distances in `select_neighbors` only: codes may be
    /// returned in an isometric (e.g. rotated) domain, not the original coordinates.
    fn load_node_coords_f32(
        &self,
        storage_view: &hyperspace_store::StorageView,
        id: NodeId,
        out: &mut Vec<f32>,
        alpha_out: &mut f64,
    ) {
        if id as usize >= storage_view.count() {
            out.clear();
            out.resize(self.dimension, 0.0);
            *alpha_out = 1.0;
            return;
        }

        let bytes = storage_view.get(id);
        if self.mode == QuantizationMode::None && self.storage_f32 {
            let n = (bytes.len().saturating_sub(4) / 4).min(self.dimension);
            out.clear();
            out.reserve(self.dimension);
            for c in bytes[..n * 4].chunks_exact(4) {
                out.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
            }
            if out.len() < self.dimension {
                out.resize(self.dimension, 0.0);
            }
            if bytes.len() >= 4 {
                let ab: [u8; 4] = bytes[bytes.len() - 4..].try_into().unwrap_or([0; 4]);
                *alpha_out = f64::from(f32::from_le_bytes(ab));
            } else {
                *alpha_out = 1.0;
            }
            return;
        }

        if self.mode == QuantizationMode::ScalarI8 {
            let alpha_offset = bytes.len().saturating_sub(4);
            let n = alpha_offset.min(self.dimension);
            let alpha = if bytes.len() >= 4 {
                let ab: [u8; 4] = bytes[alpha_offset..].try_into().unwrap_or([0; 4]);
                f32::from_le_bytes(ab)
            } else {
                1.0f32
            };
            out.clear();
            out.reserve(self.dimension);
            if M::name() == "lorentz" {
                let scale = alpha / 127.0;
                out.push(1.0);
                let mut norm_sq = 0.0f64;
                for &b in bytes.get(1..n).unwrap_or(&[]) {
                    let v = f32::from(b.cast_signed()) * scale;
                    norm_sq += f64::from(v) * f64::from(v);
                    out.push(v);
                }
                out[0] = (1.0 + norm_sq).sqrt() as f32;
            } else {
                let scale = self.sq8_step(alpha);
                for &b in &bytes[..n] {
                    out.push(f32::from(b.cast_signed()) * scale);
                }
            }
            if out.len() < self.dimension {
                out.resize(self.dimension, 0.0);
            }
            *alpha_out = if self.sq8_scaled {
                1.0
            } else {
                f64::from(alpha)
            };
            return;
        }

        if self.mode == QuantizationMode::AsymmetricHybrid801 && bytes.len() >= 132 {
            out.clear();
            out.reserve(self.dimension);
            for c in bytes[..132].chunks_exact(4) {
                out.push(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
            }
            let euc_end = bytes.len().saturating_sub(4);
            let euc_bytes = &bytes[132..euc_end];
            let scale = 1.0f32 / 127.0f32;
            for &b in euc_bytes.iter().take(self.dimension.saturating_sub(33)) {
                out.push(f32::from(b.cast_signed()) * scale);
            }
            if out.len() < self.dimension {
                out.resize(self.dimension, 0.0);
            }
            if bytes.len() >= 136 {
                let ab: [u8; 4] = bytes[euc_end..].try_into().unwrap_or([0; 4]);
                *alpha_out = f64::from(f32::from_le_bytes(ab));
            } else {
                *alpha_out = 1.0;
            }
            return;
        }

        if self.mode.rabitq_bits().is_some() {
            if let Some(ref rq) = *self.rabitq.load() {
                // Only pairwise distances are taken from these coordinates (RNG pruning), so
                // the rotated, centred decode is enough and skips the inverse FWHT.
                rq.reconstruct_rotated_into(bytes, out);
                *alpha_out = 1.0;
                return;
            }
        }
        if self.mode == QuantizationMode::PQ4 {
            if let Some(ref pq4) = *self.pq4.load() {
                pq4.decode_into(bytes, out);
                *alpha_out = 1.0;
                return;
            }
        }
        if matches!(
            self.mode,
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ
        ) {
            if let Some(ref pq) = *self.pq.load() {
                out.clear();
                out.resize(self.dimension, 0.0);
                let sub_dim = pq.subvector_dim;
                let m = pq.num_subvectors.min(bytes.len());
                for sub_idx in 0..m {
                    let code = bytes[sub_idx] as usize;
                    let sub_offset = sub_idx * sub_dim;
                    let c_offset = (sub_idx * 256 + code) * sub_dim;
                    let copy_len = sub_dim.min(self.dimension.saturating_sub(sub_offset));
                    out[sub_offset..sub_offset + copy_len]
                        .copy_from_slice(&pq.centroids[c_offset..c_offset + copy_len]);
                }
                // OPQ codebooks live in the rotated domain; pairwise distances are
                // rotation-invariant, so the inverse rotation is skipped here.
                *alpha_out = 1.0;
                return;
            }
        }

        QUERY_F64_SCRATCH.with(|cell| {
            let mut buf = cell.borrow_mut();
            self.with_vector_coords(id, &mut buf, |coords| {
                out.clear();
                out.reserve(self.dimension);
                for &c in coords.iter().take(self.dimension) {
                    out.push(c as f32);
                }
                if out.len() < self.dimension {
                    out.resize(self.dimension, 0.0);
                }
            });
        });
        *alpha_out = 1.0;
    }

    #[inline(always)]
    fn can_dist_storage_pair(&self) -> bool {
        if self.mode == QuantizationMode::None && self.storage_f32 {
            matches!(
                M::name(),
                "l2" | "cosine" | "poincare" | "lorentz" | "hybrid"
            )
        } else {
            self.sq8_scaled
                || (self.mode == QuantizationMode::ScalarI8 && M::name() == "lorentz")
                // 1-bit RaBitQ: popcount pair distance in the rotated domain. Multi-bit codes
                // decode to the rotated domain instead (`load_node_coords_f32`): an O(m)
                // unpack plus f32 SIMD pairs beats an O(m²) packed-integer pair kernel.
                || (self.mode == QuantizationMode::RaBitQ && matches!(M::name(), "l2" | "cosine"))
        }
    }

    #[inline]
    fn dist_storage_bytes_pair(
        &self,
        bytes_a: &[u8],
        bytes_b: &[u8],
        threshold: Option<f64>,
    ) -> f64 {
        if self.mode == QuantizationMode::ScalarI8 && M::name() == "lorentz" {
            return hyperspace_core::simd::sq8_lorentz_bytes_pair(bytes_a, bytes_b);
        }
        if self.sq8_scaled {
            return f64::from(hyperspace_core::simd::sq8_scaled_l2_pair(bytes_a, bytes_b));
        }
        if let Some(bits) = self.mode.rabitq_bits() {
            // Only reached for 1-bit (see `can_dist_storage_pair`); correct for every width.
            return f64::from(hyperspace_core::rabitq::code_pair_l2_sq(
                bytes_a,
                bytes_b,
                self.dimension,
                bits,
            ));
        }
        let ca = &bytes_a[..bytes_a.len().saturating_sub(4)];
        let cb = &bytes_b[..bytes_b.len().saturating_sub(4)];
        match M::name() {
            "l2" | "cosine" => {
                let thresh_f32 = threshold.map(|t| t as f32);
                f64::from(hyperspace_core::simd::l2_sq_f32_bytes_pair_adsampling(
                    ca,
                    cb,
                    thresh_f32,
                    hyperspace_core::simd::EXACT_EARLY_ABANDON,
                ))
            }
            "poincare" => {
                let alpha_a = if bytes_a.len() >= 4 {
                    f64::from(f32::from_le_bytes(
                        bytes_a[bytes_a.len() - 4..].try_into().unwrap_or([0; 4]),
                    ))
                } else {
                    1.0
                };
                let alpha_b = if bytes_b.len() >= 4 {
                    f64::from(f32::from_le_bytes(
                        bytes_b[bytes_b.len() - 4..].try_into().unwrap_or([0; 4]),
                    ))
                } else {
                    1.0
                };
                let alpha_prod = (alpha_a * alpha_b).max(1e-9);
                let diff_thresh =
                    threshold.map(|t| ((t - 1.0) / (2.0 * alpha_prod)).max(0.0) as f32);
                let diff_sq = f64::from(hyperspace_core::simd::l2_sq_f32_bytes_pair_adsampling(
                    ca,
                    cb,
                    diff_thresh,
                    hyperspace_core::simd::EXACT_EARLY_ABANDON,
                ));
                1.0 + 2.0 * diff_sq * alpha_prod
            }
            "lorentz" => hyperspace_core::simd::lorentz_f32_bytes_pair_thresh(ca, cb, threshold),
            "hybrid" => hyperspace_core::simd::hybrid_sq_f32_bytes_pair(ca, cb, threshold),
            _ => f64::MAX,
        }
    }

    #[inline]
    fn dist_f32_slices(
        a: &[f32],
        alpha_a: f64,
        b: &[f32],
        alpha_b: f64,
        threshold: Option<f64>,
    ) -> f64 {
        let name = M::name();
        let n = a.len().min(b.len());
        let a = &a[..n];
        let b = &b[..n];
        match name {
            "l2" | "cosine" => {
                let thresh_f32 = threshold.map(|t| t as f32);
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                f64::from(hyperspace_core::simd::l2_sq_f32_bytes_adsampling(
                    bytes_a,
                    b,
                    thresh_f32,
                    hyperspace_core::simd::EXACT_EARLY_ABANDON,
                ))
            }
            "poincare" => {
                let alpha_prod = (alpha_a * alpha_b).max(1e-9);
                let diff_thresh =
                    threshold.map(|t| ((t - 1.0) / (2.0 * alpha_prod)).max(0.0) as f32);
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                let diff_sq = f64::from(hyperspace_core::simd::l2_sq_f32_bytes_adsampling(
                    bytes_a,
                    b,
                    diff_thresh,
                    hyperspace_core::simd::EXACT_EARLY_ABANDON,
                ));
                1.0 + 2.0 * diff_sq * alpha_prod
            }
            "lorentz" => {
                if n == 0 {
                    return 1.0;
                }
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                hyperspace_core::simd::lorentz_f32_bytes_thresh(bytes_a, b, threshold)
            }
            "hybrid" => {
                if n < 33 {
                    return 0.0;
                }
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                hyperspace_core::simd::hybrid_sq_f32_bytes(bytes_a, b, threshold)
            }
            "dot" => {
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                1.0 - f64::from(hyperspace_core::simd::dot_f32_bytes(bytes_a, b))
            }
            "l1" => {
                let thresh_f32 = threshold.map(|t| t as f32);
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                f64::from(hyperspace_core::simd::l1_f32_bytes(bytes_a, b, thresh_f32))
            }
            "linf" => {
                let thresh_f32 = threshold.map(|t| t as f32);
                let bytes_a = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
                f64::from(hyperspace_core::simd::linf_f32_bytes(
                    bytes_a, b, thresh_f32,
                ))
            }
            _ => {
                let mut sum = 0.0f64;
                for (&x, &y) in a.iter().zip(b.iter()) {
                    let d = f64::from(x) - f64::from(y);
                    sum += d * d;
                }
                sum
            }
        }
    }

    /// HNSW Heuristic for neighbor selection
    fn select_neighbors(
        &self,
        _query_vec: &HyperVector,
        candidates: BinaryHeap<Candidate>,
        m: usize,
        alpha: f64,
    ) -> Vec<NodeId> {
        let mut sorted_candidates = candidates.into_sorted_vec();

        // Task 6.4: Density-based HNSW graph pruning
        let mut actual_m = m;
        if self.density_pruning && !sorted_candidates.is_empty() {
            let n_samples = sorted_candidates.len().min(m);
            let mut sum_dist = 0.0;
            // sorted_candidates has smallest distance at the end (due to Reverse Ord logic)
            for i in 0..n_samples {
                sum_dist +=
                    sorted_candidates[sorted_candidates.len() - 1 - i].unbiased(Self::dist_bias());
            }
            let avg_dist = sum_dist / (n_samples as f64);

            if avg_dist > 1.95 {
                actual_m = (m / 2).max(1);
            }
        }

        let mut result = Vec::with_capacity(actual_m);
        let storage_view = self.storage.view();
        let storage_count = storage_view.count();

        if self.can_dist_storage_pair() {
            let mut pruned = SmallVec::<[NodeId; 32]>::new();
            while let Some(cand) = sorted_candidates.pop() {
                if result.len() >= actual_m {
                    break;
                }
                let cand_id = cand.id();
                if (cand_id as usize) >= storage_count {
                    continue;
                }
                let cand_dist = cand.unbiased(Self::dist_bias());
                let cand_bytes = storage_view.get(cand_id);
                let mut is_good = true;
                for &neighbor_id in &result {
                    let neighbor_bytes = storage_view.get(neighbor_id);
                    let dist_to_neighbor =
                        self.dist_storage_bytes_pair(cand_bytes, neighbor_bytes, Some(cand_dist));
                    if Self::rng_dominated(dist_to_neighbor, cand_dist, alpha) {
                        is_good = false;
                        break;
                    }
                }
                if is_good {
                    result.push(cand_id);
                } else if pruned.len() < actual_m {
                    pruned.push(cand_id);
                }
            }
            let min_keep = if M::name() == "poincare" || M::name() == "lorentz" {
                (actual_m / 4).max(2).min(actual_m)
            } else {
                2.min(actual_m)
            };
            if !self.density_pruning && result.len() < min_keep {
                for &id in &pruned {
                    if result.len() >= min_keep {
                        break;
                    }
                    result.push(id);
                }
            }
            return result;
        }

        SELECT_SCRATCH.with(|sel_cell| {
            let mut sel = sel_cell.borrow_mut();
            let SelectScratch {
                result_coords,
                result_alphas,
                pruned,
            } = &mut *sel;
            result_coords.clear();
            result_alphas.clear();
            pruned.clear();

            let dim = self.dimension;
            result_coords.reserve(actual_m * dim);
            result_alphas.reserve(actual_m);

            QUERY_F32_SCRATCH.with(|cand_cell| {
                let mut cand_f32 = cand_cell.borrow_mut();
                while let Some(cand) = sorted_candidates.pop() {
                    // Gets closest (sorted vec is ascending, pop from end)
                    if result.len() >= actual_m {
                        break;
                    }

                    let cand_id = cand.id();
                    let cand_dist = cand.unbiased(Self::dist_bias());
                    let mut cand_alpha = 1.0f64;
                    self.load_node_coords_f32(
                        &storage_view,
                        cand_id,
                        &mut cand_f32,
                        &mut cand_alpha,
                    );

                    let mut is_good = true;

                    for (idx, _neighbor_id) in result.iter().enumerate() {
                        let neighbor_slice = &result_coords[idx * dim..(idx + 1) * dim];
                        let neighbor_alpha = result_alphas[idx];

                        let dist_to_neighbor = Self::dist_f32_slices(
                            &cand_f32,
                            cand_alpha,
                            neighbor_slice,
                            neighbor_alpha,
                            Some(cand_dist),
                        );

                        if Self::rng_dominated(dist_to_neighbor, cand_dist, alpha) {
                            is_good = false;
                            break;
                        }
                    }

                    if is_good {
                        result.push(cand_id);
                        result_coords.extend_from_slice(&cand_f32);
                        result_alphas.push(cand_alpha);
                    } else if pruned.len() < actual_m {
                        pruned.push(cand_id);
                    }
                }
            });

            // HNSW keepPrunedConnections: in curved spaces (Poincaré / Lorentz) or when a node
            // has low connectivity, diversity pruning can starve degree in deep hyperbolic regions;
            // backfill up to a safe minimum while preserving RNG long-range diversity.
            let min_keep = if M::name() == "poincare" || M::name() == "lorentz" {
                (actual_m / 4).max(2).min(actual_m)
            } else {
                2.min(actual_m)
            };
            if !self.density_pruning && result.len() < min_keep {
                for &id in &*pruned {
                    if result.len() >= min_keep {
                        break;
                    }
                    result.push(id);
                }
            }
        });
        result
    }

    /// Extracts node vector coordinates without heap allocation into caller-provided scratch buffer.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    pub fn with_vector_coords<R>(
        &self,
        id: NodeId,
        scratch: &mut Vec<f64>,
        f: impl FnOnce(&[f64]) -> R,
    ) -> R {
        if self.zonal {
            if let Some(zv) = self.zonal_storage.get(&id) {
                match zv.value() {
                    hyperspace_core::vector::ZonalVector::Core(ref i8_coords) => {
                        scratch.resize(self.dimension, 0.0);
                        for (i, &c) in i8_coords.iter().enumerate() {
                            scratch[i] = f64::from(c) / 127.0;
                        }
                        return f(scratch);
                    }
                    hyperspace_core::vector::ZonalVector::Boundary(ref f64_coords) => {
                        return f(f64_coords);
                    }
                }
            }
        }

        if id as usize >= self.storage.count() {
            scratch.resize(self.dimension, 0.0);
            return f(scratch);
        }

        let bytes = self.storage.get(id);
        match self.mode {
            QuantizationMode::ScalarI8 => {
                scratch.resize(self.dimension, 0.0);
                self.decode_sq8(bytes, scratch);
                f(scratch)
            }
            QuantizationMode::None => {
                if self.storage_f32 {
                    let n = (bytes.len().saturating_sub(4) / 4).min(self.dimension);
                    scratch.resize(n, 0.0);
                    for (i, c) in bytes[..n * 4].chunks_exact(4).enumerate() {
                        scratch[i] = f64::from(f32::from_le_bytes([c[0], c[1], c[2], c[3]]));
                    }
                    f(scratch)
                } else {
                    let n = (bytes.len().saturating_sub(8) / 8).min(self.dimension);
                    scratch.resize(n, 0.0);
                    for (i, c) in bytes[..n * 8].chunks_exact(8).enumerate() {
                        scratch[i] =
                            f64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
                    }
                    f(scratch)
                }
            }
            _ => {
                let vec = self.get_vector(id);
                f(&vec.coords)
            }
        }
    }

    // Helper to get HyperVector from id
    pub fn get_vector(&self, id: NodeId) -> HyperVector {
        if self.zonal {
            if let Some(zv) = self.zonal_storage.get(&id) {
                let vector = match zv.value() {
                    hyperspace_core::vector::ZonalVector::Core(ref i8_coords) => {
                        let mut coords = vec![0.0; self.dimension];
                        let mut sq_norm = 0.0;
                        for (i, &c) in i8_coords.iter().enumerate() {
                            let val = f64::from(c) / 127.0;
                            coords[i] = val;
                            sq_norm += val * val;
                        }
                        HyperVector {
                            coords,
                            alpha: if M::name() == "poincare" {
                                1.0 / (1.0 - sq_norm)
                            } else {
                                0.0
                            },
                        }
                    }
                    hyperspace_core::vector::ZonalVector::Boundary(ref f64_coords) => {
                        let mut coords = vec![0.0; self.dimension];
                        coords.copy_from_slice(f64_coords);
                        HyperVector::new_unchecked(coords)
                    }
                };
                return vector;
            }
        }

        // Defensive: Check bounds against the storage backend.
        // During MemTable swaps, a stale internal ID might be used against a fresh storage.
        if id as usize >= self.storage.count() {
            return HyperVector::new_unchecked(vec![0.0; self.dimension]);
        }

        let bytes = self.storage.get(id);
        match self.mode {
            QuantizationMode::ScalarI8 => {
                let mut coords = vec![0.0; self.dimension];
                let alpha = self.decode_sq8(bytes, &mut coords);
                HyperVector { coords, alpha }
            }
            QuantizationMode::None => {
                if self.storage_f32 {
                    let v = HyperVectorF32::from_bytes(bytes);
                    v.to_float64()
                } else {
                    let v = HyperVector::from_bytes(bytes);
                    v.clone()
                }
            }
            QuantizationMode::AsymmetricHybrid801 => {
                assert!(
                    self.dimension == 801,
                    "AsymmetricHybrid801 requires self.dimension=801"
                );
                let q = HybridQuantizedVector::from_bytes(bytes);
                let mut coords = vec![0.0; self.dimension];
                // Reconstruction: Lorentz part (f32) then Euclidean part (i8 SQ8)
                for (coord, &val) in coords[..33].iter_mut().zip(q.lorentz.iter()) {
                    *coord = f64::from(val);
                }
                for (coord, &val) in coords[33..].iter_mut().zip(q.euclidean.iter()) {
                    *coord = f64::from(val) / 127.0;
                }
                HyperVector {
                    coords,
                    alpha: f64::from(q.alpha),
                }
            }
            QuantizationMode::AsymmetricHybridLowBit => {
                assert!(
                    self.dimension == 801,
                    "AsymmetricHybridLowBit requires self.dimension=801"
                );
                let q = HybridLowBitQuantizedVector::from_bytes(bytes);
                let mut coords = vec![0.0; self.dimension];
                for (coord, &val) in coords[..33].iter_mut().zip(q.lorentz.iter()) {
                    *coord = f64::from(val);
                }
                let block_size = 16;
                let euc_dim = self.dimension - 33;
                let num_blocks = euc_dim.div_ceil(16);
                for b in 0..num_blocks {
                    let start = b * block_size;
                    let end = (start + block_size).min(euc_dim);
                    let scale = f64::from(q.scales[b]);
                    let packed_block_offset = b * 8;
                    for i in start..end {
                        let relative_idx = i - start;
                        let pair_idx = relative_idx / 2;
                        let is_second = relative_idx % 2 == 1;

                        let packed_byte = q.euclidean_packed[packed_block_offset + pair_idx];
                        let u_val = if is_second {
                            packed_byte & 0x0F
                        } else {
                            packed_byte >> 4
                        };
                        #[allow(clippy::cast_possible_wrap)]
                        let val = (u_val as i8) - 8;
                        coords[33 + i] = f64::from(val) * scale;
                    }
                }
                HyperVector::new_unchecked(coords)
            }
            QuantizationMode::AsymmetricHybridExtreme => {
                assert!(
                    self.dimension == 801,
                    "AsymmetricHybridExtreme requires self.dimension=801"
                );
                let q = HybridExtremeQuantizedVector::from_bytes(bytes);
                let mut coords = vec![0.0; self.dimension];
                for (coord, &val) in coords[..33].iter_mut().zip(q.lorentz.iter()) {
                    *coord = f64::from(val);
                }
                let scale = f64::from(q.alpha);
                let euc_dim = self.dimension - 33;
                for i in 0..euc_dim {
                    let byte = q.euclidean_bits[i / 8];
                    let bit = (byte >> (i % 8)) & 1;
                    coords[33 + i] = if bit == 1 { scale } else { -scale };
                }
                HyperVector::new_unchecked(coords)
            }
            QuantizationMode::ScalarI4 => {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let q = ScalarI4Vector::from_bytes(bytes, self.dimension, head_dim);
                let coords = q.reconstruct(self.dimension);
                HyperVector::new_unchecked(coords)
            }
            QuantizationMode::Turbo => {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let tail_dim = self.dimension - head_dim;
                let q = hyperspace_core::hybrid::TurboQuantVector::from_bytes(bytes, tail_dim);
                if let Some(tq) = q {
                    HyperVector::new_unchecked(tq.reconstruct(tail_dim))
                } else {
                    HyperVector::new_unchecked(vec![0.0; self.dimension])
                }
            }
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::from_bytes(bytes);
                let mut coords = vec![0.0; self.dimension];
                let val = 1.0 / (self.dimension as f64).sqrt() * 0.99;
                for (i, coord) in coords.iter_mut().enumerate() {
                    let byte_idx = i / 8;
                    let bit_idx = i % 8;
                    if (b.bits[byte_idx] >> bit_idx) & 1 == 1 {
                        *coord = val;
                    } else {
                        *coord = -val;
                    }
                }
                HyperVector {
                    coords,
                    alpha: f64::from(b.alpha),
                }
            }
            QuantizationMode::Ternary => {
                let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                    33
                } else {
                    0
                };
                let t = hyperspace_core::vector::TernaryHyperVector::from_bytes(
                    bytes,
                    self.dimension,
                    head_dim,
                );
                let coords = t.reconstruct(self.dimension);
                HyperVector {
                    coords,
                    alpha: f64::from(t.alpha),
                }
            }
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ => {
                if let Some(ref pq) = *self.pq.load() {
                    let enc = hyperspace_core::pq::PQVector::from_bytes(bytes);
                    let dec = pq.decode(&enc);
                    let coords: Vec<f64> = dec.into_iter().map(f64::from).collect();
                    HyperVector::new_unchecked(coords)
                } else {
                    HyperVector::new_unchecked(vec![0.0; self.dimension])
                }
            }
            QuantizationMode::RaBitQ
            | QuantizationMode::RaBitQ4
            | QuantizationMode::RaBitQ2
            | QuantizationMode::PQ4 => {
                let mut rec = Vec::with_capacity(self.dimension);
                if let Some(ref rq) = *self.rabitq.load() {
                    rq.reconstruct_into(bytes, &mut rec);
                } else if let Some(ref pq4) = *self.pq4.load() {
                    pq4.decode_into(bytes, &mut rec);
                }
                rec.resize(self.dimension, 0.0);
                HyperVector::new_unchecked(rec.into_iter().map(f64::from).collect())
            }
        }
    }

    pub fn get_vector_original(&self, id: NodeId) -> HyperVector {
        if self.mode == QuantizationMode::Turbo {
            let bytes = self.storage.get(id);
            let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                33
            } else {
                0
            };
            let tail_dim = self.dimension - head_dim;
            let q = hyperspace_core::hybrid::TurboQuantVector::from_bytes(bytes, tail_dim);
            if let Some(tq) = q {
                HyperVector::new_unchecked(tq.reconstruct_unrotated(tail_dim))
            } else {
                HyperVector::new_unchecked(vec![0.0; self.dimension])
            }
        } else if self.mode == QuantizationMode::Binary {
            let v = self.get_vector(id);
            let mut c_f32: Vec<f32> = v.coords.iter().map(|&x| x as f32).collect();
            hyperspace_core::turbo_rot::unrotate_vector_in_place(&mut c_f32);
            let unrotated: Vec<f64> = c_f32.into_iter().map(|x| x as f64).collect();
            HyperVector::new_unchecked(unrotated)
        } else if self.mode == QuantizationMode::Ternary {
            let bytes = self.storage.get(id);
            let head_dim = if self.dimension == 801 && M::name() == "hybrid" {
                33
            } else {
                0
            };
            let t = hyperspace_core::vector::TernaryHyperVector::from_bytes(
                bytes,
                self.dimension,
                head_dim,
            );
            let coords = t.reconstruct_unrotated(self.dimension);
            HyperVector {
                coords,
                alpha: f64::from(t.alpha),
            }
        } else {
            self.get_vector(id)
        }
    }

    // Insert with Metadata
    pub fn insert_to_storage(&self, vector: &[f64]) -> Result<u32, String> {
        // Validate against Metric logic (Poincare checks bounds, Euclidean doesn't)
        M::validate(vector)?;

        if self.zonal {
            let id = self.node_counter.fetch_add(1, Ordering::SeqCst);
            let zv = hyperspace_core::vector::ZonalVector::new_zonal(vector);
            self.zonal_storage.insert(id, zv);
            return Ok(id);
        }

        if self.mode == QuantizationMode::None && self.storage_f32 {
            let alpha_f32 = if M::name() == "poincare" {
                let sq_norm: f64 = vector.iter().map(|&x| x * x).sum();
                (1.0 / (1.0 - sq_norm)) as f32
            } else {
                1.0f32
            };
            let new_id = BYTES_SCRATCH.with(|cell| -> Result<u32, String> {
                let mut buf = cell.borrow_mut();
                buf.clear();
                buf.reserve(vector.len() * 4 + 4);
                for &c in vector {
                    buf.extend_from_slice(&(c as f32).to_le_bytes());
                }
                buf.extend_from_slice(&alpha_f32.to_le_bytes());
                let _g = self.append_lock.lock();
                let id = self.storage.append(&buf)?;
                let new_level = self.random_level();
                self.layer0.ensure(id);
                let pushed_id = self.nodes.push(Node::new(id, new_level));
                debug_assert_eq!(id as usize, pushed_id);
                Ok(id)
            })?;
            if self.spatial_index.indexed_count() > 0 {
                self.spatial_index.insert_vector(new_id, vector);
            }
            return Ok(new_id);
        }

        // Create vector (we already validated)
        let q_vec_full = Self::build_hyper_vector(vector.to_vec());
        // Trained modes: encode + append under the shared calibration gate so an online
        // quantizer swap can never miss a code written with the previous codebook.
        let _calibration_guard = self
            .mode
            .needs_training()
            .then(|| self.calibration_gate.read());

        let q_bytes = self.encode_for_storage(&q_vec_full)?;

        // Guarantee that storage sequence and boxcar sequence strictly match
        let new_id = {
            let _g = self.append_lock.lock();
            let id = self.storage.append(&q_bytes)?;

            let new_level = self.random_level();
            self.layer0.ensure(id);
            let pushed_id = self.nodes.push(Node::new(id, new_level));
            debug_assert_eq!(id as usize, pushed_id);
            id
        };

        if self.spatial_index.indexed_count() > 0 {
            self.spatial_index.insert_vector(new_id, vector);
        }
        Ok(new_id)
    }

    /// Storage encoding of a validated vector for this index's quantization mode (shared
    /// by inserts and updates). Trained modes must hold the calibration gate.
    fn encode_for_storage(&self, v: &HyperVector) -> Result<Vec<u8>, String> {
        let hybrid_head = if self.dimension == 801 && M::name() == "hybrid" {
            33
        } else {
            0
        };
        let require_801 = |name: &str| {
            if self.dimension == 801 {
                Ok(())
            } else {
                Err(format!("{name} requires dimension 801"))
            }
        };
        Ok(match self.mode {
            QuantizationMode::ScalarI8 => if M::name() == "lorentz" {
                QuantizedHyperVector::from_float_lorentz(v)
            } else if self.sq8_scaled {
                QuantizedHyperVector::from_float_scaled(v)
            } else {
                QuantizedHyperVector::from_float(v, self.config.is_anisotropic_enabled())
            }
            .as_bytes(),
            QuantizationMode::AsymmetricHybrid801 => {
                require_801("AsymmetricHybrid801")?;
                HybridQuantizedVector::from_float(v, 33, v.coords.len() - 33).as_bytes()
            }
            QuantizationMode::AsymmetricHybridLowBit => {
                require_801("AsymmetricHybridLowBit")?;
                HybridLowBitQuantizedVector::from_float(v, 33, v.coords.len() - 33).as_bytes()
            }
            QuantizationMode::AsymmetricHybridExtreme => {
                require_801("AsymmetricHybridExtreme")?;
                HybridExtremeQuantizedVector::from_float(v, 33, v.coords.len() - 33).as_bytes()
            }
            QuantizationMode::ScalarI4 => ScalarI4Vector::from_float(v, hybrid_head).as_bytes(),
            QuantizationMode::Turbo => {
                hyperspace_core::hybrid::TurboQuantVector::from_float(v, hybrid_head).as_bytes()
            }
            QuantizationMode::None if self.storage_f32 => {
                HyperVectorF32::from_float64(v).as_bytes()
            }
            QuantizationMode::None => v.as_bytes(),
            QuantizationMode::Binary => BinaryHyperVector::from_float(v).as_bytes(),
            QuantizationMode::Ternary => {
                hyperspace_core::vector::TernaryHyperVector::from_float(v, hybrid_head).as_bytes()
            }
            QuantizationMode::ProductQuantization | QuantizationMode::OPQ => {
                let q_f32: Vec<f32> = v.coords.iter().map(|&x| x as f32).collect();
                let pq_guard = self.pq.load();
                let pq = pq_guard
                    .as_ref()
                    .ok_or_else(|| "PQ quantizer must be initialized".to_string())?;
                pq.encode(&q_f32).to_bytes()
            }
            QuantizationMode::RaBitQ
            | QuantizationMode::RaBitQ4
            | QuantizationMode::RaBitQ2
            | QuantizationMode::PQ4 => self.encode_code(&v.coords),
        })
    }

    /// Update existing vector in storage (for upsert)
    pub fn update_storage(&self, id: u32, vector: &[f64]) -> Result<u32, String> {
        let arr = vector.to_vec();

        // Validate against Metric logic
        M::validate(&arr)?;

        if self.zonal {
            let zv = hyperspace_core::vector::ZonalVector::new_zonal(&arr);
            self.zonal_storage.insert(id, zv);
            return Ok(id);
        }

        // Same constructor as `insert_to_storage` (alpha = 1 outside Poincaré): updated and
        // inserted vectors must encode identically.
        let q_vec_full = Self::build_hyper_vector(arr);
        let _calibration_guard = self
            .mode
            .needs_training()
            .then(|| self.calibration_gate.read());
        self.storage
            .update(id, &self.encode_for_storage(&q_vec_full)?)?;
        if self.spatial_index.indexed_count() > 0 {
            self.spatial_index.insert_vector(id, vector);
        }
        Ok(id)
    }

    pub fn index_node(
        &self,
        id: NodeId,
        meta: std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        if !meta.is_empty() {
            self.has_nonempty_metadata.store(true, Ordering::Relaxed);
        }

        // 1. Index Metadata
        for (key, val) in &meta {
            // A. Inverted Index (Text)
            let tag = format!("{key}:{val}");
            self.metadata.inverted.entry(tag).or_default().insert(id);

            self.metadata
                .prefix_index
                .entry(key.clone())
                .or_default()
                .insert(val.clone());

            // B. Numeric range index (order-preserving f64 keys)
            if let Some((field, num)) = Self::numeric_entry(key, val) {
                self.numeric_insert(field, num, id);
            }
        }

        // Store full metadata for lookup (Data Explorer)
        if self.metadata.has_lexical_index.load(Ordering::Relaxed) {
            self.upsert_doc_lexical_stats(id, &meta);
        }
        self.metadata.forward.insert(id, meta);

        let q_vec = if self.mode == QuantizationMode::None && self.storage_f32 && !self.zonal {
            let mut coords = QUERY_F64_SCRATCH.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
            coords.clear();
            let bytes = self.storage.get(id);
            let n = (bytes.len().saturating_sub(4) / 4).min(self.dimension);
            coords.reserve(n);
            for c in bytes[..n * 4].chunks_exact(4) {
                coords.push(f64::from(f32::from_le_bytes([c[0], c[1], c[2], c[3]])));
            }
            let alpha = if bytes.len() >= 4 {
                f64::from(f32::from_le_bytes(
                    bytes[bytes.len() - 4..].try_into().unwrap_or([0; 4]),
                ))
            } else {
                1.0
            };
            HyperVector { coords, alpha }
        } else {
            self.get_vector(id)
        };

        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        let new_level = self.nodes.get(id as usize).map_or(0, Node::level);

        // Determine safe start layer for search
        let start_layer = {
            let nodes_count = self.nodes.count();
            if nodes_count == 0 || (entry_point as usize) >= nodes_count {
                0usize
            } else {
                self.nodes.get(entry_point as usize).map_or(0, Node::level)
            }
        };

        let mut curr_obj = entry_point;
        // q_vec already loaded at start of function

        // Need to check if entry_point is valid before dist calc
        let query_klein = if start_layer > 0 {
            self.compute_query_klein(&q_vec)
        } else {
            None
        };
        let query_klein_f32: Option<SmallVec<[f32; 256]>> = if start_layer > 0 {
            self.compute_query_klein_f32(&q_vec)
        } else {
            None
        };
        let query_klein_f32_ref = query_klein_f32.as_deref();

        let needs_f32 = self.needs_query_f32();
        let mut query_f32_scratch = if needs_f32 {
            let mut v =
                VISITED_SCRATCH.with(|cell| std::mem::take(&mut cell.borrow_mut().query_f32));
            v.clear();
            v.extend(q_vec.coords.iter().map(|&x| x as f32));
            Some(v)
        } else {
            None
        };
        let q_f32_ref = query_f32_scratch.as_deref();

        let pq_lut_buf = self.prepare_query(&q_vec, q_f32_ref);
        let pq_lut_ref = pq_lut_buf.as_ref();

        let storage_view = self.storage.view();
        let nodes_limit = self.nodes.count().min(storage_view.count());
        let mut curr_dist = if (entry_point as usize) < nodes_limit {
            self.dist_upper_bytes(
                storage_view.get(curr_obj),
                &q_vec,
                q_f32_ref,
                query_klein.as_ref(),
                query_klein_f32_ref,
                pq_lut_ref,
                None,
            )
        } else {
            f64::MAX
        };

        // 2. Phase 1: Zoom in (Greedy Search) from top to new_level
        // Ensure we don't start higher than what entry point supports
        let search_limit = std::cmp::min(max_layer as usize, start_layer);

        for level in (new_level + 1..=search_limit).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                let Some(node) = self.nodes.get(curr_obj as usize) else {
                    break;
                };
                if node.level() < level {
                    break;
                }
                let neighbor_ids: SmallVec<[NodeId; 16]> =
                    node.upper[level - 1].read().iter().copied().collect();
                let mut best = None;
                for n in neighbor_ids {
                    if (n as usize) >= nodes_limit {
                        continue;
                    }
                    let d = self.dist_upper_bytes(
                        storage_view.get(n),
                        &q_vec,
                        q_f32_ref,
                        query_klein.as_ref(),
                        query_klein_f32_ref,
                        pq_lut_ref,
                        None,
                    );
                    if d < curr_dist {
                        curr_dist = d;
                        best = Some(n);
                        changed = true;
                    }
                }
                if let Some(n) = best {
                    curr_obj = n;
                }
            }
        }

        // 3. Phase 2: Insert links from new_level down to 0
        {
            let m_base = self.config.get_m();
            let ef_construction = self.config.get_ef_construction();

            for level in (0..=std::cmp::min(new_level, max_layer as usize)).rev() {
                // HNSW: Layer 0 should be 2x denser for better recall
                let m_max = if level == 0 { m_base * 2 } else { m_base };

                // a) Search candidates
                let (q_klein_opt, q_klein_f32_opt) = if level > 0 {
                    (query_klein.as_ref(), query_klein_f32_ref)
                } else {
                    (None, None)
                };
                let candidates_heap = self.search_layer_candidates(
                    curr_obj,
                    &q_vec,
                    q_f32_ref,
                    q_klein_opt,
                    q_klein_f32_opt,
                    pq_lut_ref,
                    level,
                    ef_construction,
                );

                // b) Select neighbors with heuristic (using layer-specific M)
                let selected_neighbors =
                    self.select_neighbors(&q_vec, candidates_heap, m_max, Self::rng_alpha());

                // c) Bidirectional connect
                self.links_set(id, level, &selected_neighbors);
                // Prune policy: with a cheap SIMD pair kernel, prune as soon as a list exceeds
                // its budget (standard HNSW; search never scans more than `m_max` links). Modes
                // that must decode / reconstruct vectors for the RNG heuristic (quantized codes)
                // keep a 25% slack: an O(m²) re-selection on every overflow would dominate build
                // time at large M (e.g. ternary with M = 64).
                let prune_threshold = if self.can_dist_storage_pair() {
                    m_max
                } else {
                    (m_max * 5 / 4).max(m_max + 2)
                };
                for &neighbor_id in &selected_neighbors {
                    let neighbor_layer_len = self.add_link(neighbor_id, id, level);

                    // d) Pruning
                    if neighbor_layer_len > prune_threshold {
                        self.prune_connections(neighbor_id, level, m_max);
                    }
                }

                if !selected_neighbors.is_empty() {
                    curr_obj = selected_neighbors[0];
                }
            }
        }

        if let Some(v) = query_f32_scratch.take() {
            VISITED_SCRATCH.with(|cell| cell.borrow_mut().query_f32 = v);
        }
        QUERY_F64_SCRATCH.with(|cell| *cell.borrow_mut() = q_vec.coords);

        // Update global entry point if needed
        if (new_level as u32) > max_layer {
            self.max_layer.store(new_level as u32, Ordering::SeqCst);
            self.entry_point.store(id, Ordering::SeqCst);
        }

        Ok(())
    }

    // Wrapped insert for backward compatibility
    pub fn insert(
        &self,
        vector: &[f64],
        meta: std::collections::HashMap<String, String>,
    ) -> Result<u32, String> {
        let new_id = self.insert_to_storage(vector)?;
        self.index_node(new_id, meta)?;
        Ok(new_id)
    }

    fn add_link(&self, src: NodeId, dst: NodeId, level: usize) -> usize {
        let mut changed = false;
        let mut snapshot = layer0::Links::new();
        let len = self.links_update(src, level, |links| {
            if !links.contains(&dst) {
                links.push(dst);
                changed = true;
                if level == 0 {
                    snapshot.clone_from(links);
                }
            }
        });
        if changed && level == 0 {
            self.sync_flat_neighbors(src, &snapshot);
        }
        len
    }

    fn prune_connections(&self, node_id: NodeId, level: usize, max_links: usize) {
        let _ = self.relink(node_id, level, max_links, &[], None, Self::rng_alpha());
    }

    /// Re-selects `node_id`'s neighbors on `level` with the RNG heuristic from its current
    /// links ∪ `extra`, skipping ids in `exclude` (e.g. deleted nodes). Links added
    /// concurrently while distances were computed are preserved.
    fn relink(
        &self,
        node_id: NodeId,
        level: usize,
        max_links: usize,
        extra: &[NodeId],
        exclude: Option<&RoaringBitmap>,
        alpha: f64,
    ) -> Vec<NodeId> {
        // 1. Snapshot current links into stack SmallVec (no heap allocation)
        let mut current = layer0::Links::new();
        if !self.links_into(node_id, level, &mut current) {
            return Vec::new();
        }
        let excluded = |id: NodeId| id == node_id || exclude.is_some_and(|bm| bm.contains(id));
        let mut initial_links = layer0::Links::new();
        for &id in current.iter().chain(extra) {
            if !excluded(id) && !initial_links.contains(&id) {
                initial_links.push(id);
            }
        }

        let storage_view = self.storage.view();
        let storage_count = storage_view.count();

        let keepers = if self.can_dist_storage_pair() && (node_id as usize) < storage_count {
            let node_bytes = storage_view.get(node_id);
            let mut candidates = Vec::with_capacity(initial_links.len());
            for &n in &initial_links {
                let d = if (n as usize) < storage_count {
                    self.dist_storage_bytes_pair(node_bytes, storage_view.get(n), None)
                } else {
                    f64::MAX
                };
                candidates.push(Candidate::biased(n, d, Self::dist_bias()));
            }
            let heap = BinaryHeap::from(candidates);
            let dummy = HyperVector {
                coords: Vec::new(),
                alpha: 1.0,
            };
            self.select_neighbors(&dummy, heap, max_links, alpha)
        } else {
            // 2. Heavy work: calculate distances (NO LOCKS HELD), with precomputed f32 query slice & PQ LUT
            let node_vec = self.get_vector(node_id);
            let needs_f32 = self.needs_query_f32();
            let q_f32_buf: Option<SmallVec<[f32; 128]>> = if needs_f32 {
                Some(node_vec.coords.iter().map(|&x| x as f32).collect())
            } else {
                None
            };
            let q_f32_ref = q_f32_buf.as_deref();

            let pq_lut_buf = self.prepare_query(&node_vec, q_f32_ref);
            let pq_lut_ref = pq_lut_buf.as_ref();

            let mut candidates = Vec::with_capacity(initial_links.len());
            for &n in &initial_links {
                let d = if (n as usize) < storage_count {
                    self.dist_bytes_opt(
                        storage_view.get(n),
                        &node_vec,
                        q_f32_ref,
                        pq_lut_ref,
                        None,
                        None,
                    )
                } else {
                    f64::MAX
                };
                candidates.push(Candidate::biased(n, d, Self::dist_bias()));
            }

            // Select best from snapshot
            let heap = BinaryHeap::from(candidates);
            self.select_neighbors(&node_vec, heap, max_links, alpha)
        };

        // 3. Update links in-place (preserving any concurrently added links)
        let mut snapshot = layer0::Links::new();
        self.links_update(node_id, level, |current_links| {
            let mut merged = layer0::Links::from_slice(&keepers);
            for &id in current_links.iter() {
                if !current.contains(&id) && !excluded(id) && !merged.contains(&id) {
                    merged.push(id);
                }
            }
            *current_links = merged;
            if level == 0 {
                snapshot.clone_from(current_links);
            }
        });
        if level == 0 {
            self.sync_flat_neighbors(node_id, &snapshot);
        }
        keepers
    }

    pub fn count_nodes(&self) -> usize {
        if self.zonal {
            self.node_counter.load(Ordering::Relaxed) as usize
        } else {
            self.storage.count()
        }
    }

    /// Task 7.2: DiskANN / Vamana Graph Pruning
    /// Converts the dense HNSW base layer into a Spatial Navigable Graph (DiskANN).
    /// Prevents excessive page faults on NVMe when searching evicted cold chunks.
    pub fn optimize_as_sng(&self, alpha: f64) {
        println!(
            "🔧 Optimizing HNSW Graph -> Spatial Navigable Graph (DiskANN) with alpha={alpha}"
        );
        let num_nodes = self.count_nodes() as u32;

        for i in 0..num_nodes {
            let node_vec = self.get_vector(i);

            let mut candidates = layer0::Links::new();
            self.layer0.read_into(i, &mut candidates);

            if candidates.is_empty() {
                continue;
            }

            // 2. Compute distances between node `i` and all candidates
            let mut c_with_dist = Vec::with_capacity(candidates.len());
            for &c in &candidates {
                if c != i {
                    let c_vec = self.get_vector(c);
                    let dist = M::distance(&node_vec.coords, &c_vec.coords);
                    c_with_dist.push((c, dist));
                }
            }

            c_with_dist.sort_by(|a, b| a.1.total_cmp(&b.1));

            // 3. Alpha pruning (RobustPrune from Vamana/DiskANN)
            let max_degree = self.config.m.load(std::sync::atomic::Ordering::Relaxed);
            let mut new_neighbors = SmallVec::<[NodeId; 32]>::with_capacity(max_degree);

            for (c, cand_dist) in c_with_dist {
                if new_neighbors.len() >= max_degree {
                    break;
                }
                let c_vec = self.get_vector(c);
                let mut prune = false;

                for &existing in &new_neighbors {
                    let existing_vec = self.get_vector(existing);
                    let dist_existing_c = M::distance(&existing_vec.coords, &c_vec.coords);

                    // If candidate can be reached easily from an already selected neighbor, prune it.
                    if alpha * dist_existing_c <= cand_dist {
                        prune = true;
                        break;
                    }
                }

                if !prune {
                    new_neighbors.push(c);
                }
            }

            self.layer0.set(i, &new_neighbors);
        }
        println!("✅ Graph optimization complete.");
    }

    /// `true` once enough deletes accumulated that tombstones start hurting recall/latency
    /// (`max(256, 5% of nodes)`, override with `HS_REPAIR_DELETE_THRESHOLD`).
    pub fn needs_repair(&self) -> bool {
        static T: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
        let fixed = *T.get_or_init(|| {
            std::env::var("HS_REPAIR_DELETE_THRESHOLD")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
        });
        let threshold = fixed.unwrap_or_else(|| (self.nodes.count() / 20).max(256));
        self.deletes_since_repair.load(Ordering::Relaxed) >= threshold
            && !self.repair_running.load(Ordering::Relaxed)
    }

    /// FreshDiskANN-style graph consolidation after deletes.
    ///
    /// Every live node that links to a deleted node gets its list re-selected (RNG heuristic)
    /// from its live neighbors ∪ the live neighbors of its deleted neighbors, so the graph
    /// stays navigable without traversing tombstones. Also moves a deleted entry point to a
    /// live node on the top layer. Runs in parallel; safe alongside searches and inserts.
    /// Returns the number of repaired adjacency lists.
    pub fn repair_deleted(&self) -> usize {
        if self
            .repair_running
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            return 0;
        }
        let pending = self.deletes_since_repair.load(Ordering::Relaxed);
        let deleted = self.metadata.deleted.read().clone();
        let mut repaired = 0usize;
        if !deleted.is_empty() {
            let m_base = self.config.get_m();
            let n = self.nodes.count() as u32;
            // FreshDiskANN consolidation keeps longer edges (α ≈ 1.2) to bridge the holes.
            let repair_alpha = Self::rng_alpha().max(1.2);
            repaired = (0..n)
                .into_par_iter()
                .filter(|id| !deleted.contains(*id))
                .map(|id| {
                    let Some(node) = self.nodes.get(id as usize) else {
                        return 0usize;
                    };
                    let mut links = layer0::Links::new();
                    let mut hop = layer0::Links::new();
                    let mut extra: SmallVec<[NodeId; 64]> = SmallVec::new();
                    let mut fixed = 0usize;
                    for level in 0..=node.level() {
                        if !self.links_into(id, level, &mut links)
                            || !links.iter().any(|x| deleted.contains(*x))
                        {
                            continue;
                        }
                        extra.clear();
                        for &dead in links.iter().filter(|x| deleted.contains(**x)) {
                            if self.links_into(dead, level, &mut hop) {
                                extra.extend(
                                    hop.iter()
                                        .copied()
                                        .filter(|h| *h != id && !deleted.contains(*h)),
                                );
                            }
                        }
                        let m_max = if level == 0 { m_base * 2 } else { m_base };
                        let kept =
                            self.relink(id, level, m_max, &extra, Some(&deleted), repair_alpha);
                        // Restore in-edges lost with the deleted bridges (as on insert).
                        for nb in kept {
                            if self.add_link(nb, id, level) > m_max {
                                self.relink(nb, level, m_max, &[], Some(&deleted), repair_alpha);
                            }
                        }
                        fixed += 1;
                    }
                    fixed
                })
                .sum();

            // Move a deleted entry point to a live node on the highest populated level.
            let ep = self.entry_point.load(Ordering::Acquire);
            if deleted.contains(ep) {
                let best = (0..n)
                    .filter(|id| !deleted.contains(*id))
                    .filter_map(|id| self.nodes.get(id as usize).map(|nd| (id, nd.level())))
                    .max_by_key(|&(_, lvl)| lvl);
                if let Some((id, lvl)) = best {
                    self.entry_point.store(id, Ordering::Release);
                    self.max_layer.store(lvl as u32, Ordering::Release);
                }
            }
        }
        self.deletes_since_repair
            .fetch_sub(pending, Ordering::Relaxed);
        self.repair_running.store(false, Ordering::Release);
        repaired
    }

    pub fn count_deleted(&self) -> usize {
        self.metadata.deleted.read().len() as usize
    }

    pub fn graph_neighbors(
        &self,
        node_id: NodeId,
        layer: usize,
        limit: usize,
    ) -> Result<Vec<NodeId>, String> {
        if self.nodes.get(node_id as usize).is_none() {
            return Err(format!("Node {node_id} not found"));
        }
        let mut links = layer0::Links::new();
        if !self.links_into(node_id, layer, &mut links) {
            return Err(format!("Layer {layer} is out of bounds for node {node_id}"));
        }
        let deleted = self.metadata.deleted.read();
        let out = links
            .iter()
            .copied()
            .filter(|id| !deleted.contains(*id))
            .take(limit)
            .collect();
        Ok(out)
    }

    pub fn graph_traverse(
        &self,
        start_id: NodeId,
        layer: usize,
        max_depth: usize,
        max_nodes: usize,
        breadth_limit: usize,
    ) -> Result<Vec<NodeId>, String> {
        if max_nodes == 0 {
            return Ok(Vec::new());
        }
        let Some(start) = self.nodes.get(start_id as usize) else {
            return Err(format!("Start node {start_id} not found"));
        };
        if start.level() < layer {
            return Err(format!(
                "Layer {layer} is out of bounds for node {start_id}"
            ));
        }
        let deleted = self.metadata.deleted.read();
        if deleted.contains(start_id) {
            return Ok(Vec::new());
        }

        let mut queue = std::collections::VecDeque::new();
        let mut visited = std::collections::HashSet::new();
        let mut out = Vec::new();
        queue.push_back((start_id, 0usize));
        visited.insert(start_id);

        while let Some((node_id, depth)) = queue.pop_front() {
            out.push(node_id);
            if out.len() >= max_nodes {
                break;
            }
            if depth >= max_depth {
                continue;
            }
            let mut neighbors = layer0::Links::new();
            if self.links_into(node_id, layer, &mut neighbors) {
                let limit = if breadth_limit == 0 {
                    usize::MAX
                } else {
                    breadth_limit
                };
                for &next in neighbors.iter().take(limit) {
                    if deleted.contains(next) {
                        continue;
                    }
                    if visited.insert(next) {
                        queue.push_back((next, depth + 1));
                    }
                }
            }
        }
        Ok(out)
    }

    pub fn graph_connected_components(
        &self,
        layer: usize,
        min_cluster_size: usize,
        max_clusters: usize,
        max_nodes: usize,
    ) -> Vec<Vec<NodeId>> {
        if max_nodes == 0 || max_clusters == 0 {
            return Vec::new();
        }
        let nodes_count = self.nodes.count();
        let deleted = self.metadata.deleted.read();
        let scan_limit = std::cmp::min(nodes_count, max_nodes);
        let mut visited = std::collections::HashSet::new();
        let mut clusters = Vec::new();

        for node_id in 0..scan_limit as u32 {
            if deleted.contains(node_id) || !visited.insert(node_id) {
                continue;
            }
            let Some(node) = self.nodes.get(node_id as usize) else {
                continue;
            };
            if node.level() < layer {
                continue;
            }

            let mut queue = std::collections::VecDeque::from([(node_id, 0usize)]);
            let mut next_links = layer0::Links::new();
            let mut component = Vec::new();

            while let Some((curr, _)) = queue.pop_front() {
                component.push(curr);
                if component.len() >= max_nodes {
                    break;
                }
                if !self.links_into(curr, layer, &mut next_links) {
                    continue;
                }
                for &next in &next_links {
                    if deleted.contains(next) {
                        continue;
                    }
                    if visited.insert(next) {
                        queue.push_back((next, 0));
                    }
                }
            }

            if component.len() >= min_cluster_size {
                clusters.push(component);
                if clusters.len() >= max_clusters {
                    break;
                }
            }
        }

        clusters
    }

    pub fn metadata_by_id(&self, id: NodeId) -> std::collections::HashMap<String, String> {
        self.metadata
            .forward
            .get(&id)
            .map_or_else(std::collections::HashMap::new, |m| m.clone())
    }

    /// Merge a metadata patch into an existing node without modifying the vector or graph.
    /// Returns `Err` if `id` does not exist.
    pub fn update_payload_metadata(
        &self,
        id: NodeId,
        patch: &std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        if !patch.is_empty() {
            self.has_nonempty_metadata.store(true, Ordering::Relaxed);
        }
        if let Some(mut entry) = self.metadata.forward.get_mut(&id) {
            for (key, new_val) in patch {
                // If key existed with a different value, clean up old inverted/numeric entries
                if let Some(old_val) = entry.get(key) {
                    if old_val != new_val {
                        let old_tag = format!("{key}:{old_val}");
                        if let Some(mut bm) = self.metadata.inverted.get_mut(&old_tag) {
                            bm.remove(id);
                        }
                        if let Some((field, old_num)) = Self::numeric_entry(key, old_val) {
                            self.numeric_remove(field, old_num, id);
                        }
                    }
                }

                // Index new value in inverted index
                let tag = format!("{key}:{new_val}");
                self.metadata.inverted.entry(tag).or_default().insert(id);

                self.metadata
                    .prefix_index
                    .entry(key.clone())
                    .or_default()
                    .insert(new_val.clone());

                // Index new value in numeric index if parseable
                if let Some((field, num)) = Self::numeric_entry(key, new_val) {
                    self.numeric_insert(field, num, id);
                }

                entry.insert(key.clone(), new_val.clone());
            }

            let updated_meta = entry.clone();
            drop(entry);

            if self.metadata.has_lexical_index.load(Ordering::Relaxed) {
                self.upsert_doc_lexical_stats(id, &updated_meta);
            }
            Ok(())
        } else {
            Err(format!("Node {id} not found"))
        }
    }

    pub fn storage_stats(&self) -> (usize, usize) {
        (
            self.storage.segment_count(),
            self.storage.total_size_bytes(),
        )
    }

    #[allow(clippy::unused_self)]
    /// Standard HNSW level assignment: promote with probability `1/M`
    /// (`mL = 1/ln M`). The former fixed `0.5` put ~8× more nodes on upper layers
    /// for `M = 16`, inflating memory and insertion work without improving recall.
    fn random_level(&self) -> usize {
        let m = self.config.get_m().max(2) as f64;
        let p = 1.0 / m;
        let mut rng = rand::thread_rng();
        let mut level = 0;
        while rng.gen::<f64>() < p && level < MAX_LAYERS - 1 {
            level += 1;
        }
        level
    }

    fn get_tokenizer(
        language: &str,
    ) -> dashmap::mapref::one::Ref<'static, String, crate::tokenizer::Tokenizer> {
        static TOKENIZERS: std::sync::OnceLock<
            dashmap::DashMap<String, crate::tokenizer::Tokenizer>,
        > = std::sync::OnceLock::new();

        let map = TOKENIZERS.get_or_init(dashmap::DashMap::new);
        if !map.contains_key(language) {
            let tok = crate::tokenizer::Tokenizer::builder()
                .language(language)
                .build()
                .unwrap_or_else(|_| crate::tokenizer::Tokenizer::default());
            map.insert(language.to_string(), tok);
        }
        map.get(language).unwrap()
    }

    fn tokenize(text: &str, config: &GlobalConfig) -> Vec<String> {
        let params = config.get_bm25_params();
        let mut tokens = Self::get_tokenizer(&params.language).tokenize(text);

        if params.ngrams > 1 {
            let limit = params.ngrams as usize;
            let unigrams = tokens.clone();
            for n in 2..=limit {
                for window in unigrams.windows(n) {
                    tokens.push(window.join("_"));
                }
            }
        }
        tokens
    }

    fn build_doc_term_stats(
        meta: &HashMap<String, String>,
        config: &GlobalConfig,
    ) -> (HashMap<String, u16>, u32) {
        let mut term_freq = HashMap::new();
        let mut doc_len: u32 = 0;

        for (key, value) in meta {
            if key.starts_with("__hs_") {
                continue;
            }
            for token in Self::tokenize(value, config) {
                doc_len = doc_len.saturating_add(1);
                let entry = term_freq.entry(token).or_insert(0_u16);
                *entry = (*entry).saturating_add(1_u16);
            }
        }
        (term_freq, doc_len)
    }

    #[inline]
    fn make_token_key(token: &str) -> String {
        let mut s = String::with_capacity(5 + token.len());
        s.push_str("_txt:");
        s.push_str(token);
        s
    }

    fn remove_doc_lexical_stats(&self, id: NodeId) {
        let old_len = if let Some(entry) = self.metadata.doc_token_len.get(id as usize) {
            entry.swap(0, Ordering::Relaxed)
        } else {
            0
        };
        if old_len > 0 {
            self.metadata
                .total_token_len
                .fetch_sub(u64::from(old_len), Ordering::Relaxed);
        }
        if old_len == 0 && !self.metadata.forward.contains_key(&id) {
            return;
        }
        if let Some(old_meta) = self.metadata.forward.get(&id) {
            let (term_freq, _) = Self::build_doc_term_stats(old_meta.value(), &self.config);
            for token in term_freq.keys() {
                let token_key = Self::make_token_key(token);
                if let Some(mut bitmap) = self.metadata.inverted.get_mut(&token_key) {
                    bitmap.remove(id);
                }
                if let Some(mut df_ref) = self.metadata.token_df.get_mut(token) {
                    if *df_ref > 1 {
                        *df_ref -= 1;
                    } else {
                        drop(df_ref);
                        self.metadata.token_df.remove(token);
                    }
                }
                if let Some(mut tdf_ref) = self.metadata.term_doc_freq.get_mut(token) {
                    if let Some(pos) = tdf_ref.iter().position(|(doc_id, _)| *doc_id == id) {
                        tdf_ref.swap_remove(pos);
                    }
                }
            }
        }
    }

    fn upsert_doc_lexical_stats(&self, id: NodeId, meta: &HashMap<String, String>) {
        self.remove_doc_lexical_stats(id);
        let (term_freq, doc_len) = Self::build_doc_term_stats(meta, &self.config);
        while id >= self.metadata.doc_token_len.count() as u32 {
            self.metadata.doc_token_len.push(AtomicU32::new(0));
        }
        if let Some(entry) = self.metadata.doc_token_len.get(id as usize) {
            entry.store(doc_len, Ordering::Relaxed);
        }
        self.metadata
            .total_token_len
            .fetch_add(u64::from(doc_len), Ordering::Relaxed);
        for (token, tf) in term_freq {
            let token_key = Self::make_token_key(&token);
            self.metadata
                .inverted
                .entry(token_key)
                .or_default()
                .insert(id);
            *self.metadata.token_df.entry(token.clone()).or_insert(0) += 1;
            self.metadata
                .term_doc_freq
                .entry(token)
                .or_default()
                .push((id, tf));
        }
    }

    fn rebuild_lexical_stats(&self) {
        self.metadata
            .has_lexical_index
            .store(true, Ordering::SeqCst);
        self.metadata.token_df.clear();
        for i in 0..self.metadata.doc_token_len.count() {
            if let Some(entry) = self.metadata.doc_token_len.get(i) {
                entry.store(0, Ordering::Relaxed);
            }
        }
        self.metadata.term_doc_freq.clear();
        self.metadata.total_token_len.store(0, Ordering::Relaxed);
        for item in &self.metadata.forward {
            self.upsert_doc_lexical_stats(*item.key(), item.value());
        }
    }

    // RRF Fusion Logic
    fn search_hybrid(
        &self,
        query: &[f64],
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        text: &str,
        params: &hyperspace_core::SearchParams,
    ) -> Vec<(NodeId, f64)> {
        #[derive(PartialEq)]
        struct MinHeapEntry {
            score: f64,
            id: u32,
        }
        impl Eq for MinHeapEntry {}
        impl PartialOrd for MinHeapEntry {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for MinHeapEntry {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other.score.total_cmp(&self.score)
            }
        }

        if !self.metadata.has_lexical_index.swap(true, Ordering::SeqCst) {
            self.rebuild_lexical_stats();
        }

        // Precompute allowed_bitmap ONCE for both semantic and lexical passes
        let allowed_bitmap = self.build_allowed_bitmap(filter, complex_filters);
        if allowed_bitmap
            .as_ref()
            .is_some_and(roaring::RoaringBitmap::is_empty)
        {
            return Vec::new();
        }

        // 1. Get Vector Search Results (Semantic) -> Top K*10 (min 100) for recall
        // We reuse search_with_allowed with the precomputed bitmap to avoid duplicate work and recursion
        let vec_k = (params.top_k * 10).max(100);
        let mut inner_params = params.clone();
        inner_params.hybrid_query = None;
        inner_params.top_k = vec_k;
        inner_params.ef_search = inner_params.ef_search.max(vec_k);

        let vector_results =
            self.search_with_allowed(query, allowed_bitmap.as_ref(), &inner_params);

        // 2. BM25 lexical ranking over the same filtered space.
        let tokens = Self::tokenize(text, &self.config);
        if tokens.is_empty() {
            return vector_results.into_iter().take(params.top_k).collect();
        }
        let mut uniq_tokens = std::collections::HashSet::new();
        for token in tokens {
            uniq_tokens.insert(token);
        }
        let deleted = self.metadata.deleted.read();
        let is_allowed = |id: u32| -> bool {
            if deleted.contains(id) {
                return false;
            }
            if let Some(bm) = &allowed_bitmap {
                bm.contains(id)
            } else {
                true
            }
        };

        let global_docs = self.nodes.count().saturating_sub(deleted.len() as usize);
        let total_docs = global_docs.max(1) as f64;
        let avgdl = if global_docs == 0 {
            1.0
        } else {
            self.metadata.total_token_len.load(Ordering::Relaxed) as f64 / total_docs
        }
        .max(1.0);
        let bm25_params = params
            .bm25_options
            .clone()
            .unwrap_or_else(|| self.config.get_bm25_params());

        let keyword_scores = if uniq_tokens.len() <= 8 {
            let mut acc = HashMap::with_capacity(1024);
            for token in uniq_tokens {
                if let Some(doc_freqs) = self.metadata.term_doc_freq.get(&token) {
                    let df = (doc_freqs.value().len() as u32).max(1);
                    let idf_val =
                        hyperspace_core::bm25::idf(bm25_params.method, global_docs as u32, df);

                    for &(id, tf) in doc_freqs.value() {
                        if !is_allowed(id) {
                            continue;
                        }
                        let tf_f32 = f32::from(tf);
                        let dl = self
                            .metadata
                            .doc_token_len
                            .get(id as usize)
                            .map_or(0.0, |v| v.load(Ordering::Relaxed) as f32)
                            .max(1.0);

                        let score = f64::from(hyperspace_core::bm25::score_with_idf(
                            idf_val,
                            bm25_params.method,
                            tf_f32,
                            dl,
                            avgdl as f32,
                            bm25_params.k1,
                            bm25_params.b,
                            bm25_params.delta,
                        ));

                        *acc.entry(id).or_insert(0.0) += score;
                    }
                }
            }
            acc
        } else {
            uniq_tokens
                .into_par_iter()
                .fold(HashMap::new, |mut acc, token| {
                    if let Some(doc_freqs) = self.metadata.term_doc_freq.get(&token) {
                        let df = (doc_freqs.value().len() as u32).max(1);
                        let idf_val =
                            hyperspace_core::bm25::idf(bm25_params.method, global_docs as u32, df);

                        for &(id, tf) in doc_freqs.value() {
                            if !is_allowed(id) {
                                continue;
                            }
                            let tf_f32 = f32::from(tf);
                            let dl = self
                                .metadata
                                .doc_token_len
                                .get(id as usize)
                                .map_or(0.0, |v| v.load(Ordering::Relaxed) as f32)
                                .max(1.0);

                            let score = f64::from(hyperspace_core::bm25::score_with_idf(
                                idf_val,
                                bm25_params.method,
                                tf_f32,
                                dl,
                                avgdl as f32,
                                bm25_params.k1,
                                bm25_params.b,
                                bm25_params.delta,
                            ));

                            *acc.entry(id).or_insert(0.0) += score;
                        }
                    }
                    acc
                })
                .reduce(HashMap::new, |mut a, b| {
                    for (k, v) in b {
                        *a.entry(k).or_insert(0.0) += v;
                    }
                    a
                })
        };

        let max_k = (params.top_k * 3).max(200);
        let mut heap = std::collections::BinaryHeap::with_capacity(max_k + 1);
        for (id, score) in keyword_scores {
            if heap.len() < max_k {
                heap.push(MinHeapEntry { score, id });
            } else if score > heap.peek().unwrap().score {
                heap.pop();
                heap.push(MinHeapEntry { score, id });
            }
        }
        let mut keyword_results: Vec<(u32, f64)> = heap
            .into_iter()
            .map(|entry| (entry.id, entry.score))
            .collect();
        keyword_results.sort_by(|a, b| b.1.total_cmp(&a.1));

        // 3. Fusion
        let mut final_scores: std::collections::HashMap<u32, f32> =
            std::collections::HashMap::new();

        let fusion_method = params
            .fusion_method
            .clone()
            .unwrap_or_else(|| self.config.get_fusion_method());
        let alpha = params.hybrid_alpha.unwrap_or(60.0);

        if fusion_method == "weighted" {
            let v_min = vector_results
                .iter()
                .map(|&(_, d)| d)
                .fold(f64::INFINITY, f64::min);
            let v_max = vector_results
                .iter()
                .map(|&(_, d)| d)
                .fold(f64::NEG_INFINITY, f64::max);
            let v_range = (v_max - v_min).max(1e-9);

            let k_min = keyword_results
                .iter()
                .map(|&(_, s)| s)
                .fold(f64::INFINITY, f64::min);
            let k_max = keyword_results
                .iter()
                .map(|&(_, s)| s)
                .fold(f64::NEG_INFINITY, f64::max);
            let k_range = (k_max - k_min).max(1e-9);

            let vec_alpha = alpha.clamp(0.0, 1.0);
            let key_alpha = 1.0 - vec_alpha;

            // Distance: smaller is better -> inverted normalized score [0, 1]
            for (id, dist) in &vector_results {
                let norm_score = 1.0 - ((dist - v_min) / v_range);
                *final_scores.entry(*id).or_default() += (norm_score as f32) * vec_alpha;
            }

            // BM25: larger is better -> normalized score [0, 1]
            for (id, score) in &keyword_results {
                let norm_score = (score - k_min) / k_range;
                *final_scores.entry(*id).or_default() += (norm_score as f32) * key_alpha;
            }
        } else {
            // RRF Fusion (Default)
            // Process Vector Ranks (1-based)
            for (rank, (id, _dist)) in vector_results.iter().enumerate() {
                let rrf = 1.0 / (alpha + (rank as f32 + 1.0));
                *final_scores.entry(*id).or_default() += rrf;
            }

            // Process Keyword Ranks
            for (rank, (id, _score)) in keyword_results.iter().enumerate() {
                let rrf = 1.0 / (alpha + (rank as f32 + 1.0));
                *final_scores.entry(*id).or_default() += rrf;
            }
        }

        // Sort Final
        let mut final_ranking: Vec<(NodeId, f32)> = final_scores.into_iter().collect();
        // Sort DESCENDING by score (Higher is better)
        final_ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Convert back to (id, distance) interface.
        // Note: Distance is weird here because RRF score is not distance.
        // We invert it back? Or just return 1.0 - score?
        // Let's return (1.0 / score) to mimic "smaller is better"?
        // Or just negative score? Proto expects 'double distance'.
        // For semantic search, smaller is better.
        // Let's make it 1.0 - normalized_score?
        // Limit to K
        final_ranking
            .into_iter()
            .take(params.top_k)
            .map(|(id, score)| (id, f64::from(10.0 - score)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visited_scratch_epoch_lifecycle_and_wraparound() {
        let mut scratch = VisitedScratch::default();
        let nodes = 1000;

        // Query 1
        scratch.prepare(nodes);
        assert_eq!(scratch.epoch, 1);
        assert!(scratch.mark_visited(10));
        assert!(scratch.mark_visited(42));
        assert!(!scratch.mark_visited(10));
        assert!(!scratch.mark_visited(42));
        assert!(scratch.mark_visited(999));

        // Query 2 (no wrap, different epoch)
        scratch.prepare(nodes);
        assert_eq!(scratch.epoch, 2);
        assert!(scratch.mark_visited(10)); // unvisited in new epoch!
        assert!(!scratch.mark_visited(10));

        // Simulate reaching near-overflow
        scratch.epoch = u16::MAX - 1;
        scratch.prepare(nodes);
        assert_eq!(scratch.epoch, u16::MAX);
        assert!(scratch.mark_visited(50));
        assert!(!scratch.mark_visited(50));

        // Wrap around to 0 -> resets to 1
        scratch.prepare(nodes);
        assert_eq!(scratch.epoch, 1);
        assert!(scratch.mark_visited(50));
        assert!(!scratch.mark_visited(50));
        assert!(scratch.mark_visited(100));
    }

    #[test]
    fn test_packed_candidate_and_live_flat_l0() {
        assert_eq!(std::mem::size_of::<Candidate>(), 8);
        assert_eq!(std::mem::size_of::<MaxCandidate>(), 8);

        let mut min_heap = BinaryHeap::new();
        min_heap.push(Candidate::new(10, 2.5));
        min_heap.push(Candidate::new(3, 0.5));
        min_heap.push(Candidate::new(7, 1.25));
        assert_eq!(min_heap.pop().unwrap().id(), 3);
        assert_eq!(min_heap.pop().unwrap().id(), 7);
        assert_eq!(min_heap.pop().unwrap().id(), 10);

        let mut seg = FlatLayer0Segment::new(4, 8, 16);
        seg.set_node(1, &[0, 2], &[1u8; 16]);
        assert_eq!(seg.get_neighbors(1), &[0, 2]);
        assert!(seg.update_neighbors_in_place(1, &[0, 2, 3]));
        assert_eq!(seg.get_neighbors(1), &[0, 2, 3]);
    }
}
