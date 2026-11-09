#![allow(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_range_loop)]

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::time::Instant;

struct TernaryVector {
    nz: Vec<u64>,
    sign: Vec<u64>,
    scale: f32,
}

struct BinaryVector {
    bits: Vec<u64>,
}

fn quantize_ternary(coords: &[f32], tau_factor: f32) -> TernaryVector {
    let dim = coords.len();
    let num_words = dim.div_ceil(64);
    let mut nz = vec![0u64; num_words];
    let mut sign = vec![0u64; num_words];

    let mean_abs: f32 = coords.iter().map(|&x| x.abs()).sum::<f32>() / (dim as f32);
    let tau = mean_abs * tau_factor;

    let mut dot_orig_ternary = 0.0f32;
    let mut ternary_norm_sq = 0.0f32;

    for (i, &x) in coords.iter().enumerate() {
        let word = i / 64;
        let bit = i % 64;
        if x > tau {
            nz[word] |= 1u64 << bit;
            sign[word] |= 1u64 << bit;
            dot_orig_ternary += x;
            ternary_norm_sq += 1.0;
        } else if x < -tau {
            nz[word] |= 1u64 << bit;
            dot_orig_ternary -= x;
            ternary_norm_sq += 1.0;
        }
    }

    let scale = if ternary_norm_sq > 0.0 {
        dot_orig_ternary / ternary_norm_sq
    } else {
        1.0
    };

    TernaryVector { nz, sign, scale }
}

fn quantize_binary(coords: &[f32]) -> BinaryVector {
    let dim = coords.len();
    let num_words = dim.div_ceil(64);
    let mut bits = vec![0u64; num_words];
    for (i, &x) in coords.iter().enumerate() {
        if x > 0.0 {
            bits[i / 64] |= 1u64 << (i % 64);
        }
    }
    BinaryVector { bits }
}

#[inline(always)]
fn ternary_dot(a: &TernaryVector, b: &TernaryVector) -> f32 {
    let mut raw_dot = 0i32;
    let n = a.nz.len();
    for i in 0..n {
        let nza = a.nz[i];
        let nzb = b.nz[i];
        let common = nza & nzb;
        let term1 = common.count_ones() as i32;
        let term2 = ((a.sign[i] ^ b.sign[i]) & common).count_ones() as i32;
        raw_dot += term1 - 2 * term2;
    }
    (raw_dot as f32) * a.scale * b.scale
}

#[inline(always)]
fn binary_dot(a: &BinaryVector, b: &BinaryVector, dim: usize) -> i32 {
    let mut matches = 0i32;
    for i in 0..a.bits.len() {
        matches += (!(a.bits[i] ^ b.bits[i])).count_ones() as i32;
    }
    2 * matches - (dim as i32)
}

#[inline(always)]
fn f32_dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

#[test]
fn test_ternary_vs_binary_and_f32_benchmark() {
    let dim = 1024;
    let n_dataset = 5000;
    let n_queries = 50;
    let mut rng = StdRng::seed_from_u64(42);

    println!("\n========================================================");
    println!("  TERNARY (1.58-bit) vs BINARY (1-bit) vs F32 BENCHMARK ");
    println!(
        "  Vector Dim: {}, Dataset: {} vectors, Queries: {}",
        dim, n_dataset, n_queries
    );
    println!("========================================================");

    // 1. Generate realistic synthetic normalized embeddings
    let mut raw_dataset = Vec::with_capacity(n_dataset);
    for _ in 0..n_dataset {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0f32)).collect();
        let norm: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
        for x in &mut v {
            *x /= norm;
        }
        raw_dataset.push(v);
    }

    let mut raw_queries = Vec::with_capacity(n_queries);
    for _ in 0..n_queries {
        let mut v: Vec<f32> = (0..dim).map(|_| rng.gen_range(-1.0f32..1.0f32)).collect();
        let norm: f32 = v.iter().map(|&x| x * x).sum::<f32>().sqrt();
        for x in &mut v {
            *x /= norm;
        }
        raw_queries.push(v);
    }

    // 2. Quantize dataset
    let tau_factor = 0.67f32; // Optimal Lloyd-Max threshold for standard Gaussian
    let ternary_dataset: Vec<TernaryVector> = raw_dataset
        .iter()
        .map(|v| quantize_ternary(v, tau_factor))
        .collect();
    let binary_dataset: Vec<BinaryVector> =
        raw_dataset.iter().map(|v| quantize_binary(v)).collect();

    let ternary_queries: Vec<TernaryVector> = raw_queries
        .iter()
        .map(|v| quantize_ternary(v, tau_factor))
        .collect();
    let binary_queries: Vec<BinaryVector> =
        raw_queries.iter().map(|v| quantize_binary(v)).collect();

    // 3. Ground Truth Top-10 calculation (f32 exact)
    let k = 10;
    let mut ground_truth: Vec<Vec<usize>> = Vec::with_capacity(n_queries);
    for q in &raw_queries {
        let mut scores: Vec<(usize, f32)> = raw_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, f32_dot(q, doc)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.truncate(k);
        ground_truth.push(scores.into_iter().map(|(idx, _)| idx).collect());
    }

    // 4. Recall Evaluation Helper
    let evaluate_recall = |predicted: &[Vec<usize>]| -> f32 {
        let mut total_hits = 0;
        for (q_idx, preds) in predicted.iter().enumerate() {
            let gt = &ground_truth[q_idx];
            for &p in preds {
                if gt.contains(&p) {
                    total_hits += 1;
                }
            }
        }
        (total_hits as f32) / ((n_queries * k) as f32) * 100.0
    };

    // A. Pure Binary (no rerank)
    let mut binary_preds_pure = Vec::with_capacity(n_queries);
    for q in &binary_queries {
        let mut scores: Vec<(usize, i32)> = binary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, binary_dot(q, doc, dim)))
            .collect();
        scores.sort_by_key(|b| -b.1);
        scores.truncate(k);
        binary_preds_pure.push(scores.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_bin_pure = evaluate_recall(&binary_preds_pure);

    // B. Pure Ternary (no rerank)
    let mut ternary_preds_pure = Vec::with_capacity(n_queries);
    for q in &ternary_queries {
        let mut scores: Vec<(usize, f32)> = ternary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, ternary_dot(q, doc)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.truncate(k);
        ternary_preds_pure.push(scores.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_ternary_pure = evaluate_recall(&ternary_preds_pure);

    // C. Binary + Rerank Top-50 (oversample 5x)
    let mut binary_preds_rerank50 = Vec::with_capacity(n_queries);
    for (q_idx, q) in binary_queries.iter().enumerate() {
        let mut scores: Vec<(usize, i32)> = binary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, binary_dot(q, doc, dim)))
            .collect();
        scores.sort_by_key(|b| -b.1);
        scores.truncate(50);
        let raw_q = &raw_queries[q_idx];
        let mut reranked: Vec<(usize, f32)> = scores
            .into_iter()
            .map(|(idx, _)| (idx, f32_dot(raw_q, &raw_dataset[idx])))
            .collect();
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        reranked.truncate(k);
        binary_preds_rerank50.push(reranked.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_bin_rerank50 = evaluate_recall(&binary_preds_rerank50);

    // D. Ternary + Rerank Top-50 (oversample 5x)
    let mut ternary_preds_rerank50 = Vec::with_capacity(n_queries);
    for (q_idx, q) in ternary_queries.iter().enumerate() {
        let mut scores: Vec<(usize, f32)> = ternary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, ternary_dot(q, doc)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.truncate(50);
        let raw_q = &raw_queries[q_idx];
        let mut reranked: Vec<(usize, f32)> = scores
            .into_iter()
            .map(|(idx, _)| (idx, f32_dot(raw_q, &raw_dataset[idx])))
            .collect();
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        reranked.truncate(k);
        ternary_preds_rerank50.push(reranked.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_ternary_rerank50 = evaluate_recall(&ternary_preds_rerank50);

    // E. Binary + Rerank Top-100 (oversample 10x)
    let mut binary_preds_rerank100 = Vec::with_capacity(n_queries);
    for (q_idx, q) in binary_queries.iter().enumerate() {
        let mut scores: Vec<(usize, i32)> = binary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, binary_dot(q, doc, dim)))
            .collect();
        scores.sort_by_key(|b| -b.1);
        scores.truncate(100);
        let raw_q = &raw_queries[q_idx];
        let mut reranked: Vec<(usize, f32)> = scores
            .into_iter()
            .map(|(idx, _)| (idx, f32_dot(raw_q, &raw_dataset[idx])))
            .collect();
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        reranked.truncate(k);
        binary_preds_rerank100.push(reranked.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_bin_rerank100 = evaluate_recall(&binary_preds_rerank100);

    // F. Ternary + Rerank Top-100 (oversample 10x)
    let mut ternary_preds_rerank100 = Vec::with_capacity(n_queries);
    for (q_idx, q) in ternary_queries.iter().enumerate() {
        let mut scores: Vec<(usize, f32)> = ternary_dataset
            .iter()
            .enumerate()
            .map(|(idx, doc)| (idx, ternary_dot(q, doc)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores.truncate(100);
        let raw_q = &raw_queries[q_idx];
        let mut reranked: Vec<(usize, f32)> = scores
            .into_iter()
            .map(|(idx, _)| (idx, f32_dot(raw_q, &raw_dataset[idx])))
            .collect();
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        reranked.truncate(k);
        ternary_preds_rerank100.push(reranked.into_iter().map(|(idx, _)| idx).collect());
    }
    let recall_ternary_rerank100 = evaluate_recall(&ternary_preds_rerank100);

    // 5. Throughput Benchmarking (100,000 vector dot products)
    let iters = 100_000;
    let t_vec_a = &ternary_dataset[0];
    let t_vec_b = &ternary_dataset[1];
    let b_vec_a = &binary_dataset[0];
    let b_vec_b = &binary_dataset[1];
    let f_vec_a = &raw_dataset[0];
    let f_vec_b = &raw_dataset[1];

    let start = Instant::now();
    let mut blackbox_t = 0.0f32;
    for _ in 0..iters {
        blackbox_t += ternary_dot(t_vec_a, t_vec_b);
    }
    let duration_ternary = start.elapsed();
    let ns_ternary = (duration_ternary.as_nanos() as f64) / (iters as f64);
    let m_ops_ternary = (iters as f64) / duration_ternary.as_secs_f64() / 1_000_000.0;

    let start = Instant::now();
    let mut blackbox_b = 0i32;
    for _ in 0..iters {
        blackbox_b += binary_dot(b_vec_a, b_vec_b, dim);
    }
    let duration_binary = start.elapsed();
    let ns_binary = (duration_binary.as_nanos() as f64) / (iters as f64);
    let m_ops_binary = (iters as f64) / duration_binary.as_secs_f64() / 1_000_000.0;

    let start = Instant::now();
    let mut blackbox_f = 0.0f32;
    for _ in 0..iters {
        blackbox_f += f32_dot(f_vec_a, f_vec_b);
    }
    let duration_f32 = start.elapsed();
    let ns_f32 = (duration_f32.as_nanos() as f64) / (iters as f64);
    let m_ops_f32 = (iters as f64) / duration_f32.as_secs_f64() / 1_000_000.0;

    println!("\n--- ACCURACY (Recall@10 against Ground Truth f32) ---");
    println!(
        "  1-bit Binary (Pure):         {:>5.1}% (size: 128 B / vector)",
        recall_bin_pure
    );
    println!(
        "  1.58-bit Ternary (Pure):      {:>5.1}% (size: 256 B / vector)",
        recall_ternary_pure
    );
    println!(
        "  1-bit Binary + Rerank 5x:    {:>5.1}%",
        recall_bin_rerank50
    );
    println!(
        "  1.58-bit Ternary + Rerank 5x: {:>5.1}%",
        recall_ternary_rerank50
    );
    println!(
        "  1-bit Binary + Rerank 10x:   {:>5.1}%",
        recall_bin_rerank100
    );
    println!(
        "  1.58-bit Ternary + Rerank 10x:{:>5.1}%",
        recall_ternary_rerank100
    );

    println!("\n--- COMPUTATIONAL SPEED (1024D dot product) ---");
    println!(
        "  f32 Dot Product:    {:>6.1} ns/op | {:>6.2} M vectors/sec",
        ns_f32, m_ops_f32
    );
    println!(
        "  1.58-bit Ternary:   {:>6.1} ns/op | {:>6.2} M vectors/sec",
        ns_ternary, m_ops_ternary
    );
    println!(
        "  1-bit Binary:       {:>6.1} ns/op | {:>6.2} M vectors/sec",
        ns_binary, m_ops_binary
    );
    println!("--------------------------------------------------------");

    // Sanity checks:
    assert!(blackbox_t != 0.0);
    assert!(blackbox_b != 0);
    assert!(blackbox_f != 0.0);
    assert!(
        recall_ternary_rerank100 > recall_bin_rerank100,
        "Ternary must outperform Binary in recall"
    );
}
