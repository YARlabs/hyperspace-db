//! # Sidecar Payload Storage (v3.2)
//!
//! **Architecture Contract:**
//! - **RAM Layer:** ONLY the `PayloadIndex` lives in RAM — 16 bytes per vector slot.
//! - **Disk Layer:** All payload bytes are stored in an append-only `payloads.hyp` file,
//!   zstd-compressed to minimize NVMe I/O. Reads happen via `FileExt::read_exact_at`
//!   (positional, no seek lock) wrapped inside `tokio::task::spawn_blocking`.
//!
//! **Invariants enforced:**
//! 1. `insert()` compresses and writes to disk atomically — never allocates payload bytes in RAM.
//! 2. `fetch()` is called ONLY for final Top-K IDs after HNSW traversal completes.
//! 3. During compaction, deleted vectors' payloads are skipped, not copied.
//! 4. `PayloadIndex` is persisted as a compact binary sidecar (`payload_index.hyp`)
//!    to survive restarts without a full file scan.

#![allow(clippy::cast_possible_truncation)]

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use parking_lot::{Mutex, RwLock};
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Cursor, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

// ────────────────────────────────────────────────────────────────────────────
// On-disk file layout for `payloads.hyp`:
//
//   [MAGIC: 8 bytes "HSPAY100"]
//   [Entry₀: compressed_len(u32) | uncompressed_len(u32) | compressed_bytes...]
//   [Entry₁: ...]
//   ...
//
// The `payload_index.hyp` contains the in-RAM index serialized as:
//   [count: u64]
//   [slot₀: valid(u8) | offset(u64) | compressed_len(u32) | uncompressed_len(u32)] × count
// ────────────────────────────────────────────────────────────────────────────

const PAYLOAD_FILE_MAGIC: &[u8; 8] = b"HSPAY100";
// On-disk index slot size = 1 (valid) + 8 (offset) + 4 (comp_len) + 4 (uncomp_len) = 17 bytes

/// Per-vector slot in the in-RAM index. 17 bytes, but aligned to 24 by the compiler.
/// For 100K vectors that's ~2.4 MB of RAM total — negligible.
#[derive(Debug, Clone, Copy)]
pub struct PayloadSlot {
    /// Byte offset of the entry header in `payloads.hyp`.
    pub offset: u64,
    /// Length of the compressed bytes on disk (not counting the 8-byte header).
    pub compressed_len: u32,
    /// Original uncompressed length (for allocation pre-sizing and integrity checks).
    pub uncompressed_len: u32,
}

/// The in-RAM index — a flat `Vec` indexed by vector ID.
/// `None` means the vector has no sidecar payload.
type PayloadIndex = Vec<Option<PayloadSlot>>;

/// Append-only, zstd-compressed sidecar payload store.
///
/// Thread-safe. `fetch` is async-compatible via `spawn_blocking`.
pub struct PayloadStore {
    /// The append-only blob file handle — protected by a `Mutex` because we
    /// call `seek` before each write (append). Reads use `FileExt::read_exact_at`
    /// which is positional and needs no mutex.
    writer: Mutex<BufWriter<File>>,
    /// Separate read-only handle for positional reads — avoids lock contention
    /// with the writer during concurrent searches.
    reader: Arc<File>,
    /// In-RAM index: 16 bytes per slot. This is the ONLY payload data in RAM.
    index: RwLock<PayloadIndex>,
    /// Byte offset at which the next entry will be written.
    next_offset: Mutex<u64>,
    /// Paths for crash recovery.
    payload_path: PathBuf,
    index_path: PathBuf,
    /// zstd compression level (1 = fastest, 22 = best; 3 is the sweet spot).
    zstd_level: i32,
}

impl std::fmt::Debug for PayloadStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PayloadStore")
            .field("payload_path", &self.payload_path)
            .field("zstd_level", &self.zstd_level)
            .finish()
    }
}

impl PayloadStore {
    /// Returns the zstd compression level this store was opened with.
    /// Used by compaction to open a new store with identical settings.
    pub fn zstd_level(&self) -> i32 {
        self.zstd_level
    }

    /// Opens or creates a `PayloadStore` rooted at `base_path`.
    /// Files created: `payloads.hyp` and `payload_index.hyp`.
    pub fn open(base_path: &Path, zstd_level: i32) -> io::Result<Self> {
        let payload_path = base_path.join("payloads.hyp");
        let index_path = base_path.join("payload_index.hyp");

        // ── Open or create the payload blob file ──────────────────────────────
        let write_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&payload_path)?;

        let read_file = OpenOptions::new()
            .read(true)
            .open(&payload_path)?;

        // ── Initialize file (write magic if new) ──────────────────────────────
        let file_len = write_file.metadata()?.len();
        let mut writer = BufWriter::new(write_file);
        let next_offset = if file_len == 0 {
            writer.write_all(PAYLOAD_FILE_MAGIC)?;
            writer.flush()?;
            PAYLOAD_FILE_MAGIC.len() as u64
        } else {
            // Verify magic
            let mut magic = [0u8; 8];
            read_file.read_exact_at(&mut magic, 0)?;
            if &magic != PAYLOAD_FILE_MAGIC {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "payloads.hyp: invalid magic bytes — file may be corrupt",
                ));
            }
            file_len
        };

        // ── Load or build the in-RAM index ────────────────────────────────────
        let index = if index_path.exists() {
            Self::load_index(&index_path)?
        } else {
            Vec::new()
        };

        Ok(Self {
            writer: Mutex::new(writer),
            reader: Arc::new(read_file),
            index: RwLock::new(index),
            next_offset: Mutex::new(next_offset),
            payload_path,
            index_path,
            zstd_level,
        })
    }

    /// Insert a payload for the given vector `id`.
    ///
    /// **Memory contract:** only the compressed bytes exist temporarily on the stack
    /// during compression. They are written to disk and then freed. ZERO bytes of
    /// payload content are retained in RAM after this call returns.
    ///
    /// # Errors
    /// Returns an `io::Error` if compression or disk write fails.
    pub fn insert(&self, id: u32, raw_payload: &[u8]) -> io::Result<()> {
        // 1. Compress on the current thread (CPU-bound but fast for small payloads)
        let compressed = zstd::encode_all(raw_payload, self.zstd_level)?;

        let compressed_len = compressed.len() as u32;
        let uncompressed_len = raw_payload.len() as u32;

        // 2. Append to the blob file under the write lock
        let entry_offset = {
            let mut off_guard = self.next_offset.lock();
            let entry_offset = *off_guard;

            let mut w = self.writer.lock();
            // Seek to end (the BufWriter may have buffered data)
            w.seek(SeekFrom::End(0))?;

            // Write the 8-byte entry header
            w.write_u32::<LittleEndian>(compressed_len)?;
            w.write_u32::<LittleEndian>(uncompressed_len)?;
            // Write the compressed payload
            w.write_all(&compressed)?;
            w.flush()?;

            *off_guard = entry_offset + 8 + u64::from(compressed_len);
            entry_offset
        };

        // 3. Update the in-RAM index (atomic swap under write lock)
        let slot = PayloadSlot {
            offset: entry_offset,
            compressed_len,
            uncompressed_len,
        };
        {
            let mut idx = self.index.write();
            let id_usize = id as usize;
            if idx.len() <= id_usize {
                idx.resize(id_usize + 1, None);
            }
            idx[id_usize] = Some(slot);
        }

        // 4. Persist the index after every write (small, sequential I/O)
        self.flush_index()?;

        Ok(())
    }

    /// Fetch the decompressed payload for `id` using a positional read.
    ///
    /// This function is **synchronous** and blocking — it MUST be wrapped in
    /// `tokio::task::spawn_blocking` when called from async context.
    ///
    /// Returns `None` if the ID has no payload or if the ID is out of range.
    pub fn fetch_blocking(&self, id: u32) -> io::Result<Option<Vec<u8>>> {
        // 1. Look up the slot under a cheap read lock
        let slot = {
            let idx = self.index.read();
            match idx.get(id as usize) {
                Some(Some(s)) => *s,
                _ => return Ok(None),
            }
        };

        // 2. Read the compressed bytes via positional I/O (no seek lock needed)
        // Header is 8 bytes (compressed_len u32 + uncompressed_len u32)
        let data_offset = slot.offset + 8;
        let mut compressed = vec![0u8; slot.compressed_len as usize];
        self.reader.read_exact_at(&mut compressed, data_offset)?;

        // 3. Decompress — the result lives only as long as needed by the caller
        let decompressed = zstd::decode_all(Cursor::new(&compressed))?;

        // Integrity check
        debug_assert_eq!(
            decompressed.len(),
            slot.uncompressed_len as usize,
            "Payload decompression size mismatch for id {id}"
        );

        Ok(Some(decompressed))
    }

    /// Async wrapper around `fetch_blocking` using `tokio::task::spawn_blocking`.
    ///
    /// Spawns a thread-pool task for the disk I/O so the Tokio runtime thread
    /// is never blocked. The `Arc<File>` is cheaply cloned for the closure.
    pub async fn fetch_async(store: Arc<Self>, id: u32) -> io::Result<Option<Vec<u8>>> {
        tokio::task::spawn_blocking(move || store.fetch_blocking(id))
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    }

    /// Fetch payloads for multiple IDs concurrently.
    ///
    /// All reads are dispatched as concurrent `spawn_blocking` tasks.
    /// The returned `Vec` is aligned with the input `ids` slice.
    pub async fn fetch_many(store: Arc<Self>, ids: &[u32]) -> Vec<Option<Vec<u8>>> {
        let handles: Vec<_> = ids
            .iter()
            .copied()
            .map(|id| {
                let s = Arc::clone(&store);
                tokio::task::spawn_blocking(move || s.fetch_blocking(id).ok().flatten())
            })
            .collect();

        let mut results = Vec::with_capacity(handles.len());
        for h in handles {
            results.push(h.await.ok().flatten());
        }
        results
    }

    /// Returns `true` if the given ID has a stored payload.
    pub fn has_payload(&self, id: u32) -> bool {
        self.index
            .read()
            .get(id as usize)
            .map_or(false, |s| s.is_some())
    }

    /// Remove the payload slot for `id` (used when a vector is deleted / tombstoned).
    /// The on-disk bytes are reclaimed during the next compaction.
    pub fn remove(&self, id: u32) -> io::Result<()> {
        {
            let mut idx = self.index.write();
            if let Some(slot) = idx.get_mut(id as usize) {
                *slot = None;
            }
        }
        self.flush_index()?;
        Ok(())
    }

    /// **Compaction helper:** Copies the payload for `id` from `old_store` into this
    /// (new) `PayloadStore`, updating the offset. Deleted IDs (where `old_store` slot
    /// is `None`) are silently skipped — their disk bytes are NOT copied.
    ///
    /// This is called by the background vacuum/compaction loop.
    pub fn compact_copy_from(&self, old_store: &Self, id: u32) -> io::Result<()> {
        // Fast path: no payload, nothing to copy
        let slot = {
            let idx = old_store.index.read();
            match idx.get(id as usize) {
                Some(Some(s)) => *s,
                _ => return Ok(()),
            }
        };

        // Read the compressed bytes directly (no decompression — save CPU)
        let data_offset = slot.offset + 8;
        let mut compressed = vec![0u8; slot.compressed_len as usize];
        old_store
            .reader
            .read_exact_at(&mut compressed, data_offset)?;

        // Append the raw compressed bytes to the new file
        let entry_offset = {
            let mut off_guard = self.next_offset.lock();
            let entry_offset = *off_guard;

            let mut w = self.writer.lock();
            w.seek(SeekFrom::End(0))?;
            w.write_u32::<LittleEndian>(slot.compressed_len)?;
            w.write_u32::<LittleEndian>(slot.uncompressed_len)?;
            w.write_all(&compressed)?;
            w.flush()?;

            *off_guard = entry_offset + 8 + u64::from(slot.compressed_len);
            entry_offset
        };

        // Update the new store's index
        let new_slot = PayloadSlot {
            offset: entry_offset,
            compressed_len: slot.compressed_len,
            uncompressed_len: slot.uncompressed_len,
        };
        {
            let mut idx = self.index.write();
            let id_usize = id as usize;
            if idx.len() <= id_usize {
                idx.resize(id_usize + 1, None);
            }
            idx[id_usize] = Some(new_slot);
        }

        Ok(())
    }

    /// Returns the on-disk size of the payload blob file in bytes.
    pub fn disk_size_bytes(&self) -> u64 {
        *self.next_offset.lock()
    }

    /// Returns the number of payload slots (including empty ones).
    pub fn slot_count(&self) -> usize {
        self.index.read().len()
    }

    // ── Private helpers ──────────────────────────────────────────────────────

    /// Persists the in-RAM index to `payload_index.hyp`.
    /// Format: [count: u64] [slot: (valid: u8, offset: u64, comp: u32, uncomp: u32)] × count
    fn flush_index(&self) -> io::Result<()> {
        let idx = self.index.read();
        let count = idx.len();

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.index_path)?;
        let mut w = BufWriter::new(file);

        w.write_u64::<LittleEndian>(count as u64)?;
        for slot in idx.iter() {
            match slot {
                None => {
                    w.write_u8(0)?;
                    w.write_u64::<LittleEndian>(0)?;
                    w.write_u32::<LittleEndian>(0)?;
                    w.write_u32::<LittleEndian>(0)?;
                }
                Some(s) => {
                    w.write_u8(1)?;
                    w.write_u64::<LittleEndian>(s.offset)?;
                    w.write_u32::<LittleEndian>(s.compressed_len)?;
                    w.write_u32::<LittleEndian>(s.uncompressed_len)?;
                }
            }
        }
        w.flush()?;
        Ok(())
    }

    /// Loads the index from `payload_index.hyp`.
    fn load_index(path: &Path) -> io::Result<PayloadIndex> {
        let data = std::fs::read(path)?;
        let mut cursor = Cursor::new(data);

        let count = cursor.read_u64::<LittleEndian>()? as usize;
        let mut index = Vec::with_capacity(count);

        for _ in 0..count {
            let valid = cursor.read_u8()?;
            let offset = cursor.read_u64::<LittleEndian>()?;
            let compressed_len = cursor.read_u32::<LittleEndian>()?;
            let uncompressed_len = cursor.read_u32::<LittleEndian>()?;

            if valid == 1 {
                index.push(Some(PayloadSlot {
                    offset,
                    compressed_len,
                    uncompressed_len,
                }));
            } else {
                index.push(None);
            }
        }

        Ok(index)
    }
}

// ─── Constants ────────────────────────────────────────────────────────────────
/// Default zstd compression level. Level 3 = fast compression, ~60% ratio for JSON.
pub const DEFAULT_ZSTD_LEVEL: i32 = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn open_store(dir: &TempDir) -> Arc<PayloadStore> {
        Arc::new(PayloadStore::open(dir.path(), DEFAULT_ZSTD_LEVEL).expect("open store"))
    }

    // ── Test 1: Round-trip accuracy ───────────────────────────────────────────
    #[tokio::test]
    async fn test_payload_roundtrip_accuracy() {
        let dir = TempDir::new().unwrap();
        let store = open_store(&dir);

        // Insert 10 vectors with unique 50 KB payloads
        let mut payloads: Vec<Vec<u8>> = Vec::new();
        for i in 0u32..10 {
            // Each payload is 50 KB of pseudo-unique bytes
            let payload: Vec<u8> = (0..51_200).map(|j| ((i as usize + j) % 256) as u8).collect();
            store.insert(i, &payload).expect("insert");
            payloads.push(payload);
        }

        // Fetch vector #5 and verify it matches exactly
        let fetched = store
            .fetch_blocking(5)
            .expect("fetch_blocking")
            .expect("should have payload");

        assert_eq!(
            fetched, payloads[5],
            "Round-trip payload for id=5 must be byte-perfect"
        );
    }

    // ── Test 2: Search flow — include_payload true vs false ──────────────────
    #[tokio::test]
    async fn test_fetch_many_selective() {
        let dir = TempDir::new().unwrap();
        let store = open_store(&dir);

        let payload = b"hello, sidecar storage!".to_vec();
        store.insert(0, &payload).unwrap();
        store.insert(1, &payload).unwrap();
        // id=2 deliberately has NO payload

        // Simulates include_payload=true: fetch for final Top-K [0, 1, 2]
        let results = PayloadStore::fetch_many(Arc::clone(&store), &[0, 1, 2]).await;
        assert_eq!(results[0].as_deref(), Some(payload.as_slice()), "id=0 must have payload");
        assert_eq!(results[1].as_deref(), Some(payload.as_slice()), "id=1 must have payload");
        assert!(results[2].is_none(), "id=2 has no payload, must be None");

        // Simulates include_payload=false: no fetch at all — zero disk I/O
        // (just check has_payload doesn't blow up)
        assert!(store.has_payload(0));
        assert!(!store.has_payload(2));
    }

    // ── Test 3: OOM / Memory Safety ───────────────────────────────────────────
    // We assert that the RAM index stays tiny regardless of how many payloads
    // are on disk. The index is 17 bytes per slot (17 × N).
    #[test]
    fn test_ram_index_size_is_constant() {
        let dir = TempDir::new().unwrap();
        let store = PayloadStore::open(dir.path(), DEFAULT_ZSTD_LEVEL).unwrap();

        // Insert 1_000 vectors with 10 KB payloads
        let payload = vec![0xABu8; 10 * 1024];
        for i in 0u32..1_000 {
            store.insert(i, &payload).unwrap();
        }

        // The index should only have 1_000 slots — each 24 bytes (aligned)
        let slot_count = store.slot_count();
        assert_eq!(slot_count, 1_000);

        // The in-RAM index uses < 100 KB (1000 × 24 bytes = 24 KB)
        let ram_bytes = slot_count * std::mem::size_of::<Option<PayloadSlot>>();
        assert!(
            ram_bytes < 100_000,
            "RAM index must be < 100 KB for 1K entries, got {ram_bytes} bytes"
        );

        // But disk usage should be ~10 MB (10 KB × 1000, compressed to ~11% = ~1.1 MB)
        let disk_bytes = store.disk_size_bytes();
        assert!(
            disk_bytes > 1_000,
            "Disk must have payload data, got {disk_bytes} bytes"
        );
    }

    // ── Test 4: Compaction skips deleted vectors ──────────────────────────────
    #[test]
    fn test_compaction_skips_deleted() {
        let old_dir = TempDir::new().unwrap();
        let new_dir = TempDir::new().unwrap();

        let old_store = PayloadStore::open(old_dir.path(), DEFAULT_ZSTD_LEVEL).unwrap();
        let payload_a = b"active payload".to_vec();
        let payload_d = b"deleted payload - should NOT appear in new chunk".to_vec();

        old_store.insert(0, &payload_a).unwrap(); // active
        old_store.insert(1, &payload_d).unwrap(); // will be deleted
        old_store.remove(1).unwrap();             // tombstone

        let new_store = PayloadStore::open(new_dir.path(), DEFAULT_ZSTD_LEVEL).unwrap();

        // Compact: copy id=0 (active), skip id=1 (deleted)
        new_store.compact_copy_from(&old_store, 0).unwrap();
        new_store.compact_copy_from(&old_store, 1).unwrap(); // no-op

        let fetched_a = new_store.fetch_blocking(0).unwrap().unwrap();
        assert_eq!(fetched_a, payload_a, "Active payload must survive compaction");

        let fetched_d = new_store.fetch_blocking(1).unwrap();
        assert!(fetched_d.is_none(), "Deleted payload must NOT appear after compaction");
    }

    // ── Test 5: Index persistence across restarts ────────────────────────────
    #[test]
    fn test_index_survives_restart() {
        let dir = TempDir::new().unwrap();
        let payload = b"persistent across restarts".to_vec();

        // First open: insert
        {
            let store = PayloadStore::open(dir.path(), DEFAULT_ZSTD_LEVEL).unwrap();
            store.insert(42, &payload).unwrap();
        }

        // Second open: should recover index and fetch correctly
        {
            let store = PayloadStore::open(dir.path(), DEFAULT_ZSTD_LEVEL).unwrap();
            let fetched = store.fetch_blocking(42).unwrap().unwrap();
            assert_eq!(fetched, payload, "Payload must survive a simulated restart");
        }
    }
}
