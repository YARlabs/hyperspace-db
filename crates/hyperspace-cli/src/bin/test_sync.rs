use hyperspace_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Leader-Follower Replication");

    // Connect to Leader
    let mut leader = Client::connect(
        "http://localhost:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Create a fresh collection with unique name
    let collection_name = format!(
        "sync_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    println!("📦 Creating test collection: {}...", collection_name);
    leader
        .create_collection(collection_name.clone(), 128, "l2".to_string())
        .await?;

    // Also create on Follower (since CREATE is not replicated yet)
    let mut follower = Client::connect(
        "http://localhost:50052".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    follower
        .create_collection(collection_name.clone(), 128, "l2".to_string())
        .await?;
    println!("✅ Collections created");

    // Insert test data
    println!("📝 Inserting test vector to Leader...");
    leader
        .insert(
            999,
            vec![0.5; 128],
            [("test".to_string(), "cluster_demo".to_string())].into(),
            Some(collection_name.clone()),
        )
        .await?;
    println!("✅ Insert successful");

    // Get Leader digest
    let leader_digest = leader.get_digest(Some(collection_name.clone())).await?;
    println!("\n📊 Leader Digest:");
    println!("   Logical Clock: {}", leader_digest.logical_clock);
    println!("   State Hash: {}", leader_digest.state_hash);
    println!("   Count: {}", leader_digest.count);

    // Wait for replication
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Get Follower digest
    let follower_digest = follower.get_digest(Some(collection_name.clone())).await?;
    println!("\n📊 Follower Digest:");
    println!("   Logical Clock: {}", follower_digest.logical_clock);
    println!("   State Hash: {}", follower_digest.state_hash);
    println!("   Count: {}", follower_digest.count);

    // Verify sync
    println!("\n🔍 Verification:");
    if leader_digest.state_hash == follower_digest.state_hash {
        println!("🎉 SUCCESS! Leader and Follower are in sync!");
        println!("   ✓ State hashes match: {}", leader_digest.state_hash);
        println!("   ✓ Both have {} vectors", leader_digest.count);
    } else {
        println!("❌ MISMATCH! Data drift detected.");
        println!("   Leader hash:   {}", leader_digest.state_hash);
        println!("   Follower hash: {}", follower_digest.state_hash);
    }

    Ok(())
}
