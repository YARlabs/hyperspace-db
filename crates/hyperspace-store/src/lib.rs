pub mod wal;
use memmap2::{MmapMut, MmapOptions};
use parking_lot::RwLock;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 65536; // 2^16
const CHUNK_SHIFT: usize = 16;
const CHUNK_MASK: usize = 0xFFFF;

#[derive(Debug)]
struct Segment {
    mmap: RwLock<MmapMut>,
    #[allow(dead_code)]
    file: File,
}

#[derive(Debug)]
pub struct VectorStore {
    segments: RwLock<Vec<Arc<Segment>>>,
    count: AtomicUsize,
    element_size: usize,
    base_path: PathBuf,
}

impl VectorStore {
    pub fn new(base_path: &Path, element_size: usize) -> Self {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path).expect("Failed to create data dir");
        }

        let mut segments = Vec::new();
        let mut i = 0;
        loop {
            let path = base_path.join(format!("chunk_{}.hyp", i));
            if !path.exists() {
                if i == 0 {
                    let seg = Self::create_segment(&path, element_size)
                        .expect("Failed to create init segment");
                    segments.push(Arc::new(seg));
                }
                break;
            }
            let seg = Self::create_segment(&path, element_size).expect("Failed to open segment");
            segments.push(Arc::new(seg));
            i += 1;
        }

        Self {
            segments: RwLock::new(segments),
            count: AtomicUsize::new(0),
            element_size,
            base_path: base_path.to_path_buf(),
        }
    }

    fn create_segment(path: &Path, element_size: usize) -> std::io::Result<Segment> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        let size = (element_size * CHUNK_SIZE) as u64;
        file.set_len(size)?;

        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };

        Ok(Segment {
            mmap: RwLock::new(mmap),
            file,
        })
    }

    pub fn append(&self, vector_bytes: &[u8]) -> Result<u32, String> {
        if vector_bytes.len() != self.element_size {
            return Err(format!(
                "Vector size mismatch: {} vs {}",
                vector_bytes.len(),
                self.element_size
            ));
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst);
        let segment_idx = id >> CHUNK_SHIFT;
        let local_idx = id & CHUNK_MASK;

        let has_segment = {
            let segs = self.segments.read();
            segment_idx < segs.len()
        };

        if !has_segment {
            let mut segs = self.segments.write();
            if segment_idx >= segs.len() {
                let new_chunk_id = segs.len();
                let path = self.base_path.join(format!("chunk_{}.hyp", new_chunk_id));
                match Self::create_segment(&path, self.element_size) {
                    Ok(seg) => {
                        segs.push(Arc::new(seg));
                        println!("📦 Storage grew! Created segment {}", new_chunk_id);
                    }
                    Err(e) => return Err(format!("Failed to grow storage: {}", e)),
                }
            }
        }

        {
            let segs = self.segments.read();
            let segment = &segs[segment_idx];

            let start = local_idx * self.element_size;

            let mut guard = segment.mmap.write();
            let ptr = unsafe { guard.as_mut_ptr().add(start) };

            unsafe {
                std::ptr::copy_nonoverlapping(vector_bytes.as_ptr(), ptr, self.element_size);
            }
        }

        Ok(id as u32)
    }

    pub fn get(&self, id: u32) -> &[u8] {
        let id_val = id as usize;
        let segment_idx = id_val >> CHUNK_SHIFT;
        let local_idx = id_val & CHUNK_MASK;

        let segs = self.segments.read();
        if segment_idx >= segs.len() {
            panic!("VectorStore: Access out of bounds segment {}", segment_idx);
        }
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;

        let guard = segment.mmap.read();
        let ptr = unsafe { guard.as_ptr().add(start) };

        unsafe { std::slice::from_raw_parts(ptr, self.element_size) }
    }

    /// Update existing vector in-place (for upsert)
    pub fn update(&self, id: u32, vector_bytes: &[u8]) -> Result<(), String> {
        if vector_bytes.len() != self.element_size {
            return Err(format!(
                "Vector size mismatch: {} vs {}",
                vector_bytes.len(),
                self.element_size
            ));
        }

        let id_val = id as usize;
        let segment_idx = id_val >> CHUNK_SHIFT;
        let local_idx = id_val & CHUNK_MASK;

        let segs = self.segments.read();
        if segment_idx >= segs.len() {
            return Err(format!("VectorStore: ID {} out of bounds", id));
        }
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;

        let mut guard = segment.mmap.write();
        let ptr = unsafe { guard.as_mut_ptr().add(start) };

        unsafe {
            std::ptr::copy_nonoverlapping(vector_bytes.as_ptr(), ptr, self.element_size);
        }

        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.segments.read().len()
    }

    pub fn total_size_bytes(&self) -> usize {
        let segs = self.segments.read();
        if segs.is_empty() {
            return 0;
        }
        let segment_capacity = self.element_size * CHUNK_SIZE;
        segs.len() * segment_capacity
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
    use tempfile::tempdir;

    #[test]
    fn test_store_segments() {
        let dir = tempdir().unwrap();
        let store = VectorStore::new(dir.path(), 8);

        // Initially 1 segment
        assert_eq!(store.segment_count(), 1);

        let data = [1u8; 8];
        // Append 10 vectors
        for _ in 0..10 {
            store.append(&data).unwrap();
        }
        assert_eq!(store.count(), 10);

        let retrieved = store.get(0);
        assert_eq!(retrieved, &data);
    }
}
