//! Per-vector scaled ScalarI8 (`medium`) codes and their integer kernel (since 4.2.5).
//!
//! The process-wide A/B switches are shared state, so every test here holds `LOCK`.

use hyperspace_core::{
    CosineMetric, EuclideanMetric, GlobalConfig, Metric, QuantizationMode, SearchParams,
};
use hyperspace_index::{set_flat_scan_limit, set_sq8_int_kernel, set_sq8_scaled_codes, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};
use tempfile::tempdir;

static LOCK: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

const DIM: usize = 256;

fn rnd(mut x: u64) -> f64 {
    x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 31;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 29;
    (x % 2_000_001) as f64 / 1_000_000.0 - 1.0
}

/// Clustered unit vectors, the shape of real embeddings (small per-coordinate values).
fn point(i: u32) -> Vec<f64> {
    let c = u64::from(i % 24);
    let v: Vec<f64> = (0..DIM as u64)
        .map(|d| rnd(c * 7919 + d) + 0.35 * rnd(u64::from(i) * 1_000_003 + d + 17))
        .collect();
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    v.into_iter().map(|x| x / n).collect()
}

fn query(i: u32) -> Vec<f64> {
    point(1_000_000 + i)
}

fn new_index<M: Metric>(dir: &Path) -> HnswIndex<M> {
    HnswIndex::<M>::new(
        Arc::new(VectorStore::new(dir, DIM + 4)),
        QuantizationMode::ScalarI8,
        Arc::new(GlobalConfig::default()),
        DIM,
    )
}

fn fill<M: Metric>(index: &HnswIndex<M>, n: u32) {
    for i in 0..n {
        let id = index.insert_to_storage(&point(i)).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
}

fn params(k: usize) -> SearchParams {
    SearchParams {
        top_k: k,
        ef_search: 200,
        ..Default::default()
    }
}

fn truth(n: u32, q: &[f64], k: usize) -> HashSet<u32> {
    let mut d: Vec<(f64, u32)> = (0..n)
        .map(|i| {
            let p = point(i);
            (p.iter().zip(q).map(|(a, b)| (a - b) * (a - b)).sum(), i)
        })
        .collect();
    d.sort_by(|a, b| a.0.total_cmp(&b.0));
    d.iter().take(k).map(|x| x.1).collect()
}

fn recall<M: Metric>(index: &HnswIndex<M>, n: u32, queries: u32) -> f64 {
    let k = 10;
    let mut hit = 0usize;
    for qi in 0..queries {
        let q = query(qi);
        let t = truth(n, &q, k);
        let res = index.search(&q, &HashMap::new(), &[], &params(k));
        hit += res.iter().filter(|(id, _)| t.contains(id)).count();
    }
    hit as f64 / (k * queries as usize) as f64
}

fn ids<M: Metric>(index: &HnswIndex<M>, q: &[f64]) -> Vec<u32> {
    index
        .search(q, &HashMap::new(), &[], &params(10))
        .into_iter()
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn scaled_codes_beat_legacy_codes_on_recall() {
    let _g = lock();
    let n = 3000;
    let mut r = Vec::new();
    for scaled in [false, true] {
        set_sq8_scaled_codes(scaled);
        let dir = tempdir().unwrap();
        let index = new_index::<EuclideanMetric>(dir.path());
        assert_eq!(index.sq8_scaled(), scaled);
        fill(&index, n);
        r.push(recall(&index, n, 40));
    }
    set_sq8_scaled_codes(true);
    let (legacy, scaled) = (r[0], r[1]);
    assert!(scaled >= 0.95, "scaled SQ8 recall {scaled:.3}");
    assert!(
        scaled > legacy + 0.02,
        "scaled {scaled:.3} should beat legacy {legacy:.3}"
    );
}

#[test]
fn hnsw_and_flat_scan_agree_for_scaled_codes() {
    let _g = lock();
    let dir = tempdir().unwrap();
    let index = new_index::<CosineMetric>(dir.path());
    assert!(index.sq8_scaled());
    fill(&index, 1500);
    let mut flat_hits = 0;
    for qi in 0..20 {
        let q = query(qi);
        set_flat_scan_limit(Some(0));
        let graph = ids(&index, &q);
        set_flat_scan_limit(Some(usize::MAX));
        let flat = ids(&index, &q);
        set_flat_scan_limit(None);
        flat_hits += graph.iter().filter(|id| flat.contains(id)).count();
    }
    assert!(flat_hits >= 190, "graph vs flat overlap {flat_hits}/200");
}

#[test]
fn int_kernel_and_f32_kernel_return_the_same_neighbours() {
    let _g = lock();
    let dir = tempdir().unwrap();
    let index = new_index::<EuclideanMetric>(dir.path());
    fill(&index, 2000);
    let mut same = 0;
    for qi in 0..20 {
        let q = query(qi);
        set_sq8_int_kernel(false);
        let f = ids(&index, &q);
        set_sq8_int_kernel(true);
        let i = ids(&index, &q);
        same += i.iter().filter(|id| f.contains(id)).count();
    }
    assert!(same >= 190, "int vs f32 kernel overlap {same}/200");
}

#[test]
fn distances_are_close_to_exact_and_vectors_decode() {
    let _g = lock();
    let dir = tempdir().unwrap();
    let index = new_index::<EuclideanMetric>(dir.path());
    fill(&index, 500);
    let q = query(3);
    for (id, d) in index.search(&q, &HashMap::new(), &[], &params(10)) {
        let exact = EuclideanMetric::distance(&point(id), &q);
        assert!((d - exact).abs() < 0.02, "id {id}: {d} vs {exact}");
        let v = index.get_vector(id);
        let err: f64 = v
            .coords
            .iter()
            .zip(point(id))
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        assert!(err < 0.01, "decoded vector error {err}");
    }
}

#[test]
fn updates_are_encoded_with_the_index_format() {
    let _g = lock();
    let dir = tempdir().unwrap();
    let index = new_index::<EuclideanMetric>(dir.path());
    fill(&index, 300);
    let target = point(777_777);
    index.update_storage(5, &target).unwrap();
    let decoded = index.get_vector(5);
    let err: f64 = decoded
        .coords
        .iter()
        .zip(&target)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max);
    assert!(err < 0.01, "updated vector decode error {err}");
}

#[cfg(feature = "persistence")]
fn reload<M: Metric>(dir: &Path, snap: &Path) -> HnswIndex<M> {
    HnswIndex::<M>::load_snapshot(
        snap,
        Arc::new(VectorStore::new(dir, DIM + 4)),
        QuantizationMode::ScalarI8,
        Arc::new(GlobalConfig::default()),
        DIM,
    )
    .unwrap()
}

/// The code format is a property of the stored data: a snapshot written with legacy codes
/// (no `sq8.scaled` marker) must keep decoding them that way, whatever the current default.
#[cfg(feature = "persistence")]
#[test]
fn code_format_survives_snapshot_reload() {
    let _g = lock();
    for scaled in [false, true] {
        set_sq8_scaled_codes(scaled);
        let dir = tempdir().unwrap();
        let snap = dir.path().join("index.snap");
        let before: Vec<Vec<u32>>;
        {
            let index = new_index::<EuclideanMetric>(dir.path());
            fill(&index, 800);
            before = (0..10).map(|qi| ids(&index, &query(qi))).collect();
            index.save_snapshot(&snap).unwrap();
        }
        assert_eq!(dir.path().join("sq8.scaled").exists(), scaled);
        set_sq8_scaled_codes(true);
        let reloaded = reload::<EuclideanMetric>(dir.path(), &snap);
        assert_eq!(reloaded.sq8_scaled(), scaled);
        let after: Vec<Vec<u32>> = (0..10).map(|qi| ids(&reloaded, &query(qi))).collect();
        assert_eq!(before, after, "scaled={scaled}");
    }
    set_sq8_scaled_codes(true);
}

#[test]
fn other_metrics_and_modes_keep_the_legacy_format() {
    let _g = lock();
    let dir = tempdir().unwrap();
    let poincare = HnswIndex::<hyperspace_core::PoincareMetric>::new(
        Arc::new(VectorStore::new(dir.path(), DIM + 4)),
        QuantizationMode::ScalarI8,
        Arc::new(GlobalConfig::default()),
        DIM,
    );
    assert!(!poincare.sq8_scaled());
    let dir = tempdir().unwrap();
    let none = HnswIndex::<EuclideanMetric>::new(
        Arc::new(VectorStore::new(dir.path(), DIM * 8 + 8)),
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        DIM,
    );
    assert!(!none.sq8_scaled());
}
