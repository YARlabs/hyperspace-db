#![allow(unknown_lints)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use hyperspace_core::vector::{HyperVector, QuantizedHyperVector};
use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_sq8_f32_euclidean_matches_f64() {
    for dim in [16, 32, 64, 128, 384, 768] {
        let mut coords = Vec::with_capacity(dim);
        let mut bytes = Vec::with_capacity(dim + 4);
        for i in 0..dim {
            let val = ((i as f64 * 0.17).sin()) * 0.9;
            coords.push(val);
            let b = ((val * 127.0).round().clamp(-127.0, 127.0) as i8) as u8;
            bytes.push(b);
        }
        let alpha = 1.25f32;
        bytes.extend_from_slice(&alpha.to_le_bytes());

        let q_f64 = HyperVector::new_unchecked(coords);
        let q_f32: Vec<f32> = q_f64.coords.iter().map(|&x| x as f32).collect();

        let dist_f64 = QuantizedHyperVector::distance_bytes_euclidean_sq(&bytes, &q_f64);
        let dist_f32 = QuantizedHyperVector::distance_bytes_euclidean_sq_f32(&bytes, &q_f32);

        let diff = (dist_f64 - dist_f32).abs();
        let tol = dist_f64 * 1e-4 + 1e-5;
        assert!(
            diff <= tol,
            "Euclidean mismatch at dim={dim}: f64={dist_f64}, f32={dist_f32}, diff={diff}"
        );
    }
}

#[test]
fn test_sq8_f32_poincare_matches_f64() {
    for dim in [16, 32, 64, 128, 384, 768] {
        let mut coords = Vec::with_capacity(dim);
        let mut bytes = Vec::with_capacity(dim + 4);
        let norm_scale = 0.5 / (dim as f64).sqrt();
        for i in 0..dim {
            let val = ((i as f64 * 0.23).cos()) * norm_scale;
            coords.push(val);
            let b = ((val * 127.0).round().clamp(-127.0, 127.0) as i8) as u8;
            bytes.push(b);
        }
        let alpha = 1.33f32;
        bytes.extend_from_slice(&alpha.to_le_bytes());

        let mut q_f64 = HyperVector::new_unchecked(coords);
        let sq_norm: f64 = q_f64.coords.iter().map(|&x| x * x).sum();
        q_f64.alpha = 1.0 / (1.0 - sq_norm);
        let q_f32: Vec<f32> = q_f64.coords.iter().map(|&x| x as f32).collect();

        let dist_f64 = QuantizedHyperVector::distance_bytes_poincare_sq(&bytes, &q_f64);
        let dist_f32 =
            QuantizedHyperVector::distance_bytes_poincare_sq_f32(&bytes, &q_f32, q_f64.alpha);

        let diff = (dist_f64 - dist_f32).abs();
        let tol = dist_f64 * 1e-4 + 1e-5;
        assert!(
            diff <= tol,
            "Poincare mismatch at dim={dim}: f64={dist_f64}, f32={dist_f32}, diff={diff}"
        );
    }
}

#[test]
fn test_sq8_f32_lorentz_matches_f64() {
    for dim in [16, 32, 64, 128, 384, 768] {
        let mut coords = vec![0.0; dim];
        let mut spatial_sq = 0.0;
        let scale = 0.4 / (dim as f64).sqrt();
        for i in 1..dim {
            let s = ((i as f64 * 0.31).sin()) * scale;
            coords[i] = s;
            spatial_sq += s * s;
        }
        coords[0] = (1.0 + spatial_sq).sqrt();

        let mut bytes = Vec::with_capacity(dim + 4);
        let alpha = 1.15f32;
        for i in 0..dim {
            let b = ((coords[i] * 127.0 / (alpha as f64))
                .round()
                .clamp(-127.0, 127.0) as i8) as u8;
            bytes.push(b);
        }
        bytes.extend_from_slice(&alpha.to_le_bytes());

        let q_f64 = HyperVector::new_unchecked(coords);
        let q_f32: Vec<f32> = q_f64.coords.iter().map(|&x| x as f32).collect();

        let dist_f64 = QuantizedHyperVector::distance_bytes_lorentz(&bytes, &q_f64);
        let dist_f32 = QuantizedHyperVector::distance_bytes_lorentz_f32(&bytes, &q_f32);

        let diff = (dist_f64 - dist_f32).abs();
        let tol = dist_f64 * 1e-4 + 1e-5;
        assert!(
            diff <= tol,
            "Lorentz mismatch at dim={dim}: f64={dist_f64}, f32={dist_f32}, diff={diff}"
        );
    }
}

#[test]
fn test_auto_freezing_flat_layer0_and_search_equivalence() {
    let dir = tempdir().unwrap();
    let storage_path = dir.path().join("vectors");
    let snap_path = dir.path().join("index.snap");
    let dim = 16;

    let config = Arc::new(GlobalConfig::default());
    config.set_m(16);
    config.set_ef_construction(64);

    let storage = Arc::new(VectorStore::new(&storage_path, dim * 8 + 8));
    let index: HnswIndex<EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone(), dim);

    assert!(!index.has_flat_layer0());

    let mut queries = Vec::new();
    for i in 0..48u32 {
        let mut vec = vec![0.0f64; dim];
        for (d, item) in vec.iter_mut().enumerate() {
            *item = (i as f64 * 0.1) + (d as f64 * 0.05);
        }
        let _ = index.insert(&vec, HashMap::new()).expect("insert");
        if i % 10 == 0 {
            queries.push(vec);
        }
    }

    let params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let empty_map = HashMap::new();

    // 1. Dynamic search results
    let mut dynamic_results = Vec::new();
    for q in &queries {
        let res = index.search(q, &empty_map, &[], &params);
        dynamic_results.push(res);
    }

    // 2. Auto-freeze flat layer 0
    index.ensure_flat_layer0();
    assert!(index.has_flat_layer0());

    // 3. Flat layer 0 search results
    for (i, q) in queries.iter().enumerate() {
        let res = index.search(q, &empty_map, &[], &params);
        assert_eq!(
            res.len(),
            dynamic_results[i].len(),
            "Result lengths should match"
        );
        for (a, b) in res.iter().zip(&dynamic_results[i]) {
            assert_eq!(a.0, b.0, "Candidate node IDs must match");
            assert!(
                (a.1 - b.1).abs() <= 1e-4,
                "Distances must match: {} vs {}",
                a.1,
                b.1
            );
        }
    }

    // 4. Snapshot saves flat_l0.bin automatically
    index.save_snapshot(&snap_path).unwrap();
    assert!(dir.path().join("flat_l0.bin").exists());

    // 5. Load snapshot loads flat layer 0 automatically
    let loaded: HnswIndex<EuclideanMetric> = HnswIndex::load_snapshot(
        &snap_path,
        storage.clone(),
        QuantizationMode::None,
        config.clone(),
        dim,
    )
    .unwrap();

    assert!(loaded.has_flat_layer0());
    for (i, q) in queries.iter().enumerate() {
        let res = loaded.search(q, &empty_map, &[], &params);
        assert_eq!(res.len(), dynamic_results[i].len());
        for (a, b) in res.iter().zip(&dynamic_results[i]) {
            assert_eq!(a.0, b.0);
            assert!((a.1 - b.1).abs() <= 1e-4);
        }
    }
}

#[test]
fn test_poincare_search_throughput_and_recall() {
    use hyperspace_core::PoincareMetric;

    let dir = tempdir().unwrap();
    let dim = 32;
    let n_vectors = 1000;
    let n_queries = 200;

    let storage = Arc::new(VectorStore::new(&dir.path().join("vectors"), dim * 8 + 8));
    let config = Arc::new(GlobalConfig::default());
    config.set_m(16);
    config.set_ef_construction(64);

    let index = HnswIndex::<PoincareMetric>::new(
        storage.clone(),
        QuantizationMode::None,
        config.clone(),
        dim,
    );

    let mut vectors = Vec::with_capacity(n_vectors);
    let scale = 0.5 / (dim as f64).sqrt();
    for i in 0..n_vectors {
        let mut coords = Vec::with_capacity(dim);
        for d in 0..dim {
            coords.push(((i * 7 + d * 13) as f64 * 0.05).sin() * scale);
        }
        let _ = index.insert(&coords, HashMap::new()).expect("insert");
        if i < n_queries {
            vectors.push(coords);
        }
    }

    let empty_map = HashMap::new();
    let params = SearchParams {
        top_k: 10,
        ef_search: 32,
        ..Default::default()
    };

    let start = std::time::Instant::now();
    let mut total_hits = 0;
    for (q_idx, q) in vectors.iter().enumerate() {
        let res = index.search(q, &empty_map, &[], &params);
        if !res.is_empty() {
            total_hits += 1;
            // The query itself must be the top result (distance ~0.0)
            assert_eq!(res[0].0, q_idx as u32);
            assert!(res[0].1 < 1e-4, "Top-1 distance must be ~0, got {}", res[0].1);
        }
    }
    let elapsed = start.elapsed();
    let qps = (n_queries as f64) / elapsed.as_secs_f64();
    println!(
        "🚀 Poincare HNSW Search: {} queries in {:?} => {:.0} QPS (top-1 accuracy 100%)",
        n_queries, elapsed, qps
    );
    assert_eq!(total_hits, n_queries);
}
