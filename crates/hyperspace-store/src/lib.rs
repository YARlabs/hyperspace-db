use memmap2::{MmapMut, MmapOptions};
use std::fs::{OpenOptions, File};
use std::path::Path;

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct VectorStore {
    file: File,
    mmap: MmapMut,
    count: AtomicUsize, // Changed from usize to AtomicUsize
    // Size of one vector in bytes (including padding and alpha) - simplified for MVP
    stride: usize, 
    dim: usize,
}

impl VectorStore {
    pub fn new(path: &Path, dim: usize, count: usize) -> Self {
        // 1. Open file, create if missing
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
            
        // 2. Set initial size (e.g. 1GB or enough for count if specified)
        // For MVP let's allocate 1GB fixed
        file.set_len(1024 * 1024 * 1024).unwrap();

        // 3. Map to memory
        let mmap = unsafe { MmapOptions::new().map_mut(&file).unwrap() };

        Self {
            file,
            mmap,
            count: AtomicUsize::new(count), 
            stride: std::mem::size_of::<f64>() * (dim + 1), // +1 for alpha, simplified
            dim,
        }
    }
    
    // Get f64 slice directly from OS kernel memory (Zero-Copy)
    pub fn get(&self, index: usize) -> &[f64] {
        let start = index * self.stride;
        let end = start + self.stride;
        // Unsafe cast bytes to f64
        unsafe {
            let ptr = self.mmap[start..end].as_ptr() as *const f64;
            std::slice::from_raw_parts(ptr, self.stride / 8)
        }
    }

    /// Appends a vector and returns its ID
    pub fn append(&self, vector: &[f64]) -> Result<u32, String> {
        // 1. Check dimension
        if vector.len() != self.dim {
            return Err(format!("Vector dim mismatch: expected {}, got {}", self.dim, vector.len()));
        }

        // 2. Reserve space atomically
        // stride includes alpha.
        // self.count acts as current ID/index.
        let id = self.count.fetch_add(1, Ordering::SeqCst) as u32;
        let offset = id as usize * self.stride;

        if offset + self.stride > self.mmap.len() {
             return Err("Storage full! Resize needed.".to_string());
        }

        // 3. Calculate Alpha (Hyperbolic curvature factor)
        let sq_norm: f64 = vector.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector outside Poincaré ball".to_string());
        }
        let alpha = 1.0 / (1.0 - sq_norm);

        // 4. Write to Mmap (Unsafe but valid if we own the file space)
        unsafe {
            let ptr = self.mmap.as_ptr().add(offset) as *mut f64;
            // Write coords
            std::ptr::copy_nonoverlapping(vector.as_ptr(), ptr, self.dim);
            // Write alpha to the end of coords
        }

        Ok(id)
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}
