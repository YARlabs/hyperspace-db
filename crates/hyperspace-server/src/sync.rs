use serde::{Deserialize, Serialize};

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

    pub fn hash_entry(id: u32, vector: &[f64]) -> u64 {
        // Fast FNV-1a hash instead of cryptographic SipHash (DefaultHasher)
        // Eliminates CPU bottleneck for high-dimensional vectors (e.g. 1024D = 8KB)
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        h ^= u64::from(id);
        h = h.wrapping_mul(0x0100_0000_01b3);

        for v in vector {
            h ^= v.to_bits();
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }
}
