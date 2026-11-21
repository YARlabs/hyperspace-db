//! MRL (Matryoshka) search strategies, measured on MRL-shaped synthetic data.
//!
//! `cargo test --release -p hyperspace-index --test mrl_bench -- --ignored --nocapture`
//! (`MRL_N`, `MRL_DIM`, `MRL_CUT`, `MRL_DECAY`, `MRL_NOISE`, `MRL_MODE=medium|none` override the defaults).

use hyperspace_core::{CosineMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
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

fn normalize(v: &mut [f64]) {
    let n = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-12);
    v.iter_mut().for_each(|x| *x /= n);
}

/// MRL-shaped embedding: per-dimension scale `(i + 1)^-MRL_DECAY` (0.5), so the prefix carries most of
/// the energy, clustered (100 centres) like real embeddings; unit-normalized.
fn point(i: u64, dim: usize) -> Vec<f64> {
    let decay = std::env::var("MRL_DECAY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.5f64);
    let noise = std::env::var("MRL_NOISE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.55f64);
    let c = i % 100;
    let mut v: Vec<f64> = (0..dim as u64)
        .map(|d| {
            let w = ((d + 1) as f64).powf(-decay);
            w * (rnd(c * 7919 + d) + noise * rnd(i * 1_000_003 + d + 17))
        })
        .collect();
    normalize(&mut v);
    v
}

fn prefix(v: &[f64], cut: usize) -> Vec<f64> {
    let mut p = v[..cut].to_vec();
    normalize(&mut p);
    p
}

fn l2(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum()
}

fn build(
    mode: QuantizationMode,
    data: &[Vec<f64>],
    dim: usize,
    dir: &std::path::Path,
) -> HnswIndex<CosineMetric> {
    let elem = match mode {
        QuantizationMode::None => dim * 4 + 4,
        _ => dim + 4,
    };
    let cfg = GlobalConfig::default();
    cfg.m.store(16, Ordering::Relaxed);
    cfg.ef_construction.store(200, Ordering::Relaxed);
    let idx = HnswIndex::<CosineMetric>::new(
        Arc::new(VectorStore::new(dir, elem)),
        mode,
        Arc::new(cfg),
        dim,
    );
    for v in data {
        let id = idx.insert_to_storage(v).unwrap();
        idx.index_node(id, HashMap::new()).unwrap();
    }
    idx
}

#[test]
#[ignore = "benchmark"]
fn mrl_strategies() {
    let n = env("MRL_N", 30_000);
    let dim = env("MRL_DIM", 1024);
    let cut = env("MRL_CUT", 256);
    let mode = match std::env::var("MRL_MODE").as_deref() {
        Ok("none") => QuantizationMode::None,
        _ => QuantizationMode::ScalarI8,
    };
    let nq = 300u64;
    let k = 10;
    let data: Vec<Vec<f64>> = (0..n as u64).map(|i| point(i, dim)).collect();
    let queries: Vec<Vec<f64>> = (0..nq).map(|i| point(50_000_000 + i, dim)).collect();
    let truth: Vec<Vec<u32>> = queries
        .iter()
        .map(|q| {
            let mut d: Vec<(f64, u32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (l2(v, q), i as u32))
                .collect();
            d.select_nth_unstable_by(k, |a, b| a.0.total_cmp(&b.0));
            d.truncate(k);
            d.into_iter().map(|x| x.1).collect()
        })
        .collect();
    let recall = |res: &[Vec<u32>]| {
        res.iter()
            .zip(&truth)
            .map(|(r, t)| r.iter().filter(|x| t.contains(x)).count())
            .sum::<usize>() as f64
            / (k * nq as usize) as f64
    };
    println!("MRL bench: n={n} dim={dim} cut={cut} mode={mode:?} M=16 efc=200, {nq} queries, recall@{k} vs full cosine");

    let t = Instant::now();
    let dir_full = tempfile::tempdir().unwrap();
    let full = build(mode, &data, dim, dir_full.path());
    println!("build full {dim}-D: {:.1}s", t.elapsed().as_secs_f64());
    let pre: Vec<Vec<f64>> = data.iter().map(|v| prefix(v, cut)).collect();
    let t = Instant::now();
    let dir_cut = tempfile::tempdir().unwrap();
    let small = build(mode, &pre, cut, dir_cut.path());
    println!("build prefix {cut}-D: {:.1}s", t.elapsed().as_secs_f64());

    for ef in [64usize, 128, 256] {
        let run = |f: &dyn Fn(&Vec<f64>) -> Vec<u32>| {
            let t = Instant::now();
            let r: Vec<Vec<u32>> = queries.iter().map(f).collect();
            (recall(&r), t.elapsed().as_secs_f64() * 1e6 / nq as f64)
        };
        let p = |top: usize, mrl: Option<usize>| SearchParams {
            top_k: top,
            ef_search: ef,
            mrl_dimension: mrl,
            ..Default::default()
        };
        let (r1, t1) = run(&|q| {
            full.search(q, &HashMap::new(), &[], &p(k, None))
                .into_iter()
                .map(|x| x.0)
                .collect()
        });
        let (r2, t2) = run(&|q| {
            full.search(q, &HashMap::new(), &[], &p(k, Some(cut)))
                .into_iter()
                .map(|x| x.0)
                .collect()
        });
        // Funnel: prefix index, exact full-dimension rerank of a pool of `ef` candidates.
        let funnel = |pool: usize| {
            run(&|q| {
                let qp = prefix(q, cut);
                let cands = small.search(&qp, &HashMap::new(), &[], &p(pool, None));
                let mut sc: Vec<(f64, u32)> = cands
                    .into_iter()
                    .map(|(id, _)| (l2(&data[id as usize], q), id))
                    .collect();
                sc.sort_by(|a, b| a.0.total_cmp(&b.0));
                sc.into_iter().take(k).map(|x| x.1).collect()
            })
        };
        let (r3, t3) = funnel(4 * k);
        let (r4, t4) = funnel(ef.min(10 * k).max(4 * k));
        println!(
            "ef={ef:>3} | full {:.2}% {:>6.0}us | mrl_dimension {:.2}% {:>6.0}us | prefix+rerank x4 {:.2}% {:>6.0}us | prefix+rerank x10 {:.2}% {:>6.0}us",
            r1 * 100.0, t1, r2 * 100.0, t2, r3 * 100.0, t3, r4 * 100.0, t4
        );
    }
}
