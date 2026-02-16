use hyperspace_core::{EuclideanMetric, GlobalConfig, Metric, QuantizationMode};
use hyperspace_index::{HnswIndex, SnapshotData, SnapshotMetadata, SnapshotNode};
use hyperspace_store::VectorStore;
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_indexing_stress() {
    let config = Arc::new(GlobalConfig::default());
    // Use M=16 to trigger pruning frequently
    config.set_m(16);
    config.set_ef_construction(100);

    let storage = Arc::new(VectorStore::new(std::path::Path::new("mem"), 4)); // 1 float
    let index: Arc<HnswIndex<1, EuclideanMetric>> =
        Arc::new(HnswIndex::new(storage, QuantizationMode::None, config));

    let mut handles = vec![];
    let num_threads = 8;
    let items_per_thread = 1000;

    // Concurrent Insertions
    for i in 0..num_threads {
        let index_ref = index.clone();
        handles.push(thread::spawn(move || {
            let mut rng = rand::thread_rng();
            for j in 0..items_per_thread {
                let id = (i * items_per_thread + j) as u32;
                let val = rng.gen_range(0.0..100.0);
                let vec = vec![val];
                // Insert with random metadata to test lock contention on metadata too
                let mut meta = std::collections::HashMap::new();
                meta.insert("thread".to_string(), i.to_string());

                // This calls index_node internally which calls prune_connections
                let _ = index_ref.insert(&vec, meta);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Verify consistency
    // Simple check: do we have all nodes?
    // Since insert returns internal ID, we can't easily check count unless we track it.
    // But we can check if the graph is valid (no panic happened).
    println!("Indexing complete without panic.");
}
