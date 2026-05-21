use crate::l1_exact::L1ExactStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EvictionPolicy {
    Lru,
    Lfu,
    Ttl,
}

pub trait AnnTombstone: Send + Sync {
    fn tombstone(&self, id: u32);
}

pub struct EvictionManager {
    pub policy: EvictionPolicy,
    pub capacity: usize,
    pub eviction_batch: usize,
}

impl EvictionManager {
    pub fn new(policy: EvictionPolicy, capacity: usize, eviction_batch: Option<usize>) -> Self {
        let batch = eviction_batch.unwrap_or((capacity / 10).max(1));
        Self {
            policy,
            capacity,
            eviction_batch: batch,
        }
    }

    pub fn maybe_evict(&self, l1: &L1ExactStore, l2: &dyn AnnTombstone) {
        if l1.len() > self.capacity {
            let count_to_evict = l1.len().saturating_sub(self.capacity).max(self.eviction_batch);
            let candidates = l1.eviction_candidates(count_to_evict, self.policy);
            for id in candidates {
                l1.remove(id);
                l2.tombstone(id);
            }
        }
    }
}
