//! Exact `f32` copies of the vectors of a quantized collection (since 4.2.5).
//!
//! Rerank, online calibration and `RebuildIndex` need the original vectors, which the
//! quantized index no longer holds. Before 4.2.5 they were written to the payload store
//! under the *internal* id, the same key space user payloads use with *user* ids, so the
//! two could overwrite each other, and every rerank candidate cost a `pread` plus a zstd
//! decode. This store keeps them in a fixed-stride memory map instead: slot `id` holds
//! `dim` little-endian `f32`s followed by a `u32` presence flag, reads are a bounds check
//! and a copy out of the page cache, and absent slots (never written, deleted, or holes
//! left by sparse ids) read as `None`.

use hyperspace_store::VectorStore;
use parking_lot::Mutex;
use std::path::Path;

/// Slots per memory-mapped segment of [`VectorStore`] (`chunk_N.hyp`).
const SEGMENT_SLOTS: usize = 1 << 16;
const PRESENT: u32 = 0x4558_4143; // "EXAC"

pub struct ExactVectorStore {
    store: VectorStore,
    dim: usize,
    /// Serializes segment growth (writes into existing segments are already safe).
    grow: Mutex<()>,
}

impl ExactVectorStore {
    /// Subdirectory of a collection (or shard) data directory holding the store.
    pub const DIR: &'static str = "exact_f32";

    /// Opens (or creates) the store under `data_dir/exact_f32`.
    #[must_use]
    pub fn open(data_dir: &Path, dim: usize) -> Self {
        let store = VectorStore::new(&data_dir.join(Self::DIR), Self::slot_size(dim));
        // The mmap store tracks an append cursor that is not persisted; keep it at the end
        // of the mapped segments so growth appends whole new segments.
        store.set_count(store.segment_count() * SEGMENT_SLOTS);
        Self {
            store,
            dim,
            grow: Mutex::new(()),
        }
    }

    #[inline]
    fn slot_size(dim: usize) -> usize {
        dim * 4 + 4
    }

    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dim
    }

    fn ensure_slot(&self, id: u32) -> Result<(), String> {
        let needed = id as usize / SEGMENT_SLOTS + 1;
        if self.store.segment_count() >= needed {
            return Ok(());
        }
        let _g = self.grow.lock();
        let empty = vec![0u8; Self::slot_size(self.dim)];
        while self.store.segment_count() < needed {
            // Appending at the cursor (= first slot past the mapped segments) maps one
            // more segment; the slot is written as absent.
            self.store
                .set_count(self.store.segment_count() * SEGMENT_SLOTS);
            self.store.append(&empty)?;
        }
        self.store
            .set_count(self.store.segment_count() * SEGMENT_SLOTS);
        Ok(())
    }

    /// Stores the exact vector of internal id `id` (shorter inputs are zero-padded).
    pub fn put(&self, id: u32, v: &[f32]) -> Result<(), String> {
        self.ensure_slot(id)?;
        let mut slot = vec![0u8; Self::slot_size(self.dim)];
        for (dst, x) in slot
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(v.iter().take(self.dim))
        {
            dst.copy_from_slice(&x.to_le_bytes());
        }
        slot[self.dim * 4..].copy_from_slice(&PRESENT.to_le_bytes());
        self.store.update(id, &slot)
    }

    /// Marks `id` absent (deleted vector).
    pub fn remove(&self, id: u32) {
        if (id as usize) < self.store.segment_count() * SEGMENT_SLOTS {
            let _ = self.store.update(id, &vec![0u8; Self::slot_size(self.dim)]);
        }
    }

    /// Raw `dim * 4` little-endian bytes of `id`, if present.
    #[inline]
    #[must_use]
    pub fn get_bytes(&self, id: u32) -> Option<&[u8]> {
        if id as usize >= self.store.segment_count() * SEGMENT_SLOTS {
            return None;
        }
        let slot = self.store.get(id);
        if slot.len() != Self::slot_size(self.dim) {
            return None;
        }
        let (body, flag) = slot.split_at(self.dim * 4);
        (u32::from_le_bytes(flag.try_into().ok()?) == PRESENT).then_some(body)
    }

    /// Exact vector of `id`, if present.
    #[must_use]
    pub fn get(&self, id: u32) -> Option<Vec<f32>> {
        self.get_bytes(id).map(|b| {
            b.as_chunks::<4>()
                .0
                .iter()
                .map(|c| f32::from_le_bytes(*c))
                .collect()
        })
    }

    /// Copies the exact vector of `id` into `dst` as `f64` (up to `dst.len()` values).
    #[inline]
    pub fn read_f64(&self, id: u32, dst: &mut [f64]) -> bool {
        let Some(b) = self.get_bytes(id) else {
            return false;
        };
        for (d, c) in dst.iter_mut().zip(b.as_chunks::<4>().0) {
            *d = f64::from(f32::from_le_bytes(*c));
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_remove_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let v: Vec<f32> = (0..24).map(|i| i as f32 * 0.25 - 3.0).collect();
        {
            let s = ExactVectorStore::open(dir.path(), 24);
            assert!(s.get(0).is_none(), "fresh slots are absent");
            s.put(3, &v).unwrap();
            // Sparse ids beyond the first segment grow the map.
            s.put(70_000, &v).unwrap();
            s.put(5, &v[..10]).unwrap();
            assert_eq!(s.get(3).as_deref(), Some(v.as_slice()));
            assert_eq!(s.get(70_000).as_deref(), Some(v.as_slice()));
            assert!(s.get(4).is_none() && s.get(69_999).is_none());
            assert!(s.get(1 << 30).is_none(), "out of range reads are None");
            let five = s.get(5).unwrap();
            assert_eq!(&five[..10], &v[..10]);
            assert!(five[10..].iter().all(|&x| x == 0.0));
            s.remove(3);
            assert!(s.get(3).is_none());
            let mut d = vec![0.0f64; 24];
            assert!(s.read_f64(70_000, &mut d));
            assert!((d[23] - f64::from(v[23])).abs() < 1e-9);
        }
        let s = ExactVectorStore::open(dir.path(), 24);
        assert!(s.get(3).is_none());
        assert_eq!(s.get(70_000).as_deref(), Some(v.as_slice()));
        s.put(70_001, &v).unwrap();
        assert_eq!(s.get(70_001).as_deref(), Some(v.as_slice()));
        assert_eq!(
            s.get(70_000).as_deref(),
            Some(v.as_slice()),
            "reopen + write keeps earlier slots"
        );
    }

    #[test]
    fn concurrent_writers_grow_safely() {
        let dir = tempfile::tempdir().unwrap();
        let s = std::sync::Arc::new(ExactVectorStore::open(dir.path(), 8));
        let hs: Vec<_> = (0..8u32)
            .map(|t| {
                let s = std::sync::Arc::clone(&s);
                std::thread::spawn(move || {
                    for i in 0..2000u32 {
                        let id = i * 97 + t;
                        s.put(id, &[id as f32; 8]).unwrap();
                    }
                })
            })
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        for t in 0..8u32 {
            for i in (0..2000u32).step_by(37) {
                let id = i * 97 + t;
                assert_eq!(s.get(id), Some(vec![id as f32; 8]), "id {id}");
            }
        }
    }
}
