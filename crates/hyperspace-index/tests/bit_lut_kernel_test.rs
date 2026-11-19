//! The per-query prepared kernels (`extreme` / `ternary` nibble tables, `medium_plus`
//! de-interleaved ScalarI4) must produce the results of the reference kernels.

use hyperspace_core::{
    CosineMetric, EuclideanMetric, GlobalConfig, Metric, QuantizationMode, SearchParams,
};
use hyperspace_index::{set_bit_lut_kernels, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

const DIM: usize = 100; // not a multiple of 8: exercises partial bytes / nibbles

fn point(i: u32) -> Vec<f64> {
    let v: Vec<f64> = (0..DIM)
        .map(|d| {
            let mut x = (u64::from(i) << 16 | d as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            x ^= x >> 29;
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 32;
            (x % 2000) as f64 / 1000.0 - 1.0 + f64::from(i % 3) * 0.3
        })
        .collect();
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    v.into_iter().map(|x| x / n).collect()
}

/// Builds the index and runs the queries with the given kernel setting (both the graph
/// construction and the search use it).
fn run<M: Metric>(mode: QuantizationMode, lut: bool) -> Vec<Vec<(u32, f64)>> {
    set_bit_lut_kernels(lut);
    let element = match mode {
        QuantizationMode::Binary => DIM.div_ceil(8) + 4,
        QuantizationMode::ScalarI4 => 4 + DIM.div_ceil(16) * 12,
        _ => 2 * DIM.div_ceil(8) + 8,
    };
    let dir = tempdir().unwrap();
    let index = HnswIndex::<M>::new(
        Arc::new(VectorStore::new(dir.path(), element)),
        mode,
        Arc::new(GlobalConfig::default()),
        DIM,
    );
    let n = 600u32;
    for i in 0..n {
        let id = index.insert_to_storage(&point(i)).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    let out = (0..20)
        .map(|qi| {
            index.search(
                &point(n + qi),
                &HashMap::new(),
                &[],
                &SearchParams {
                    top_k: 10,
                    ef_search: 64,
                    ..Default::default()
                },
            )
        })
        .collect();
    set_bit_lut_kernels(true);
    out
}

fn assert_equivalent<M: Metric>(mode: QuantizationMode) {
    let reference = run::<M>(mode, false);
    let fast = run::<M>(mode, true);
    let (mut same, mut total) = (0usize, 0usize);
    for (r, f) in reference.iter().zip(&fast) {
        assert_eq!(r.len(), f.len());
        for (id, d) in f {
            total += 1;
            if let Some((_, rd)) = r.iter().find(|(rid, _)| rid == id) {
                same += 1;
                assert!(
                    (rd - d).abs() <= 1e-4 * (1.0 + rd.abs()),
                    "{id}: {rd} vs {d}"
                );
            }
        }
    }
    // f32 table sums can flip exact ties in graph construction; results must still agree.
    let overlap = same as f64 / total as f64;
    assert!(
        overlap >= 0.97,
        "{mode:?}/{}: overlap {overlap:.3}",
        M::name()
    );
}

// One test function: the kernel switch is process-wide.
#[test]
fn bit_lut_kernels_match_reference_kernels() {
    assert_equivalent::<CosineMetric>(QuantizationMode::Binary);
    assert_equivalent::<EuclideanMetric>(QuantizationMode::Binary);
    assert_equivalent::<EuclideanMetric>(QuantizationMode::Ternary);
    assert_equivalent::<CosineMetric>(QuantizationMode::Ternary);
    assert_equivalent::<EuclideanMetric>(QuantizationMode::ScalarI4);
    assert_equivalent::<CosineMetric>(QuantizationMode::ScalarI4);
}
