use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Number of buckets for Anti-Entropy (ID-based sharding)
/// 256 buckets means ~4000 vectors per bucket for 1M collection.
pub const SYNC_BUCKETS: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionDigest {
    pub collection_name: String,
    pub logical_clock: u64,
    pub vector_count: usize,
    /// Root hash (XOR of all buckets)
    pub state_hash: u64,
    /// Individual bucket hashes for diffing
    pub buckets: Vec<u64>,
}

impl Default for CollectionDigest {
    fn default() -> Self {
        Self {
            collection_name: String::new(),
            logical_clock: 0,
            vector_count: 0,
            state_hash: 0,
            buckets: vec![0; SYNC_BUCKETS],
        }
    }
}

impl CollectionDigest {
    pub fn new(name: String, clock: u64, count: usize, buckets: Vec<u64>) -> Self {
        // Calculate root hash from buckets
        let mut root = 0;
        for b in &buckets {
            root ^= b;
        }

        Self {
            collection_name: name,
            logical_clock: clock,
            vector_count: count,
            state_hash: root,
            buckets,
        }
    }

    pub fn get_bucket_index(id: u32) -> usize {
        (id as usize) % SYNC_BUCKETS
    }

    /// Computes a hash for a vector entry and returns it.
    /// In a real Merkle tree, this would be a leaf node hash.
    pub fn hash_entry(id: u32, vector: &[f64]) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        // Hashing f64 is tricky due to NaN, but we assume clean vectors here.
        // Convert to bits to hash.
        for v in vector {
            v.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
}
