use hyperspace_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Comprehensive Leader-Follower Sync Test");

    // Connect to Leader
    let mut leader = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Create a fresh collection with unique name
    let collection_name = format!(
        "full_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    println!("📦 Creating test collection: {}...", collection_name);
    leader
        .create_collection(collection_name.clone(), 128, "l2".to_string())
        .await?;

    // Also create on Follower
    let mut follower = Client::connect(
        "http://localhost:50052".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    follower
        .create_collection(collection_name.clone(), 128, "l2".to_string())
        .await?;
    println!("✅ Collections created\n");

    // Test 1: Insert multiple vectors
    println!("📝 Test 1: Inserting 10 vectors...");
    for i in 0..10 {
        leader
            .insert(
                i,
                vec![0.1 * i as f64; 128],
                [("index".to_string(), i.to_string())].into(),
                Some(collection_name.clone()),
            )
            .await?;
    }
    println!("✅ Inserted 10 vectors");

    // Wait for replication
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Check digests
    let leader_digest = leader.get_digest(Some(collection_name.clone())).await?;
    let follower_digest = follower.get_digest(Some(collection_name.clone())).await?;

    println!("\n📊 After 10 inserts:");
    println!(
        "   Leader:   hash={}, count={}",
        leader_digest.state_hash, leader_digest.count
    );
    println!(
        "   Follower: hash={}, count={}",
        follower_digest.state_hash, follower_digest.count
    );

    assert_eq!(
        leader_digest.state_hash, follower_digest.state_hash,
        "Hashes should match after 10 inserts"
    );
    assert_eq!(leader_digest.count, 10, "Leader should have 10 vectors");
    assert_eq!(follower_digest.count, 10, "Follower should have 10 vectors");
    println!("   ✅ Test 1 PASSED");

    // Test 2: Insert more vectors
    println!("\n📝 Test 2: Inserting 5 more vectors...");
    for i in 10..15 {
        leader
            .insert(
                i,
                vec![0.2 * i as f64; 128],
                [("index".to_string(), i.to_string())].into(),
                Some(collection_name.clone()),
            )
            .await?;
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let leader_digest2 = leader.get_digest(Some(collection_name.clone())).await?;
    let follower_digest2 = follower.get_digest(Some(collection_name.clone())).await?;

    println!("\n📊 After 15 total inserts:");
    println!(
        "   Leader:   hash={}, count={}",
        leader_digest2.state_hash, leader_digest2.count
    );
    println!(
        "   Follower: hash={}, count={}",
        follower_digest2.state_hash, follower_digest2.count
    );

    assert_eq!(
        leader_digest2.state_hash, follower_digest2.state_hash,
        "Hashes should match after 15 inserts"
    );
    assert_eq!(leader_digest2.count, 15, "Leader should have 15 vectors");
    assert_eq!(
        follower_digest2.count, 15,
        "Follower should have 15 vectors"
    );
    println!("   ✅ Test 2 PASSED");

    // Test 3: Verify bucket-level sync
    println!("\n📝 Test 3: Verifying bucket-level sync...");
    let mut matching_buckets = 0;
    for (i, (l_bucket, f_bucket)) in leader_digest2
        .buckets
        .iter()
        .zip(follower_digest2.buckets.iter())
        .enumerate()
    {
        if l_bucket == f_bucket {
            matching_buckets += 1;
        } else {
            println!(
                "   ⚠️  Bucket {} mismatch: Leader={}, Follower={}",
                i, l_bucket, f_bucket
            );
        }
    }

    println!("   Matching buckets: {}/256", matching_buckets);
    assert_eq!(matching_buckets, 256, "All buckets should match");
    println!("   ✅ Test 3 PASSED");

    println!("\n🎉 ALL TESTS PASSED! System is 100% synchronized!");
    println!("   ✓ Hash consistency verified");
    println!("   ✓ Count consistency verified");
    println!("   ✓ Bucket-level consistency verified");
    println!("   ✓ Multi-insert replication verified");

    Ok(())
}
