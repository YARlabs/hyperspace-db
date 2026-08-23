use hyperspace_core::{EuclideanMetric, GlobalConfig, QuantizationMode};
use hyperspace_index::{HnswIndex, SnapshotData, SnapshotMetadata, SnapshotNode};
use hyperspace_store::VectorStore;
use rkyv::Deserialize;
use roaring::RoaringBitmap;
use std::sync::Arc; // Correct import

#[test]
fn test_metadata_persistence() {
    let metadata = SnapshotMetadata {
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
    let index: HnswIndex<EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone(), 1);

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
    let _layout_file = std::fs::File::open(&path).unwrap();
    let loaded_index: HnswIndex<EuclideanMetric> =
        HnswIndex::load_snapshot(&path, storage, QuantizationMode::None, config, 1)
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

// L51 ATTACK TEST: a failed snapshot save must NOT destroy the existing good
// snapshot. The atomic temp+fsync+rename path writes to `index.snap.tmp` first;
// if that write fails (here we make it fail deterministically by occupying the
// tmp path with a directory so `File::create` returns EISDIR), the live
// `index.snap` must remain the previous, fully-valid snapshot -- never a
// truncated stub. Under the old in-place `File::create(path)` code the target
// would have been truncated immediately and the good data lost (the exact
// 2026-06-10 corruption: 51509 -> 634 vectors after "No space left").
#[test]
fn test_atomic_save_preserves_old_snapshot_on_failure() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("index.snap");
    let storage_path = dir.path().join("vectors");

    let config = Arc::new(GlobalConfig::default());
    let storage = Arc::new(VectorStore::new(&storage_path, 4));
    let index: HnswIndex<1, EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone());

    // ---- v1: the GOOD snapshot we must not lose (marker id = 10) ----
    {
        let mut deleted = index.metadata.deleted.write();
        deleted.insert(10);
    }
    index.save_snapshot(&path).expect("v1 save failed");
    assert!(path.exists(), "v1 snapshot must exist");
    let v1_len = std::fs::metadata(&path).unwrap().len();
    assert!(v1_len > 0, "v1 snapshot must be non-empty");

    // ---- Sabotage the temp path so the next save fails mid-write ----
    let tmp_path = {
        let mut s = path.as_os_str().to_os_string();
        s.push(".tmp");
        std::path::PathBuf::from(s)
    };
    std::fs::create_dir(&tmp_path).expect("occupy tmp path with a directory");

    // ---- v2: attempt to overwrite with new data (marker id = 20) ----
    {
        let mut deleted = index.metadata.deleted.write();
        deleted.insert(20);
    }
    let result = index.save_snapshot(&path);
    assert!(
        result.is_err(),
        "save must fail when the temp file cannot be created"
    );

    // ---- The crucial L51 invariant: the GOOD v1 snapshot survived intact ----
    assert!(path.exists(), "old snapshot must still exist after failed save");
    assert_eq!(
        std::fs::metadata(&path).unwrap().len(),
        v1_len,
        "old snapshot must be byte-for-byte unchanged (not truncated)"
    );

    // Remove the saboteur dir and prove the surviving file still loads as v1.
    std::fs::remove_dir(&tmp_path).unwrap();
    let loaded: HnswIndex<1, EuclideanMetric> =
        HnswIndex::load_snapshot(&path, storage, QuantizationMode::None, config)
            .expect("surviving snapshot must still load");
    assert!(
        loaded.metadata.deleted.read().contains(10),
        "surviving snapshot must be v1 (contains 10)"
    );
    assert!(
        !loaded.metadata.deleted.read().contains(20),
        "failed v2 write must not have leaked into the live snapshot"
    );
}

// L51: a SUCCESSFUL save must leave no orphan temp file behind (the rename
// consumes index.snap.tmp), and the live snapshot must reflect the new data.
#[test]
fn test_atomic_save_leaves_no_orphan_tmp() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("index.snap");
    let storage_path = dir.path().join("vectors");
    let tmp_path = {
        let mut s = path.as_os_str().to_os_string();
        s.push(".tmp");
        std::path::PathBuf::from(s)
    };

    let config = Arc::new(GlobalConfig::default());
    let storage = Arc::new(VectorStore::new(&storage_path, 4));
    let index: HnswIndex<1, EuclideanMetric> =
        HnswIndex::new(storage.clone(), QuantizationMode::None, config.clone());

    {
        let mut deleted = index.metadata.deleted.write();
        deleted.insert(7);
    }
    index.save_snapshot(&path).expect("save failed");

    assert!(path.exists(), "live snapshot must exist after save");
    assert!(
        !tmp_path.exists(),
        "no orphan .tmp file may remain after a successful save"
    );

    let loaded: HnswIndex<1, EuclideanMetric> =
        HnswIndex::load_snapshot(&path, storage, QuantizationMode::None, config)
            .expect("Load failed");
    assert!(loaded.metadata.deleted.read().contains(7));
}
