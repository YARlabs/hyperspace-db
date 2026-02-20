#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]

use dashmap::DashMap;
use parking_lot::RwLock;
use rand::Rng;
use rkyv::ser::Serializer;
use rkyv::{Archive, Deserialize, Serialize};
use roaring::RoaringBitmap;
use std::cell::RefCell;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, BinaryHeap};
#[cfg(feature = "persistence")]
use std::fs::File;
#[cfg(feature = "persistence")]
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

// Imports
use hyperspace_core::vector::{BinaryHyperVector, HyperVector, HyperVectorF32, QuantizedHyperVector};
use hyperspace_core::QuantizationMode;
use hyperspace_core::{GlobalConfig, Metric};
use hyperspace_store::VectorStore;
use std::marker::PhantomData;

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

use hyperspace_core::FilterExpr;

#[derive(Debug)]
pub struct MetadataIndex {
    pub inverted: DashMap<String, RoaringBitmap>,
    pub numeric: DashMap<String, BTreeMap<i64, RoaringBitmap>>,
    pub deleted: RwLock<RoaringBitmap>,
    pub forward: DashMap<u32, std::collections::HashMap<String, String>>,
}

impl Default for MetadataIndex {
    fn default() -> Self {
        Self {
            inverted: DashMap::new(),
            numeric: DashMap::new(),
            deleted: RwLock::new(RoaringBitmap::new()),
            forward: DashMap::new(),
        }
    }
}

impl<const N: usize, M: Metric<N>> HnswIndex<N, M> {
    #[cfg(feature = "persistence")]
    pub fn save_snapshot(&self, path: &std::path::Path) -> Result<(), String> {
        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        let nodes_guard = self.nodes.read();
        let mut snapshot_nodes = Vec::with_capacity(nodes_guard.len());

        for node in nodes_guard.iter() {
            let mut layers = Vec::new();
            for layer_lock in &node.layers {
                layers.push(layer_lock.read().clone());
            }
            snapshot_nodes.push(SnapshotNode {
                id: node.id,
                layers,
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

        let mut numeric_vec = Vec::new();
        for item in &self.metadata.numeric {
            let mut inner_vec = Vec::new();
            for (val, bitmap) in item.value() {
                let mut buf = Vec::new();
                bitmap.serialize_into(&mut buf).map_err(|e| e.to_string())?;
                inner_vec.push((*val, buf));
            }
            numeric_vec.push((item.key().clone(), inner_vec));
        }

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
        let bytes = serializer.into_serializer().into_inner();

        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;

        Ok(())
    }

    #[cfg(feature = "persistence")]
    pub fn load_snapshot(
        path: &std::path::Path,
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
    ) -> Result<Self, String> {
        Self::load_snapshot_with_storage_precision(path, storage, mode, config, false)
    }

    pub fn load_snapshot_with_storage_precision(
        path: &std::path::Path,
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        storage_f32: bool,
    ) -> Result<Self, String> {
        use std::time::Instant;
        let start = Instant::now();

        println!("📂 Loading snapshot: {}", path.display());

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
        let mut nodes = Vec::with_capacity(total_nodes);

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

            // Reconstruct node
            let mut layers = Vec::with_capacity(s_node.layers.len());
            for s_layer in s_node.layers {
                layers.push(RwLock::new(s_layer));
            }
            nodes.push(Node {
                id: s_node.id,
                layers,
            });
        }

        // Sync storage count
        storage.set_count(nodes.len());

        let total_time = start.elapsed();
        println!(
            "   ✅ Loaded {} nodes in {:.3}s ({:.0} nodes/s)",
            total_nodes,
            total_time.as_secs_f64(),
            total_nodes as f64 / total_time.as_secs_f64()
        );

        println!("   📦 Restoring Metadata Index...");

        let inverted = DashMap::new();
        for (k, v) in deserialized.metadata.inverted {
            let bitmap = RoaringBitmap::deserialize_from(&v[..]).unwrap_or_default();
            inverted.insert(k, bitmap);
        }

        let numeric = DashMap::new();
        for (k, v) in deserialized.metadata.numeric {
            let mut inner_map = BTreeMap::new();
            for (val, bitmap_bytes) in v {
                let bitmap = RoaringBitmap::deserialize_from(&bitmap_bytes[..]).unwrap_or_default();
                inner_map.insert(val, bitmap);
            }
            numeric.insert(k, inner_map);
        }

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

        Ok(Self {
            nodes: RwLock::new(nodes),
            metadata: MetadataIndex {
                inverted,
                numeric,
                deleted: RwLock::new(deleted),
                forward,
            },
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            storage_f32,
            config,
            has_nonempty_metadata: AtomicBool::new(has_nonempty_metadata),
            _marker: PhantomData,
        })
    }
    pub fn save_to_bytes(&self) -> Result<Vec<u8>, String> {
        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        let nodes_guard = self.nodes.read();
        let mut snapshot_nodes = Vec::with_capacity(nodes_guard.len());
        for node in nodes_guard.iter() {
            let mut layers = Vec::new();
            for layer in &node.layers {
                layers.push(layer.read().clone());
            }
            snapshot_nodes.push(SnapshotNode {
                id: node.id,
                layers,
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

        let mut numeric_vec = Vec::new();
        for item in &self.metadata.numeric {
            let mut inner_vec = Vec::new();
            for (val, bitmap) in item.value() {
                let mut buf = Vec::new();
                bitmap.serialize_into(&mut buf).map_err(|e| e.to_string())?;
                inner_vec.push((*val, buf));
            }
            numeric_vec.push((item.key().clone(), inner_vec));
        }

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
    ) -> Result<Self, String> {
        let archived = unsafe { rkyv::archived_root::<SnapshotData>(data) };

        let deserialized: SnapshotData = archived
            .deserialize(&mut rkyv::Infallible)
            .map_err(|e| format!("Deserialization error: {e}"))?;

        let mut nodes = Vec::with_capacity(deserialized.nodes.len());
        for s_node in deserialized.nodes {
            let mut layers = Vec::new();
            for s_layer in s_node.layers {
                layers.push(RwLock::new(s_layer));
            }
            nodes.push(Node {
                id: s_node.id,
                layers,
            });
        }

        storage.set_count(nodes.len());

        let inverted = DashMap::new();
        for (k, v) in deserialized.metadata.inverted {
            let bitmap = RoaringBitmap::deserialize_from(&v[..]).unwrap_or_default();
            inverted.insert(k, bitmap);
        }

        let numeric = DashMap::new();
        for (k, v) in deserialized.metadata.numeric {
            let mut inner_map = BTreeMap::new();
            for (val, bitmap_bytes) in v {
                let bitmap = RoaringBitmap::deserialize_from(&bitmap_bytes[..]).unwrap_or_default();
                inner_map.insert(val, bitmap);
            }
            numeric.insert(k, inner_map);
        }

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

        Ok(Self {
            nodes: RwLock::new(nodes),
            metadata: MetadataIndex {
                inverted,
                numeric,
                deleted: RwLock::new(deleted),
                forward,
            },
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            storage_f32: false,
            config,
            has_nonempty_metadata: AtomicBool::new(has_nonempty_metadata),
            _marker: PhantomData,
        })
    }

    pub fn get_storage(&self) -> Arc<VectorStore> {
        self.storage.clone()
    }
}

/// Node Identifier (index in VectorStore)
pub type NodeId = u32;

const MAX_LAYERS: usize = 16;

#[derive(Debug)]
pub struct HnswIndex<const N: usize, M: Metric<N>> {
    // Topology storage. Index in vector = NodeId.
    nodes: RwLock<Vec<Node>>,

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

    _marker: PhantomData<M>,
}

#[derive(Debug, Default)]
struct Node {
    id: NodeId,
    // Neighbor lists by layer.
    // layers[0] - detailed layer.
    layers: Vec<RwLock<Vec<NodeId>>>,
}

#[derive(Default)]
struct VisitedScratch {
    marks: Vec<u32>,
    generation: u32,
    candidates_l0: BinaryHeap<Candidate>,
    results_l0: BinaryHeap<std::cmp::Reverse<Candidate>>,
    candidates_layer: BinaryHeap<Candidate>,
    results_layer: BinaryHeap<Candidate>,
}

impl VisitedScratch {
    fn prepare(&mut self, len: usize) -> u32 {
        if self.marks.len() < len {
            self.marks.resize(len, 0);
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.marks.fill(0);
            self.generation = 1;
        }
        self.generation
    }
}

#[inline]
fn mark_visited(marks: &mut [u32], generation: u32, id: u32) -> bool {
    let idx = id as usize;
    let slot = &mut marks[idx];
    if *slot == generation {
        false
    } else {
        *slot = generation;
        true
    }
}

thread_local! {
    static VISITED_SCRATCH: RefCell<VisitedScratch> = RefCell::new(VisitedScratch::default());
}

/// Nearest Neighbor Candidate
#[derive(Debug, Copy, Clone, PartialEq)]
struct Candidate {
    id: NodeId,
    distance: f64,
}

// Min-Heap implementation (smaller distance = higher priority)
impl Eq for Candidate {}
impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Reverse because BinaryHeap is MaxHeap
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(CmpOrdering::Equal)
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl<const N: usize, M: Metric<N>> HnswIndex<N, M> {
    pub fn new(
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
    ) -> Self {
        Self::new_with_storage_precision(storage, mode, config, false)
    }

    pub fn new_with_storage_precision(
        storage: Arc<VectorStore>,
        mode: QuantizationMode,
        config: Arc<GlobalConfig>,
        storage_f32: bool,
    ) -> Self {
        Self {
            nodes: RwLock::new(Vec::new()),
            metadata: MetadataIndex::default(),
            entry_point: AtomicU32::new(0),
            max_layer: AtomicU32::new(0),
            storage,
            mode,
            storage_f32,
            config,
            has_nonempty_metadata: AtomicBool::new(false),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn has_nonempty_metadata(&self) -> bool {
        self.has_nonempty_metadata.load(Ordering::Relaxed)
    }

    // Support Soft Delete
    pub fn delete(&self, id: NodeId) {
        let mut del = self.metadata.deleted.write();
        del.insert(id);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn search(
        &self,
        query: &[f64],
        k: usize,
        ef_search: usize,
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        hybrid_query: Option<&str>,
        hybrid_alpha: Option<f32>,
    ) -> Vec<(NodeId, f64)> {
        // If hybrid query is present, we use RRF Fusion
        if let Some(text) = hybrid_query {
            return self.search_hybrid(
                query,
                k,
                ef_search,
                filter,
                complex_filters,
                text,
                hybrid_alpha.unwrap_or(60.0),
            );
        }

        // 1. Prepare Filter Bitmap.
        // Filter Logic: Intersection of Tag filters, Complex filters, and non-deleted items.
        let allowed_bitmap = {
            let deleted = self.metadata.deleted.read();
            let mut bitmap: Option<RoaringBitmap> = None;

            // Helper to intersect
            let mut apply_mask = |mask: &RoaringBitmap| {
                if let Some(ref mut bm) = bitmap {
                    *bm &= mask;
                } else {
                    bitmap = Some(mask.clone());
                }
            };

            // 1. Legacy Tag Filters
            if !filter.is_empty() {
                for (key, val) in filter {
                    let tag = format!("{key}:{val}");
                    if let Some(tag_bitmap) = self.metadata.inverted.get(&tag) {
                        apply_mask(&tag_bitmap);
                    } else {
                        return Vec::new(); // Short circuit
                    }
                }
            }

            // 2. Complex Filters (Range / Match)
            for expr in complex_filters {
                match expr {
                    FilterExpr::Match { key, value } => {
                        let tag = format!("{key}:{value}");
                        if let Some(tag_bitmap) = self.metadata.inverted.get(&tag) {
                            apply_mask(&tag_bitmap);
                        } else {
                            return Vec::new();
                        }
                    }
                    FilterExpr::Range { key, gte, lte } => {
                        if let Some(tree) = self.metadata.numeric.get(key) {
                            // Union all bitmaps in range
                            let mut range_union = RoaringBitmap::new();

                            // BTreeMap range
                            let start = gte.unwrap_or(i64::MIN);
                            let end = lte.unwrap_or(i64::MAX);

                            // BTreeMap::range is (Bound, Bound).
                            // We use Included.
                            for (_, bm) in tree.range(start..=end) {
                                range_union |= bm;
                            }

                            if range_union.is_empty() {
                                return Vec::new();
                            }
                            apply_mask(&range_union);
                        } else {
                            // Key not found in numeric index
                            return Vec::new();
                        }
                    }
                }
            }

            // Apply Deleted mask
            match bitmap {
                Some(mut bm) => {
                    bm -= &*deleted;
                    Some(bm)
                }
                None => {
                    // No filters? Then ALL allowed except deleted.
                    None
                }
            }
        };

        // 1. Create HyperVector from query.
        let mut aligned_query = [0.0; N];
        assert!(
            query.len() == N,
            "Query dimension mismatch provided {}, expected {}",
            query.len(),
            N
        );
        aligned_query.copy_from_slice(query);

        M::validate(&aligned_query).expect("Invalid Query Vector for this Metric");
        let q_vec = HyperVector::new_unchecked(aligned_query);

        let entry_node = self.entry_point.load(Ordering::Relaxed);

        let start_layer = {
            let guard = self.nodes.read();
            if guard.is_empty() {
                return vec![];
            }
            if (entry_node as usize) >= guard.len() {
                // Fallback to layer 0 if entry_point is out of bounds (race condition safety).
                0
            } else {
                guard[entry_node as usize].layers.len().saturating_sub(1)
            }
        };

        // Safe check for current dist
        if (entry_node as usize) >= self.nodes.read().len() {
            return vec![];
        }

        let mut curr_dist = self.dist(entry_node, &q_vec);
        let mut curr_node = entry_node;

        // 1. Zoom-in phase: Greedy search from top to layer 1.
        // Optimization: Hold read lock for the entire zoom-in phase.
        {
            let nodes_guard = self.nodes.read();
            for level in (1..=start_layer).rev() {
                let mut changed = true;
                while changed {
                    changed = false;

                    // Check bounds
                    if (curr_node as usize) >= nodes_guard.len() {
                        break;
                    }

                    // Safety check for empty/uninitialized layers (prevent panic)
                    let node = &nodes_guard[curr_node as usize];
                    if node.layers.len() <= level {
                        break; // Stop if node doesn't have this level initialized
                    }

                    // Read lock on neighbors (granular)
                    let neighbors = node.layers[level].read();

                    for &neighbor in neighbors.iter() {
                        let d = self.dist(neighbor, &q_vec);
                        if d < curr_dist {
                            curr_dist = d;
                            curr_node = neighbor;
                            changed = true;
                        }
                    }
                }
            }
        } // Read lock released here

        // 2. Local search phase: Layer 0 with Filter
        self.search_layer0(curr_node, &q_vec, k, ef_search, allowed_bitmap.as_ref())
    }

    pub fn peek(
        &self,
        limit: usize,
    ) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        let max_len = self.nodes.read().len();
        let mut result = Vec::with_capacity(limit);

        for id in (0..max_len).rev() {
            if result.len() >= limit {
                break;
            }
            let id = id as u32;

            if self.metadata.deleted.read().contains(id) {
                continue;
            }

            let vec = self.get_vector(id).coords.to_vec();
            let meta = self
                .metadata
                .forward
                .get(&id)
                .map(|m| m.clone())
                .unwrap_or_default();
            result.push((id, vec, meta));
        }
        result
    }

    pub fn peek_all(&self) -> Vec<(u32, Vec<f64>, std::collections::HashMap<String, String>)> {
        let max_len = self.nodes.read().len();
        let mut result = Vec::with_capacity(max_len);

        for id in 0..max_len {
            let id = id as u32;
            if self.metadata.deleted.read().contains(id) {
                continue;
            }
            let vec = self.get_vector(id).coords.to_vec();
            let meta = self
                .metadata
                .forward
                .get(&id)
                .map(|m| m.clone())
                .unwrap_or_default();
            result.push((id, vec, meta));
        }
        result
    }

    // Distance calculation helper
    #[inline]
    fn dist(&self, node_id: NodeId, query: &HyperVector<N>) -> f64 {
        let bytes = self.storage.get(node_id);
        match self.mode {
            QuantizationMode::ScalarI8 => {
                let q = QuantizedHyperVector::<N>::from_bytes(bytes);
                M::distance_quantized(q, query)
            }
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::<N>::from_bytes(bytes);
                M::distance_binary(b, query)
            }
            QuantizationMode::None => {
                if self.storage_f32 {
                    let v = HyperVectorF32::<N>::from_bytes(bytes);
                    let v64 = v.to_float64();
                    M::distance(&v64.coords, &query.coords)
                } else {
                    let v = HyperVector::<N>::from_bytes(bytes);
                    M::distance(&v.coords, &query.coords)
                }
            }
        }
    }

    fn search_layer0(
        &self,
        start_node: NodeId,
        query: &HyperVector<N>,
        k: usize,
        ef: usize,
        allowed: Option<&RoaringBitmap>,
    ) -> Vec<(NodeId, f64)> {
        // Optimization: Hold read lock for entire search_layer0 duration
        let nodes_guard = self.nodes.read();

        // Helper to check validity.
        // Capture 'deleted' lock if no explicit allow list is provided.
        let deleted_guard = if allowed.is_none() {
            Some(self.metadata.deleted.read())
        } else {
            None
        };

        let is_valid = |id: u32| -> bool {
            if let Some(bm) = allowed {
                bm.contains(id)
            } else {
                !deleted_guard.as_ref().unwrap().contains(id)
            }
        };

        // Safety check start_node
        if (start_node as usize) >= nodes_guard.len() {
            return vec![];
        }

        VISITED_SCRATCH.with(|scratch_cell| {
            let mut scratch = scratch_cell.borrow_mut();
            let generation = scratch.prepare(nodes_guard.len());

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

            let d = self.dist(start_node, query);
            let first = Candidate {
                id: start_node,
                distance: d,
            };

            candidates.push(first);
            if is_valid(start_node) {
                results.push(std::cmp::Reverse(first));
            }
            let _ = mark_visited(&mut scratch.marks, generation, start_node);

            while let Some(cand) = candidates.pop() {
                // Lower Bound Pruning:
                if let Some(std::cmp::Reverse(worst)) = results.peek() {
                    if results.len() >= ef && cand.distance > worst.distance {
                        break;
                    }
                }

                if (cand.id as usize) >= nodes_guard.len() {
                    continue;
                }

                let node = &nodes_guard[cand.id as usize];
                if node.layers.is_empty() {
                    continue;
                }

                let neighbors = node.layers[0].read();
                for &neighbor in neighbors.iter() {
                    if !mark_visited(&mut scratch.marks, generation, neighbor) {
                        continue;
                    }

                    let dist = self.dist(neighbor, query);

                    // Add to Candidates (Navigation).
                    // Navigation heuristic: Traverse through invalid nodes if they are promising (closer to query).
                    let mut add_to_candidates = true;
                    if let Some(std::cmp::Reverse(worst)) = results.peek() {
                        if results.len() >= ef && dist > worst.distance {
                            add_to_candidates = false;
                        }
                    }

                    if add_to_candidates {
                        let c = Candidate {
                            id: neighbor,
                            distance: dist,
                        };
                        candidates.push(c);

                        // Add to Results (Only if Valid)
                        if is_valid(neighbor) {
                            results.push(std::cmp::Reverse(c));
                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }

            let mut output = Vec::with_capacity(k.min(results.len()));
            while let Some(std::cmp::Reverse(c)) = results.pop() {
                output.push((c.id, c.distance));
            }
            output.reverse();
            output.truncate(k);

            // Keep allocated capacity for the next query on the same thread.
            candidates.clear();
            results.clear();
            scratch.candidates_l0 = candidates;
            scratch.results_l0 = results;
            output
        })
    }

    // Search candidates on a layer (returns Heap instead of sorted vec)
    fn search_layer_candidates(
        &self,
        start_node: NodeId,
        query: &HyperVector<N>,
        level: usize,
        ef: usize,
    ) -> BinaryHeap<Candidate> {
        // Optimization: Hold read lock
        let nodes_guard = self.nodes.read();

        // Safety check
        if (start_node as usize) >= nodes_guard.len() {
            return BinaryHeap::new();
        }
        if nodes_guard[start_node as usize].layers.len() <= level {
            return BinaryHeap::new();
        }

        VISITED_SCRATCH.with(|scratch_cell| {
            let mut scratch = scratch_cell.borrow_mut();
            let generation = scratch.prepare(nodes_guard.len());

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

            let d = self.dist(start_node, query);
            let first = Candidate {
                id: start_node,
                distance: d,
            };

            candidates.push(first);
            results.push(first);
            let _ = mark_visited(&mut scratch.marks, generation, start_node);

            while let Some(cand) = candidates.pop() {
                let curr_worst = results.peek().unwrap().distance;
                if cand.distance > curr_worst && results.len() >= ef {
                    break;
                }

                if (cand.id as usize) >= nodes_guard.len() {
                    continue;
                }

                let node = &nodes_guard[cand.id as usize];
                if node.layers.len() <= level {
                    continue;
                }

                let neighbors = node.layers[level].read();
                for &neighbor in neighbors.iter() {
                    if !mark_visited(&mut scratch.marks, generation, neighbor) {
                        continue;
                    }

                    let dist = self.dist(neighbor, query);

                    if results.len() < ef || dist < curr_worst {
                        let c = Candidate {
                            id: neighbor,
                            distance: dist,
                        };
                        candidates.push(c);
                        results.push(c);

                        if results.len() > ef {
                            results.pop();
                        }
                    }
                }
            }
            // Keep allocated capacity for subsequent calls on this thread.
            candidates.clear();
            scratch.candidates_layer = candidates;
            let out = std::mem::take(&mut results);
            results.clear();
            scratch.results_layer = results;
            out
        })
    }

    /// HNSW Heuristic for neighbor selection
    fn select_neighbors(
        &self,
        _query_vec: &HyperVector<N>,
        candidates: BinaryHeap<Candidate>,
        m: usize,
    ) -> Vec<NodeId> {
        let mut result = Vec::with_capacity(m);
        let mut sorted_candidates = candidates.into_sorted_vec();

        while let Some(cand) = sorted_candidates.pop() {
            // Gets closest (sorted vec is ascending, pop from end)
            if result.len() >= m {
                break;
            }

            let mut is_good = true;
            let cand_vec = self.get_vector(cand.id);

            for &existing_neighbor in &result {
                let neighbor_vec = self.get_vector(existing_neighbor);
                let dist_to_neighbor = M::distance(&cand_vec.coords, &neighbor_vec.coords);

                if dist_to_neighbor < cand.distance {
                    is_good = false;
                    break;
                }
            }

            if is_good {
                result.push(cand.id);
            }
        }
        result
    }

    // Helper to get HyperVector from id
    pub fn get_vector(&self, id: NodeId) -> HyperVector<N> {
        let bytes = self.storage.get(id);
        match self.mode {
            QuantizationMode::ScalarI8 => {
                let q = QuantizedHyperVector::<N>::from_bytes(bytes);
                let mut coords = [0.0; N];
                for (i, &c) in q.coords.iter().enumerate() {
                    coords[i] = f64::from(c) / 127.0;
                }
                HyperVector {
                    coords,
                    alpha: f64::from(q.alpha),
                }
            }
            QuantizationMode::None => {
                if self.storage_f32 {
                    let v = HyperVectorF32::<N>::from_bytes(bytes);
                    v.to_float64()
                } else {
                    let v = HyperVector::<N>::from_bytes(bytes);
                    v.clone()
                }
            }
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::<N>::from_bytes(bytes);
                let mut coords = [0.0; N];
                let val = 1.0 / (N as f64).sqrt() * 0.99;
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
        }
    }

    // Insert with Metadata
    pub fn insert_to_storage(&self, vector: &[f64]) -> Result<u32, String> {
        let mut arr = [0.0; N];
        if vector.len() != N {
            return Err("Dim mismatch".into());
        }
        arr.copy_from_slice(vector);

        // Validate against Metric logic (Poincare checks bounds, Euclidean doesn't)
        M::validate(&arr)?;

        // Create vector (we already validated)
        let q_vec_full = HyperVector::new_unchecked(arr);

        let new_id = match self.mode {
            QuantizationMode::ScalarI8 => {
                let q = QuantizedHyperVector::from_float(&q_vec_full);
                self.storage.append(q.as_bytes())?
            }
            QuantizationMode::None if self.storage_f32 => {
                let v32 = HyperVectorF32::from_float64(&q_vec_full);
                self.storage.append(v32.as_bytes())?
            }
            QuantizationMode::None => self.storage.append(q_vec_full.as_bytes())?,
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::from_float(&q_vec_full);
                self.storage.append(b.as_bytes())?
            }
        };
        Ok(new_id)
    }

    /// Update existing vector in storage (for upsert)
    pub fn update_storage(&self, id: u32, vector: &[f64]) -> Result<u32, String> {
        let mut arr = [0.0; N];
        if vector.len() != N {
            return Err("Dim mismatch".into());
        }
        arr.copy_from_slice(vector);

        // Validate against Metric logic
        M::validate(&arr)?;

        // Create vector
        let q_vec_full = HyperVector::new_unchecked(arr);

        // Update storage at existing ID
        match self.mode {
            QuantizationMode::ScalarI8 => {
                let q = QuantizedHyperVector::from_float(&q_vec_full);
                self.storage.update(id, q.as_bytes())?;
            }
            QuantizationMode::None => {
                if self.storage_f32 {
                    let v32 = HyperVectorF32::from_float64(&q_vec_full);
                    self.storage.update(id, v32.as_bytes())?;
                } else {
                    self.storage.update(id, q_vec_full.as_bytes())?;
                }
            }
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::from_float(&q_vec_full);
                self.storage.update(id, b.as_bytes())?;
            }
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

            // B. Numeric Index (i64)
            // Try parsing
            if let Ok(num) = val.parse::<i64>() {
                self.metadata
                    .numeric
                    .entry(key.clone())
                    .or_default()
                    .entry(num)
                    .or_default()
                    .insert(id);
            }

            // C. Full Text Tokenization (Simple)
            let tokens = Self::tokenize(val);
            for token in tokens {
                let token_key = format!("_txt:{token}");
                self.metadata
                    .inverted
                    .entry(token_key)
                    .or_default()
                    .insert(id);
            }
        }

        // Store full metadata for lookup (Data Explorer) - Move here to avoid clone
        self.metadata.forward.insert(id, meta);

        let q_vec = self.get_vector(id); // Helper reads from storage

        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);

        // Generate Level
        let new_level = self.random_level();

        // Create Node
        {
            let mut nodes = self.nodes.write();
            if nodes.len() <= id as usize {
                nodes.resize_with(id as usize + 1, Node::default);
            }
            let mut layers = Vec::new();
            for _ in 0..=new_level {
                layers.push(RwLock::new(Vec::new()));
            }
            nodes[id as usize] = Node { id, layers };
        }

        // Determine safe start layer for search
        let start_layer = {
            let guard = self.nodes.read();
            if guard.is_empty() || (entry_point as usize) >= guard.len() {
                0
            } else {
                guard[entry_point as usize].layers.len().saturating_sub(1)
            }
        };

        let mut curr_obj = entry_point;
        // q_vec already loaded at start of function

        // Need to check if entry_point is valid before dist calc
        let mut curr_dist = if (entry_point as usize) < self.nodes.read().len() {
            self.dist(curr_obj, &q_vec)
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
                // Read lock scope
                let neighbor = {
                    let nodes_guard = self.nodes.read();
                    if curr_obj as usize >= nodes_guard.len() {
                        break;
                    }
                    let neighbors = nodes_guard[curr_obj as usize].layers[level].read();
                    let mut best_n = None;
                    for &n in neighbors.iter() {
                        let d = self.dist(n, &q_vec);
                        if d < curr_dist {
                            curr_dist = d;
                            best_n = Some(n);
                            changed = true;
                        }
                    }
                    best_n
                };

                if let Some(n) = neighbor {
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
                let candidates_heap =
                    self.search_layer_candidates(curr_obj, &q_vec, level, ef_construction);

                // b) Select neighbors with heuristic (using layer-specific M)
                let selected_neighbors = self.select_neighbors(&q_vec, candidates_heap, m_max);

                // c) Bidirectional connect
                for &neighbor_id in &selected_neighbors {
                    self.add_link(id, neighbor_id, level);
                    self.add_link(neighbor_id, id, level);

                    // d) Pruning
                    let neighbors_len = self.nodes.read()[neighbor_id as usize].layers[level]
                        .read()
                        .len();
                    if neighbors_len > m_max {
                        self.prune_connections(neighbor_id, level, m_max);
                    }
                }

                if !selected_neighbors.is_empty() {
                    curr_obj = selected_neighbors[0];
                }
            }
        }

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

    fn add_link(&self, src: NodeId, dst: NodeId, level: usize) {
        let nodes = self.nodes.read();
        // Ensure src exists
        if src as usize >= nodes.len() {
            return;
        }

        // Potential deadlock if we hold read lock on nodes and try to write lock on layer?
        // No, RwLock is reentrant only for Read. Here we have Read on nodes, Write on layer.
        // parking_lot RwLock IS NOT Reentrant. But nodes is RwLock<Vec<Node>>.
        // We hold ReadGuard on nodes. Inside Node, layers is Vec<RwLock<Vec<u32>>>.
        // Taking WriteGuard on inner RwLock while holding ReadGuard on outer RwLock is fine as long as they are different locks.
        let mut links = nodes[src as usize].layers[level].write();
        if !links.contains(&dst) {
            links.push(dst);
        }
    }

    fn prune_connections(&self, node_id: NodeId, level: usize, max_links: usize) {
        // 1. Snapshot current links (Read Lock)
        let initial_links: Vec<u32> = {
            let nodes = self.nodes.read();
            if node_id as usize >= nodes.len() {
                return;
            }
            let layer_read = nodes[node_id as usize].layers[level].read();
            layer_read.clone()
        };

        // 2. Heavy work: calculate distances (NO LOCKS HELD)
        let node_vec = self.get_vector(node_id);
        let mut candidates = Vec::new();
        for &n in &initial_links {
            let n_vec = self.get_vector(n);
            let d = M::distance(&node_vec.coords, &n_vec.coords);
            candidates.push(Candidate { id: n, distance: d });
        }

        // Select best from snapshot
        let heap = BinaryHeap::from(candidates);
        let mut keepers = self.select_neighbors(&node_vec, heap, max_links);

        // 3. Atomic update merge (Write Lock)
        let nodes = self.nodes.read();
        let mut links_lock = nodes[node_id as usize].layers[level].write();

        // RACE CONDITION CHECK:
        // If length changed (someone added a link while we calculated),
        // we must preserve those new links!
        if links_lock.len() > initial_links.len() {
            // Find new elements strictly added after our snapshot
            for &id in links_lock.iter() {
                if !initial_links.contains(&id) {
                    // Simple strategy: always keep new links to avoid graph tearing.
                    // Even if we exceed M slightly, it's safer than losing connectivity.
                    if keepers.len() < max_links {
                        keepers.push(id);
                    }
                }
            }
        }

        *links_lock = keepers;
    }

    pub fn count_nodes(&self) -> usize {
        self.storage.count()
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
        let nodes = self.nodes.read();
        let Some(node) = nodes.get(node_id as usize) else {
            return Err(format!("Node {node_id} not found"));
        };
        if node.layers.len() <= layer {
            return Err(format!("Layer {layer} is out of bounds for node {node_id}"));
        }
        let deleted = self.metadata.deleted.read();
        let out = node.layers[layer]
            .read()
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
    ) -> Result<Vec<NodeId>, String> {
        if max_nodes == 0 {
            return Ok(Vec::new());
        }
        let nodes = self.nodes.read();
        let Some(start) = nodes.get(start_id as usize) else {
            return Err(format!("Start node {start_id} not found"));
        };
        if start.layers.len() <= layer {
            return Err(format!("Layer {layer} is out of bounds for node {start_id}"));
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
            if let Some(node) = nodes.get(node_id as usize) {
                if node.layers.len() <= layer {
                    continue;
                }
                for &next in node.layers[layer].read().iter() {
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
        let nodes = self.nodes.read();
        let deleted = self.metadata.deleted.read();
        let scan_limit = std::cmp::min(nodes.len(), max_nodes);
        let mut visited = std::collections::HashSet::new();
        let mut clusters = Vec::new();

        for node_id in 0..scan_limit as u32 {
            if deleted.contains(node_id) || !visited.insert(node_id) {
                continue;
            }
            let Some(node) = nodes.get(node_id as usize) else {
                continue;
            };
            if node.layers.len() <= layer {
                continue;
            }

            let mut queue = std::collections::VecDeque::from([(node_id, 0usize)]);
            let mut component = Vec::new();

            while let Some((curr, _)) = queue.pop_front() {
                component.push(curr);
                if component.len() >= max_nodes {
                    break;
                }
                let Some(curr_node) = nodes.get(curr as usize) else {
                    continue;
                };
                if curr_node.layers.len() <= layer {
                    continue;
                }
                for &next in curr_node.layers[layer].read().iter() {
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

    pub fn storage_stats(&self) -> (usize, usize) {
        (
            self.storage.segment_count(),
            self.storage.total_size_bytes(),
        )
    }

    #[allow(clippy::unused_self)]
    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut level = 0;
        while rng.gen::<f64>() < 0.5 && level < MAX_LAYERS - 1 {
            level += 1;
        }
        level
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(str::to_lowercase)
            .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|s: &String| !s.is_empty())
            .collect()
    }

    // RRF Fusion Logic
    #[allow(clippy::too_many_arguments)]
    fn search_hybrid(
        &self,
        query: &[f64],
        k: usize,
        ef_search: usize,
        filter: &std::collections::HashMap<String, String>,
        complex_filters: &[FilterExpr],
        text: &str,
        alpha: f32,
    ) -> Vec<(NodeId, f64)> {
        // 1. Get Vector Search Results (Semantic) -> Top K*2 for recall
        // We reuse the basic search but with NO hybrid query to avoid recursion
        let vec_k = k * 2;
        let vector_results =
            self.search(query, vec_k, ef_search, filter, complex_filters, None, None);

        // 2. Get Keyword Search Results (Lexical) -> All matching or Top N
        // Scan inverted index for tokens
        let tokens = Self::tokenize(text);
        if tokens.is_empty() {
            return vector_results.into_iter().take(k).collect();
        }

        // We calculate a score for each doc based on token overlap
        // Map<NodeId, score>
        let mut keyword_scores: std::collections::HashMap<u32, f32> =
            std::collections::HashMap::new();

        for token in tokens {
            let key = format!("_txt:{token}");
            if let Some(bitmap) = self.metadata.inverted.get(&key) {
                for id in bitmap.iter() {
                    *keyword_scores.entry(id).or_default() += 1.0;
                }
            }
        }

        // Sort keyword results
        let mut keyword_ranking: Vec<(u32, f32)> = keyword_scores.into_iter().collect();
        keyword_ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        // Take Top appropriate
        let keyword_results = keyword_ranking;

        // 3. RRF Fusion
        // RRF_score = 1 / (alpha + rank_vec) + 1 / (alpha + rank_key)

        let mut final_scores: std::collections::HashMap<u32, f32> =
            std::collections::HashMap::new();

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
            .take(k)
            .map(|(id, score)| (id, f64::from(10.0 - score)))
            .collect()
    }
}
