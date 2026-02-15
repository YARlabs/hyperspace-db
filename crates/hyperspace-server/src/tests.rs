use super::manager::CollectionManager;
use tokio::sync::broadcast;
use std::env;
use std::fs;
use uuid::Uuid;
use std::time::Duration;
use std::collections::HashMap;

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
    manager.create_collection(col_name, 128, "l2").await.expect("Create failed");

    let col = manager.get(col_name).await.expect("Collection not found");
    assert_eq!(col.count(), 0);

    // 2. Insert Data & Check Queue
    println!("Inserting 100 vectors...");
    let vec = vec![0.1; 128];
    for i in 0..100 {
        col.insert(&vec, i as u32, HashMap::new(), 0, Durability::Default).expect("Insert failed");
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
        assert!(start.elapsed() <= Duration::from_secs(10), "Indexing timeout. Queue: {}", col.queue_size());
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // assert_eq!(col.count(), 100); 
    // Wait, count() returns usize.
    // Wait, Collection trait count() signature? 
    // Usually usize.
    
    println!("Indexing complete. Count: {}", col.count());
    assert_eq!(col.count(), 100);

    // 3. Rebuild Index
    println!("Triggering rebuild...");
    manager.rebuild_collection(col_name).await.expect("Rebuild failed");

    // After rebuild, verify data
    let col_new = manager.get(col_name).await.expect("Collection not found after rebuild");
    assert_eq!(col_new.count(), 100);
    
    // Verify optimized file exists
    let index_path = tmp_dir.join(col_name).join("index");
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
   
   manager.create_collection("vac_col", 64, "l2").await.unwrap();
   
   let _ = fs::remove_dir_all(&tmp_dir);
}
