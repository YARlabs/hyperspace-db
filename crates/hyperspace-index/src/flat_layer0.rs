//! Cache-Aligned Flat Layer-0 Layout for Immutable/Frozen HNSW Graph Segments.
//!
//! Provides zero-pointer-chasing, cacheline-aligned data-oriented layout for Layer-0.
//! Combines node metadata, Layer-0 neighbor IDs, and quantized vector payloads into
//! contiguous 64-byte aligned strides.

#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::borrow_as_ptr)]

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::ptr::NonNull;

/// Magic signature for serialized Flat Layer-0 files: "HSFL0" + version 1.
pub const FLAT_L0_MAGIC: &[u8; 8] = b"HSFL0\x01\x00\x00";

/// 64-byte node header in the flat strided segment.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlatNodeHeader {
    /// Number of active neighbors for this node on Layer-0.
    pub neighbor_count: u16,
    /// Flags (e.g., bit 0 = active / not deleted).
    pub flags: u16,
    /// Length of the quantized vector in bytes.
    pub vector_len: u32,
    /// Reserved padding ensuring 64-byte total size.
    pub reserved: [u8; 56],
}

impl Default for FlatNodeHeader {
    fn default() -> Self {
        Self {
            neighbor_count: 0,
            flags: 0,
            vector_len: 0,
            reserved: [0u8; 56],
        }
    }
}

/// Aligned memory buffer ensuring 64-byte CPU cache-line alignment.
pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    layout: Layout,
    len: usize,
}

impl AlignedBuffer {
    /// Allocates a zeroed memory buffer with specified size and alignment.
    ///
    /// # Panics
    /// Panics if layout creation fails or system is out of memory.
    #[must_use]
    pub fn new(size: usize, align: usize) -> Self {
        let actual_size = size.max(64);
        let layout = Layout::from_size_align(actual_size, align).expect("Invalid layout");
        let raw = unsafe { alloc_zeroed(layout) };
        let ptr = NonNull::new(raw).expect("Out of memory for AlignedBuffer");
        Self {
            ptr,
            layout,
            len: size,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

/// Backing memory storage for the flat segment (Aligned Heap or Zero-Copy Memory Map).
pub enum FlatBuffer {
    /// In-memory 64-byte aligned heap allocation.
    Heap(AlignedBuffer),
    /// Memory-mapped file for zero-copy OS page cache reads.
    #[cfg(feature = "persistence")]
    Mmap(memmap2::Mmap),
}

impl AsRef<[u8]> for FlatBuffer {
    #[inline(always)]
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Heap(buf) => buf.as_slice(),
            #[cfg(feature = "persistence")]
            Self::Mmap(mmap) => mmap.as_ref(),
        }
    }
}

/// Monolithic, Cache-Aligned Flat Layer-0 Graph Segment.
///
/// Layout per node in memory:
/// ```text
/// [FlatNodeHeader (64B)] -> [Neighbors: [u32; max_m]] -> [Vector Bytes] -> [64B Padding]
/// ```
///
/// Guarantees:
/// 1. Node headers, neighbor arrays, and vector slices start on 64-byte cache-line boundaries.
/// 2. Total stride is a multiple of 64 bytes.
/// 3. Lookup is pure offset calculation `id * stride` with zero heap dereferencing.
pub struct FlatLayer0Segment {
    buffer: FlatBuffer,
    node_count: usize,
    stride: usize,
    max_m: usize,
    vector_len: usize,
    vector_offset: usize,
}

impl std::fmt::Debug for FlatLayer0Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlatLayer0Segment")
            .field("node_count", &self.node_count)
            .field("stride", &self.stride)
            .field("max_m", &self.max_m)
            .field("vector_len", &self.vector_len)
            .field("vector_offset", &self.vector_offset)
            .finish_non_exhaustive()
    }
}

impl FlatLayer0Segment {
    /// Computes the exact stride and offsets for given parameters, aligned to 64 bytes.
    #[inline]
    #[must_use]
    pub fn compute_layout(max_m: usize, vector_len: usize) -> (usize, usize) {
        let header_size = std::mem::size_of::<FlatNodeHeader>(); // 64 bytes
        let neighbors_bytes = max_m * std::mem::size_of::<u32>();
        // Align vector_offset to 64 bytes
        let vector_offset = (header_size + neighbors_bytes + 63) & !63;
        let raw_stride = vector_offset + vector_len;
        // Total stride aligned to 64 bytes
        let stride = (raw_stride + 63) & !63;
        (vector_offset, stride)
    }

    /// Creates a new in-memory 64-byte aligned heap-allocated `FlatLayer0Segment`.
    #[must_use]
    pub fn new(node_count: usize, max_m: usize, vector_len: usize) -> Self {
        let (vector_offset, stride) = Self::compute_layout(max_m, vector_len);
        let total_bytes = node_count * stride;

        let aligned_buf = AlignedBuffer::new(total_bytes, 64);

        Self {
            buffer: FlatBuffer::Heap(aligned_buf),
            node_count,
            stride,
            max_m,
            vector_len,
            vector_offset,
        }
    }

    /// Sets data for an individual node inside an in-memory mutable heap segment.
    ///
    /// # Panics
    /// Panics if called on a memory-mapped segment or if `id >= node_count`.
    pub fn set_node(&mut self, id: u32, neighbors: &[u32], vector: &[u8]) {
        assert!(
            (id as usize) < self.node_count,
            "Node ID {id} out of bounds (count {})",
            self.node_count
        );

        let data = match &mut self.buffer {
            FlatBuffer::Heap(ref mut buf) => buf.as_mut_slice(),
            #[cfg(feature = "persistence")]
            FlatBuffer::Mmap(_) => panic!("Cannot modify memory-mapped FlatLayer0Segment"),
        };

        let offset = id as usize * self.stride;
        let count = neighbors.len().min(self.max_m);
        let header = FlatNodeHeader {
            neighbor_count: count as u16,
            flags: 1, // Active
            vector_len: self.vector_len as u32,
            reserved: [0u8; 56],
        };

        let ptr = unsafe { data.as_mut_ptr().add(offset) };

        unsafe {
            // Write 64B header
            std::ptr::copy_nonoverlapping(
                &header as *const _ as *const u8,
                ptr,
                std::mem::size_of::<FlatNodeHeader>(),
            );

            // Write neighbors array at offset 64
            if count > 0 {
                let neighbors_ptr = ptr.add(std::mem::size_of::<FlatNodeHeader>()) as *mut u32;
                std::ptr::copy_nonoverlapping(neighbors.as_ptr(), neighbors_ptr, count);
            }

            // Write vector bytes at aligned vector_offset
            let copy_vec_len = self.vector_len.min(vector.len());
            if copy_vec_len > 0 {
                let vec_dst = ptr.add(self.vector_offset);
                std::ptr::copy_nonoverlapping(vector.as_ptr(), vec_dst, copy_vec_len);
            }
        }
    }

    /// Total number of nodes stored in this flat segment.
    #[inline(always)]
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }

    /// Stride in bytes per node.
    #[inline(always)]
    #[must_use]
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Maximum neighbors supported on Layer-0.
    #[inline(always)]
    #[must_use]
    pub fn max_m(&self) -> usize {
        self.max_m
    }

    /// Length of vector payload per node in bytes.
    #[inline(always)]
    #[must_use]
    pub fn vector_len(&self) -> usize {
        self.vector_len
    }

    /// Total memory footprint of this segment in bytes.
    #[inline(always)]
    #[must_use]
    pub fn total_bytes(&self) -> usize {
        self.buffer.as_ref().len()
    }

    /// Returns header, neighbors, and vector slice for the given node.
    #[inline(always)]
    #[must_use]
    pub fn get_node(&self, id: u32) -> Option<(&FlatNodeHeader, &[u32], &[u8])> {
        if id as usize >= self.node_count {
            return None;
        }
        let offset = id as usize * self.stride;
        let slice = self.buffer.as_ref();
        let ptr = unsafe { slice.as_ptr().add(offset) };
        let header = unsafe { &*(ptr as *const FlatNodeHeader) };
        let count = (header.neighbor_count as usize).min(self.max_m);

        let neighbors = unsafe {
            std::slice::from_raw_parts(
                ptr.add(std::mem::size_of::<FlatNodeHeader>()) as *const u32,
                count,
            )
        };
        let vector =
            unsafe { std::slice::from_raw_parts(ptr.add(self.vector_offset), self.vector_len) };

        Some((header, neighbors, vector))
    }

    /// Returns Layer-0 neighbors slice for the given node ID with zero allocations.
    #[inline(always)]
    #[must_use]
    pub fn get_neighbors(&self, id: u32) -> &[u32] {
        if id as usize >= self.node_count {
            return &[];
        }
        let offset = id as usize * self.stride;
        let slice = self.buffer.as_ref();
        let ptr = unsafe { slice.as_ptr().add(offset) };
        let header = unsafe { &*(ptr as *const FlatNodeHeader) };
        let count = (header.neighbor_count as usize).min(self.max_m);

        unsafe {
            std::slice::from_raw_parts(
                ptr.add(std::mem::size_of::<FlatNodeHeader>()) as *const u32,
                count,
            )
        }
    }

    /// Returns the raw vector byte slice for the given node ID with zero allocations.
    #[inline(always)]
    #[must_use]
    pub fn get_vector(&self, id: u32) -> &[u8] {
        if id as usize >= self.node_count {
            return &[];
        }
        let offset = id as usize * self.stride;
        let slice = self.buffer.as_ref();
        let ptr = unsafe { slice.as_ptr().add(offset) };
        unsafe { std::slice::from_raw_parts(ptr.add(self.vector_offset), self.vector_len) }
    }

    /// Returns the raw base pointer for hardware prefetching.
    #[inline(always)]
    #[must_use]
    pub fn get_node_ptr(&self, id: u32) -> *const u8 {
        let offset = id as usize * self.stride;
        let slice = self.buffer.as_ref();
        unsafe { slice.as_ptr().add(offset) }
    }

    /// Serializes this flat segment to disk.
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let mut file = File::create(path)?;

        // 64-byte metadata header
        let mut meta = [0u8; 64];
        meta[0..8].copy_from_slice(FLAT_L0_MAGIC);
        meta[8..16].copy_from_slice(&(self.node_count as u64).to_le_bytes());
        meta[16..24].copy_from_slice(&(self.stride as u64).to_le_bytes());
        meta[24..32].copy_from_slice(&(self.max_m as u64).to_le_bytes());
        meta[32..40].copy_from_slice(&(self.vector_len as u64).to_le_bytes());
        meta[40..48].copy_from_slice(&(self.vector_offset as u64).to_le_bytes());

        file.write_all(&meta)?;
        file.write_all(self.buffer.as_ref())?;
        file.flush()?;
        Ok(())
    }

    /// Loads a flat segment from disk using zero-copy memory mapping if available.
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let mut meta = [0u8; 64];
        file.read_exact(&mut meta)?;

        if &meta[0..8] != FLAT_L0_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid FlatLayer0 signature",
            ));
        }

        let node_count = u64::from_le_bytes(meta[8..16].try_into().unwrap()) as usize;
        let stride = u64::from_le_bytes(meta[16..24].try_into().unwrap()) as usize;
        let max_m = u64::from_le_bytes(meta[24..32].try_into().unwrap()) as usize;
        let vector_len = u64::from_le_bytes(meta[32..40].try_into().unwrap()) as usize;
        let vector_offset = u64::from_le_bytes(meta[40..48].try_into().unwrap()) as usize;

        let expected_payload_len = node_count * stride;

        #[cfg(feature = "persistence")]
        {
            let mmap = unsafe {
                memmap2::MmapOptions::new()
                    .offset(64)
                    .len(expected_payload_len)
                    .map(&file)?
            };
            Ok(Self {
                buffer: FlatBuffer::Mmap(mmap),
                node_count,
                stride,
                max_m,
                vector_len,
                vector_offset,
            })
        }

        #[cfg(not(feature = "persistence"))]
        {
            let mut buf = AlignedBuffer::new(expected_payload_len, 64);
            file.read_exact(buf.as_mut_slice())?;
            Ok(Self {
                buffer: FlatBuffer::Heap(buf),
                node_count,
                stride,
                max_m,
                vector_len,
                vector_offset,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_layer0_layout_alignment() {
        let (v_offset, stride) = FlatLayer0Segment::compute_layout(32, 128);
        assert_eq!(v_offset % 64, 0, "Vector offset must be 64B aligned");
        assert_eq!(stride % 64, 0, "Stride must be 64B aligned");
        assert_eq!(v_offset, 192); // 64 (header) + 128 (neighbors) = 192 (aligned)
        assert_eq!(stride, 320); // 192 + 128 = 320 (aligned)
    }

    #[test]
    fn test_flat_layer0_node_roundtrip() {
        let mut segment = FlatLayer0Segment::new(10, 16, 32);
        let neighbors = vec![1, 5, 8, 9];
        let vector = vec![42u8; 32];

        segment.set_node(3, &neighbors, &vector);

        let retrieved_neighbors = segment.get_neighbors(3);
        assert_eq!(retrieved_neighbors, &[1, 5, 8, 9]);

        let retrieved_vec = segment.get_vector(3);
        assert_eq!(retrieved_vec, &vector[..]);

        let (header, n, v) = segment.get_node(3).expect("node 3");
        assert_eq!(header.neighbor_count, 4);
        assert_eq!(header.flags, 1);
        assert_eq!(n, &[1, 5, 8, 9]);
        assert_eq!(v, &vector[..]);

        // Unset node
        assert!(segment.get_neighbors(0).is_empty());
    }

    #[test]
    fn test_flat_layer0_save_load() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("test_layer0.flat");

        let mut segment = FlatLayer0Segment::new(5, 8, 16);
        segment.set_node(2, &[0, 1, 3], &[7u8; 16]);
        segment.save_to_file(&path).expect("save");

        let loaded = FlatLayer0Segment::load_from_file(&path).expect("load");
        assert_eq!(loaded.node_count(), 5);
        assert_eq!(loaded.get_neighbors(2), &[0, 1, 3]);
        assert_eq!(loaded.get_vector(2), &[7u8; 16]);
    }
}
