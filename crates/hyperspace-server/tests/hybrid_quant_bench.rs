//! Hybrid (33-D Lorentz head + Euclidean tail) quantization: recall, bytes and latency of
//! every level the server can pick for a `hybrid` collection.
//!
//! `cargo test --release -p hyperspace-server --test hybrid_quant_bench -- --ignored --nocapture`
//! (`HYB_N`, `HYB_TAIL` override the defaults).

use hyperspace_core::{GlobalConfig, HybridMetric, Metric, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_server::chunk_searcher::{storage_element_size, storage_f32_enabled};
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

fn env(k: &str, d: usize) -> usize {
    std::env::var(k)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(d)
}

fn rnd(mut x: u64) -> f64 {
    x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 31;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 29;
    (x % 2_000_001) as f64 / 1_000_000.0 - 1.0
}

/// Hybrid point: Lorentz head on H^32 (spatial part clustered, x0 = sqrt(1 + |x|^2)) and a
/// clustered unit-norm Euclidean tail.
pub fn hybrid_point(i: u64, tail: usize) -> Vec<f64> {
    let c = i % 60;
    let spatial: Vec<f64> = (0..32u64)
        .map(|d| 0.8 * rnd(c * 131 + d) + 0.25 * rnd(i * 7_000_003 + d + 5))
        .collect();
    let x0 = (1.0 + spatial.iter().map(|x| x * x).sum::<f64>()).sqrt();
    let mut t: Vec<f64> = (0..tail as u64)
        .map(|d| rnd(c * 7919 + d + 1_000) + 0.6 * rnd(i * 1_000_003 + d + 17))
        .collect();
    let n = t.iter().map(|x| x * x).sum::<f64>().sqrt();
    t.iter_mut().for_each(|x| *x /= n);
    let mut v = Vec::with_capacity(33 + tail);
    v.push(x0);
    v.extend(spatial);
    v.extend(t);
    v
}

#[test]
#[ignore = "benchmark"]
fn hybrid_tail_quantization() {
    let n = env("HYB_N", 20_000);
    let tail = env("HYB_TAIL", 768);
    let dim = 33 + tail;
    let nq = 200u64;
    let k = 10;
    let data: Vec<Vec<f64>> = (0..n as u64).map(|i| hybrid_point(i, tail)).collect();
    let qs: Vec<Vec<f64>> = (0..nq)
        .map(|i| hybrid_point(90_000_000 + i, tail))
        .collect();
    let truth: Vec<Vec<u32>> = qs
        .iter()
        .map(|q| {
            let mut d: Vec<(f64, u32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (HybridMetric::distance(v, q), i as u32))
                .collect();
            d.select_nth_unstable_by(k, |a, b| a.0.total_cmp(&b.0));
            d.truncate(k);
            d.into_iter().map(|x| x.1).collect()
        })
        .collect();
    println!("hybrid bench: n={n} dim={dim} (33 + {tail}), M=16 efc=200 ef=128, recall@{k} vs exact hybrid distance");
    let modes = [
        QuantizationMode::None,
        QuantizationMode::AsymmetricHybrid801,
        QuantizationMode::ScalarI8,
        QuantizationMode::AsymmetricHybridLowBit,
        QuantizationMode::ScalarI4,
        QuantizationMode::Turbo,
        QuantizationMode::Ternary,
        QuantizationMode::AsymmetricHybridExtreme,
        QuantizationMode::Binary,
        QuantizationMode::RaBitQ4,
        QuantizationMode::RaBitQ2,
        QuantizationMode::RaBitQ,
        QuantizationMode::PQ4,
        QuantizationMode::ProductQuantization,
        QuantizationMode::OPQ,
    ];
    let only = std::env::var("HYB_MODES").ok();
    for mode in modes {
        if let Some(o) = &only {
            if !o.split(',').any(|m| m.trim() == format!("{mode:?}")) {
                continue;
            }
        }
        let f32s = storage_f32_enabled(mode);
        let elem = storage_element_size::<HybridMetric>(mode, dim, f32s);
        let dir = tempfile::tempdir().unwrap();
        let cfg = GlobalConfig::default();
        cfg.m.store(16, Ordering::Relaxed);
        cfg.ef_construction.store(200, Ordering::Relaxed);
        let mut idx = HnswIndex::<HybridMetric>::new_with_storage_precision(
            Arc::new(VectorStore::new(dir.path(), elem)),
            mode,
            Arc::new(cfg),
            f32s,
            dim,
        );
        if mode.needs_training() {
            let s: Vec<Vec<f32>> = data
                .iter()
                .step_by(4)
                .map(|v| v.iter().map(|&x| x as f32).collect())
                .collect();
            let r: Vec<&[f32]> = s.iter().map(Vec::as_slice).collect();
            idx.train_pq(&r, 10);
        }
        let t = Instant::now();
        for v in &data {
            let id = idx.insert_to_storage(v).unwrap();
            idx.index_node(id, HashMap::new()).unwrap();
        }
        let build = t.elapsed().as_secs_f64();
        let p = |top: usize| SearchParams {
            top_k: top,
            ef_search: 128,
            ..Default::default()
        };
        let t = Instant::now();
        let mut hit1 = 0;
        let mut hit4 = 0;
        for (q, tr) in qs.iter().zip(&truth) {
            let r = idx.search(q, &HashMap::new(), &[], &p(k));
            hit1 += r.iter().filter(|x| tr.contains(&x.0)).count();
            // Rerank x4 on exact vectors (what the server does with its exact store).
            let mut pool: Vec<(f64, u32)> = idx
                .search(q, &HashMap::new(), &[], &p(4 * k))
                .into_iter()
                .map(|(id, _)| (HybridMetric::distance(&data[id as usize], q), id))
                .collect();
            pool.sort_by(|a, b| a.0.total_cmp(&b.0));
            hit4 += pool.iter().take(k).filter(|x| tr.contains(&x.1)).count();
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / (2 * nq) as f64;
        let tot = (k * nq as usize) as f64;
        println!(
            "{:<24} {:>5} B  build {:>5.1}s  recall {:>6.2}%  rerank x4 {:>6.2}%  {:>6.0} us/q",
            format!("{mode:?}"),
            elem,
            build,
            hit1 as f64 * 100.0 / tot,
            hit4 as f64 * 100.0 / tot,
            us
        );
    }
}

/// Every hybrid code (exact Lorentz head + coded Euclidean tail) ranks by the true hybrid
/// distance, at any tail width, decodes to the original vector and survives a reload.
#[test]
fn hybrid_tail_codes_rank_by_the_hybrid_distance() {
    let tail = 96usize;
    let dim = 33 + tail;
    let n = 2000u64;
    let data: Vec<Vec<f64>> = (0..n).map(|i| hybrid_point(i, tail)).collect();
    let qs: Vec<Vec<f64>> = (0..30u64)
        .map(|i| hybrid_point(70_000_000 + i, tail))
        .collect();
    let truth: Vec<Vec<u32>> = qs
        .iter()
        .map(|q| {
            let mut d: Vec<(f64, u32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (HybridMetric::distance(v, q), i as u32))
                .collect();
            d.sort_by(|a, b| a.0.total_cmp(&b.0));
            d.into_iter().take(10).map(|x| x.1).collect()
        })
        .collect();
    for (mode, min_single, max_err) in [
        (QuantizationMode::AsymmetricHybrid801, 0.95, 0.02),
        (QuantizationMode::RaBitQ4, 0.85, 0.15),
        (QuantizationMode::RaBitQ2, 0.70, 0.35),
        (QuantizationMode::RaBitQ, 0.50, 0.75),
        (QuantizationMode::PQ4, 0.60, 0.75),
    ] {
        let f32s = storage_f32_enabled(mode);
        let elem = storage_element_size::<HybridMetric>(mode, dim, f32s);
        let dir = tempfile::tempdir().unwrap();
        let mut idx = HnswIndex::<HybridMetric>::new_with_storage_precision(
            Arc::new(VectorStore::new(dir.path(), elem)),
            mode,
            Arc::new(GlobalConfig::default()),
            f32s,
            dim,
        );
        if mode.needs_training() {
            let s: Vec<Vec<f32>> = data
                .iter()
                .map(|v| v.iter().map(|&x| x as f32).collect())
                .collect();
            let r: Vec<&[f32]> = s.iter().map(Vec::as_slice).collect();
            idx.train_pq(&r, 8);
        }
        for v in &data {
            let id = idx.insert_to_storage(v).unwrap();
            idx.index_node(id, HashMap::new()).unwrap();
        }
        let p = |k: usize| SearchParams {
            top_k: k,
            ef_search: 128,
            ..Default::default()
        };
        let run = |idx: &HnswIndex<HybridMetric>| -> (f64, f64) {
            let (mut single, mut rr) = (0usize, 0usize);
            for (q, t) in qs.iter().zip(&truth) {
                let r = idx.search(q, &HashMap::new(), &[], &p(10));
                single += r.iter().filter(|x| t.contains(&x.0)).count();
                let mut pool: Vec<(f64, u32)> = idx
                    .search(q, &HashMap::new(), &[], &p(40))
                    .into_iter()
                    .map(|(id, _)| (HybridMetric::distance(&data[id as usize], q), id))
                    .collect();
                pool.sort_by(|a, b| a.0.total_cmp(&b.0));
                rr += pool.iter().take(10).filter(|x| t.contains(&x.1)).count();
            }
            (single as f64 / 300.0, rr as f64 / 300.0)
        };
        let (single, reranked) = run(&idx);
        assert!(
            single >= min_single,
            "{mode:?}: single-pass recall {single:.3}"
        );
        assert!(
            reranked >= 0.98,
            "{mode:?}: recall with rerank {reranked:.3}"
        );
        // The head is exact and the tail close: decode matches the stored vector.
        let v = idx.get_vector(7).coords;
        assert_eq!(v.len(), dim);
        let head_err = v[..33]
            .iter()
            .zip(&data[7][..33])
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        assert!(head_err < 1e-4, "{mode:?}: head error {head_err}");
        let tail_err: f64 = v[33..]
            .iter()
            .zip(&data[7][33..])
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();
        assert!(tail_err < max_err, "{mode:?}: tail error {tail_err}");
        // Snapshot reload keeps the layout and the quantizer.
        let snap = dir.path().join("index.snap");
        idx.save_snapshot(&snap).unwrap();
        let before: Vec<Vec<u32>> = qs
            .iter()
            .take(5)
            .map(|q| {
                idx.search(q, &HashMap::new(), &[], &p(10))
                    .into_iter()
                    .map(|x| x.0)
                    .collect()
            })
            .collect();
        drop(idx);
        let re = HnswIndex::<HybridMetric>::load_snapshot_with_storage_precision(
            &snap,
            Arc::new(VectorStore::new(dir.path(), elem)),
            mode,
            Arc::new(GlobalConfig::default()),
            f32s,
            dim,
        )
        .unwrap();
        let after: Vec<Vec<u32>> = qs
            .iter()
            .take(5)
            .map(|q| {
                re.search(q, &HashMap::new(), &[], &p(10))
                    .into_iter()
                    .map(|x| x.0)
                    .collect()
            })
            .collect();
        assert_eq!(before, after, "{mode:?}: reload");
    }
}

/// Pre-4.2.5 hybrid SQ8 codes (fixed step 1/127, alpha = 1) decode as before.
#[test]
fn legacy_hybrid_sq8_codes_still_decode() {
    use hyperspace_core::hybrid::HybridQuantizedVector;
    let v = hybrid_point(3, 64);
    let mut legacy = Vec::new();
    for &x in &v[..33] {
        legacy.extend_from_slice(&(x as f32).to_le_bytes());
    }
    for &x in &v[33..] {
        legacy.push(((x * 127.0).clamp(-127.0, 127.0) as i8).cast_unsigned());
    }
    legacy.extend_from_slice(&1.0f32.to_le_bytes());
    let q = HybridQuantizedVector::from_bytes(&legacy);
    assert!((q.tail_step() - 1.0 / 127.0).abs() < 1e-12);
    let hv = hyperspace_core::HyperVector::new_unchecked(v.clone());
    let d_legacy = HybridQuantizedVector::distance_bytes_mrl(&legacy, &hv, None);
    let fresh = HybridQuantizedVector::from_float(&hv, 33, 64).as_bytes();
    let d_fresh = HybridQuantizedVector::distance_bytes_mrl(&fresh, &hv, None);
    assert!(
        d_fresh < d_legacy,
        "per-vector scale is closer: {d_fresh} vs {d_legacy}"
    );
    assert!(d_legacy < 0.2 && d_fresh < 0.02, "{d_legacy} {d_fresh}");
}
