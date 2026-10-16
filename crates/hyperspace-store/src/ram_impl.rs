use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 65536;

#[derive(Debug)]
struct RamSegment {
    data: RwLock<Vec<u8>>,
    raw_ptr: *mut u8,
}

unsafe impl Send for RamSegment {}
unsafe impl Sync for RamSegment {}

impl RamSegment {
    fn new(element_size: usize) -> Self {
        let seg_size = element_size * CHUNK_SIZE;
        let mut vec = vec![0u8; seg_size];
        let raw_ptr = vec.as_mut_ptr();
        Self {
            data: RwLock::new(vec),
            raw_ptr,
        }
    }
}

#[derive(Debug)]
pub struct VectorStore {
    segments: ArcSwap<Vec<Arc<RamSegment>>>,
    growth_lock: Mutex<()>,
    count: AtomicUsize,
    element_size: usize,
}

impl VectorStore {
    pub fn new(_base_path: &Path, element_size: usize) -> Self {
        let segments = vec![Arc::new(RamSegment::new(element_size))];

        Self {
            segments: ArcSwap::from_pointee(segments),
            growth_lock: Mutex::new(()),
            count: AtomicUsize::new(0),
            element_size,
        }
    }

    fn ensure_segment(&self, segment_idx: usize) {
        if segment_idx < self.segments.load().len() {
            return;
        }
        let _guard = self.growth_lock.lock();
        let current_segs = self.segments.load();
        if segment_idx >= current_segs.len() {
            let mut new_segs = (**current_segs).clone();
            while new_segs.len() <= segment_idx {
                new_segs.push(Arc::new(RamSegment::new(self.element_size)));
            }
            self.segments.store(Arc::new(new_segs));
        }
    }

    pub fn append(&self, vector_bytes: &[u8]) -> Result<u32, String> {
        if vector_bytes.len() != self.element_size {
            return Err("Vector size mismatch".into());
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst);
        let segment_idx = id / CHUNK_SIZE;
        let local_idx = id % CHUNK_SIZE;

        self.ensure_segment(segment_idx);

        let segs = self.segments.load();
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;
        unsafe {
            let dest_ptr = segment.raw_ptr.add(start);
            std::ptr::copy_nonoverlapping(vector_bytes.as_ptr(), dest_ptr, self.element_size);
        }

        #[allow(clippy::cast_possible_truncation)]
        Ok(id as u32)
    }

    #[inline(always)]
    pub fn get(&self, id: u32) -> &[u8] {
        let id_val = id as usize;
        let segment_idx = id_val / CHUNK_SIZE;
        let local_idx = id_val % CHUNK_SIZE;

        let segs = self.segments.load();
        assert!(
            segment_idx < segs.len(),
            "VectorStore RAM: OOB access id {id}"
        );
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;

        unsafe {
            let ptr = segment.raw_ptr.add(start);
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

        let segs = self.segments.load();
        if segment_idx >= segs.len() {
            return Err("OOB".into());
        }

        let segment = &segs[segment_idx];
        let mut data = segment.data.write();
        let start = local_idx * self.element_size;
        data[start..start + self.element_size].copy_from_slice(vector_bytes);
        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.segments.load().len()
    }

    pub fn total_size_bytes(&self) -> usize {
        self.segments.load().len() * CHUNK_SIZE * self.element_size
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

        let segs = self.segments.load();
        let mut bytes_read = 0;

        for segment in segs.iter() {
            let data = segment.data.read();
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
    pub fn from_bytes(_path: &Path, element_size: usize, data: &[u8]) -> Self {
        let count = if element_size > 0 {
            data.len() / element_size
        } else {
            0
        };
        let seg_size = element_size * CHUNK_SIZE;
        let num_segs = if seg_size > 0 {
            data.len().div_ceil(seg_size).max(1)
        } else {
            1
        };

        let mut segs = Vec::with_capacity(num_segs);
        let mut offset = 0;
        for _ in 0..num_segs {
            let mut seg = RamSegment::new(element_size);
            if offset < data.len() {
                let to_copy = std::cmp::min(data.len() - offset, seg_size);
                seg.data.get_mut()[..to_copy].copy_from_slice(&data[offset..offset + to_copy]);
                offset += to_copy;
            }
            segs.push(Arc::new(seg));
        }

        Self {
            segments: ArcSwap::from_pointee(segs),
            growth_lock: Mutex::new(()),
            count: AtomicUsize::new(count),
            element_size,
        }
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
