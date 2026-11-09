#![allow(clippy::pedantic)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

use hyperspace_core::{
    CosineMetric, EuclideanMetric, GlobalConfig, HybridMetric, LorentzMetric, PoincareMetric,
    QuantizationMode, SearchParams,
};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_ternary_euclidean_search_and_reconstruct() {
    let dir = tempdir().unwrap();
    let dim: usize = 128;
    let elem_size = 2 * dim.div_ceil(8) + 8;
    let store_path = dir.path().join("store_euc.bin");
    let store = Arc::new(VectorStore::new(&store_path, elem_size));
    let config = Arc::new(GlobalConfig::default());
    let index = HnswIndex::<EuclideanMetric>::new(store, QuantizationMode::Ternary, config, dim);

    let mut rng_seed = 12345u64;
    let mut next_f64 = || -> f64 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5
    };

    let mut query = vec![0.0; dim];
    for x in &mut query {
        *x = next_f64();
    }

    for _ in 0..100 {
        let mut v = vec![0.0; dim];
        for x in &mut v {
            *x = next_f64();
        }
        let id = index.insert_to_storage(&v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();

        let reconstructed = index.get_vector_original(id);
        assert_eq!(reconstructed.coords.len(), dim);
        assert!(reconstructed.coords.iter().all(|c| c.is_finite()));
    }

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = index.search(&query, &HashMap::new(), &[], &search_params);
    assert_eq!(results.len(), 5);
    for (_id, dist) in &results {
        assert!(dist.is_finite());
        assert!(*dist >= 0.0);
    }
}

#[test]
fn test_ternary_cosine_search() {
    let dir = tempdir().unwrap();
    let dim: usize = 128;
    let elem_size = 2 * dim.div_ceil(8) + 8;
    let store_path = dir.path().join("store_cos.bin");
    let store = Arc::new(VectorStore::new(&store_path, elem_size));
    let config = Arc::new(GlobalConfig::default());
    let index = HnswIndex::<CosineMetric>::new(store, QuantizationMode::Ternary, config, dim);

    let mut rng_seed = 54321u64;
    let mut next_f64 = || -> f64 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5
    };

    let mut query = vec![0.0; dim];
    for x in &mut query {
        *x = next_f64();
    }

    for _ in 0..50 {
        let mut v = vec![0.0; dim];
        for x in &mut v {
            *x = next_f64();
        }
        let id = index.insert_to_storage(&v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = index.search(&query, &HashMap::new(), &[], &search_params);
    assert_eq!(results.len(), 5);
    for (_id, dist) in &results {
        assert!(dist.is_finite());
        assert!(*dist >= 0.0);
    }
}

#[test]
fn test_ternary_poincare_search() {
    let dir = tempdir().unwrap();
    let dim: usize = 64;
    let elem_size = 2 * dim.div_ceil(8) + 8;
    let store_path = dir.path().join("store_poin.bin");
    let store = Arc::new(VectorStore::new(&store_path, elem_size));
    let config = Arc::new(GlobalConfig::default());
    let index = HnswIndex::<PoincareMetric>::new(store, QuantizationMode::Ternary, config, dim);

    let mut rng_seed = 99999u64;
    let mut next_f64 = || -> f64 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5
    };

    let make_poincare_vec = |seed_fn: &mut dyn FnMut() -> f64| -> Vec<f64> {
        let mut v = vec![0.0; dim];
        for x in &mut v {
            *x = seed_fn();
        }
        let norm: f64 = v.iter().map(|&x| x * x).sum::<f64>().sqrt();
        let target_norm = 0.8;
        for x in &mut v {
            *x = (*x / norm) * target_norm;
        }
        v
    };

    let query = make_poincare_vec(&mut next_f64);

    for _ in 0..50 {
        let v = make_poincare_vec(&mut next_f64);
        let id = index.insert_to_storage(&v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = index.search(&query, &HashMap::new(), &[], &search_params);
    assert_eq!(results.len(), 5);
    for (_id, dist) in &results {
        assert!(dist.is_finite());
        assert!(*dist >= 0.0);
    }
}

#[test]
fn test_ternary_lorentz_search() {
    let dir = tempdir().unwrap();
    let dim: usize = 64;
    let elem_size = 2 * dim.div_ceil(8) + 8;
    let store_path = dir.path().join("store_lor.bin");
    let store = Arc::new(VectorStore::new(&store_path, elem_size));
    let config = Arc::new(GlobalConfig::default());
    let index = HnswIndex::<LorentzMetric>::new(store, QuantizationMode::Ternary, config, dim);

    let mut rng_seed = 77777u64;
    let mut next_f64 = || -> f64 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5
    };

    let make_lorentz_vec = |seed_fn: &mut dyn FnMut() -> f64| -> Vec<f64> {
        let mut v = vec![0.0; dim];
        let mut spatial_sq = 0.0;
        for i in 1..dim {
            let x = seed_fn() * 0.05;
            v[i] = x;
            spatial_sq += x * x;
        }
        v[0] = (1.0 + spatial_sq).sqrt();
        v
    };

    let query = make_lorentz_vec(&mut next_f64);

    for _ in 0..50 {
        let v = make_lorentz_vec(&mut next_f64);
        let id = index.insert_to_storage(&v).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }

    let search_params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = index.search(&query, &HashMap::new(), &[], &search_params);
    assert_eq!(results.len(), 5);
    for (_id, dist) in &results {
        assert!(dist.is_finite());
        assert!(*dist >= 0.0);
    }
}

#[test]
fn test_ternary_hybrid_801d_search_and_mrl() {
    let dir = tempdir().unwrap();
    let dim: usize = 801;
    let elem_size = 33 * 4 + 2 * (768usize).div_ceil(8) + 8;
    assert_eq!(elem_size, 332);

    let store_path = dir.path().join("store_hyb.bin");
    let store = Arc::new(VectorStore::new(&store_path, elem_size));
    let config = Arc::new(GlobalConfig::default());
    let index = HnswIndex::<HybridMetric>::new(store, QuantizationMode::Ternary, config, dim);

    let mut rng_seed = 11111u64;
    let mut next_f64 = || -> f64 {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((rng_seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5
    };

    let make_hybrid_vec = |seed_fn: &mut dyn FnMut() -> f64| -> Vec<f64> {
        let mut v = vec![0.0; dim];
        let mut spatial_sq = 0.0;
        for i in 1..33 {
            let x = seed_fn() * 0.05;
            v[i] = x;
            spatial_sq += x * x;
        }
        v[0] = (1.0 + spatial_sq).sqrt();

        for i in 33..dim {
            v[i] = seed_fn();
        }
        v
    };

    let query = make_hybrid_vec(&mut next_f64);

    for i in 0..60 {
        let v = make_hybrid_vec(&mut next_f64);
        let id = index.insert_to_storage(&v).unwrap();
        let mut meta = HashMap::new();
        if i % 2 == 0 {
            meta.insert("even".to_string(), "true".to_string());
        }
        index.index_node(id, meta).unwrap();
    }

    // 1. Full search
    let search_params = SearchParams {
        top_k: 10,
        ef_search: 64,
        ..Default::default()
    };
    let results = index.search(&query, &HashMap::new(), &[], &search_params);
    assert_eq!(results.len(), 10);
    for (_id, dist) in &results {
        assert!(dist.is_finite());
        assert!(*dist >= 0.0);
    }

    // 2. Filtered search
    let mut filter = HashMap::new();
    filter.insert("even".to_string(), "true".to_string());
    let filtered_results = index.search(&query, &filter, &[], &search_params);
    assert!(!filtered_results.is_empty());
    assert!(filtered_results.len() <= 10);
    for (id, _dist) in &filtered_results {
        assert_eq!(*id % 2, 0);
    }

    // 3. MRL search (cutoff at 256)
    let mrl_params = SearchParams {
        top_k: 5,
        ef_search: 32,
        mrl_dimension: Some(256),
        ..Default::default()
    };
    let mrl_results = index.search(&query, &HashMap::new(), &[], &mrl_params);
    assert_eq!(mrl_results.len(), 5);
}
