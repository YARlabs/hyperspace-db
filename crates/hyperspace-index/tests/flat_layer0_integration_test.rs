#![allow(unknown_lints)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_precision_loss)]

use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_flat_layer0_search_equivalence_and_persistence() {
    let dir = tempfile::tempdir().expect("tempdir");
    let storage_path = dir.path().join("vectors");
    let config = Arc::new(GlobalConfig::default());
    config.set_m(16);
    config.set_ef_construction(64);

    let dim = 16;
    let storage = Arc::new(VectorStore::new(&storage_path, dim * 8 + 8));
    let index: HnswIndex<EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone(), dim);

    // Insert 64 structured vectors
    let mut all_vectors = Vec::new();
    for i in 0..64u32 {
        let mut vec = vec![0.0f64; dim];
        for (d, item) in vec.iter_mut().enumerate() {
            *item = (i as f64 * 0.1) + (d as f64 * 0.05);
        }
        let meta = HashMap::new();
        let _ = index.insert(&vec, meta).expect("insert");
        all_vectors.push(vec);
    }

    let query: Vec<f64> = (0..dim).map(|d| 2.5 + (d as f64 * 0.05)).collect();
    let params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let empty_map = HashMap::new();

    // 1. Search using the dynamic path
    assert!(!index.has_flat_layer0());
    let dynamic_results = index.search(&query, &empty_map, &[], &params);
    assert_eq!(dynamic_results.len(), 5);

    // 2. Freeze into FlatLayer0Segment and search using flat path
    let flat_seg = index.freeze_flat_layer0();
    assert!(index.has_flat_layer0());
    assert_eq!(flat_seg.node_count(), 64);

    let flat_results = index.search(&query, &empty_map, &[], &params);
    assert_eq!(flat_results.len(), 5);

    // Results must match between dynamic and flat layer-0 search!
    for i in 0..5 {
        assert_eq!(
            dynamic_results[i].0, flat_results[i].0,
            "Result mismatch at rank {i}"
        );
        assert!(
            (dynamic_results[i].1 - flat_results[i].1).abs() < 1e-5,
            "Distance mismatch at rank {i}"
        );
    }

    // 3. Test persistence: save and reload snapshot
    let snap_path = dir.path().join("index.snap");
    index.save_snapshot(&snap_path).expect("save_snapshot");

    // Sidecar flat_l0.bin must exist
    let flat_bin_path = dir.path().join("flat_l0.bin");
    assert!(flat_bin_path.exists(), "flat_l0.bin sidecar must exist");

    let loaded_index: HnswIndex<EuclideanMetric> =
        HnswIndex::load_snapshot(&snap_path, storage, QuantizationMode::None, config, dim)
            .expect("load_snapshot");

    assert!(
        loaded_index.has_flat_layer0(),
        "Loaded index must automatically activate flat_l0 sidecar"
    );

    let reloaded_results = loaded_index.search(&query, &empty_map, &[], &params);
    assert_eq!(reloaded_results.len(), 5);
    for i in 0..5 {
        assert_eq!(reloaded_results[i].0, dynamic_results[i].0);
    }
}
