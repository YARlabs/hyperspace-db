#![allow(clippy::cast_possible_truncation)]
use arc_swap::ArcSwap;
use memmap2::{Mmap, MmapMut, MmapOptions};
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const CHUNK_SIZE: usize = 65536; // 2^16
const CHUNK_SHIFT: usize = 16;
const CHUNK_MASK: usize = 0xFFFF;

#[derive(Debug)]
struct Segment {
    read_mmap: Mmap,
    write_mmap: Mutex<MmapMut>,
    #[allow(dead_code)]
    file: File,
}

/// Persistent vector storage using memory-mapped files.
/// Data is split into 64K chunks (`chunk_N.hyp`).
#[derive(Debug)]
pub struct VectorStore {
    segments: ArcSwap<Vec<Arc<Segment>>>,
    growth_lock: Mutex<()>,
    count: AtomicUsize,
    element_size: usize,
    base_path: PathBuf,
}

impl VectorStore {
    /// Creates or opens a `VectorStore` at the given path.
    pub fn new(base_path: &Path, element_size: usize) -> Self {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path).expect("Failed to create data dir");
        }

        let mut segments = Vec::new();
        let mut i = 0;
        loop {
            let path = base_path.join(format!("chunk_{i}.hyp"));
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
            segments: ArcSwap::from_pointee(segments),
            growth_lock: Mutex::new(()),
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
        let read_mmap = unsafe { MmapOptions::new().map(&file)? };

        Ok(Segment {
            read_mmap,
            write_mmap: Mutex::new(mmap),
            file,
        })
    }

    /// Appends a vector to the end of the store. Returns the new ID.
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

        self.ensure_segment(segment_idx)?;

        {
            let segs = self.segments.load();
            let segment = &segs[segment_idx];
            let start = local_idx * self.element_size;

            let mut guard = segment.write_mmap.lock();
            let ptr = unsafe { guard.as_mut_ptr().add(start) };

            unsafe {
                std::ptr::copy_nonoverlapping(vector_bytes.as_ptr(), ptr, self.element_size);
            }
        }

        Ok(id as u32)
    }

    /// Retrieves a vector by ID. Returns a view into the memory map.
    pub fn get(&self, id: u32) -> &[u8] {
        let id_val = id as usize;
        let segment_idx = id_val >> CHUNK_SHIFT;
        let local_idx = id_val & CHUNK_MASK;

        let segs = self.segments.load();
        assert!(
            segment_idx < segs.len(),
            "VectorStore: Access out of bounds segment {segment_idx}"
        );
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;

        let ptr = unsafe { segment.read_mmap.as_ptr().add(start) };

        unsafe { std::slice::from_raw_parts(ptr, self.element_size) }
    }

    /// Updates an existing vector in place.
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

        let segs = self.segments.load();
        if segment_idx >= segs.len() {
            return Err(format!("VectorStore: ID {id} out of bounds"));
        }
        let segment = &segs[segment_idx];

        let start = local_idx * self.element_size;

        let mut guard = segment.write_mmap.lock();
        let ptr = unsafe { guard.as_mut_ptr().add(start) };

        unsafe {
            std::ptr::copy_nonoverlapping(vector_bytes.as_ptr(), ptr, self.element_size);
        }

        Ok(())
    }

    pub fn segment_count(&self) -> usize {
        self.segments.load().len()
    }

    pub fn total_size_bytes(&self) -> usize {
        let segs = self.segments.load();
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

    /// Serializes only the used portion of the storage to a byte vector.
    /// This is primarily for WASM/serialization use cases.
    pub fn export(&self) -> Vec<u8> {
        let count = self.count.load(Ordering::Relaxed);
        let total_bytes = count * self.element_size;
        let mut result = Vec::with_capacity(total_bytes);

        let segs = self.segments.load();
        let mut bytes_read = 0;

        for segment in segs.iter() {
            let data_guard = &segment.read_mmap;
            let remaining = total_bytes - bytes_read;
            if remaining == 0 {
                break;
            }

            let chunk_data_size = self.element_size * CHUNK_SIZE;
            let to_copy = std::cmp::min(remaining, chunk_data_size);

            unsafe {
                let ptr = data_guard.as_ptr();
                let slice = std::slice::from_raw_parts(ptr, to_copy);
                result.extend_from_slice(slice);
            }
            bytes_read += to_copy;
        }

        result
    }

    /// Reconstructs the store from bytes.
    /// This is primarily for WASM/deserialization use cases.
    /// Note: For mmap implementation, this creates temporary files.
    pub fn from_bytes(path: &Path, element_size: usize, data: &[u8]) -> Self {
        let store = Self::new(path, element_size);

        // Calculate count derived from data length
        let count = data.len() / element_size;
        store.set_count(count);

        // Fill segments by writing data
        let mut offset = 0;
        let mut segment_idx = 0;

        while offset < data.len() {
            store
                .ensure_segment(segment_idx)
                .unwrap_or_else(|e| panic!("Failed to grow storage during from_bytes: {e}"));

            let segs = store.segments.load();
            let segment = &segs[segment_idx];
            let mut mmap_guard = segment.write_mmap.lock();

            let seg_capacity = element_size * CHUNK_SIZE;
            let remaining_data = data.len() - offset;
            let to_copy = std::cmp::min(remaining_data, seg_capacity);

            unsafe {
                let ptr = mmap_guard.as_mut_ptr();
                std::ptr::copy_nonoverlapping(data[offset..].as_ptr(), ptr, to_copy);
            }

            offset += to_copy;
            segment_idx += 1;
        }

        store
    }

    fn ensure_segment(&self, segment_idx: usize) -> Result<(), String> {
        if segment_idx < self.segments.load().len() {
            return Ok(());
        }

        let _growth_guard = self.growth_lock.lock();

        let current = self.segments.load();
        if segment_idx < current.len() {
            return Ok(());
        }

        let mut next = (**current).clone();
        while segment_idx >= next.len() {
            let new_chunk_id = next.len();
            let path = self.base_path.join(format!("chunk_{new_chunk_id}.hyp"));
            let seg = Self::create_segment(&path, self.element_size)
                .map_err(|e| format!("Failed to grow storage: {e}"))?;
            next.push(Arc::new(seg));
        }

        self.segments.store(Arc::new(next));
        Ok(())
    }
}
