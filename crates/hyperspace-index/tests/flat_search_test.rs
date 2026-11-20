//! Exact flat scans: filtered brute force and (since 4.2.5) small unfiltered indexes.

use hyperspace_core::{
    FilterExpr, GlobalConfig, Metric, PoincareMetric, QuantizationMode, SearchParams,
};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

const DIM: usize = 8;

fn poincare_point(i: u32) -> Vec<f64> {
    (0..DIM)
        .map(|d| {
            let x = ((u64::from(i) * 2_654_435_761 + d as u64 * 40_503) % 1000) as f64 / 1000.0;
            (x - 0.5) * 0.3
        })
        .collect()
}

fn params(k: usize) -> SearchParams {
    SearchParams {
        top_k: k,
        ef_search: 64,
        ..Default::default()
    }
}

/// A selective filter routes through the brute-force path; its distances must be the
/// same metric distances an unfiltered search reports (finalized exactly once).
#[test]
fn filtered_bruteforce_distances_are_finalized_once() {
    let dir = tempdir().unwrap();
    let index = HnswIndex::<PoincareMetric>::new_with_storage_precision(
        Arc::new(VectorStore::new(dir.path(), DIM * 4 + 4)),
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        DIM,
    );
    for i in 0..400u32 {
        let id = index.insert_to_storage(&poincare_point(i)).unwrap();
        let mut meta = HashMap::new();
        meta.insert("bucket".to_string(), (i % 50).to_string());
        index.index_node(id, meta).unwrap();
    }
    let q = poincare_point(10_000);
    let unfiltered = index.search(&q, &HashMap::new(), &[], &params(400));
    let filter = [FilterExpr::Match {
        key: "bucket".into(),
        value: "7".into(),
    }];
    let filtered = index.search(&q, &HashMap::new(), &filter, &params(5));
    assert!(!filtered.is_empty());
    for (id, d) in &filtered {
        let reference = PoincareMetric::distance(&poincare_point(*id), &q);
        assert!(
            (d - reference).abs() < 1e-4,
            "id {id}: filtered distance {d} vs metric distance {reference}"
        );
        if let Some((_, du)) = unfiltered.iter().find(|(u, _)| u == id) {
            assert!(
                (d - du).abs() < 1e-4,
                "id {id}: filtered {d} vs unfiltered {du}"
            );
        }
    }
}

mod flat_scan {
    use super::*;
    use hyperspace_core::{CosineMetric, EuclideanMetric};
    use hyperspace_index::set_flat_scan_limit;

    fn point(i: u32, dim: usize) -> Vec<f64> {
        let v: Vec<f64> = (0..dim)
            .map(|d| {
                let mut x = (u64::from(i) << 16 | d as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                x ^= x >> 29;
                x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
                x ^= x >> 32;
                (x % 2000) as f64 / 1000.0 - 1.0
            })
            .collect();
        let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        v.into_iter().map(|x| x / n).collect()
    }

    fn build<M: Metric>(
        mode: QuantizationMode,
        n: u32,
        dim: usize,
    ) -> (tempfile::TempDir, HnswIndex<M>) {
        let dir = tempdir().unwrap();
        let element = match mode {
            QuantizationMode::Binary => dim.div_ceil(8) + 4,
            QuantizationMode::ScalarI8 => dim + 4,
            _ => dim * 4 + 4,
        };
        let index = HnswIndex::<M>::new_with_storage_precision(
            Arc::new(VectorStore::new(dir.path(), element)),
            mode,
            Arc::new(GlobalConfig::default()),
            mode == QuantizationMode::None,
            dim,
        );
        for i in 0..n {
            let id = index.insert_to_storage(&point(i, dim)).unwrap();
            index.index_node(id, HashMap::new()).unwrap();
        }
        (dir, index)
    }

    fn exact_top_k(n: u32, dim: usize, q: &[f64], k: usize) -> Vec<u32> {
        let mut all: Vec<(u32, f64)> = (0..n)
            .map(|i| {
                let p = point(i, dim);
                (i, p.iter().zip(q).map(|(a, b)| (a - b) * (a - b)).sum())
            })
            .collect();
        all.sort_by(|a, b| a.1.total_cmp(&b.1));
        all.into_iter().take(k).map(|(i, _)| i).collect()
    }

    // One test function: the flat-scan override is process-wide.
    #[test]
    fn flat_scan_is_exact_matches_graph_and_respects_deletes() {
        let (n, dim, k) = (1500u32, 32usize, 10usize);

        // 1. Small index: flat scan returns the exact top-k (100% recall), sorted.
        set_flat_scan_limit(None);
        let (_d, index) = build::<EuclideanMetric>(QuantizationMode::None, n, dim);
        for qi in 0..20 {
            let q = point(n + qi, dim);
            let res = index.search(&q, &HashMap::new(), &[], &params(k));
            let ids: Vec<u32> = res.iter().map(|r| r.0).collect();
            assert_eq!(ids, exact_top_k(n, dim, &q, k), "query {qi}");
            assert!(res.windows(2).all(|w| w[0].1 <= w[1].1));
        }

        // 2. Same distances as the graph walk for the ids both return.
        let q = point(n + 99, dim);
        let flat = index.search(&q, &HashMap::new(), &[], &params(k));
        set_flat_scan_limit(Some(0));
        let graph = index.search(&q, &HashMap::new(), &[], &params(k));
        for (id, d) in &graph {
            if let Some((_, fd)) = flat.iter().find(|(f, _)| f == id) {
                assert!((d - fd).abs() < 1e-9, "id {id}: graph {d} vs flat {fd}");
            }
        }
        set_flat_scan_limit(None);

        // 3. Deleted nodes never come back from the scan.
        let victim = flat[0].0;
        index.delete(victim);
        let after = index.search(&q, &HashMap::new(), &[], &params(k));
        assert!(after.iter().all(|(id, _)| *id != victim));
        assert_eq!(after.len(), k);

        // 4. Cosine + 1-bit codes go through the same exact scan without panicking and
        //    return k results.
        let (_d2, bin) = build::<CosineMetric>(QuantizationMode::Binary, 600, 64);
        assert_eq!(
            bin.search(&point(9999, 64), &HashMap::new(), &[], &params(k))
                .len(),
            k
        );

        // 5. A limit below the index size keeps the graph walk.
        set_flat_scan_limit(Some(10));
        let walked = index.search(&q, &HashMap::new(), &[], &params(k));
        assert_eq!(walked.len(), k);
        set_flat_scan_limit(None);
    }
}
