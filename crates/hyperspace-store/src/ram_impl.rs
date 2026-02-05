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
