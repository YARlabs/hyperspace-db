use dashmap::DashMap;
use parking_lot::RwLock;
use rand::Rng;
use rkyv::ser::Serializer;
use rkyv::{Archive, Deserialize, Serialize};
use roaring::RoaringBitmap;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, BinaryHeap, HashSet};
#[cfg(feature = "persistence")]
use std::fs::File;
#[cfg(feature = "persistence")]
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// Imports
use hyperspace_core::vector::{BinaryHyperVector, HyperVector, QuantizedHyperVector};
use hyperspace_core::QuantizationMode;
use hyperspace_core::{GlobalConfig, Metric};
use hyperspace_store::VectorStore;
use std::marker::PhantomData;

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)] // Requires "validation" feature
pub struct SnapshotData {
    pub max_layer: u32,
    pub entry_point: u32,
    pub nodes: Vec<SnapshotNode>,
}

#[derive(Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct SnapshotNode {
    pub id: u32,
    pub layers: Vec<Vec<u32>>,
}

// Consts removed here to use existing ones defined below in the file

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

        let data = SnapshotData {
            max_layer,
            entry_point,
            nodes: snapshot_nodes,
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
        use std::time::Instant;
        let start = Instant::now();

        println!("📂 Loading snapshot: {}", path.display());

        // 1. Memory-map the file instead of reading it all into memory
        let file = File::open(path).map_err(|e| format!("Failed to open snapshot: {}", e))?;
        let file_size = file.metadata().map_err(|e| e.to_string())?.len();
        println!("   File size: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .map(&file)
                .map_err(|e| format!("Failed to mmap snapshot: {}", e))?
        };
        let mmap_time = start.elapsed();
        println!("   ✓ Memory-mapped in {:.3}s", mmap_time.as_secs_f64());

        // 2. Validate archived data
        let archived = rkyv::check_archived_root::<SnapshotData>(&mmap)
            .map_err(|e| format!("Snapshot corruption: {}", e))?;
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

        println!("   ⏳ Reconstructing HNSW graph: {} nodes...", total_nodes);

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
                    "      Progress: {}/{} ({:.1}%) | {:.0} nodes/s | ETA: {:.1}s",
                    i, total_nodes, progress_pct, nodes_per_sec, eta
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

        Ok(Self {
            nodes: RwLock::new(nodes),
            metadata: MetadataIndex::default(),
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            config,
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

        let snapshot = SnapshotData {
            max_layer,
            entry_point,
            nodes: snapshot_nodes,
        };

        let bytes = rkyv::to_bytes::<_, 1024>(&snapshot)
            .map_err(|e| format!("Serialization error: {}", e))?;

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
            .map_err(|e| format!("Deserialization error: {}", e))?;

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

        Ok(Self {
            nodes: RwLock::new(nodes),
            metadata: MetadataIndex::default(),
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
            mode,
            config,
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
const M: usize = 16;
// const M_MAX0: usize = M * 2; // Not used in MVP yet

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

    // Runtime configuration
    pub config: Arc<GlobalConfig>,

    _marker: PhantomData<M>,
}

#[derive(Debug, Default)]
struct Node {
    id: NodeId,
    // Neighbor lists by layer.
    // layers[0] - detailed layer.
    layers: Vec<RwLock<Vec<NodeId>>>,
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
        Self {
            nodes: RwLock::new(Vec::new()),
            metadata: MetadataIndex::default(),
            entry_point: AtomicU32::new(0),
            max_layer: AtomicU32::new(0),
            storage,
            mode,
            config,
            _marker: PhantomData,
        }
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

        // 1. Prepare Filter Bitmap
        // Start with Logic: (Tag1 AND Tag2 ...) AND (Complex1 AND Complex2 ...) AND !Deleted
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
                    let tag = format!("{}:{}", key, val);
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
                        let tag = format!("{}:{}", key, value);
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
        if query.len() != N {
            panic!(
                "Query dimension mismatch provided {}, expected {}",
                query.len(),
                N
            );
        }
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
                // Determine what to do: return empty or fallback?
                // Using 0 as safe fallback if entry_node is somehow out of bounds (race?)
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

        // 1. Zoom-in phase: Greedy search from top to layer 1
        // Optimization: Hold read lock for the entire zoom-in phase to avoid repeated acquisition
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
                    let neighbors = node.layers[level as usize].read();

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

    // Distance calculation helper
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
                let v = HyperVector::<N>::from_bytes(bytes);
                M::distance(&v.coords, &query.coords)
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

        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();
        let mut visited = HashSet::new();

        // Helper to check validity
        // Note: We need to access 'deleted' lock if 'allowed' is None?
        // Or we assume 'allowed = None' means 'Check deleted manually'.
        // For perf, let's capture 'deleted' if allowed is None.
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

        let d = self.dist(start_node, query);
        let first = Candidate {
            id: start_node,
            distance: d,
        };

        candidates.push(first);
        if is_valid(start_node) {
            results.push(first);
        }
        visited.insert(start_node);

        while let Some(cand) = candidates.pop() {
            // Lower Bound Pruning:
            if let Some(worst) = results.peek() {
                if results.len() >= ef && cand.distance > worst.distance {
                    break;
                }
            }

            // Using hoisted lock
            let neighbors_ids = if (cand.id as usize) >= nodes_guard.len() {
                Vec::new()
            } else {
                let node = &nodes_guard[cand.id as usize];
                if node.layers.is_empty() {
                    Vec::new()
                } else {
                    node.layers[0].read().clone()
                }
            };

            for neighbor in neighbors_ids {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    let dist = self.dist(neighbor, query);

                    // Add to Candidates (Navigation)
                    // Logic: Keep expanding?
                    // We add to candidates if dist < worst_result OR results not full.
                    // This ensures we traverse "through" invalid nodes if they are promising (close to query).
                    let mut add_to_candidates = true;
                    if let Some(worst) = results.peek() {
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
                            results.push(c);
                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }
        }

        let mut output = Vec::new();
        while let Some(c) = results.pop() {
            output.push((c.id, c.distance));
        }
        output.reverse();
        output.truncate(k);
        output
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

        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();
        let mut visited = HashSet::new();

        let d = self.dist(start_node, query);
        let first = Candidate {
            id: start_node,
            distance: d,
        };

        candidates.push(first);
        results.push(first);
        visited.insert(start_node);

        while let Some(cand) = candidates.pop() {
            let curr_worst = results.peek().unwrap().distance;
            if cand.distance > curr_worst && results.len() >= ef {
                break;
            }

            // Using hoisted lock with bounds check
            let neighbors_ids = if (cand.id as usize) >= nodes_guard.len() {
                Vec::new()
            } else {
                let node = &nodes_guard[cand.id as usize];
                if node.layers.len() <= level {
                    Vec::new()
                } else {
                    node.layers[level].read().clone()
                }
            };

            for neighbor in neighbors_ids {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
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
        }
        results
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
                let dist_to_neighbor = cand_vec.poincare_distance_sq(&neighbor_vec);

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
                    coords[i] = c as f64 / 127.0;
                }
                HyperVector {
                    coords,
                    alpha: q.alpha as f64,
                }
            }
            QuantizationMode::None => {
                let v = HyperVector::<N>::from_bytes(bytes);
                v.clone()
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
                    alpha: b.alpha as f64,
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
                self.storage.update(id, q_vec_full.as_bytes())?;
            }
            QuantizationMode::Binary => {
                let b = BinaryHyperVector::from_float(&q_vec_full);
                self.storage.update(id, b.as_bytes())?;
            }
        };
        Ok(id)
    }

    pub fn index_node(
        &self,
        id: NodeId,
        meta: std::collections::HashMap<String, String>,
    ) -> Result<(), String> {
        // Store full metadata for lookup (Data Explorer)
        self.metadata.forward.insert(id, meta.clone());

        // 2. Index Metadata
        for (key, val) in &meta {
            // A. Inverted Index (Text)
            let tag = format!("{}:{}", key, val);
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
                let token_key = format!("_txt:{}", token);
                self.metadata
                    .inverted
                    .entry(token_key)
                    .or_default()
                    .insert(id);
            }
        }

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
            if guard.is_empty() {
                0
            } else if (entry_point as usize) >= guard.len() {
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
        let ef_construction = self.config.get_ef_construction();
        const M_MAX: usize = M;

        for level in (0..=std::cmp::min(new_level, max_layer as usize)).rev() {
            // a) Search candidates
            let candidates_heap =
                self.search_layer_candidates(curr_obj, &q_vec, level, ef_construction);

            // b) Select neighbors with heuristic
            let selected_neighbors = self.select_neighbors(&q_vec, candidates_heap, M);

            // c) Bidirectional connect
            for &neighbor_id in &selected_neighbors {
                self.add_link(id, neighbor_id, level);
                self.add_link(neighbor_id, id, level);

                // d) Pruning
                let neighbors_len = self.nodes.read()[neighbor_id as usize].layers[level]
                    .read()
                    .len();
                if neighbors_len > M_MAX {
                    self.prune_connections(neighbor_id, level, M_MAX);
                }
            }

            // Move entry point for next layer to the best found candidate
            if !selected_neighbors.is_empty() {
                curr_obj = selected_neighbors[0];
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
        // Drop logic inside to avoid long write locks block
        let links_copy: Vec<u32> = {
            let nodes = self.nodes.read();
            if node_id as usize >= nodes.len() {
                return;
            }
            let layer_read = nodes[node_id as usize].layers[level].read();
            layer_read.clone()
        };

        // 1. Get vectors
        let node_vec = self.get_vector(node_id);
        let mut candidates = Vec::new();
        for &n in links_copy.iter() {
            let n_vec = self.get_vector(n);
            let d = node_vec.poincare_distance_sq(&n_vec);
            candidates.push(Candidate { id: n, distance: d });
        }

        // 2. Select best
        let heap = BinaryHeap::from(candidates);
        let keepers = self.select_neighbors(&node_vec, heap, max_links);

        // 3. Replace list
        let nodes = self.nodes.read();
        let mut links = nodes[node_id as usize].layers[level].write();
        *links = keepers;
    }

    pub fn count_nodes(&self) -> usize {
        self.storage.count()
    }

    pub fn count_deleted(&self) -> usize {
        self.metadata.deleted.read().len() as usize
    }

    pub fn storage_stats(&self) -> (usize, usize) {
        (
            self.storage.segment_count(),
            self.storage.total_size_bytes(),
        )
    }

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
            .map(|s| s.to_lowercase())
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
            let key = format!("_txt:{}", token);
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
            .map(|(id, score)| (id, (10.0 - score) as f64))
            .collect()
    }
}
