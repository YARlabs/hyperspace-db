use parking_lot::RwLock;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 65536;

#[derive(Debug)]
pub struct VectorStore {
    segments: RwLock<Vec<Arc<RwLock<Vec<u8>>>>>,
    count: AtomicUsize,
    element_size: usize,
}

impl VectorStore {
    pub fn new(_base_path: &Path, element_size: usize) -> Self {
        let mut segments = Vec::new();
        // Pre-allocate first segment
        let seg_size = element_size * CHUNK_SIZE;
        let vec = vec![0u8; seg_size];
        segments.push(Arc::new(RwLock::new(vec)));

        Self {
            segments: RwLock::new(segments),
            count: AtomicUsize::new(0),
            element_size,
        }
    }

    pub fn append(&self, vector_bytes: &[u8]) -> Result<u32, String> {
        if vector_bytes.len() != self.element_size {
            return Err("Vector size mismatch".into());
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst);
        let segment_idx = id / CHUNK_SIZE;
        let local_idx = id % CHUNK_SIZE;

        let has_segment = {
            let segs = self.segments.read();
            segment_idx < segs.len()
        };

        if !has_segment {
            let mut segs = self.segments.write();
            if segment_idx >= segs.len() {
                // Grow
                let seg_size = self.element_size * CHUNK_SIZE;
                let vec = vec![0u8; seg_size];
                segs.push(Arc::new(RwLock::new(vec)));
            }
        }

        {
            let segs = self.segments.read();
            let segment = &segs[segment_idx];
            let mut data = segment.write();

            let start = local_idx * self.element_size;
            let end = start + self.element_size;
            data[start..end].copy_from_slice(vector_bytes);
        }

        Ok(id as u32)
    }

    pub fn get(&self, id: u32) -> &[u8] {
        let id_val = id as usize;
        let segment_idx = id_val / CHUNK_SIZE;
        let local_idx = id_val % CHUNK_SIZE;

        let segs = self.segments.read();
        if segment_idx >= segs.len() {
            panic!("VectorStore RAM: OOB access id {}", id);
        }
        let segment = &segs[segment_idx];

        let data_guard = segment.read();
        let ptr = data_guard.as_ptr();
        let start = local_idx * self.element_size;

        // UNSAFE: We assume the Vec is pinned and never reallocated/resized.
        // The pointer is valid as long as Vec exists (which is kept in Arc in self.segments).
        // Since we wrap inner Vec in RwLock and Arc, the buffer address is stable.
        unsafe {
            let ptr = ptr.add(start);
            std::slice::from_raw_parts(ptr, self.element_size)
        }
    }

    pub fn update(&self, id: u32, vector_bytes: &[u8]) -> Result<(), String> {
        if vector_bytes.len() != self.element_size {
            return Err("Size mismatch".into());
        }
        let id_val = id as usize;
        let segment_idx = id_val / CHUNK_SIZE;
        let local_idx = id_val % CHUNK_SIZE;

        let segs = self.segments.read();
        if segment_idx >= segs.len() {
            return Err("OOB".into());
        }

        let segment = &segs[segment_idx];
        let mut data = segment.write();
        let start = local_idx * self.element_size;
        data[start..start + self.element_size].copy_from_slice(vector_bytes);
        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.segments.read().len()
    }

    pub fn total_size_bytes(&self) -> usize {
        self.segments.read().len() * CHUNK_SIZE * self.element_size
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn set_count(&self, c: usize) {
        self.count.store(c, Ordering::Relaxed);
    }

    /// Serializes only the used portion of the storage to a byte vector.
    pub fn export(&self) -> Vec<u8> {
        let count = self.count.load(Ordering::Relaxed);
        let total_bytes = count * self.element_size;
        let mut result = Vec::with_capacity(total_bytes);

        let segs = self.segments.read();
        let mut bytes_read = 0;

        for segment in segs.iter() {
            let data = segment.read();
            let remaining = total_bytes - bytes_read;
            if remaining == 0 {
                break;
            }

            let chunk_data_size = data.len();
            let to_copy = std::cmp::min(remaining, chunk_data_size);

            result.extend_from_slice(&data[0..to_copy]);
            bytes_read += to_copy;
        }

        result
    }

    /// Reconstructs the store from bytes.
    pub fn from_bytes(path: &Path, element_size: usize, data: &[u8]) -> Self {
        let store = Self::new(path, element_size);

        // Calculate count derived from data length
        let count = data.len() / element_size;
        store.set_count(count);

        // Fill segments
        // Self::new created the first empty segment.
        // We write data into it (and grow if needed).
        // Since 'new' allocates full Chunk, we can just copy if it fits.

        let mut offset = 0;
        let segs = store.segments.read(); // Read lock is enough to access Arc<RwLock>

        // We might need write lock on segments vector if we need to grow beyond first chunk
        // Use loop logic similar to append but batching
        drop(segs); // Drop read lock to allow logic

        // Naive implementation: just use update/append logic or unsafe copy?
        // Better: fill segment 0, then create segment 1...

        let mut current_seg_idx = 0;

        while offset < data.len() {
            let segs = store.segments.read();
            if current_seg_idx >= segs.len() {
                drop(segs);
                let mut w_segs = store.segments.write();
                // Grow
                let seg_size = element_size * CHUNK_SIZE;
                let vec = vec![0u8; seg_size];
                w_segs.push(Arc::new(RwLock::new(vec)));
                continue;
            }

            let segment = &segs[current_seg_idx];
            let mut seg_data = segment.write();

            let seg_capacity = seg_data.len();
            let remaining_data = data.len() - offset;
            let to_copy = std::cmp::min(remaining_data, seg_capacity);

            seg_data[0..to_copy].copy_from_slice(&data[offset..offset + to_copy]);

            offset += to_copy;
            current_seg_idx += 1;
        }

        store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_store() {
        let store = VectorStore::new(Path::new("mem"), 8);
        assert_eq!(store.segment_count(), 1);

        let data = [1u8; 8];
        for _ in 0..10 {
            store.append(&data).unwrap();
        }

        assert_eq!(store.count(), 10);

        let retrieved = store.get(0);
        assert_eq!(retrieved, &data);

        // Test update
        let new_data = [2u8; 8];
        store.update(0, &new_data).unwrap();
        assert_eq!(store.get(0), &new_data);
    }
}
