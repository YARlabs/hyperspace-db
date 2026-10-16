use dashmap::DashMap;
use hyperspace_core::{FilterExpr, Metric, SearchResult};
use std::collections::HashMap;

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct WriteBufferEntry {
    pub user_id: u32,
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

pub struct WriteBuffer {
    entries: DashMap<u32, WriteBufferEntry>, // Key: internal_id
    user_to_internal: DashMap<u32, u32>,
    count: AtomicUsize,
}

impl Default for WriteBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteBuffer {
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
            user_to_internal: DashMap::new(),
            count: AtomicUsize::new(0),
        }
    }

    /// Fast concurrent insertion
    pub fn insert(
        &self,
        internal_id: u32,
        user_id: u32,
        vector: Vec<f64>,
        metadata: HashMap<String, String>,
    ) {
        self.user_to_internal.insert(user_id, internal_id);
        if self
            .entries
            .insert(
                internal_id,
                WriteBufferEntry {
                    user_id,
                    vector,
                    metadata,
                },
            )
            .is_none()
        {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Removal when indexing is completed or failed
    pub fn remove(&self, internal_id: u32) {
        if let Some((_, entry)) = self.entries.remove(&internal_id) {
            self.user_to_internal.remove(&entry.user_id);
            self.count.fetch_sub(1, Ordering::Relaxed);
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::Relaxed) == 0
    }

    /// O(1) point lookup by user_id across in-memory write buffer entries.
    pub fn get_by_user_id(&self, user_id: u32) -> Option<(Vec<f64>, HashMap<String, String>)> {
        if let Some(internal_id) = self.user_to_internal.get(&user_id) {
            if let Some(entry) = self.entries.get(&*internal_id) {
                return Some((entry.vector.clone(), entry.metadata.clone()));
            }
        }
        None
    }

    /// Zero-copy scan across in-memory write buffer entries.
    /// Filters and distances are evaluated by reference; only top-K winners clone metadata.
    pub fn search<M: Metric>(
        &self,
        query: &[f64],
        k: usize,
        filters: &HashMap<String, String>,
        complex_filters: &[FilterExpr],
    ) -> Vec<SearchResult> {
        if self.is_empty() {
            return Vec::new();
        }

        if k == 0 {
            return Vec::new();
        }

        #[derive(Clone)]
        struct Candidate {
            internal_id: u32,
            user_id: u32,
            dist: f64,
        }
        impl PartialEq for Candidate {
            fn eq(&self, other: &Self) -> bool {
                self.dist == other.dist
            }
        }
        impl Eq for Candidate {}
        impl PartialOrd for Candidate {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for Candidate {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.dist.total_cmp(&other.dist) // Max-heap: largest distance on top
            }
        }

        let mut heap: std::collections::BinaryHeap<Candidate> =
            std::collections::BinaryHeap::with_capacity(k.min(self.size()).min(1024));

        for kv in &self.entries {
            let entry = kv.value();
            // 1. Match legacy exact key-value filters
            let mut matches = true;
            for (kf, vf) in filters {
                if entry.metadata.get(kf) != Some(vf) {
                    matches = false;
                    break;
                }
            }
            if !matches {
                continue;
            }

            // 2. Match advanced semantic/geometric/logical FilterExprs (Zero-Copy)
            if !complex_filters.is_empty() {
                for expr in complex_filters {
                    if !expr.check_coords(&entry.vector, &entry.metadata) {
                        matches = false;
                        break;
                    }
                }
                if !matches {
                    continue;
                }
            }

            let dist = M::distance(query, &entry.vector);
            if heap.len() < k {
                heap.push(Candidate {
                    internal_id: *kv.key(),
                    user_id: entry.user_id,
                    dist,
                });
            } else if let Some(top) = heap.peek() {
                if dist < top.dist {
                    heap.pop();
                    heap.push(Candidate {
                        internal_id: *kv.key(),
                        user_id: entry.user_id,
                        dist,
                    });
                }
            }
        }

        // into_sorted_vec() returns elements in ascending order (closest first)
        heap.into_sorted_vec()
            .into_iter()
            .filter_map(|cand| {
                self.entries
                    .get(&cand.internal_id)
                    .map(|entry| (cand.user_id, cand.dist, entry.metadata.clone(), None))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperspace_core::EuclideanMetric;

    #[test]
    fn test_write_buffer_basic_ops() {
        let wb = WriteBuffer::new();
        assert_eq!(wb.size(), 0);
        assert!(wb.is_empty());

        let mut meta = HashMap::new();
        meta.insert("tag".to_string(), "v1".to_string());
        wb.insert(1, 100, vec![1.0, 2.0], meta);

        assert_eq!(wb.size(), 1);
        assert!(!wb.is_empty());

        wb.remove(1);
        assert_eq!(wb.size(), 0);
        assert!(wb.is_empty());
    }

    #[test]
    fn test_write_buffer_search() {
        let wb = WriteBuffer::new();
        let mut meta1 = HashMap::new();
        meta1.insert("tag".to_string(), "a".to_string());
        wb.insert(1, 101, vec![1.0, 0.0], meta1);

        let mut meta2 = HashMap::new();
        meta2.insert("tag".to_string(), "b".to_string());
        wb.insert(2, 102, vec![0.0, 1.0], meta2);

        let query = vec![1.0, 0.5];
        let filters = HashMap::new();
        let complex_filters = vec![];

        let results = wb.search::<EuclideanMetric>(&query, 2, &filters, &complex_filters);
        assert_eq!(results.len(), 2);
        // Entry 101 should be closer (distance = sqrt((1-1)^2 + (0.5-0)^2) = 0.5)
        // than Entry 102 (distance = sqrt((1-0)^2 + (0.5-1)^2) = sqrt(1.25) ~ 1.118)
        assert_eq!(results[0].0, 101);
        assert_eq!(results[1].0, 102);

        // Test filtering
        let mut legacy_filter = HashMap::new();
        legacy_filter.insert("tag".to_string(), "b".to_string());
        let results_filtered =
            wb.search::<EuclideanMetric>(&query, 2, &legacy_filter, &complex_filters);
        assert_eq!(results_filtered.len(), 1);
        assert_eq!(results_filtered[0].0, 102);

        // Test FilterExpr
        let complex_filter = vec![FilterExpr::Match {
            key: "tag".to_string(),
            value: "a".to_string(),
        }];
        let results_complex = wb.search::<EuclideanMetric>(&query, 2, &filters, &complex_filter);
        assert_eq!(results_complex.len(), 1);
        assert_eq!(results_complex[0].0, 101);
    }
}
