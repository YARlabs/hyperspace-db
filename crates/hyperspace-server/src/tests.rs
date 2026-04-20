use super::manager::CollectionManager;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::Duration;
use tokio::sync::broadcast;
use uuid::Uuid;

use hyperspace_core::Durability;

#[tokio::test]
async fn test_rebuild_and_queue() {
    // Setup temporary directory
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();
    println!("Test dir: {tmp_dir:?}");

    let (tx, _rx) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx);

    // 1. Create Collection
    let col_name = "test_rebuild";
    manager
        .create_collection("default_admin", col_name, 128, "l2")
        .await
        .expect("Create failed");

    if let Some(col) = manager.get("default_admin", col_name).await {
        assert_eq!(col.count(), 0);

        // 2. Insert Data & Check Queue
        println!("Inserting 100 vectors...");
        let vec = vec![0.1; 128];
        for i in 0..100 {
            col.insert(&vec, i as u32, HashMap::new(), 0, Durability::Default)
                .await
                .expect("Insert failed");
        }

        // Check queue size
        let q = col.queue_size();
        println!("Queue size after insert: {q}");

        // Wait for indexing to finish
        let start = std::time::Instant::now();
        loop {
            if col.queue_size() == 0 {
                break;
            }
            assert!(
                start.elapsed() <= Duration::from_secs(10),
                "Indexing timeout. Queue: {}",
                col.queue_size()
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        println!("Indexing complete. Count: {}", col.count());
        assert_eq!(col.count(), 100);
    } else {
        panic!("Collection not found");
    }

    // 3. Rebuild Index
    println!("Triggering rebuild...");
    manager
        .rebuild_collection("default_admin", col_name)
        .await
        .expect("Rebuild failed");

    // After rebuild, verify data
    if let Some(col_new) = manager.get("default_admin", col_name).await {
        assert_eq!(col_new.count(), 100);
    } else {
        panic!("Collection not found after rebuild");
    }

    // Verify optimized file exists
    let folder_name = format!("default_admin_{col_name}");
    let index_path = tmp_dir.join(folder_name).join("index.snap");
    assert!(index_path.exists());

    println!("Rebuild successful. Cleaning up.");

    // 4. Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_vacuum() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_test_vac_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx);

    manager
        .create_collection("default_admin", "vac_col", 64, "l2")
        .await
        .unwrap();

    let _ = fs::remove_dir_all(&tmp_dir);
}

/// Task 2.1: Delta Sync test — simulates Network Partition and recovery.
/// Two "nodes" (CollectionManager instances) insert different vectors,
/// then use the digest-based diff protocol to synchronize.
#[tokio::test]
async fn test_delta_sync() {
    // Setup two temporary directories (Node A, Node B)
    let uuid = Uuid::new_v4();
    let dir_a = env::temp_dir().join(format!("hyperspace_sync_a_{uuid}"));
    let dir_b = env::temp_dir().join(format!("hyperspace_sync_b_{uuid}"));
    fs::create_dir_all(&dir_a).unwrap();
    fs::create_dir_all(&dir_b).unwrap();

    // Use f64 storage (no quantization) to ensure lossless vector roundtrip.
    // With scalar quantization, f64→i8→f64 introduces precision loss that
    // causes hash mismatches between original insert and peek-reconstructed vectors.
    env::set_var("HS_QUANTIZATION_LEVEL", "none");
    env::set_var("HS_GOSSIP_ENABLED", "true");

    let (tx_a, _) = broadcast::channel(100);
    let (tx_b, _) = broadcast::channel(100);
    let manager_a = CollectionManager::new(dir_a.clone(), tx_a);
    let manager_b = CollectionManager::new(dir_b.clone(), tx_b);

    let col_name = "sync_col";
    let dim = 64;

    // Create the same collection on both nodes
    manager_a
        .create_collection("default_admin", col_name, dim, "l2")
        .await
        .unwrap();
    manager_b
        .create_collection("default_admin", col_name, dim, "l2")
        .await
        .unwrap();

    let col_a = manager_a.get("default_admin", col_name).await.unwrap();
    let col_b = manager_b.get("default_admin", col_name).await.unwrap();

    // Phase 1: Both nodes insert shared vectors (ID 0..49)
    for i in 0u32..50 {
        let vec = vec![f64::from(i) * 0.01; dim as usize];
        col_a
            .insert(&vec, i, HashMap::new(), 0, Durability::Default)
            .await
            .unwrap();
        col_b
            .insert(&vec, i, HashMap::new(), 0, Durability::Default)
            .await
            .unwrap();
    }

    // Phase 2: NETWORK PARTITION — Node A gets IDs 50..74, Node B gets IDs 75..99
    for i in 50u32..75 {
        let vec = vec![f64::from(i) * 0.01; dim as usize];
        col_a
            .insert(&vec, i, HashMap::new(), 0, Durability::Default)
            .await
            .unwrap();
    }
    for i in 75u32..100 {
        let vec = vec![f64::from(i) * 0.01; dim as usize];
        col_b
            .insert(&vec, i, HashMap::new(), 0, Durability::Default)
            .await
            .unwrap();
    }

    // Wait for indexing to settle
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify divergence
    assert_eq!(col_a.count(), 75, "Node A should have 75 vectors");
    assert_eq!(col_b.count(), 75, "Node B should have 75 vectors");
    assert_ne!(
        col_a.state_hash(),
        col_b.state_hash(),
        "State hashes should differ after partition"
    );

    // Phase 3: SYNC — Compare digests, find dirty buckets
    let buckets_a = col_a.buckets();
    let buckets_b = col_b.buckets();
    assert_eq!(buckets_a.len(), 256);
    assert_eq!(buckets_b.len(), 256);

    let mut dirty_a_to_b: Vec<u32> = Vec::new(); // Buckets A has but B doesn't
    let mut dirty_b_to_a: Vec<u32> = Vec::new(); // Buckets B has but A doesn't

    for i in 0..256 {
        if buckets_a[i] != buckets_b[i] {
            // Both directions differ — we need to pull from both
            dirty_a_to_b.push(i as u32);
            dirty_b_to_a.push(i as u32);
        }
    }

    println!("Dirty buckets: {} indices (out of 256)", dirty_a_to_b.len());
    assert!(!dirty_a_to_b.is_empty(), "There should be dirty buckets");
    // With 50 unique IDs spread across 256 buckets, we expect roughly 50 dirty buckets,
    // not all 256.
    assert!(
        dirty_a_to_b.len() < 256,
        "Not ALL buckets should be dirty (efficiency check)"
    );

    // Phase 4: Transfer vectors from dirty buckets
    // Build sets of IDs each node already has (to skip duplicates)
    let b_existing_ids: std::collections::HashSet<u32> = col_b
        .peek(col_b.count().max(1), 0)
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();
    let a_existing_ids: std::collections::HashSet<u32> = col_a
        .peek(col_a.count().max(1), 0)
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();

    // A → B: Pull vectors from A that B is missing
    let a_delta = col_a.peek_buckets(&dirty_a_to_b);
    let mut synced_to_b = 0u32;
    for (id, vec, meta) in &a_delta {
        if b_existing_ids.contains(id) {
            continue; // B already has this vector
        }
        if col_b
            .insert(vec, *id, meta.clone(), 0, Durability::Default)
            .await
            .is_ok()
        {
            synced_to_b += 1;
        }
    }

    // B → A: Pull vectors from B that A is missing
    let b_delta = col_b.peek_buckets(&dirty_b_to_a);
    let mut synced_to_a = 0u32;
    for (id, vec, meta) in &b_delta {
        if a_existing_ids.contains(id) {
            continue; // A already has this vector
        }
        if col_a
            .insert(vec, *id, meta.clone(), 0, Durability::Default)
            .await
            .is_ok()
        {
            synced_to_a += 1;
        }
    }

    println!("Synced A→B: {synced_to_b}, B→A: {synced_to_a}");

    // Phase 5: Verify convergence
    assert_eq!(
        col_a.count(),
        100,
        "Node A should have 100 vectors after sync"
    );
    assert_eq!(
        col_b.count(),
        100,
        "Node B should have 100 vectors after sync"
    );
    assert_eq!(
        col_a.state_hash(),
        col_b.state_hash(),
        "State hashes should match after sync"
    );
    assert_eq!(
        col_a.buckets(),
        col_b.buckets(),
        "All bucket hashes should match after sync"
    );

    println!("✅ Delta Sync test passed: both nodes converged.");

    // Cleanup
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}
