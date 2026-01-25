use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use rkyv::{Archive, Deserialize, Serialize};
use rkyv::ser::Serializer;
use std::fs::File;
use std::io::Write;
use parking_lot::RwLock;
use rand::Rng;
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BinaryHeap, HashSet};

// Imports
use hyperspace_core::vector::HyperVector;
use hyperspace_store::VectorStore;

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

impl HnswIndex {
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
        serializer.serialize_value(&data).map_err(|e: rkyv::ser::serializers::CompositeSerializerError<_,_,_>| e.to_string())?;
        let bytes = serializer.into_serializer().into_inner();
        
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(&bytes).map_err(|e| e.to_string())?;
        
        Ok(())
    }

    pub fn load_snapshot(path: &std::path::Path, storage: Arc<VectorStore>) -> Result<Self, String> {
        let file_content = std::fs::read(path).map_err(|e| e.to_string())?;
        
        // Validate and deserialize
        let archived = rkyv::check_archived_root::<SnapshotData>(&file_content)
            .map_err(|e| format!("Snapshot corruption: {}", e))?;
            
        let deserialized: SnapshotData = archived.deserialize(&mut rkyv::Infallible).unwrap();
        
        // Reconstruct Graph
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
        
        // Sync storage count
        storage.set_count(nodes.len());
        println!("Restored {} nodes from snapshot topology.", nodes.len());

        Ok(Self {
            nodes: RwLock::new(nodes),
            metadata: RwLock::new(std::collections::HashMap::new()), 
            entry_point: AtomicU32::new(deserialized.entry_point),
            max_layer: AtomicU32::new(deserialized.max_layer),
            storage,
        })
    }
}

/// Node Identifier (index in VectorStore)
pub type NodeId = u32;

const MAX_LAYERS: usize = 16;
const M: usize = 16; 
// const M_MAX0: usize = M * 2; // Not used in MVP yet

#[derive(Debug)]
pub struct HnswIndex {
    // Topology storage. Index in vector = NodeId.
    nodes: RwLock<Vec<Node>>,
    
    // Metadata storage
    pub metadata: RwLock<std::collections::HashMap<NodeId, std::collections::HashMap<String, String>>>,

    // Graph entry point (top level)
    entry_point: AtomicU32,
    
    // Current max layer
    max_layer: AtomicU32,
    
    // Reference to data (raw vectors)
    storage: Arc<VectorStore>,
}

#[derive(Debug)]
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
        other.distance.partial_cmp(&self.distance).unwrap_or(CmpOrdering::Equal)
    }
}
impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl HnswIndex {
    pub fn new(storage: Arc<VectorStore>) -> Self {
        Self {
            nodes: RwLock::new(Vec::new()),
            metadata: RwLock::new(std::collections::HashMap::new()),
            entry_point: AtomicU32::new(0),
            max_layer: AtomicU32::new(0),
            storage,
        }
    }

    /// Search K nearest neighbors with Filter
    pub fn search(&self, query: &[f64], k: usize, ef_search: usize, filter: &std::collections::HashMap<String, String>) -> Vec<(NodeId, f64)> {
        // 1. Create HyperVector from query.
        // Assuming DIM=8 for MVP as per user hardcode in dist()
        let mut aligned_query = [0.0; 8];
        if query.len() != 8 {
            // Panic for now, real code should handle error
            panic!("Query dimension mismatch provided {}, expected 8", query.len());
        }
        aligned_query.copy_from_slice(query);
        let q_vec = HyperVector::new(aligned_query).unwrap(); // Validate logic in real app

        let entry_node = self.entry_point.load(Ordering::Relaxed);
        let max_layer = self.max_layer.load(Ordering::Relaxed);
        
        let mut curr_dist = self.dist(entry_node, &q_vec);
        let mut curr_node = entry_node;

        // 1. Zoom-in phase: Greedy search from top to layer 1
        for level in (1..=max_layer).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                // Read lock on nodes vector, then read lock on neighbors
                // Note: This 2-step lock might be tricky if nodes is resizing, but RwLock<Vec> holds the reference.
                // Actually nodes[id] gives us the Node struct.
                let nodes_guard = self.nodes.read();
                // Check bounds 
                if (curr_node as usize) >= nodes_guard.len() { break; } 

                let neighbors = nodes_guard[curr_node as usize]
                    .layers[level as usize]
                    .read();
                
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

        // 2. Local search phase: Layer 0 with Filter
        self.search_layer0(curr_node, &q_vec, k, ef_search, filter)
    }

    // Distance calculation helper
    #[inline]
    fn dist(&self, node_id: NodeId, query: &HyperVector<8>) -> f64 { 
        let slice = self.storage.get(node_id as usize); 
        // Force copy into simple array to avoid reference issues
        let mut array = [0.0; 8];
        // Ensure slice has enough data (should be guaranteed by storage logic, but safe check needed in real prod)
        if slice.len() >= 8 {
            array.copy_from_slice(&slice[0..8]);
        }
        
        // Create temp vector on stack (copy)
        let node_vec = HyperVector::new(array).unwrap(); 
        
        query.poincare_distance_sq(&node_vec)
    }
    
    fn search_layer0(&self, start_node: NodeId, query: &HyperVector<8>, k: usize, ef: usize, filter: &std::collections::HashMap<String, String>) -> Vec<(NodeId, f64)> {
        let mut candidates = BinaryHeap::new(); 
        let mut results = BinaryHeap::new();    
        let mut visited = HashSet::new();

        let d = self.dist(start_node, query);
        let first = Candidate { id: start_node, distance: d };
        
        candidates.push(first);
        results.push(first);
        visited.insert(start_node);

        while let Some(cand) = candidates.pop() {
            let curr_worst = results.peek().unwrap().distance;
            if cand.distance > curr_worst && results.len() >= ef {
                break;
            }

            // Lock scope: get neighbors list
            let neighbors_ids = {
                let nodes_guard = self.nodes.read();
                if (cand.id as usize) >= nodes_guard.len() { 
                    Vec::new() // Handle out of bounds gracefully
                } else {
                    let layer_guard = nodes_guard[cand.id as usize].layers[0].read();
                    layer_guard.clone() 
                }
            };

            for neighbor in neighbors_ids {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    let dist = self.dist(neighbor, query);
                    
                    if results.len() < ef || dist < curr_worst {
                        let c = Candidate { id: neighbor, distance: dist };
                        candidates.push(c);
                        results.push(c);
                        
                        if results.len() > ef {
                            results.pop(); 
                        }
                    }
                }
            }
        }
        
        // Filter results after traversal (Post-filtering)
        // Note: Ideally filtering should happen during collection to guarantee K results.
        // HNSW Post-filtering can return < K results if many are filtered.
        // For MVP, we filter the results heap.
        let meta_guard = self.metadata.read();
        
        let mut output = Vec::new();
        while let Some(c) = results.pop() {
            // Check filter
            let mut match_filter = true;
            if !filter.is_empty() {
                if let Some(node_meta) = meta_guard.get(&c.id) {
                     for (k, v) in filter {
                         if node_meta.get(k) != Some(v) {
                             match_filter = false;
                             break;
                         }
                     }
                } else {
                    match_filter = false; // No metadata, but filter requested -> mismatch? Or assume allow? USUALLY mismatch.
                }
            }
            
            if match_filter {
                output.push((c.id, c.distance));
            }
        }
        output.reverse();
        output.truncate(k);
        output
    }

    // Search candidates on a layer (returns Heap instead of sorted vec)
    fn search_layer_candidates(&self, start_node: NodeId, query: &HyperVector<8>, level: usize, ef: usize) -> BinaryHeap<Candidate> {
        let mut candidates = BinaryHeap::new(); 
        let mut results = BinaryHeap::new();    
        let mut visited = HashSet::new();

        let d = self.dist(start_node, query);
        let first = Candidate { id: start_node, distance: d };
        
        candidates.push(first);
        results.push(first);
        visited.insert(start_node);

        while let Some(cand) = candidates.pop() {
            let curr_worst = results.peek().unwrap().distance;
            if cand.distance > curr_worst && results.len() >= ef {
                break;
            }

            // Lock scope
            let neighbors_ids = {
                let nodes_guard = self.nodes.read();
                if (cand.id as usize) >= nodes_guard.len() { 
                    Vec::new() 
                } else {
                    let layer_guard = nodes_guard[cand.id as usize].layers[level].read();
                    layer_guard.clone() 
                }
            };

            for neighbor in neighbors_ids {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    let dist = self.dist(neighbor, query);
                    
                    if results.len() < ef || dist < curr_worst {
                        let c = Candidate { id: neighbor, distance: dist };
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
        query_vec: &HyperVector<8>,
        mut candidates: BinaryHeap<Candidate>,
        m: usize,
    ) -> Vec<NodeId> {
        let mut result = Vec::with_capacity(m);
        let mut sorted_candidates = candidates.into_sorted_vec(); 
        
        while let Some(cand) = sorted_candidates.pop() { // Gets closest (sorted vec is ascending, pop from end)
            if result.len() >= m { break; }

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
    fn get_vector(&self, id: NodeId) -> HyperVector<8> {
         let slice = self.storage.get(id as usize);
         let arr: [f64; 8] = slice[0..8].try_into().unwrap();
         // Alpha is stored at the end of DIM coords. 
         // Stride was (DIM+1)*8 bytes. Slice includes alpha at index 8.
         let alpha = slice[8]; 
         HyperVector { coords: arr, alpha } 
    }

    // Insert with Metadata
    pub fn insert(&self, vector: &[f64], meta: std::collections::HashMap<String, String>) -> Result<u32, String> {
        // 1. Storage Append
        let new_id = self.storage.append(vector)?;
        
        // 2. Save Metadata
        {
            let mut m = self.metadata.write();
            m.insert(new_id, meta);
        }

        let q_vec = self.get_vector(new_id);

        let max_layer = self.max_layer.load(Ordering::Relaxed);
        let entry_point = self.entry_point.load(Ordering::Relaxed);
        
        // Generate Level
        let new_level = self.random_level();
        
        // Create Node
        {
            let mut nodes = self.nodes.write();
            if nodes.len() <= new_id as usize {
                 nodes.resize_with(new_id as usize + 1, Node::default); 
            }
             let mut layers = Vec::new();
             for _ in 0..=new_level { layers.push(RwLock::new(Vec::new())); }
             nodes[new_id as usize] = Node { id: new_id, layers };
        }

        let mut curr_obj = entry_point;
        let mut curr_dist = self.dist(curr_obj, &q_vec);

        // 2. Phase 1: Zoom in (Greedy Search) from top to new_level
        for level in (new_level+1..=(max_layer as usize)).rev() {
             let mut changed = true;
             while changed {
                 changed = false;
                 // Read lock scope
                 let neighbor = {
                    let nodes_guard = self.nodes.read();
                    if curr_obj as usize >= nodes_guard.len() { break; }
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
        const EF_CONSTRUCTION: usize = 100; // Tuning param
        const M_MAX: usize = M; 

        for level in (0..=std::cmp::min(new_level, max_layer as usize)).rev() {
            // a) Search candidates
            let candidates_heap = self.search_layer_candidates(curr_obj, &q_vec, level, EF_CONSTRUCTION);
            
            // b) Select neighbors with heuristic
            let selected_neighbors = self.select_neighbors(&q_vec, candidates_heap, M);

            // c) Bidirectional connect
            for &neighbor_id in &selected_neighbors {
                self.add_link(new_id, neighbor_id, level);
                self.add_link(neighbor_id, new_id, level);
                
                // d) Pruning
                let neighbors_len = self.nodes.read()[neighbor_id as usize].layers[level].read().len();
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
            self.entry_point.store(new_id, Ordering::SeqCst);
        }

        Ok(new_id)
    }

    fn add_link(&self, src: NodeId, dst: NodeId, level: usize) {
        let nodes = self.nodes.read(); 
        // Ensure src exists
        if src as usize >= nodes.len() { return; }
        
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
            if node_id as usize >= nodes.len() { return; }
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

    fn random_level(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut level = 0;
        while rng.gen::<f64>() < 0.5 && level < MAX_LAYERS - 1 {
            level += 1;
        }
        level
    }
}

impl Default for Node {
    fn default() -> Self {
        Self { id: 0, layers: Vec::new() }
    }
}
