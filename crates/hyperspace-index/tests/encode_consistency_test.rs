//! Inserts and updates share one storage encoder (since 4.2.5): a vector written by
//! `update_storage` decodes exactly like the same vector written by `insert_to_storage`.
//! Updates used to build the vector with `alpha = 1 / (1 − ‖x‖²)` for every metric (∞ for
//! unit-norm cosine vectors) while inserts used 1.

use hyperspace_core::{CosineMetric, EuclideanMetric, GlobalConfig, Metric, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::sync::Arc;

const DIM: usize = 16;

fn unit(i: u32) -> Vec<f64> {
    let v: Vec<f64> = (0..DIM)
        .map(|d| ((u64::from(i) * 7919 + d as u64 * 104_729) % 2001) as f64 / 1000.0 - 1.0)
        .collect();
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    v.into_iter().map(|x| x / n).collect()
}

fn check<M: Metric>(mode: QuantizationMode, elem: usize, storage_f32: bool) {
    let dir = tempfile::tempdir().unwrap();
    let idx = HnswIndex::<M>::new_with_storage_precision(
        Arc::new(VectorStore::new(dir.path(), elem)),
        mode,
        Arc::new(GlobalConfig::default()),
        storage_f32,
        DIM,
    );
    let a = idx.insert_to_storage(&unit(1)).unwrap();
    let b = idx.insert_to_storage(&unit(2)).unwrap();
    idx.update_storage(b, &unit(1)).unwrap();
    let (va, vb) = (idx.get_vector(a), idx.get_vector(b));
    assert_eq!(va.coords, vb.coords, "{mode:?} coords");
    assert_eq!(
        va.alpha.to_bits(),
        vb.alpha.to_bits(),
        "{mode:?} alpha {} vs {}",
        va.alpha,
        vb.alpha
    );
    assert!(vb.alpha.is_finite(), "{mode:?}: alpha {}", vb.alpha);
}

#[test]
fn update_encodes_like_insert() {
    check::<CosineMetric>(QuantizationMode::None, DIM * 4 + 4, true);
    check::<EuclideanMetric>(QuantizationMode::None, DIM * 8 + 8, false);
    check::<CosineMetric>(QuantizationMode::ScalarI8, DIM + 4, false);
    check::<EuclideanMetric>(QuantizationMode::Binary, DIM.div_ceil(8) + 4, false);
}
