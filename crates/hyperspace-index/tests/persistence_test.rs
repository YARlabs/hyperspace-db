use hyperspace_core::{EuclideanMetric, GlobalConfig, Metric, QuantizationMode};
use hyperspace_index::{HnswIndex, SnapshotData, SnapshotMetadata, SnapshotNode};
use hyperspace_store::VectorStore;
use parking_lot::RwLock;
use rkyv::Deserialize;
use roaring::RoaringBitmap;
use std::sync::Arc; // Correct import

#[test]
fn test_metadata_persistence() {
    let mut metadata = SnapshotMetadata {
        inverted: vec![("tag1".to_string(), {
            let mut b = RoaringBitmap::new();
            b.insert(1);
            let mut buf = Vec::new();
            b.serialize_into(&mut buf).unwrap();
            buf
        })],
        numeric: vec![(
            "score".to_string(),
            vec![(100, {
                let mut b = RoaringBitmap::new();
                b.insert(2);
                let mut buf = Vec::new();
                b.serialize_into(&mut buf).unwrap();
                buf
            })],
        )],
        deleted: {
            let mut b = RoaringBitmap::new();
            b.insert(5);
            let mut buf = Vec::new();
            b.serialize_into(&mut buf).unwrap();
            buf
        },
        forward: vec![(1, vec![("tag1".to_string(), "true".to_string())])],
    };

    let snapshot = SnapshotData {
        max_layer: 0,
        entry_point: 0,
        nodes: vec![SnapshotNode {
            id: 1,
            layers: vec![vec![]],
        }],
        metadata,
    };

    // Serialize
    let bytes = rkyv::to_bytes::<_, 1024>(&snapshot).expect("Serialization failed");

    // Deserialize
    let archived = unsafe { rkyv::archived_root::<SnapshotData>(&bytes) };
    let deserialized: SnapshotData = archived.deserialize(&mut rkyv::Infallible).unwrap();

    // Verify
    assert_eq!(deserialized.metadata.forward.len(), 1);
    assert_eq!(deserialized.metadata.forward[0].0, 1);
    assert_eq!(deserialized.metadata.forward[0].1[0].0, "tag1");

    // Verify Bitmap Loading Logic (manually check)
    let bitmap = RoaringBitmap::deserialize_from(&deserialized.metadata.deleted[..]).unwrap();
    assert!(bitmap.contains(5));
}

#[test]
fn test_index_save_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("index.snap");
    let storage_path = dir.path().join("vectors");

    let config = Arc::new(GlobalConfig::default());
    let storage = Arc::new(VectorStore::new(&storage_path, 4)); // 1 float dim
    let index: HnswIndex<1, EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone());

    // Add metadata
    {
        let mut deleted = index.metadata.deleted.write();
        deleted.insert(10);
    }
    index.metadata.inverted.insert("category".to_string(), {
        let mut r = RoaringBitmap::new();
        r.insert(1);
        r
    });

    // Save
    index.save_snapshot(&path).expect("Save failed");

    // Load
    let layout_file = std::fs::File::open(&path).unwrap();
    let loaded_index: HnswIndex<1, EuclideanMetric> =
        HnswIndex::load_snapshot(&path, storage, QuantizationMode::None, config)
            .expect("Load failed");

    // Check Metadata
    assert!(loaded_index.metadata.deleted.read().contains(10));
    assert!(loaded_index
        .metadata
        .inverted
        .get("category")
        .unwrap()
        .contains(1));
}
