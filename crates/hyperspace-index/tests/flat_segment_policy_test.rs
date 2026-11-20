//! When the frozen layer-0 segment is built, kept, or dropped.

use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode, SearchParams};
use hyperspace_index::{set_flat_scan_limit, HnswIndex};
use hyperspace_store::VectorStore;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;

const DIM: usize = 16;

fn point(i: u32) -> Vec<f64> {
    (0..DIM)
        .map(|d| f64::from((i * 31 + d as u32 * 7) % 113) / 113.0)
        .collect()
}

fn build(dir: &std::path::Path, n: u32) -> HnswIndex<EuclideanMetric> {
    let index = HnswIndex::<EuclideanMetric>::new_with_storage_precision(
        Arc::new(VectorStore::new(dir, DIM * 4 + 4)),
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        DIM,
    );
    for i in 0..n {
        let id = index.insert_to_storage(&point(i)).unwrap();
        index.index_node(id, HashMap::new()).unwrap();
    }
    index
}

fn reload(dir: &std::path::Path) -> HnswIndex<EuclideanMetric> {
    HnswIndex::<EuclideanMetric>::load_snapshot_with_storage_precision(
        &dir.join("index.snap"),
        Arc::new(VectorStore::new(dir, DIM * 4 + 4)),
        QuantizationMode::None,
        Arc::new(GlobalConfig::default()),
        true,
        DIM,
    )
    .unwrap()
}

// One test function: the flat-scan override is process-wide.
#[test]
fn frozen_segment_policy() {
    let params = SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };

    // 1. A small index (flat-scanned) never builds or loads the segment copy.
    set_flat_scan_limit(None);
    let dir = tempdir().unwrap();
    let small = build(dir.path(), 300);
    small.save_snapshot(&dir.path().join("index.snap")).unwrap();
    assert!(
        !small.has_flat_layer0(),
        "no segment copy for a flat-scanned index"
    );
    let small2 = reload(dir.path());
    assert!(!small2.has_flat_layer0());
    assert_eq!(
        small2
            .search(&point(3), &HashMap::new(), &[], &params)
            .len(),
        5
    );

    // 2. A memory-mapped segment cannot take neighbor updates: the first update drops it
    //    instead of leaving stale adjacency in the search path.
    set_flat_scan_limit(Some(0)); // force graph walks + segment use
    let dir = tempdir().unwrap();
    let big = build(dir.path(), 300);
    big.save_snapshot(&dir.path().join("index.snap")).unwrap();
    assert!(big.has_flat_layer0());
    let mapped = reload(dir.path());
    assert!(mapped.has_flat_layer0(), "segment loaded from flat_l0.bin");
    // Deleting enough nodes triggers graph repair, which relinks existing nodes.
    for id in 0..120u32 {
        mapped.delete(id);
    }
    let _ = mapped.repair_deleted();
    assert!(
        !mapped.has_flat_layer0(),
        "read-only segment must be dropped once its adjacency is stale"
    );
    let res = mapped.search(&point(200), &HashMap::new(), &[], &params);
    assert_eq!(res.len(), 5);
    assert!(
        res.iter().all(|(id, _)| *id >= 120),
        "no deleted node returned"
    );
    set_flat_scan_limit(None);
}
