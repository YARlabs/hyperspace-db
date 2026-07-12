use dashmap::DashMap;
use hyperspace_core::{vector::HyperVector, FilterExpr, Metric, SearchResult};
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct WriteBufferEntry {
    pub user_id: u32,
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
}

pub struct WriteBuffer {
    entries: DashMap<u32, WriteBufferEntry>, // Key: internal_id
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
        self.entries.insert(
            internal_id,
            WriteBufferEntry {
                user_id,
                vector,
                metadata,
            },
        );
    }

    /// Removal when indexing is completed or failed
    pub fn remove(&self, internal_id: u32) {
        self.entries.remove(&internal_id);
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Parallel scan across in-memory entries using Rayon.
    /// Filters and distances are evaluated in parallel on CPU.
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

        // Collect entries to scan them in parallel.
        // Cloning is cheap because vectors are small and we only have up to HS_WRITE_BUFFER_MAX_SIZE elements.
        let entries_vec: Vec<(u32, WriteBufferEntry)> = self
            .entries
            .iter()
            .map(|kv| (*kv.key(), kv.value().clone()))
            .collect();

        // Perform parallel filtering and distance scoring
        let mut candidates: Vec<SearchResult> = entries_vec
            .into_par_iter()
            .filter(|(_, entry)| {
                // 1. Match legacy exact key-value filters
                for (k, v) in filters {
                    if entry.metadata.get(k) != Some(v) {
                        return false;
                    }
                }

                // 2. Match advanced semantic/geometric/logical FilterExprs
                if !complex_filters.is_empty() {
                    let hv = HyperVector {
                        coords: entry.vector.clone(),
                        alpha: 1.0, // Geometric regions only read .coords, so 1.0 is perfectly safe and avoids Poincaré bounds checks
                    };
                    for expr in complex_filters {
                        if !expr.check(&hv, &entry.metadata) {
                            return false;
                        }
                    }
                }
                true
            })
            .map(|(_, entry)| {
                let dist = M::distance(query, &entry.vector);
                (entry.user_id, dist, entry.metadata.clone(), None)
            })
            .collect();

        // Sort ascending by distance (closest first)
        candidates.sort_by(|a, b| a.1.total_cmp(&b.1));
        candidates.truncate(k);
        candidates
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
