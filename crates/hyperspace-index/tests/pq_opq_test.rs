use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;

fn generate_random_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f64>> {
    let mut rng_state = seed;
    let mut rand_f64 = || {
        // Simple 64-bit LCG
        rng_state = rng_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[allow(clippy::cast_precision_loss)]
        let val = (rng_state >> 32) as f64 / f64::from(u32::MAX);
        val * 2.0 - 1.0
    };

    (0..n)
        .map(|_| {
            let mut v: Vec<f64> = (0..dim).map(|_| rand_f64()).collect();
            // L2 normalize
            let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-9);
            for x in &mut v {
                *x /= norm;
            }
            v
        })
        .collect()
}

#[test]
fn test_product_quantization_hnsw_insertion_and_search() {
    let dir = tempfile::tempdir().unwrap();
    let storage_path = dir.path().join("vectors_pq");
    let dim = 128;
    let num_sub = hyperspace_core::pq::default_num_subvectors(dim);
    assert_eq!(num_sub, 8, "128D / 16 = 8 subvectors");

    // Element size for 8 subvectors is 8 bytes per vector
    let storage = Arc::new(VectorStore::new(&storage_path, num_sub));
    let config = Arc::new(GlobalConfig::default());
    let index: HnswIndex<EuclideanMetric> = HnswIndex::new(
        storage.clone(),
        QuantizationMode::ProductQuantization,
        config,
        dim,
    );

    assert!(index.pq.is_some());
    assert!(!index.pq.as_ref().unwrap().is_opq);

    let vectors = generate_random_vectors(150, dim, 42);
    for (i, v) in vectors.iter().enumerate() {
        let mut meta = HashMap::new();
        meta.insert("tag".to_string(), format!("item_{i}"));
        index.insert(v, meta).expect("Insert must succeed");
    }

    assert_eq!(index.count(), 150);
    // Vector store byte count should be 150 * 8 bytes = 1200 bytes
    assert_eq!(storage.count(), 150);

    // Test reconstruction via get_vector
    let reconstructed = index.get_vector(0);
    assert_eq!(reconstructed.coords.len(), dim);

    // Search nearest neighbors
    let query = &vectors[10];
    let params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = index.search(query, &HashMap::new(), &[], &params);
    assert!(!results.is_empty());
    // Since query vector is identical to vector 10, ID 10 should be in top results
    let top_ids: Vec<u32> = results.iter().map(|r| r.0).collect();
    assert!(
        top_ids.contains(&10),
        "Search should find query vector 10 in top-5, found: {top_ids:?}"
    );
}

#[test]
fn test_opq_hnsw_insertion_search_and_persistence() {
    let dir = tempfile::tempdir().unwrap();
    let snap_path = dir.path().join("opq_index.snap");
    let storage_path = dir.path().join("vectors_opq");
    let dim = 128;
    let num_sub = hyperspace_core::pq::default_num_subvectors(dim);

    let storage = Arc::new(VectorStore::new(&storage_path, num_sub));
    let config = Arc::new(GlobalConfig::default());
    let index: HnswIndex<EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::OPQ, config.clone(), dim);

    assert!(index.pq.is_some());
    assert!(index.pq.as_ref().unwrap().is_opq);

    let vectors = generate_random_vectors(200, dim, 12345);
    for (i, v) in vectors.iter().enumerate() {
        let mut meta = HashMap::new();
        meta.insert(
            "cat".to_string(),
            if i % 2 == 0 {
                "even".into()
            } else {
                "odd".into()
            },
        );
        index.insert(v, meta).expect("Insert must succeed");
    }

    // Search before saving
    let query = &vectors[25];
    let params = SearchParams {
        top_k: 5,
        ef_search: 64,
        ..Default::default()
    };
    let results_before = index.search(query, &HashMap::new(), &[], &params);
    let top_ids: Vec<u32> = results_before.iter().map(|r| r.0).collect();
    assert!(
        top_ids.contains(&25),
        "OPQ search should find query vector 25 in top-5, found: {top_ids:?}"
    );

    // Save snapshot
    index
        .save_snapshot(&snap_path)
        .expect("Save snapshot failed");

    // Reload snapshot
    let loaded_index: HnswIndex<EuclideanMetric> =
        HnswIndex::load_snapshot(&snap_path, storage, QuantizationMode::OPQ, config, dim)
            .expect("Load snapshot failed");

    assert!(loaded_index.pq.is_some());
    assert!(loaded_index.pq.as_ref().unwrap().is_opq);
    assert_eq!(loaded_index.count(), 200);

    // Search after reloading
    let results_after = loaded_index.search(query, &HashMap::new(), &[], &params);
    assert_eq!(results_before.len(), results_after.len());
    for (b, a) in results_before.iter().zip(results_after.iter()) {
        assert_eq!(
            b.0, a.0,
            "Result IDs must match before and after snapshot reload"
        );
        assert!(
            (b.1 - a.1).abs() < 1e-4,
            "Result distances must match before and after reload"
        );
    }
}

#[test]
fn test_pq_custom_codebook_training() {
    let dir = tempfile::tempdir().unwrap();
    let storage_path = dir.path().join("vectors_pq_trained");
    let dim = 64;
    let num_sub = hyperspace_core::pq::default_num_subvectors(dim);
    assert_eq!(num_sub, 4, "64D / 16 = 4 subvectors");

    let storage = Arc::new(VectorStore::new(&storage_path, num_sub));
    let config = Arc::new(GlobalConfig::default());
    let mut index: HnswIndex<EuclideanMetric> =
        HnswIndex::new(storage, QuantizationMode::ProductQuantization, config, dim);

    // Train on sample dataset
    #[allow(clippy::cast_precision_loss)]
    let train_data: Vec<Vec<f32>> = (0..300)
        .map(|i| {
            (0..dim)
                .map(|d| ((i * 17 + d * 31) % 100) as f32 / 100.0)
                .collect()
        })
        .collect();
    let train_slices: Vec<&[f32]> = train_data.iter().map(std::vec::Vec::as_slice).collect();

    index.train_pq(&train_slices, 5);

    // Insert vectors
    for (i, v_f32) in train_data.iter().take(50).enumerate() {
        let v_f64: Vec<f64> = v_f32.iter().copied().map(f64::from).collect();
        index
            .insert(&v_f64, HashMap::new())
            .expect("Insert succeeded");
        #[allow(clippy::cast_possible_truncation)]
        let rec = index.get_vector(i as u32);
        assert_eq!(rec.coords.len(), dim);
    }

    assert_eq!(index.count(), 50);
}
