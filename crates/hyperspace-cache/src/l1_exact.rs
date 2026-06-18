use crate::eviction::EvictionPolicy;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct CacheEntry {
    pub vector: Arc<Vec<f64>>,
    pub metadata: Arc<HashMap<String, String>>,
    pub inserted_at: Instant,
    pub expires_at: Option<Instant>,
    pub access_count: AtomicU64,  // for LFU eviction
    pub last_accessed: AtomicU64, // unix timestamp ms, for LRU
}

impl CacheEntry {
    #[must_use]
    pub fn new(
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
        ttl: Option<std::time::Duration>,
    ) -> Self {
        let inserted_at = Instant::now();
        let expires_at = ttl.map(|t| inserted_at + t);
        #[allow(clippy::cast_possible_truncation)]
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            vector: Arc::new(vector),
            metadata: Arc::new(metadata),
            inserted_at,
            expires_at,
            access_count: AtomicU64::new(1),
            last_accessed: AtomicU64::new(now_ms),
        }
    }
}

pub type L1CacheEntryParts = (Arc<Vec<f64>>, Arc<HashMap<String, String>>);

pub struct L1ExactStore {
    pub(crate) map: DashMap<u32, CacheEntry>,
    pub(crate) capacity: usize,
}

impl L1ExactStore {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            map: DashMap::with_shard_amount(64), // Increase shard amount to reduce contention
            capacity,
        }
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[must_use]
    pub fn get(&self, id: u32) -> Option<Arc<Vec<f64>>> {
        // Expired entries are removed lazily.
        if self.is_expired(id) {
            let _ = self.remove(id);
            return None;
        }

        if let Some(entry) = self.map.get(&id) {
            #[allow(clippy::cast_possible_truncation)]
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            entry.last_accessed.store(now_ms, Ordering::Relaxed);
            entry.access_count.fetch_add(1, Ordering::Relaxed);

            Some(entry.vector.clone())
        } else {
            None
        }
    }

    #[must_use]
    pub fn get_entry(&self, id: u32) -> Option<L1CacheEntryParts> {
        if self.is_expired(id) {
            let _ = self.remove(id);
            return None;
        }
        if let Some(entry) = self.map.get(&id) {
            #[allow(clippy::cast_possible_truncation)]
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            entry.last_accessed.store(now_ms, Ordering::Relaxed);
            entry.access_count.fetch_add(1, Ordering::Relaxed);

            Some((entry.vector.clone(), entry.metadata.clone()))
        } else {
            None
        }
    }

    pub fn insert(&self, id: u32, entry: CacheEntry) {
        self.map.insert(id, entry);
    }

    #[must_use]
    pub fn remove(&self, id: u32) -> Option<CacheEntry> {
        self.map.remove(&id).map(|(_, entry)| entry)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    #[must_use]
    pub fn is_expired(&self, id: u32) -> bool {
        if let Some(entry) = self.map.get(&id) {
            if let Some(expires_at) = entry.expires_at {
                if Instant::now() >= expires_at {
                    return true;
                }
            }
        }
        false
    }

    /// Returns list of IDs for eviction according to the policy.
    #[must_use]
    pub fn eviction_candidates(&self, count: usize, policy: EvictionPolicy) -> Vec<u32> {
        struct Candidate {
            id: u32,
            last_accessed: u64,
            access_count: u64,
            expires_at: Option<Instant>,
        }

        if self.map.is_empty() || count == 0 {
            return Vec::new();
        }

        let mut candidates = Vec::with_capacity(self.map.len());
        for r in &self.map {
            candidates.push(Candidate {
                id: *r.key(),
                last_accessed: r.value().last_accessed.load(Ordering::Relaxed),
                access_count: r.value().access_count.load(Ordering::Relaxed),
                expires_at: r.value().expires_at,
            });
        }

        match policy {
            EvictionPolicy::Lru => {
                candidates.sort_by_key(|c| c.last_accessed);
            }
            EvictionPolicy::Lfu => {
                candidates.sort_by_key(|c| c.access_count);
            }
            EvictionPolicy::Ttl => {
                #[allow(clippy::non_std_lazy_statics)]
                // 365 * 24 * 3600 seconds is the standard way to express 1 year in Rust std.
                // clippy::duration_subsec suggests a non-existent std method.
                #[allow(clippy::items_after_statements, clippy::duration_suboptimal_units)]
                let far_future = Instant::now()
                    // 365 * 24 * 3600 = 31_536_000 secs = 1 year
                    + std::time::Duration::from_secs(31_536_000);
                candidates.sort_by_key(|c| c.expires_at.unwrap_or(far_future));
            }
        }

        candidates.into_iter().take(count).map(|c| c.id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eviction::EvictionPolicy;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_exact_hit() {
        let store = L1ExactStore::new(10);
        let vector = vec![1.0, 2.0, 3.0];
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "val".to_string());

        let entry = CacheEntry::new(vector.clone(), metadata.clone(), None);
        store.insert(42, entry);

        assert_eq!(store.len(), 1);
        let hit = store.get(42).unwrap();
        assert_eq!(*hit, vector);

        let (v, m) = store.get_entry(42).unwrap();
        assert_eq!(*v, vector);
        assert_eq!(*m, metadata);
    }

    #[test]
    fn test_expired_entry() {
        let store = L1ExactStore::new(10);
        let vector = vec![1.0, 2.0, 3.0];
        let entry = CacheEntry::new(vector, HashMap::new(), Some(Duration::from_millis(1)));
        store.insert(42, entry);

        thread::sleep(Duration::from_millis(2));
        assert!(store.is_expired(42));
        assert!(store.get(42).is_none());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_capacity_eviction() {
        let store = L1ExactStore::new(3);
        store.insert(1, CacheEntry::new(vec![1.0], HashMap::new(), None));
        thread::sleep(Duration::from_millis(5));
        store.insert(2, CacheEntry::new(vec![2.0], HashMap::new(), None));
        thread::sleep(Duration::from_millis(5));
        store.insert(3, CacheEntry::new(vec![3.0], HashMap::new(), None));
        thread::sleep(Duration::from_millis(5));

        // Touch 1 and 2 to update last_accessed/access_count
        let _ = store.get(1);
        thread::sleep(Duration::from_millis(5));
        let _ = store.get(2);
        thread::sleep(Duration::from_millis(5));

        // Eviction candidates for count=1 using LRU should be 3 (it was inserted, but not accessed since then)
        let candidates = store.eviction_candidates(1, EvictionPolicy::Lru);
        assert_eq!(candidates, vec![3]);

        // Eviction candidates using LFU should be 3 (access count is 1, while 1 and 2 have 2)
        let candidates_lfu = store.eviction_candidates(1, EvictionPolicy::Lfu);
        assert_eq!(candidates_lfu, vec![3]);
    }

    #[test]
    fn test_concurrent_reads() {
        use std::sync::Arc;
        let store = Arc::new(L1ExactStore::new(100));
        store.insert(42, CacheEntry::new(vec![1.0, 2.0], HashMap::new(), None));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let store_clone = store.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    let hit = store_clone.get(42).unwrap();
                    #[allow(clippy::float_cmp)]
                    {
                        assert_eq!(hit[0], 1.0);
                    }
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }
}
