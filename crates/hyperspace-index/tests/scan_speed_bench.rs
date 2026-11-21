//! In-process scan timing (`cargo test --release -p hyperspace-index --test scan_speed_bench
//! -- --ignored --nocapture`): where a 1024-D exact scan spends its time.

use hyperspace_core::{CosineMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::{set_flat_scan_limit, set_sq8_int_kernel, set_sq8_scaled_codes, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

fn vecs(n: usize, dim: usize) -> Vec<Vec<f64>> {
    (0..n)
        .map(|i| {
            let v: Vec<f64> = (0..dim)
                .map(|d| ((i * 7919 + d * 104_729) % 2001) as f64 / 1000.0 - 1.0)
                .collect();
            let s = v.iter().map(|x| x * x).sum::<f64>().sqrt();
            v.into_iter().map(|x| x / s).collect()
        })
        .collect()
}

#[test]
#[ignore = "timing benchmark"]
fn flat_scan_speed() {
    let dim = 1024;
    // (n, mode, element size, label, ScalarI8 scaled codes, ScalarI8 integer kernel)
    for &(n, mode, elem, label, scaled, int_kernel) in &[
        (
            20_000usize,
            QuantizationMode::None,
            dim * 4 + 4,
            "f32",
            true,
            true,
        ),
        (
            1_667,
            QuantizationMode::None,
            dim * 4 + 4,
            "f32",
            true,
            true,
        ),
        (
            20_000,
            QuantizationMode::ScalarI8,
            dim + 4,
            "sq8 legacy codes",
            false,
            false,
        ),
        (
            20_000,
            QuantizationMode::ScalarI8,
            dim + 4,
            "sq8 scaled, f32 kernel",
            true,
            false,
        ),
        (
            20_000,
            QuantizationMode::ScalarI8,
            dim + 4,
            "sq8 scaled, int kernel",
            true,
            true,
        ),
    ] {
        set_sq8_scaled_codes(scaled);
        set_sq8_int_kernel(int_kernel);
        let dir = tempfile::tempdir().unwrap();
        let index = HnswIndex::<CosineMetric>::new_with_storage_precision(
            Arc::new(VectorStore::new(dir.path(), elem)),
            mode,
            Arc::new(GlobalConfig::default()),
            mode == QuantizationMode::None,
            dim,
        );
        let data = vecs(n, dim);
        for v in &data {
            index.insert_to_storage(v).unwrap(); // flat scan needs no graph
        }
        set_flat_scan_limit(Some(usize::MAX >> 1));
        let q = vecs(1, dim).pop().unwrap();
        let p = SearchParams {
            top_k: 10,
            ef_search: 100,
            ..Default::default()
        };
        for _ in 0..3 {
            black_box(index.search(&q, &HashMap::new(), &[], &p));
        }
        let iters = 200;
        let t = Instant::now();
        for _ in 0..iters {
            black_box(index.search(&q, &HashMap::new(), &[], &p));
        }
        let per = t.elapsed().as_secs_f64() / f64::from(iters) * 1e3;
        // Lower bound: plain SIMD dot over a contiguous f32 copy (same bytes).
        let flat: Vec<f32> = data
            .iter()
            .flat_map(|v| v.iter().map(|&x| x as f32))
            .collect();
        let qf: Vec<f32> = q.iter().map(|&x| x as f32).collect();
        let t = Instant::now();
        for _ in 0..iters {
            let mut best = f32::MAX;
            for row in flat.chunks_exact(dim) {
                let d = hyperspace_core::simd::l2_sq_f32(row, &qf);
                best = best.min(d);
            }
            black_box(best);
        }
        let lb = t.elapsed().as_secs_f64() / f64::from(iters) * 1e3;
        let gbs = (n * dim * if mode == QuantizationMode::None { 4 } else { 1 }) as f64
            / (per * 1e-3)
            / 1e9;
        println!("{label:<24} n={n:>6}: index flat scan {per:7.3} ms/query ({gbs:5.1} GB/s) | plain f32 SIMD loop {lb:7.3} ms");
    }
    set_flat_scan_limit(None);
    set_sq8_scaled_codes(true);
    set_sq8_int_kernel(true);
}
