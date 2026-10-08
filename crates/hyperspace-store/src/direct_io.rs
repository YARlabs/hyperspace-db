use arc_swap::ArcSwap;
use parking_lot::Mutex;
use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;

/// Standard NVMe / Direct I/O hardware alignment boundary (4096 bytes).
pub const DIRECT_IO_ALIGNMENT: usize = 4096;

/// A heap-allocated byte buffer aligned to the Direct I/O boundary (4096 bytes).
pub struct AlignedBuffer {
    ptr: *mut u8,
    layout: Layout,
    len: usize,
}

unsafe impl Send for AlignedBuffer {}
unsafe impl Sync for AlignedBuffer {}

impl AlignedBuffer {
    /// Creates a new zeroed buffer aligned to 4096 bytes with the specified capacity.
    /// The size is rounded up to the nearest multiple of `DIRECT_IO_ALIGNMENT`.
    pub fn new(min_size: usize) -> Self {
        let aligned_size = (min_size + DIRECT_IO_ALIGNMENT - 1) & !(DIRECT_IO_ALIGNMENT - 1);
        let layout =
            Layout::from_size_align(aligned_size.max(DIRECT_IO_ALIGNMENT), DIRECT_IO_ALIGNMENT)
                .expect("Invalid layout for AlignedBuffer");
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        Self {
            ptr,
            layout,
            len: aligned_size,
        }
    }

    /// Creates an aligned buffer initialized with the provided data slice.
    pub fn from_slice(data: &[u8]) -> Self {
        let buf = Self::new(data.len());
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), buf.ptr, data.len());
        }
        buf
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

impl std::ops::Deref for AlignedBuffer {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for AlignedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl Clone for AlignedBuffer {
    fn clone(&self) -> Self {
        let new_buf = Self::new(self.len);
        unsafe {
            std::ptr::copy_nonoverlapping(self.ptr, new_buf.ptr, self.len);
        }
        new_buf
    }
}

/// Direct I/O File wrapper supporting zero OS page cache pollution.
pub struct DirectFile {
    file: File,
    path: PathBuf,
    is_direct: bool,
}

impl DirectFile {
    /// Opens or creates a file with Direct I/O enabled where supported.
    pub fn open(path: &Path, read: bool, write: bool, create: bool) -> io::Result<Self> {
        let mut options = OpenOptions::new();
        options.read(read).write(write).create(create);

        #[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]
        {
            use std::os::unix::fs::OpenOptionsExt;
            // On Linux, try opening with O_DIRECT flag
            let direct_enabled =
                std::env::var("HS_DIRECT_IO").unwrap_or_else(|_| "1".to_string()) != "0";
            if direct_enabled {
                options.custom_flags(libc::O_DIRECT);
            }
        }

        #[cfg(target_os = "linux")]
        let (file, is_direct) = match options.open(path) {
            Ok(f) => {
                let direct_enabled =
                    std::env::var("HS_DIRECT_IO").unwrap_or_else(|_| "1".to_string()) != "0";
                (f, direct_enabled)
            }
            Err(_e) => {
                // If O_DIRECT failed (e.g. tmpfs or filesystem without O_DIRECT support), fallback to standard OpenOptions
                let f = OpenOptions::new()
                    .read(read)
                    .write(write)
                    .create(create)
                    .open(path)?;
                (f, false)
            }
        };

        #[cfg(not(target_os = "linux"))]
        let file = options.open(path)?;
        #[cfg(not(target_os = "linux"))]
        let mut is_direct = false;

        #[cfg(all(target_os = "macos", unix))]
        {
            let direct_enabled =
                std::env::var("HS_DIRECT_IO").unwrap_or_else(|_| "1".to_string()) != "0";
            if direct_enabled {
                // F_NOCACHE disables OS page cache buffering on macOS
                let fd = file.as_raw_fd();
                let res = unsafe { libc::fcntl(fd, libc::F_NOCACHE, 1) };
                if res != -1 {
                    is_direct = true;
                }
            }
        }

        Ok(Self {
            file,
            path: path.to_path_buf(),
            is_direct,
        })
    }

    #[inline]
    pub fn is_direct(&self) -> bool {
        self.is_direct
    }

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads aligned bytes from the specified file offset directly into an `AlignedBuffer`.
    pub fn read_aligned_at(&self, offset: u64, buf: &mut AlignedBuffer) -> io::Result<()> {
        #[cfg(unix)]
        {
            self.file.read_exact_at(buf.as_mut_slice(), offset)
        }
        #[cfg(not(unix))]
        {
            use std::io::Read;
            let mut f = &self.file;
            f.seek(SeekFrom::Start(offset))?;
            f.read_exact(buf.as_mut_slice())
        }
    }

    /// Writes an aligned slice to the specified file offset.
    pub fn write_aligned_at(&self, offset: u64, buf: &[u8]) -> io::Result<()> {
        #[cfg(unix)]
        {
            self.file.write_all_at(buf, offset)
        }
        #[cfg(not(unix))]
        {
            use std::io::Write;
            let mut f = &self.file;
            f.seek(SeekFrom::Start(offset))?;
            f.write_all(buf)
        }
    }

    pub fn set_len(&self, size: u64) -> io::Result<()> {
        self.file.set_len(size)
    }

    pub fn len(&self) -> io::Result<u64> {
        self.file.metadata().map(|m| m.len())
    }

    pub fn sync_all(&self) -> io::Result<()> {
        self.file.sync_all()
    }
}

/// Chunk segment for Direct I/O vector storage.
struct DirectSegment {
    file: DirectFile,
    #[allow(dead_code)]
    chunk_id: usize,
    #[allow(dead_code)]
    capacity: usize,
}

/// Direct I/O & Thread-Per-Core Vector Storage Engine (v4.0.0).
/// Stores vectors in 4096-byte aligned segments (`v4_chunk_N.hyp`),
/// bypassing kernel page cache locks and TLB shootdowns.
pub struct DirectVectorStore {
    base_path: PathBuf,
    element_size: usize,
    stride: usize,
    count: AtomicUsize,
    segments: ArcSwap<Vec<Arc<DirectSegment>>>,
    append_lock: Mutex<()>,
    direct_io_enabled: bool,
}

impl DirectVectorStore {
    const VECTORS_PER_SEGMENT: usize = 32768; // 32K vectors per segment chunk

    pub fn new(base_path: &Path, element_size: usize) -> io::Result<Self> {
        if !base_path.exists() {
            std::fs::create_dir_all(base_path)?;
        }

        // Each vector slot is aligned to at least 64 bytes (cache-line)
        let stride = (element_size + 63) & !63;

        let mut segments = Vec::new();
        let mut i = 0;
        loop {
            let path = base_path.join(format!("v4_chunk_{i}.hyp"));
            if !path.exists() {
                if i == 0 {
                    let seg = Self::create_segment(&path, i, stride)?;
                    segments.push(Arc::new(seg));
                }
                break;
            }
            let seg = Self::open_segment(&path, i, stride)?;
            segments.push(Arc::new(seg));
            i += 1;
        }

        let direct_io_enabled = segments.first().map(|s| s.file.is_direct()).unwrap_or(true);

        Ok(Self {
            base_path: base_path.to_path_buf(),
            element_size,
            stride,
            count: AtomicUsize::new(0),
            segments: ArcSwap::from_pointee(segments),
            append_lock: Mutex::new(()),
            direct_io_enabled,
        })
    }

    fn create_segment(path: &Path, chunk_id: usize, stride: usize) -> io::Result<DirectSegment> {
        let file = DirectFile::open(path, true, true, true)?;
        let total_bytes = (Self::VECTORS_PER_SEGMENT * stride) as u64;
        // Pre-allocate segment aligned to 4096 bytes
        let aligned_total =
            (total_bytes + DIRECT_IO_ALIGNMENT as u64 - 1) & !(DIRECT_IO_ALIGNMENT as u64 - 1);
        file.set_len(aligned_total)?;

        Ok(DirectSegment {
            file,
            chunk_id,
            capacity: Self::VECTORS_PER_SEGMENT,
        })
    }

    fn open_segment(path: &Path, chunk_id: usize, _stride: usize) -> io::Result<DirectSegment> {
        let file = DirectFile::open(path, true, true, false)?;
        Ok(DirectSegment {
            file,
            chunk_id,
            capacity: Self::VECTORS_PER_SEGMENT,
        })
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn is_direct_io(&self) -> bool {
        self.direct_io_enabled
    }

    /// Appends a vector to Direct I/O storage.
    pub fn append(&self, vector_bytes: &[u8]) -> Result<u32, String> {
        if vector_bytes.len() != self.element_size {
            return Err(format!(
                "Vector size mismatch: {} vs {}",
                vector_bytes.len(),
                self.element_size
            ));
        }

        let id = self.count.fetch_add(1, Ordering::SeqCst);
        let segment_idx = id / Self::VECTORS_PER_SEGMENT;
        let local_idx = id % Self::VECTORS_PER_SEGMENT;

        self.ensure_segment(segment_idx)?;

        let segs = self.segments.load();
        let segment = &segs[segment_idx];

        let offset = (local_idx * self.stride) as u64;
        // Direct write with block alignment
        segment
            .file
            .write_aligned_at(offset, vector_bytes)
            .map_err(|e| format!("Direct I/O write failed at offset {offset}: {e}"))?;

        Ok(id as u32)
    }

    /// Reads a vector by ID using direct aligned buffer read.
    pub fn read_vector(&self, id: u32, out: &mut [u8]) -> Result<(), String> {
        let id_val = id as usize;
        let segment_idx = id_val / Self::VECTORS_PER_SEGMENT;
        let local_idx = id_val % Self::VECTORS_PER_SEGMENT;

        let segs = self.segments.load();
        if segment_idx >= segs.len() {
            return Err(format!("Vector ID {id} out of bounds"));
        }

        let segment = &segs[segment_idx];
        let offset = (local_idx * self.stride) as u64;

        // Position of the 4KB block containing this vector
        let block_offset = offset & !(DIRECT_IO_ALIGNMENT as u64 - 1);
        let in_block_offset = (offset - block_offset) as usize;

        let mut aligned_buf = AlignedBuffer::new(self.stride + DIRECT_IO_ALIGNMENT);
        segment
            .file
            .read_aligned_at(block_offset, &mut aligned_buf)
            .map_err(|e| format!("Direct I/O read failed at offset {block_offset}: {e}"))?;

        let end = (in_block_offset + self.element_size).min(aligned_buf.len());
        out.copy_from_slice(&aligned_buf[in_block_offset..end]);
        Ok(())
    }

    fn ensure_segment(&self, segment_idx: usize) -> Result<(), String> {
        let segs = self.segments.load();
        if segment_idx < segs.len() {
            return Ok(());
        }

        let _guard = self.append_lock.lock();
        let segs = self.segments.load();
        if segment_idx < segs.len() {
            return Ok(());
        }

        let mut new_segs = (**segs).clone();
        for i in new_segs.len()..=segment_idx {
            let path = self.base_path.join(format!("v4_chunk_{i}.hyp"));
            let seg = Self::create_segment(&path, i, self.stride)
                .map_err(|e| format!("Failed to create direct segment {i}: {e}"))?;
            new_segs.push(Arc::new(seg));
        }

        self.segments.store(Arc::new(new_segs));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_aligned_buffer_alignment() {
        let buf = AlignedBuffer::new(100);
        assert_eq!(buf.len() % DIRECT_IO_ALIGNMENT, 0);
        assert_eq!(buf.as_ptr() as usize % DIRECT_IO_ALIGNMENT, 0);

        let data = vec![42u8; 5000];
        let buf_data = AlignedBuffer::from_slice(&data);
        assert_eq!(buf_data.as_ptr() as usize % DIRECT_IO_ALIGNMENT, 0);
        assert_eq!(&buf_data[..5000], &data[..]);
    }

    #[test]
    fn test_direct_vector_store_roundtrip() {
        let dir = tempdir().unwrap();
        let element_size = 128; // e.g. 128-byte quantized vector
        let store = DirectVectorStore::new(dir.path(), element_size).unwrap();

        let v0 = vec![1u8; element_size];
        let v1 = vec![2u8; element_size];
        let v2 = vec![3u8; element_size];

        let id0 = store.append(&v0).unwrap();
        let id1 = store.append(&v1).unwrap();
        let id2 = store.append(&v2).unwrap();

        assert_eq!(id0, 0);
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let mut out = vec![0u8; element_size];
        store.read_vector(0, &mut out).unwrap();
        assert_eq!(out, v0);

        store.read_vector(1, &mut out).unwrap();
        assert_eq!(out, v1);

        store.read_vector(2, &mut out).unwrap();
        assert_eq!(out, v2);
    }
}
