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
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    // 1. Create Collection
    let col_name = "test_rebuild";
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: 128,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", col_name, schema)
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
            if col.queue_size() == 0 && col.write_buffer_size() == 0 {
                break;
            }
            assert!(
                start.elapsed() <= Duration::from_secs(10),
                "Indexing timeout. Queue: {}, Buffer: {}",
                col.queue_size(),
                col.write_buffer_size()
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

    // Verify optimized file exists (either in collection root or shard directories)
    let folder_name = format!("default_admin_{col_name}");
    let col_path = tmp_dir.join(folder_name);
    let index_exists = col_path.join("index.snap").exists()
        || col_path.join("shard_0").join("index.snap").exists();
    assert!(index_exists);

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
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: 64,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", "vac_col", schema)
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
    let (etx_a, _) = broadcast::channel(100);
    let (etx_b, _) = broadcast::channel(100);
    let manager_a = CollectionManager::new(dir_a.clone(), tx_a, etx_a);
    let manager_b = CollectionManager::new(dir_b.clone(), tx_b, etx_b);

    let col_name = "sync_col";
    let dim = 64;

    // Create the same collection on both nodes
    let schema_a = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    let schema_b = schema_a.clone();
    manager_a
        .create_collection("default_admin", col_name, schema_a)
        .await
        .unwrap();
    manager_b
        .create_collection("default_admin", col_name, schema_b)
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

#[tokio::test]
async fn test_opq_collection_lifecycle_and_search() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_opq_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "test_opq_col";
    let dim = 128;
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };

    manager
        .create_collection_with_quantization(
            "default_admin",
            col_name,
            schema,
            Some("opq".to_string()),
        )
        .await
        .expect("Create OPQ collection failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection must be retrieved");

    assert_eq!(
        col.quantization_mode(),
        hyperspace_core::QuantizationMode::OPQ
    );

    // Insert 50 vectors
    for i in 0..50u32 {
        let mut v = vec![0.0f64; dim as usize];
        v[0] = f64::from(i) * 0.1;
        v[1] = 1.0;
        let mut meta = HashMap::new();
        meta.insert("idx".to_string(), i.to_string());
        col.insert(&v, i, meta, (i + 1) as u64, Durability::Default)
            .await
            .expect("Insert failed");
    }

    // Wait for indexing to complete
    let start = std::time::Instant::now();
    loop {
        if col.queue_size() == 0 && col.write_buffer_size() == 0 {
            break;
        }
        assert!(start.elapsed() <= Duration::from_secs(10));
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert_eq!(col.count(), 50);

    // Search
    let mut query = vec![0.0f64; dim as usize];
    query[0] = 1.0; // closest to i=10 (10 * 0.1 = 1.0)
    query[1] = 1.0;
    let params = hyperspace_core::SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let res = col
        .search(&query, &HashMap::new(), &[], &params)
        .await
        .expect("Search must succeed");

    assert!(!res.is_empty());
    let found_ids: Vec<u32> = res.iter().map(|r| r.0).collect();
    assert!(
        found_ids.contains(&10),
        "Search should locate vector 10 as nearest neighbor, got: {found_ids:?}"
    );

    // Drop original manager and re-open from disk
    drop(col);
    drop(manager);

    let (tx2, _rx2) = broadcast::channel(100);
    let (etx2, _) = broadcast::channel(100);
    let manager2 = CollectionManager::new(tmp_dir.clone(), tx2, etx2);

    let reloaded_col = manager2
        .get("default_admin", col_name)
        .await
        .expect("Collection must reload from disk");

    assert_eq!(
        reloaded_col.quantization_mode(),
        hyperspace_core::QuantizationMode::OPQ
    );
    assert_eq!(reloaded_col.count(), 50);

    let res_reloaded = reloaded_col
        .search(&query, &HashMap::new(), &[], &params)
        .await
        .expect("Search on reloaded collection must succeed");

    assert!(!res_reloaded.is_empty());
    let reloaded_ids: Vec<u32> = res_reloaded.iter().map(|r| r.0).collect();
    assert!(
        reloaded_ids.contains(&10),
        "Reloaded search should locate vector 10, got: {reloaded_ids:?}"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_pq_collection_lifecycle_and_search() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_pq_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "test_pq_col";
    let dim = 128;
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };

    manager
        .create_collection_with_quantization(
            "default_admin",
            col_name,
            schema,
            Some("pq".to_string()),
        )
        .await
        .expect("Create PQ collection failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection must be retrieved");

    assert_eq!(
        col.quantization_mode(),
        hyperspace_core::QuantizationMode::ProductQuantization
    );

    // Insert 50 vectors
    for i in 0..50u32 {
        let mut v = vec![0.0f64; dim as usize];
        v[0] = f64::from(i) * 0.05;
        v[1] = 0.5;
        let mut meta = HashMap::new();
        meta.insert("idx".to_string(), i.to_string());
        col.insert(&v, i, meta, (i + 1) as u64, Durability::Default)
            .await
            .expect("Insert failed");
    }

    // Wait for indexing to complete
    let start = std::time::Instant::now();
    loop {
        if col.queue_size() == 0 && col.write_buffer_size() == 0 {
            break;
        }
        assert!(start.elapsed() <= Duration::from_secs(10));
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert_eq!(col.count(), 50);

    // Search
    let mut query = vec![0.0f64; dim as usize];
    query[0] = 0.05 * 5.0; // closest to i=5
    query[1] = 0.5;
    let params = hyperspace_core::SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let res = col
        .search(&query, &HashMap::new(), &[], &params)
        .await
        .expect("Search must succeed");

    assert!(!res.is_empty());
    let found_ids: Vec<u32> = res.iter().map(|r| r.0).collect();
    assert!(
        found_ids.contains(&5),
        "Search should locate vector 5 as nearest neighbor, got: {found_ids:?}"
    );

    // Drop original manager and re-open from disk to test persistence
    drop(col);
    drop(manager);

    let (tx2, _rx2) = broadcast::channel(100);
    let (etx2, _) = broadcast::channel(100);
    let manager2 = CollectionManager::new(tmp_dir.clone(), tx2, etx2);

    let reloaded_col = manager2
        .get("default_admin", col_name)
        .await
        .expect("PQ collection must reload from disk");

    assert_eq!(
        reloaded_col.quantization_mode(),
        hyperspace_core::QuantizationMode::ProductQuantization
    );
    assert_eq!(reloaded_col.count(), 50);

    let res_reloaded = reloaded_col
        .search(&query, &HashMap::new(), &[], &params)
        .await
        .expect("Search on reloaded PQ collection must succeed");

    assert!(!res_reloaded.is_empty());
    let reloaded_ids: Vec<u32> = res_reloaded.iter().map(|r| r.0).collect();
    assert!(
        reloaded_ids.contains(&5),
        "Reloaded PQ search should locate vector 5, got: {reloaded_ids:?}"
    );

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_tenant_isolation_list_and_access() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_test_tenant_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let make_schema = || hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "cosine".to_string(),
            full_dimension: 32,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };

    // 1. default_admin creates playbooks
    manager
        .create_collection("default_admin", "soc_playbooks", make_schema())
        .await
        .expect("Admin collection failed");
    manager
        .create_collection("default_admin", "fintech_playbook", make_schema())
        .await
        .expect("Admin collection failed");

    // 2. Tenant A creates private collections
    let tenant_a = "0x4E41e9f528aCfcDAD396B89438E2083Cefc0E3a4";
    manager
        .create_collection(tenant_a, "my_portfolio", make_schema())
        .await
        .expect("Tenant A collection failed");

    // 3. Tenant B creates private collections
    let tenant_b = "0x9876543210fedcba";
    manager
        .create_collection(tenant_b, "confidential_data", make_schema())
        .await
        .expect("Tenant B collection failed");

    // 4. Verify list isolation
    let list_a = manager.list(tenant_a);
    assert_eq!(list_a, vec!["my_portfolio".to_string()]);

    let list_b = manager.list(tenant_b);
    assert_eq!(list_b, vec!["confidential_data".to_string()]);

    let list_admin = manager.list("default_admin");
    assert_eq!(
        list_admin,
        vec!["fintech_playbook".to_string(), "soc_playbooks".to_string()]
    );

    // 5. Verify direct cross-tenant get returns None
    assert!(manager.get(tenant_a, "soc_playbooks").await.is_none());
    assert!(manager.get(tenant_a, "confidential_data").await.is_none());
    assert!(manager.get(tenant_b, "my_portfolio").await.is_none());

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_large_uint32_vector_ids_no_oom() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "test_large_ids";
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "cosine".to_string(),
            full_dimension: 16,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", col_name, schema)
        .await
        .expect("Create collection failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection not found");

    // Insert large IDs near u32 boundaries and the specific reproducer ID 3_904_440_873
    let test_ids: Vec<u32> = vec![
        3_904_440_873,
        u32::MAX - 1,
        2_147_483_648, // 2^31, would be negative in signed i32
        4_000_000_000,
        42,
    ];

    let vec = vec![0.25; 16];
    for &id in &test_ids {
        col.insert(&vec, id, HashMap::new(), 0, Durability::Default)
            .await
            .expect("Insert with large uint32 id must succeed without OOM");
    }

    assert_eq!(col.count(), test_ids.len());

    // Search query
    let search_params = hyperspace_core::SearchParams {
        top_k: 5,
        ef_search: 32,
        ..Default::default()
    };
    let results = col
        .search(&vec, &HashMap::new(), &[], &search_params)
        .await
        .expect("Search must succeed");

    assert!(!results.is_empty());
    let returned_ids: Vec<u32> = results.iter().map(|(id, _, _, _)| *id).collect();
    assert!(
        returned_ids.contains(&3_904_440_873),
        "Must find vector with ID 3_904_440_873"
    );

    // Verify deletion works cleanly
    for &id in &test_ids {
        col.delete(id).expect("Delete must succeed");
    }

    // Cleanup
    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_superkmeans_hierarchical_memory_clustering_efficiency() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_cluster_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "agent_memory_clustering";
    let dim = 128;
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim as u32,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", col_name, schema)
        .await
        .expect("Create failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection exists");

    // Generate 200 vectors in 4 distinct clusters: 50 each
    let centers = [-2.0f64, -1.0f64, 1.0f64, 2.0f64];
    let mut id_counter = 1u32;
    for &center in &centers {
        for _ in 0..50 {
            let mut v = vec![center; dim];
            v[0] += (id_counter as f64 % 5.0) * 0.01;
            col.insert(&v, id_counter, HashMap::new(), 0, Durability::Default)
                .await
                .expect("Insert failed");
            id_counter += 1;
        }
    }

    // Wait for indexing queue to clear
    let start_wait = std::time::Instant::now();
    while col.queue_size() > 0 || col.write_buffer_size() > 0 {
        if start_wait.elapsed() > Duration::from_secs(5) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Measure SuperKMeans hierarchical clustering efficiency
    let cluster_start = std::time::Instant::now();
    let clusters = col
        .graph_clusters(0, 5, 8, 500)
        .expect("Clustering must succeed");
    let cluster_elapsed = cluster_start.elapsed();
    println!(
        "⏱️ SuperKMeans Hierarchical Clustering Time: {:?} (found {} clusters)",
        cluster_elapsed,
        clusters.len()
    );

    assert!(!clusters.is_empty(), "Must discover semantic clusters");
    for cluster in &clusters {
        assert!(
            cluster.len() >= 5,
            "Every cluster must meet min_cluster_size"
        );
    }
    assert!(
        cluster_elapsed.as_millis() < 3000,
        "Hierarchical clustering must be faster than 3000ms"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_superkmeans_semantic_compactor_and_meta_router() {
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_compact_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "compaction_collection";
    let dim = 128;
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim as u32,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", col_name, schema)
        .await
        .expect("Create failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection exists");

    // Insert 200 vectors so that each of the 8 shards receives >= 16 vectors
    for i in 1..=200 {
        let center = if i <= 70 {
            1.0
        } else if i <= 140 {
            5.0
        } else {
            10.0
        };
        let mut v = vec![center; dim];
        v[1] += (i as f64) * 0.001;
        col.insert(&v, i as u32, HashMap::new(), 0, Durability::Default)
            .await
            .expect("Insert failed");
    }

    // Wait for indexing
    let start_wait = std::time::Instant::now();
    while col.queue_size() > 0 || col.write_buffer_size() > 0 {
        if start_wait.elapsed() > Duration::from_secs(5) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Trigger semantic compaction into 3 Voronoi chunks
    let comp_start = std::time::Instant::now();
    let comp_res = col
        .compact_semantic_chunks(3)
        .expect("Semantic compaction must succeed");
    let comp_elapsed = comp_start.elapsed();
    println!(
        "⏱️ Semantic Compaction Result: {} in {:?}",
        comp_res, comp_elapsed
    );

    assert!(
        comp_res.contains("Semantic compaction completed"),
        "Report must confirm completion"
    );
    assert!(
        comp_res.contains("Voronoi chunks"),
        "Report must confirm Voronoi partitions"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test]
async fn test_billion_scale_64bit_id_addressing_and_routing() {
    std::env::set_var("HS_SHARDS", "8");
    let uuid = Uuid::new_v4();
    let tmp_dir = env::temp_dir().join(format!("hyperspace_64bit_test_{uuid}"));
    fs::create_dir_all(&tmp_dir).unwrap();

    let (tx, _rx) = broadcast::channel(100);
    let (etx, _) = broadcast::channel(100);
    let manager = CollectionManager::new(tmp_dir.clone(), tx, etx);

    let col_name = "billion_scale_collection";
    let dim = 32;
    let schema = hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: "l2".to_string(),
            full_dimension: dim as u32,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };
    manager
        .create_collection("default_admin", col_name, schema)
        .await
        .expect("Create failed");

    let col = manager
        .get("default_admin", col_name)
        .await
        .expect("Collection exists");

    // IDs exceeding 32-bit limit (4,294,967,295):
    let id_5b = 5_000_000_000u64;
    let id_12b = 12_345_678_901u64;
    let id_21b = 21_200_000_000u64;

    let mut meta_5b = HashMap::new();
    meta_5b.insert("scale".to_string(), "5B".to_string());
    let mut meta_12b = HashMap::new();
    meta_12b.insert("scale".to_string(), "12.3B".to_string());
    let mut meta_21b = HashMap::new();
    meta_21b.insert("scale".to_string(), "21.2B".to_string());

    let vec_5b = vec![1.0; dim];
    let vec_12b = vec![5.0; dim];
    let vec_21b = vec![10.0; dim];

    // 1. Insert 64-bit points
    col.insert_u64(&vec_5b, id_5b, meta_5b.clone(), 0, Durability::Default)
        .await
        .expect("Insert 5B must succeed");
    col.insert_u64(&vec_12b, id_12b, meta_12b.clone(), 0, Durability::Default)
        .await
        .expect("Insert 12.3B must succeed");
    col.insert_u64(&vec_21b, id_21b, meta_21b.clone(), 0, Durability::Default)
        .await
        .expect("Insert 21.2B must succeed");

    // 2. Test get_points_u64 retrieval
    let points = col.get_points_u64(&[id_5b, id_12b, id_21b]);
    assert_eq!(
        points.len(),
        3,
        "All 3 billion-scale points must be retrievable"
    );

    let p_5b = points.iter().find(|p| p.0 == id_5b).expect("5B found");
    assert_eq!(p_5b.2.get("scale").map(|s| s.as_str()), Some("5B"));
    assert_eq!(p_5b.1, vec_5b);

    let p_21b = points.iter().find(|p| p.0 == id_21b).expect("21.2B found");
    assert_eq!(p_21b.2.get("scale").map(|s| s.as_str()), Some("21.2B"));
    assert_eq!(p_21b.1, vec_21b);

    // 3. Test search_u64 with 64-bit top-K results
    let search_params = hyperspace_core::SearchParams {
        top_k: 2,
        ef_search: 32,
        ..Default::default()
    };
    let query_near_21b = vec![9.99; dim];
    let search_results = col
        .search_u64(&query_near_21b, &HashMap::new(), &[], &search_params)
        .await
        .expect("Search must succeed");

    assert!(
        !search_results.is_empty(),
        "Search results must not be empty"
    );
    assert_eq!(
        search_results[0].0, id_21b,
        "Top search candidate must correctly return 64-bit ID 21,200,000,000"
    );

    // 4. Test delete_u64
    col.delete_u64(id_12b)
        .expect("Delete 12.3B point must succeed");
    let after_delete = col.get_points_u64(&[id_12b]);
    assert!(
        after_delete.is_empty(),
        "Deleted 64-bit point must not be returned"
    );

    // Other points remain intact
    let remaining = col.get_points_u64(&[id_5b, id_21b]);
    assert_eq!(remaining.len(), 2, "Remaining 64-bit points must persist");

    // 5. Test batch insert with 64-bit IDs
    let batch = vec![
        (vec![2.0; dim], 21_200_000_001u64, HashMap::new()),
        (vec![2.1; dim], 21_200_000_002u64, HashMap::new()),
    ];
    col.insert_batch_u64(batch, 0, Durability::Default)
        .await
        .expect("Batch insert 64-bit must succeed");

    let batch_retrieved = col.get_points_u64(&[21_200_000_001u64, 21_200_000_002u64]);
    assert_eq!(
        batch_retrieved.len(),
        2,
        "Batch 64-bit points must be retrieved"
    );

    let _ = fs::remove_dir_all(&tmp_dir);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_v4_2_0_plugins_and_multi_metric_engine() {
    let tmp_dir = std::env::temp_dir().join(format!(
        "hs_test_v4_2_0_pdk_{}_{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let (rep_tx, _) = broadcast::channel(64);
    let (evt_tx, _) = broadcast::channel(64);
    let manager = CollectionManager::new(tmp_dir.clone(), rep_tx, evt_tx);

    // 1. Verify standard built-in (embed, shield) and external (git_ast, eco) plugins are registered
    let manifests = manager.plugins.list_manifests();
    assert!(
        manifests.len() >= 3,
        "Expected at least 3 plugins, got {}",
        manifests.len()
    );
    assert!(manifests.iter().any(|m| m.name == "embed"));
    assert!(manifests.iter().any(|m| m.name == "shield"));
    assert!(manifests.iter().any(|m| m.name == "git_ast"));

    let simple_schema = |dim: u32, metric: &str| hyperspace_proto::hyperspace::CollectionSchema {
        components: vec![hyperspace_proto::hyperspace::VectorComponent {
            name: "default".to_string(),
            metric: metric.to_string(),
            full_dimension: dim,
            weight: 1.0,
        }],
        cascade_pipeline: vec![],
        encrypted: Some(false),
    };

    // 2. Test hs-embed end-to-end on a collection
    manager
        .create_collection("test_user", "docs", simple_schema(64, "cosine"))
        .await
        .expect("create docs collection");

    let embed_res = manager
        .call_extension(
            "test_user",
            Some("docs"),
            "embed",
            "ingest_text",
            serde_json::json!({
                "text": "# HyperspaceDB v4.2.0\nZero-Python in-engine embedding.\n\n## Architecture\nDeterministic manifold projection and KAG linking.",
                "start_id": 100,
                "max_chars": 80,
                "link_sequential": true
            }),
        )
        .await
        .expect("hs-embed ingest_text must succeed");
    let chunks_inserted = embed_res["chunks_inserted"].as_u64().unwrap_or(0);
    assert!(chunks_inserted >= 2, "Expected at least 2 markdown chunks");

    let docs_col = manager.get("test_user", "docs").await.unwrap();
    assert_eq!(docs_col.count() as u64, chunks_inserted);

    // 3. Test hs-git-ast end-to-end on a codebase collection
    manager
        .create_collection("test_user", "code_kag", simple_schema(64, "l2"))
        .await
        .expect("create code_kag collection");

    let ast_res = manager
        .call_extension(
            "test_user",
            Some("code_kag"),
            "git_ast",
            "ingest_files",
            serde_json::json!({
                "base_id": 500,
                "files": [
                    {
                        "path": "src/engine.rs",
                        "content": "pub trait Plugin {}\npub struct Host;\nimpl Plugin for Host {}\npub fn init() { run(); }\npub fn run() {}"
                    }
                ]
            }),
        )
        .await
        .expect("hs-git-ast ingest_files must succeed");
    assert!(ast_res["symbols_inserted"].as_u64().unwrap_or(0) >= 4);
    assert!(ast_res["edges_linked"].as_u64().unwrap_or(0) >= 4);

    // 4. Test hs-shield configure + isometric vector rotation
    let _ = manager
        .call_extension(
            "test_user",
            Some("code_kag"),
            "shield",
            "configure",
            serde_json::json!({
                "collection": "code_kag",
                "secret_key": "enterprise-ciso-key-2026"
            }),
        )
        .await
        .expect("hs-shield configure must succeed");

    // 5. Verify multi-metric collections instantiate and search cleanly across new metrics
    for metric in ["dot", "l1", "linf", "angular", "jaccard", "hamming"] {
        let cname = format!("col_{metric}");
        manager
            .create_collection("test_user", &cname, simple_schema(16, metric))
            .await
            .unwrap_or_else(|e| panic!("Failed to create collection for metric {metric}: {e}"));
        let c = manager.get("test_user", &cname).await.unwrap();
        c.insert(&[0.5; 16], 1, HashMap::new(), 1, Durability::Default)
            .await
            .unwrap();
        c.insert(&[0.25; 16], 2, HashMap::new(), 2, Durability::Default)
            .await
            .unwrap();
        let hits = c
            .search(
                &[0.5; 16],
                &HashMap::new(),
                &[],
                &hyperspace_core::SearchParams {
                    top_k: 2,
                    ef_search: 16,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(
            !hits.is_empty(),
            "Metric {metric} search must return results"
        );
    }

    let _ = fs::remove_dir_all(&tmp_dir);
}
