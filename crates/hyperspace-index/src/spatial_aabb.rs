use parking_lot::RwLock;
use roaring::RoaringBitmap;
use serde::{Deserialize, Serialize};

/// Segment block size for AABB bounding volume pruning.
pub const AABB_SEGMENT_SIZE: usize = 512;

/// Axis-Aligned Bounding Box for a range of vector IDs [start_id, end_id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AabbSegment {
    pub start_id: u32,
    pub end_id: u32,
    pub min_bounds: Vec<f64>,
    pub max_bounds: Vec<f64>,
}

impl AabbSegment {
    pub fn new(start_id: u32, end_id: u32, dim: usize) -> Self {
        Self {
            start_id,
            end_id,
            min_bounds: vec![f64::INFINITY; dim],
            max_bounds: vec![f64::NEG_INFINITY; dim],
        }
    }

    /// Extends this segment's bounding box with the given coordinates.
    pub fn extend(&mut self, coords: &[f64]) {
        let dim = self.min_bounds.len().min(coords.len());
        for d in 0..dim {
            if coords[d] < self.min_bounds[d] {
                self.min_bounds[d] = coords[d];
            }
            if coords[d] > self.max_bounds[d] {
                self.max_bounds[d] = coords[d];
            }
        }
    }

    /// Tests if this segment's AABB overlaps with the query bounding box.
    #[inline]
    pub fn intersects(&self, query_min: &[f64], query_max: &[f64]) -> bool {
        let dim = self
            .min_bounds
            .len()
            .min(query_min.len())
            .min(query_max.len());
        for d in 0..dim {
            if self.min_bounds[d] > query_max[d] || self.max_bounds[d] < query_min[d] {
                return false;
            }
        }
        true
    }
}

/// Thread-safe Spatial Axis-Aligned Bounding Box Index for rapid geometric filter pruning.
#[derive(Debug)]
pub struct SpatialAabbIndex {
    dimension: usize,
    segments: RwLock<Vec<AabbSegment>>,
}

impl SpatialAabbIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            segments: RwLock::new(Vec::new()),
        }
    }

    /// Registers a newly inserted vector into the spatial bounding index.
    pub fn insert_vector(&self, id: u32, coords: &[f64]) {
        let mut segments = self.segments.write();
        let target_seg = (id as usize) / AABB_SEGMENT_SIZE;

        while segments.len() <= target_seg {
            let start = (segments.len() * AABB_SEGMENT_SIZE) as u32;
            let end = start + AABB_SEGMENT_SIZE as u32;
            segments.push(AabbSegment::new(start, end, self.dimension));
        }

        segments[target_seg].extend(coords);
    }

    /// Prunes segments and returns candidate vector IDs matching the query bounding box.
    ///
    /// If an optional candidate bitmap is provided (e.g. from inverted/range filters),
    /// pruning only inspects vectors already present in that bitmap.
    pub fn query_candidates_box(
        &self,
        query_min: &[f64],
        query_max: &[f64],
        candidate_mask: Option<&RoaringBitmap>,
        deleted: &RoaringBitmap,
        total_count: u32,
    ) -> RoaringBitmap {
        let segments = self.segments.read();
        let mut candidates = RoaringBitmap::new();

        if segments.is_empty() {
            // Fallback: if index is not yet built, include all non-deleted
            if let Some(mask) = candidate_mask {
                for id in mask {
                    if !deleted.contains(id) {
                        candidates.insert(id);
                    }
                }
            } else {
                for id in 0..total_count {
                    if !deleted.contains(id) {
                        candidates.insert(id);
                    }
                }
            }
            return candidates;
        }

        for seg in segments.iter() {
            if !seg.intersects(query_min, query_max) {
                // Whole 512-vector segment is completely outside the box! Prune immediately.
                continue;
            }

            let start = seg.start_id;
            let end = seg.end_id.min(total_count);

            if let Some(mask) = candidate_mask {
                for id in start..end {
                    if mask.contains(id) && !deleted.contains(id) {
                        candidates.insert(id);
                    }
                }
            } else {
                for id in start..end {
                    if !deleted.contains(id) {
                        candidates.insert(id);
                    }
                }
            }
        }

        candidates
    }

    /// Prunes segments for Euclidean ball filter: [center - radius, center + radius].
    pub fn query_candidates_ball(
        &self,
        center: &[f64],
        radius: f64,
        candidate_mask: Option<&RoaringBitmap>,
        deleted: &RoaringBitmap,
        total_count: u32,
    ) -> RoaringBitmap {
        let dim = self.dimension.min(center.len());
        let mut q_min = Vec::with_capacity(dim);
        let mut q_max = Vec::with_capacity(dim);

        for d in 0..dim {
            q_min.push(center[d] - radius);
            q_max.push(center[d] + radius);
        }

        self.query_candidates_box(&q_min, &q_max, candidate_mask, deleted, total_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_segment_intersection() {
        let mut seg = AabbSegment::new(0, 512, 3);
        seg.extend(&[1.0, 2.0, 3.0]);
        seg.extend(&[4.0, 5.0, 6.0]);

        // Overlapping query box
        assert!(seg.intersects(&[0.0, 0.0, 0.0], &[2.0, 3.0, 4.0]));
        // Non-overlapping query box (below min)
        assert!(!seg.intersects(&[-5.0, -5.0, -5.0], &[0.5, 1.5, 2.5]));
        // Non-overlapping query box (above max)
        assert!(!seg.intersects(&[5.0, 6.0, 7.0], &[10.0, 10.0, 10.0]));
    }

    #[test]
    fn test_spatial_aabb_pruning_performance() {
        let index = SpatialAabbIndex::new(3);
        // Insert 1024 vectors across 2 segments:
        // Segment 0: vectors in [0.0..1.0]
        for i in 0..512 {
            index.insert_vector(i, &[0.1, 0.2, 0.3]);
        }
        // Segment 1: vectors in [100.0..101.0]
        for i in 512..1024 {
            index.insert_vector(i, &[100.1, 100.2, 100.3]);
        }

        let deleted = RoaringBitmap::new();
        // Query box around [0.0..1.0]
        let candidates =
            index.query_candidates_box(&[0.0, 0.0, 0.0], &[1.0, 1.0, 1.0], None, &deleted, 1024);

        // Pruned Segment 1 completely: candidates should contain only IDs 0..512
        assert_eq!(candidates.len(), 512);
        assert!(candidates.contains(0));
        assert!(candidates.contains(511));
        assert!(!candidates.contains(512));
        assert!(!candidates.contains(1023));

        // Ball query test
        let ball_cands = index.query_candidates_ball(&[0.1, 0.2, 0.3], 0.5, None, &deleted, 1024);
        assert_eq!(ball_cands.len(), 512);
        assert!(!ball_cands.contains(512));
    }
}
