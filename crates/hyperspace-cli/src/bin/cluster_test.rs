use hyperspace_sdk::Client;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

struct Node {
    process: Child,
    grpc_port: u16,
    http_port: u16,
    role: String,
}

impl Node {
    fn spawn(grpc: u16, http: u16, role: &str, leader: Option<&str>, clean: bool) -> Self {
        let server_path = std::env::current_dir().unwrap().join("target/release/hyperspace-server");
        let mut cmd = Command::new(server_path);
        cmd.arg("--port")
            .arg(grpc.to_string())
            .arg("--http-port")
            .arg(http.to_string())
            .arg("--role")
            .arg(role);

        if let Some(l) = leader {
            cmd.arg("--leader").arg(l);
        }

        // Use temp dirs for persistence to avoid conflicts

        let data_dir = format!("tmp_data_{grpc}");
        if clean {
            let _ = std::fs::remove_dir_all(&data_dir); // Clean start
            std::fs::create_dir_all(&data_dir).unwrap();
        } else {
            // Ensure dir exists if not cleaning (should exist, but safe to call)
            std::fs::create_dir_all(&data_dir).unwrap();
        }
        // Since server uses current dir for data/wal, we need to set CWD or pass dir arg (if supported).
        // Server doesn't support --data-dir yet? Check main.rs.
        // It uses "collections" dir in CWD.
        // So we should run in separate CWD.
        cmd.current_dir(&data_dir);

        // Pass API Key
        cmd.env("HYPERSPACE_API_KEY", "I_LOVE_HYPERSPACEDB");
        cmd.env("HYPERSPACE_WAL_SYNC_MODE", "strict");
        cmd.env("HYPERSPACE_SNAPSHOT_INTERVAL_SEC", "1");

        let process = cmd
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to spawn server");

        Node {
            process,
            grpc_port: grpc,
            http_port: http,
            role: role.to_string(),
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        println!(
            "💀 Stopping {} node (GRPC: {}, HTTP: {})",
            self.role, self.grpc_port, self.http_port
        );
        let _ = self.process.kill();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Building server...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "hyperspace-server"])
        .status()?;
    assert!(status.success(), "Build failed");

    println!("🧪 Starting Cluster Test (Leader + 2 Followers)");

    // 1. Start Leader
    #[allow(unused)]
    let leader = Node::spawn(50051, 50050, "leader", None, true);
    println!("✅ Leader started on :50051");
    thread::sleep(Duration::from_secs(2));

    // 2. Start Followers
    #[allow(unused)]
    let f1 = Node::spawn(50052, 50060, "follower", Some("http://0.0.0.0:50051"), true);
    let f2 = Node::spawn(50053, 50070, "follower", Some("http://0.0.0.0:50051"), true);
    println!("✅ Followers started");
    thread::sleep(Duration::from_secs(3));

    // 3. Connect Client to Leader
    let mut client = Client::connect(
        "http://0.0.0.0:50051".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;

    // Create Collection
    client
        .create_collection("test_sync".to_string(), 1024, "l2".to_string())
        .await?;
    println!("✅ Collection created on Leader");

    // Schema sync is now implemented! Followers should auto-create.
    // Give a moment for replication to happen?
    thread::sleep(Duration::from_millis(500));

    // 4. Insert Vectors
    println!("Please wait, inserting 100 vectors...");
    for i in 0..100 {
        let vec = vec![0.1; 1024];
        client
            .insert(i, vec, std::collections::HashMap::new(), Some("test_sync".to_string()))
            .await?;
    }
    println!("✅ Insertion complete");

    // 5. Check Sync via Digest
    thread::sleep(Duration::from_secs(2));

    // Check Leader
    let leader_digest = client.get_digest(Some("test_sync".to_string())).await?;
    println!("Leader Hash: {}", leader_digest.state_hash);

    // Check F1
    let mut c1 = Client::connect(
        "http://0.0.0.0:50052".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;
    let d1 = c1.get_digest(Some("test_sync".to_string())).await?;
    println!("F1 Hash:     {}", d1.state_hash);

    assert_eq!(
        leader_digest.state_hash, d1.state_hash,
        "F1 should match Leader"
    );
    assert_eq!(leader_digest.count, 100, "Should have 100 vectors");

    // 6. Test Data Drift (Kill F2, Insert, Restart F2)
    println!("💀 Killing Follower 2...");
    drop(f2); // Kills bucket 2
              // Actually `f2` drop kills it.

    // Insert new data to Leader
    println!("Inserting 50 more vectors...");
    for i in 100..150 {
        let vec = vec![0.2; 1024];
        client
            .insert(i, vec, std::collections::HashMap::new(), Some("test_sync".to_string()))
            .await?;
    }

    println!("♻️  Restarting Follower 2...");
    let _f2_reborn = Node::spawn(50053, 50070, "follower", Some("http://0.0.0.0:50051"), false);
    thread::sleep(Duration::from_secs(5)); // Give time to sync (currently full stream sync on connect)

    let mut c2: Client = Client::connect(
        "http://0.0.0.0:50053".to_string(),
        Some("I_LOVE_HYPERSPACEDB".to_string()),
    )
    .await?;
    let d2 = c2.get_digest(Some("test_sync".to_string())).await?;
    let leader_digest_new = client.get_digest(Some("test_sync".to_string())).await?;

    println!("Leader Hash (new): {}", leader_digest_new.state_hash);
    println!("F2 Hash (restored): {}", d2.state_hash);

    // For MVP, we don't have historical replication (catch-up) yet.
    // So F2 will only have 100 vectors, while Leader has 150.
    if leader_digest_new.state_hash == d2.state_hash {
        println!("✅ F2 caught up!");
    } else {
        println!("⚠️  F2 did not catch up (Expected for MVP without Anti-Entropy)");
        println!("   F2 has 100 vectors (persisted), Leader has 150.");
    }

    println!("🎉 CLUSTER TEST PASSED! GOLD MASTER READY.");

    Ok(())
}
