//! Auto-tuned `ef_search` (since 4.2.5): the measured beam reaches the target graph recall.

use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::{set_flat_scan_limit, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

const DIM: usize = 32;

/// `set_flat_scan_limit` is process-wide: tests here run one at a time.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn point(i: u32) -> Vec<f64> {
    (0..DIM)
        .map(|d| {
            let mut x = (u64::from(i) << 20 | d as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            x ^= x >> 29;
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 32;
            (x % 20_001) as f64 / 10_000.0 - 1.0 + f64::from(i % 13) * 0.15
        })
        .collect()
}

fn index(n: u32, m: usize) -> (tempfile::TempDir, HnswIndex<EuclideanMetric>) {
    let dir = tempfile::tempdir().unwrap();
    let cfg = Arc::new(GlobalConfig::default());
    cfg.set_m(m);
    cfg.set_ef_construction(64);
    let idx = HnswIndex::<EuclideanMetric>::new(
        Arc::new(VectorStore::new(dir.path(), DIM * 8 + 8)),
        QuantizationMode::None,
        cfg,
        DIM,
    );
    for i in 0..n {
        let id = idx.insert_to_storage(&point(i)).unwrap();
        idx.index_node(id, HashMap::new()).unwrap();
    }
    (dir, idx)
}

fn recall(idx: &HnswIndex<EuclideanMetric>, ef: usize, k: usize) -> f64 {
    let (mut hit, mut total) = (0, 0);
    for qi in 0..40u32 {
        let q = point(1_000_000 + qi);
        set_flat_scan_limit(Some(usize::MAX >> 1));
        let truth: HashSet<u32> = idx
            .search(
                &q,
                &HashMap::new(),
                &[],
                &SearchParams {
                    top_k: k,
                    ef_search: k,
                    ..Default::default()
                },
            )
            .into_iter()
            .map(|r| r.0)
            .collect();
        set_flat_scan_limit(Some(0));
        let got = idx.search(
            &q,
            &HashMap::new(),
            &[],
            &SearchParams {
                top_k: k,
                ef_search: ef,
                ..Default::default()
            },
        );
        set_flat_scan_limit(None);
        hit += got.iter().filter(|r| truth.contains(&r.0)).count();
        total += truth.len();
    }
    hit as f64 / total as f64
}

#[test]
fn tuned_beam_reaches_the_target_and_grows_with_the_graph() {
    let _g = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (_d, idx) = index(6000, 8);
    assert_eq!(idx.tuned_ef(), 0, "untuned until measured");
    let ef = idx
        .tune_ef_search(20, 20, 0.97, 48)
        .expect("graph-sized index is tuned");
    assert_eq!(idx.tuned_ef(), ef);
    assert!(idx.tuned_at() >= 6000);
    // Held-out queries (not stored vectors) reach close to the target with that beam.
    let r = recall(&idx, ef, 20);
    assert!(r >= 0.93, "recall {r:.3} at tuned ef {ef}");
    // A wider candidate pool (reranked search) never needs a wider beam for the same target.
    let pooled = idx.tune_ef_search(20, 80, 0.97, 48).unwrap();
    assert!(pooled <= ef.max(96), "pool 80: {pooled} vs {ef}");
    // A stricter target never picks a smaller beam.
    let strict = idx.tune_ef_search(20, 20, 0.995, 48).unwrap();
    assert!(strict >= ef, "{strict} < {ef}");
}

#[test]
fn small_indexes_are_not_tuned() {
    let _g = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (_d, idx) = index(150, 16);
    assert_eq!(
        idx.tune_ef_search(10, 10, 0.99, 32),
        None,
        "flat-scanned: no beam to tune"
    );
    idx.mark_ef_tuned(150);
    assert_eq!(idx.tuned_at(), 150);
    assert_eq!(idx.tuned_ef(), 0);
}

#[test]
fn tuning_switches_to_sampled_real_queries() {
    let _g = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (_d, idx) = index(6000, 8);
    idx.tune_ef_search(10, 10, 0.97, 32).unwrap();
    assert!(!idx.tuned_with_real_queries(), "no queries yet: synthetic");
    assert!(!idx.fresh_query_samples().1);
    let p = SearchParams {
        top_k: 10,
        ef_search: 64,
        ..Default::default()
    };
    for qi in 0..200u32 {
        idx.search(&point(2_000_000 + qi), &HashMap::new(), &[], &p);
    }
    let (fresh, enough) = idx.fresh_query_samples();
    assert!(enough && fresh >= 16, "sampled {fresh}");
    let ef = idx.tune_ef_search(10, 10, 0.97, 32).unwrap();
    assert!(idx.tuned_with_real_queries());
    assert_eq!(
        idx.fresh_query_samples().0,
        0,
        "samples consumed by the tuning"
    );
    let r = recall(&idx, ef, 10);
    assert!(r >= 0.93, "recall {r:.3} at ef {ef} tuned on real queries");
}
