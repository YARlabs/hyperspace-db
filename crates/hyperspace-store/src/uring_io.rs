//! Batched positional reads.
//!
//! On Linux with the `io-uring` cargo feature enabled, a batch of reads is submitted to the
//! kernel through a single `io_uring_enter` syscall (instead of one `pread` syscall per
//! request) and completed in parallel by the block layer. This is what makes top-K payload
//! fetching and Direct-I/O vector gathers scale on NVMe.
//!
//! Everywhere else (macOS, Windows, Linux without the feature, kernels without io_uring,
//! seccomp-restricted containers) the exact same API transparently falls back to a plain
//! `read_exact_at` loop, so callers never need to branch on platform.
//!
//! Runtime opt-out: `HS_IO_URING=0`.

use std::fs::File;
use std::io;

/// One positional read: fill `buf` completely from `offset`.
pub struct ReadRequest {
    pub offset: u64,
    pub buf: Vec<u8>,
    /// Set to `true` once `buf` has been completely filled.
    pub ok: bool,
}

impl ReadRequest {
    pub fn new(offset: u64, len: usize) -> Self {
        Self {
            offset,
            buf: vec![0u8; len],
            ok: false,
        }
    }
}

/// Raw read descriptor used for caller-owned (e.g. 4096-aligned) buffers.
pub struct RawRead {
    pub offset: u64,
    pub ptr: *mut u8,
    pub len: usize,
}

/// Returns `true` if the io_uring backend is compiled in and not disabled at runtime.
pub fn io_uring_active() -> bool {
    #[cfg(all(target_os = "linux", feature = "io-uring"))]
    {
        linux::enabled()
    }
    #[cfg(not(all(target_os = "linux", feature = "io-uring")))]
    {
        false
    }
}

/// Reads every request in the batch. Individual failures leave `ok == false` (mirroring the
/// tolerant behaviour of the previous per-item `read_exact_at` loop).
pub fn read_batch_at(file: &File, reqs: &mut [ReadRequest]) -> io::Result<()> {
    if reqs.is_empty() {
        return Ok(());
    }
    #[cfg(all(target_os = "linux", feature = "io-uring"))]
    if reqs.len() >= 2 && linux::enabled() {
        let raw: Vec<RawRead> = reqs
            .iter_mut()
            .map(|r| RawRead {
                offset: r.offset,
                ptr: r.buf.as_mut_ptr(),
                len: r.buf.len(),
            })
            .collect();
        // SAFETY: every pointer refers to a live `Vec<u8>` of exactly `len` bytes owned by
        // `reqs`, which outlives the call; all completions are reaped before returning.
        if unsafe { linux::submit_reads(file, &raw) }.is_ok() {
            for r in reqs.iter_mut() {
                r.ok = true;
            }
            return Ok(());
        }
    }
    read_batch_fallback(file, reqs);
    Ok(())
}

/// Raw variant for pre-aligned buffers (Direct I/O). All-or-nothing: returns `Err` if any
/// read failed so the caller can retry through its synchronous path.
///
/// # Safety
/// Every `RawRead` must point to writable memory of at least `len` bytes that stays valid
/// for the duration of the call and is not aliased by another request.
pub unsafe fn read_raw_batch_at(file: &File, reqs: &[RawRead]) -> io::Result<()> {
    #[cfg(all(target_os = "linux", feature = "io-uring"))]
    if reqs.len() >= 2 && linux::enabled() {
        return linux::submit_reads(file, reqs);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        for r in reqs {
            let buf = std::slice::from_raw_parts_mut(r.ptr, r.len);
            file.read_exact_at(buf, r.offset)?;
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (file, reqs);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "positional reads unsupported on this platform",
        ))
    }
}

fn read_batch_fallback(file: &File, reqs: &mut [ReadRequest]) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::FileExt;
        for r in reqs.iter_mut() {
            r.ok = file.read_exact_at(&mut r.buf, r.offset).is_ok();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = file;
        for r in reqs.iter_mut() {
            r.ok = false;
        }
    }
}

#[cfg(all(target_os = "linux", feature = "io-uring"))]
mod linux {
    use super::RawRead;
    use io_uring::{opcode, types, IoUring};
    use std::cell::RefCell;
    use std::fs::File;
    use std::io;
    use std::os::unix::io::AsRawFd;
    use std::sync::atomic::{AtomicU8, Ordering};

    /// Submission queue depth per thread-local ring.
    const RING_ENTRIES: u32 = 256;

    // 0 = unknown, 1 = enabled, 2 = disabled (env opt-out or kernel refused to create a ring).
    static STATE: AtomicU8 = AtomicU8::new(0);

    thread_local! {
        static RING: RefCell<Option<IoUring>> = const { RefCell::new(None) };
    }

    pub(super) fn enabled() -> bool {
        match STATE.load(Ordering::Relaxed) {
            1 => true,
            2 => false,
            _ => {
                let off = std::env::var("HS_IO_URING").is_ok_and(|v| v.trim() == "0");
                let ok = !off && IoUring::new(8).is_ok();
                STATE.store(if ok { 1 } else { 2 }, Ordering::Relaxed);
                ok
            }
        }
    }

    pub(super) unsafe fn submit_reads(file: &File, reqs: &[RawRead]) -> io::Result<()> {
        let fd = types::Fd(file.as_raw_fd());
        RING.with(|cell| {
            let mut slot = cell.borrow_mut();
            if slot.is_none() {
                *slot = Some(IoUring::new(RING_ENTRIES)?);
            }
            let result = {
                let ring = slot.as_mut().expect("ring initialised above");
                run_batches(ring, fd, reqs)
            };
            if result.is_err() {
                // A failed submit may leave queued entries behind; start from a clean ring.
                *slot = None;
            }
            result
        })
    }

    unsafe fn run_batches(ring: &mut IoUring, fd: types::Fd, reqs: &[RawRead]) -> io::Result<()> {
        for group in reqs.chunks(RING_ENTRIES as usize) {
            for (i, r) in group.iter().enumerate() {
                let len = u32::try_from(r.len)
                    .map_err(|_| io::Error::other("read larger than u32::MAX"))?;
                let entry = opcode::Read::new(fd, r.ptr, len)
                    .offset(r.offset)
                    .build()
                    .user_data(i as u64);
                ring.submission()
                    .push(&entry)
                    .map_err(|_| io::Error::other("io_uring submission queue full"))?;
            }
            ring.submit_and_wait(group.len())?;

            let mut completed = 0usize;
            let mut first_err: Option<io::Error> = None;
            for cqe in ring.completion() {
                completed += 1;
                let idx = cqe.user_data() as usize;
                let code = cqe.result();
                if code < 0 {
                    first_err.get_or_insert_with(|| io::Error::from_raw_os_error(-code));
                } else if usize::try_from(code).ok() != Some(group[idx].len) {
                    first_err.get_or_insert_with(|| {
                        io::Error::new(io::ErrorKind::UnexpectedEof, "short io_uring read")
                    });
                }
            }
            if let Some(e) = first_err {
                return Err(e);
            }
            if completed != group.len() {
                return Err(io::Error::other("io_uring completion count mismatch"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn batch_reads_match_file_contents() {
        let mut tmp = tempfile::tempfile().unwrap();
        let data: Vec<u8> = (0..65536u32).map(|i| (i * 31 % 251) as u8).collect();
        tmp.write_all(&data).unwrap();

        let mut reqs: Vec<ReadRequest> = [(0u64, 17usize), (1000, 4096), (65000, 536), (5, 1)]
            .iter()
            .map(|&(o, l)| ReadRequest::new(o, l))
            .collect();
        read_batch_at(&tmp, &mut reqs).unwrap();
        for r in &reqs {
            assert!(r.ok);
            assert_eq!(
                &r.buf[..],
                &data[r.offset as usize..r.offset as usize + r.buf.len()]
            );
        }
    }

    #[test]
    fn out_of_range_request_is_reported_not_panicking() {
        let mut tmp = tempfile::tempfile().unwrap();
        tmp.write_all(&[7u8; 128]).unwrap();
        let mut reqs = vec![ReadRequest::new(0, 64), ReadRequest::new(100, 64)];
        read_batch_at(&tmp, &mut reqs).unwrap();
        assert!(reqs[0].ok);
        assert!(!reqs[1].ok);
    }
}
