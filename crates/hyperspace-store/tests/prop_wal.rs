use hyperspace_store::wal::{Wal, WalEntry, WalSyncMode};
use proptest::prelude::*;
use std::collections::HashMap;
use tempfile::tempdir;

const D: usize = 4;

fn arb_vector() -> impl Strategy<Value = Vec<f64>> {
    proptest::collection::vec(any::<f64>(), D)
}

fn arb_metadata() -> impl Strategy<Value = HashMap<String, String>> {
    proptest::collection::hash_map(
        "[a-z]", // Single char keys for speed
        "[a-z0-9]", // Single char values
        0..5
    )
}

#[derive(Debug, Clone)]
struct TestEntry {
    id: u32,
    vector: Vec<f64>,
    metadata: HashMap<String, String>,
}

fn arb_entries() -> impl Strategy<Value = Vec<TestEntry>> {
    proptest::collection::vec(
        (any::<u32>(), arb_vector(), arb_metadata()).prop_map(|(id, vector, metadata)| TestEntry {
            id,
            vector,
            metadata,
        }),
        1..50
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))] // Keep it fast

    #[test]
    fn test_wal_append_replay_prop(
        entries in arb_entries()
    ) {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("wal_test.hyp");

        // 1. Write
        {
            let mut wal = Wal::new(&wal_path, WalSyncMode::Async).unwrap();
            for entry in &entries {
                wal.append(entry.id, &entry.vector, &entry.metadata).unwrap();
            }
            wal.sync().unwrap();
        }

        // 2. Replay & verify
        let mut replayed = Vec::new();
        Wal::replay(&wal_path, |entry| {
            let WalEntry::Insert { id, vector, metadata } = entry;
            replayed.push(TestEntry { id, vector, metadata });
        }).unwrap();

        assert_eq!(replayed.len(), entries.len());
        for (i, (original, replayed)) in entries.iter().zip(replayed.iter()).enumerate() {
            assert_eq!(original.id, replayed.id, "Mismatch at index {i}");
            assert_eq!(original.vector, replayed.vector);
            assert_eq!(original.metadata, replayed.metadata);
        }
    }

    #[test]
    fn test_wal_corruption_recovery(
        entries in arb_entries(),
        cut_bytes in 1usize..100usize
    ) {
        let dir = tempdir().unwrap();
        let wal_path = dir.path().join("wal_corrupt.hyp");

        // 1. Write ALL entries
        {
            let mut wal = Wal::new(&wal_path, WalSyncMode::Async).unwrap();
            for entry in &entries {
                wal.append(entry.id, &entry.vector, &entry.metadata).unwrap();
            }
            wal.sync().unwrap();
        }

        // 2. Corrupt the file (truncate last `cut_bytes`)
        let file_len = std::fs::metadata(&wal_path).unwrap().len();
        if file_len > cut_bytes as u64 {
            let file = std::fs::OpenOptions::new().write(true).open(&wal_path).unwrap();
            file.set_len(file_len - cut_bytes as u64).unwrap();
        }

        // 3. Try to Replay
        let mut replayed = Vec::new();
        let res = Wal::replay(&wal_path, |entry| {
             let WalEntry::Insert { id, vector, metadata } = entry;
             replayed.push(TestEntry { id, vector, metadata });
        });

        assert!(res.is_ok(), "Replay failed on corrupted WAL: {:?}", res.err());
        
        // Check content consistency (prefix match)
        let common_len = replayed.len();
        for i in 0..common_len {
            assert_eq!(entries[i].id, replayed[i].id, "Mismatch at index {i}");
            assert_eq!(entries[i].vector, replayed[i].vector);
            assert_eq!(entries[i].metadata, replayed[i].metadata);
        }
    }
}
