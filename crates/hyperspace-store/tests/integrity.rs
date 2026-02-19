use hyperspace_store::wal::{Wal, WalSyncMode};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};

#[test]
fn test_wal_partial_write_truncation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wal_trunc.log");

    {
        let mut wal = Wal::new(&path, WalSyncMode::Async).unwrap();
        // Write 3 entries
        // Entry size estimate:
        // Header: 1(Magic) + 4(Len) + 4(CRC) = 9 bytes
        // Payload: 1(Op) + 4(Id) + 4(VecLen) + (VecLen*8) + 4(MetaCount) + MetaData
        // Let VecLen = 10. Vector = 80 bytes.
        // Meta = 0.
        // Payload overhead = 1+4+4+4 = 13.
        // Total Payload = 93 bytes.
        // Total Record = 102 bytes.
        for i in 0..3 {
            let vec = vec![0.5f64; 10];
            wal.append(i, &vec, &HashMap::new(), 0).unwrap();
        }
    }

    // File size should be ~306 bytes.
    let full_len = fs::metadata(&path).unwrap().len();
    assert!(full_len > 300);

    // Corrupt the file by truncating mid-way through the 3rd record
    let truncated_len = full_len - 50;
    let file = OpenOptions::new().write(true).open(&path).unwrap();
    file.set_len(truncated_len).unwrap();
    drop(file);

    // Replay. Should read 2 records successfully, then detect potential partial read/EOF and stop.
    // Our implementation truncates if it detects partial read.
    let mut count = 0;
    Wal::replay(&path, |_| {
        count += 1;
    })
    .unwrap();

    assert_eq!(count, 2, "Should recover exactly 2 records");

    // Verify file size was corrected (truncated to end of 2nd record)
    // 2 records * ~102 bytes = ~204 bytes.
    let new_len = fs::metadata(&path).unwrap().len();
    assert!(
        new_len < truncated_len,
        "File should be truncated to valid length"
    );
    assert!(new_len > 0);
}

#[test]
fn test_wal_crc_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("wal_crc.log");

    {
        let mut wal = Wal::new(&path, WalSyncMode::Async).unwrap();
        for i in 0..3 {
            let vec = vec![0.5f64; 10];
            wal.append(i, &vec, &HashMap::new(), 0).unwrap();
        }
    }

    // Corrupt the 2nd record.
    // Record 1 ends at ~102 bytes.
    // Record 2 is from ~102 to ~204.
    // Let's modify a byte at offset 150 (in payload of 2nd record).
    let mut data = fs::read(&path).unwrap();
    data[150] = data[150].wrapping_add(1);
    fs::write(&path, &data).unwrap();

    let mut count = 0;
    Wal::replay(&path, |_| {
        count += 1;
    })
    .unwrap();

    // Since 2nd record is corrupted, we should only get 1st record.
    // And file should be truncated to end of 1st record.
    assert_eq!(
        count, 1,
        "Should recover only 1 record due to CRC failure in 2nd"
    );

    let recovered_len = fs::metadata(&path).unwrap().len();
    assert!(recovered_len < data.len() as u64);
}
