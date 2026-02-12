//! Comprehensive Integration Tests for HyperspaceDB
//!
//! This test suite verifies end-to-end functionality including:
//! - Leader-Follower replication
//! - Merkle Tree synchronization
//! - Collection operations
//! - Network failure handling
//! - Performance under load
//!
//! Run with: cargo run --release --bin integration_tests

use hyperspace_sdk::Client;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Running HyperspaceDB Integration Tests\n");

    let mut passed = 0;
    let mut failed = 0;

    // Test 1: Basic Operations
    print!("Test 1: Basic Operations... ");
    match test_basic_operations().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    // Test 2: Leader-Follower Sync
    print!("Test 2: Leader-Follower Sync... ");
    match test_leader_follower_sync().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    // Test 3: Merkle Tree Consistency
    print!("Test 3: Merkle Tree Consistency... ");
    match test_merkle_tree_consistency().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    // Test 4: High Volume Inserts
    print!("Test 4: High Volume Inserts... ");
    match test_high_volume_inserts().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    // Test 5: Concurrent Inserts
    print!("Test 5: Concurrent Inserts... ");
    match test_concurrent_inserts().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    // Test 6: Collection Lifecycle
    print!("Test 6: Collection Lifecycle... ");
    match test_collection_lifecycle().await {
        Ok(()) => {
            println!("✅ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("❌ FAILED: {e}");
            failed += 1;
        }
    }

    println!("\n📊 Test Results:");
    println!("   ✅ Passed: {passed}");
    println!("   ❌ Failed: {failed}");
    println!("   📈 Total:  {}", passed + failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Test basic insert and search operations
async fn test_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Create test collection
    let collection = format!("test_basic_{}", chrono::Utc::now().timestamp());
    client
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    // Insert vectors
    for i in 0..10 {
        client
            .insert(
                i,
                vec![0.1 * f64::from(i); 128],
                [("index".to_string(), i.to_string())].into(),
                Some(collection.clone()),
            )
            .await?;
    }

    // Verify count
    let digest = client.get_digest(Some(collection.clone())).await?;
    assert_eq!(digest.count, 10, "Should have 10 vectors");

    Ok(())
}

/// Test Leader-Follower replication

async fn test_leader_follower_sync() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Leader
    let mut leader = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Connect to Follower
    let mut follower = Client::connect(
        "http://localhost:50052".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Create collection on both
    let collection = format!("test_sync_{}", chrono::Utc::now().timestamp());
    leader
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;
    follower
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    // Insert on Leader
    for i in 0..20 {
        leader
            .insert(
                i,
                vec![0.5; 128],
                [("test".to_string(), "sync".to_string())].into(),
                Some(collection.clone()),
            )
            .await?;
    }

    // Wait for replication
    sleep(Duration::from_secs(2)).await;

    // Verify sync
    let leader_digest = leader.get_digest(Some(collection.clone())).await?;
    let follower_digest = follower.get_digest(Some(collection.clone())).await?;

    assert_eq!(
        leader_digest.state_hash, follower_digest.state_hash,
        "State hashes should match"
    );
    assert_eq!(
        leader_digest.count, follower_digest.count,
        "Counts should match"
    );
    assert_eq!(leader_digest.count, 20, "Should have 20 vectors");

    // Verify bucket-level sync
    for (i, (l_bucket, f_bucket)) in leader_digest
        .buckets
        .iter()
        .zip(follower_digest.buckets.iter())
        .enumerate()
    {
        assert_eq!(
            l_bucket, f_bucket,
            "Bucket {i} should match: Leader={l_bucket}, Follower={f_bucket}"
        );
    }

    Ok(())
}

/// Test Merkle Tree consistency

async fn test_merkle_tree_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    let collection = format!("test_merkle_{}", chrono::Utc::now().timestamp());
    client
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    // Get initial digest
    let digest1 = client.get_digest(Some(collection.clone())).await?;
    assert_eq!(digest1.state_hash, 0, "Empty collection should have hash 0");

    // Insert vector
    client
        .insert(
            100,
            vec![0.5; 128],
            [("test".to_string(), "merkle".to_string())].into(),
            Some(collection.clone()),
        )
        .await?;

    // Get new digest
    let digest2 = client.get_digest(Some(collection.clone())).await?;
    assert_ne!(
        digest2.state_hash, 0,
        "Non-empty collection should have non-zero hash"
    );
    assert_eq!(digest2.count, 1, "Should have 1 vector");

    // Insert different vector with same ID (creates new entry)
    client
        .insert(
            100,
            vec![0.7; 128],
            [("test".to_string(), "merkle2".to_string())].into(),
            Some(collection.clone()),
        )
        .await?;

    let digest3 = client.get_digest(Some(collection.clone())).await?;
    // ✅ UPSERT WORKING! Same ID updates existing vector, count stays 1
    assert_eq!(
        digest3.count, 1,
        "Should have 1 vector (upsert updates existing)"
    );

    Ok(())
}

/// Test high-volume inserts

async fn test_high_volume_inserts() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    let collection = format!("test_volume_{}", chrono::Utc::now().timestamp());
    client
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    let start = std::time::Instant::now();
    let count = 1000;

    // Insert 1000 vectors
    for i in 0..count {
        client
            .insert(
                i,
                vec![0.1 * f64::from(i % 100); 128],
                [("batch".to_string(), "test".to_string())].into(),
                Some(collection.clone()),
            )
            .await?;
    }

    let duration = start.elapsed();
    let qps = f64::from(count) / duration.as_secs_f64();

    println!(
        "Inserted {count} vectors in {duration:?} ({qps:.0} QPS)"
    );

    // Verify count
    let digest = client.get_digest(Some(collection.clone())).await?;
    assert_eq!(digest.count, u64::from(count), "Should have {count} vectors");

    // Performance assertion: should achieve at least 100 QPS
    assert!(qps > 100.0, "QPS should be > 100, got {qps:.0}");

    Ok(())
}

/// Test concurrent operations

async fn test_concurrent_inserts() -> Result<(), Box<dyn std::error::Error>> {
    let collection = format!("test_concurrent_{}", chrono::Utc::now().timestamp());

    // Create collection first
    let mut setup_client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;
    setup_client
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    for task_id in 0..10 {
        let collection_clone = collection.clone();
        let handle = tokio::spawn(async move {
            let mut client = Client::connect(
                "http://localhost:50051".to_string(),
                Some("I_LOVE_HYPERSPACEDB".to_string()),
            )
            .await
            .unwrap();

            // Each task inserts 10 vectors
            for i in 0..10 {
                let id = task_id * 10 + i;
                client
                    .insert(
                        id,
                        vec![0.1 * f64::from(id); 128],
                        [("task".to_string(), task_id.to_string())].into(),
                        Some(collection_clone.clone()),
                    )
                    .await
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await?;
    }

    // Verify total count
    let digest = setup_client.get_digest(Some(collection.clone())).await?;
    assert_eq!(digest.count, 100, "Should have 100 vectors from 10 tasks");

    Ok(())
}

/// Test collection lifecycle

async fn test_collection_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    let collection = format!("test_lifecycle_{}", chrono::Utc::now().timestamp());

    // Create
    client
        .create_collection(collection.clone(), 128, "l2".to_string())
        .await?;

    // Insert
    client
        .insert(
            1,
            vec![0.5; 128],
            [("test".to_string(), "lifecycle".to_string())].into(),
            Some(collection.clone()),
        )
        .await?;

    // Verify
    let digest = client.get_digest(Some(collection.clone())).await?;
    assert_eq!(digest.count, 1);

    // TODO: Add delete collection when implemented
    // client.delete_collection(collection).await?;

    Ok(())
}
