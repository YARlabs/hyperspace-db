use serde::{Serialize, Deserialize};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// Represents a digest of a collection's state for synchronization.
/// Uses a commutative XOR-based rolling hash for set reconciliation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionDigest {
    pub collection_name: String,
    pub logical_clock: u64,
    pub vector_count: usize,
    /// Commutative hash of the collection state (XOR of all item hashes)
    pub state_hash: u64, 
}

impl CollectionDigest {
    pub fn new(name: String, clock: u64, count: usize, hash: u64) -> Self {
        Self {
            collection_name: name,
            logical_clock: clock,
            vector_count: count,
            state_hash: hash,
        }
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

