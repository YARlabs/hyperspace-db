/// MetaRouter — DHT-backed chunk routing table.
///
/// Each miner node maintains a local cache of `MetaRouterEntry` records.
/// When a new chunk is committed, the entry is:
///   1. Published to the Kademlia DHT as a provider record.
///   2. Broadcast over GossipSub to all reachable peers.
///
/// When a client requests a semantic search, the MetaRouter finds the closest
/// chunk centroid (by Lorentz distance) and returns the holder peer addresses.

use dashmap::DashMap;
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A single record in the MetaRouter DHT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaRouterEntry {
    /// Chunk identifier (monotonically increasing u64).
    pub chunk_id: u64,
    /// Semantic centroid of the chunk (low-dim projection for routing).
    /// Stored as 33 f32 values — the first 32 dims + 1 hyperbolic radius.
    pub semantic_centroid: Vec<f32>,
    /// blake3 Merkle root of all vectors in this chunk.
    pub merkle_root: [u8; 32],
    /// Peers that currently hold a replica of this chunk.
    pub holder_peers: Vec<String>, // PeerId as base58 strings (serde-friendly)
    /// Unix timestamp of the last update.
    pub updated_at: u64,
}

/// Thread-safe in-memory MetaRouter cache.
#[derive(Clone)]
pub struct MetaRouter {
    /// chunk_id → MetaRouterEntry
    entries: Arc<DashMap<u64, MetaRouterEntry>>,
}

impl MetaRouter {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Insert or update an entry.
    pub fn upsert(&self, entry: MetaRouterEntry) {
        self.entries.insert(entry.chunk_id, entry);
    }

    /// Get an entry by chunk_id.
    pub fn get(&self, chunk_id: u64) -> Option<MetaRouterEntry> {
        self.entries.get(&chunk_id).map(|e| e.clone())
    }

    /// Find the N chunks whose centroids are closest to the query vector.
    /// Uses cosine similarity as a first-pass routing heuristic.
    pub fn find_nearest(&self, query: &[f32], top_n: usize) -> Vec<MetaRouterEntry> {
        let mut scored: Vec<(f32, MetaRouterEntry)> = self
            .entries
            .iter()
            .filter_map(|e| {
                let sim = cosine_similarity(query, &e.value().semantic_centroid);
                sim.map(|s| (s, e.value().clone()))
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.into_iter().take(top_n).map(|(_, e)| e).collect()
    }

    /// Return all known chunk IDs.
    pub fn all_chunk_ids(&self) -> Vec<u64> {
        self.entries.iter().map(|e| *e.key()).collect()
    }

    /// Number of known chunks.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for MetaRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Cosine similarity between two vectors.  Returns `None` if either is zero-length.
fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    Some(dot / (norm_a * norm_b))
}

/// Compute the blake3 Merkle root of a list of raw vector bytes.
/// Each leaf = blake3 hash of the serialized vector bytes.
pub fn compute_merkle_root(vector_bytes: &[Vec<u8>]) -> [u8; 32] {
    use rs_merkle::{Hasher, MerkleTree};

    #[derive(Clone)]
    struct Blake3Hasher;
    impl Hasher for Blake3Hasher {
        type Hash = [u8; 32];
        fn hash(data: &[u8]) -> [u8; 32] {
            *blake3::hash(data).as_bytes()
        }
    }

    let leaves: Vec<[u8; 32]> = vector_bytes
        .iter()
        .map(|v| *blake3::hash(v).as_bytes())
        .collect();

    let tree = MerkleTree::<Blake3Hasher>::from_leaves(&leaves);
    tree.root().unwrap_or([0u8; 32])
}
