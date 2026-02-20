use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_index::HnswIndex;
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_graph_traversal_api_basics() {
    let dir = tempfile::tempdir().expect("tempdir");
    let storage_path = dir.path().join("vectors");
    let config = Arc::new(GlobalConfig::default());
    config.set_m(32);
    config.set_ef_construction(120);

    let storage = Arc::new(VectorStore::new(
        &storage_path,
        hyperspace_core::vector::HyperVector::<8>::SIZE,
    ));
    let index: HnswIndex<8, EuclideanMetric> =
        HnswIndex::new(storage, QuantizationMode::None, config);

    for i in 0..128u32 {
        let mut vec = vec![0.0; 8];
        let base = if i < 64 { 0.1 } else { 0.9 };
        for (j, item) in vec.iter_mut().enumerate().take(8) {
            *item = base + (j as f64) * 0.001;
        }
        let mut meta = HashMap::new();
        meta.insert("bucket".to_string(), if i < 64 { "a" } else { "b" }.to_string());
        let _ = index.insert(&vec, meta).expect("insert");
    }

    let neighbors = index.graph_neighbors(0, 0, 16).expect("neighbors");
    assert!(!neighbors.is_empty(), "neighbors should not be empty");
    assert!(neighbors.len() <= 16);

    let traversed = index.graph_traverse(0, 0, 2, 64).expect("traverse");
    assert!(!traversed.is_empty(), "traverse should return at least start node");
    assert_eq!(traversed[0], 0);
    assert!(traversed.len() <= 64);

    let clusters = index.graph_connected_components(0, 3, 16, 256);
    assert!(!clusters.is_empty(), "clusters should not be empty");
    assert!(clusters.iter().all(|c| c.len() >= 3));
}
