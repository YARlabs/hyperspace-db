pub mod wal;
use memmap2::{MmapMut, MmapOptions};
use std::fs::{OpenOptions, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use parking_lot::RwLock;

const CHUNK_SIZE: usize = 65536; // 2^16
const CHUNK_SHIFT: usize = 16;
const CHUNK_MASK: usize = 0xFFFF;

#[derive(Debug)]
struct Segment {
    mmap: RwLock<MmapMut>,
    file: File,
}

#[derive(Debug)]
pub struct VectorStore {
    segments: RwLock<Vec<Arc<Segment>>>,
    count: AtomicUsize,
    dim: usize,
    base_path: PathBuf,
}

impl VectorStore {
    pub fn new(base_path: &Path, dim: usize) -> Self {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path).expect("Failed to create data dir");
        }

        let mut segments = Vec::new();
        let mut i = 0;
        loop {
            let path = base_path.join(format!("chunk_{}.hyp", i));
            if !path.exists() {
                if i == 0 {
                   // Create first segment if none
                   let seg = Self::create_segment(&path, dim).expect("Failed to create init segment");
                   segments.push(Arc::new(seg));
                }
                break;
            }
            let seg = Self::create_segment(&path, dim).expect("Failed to open segment");
            segments.push(Arc::new(seg));
            i += 1;
        }

        Self {
            segments: RwLock::new(segments),
            count: AtomicUsize::new(0), 
            dim,
            base_path: base_path.to_path_buf(),
        }
    }
    
    fn create_segment(path: &Path, dim: usize) -> std::io::Result<Segment> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
            
        let stride = std::mem::size_of::<f64>() * (dim + 1);
        let size = (stride * CHUNK_SIZE) as u64;
        
        file.set_len(size)?;
        
        let mmap = unsafe { MmapOptions::new().map_mut(&file)? };
        
        Ok(Segment {
            mmap: RwLock::new(mmap),
            file,
        })
    }

    pub fn append(&self, vector: &[f64]) -> Result<u32, String> {
        if vector.len() != self.dim {
             return Err(format!("Vector dim mismatch: {} vs {}", vector.len(), self.dim));
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst);
        let segment_idx = id >> CHUNK_SHIFT;
        let local_idx = id & CHUNK_MASK;

        // Check if we need new segment
        // Fast path:
        let has_segment = {
            let segs = self.segments.read();
            segment_idx < segs.len()
        };
        
        if !has_segment {
            let mut segs = self.segments.write();
             // Double check
             if segment_idx >= segs.len() {
                 let new_chunk_id = segs.len();
                 let path = self.base_path.join(format!("chunk_{}.hyp", new_chunk_id));
                 match Self::create_segment(&path, self.dim) {
                     Ok(seg) => {
                         segs.push(Arc::new(seg));
                         println!("📦 Storage grew! Created segment {}", new_chunk_id);
                     },
                     Err(e) => return Err(format!("Failed to grow storage: {}", e)),
                 }
             }
        }
        
        // Write logic
        {
             let segs = self.segments.read();
             let segment = &segs[segment_idx];
             
             let stride = std::mem::size_of::<f64>() * (self.dim + 1);
             let start = local_idx * stride;
             
             let mut guard = segment.mmap.write();
             let ptr = unsafe { guard.as_mut_ptr().add(start) as *mut f64 };
             
             // Alpha calc
            let sq_norm: f64 = vector.iter().map(|&x| x * x).sum();
            if sq_norm >= 1.0 - 1e-9 { 
                return Err("Vector outside Poincaré ball".to_string());
            }
            let alpha = 1.0 / (1.0 - sq_norm);

            unsafe {
                std::ptr::copy_nonoverlapping(vector.as_ptr(), ptr, self.dim);
                std::ptr::write(ptr.add(self.dim), alpha);
            }
        }

        Ok(id as u32)
    }

    pub fn get(&self, id: u32) -> &[f64] {
        let id_val = id as usize;
        let segment_idx = id_val >> CHUNK_SHIFT;
        let local_idx = id_val & CHUNK_MASK;
        
        let segs = self.segments.read();
        if segment_idx >= segs.len() {
            // Should not happen if logic matches
            // However, this panic might happen during concurrent access if logic is flawed
            panic!("VectorStore: Access out of bounds segment {}", segment_idx);
        }
        let segment = &segs[segment_idx];
        
        let stride = std::mem::size_of::<f64>() * (self.dim + 1);
        let start = local_idx * stride;
        
        let guard = segment.mmap.read();
        let ptr = unsafe { guard.as_ptr().add(start) as *const f64 };
        
        // Return slice covering coords + alpha (size = dim + 1)
        unsafe { std::slice::from_raw_parts(ptr, self.dim + 1) }
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    pub fn set_count(&self, c: usize) {
        self.count.store(c, Ordering::Relaxed);
    }
}
