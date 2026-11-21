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

/// Exact-pruning layouts for a full-precision flat scan (in-process, single thread):
/// full rows, early abandon inside rows (current), and a separate prefix block whose
/// partial distance (a lower bound of the L2 distance) lets the scan skip the row tail.
///
/// Measured (4.2.5, 20k × 1024-D clustered, k = 40): A 1.17 ms, B 1.05 ms, C 1.23 ms with
/// 71% of rows still reading the tail. In natural coordinates the first quarter of the
/// dimensions carries a quarter of the distance, so a prefix bound prunes little; the
/// layout would only pay after a variance-concentrating rotation (PCA). Kept as a
/// regression benchmark for that experiment.
#[test]
#[ignore = "timing benchmark"]
fn prefix_block_pruning() {
    use hyperspace_core::simd::{l2_sq_f32, l2_sq_f32_bytes_adsampling, EXACT_EARLY_ABANDON};
    let (dim, n, k, prefix) = (1024usize, 20_000usize, 40usize, 256usize);
    // Clustered unit vectors (50 centroids), like real embeddings.
    let mut seed = 42u64;
    let mut rnd = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        ((seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5
    };
    let cents: Vec<Vec<f32>> = (0..50).map(|_| (0..dim).map(|_| rnd()).collect()).collect();
    let mk = |c: &Vec<f32>, r: &mut dyn FnMut() -> f32| {
        let mut v: Vec<f32> = c.iter().map(|&x| x + 0.6 * r()).collect();
        let s = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= s);
        v
    };
    let rows: Vec<Vec<f32>> = (0..n).map(|i| mk(&cents[i % 50], &mut rnd)).collect();
    let queries: Vec<Vec<f32>> = (0..50)
        .map(|i| mk(&cents[(i * 7) % 50], &mut rnd))
        .collect();
    let flat: Vec<f32> = rows.iter().flatten().copied().collect();
    let bytes: Vec<u8> = flat.iter().flat_map(|x| x.to_le_bytes()).collect();
    let pre: Vec<f32> = rows
        .iter()
        .flat_map(|r| r[..prefix].iter().copied())
        .collect();
    let suf: Vec<f32> = rows
        .iter()
        .flat_map(|r| r[prefix..].iter().copied())
        .collect();
    type Scan<'a> = &'a dyn Fn(&[f32]) -> (Vec<f32>, usize);
    let run = |name: &str, f: Scan<'_>| {
        let t = Instant::now();
        let mut tails = 0;
        let mut out = Vec::new();
        for q in &queries {
            let (r, tl) = f(q);
            tails += tl;
            out.push(r);
        }
        let ms = t.elapsed().as_secs_f64() * 1e3 / queries.len() as f64;
        println!(
            "{name:<32} {ms:7.3} ms/query, rows reading the tail: {:5.1}%",
            100.0 * tails as f64 / (n * queries.len()) as f64
        );
        out
    };
    let insert = |h: &mut Vec<f32>, d: f32| {
        if h.len() < k {
            h.push(d);
            h.sort_by(f32::total_cmp);
        } else if d < h[k - 1] {
            h[k - 1] = d;
            h.sort_by(f32::total_cmp);
        }
    };
    let a = run("A full rows", &|q| {
        let mut h = Vec::new();
        for r in flat.chunks_exact(dim) {
            insert(&mut h, l2_sq_f32(r, q));
        }
        (h, n)
    });
    let b = run("B early abandon in rows", &|q| {
        let mut h: Vec<f32> = Vec::new();
        for r in bytes.chunks_exact(dim * 4) {
            let t = (h.len() == k).then(|| h[k - 1]);
            insert(
                &mut h,
                l2_sq_f32_bytes_adsampling(r, q, t, EXACT_EARLY_ABANDON),
            );
        }
        (h, n)
    });
    let c = run("C prefix block + tail on demand", &|q| {
        let mut h: Vec<f32> = Vec::new();
        let mut tails = 0;
        for (p, s) in pre.chunks_exact(prefix).zip(suf.chunks_exact(dim - prefix)) {
            let dp = l2_sq_f32(p, &q[..prefix]);
            if h.len() == k && dp >= h[k - 1] {
                continue;
            }
            tails += 1;
            insert(&mut h, dp + l2_sq_f32(s, &q[prefix..]));
        }
        (h, tails)
    });
    for ((x, y), z) in a.iter().zip(&b).zip(&c) {
        let close = |u: &Vec<f32>, v: &Vec<f32>| u.iter().zip(v).all(|(p, q)| (p - q).abs() < 1e-3);
        assert!(close(x, y) && close(x, z), "pruning is exact");
    }
}
