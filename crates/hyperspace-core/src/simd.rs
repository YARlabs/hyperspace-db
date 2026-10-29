//! Runtime-dispatched SIMD kernels for the HNSW hot path.
//!
//! The portable `as_chunks::<4>()` kernels rely on LLVM auto-vectorisation and are limited to
//! the baseline ISA the binary was compiled for (SSE2 on generic `x86_64` builds). The kernels
//! in this module are compiled with explicit `#[target_feature]` and selected **once at
//! runtime** via CPU feature detection:
//!
//! | Arch      | Preferred        | Fallback chain                    |
//! |-----------|------------------|-----------------------------------|
//! | `x86_64`  | AVX-512F (16xf32)| AVX2 + FMA (8xf32) -> scalar      |
//! | `aarch64` | NEON (4xf32 FMA) | scalar                            |
//! | other     | scalar (4 accumulators)                              |
//!
//! Set `HS_SIMD=scalar|avx2|avx512|neon` to force a backend (useful for A/B benchmarking and
//! for hosts where AVX-512 down-clocking is undesirable). A forced backend that the CPU does
//! not support is ignored.
//!
//! All kernels read vector data as little-endian `f32` straight from a byte slice with
//! unaligned loads, so they can run directly on mmap / RAM storage without any copy.

// Pointers are only dereferenced via unaligned vector loads (`loadu` / `vld1q`).
#![allow(clippy::cast_ptr_alignment)]

use std::sync::atomic::{AtomicU8, Ordering};

/// Instruction-set backend selected for the f32 kernels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimdBackend {
    Scalar,
    Avx2,
    Avx512,
    Neon,
}

impl SimdBackend {
    pub fn name(self) -> &'static str {
        match self {
            Self::Scalar => "scalar",
            Self::Avx2 => "avx2+fma",
            Self::Avx512 => "avx512f",
            Self::Neon => "neon",
        }
    }

    const fn to_u8(self) -> u8 {
        match self {
            Self::Scalar => 1,
            Self::Avx2 => 2,
            Self::Avx512 => 3,
            Self::Neon => 4,
        }
    }

    const fn from_u8(v: u8) -> Self {
        match v {
            2 => Self::Avx2,
            3 => Self::Avx512,
            4 => Self::Neon,
            _ => Self::Scalar,
        }
    }
}

static BACKEND: AtomicU8 = AtomicU8::new(0);

fn supported(backend: SimdBackend) -> bool {
    match backend {
        SimdBackend::Scalar => true,
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx2 => is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma"),
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx512 => is_x86_feature_detected!("avx512f"),
        #[cfg(target_arch = "aarch64")]
        SimdBackend::Neon => std::arch::is_aarch64_feature_detected!("neon"),
        #[allow(unreachable_patterns)]
        _ => false,
    }
}

fn detect() -> SimdBackend {
    if let Ok(forced) = std::env::var("HS_SIMD") {
        let want = match forced.trim().to_ascii_lowercase().as_str() {
            "scalar" | "off" | "none" => Some(SimdBackend::Scalar),
            "avx2" => Some(SimdBackend::Avx2),
            "avx512" | "avx512f" => Some(SimdBackend::Avx512),
            "neon" => Some(SimdBackend::Neon),
            _ => None,
        };
        if let Some(b) = want {
            if supported(b) {
                return b;
            }
        }
    }
    for candidate in [SimdBackend::Avx512, SimdBackend::Avx2, SimdBackend::Neon] {
        if supported(candidate) {
            return candidate;
        }
    }
    SimdBackend::Scalar
}

/// Returns the (cached) backend used by the f32 kernels.
#[inline]
pub fn backend() -> SimdBackend {
    let cached = BACKEND.load(Ordering::Relaxed);
    if cached != 0 {
        return SimdBackend::from_u8(cached);
    }
    let b = detect();
    BACKEND.store(b.to_u8(), Ordering::Relaxed);
    b
}

/// Number of floats processed per early-abandon check.
const THRESHOLD_BLOCK: usize = 64;

#[inline(always)]
fn read_f32(bytes: &[u8], i: usize) -> f32 {
    let o = i * 4;
    f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]])
}

/// Portable reference kernel (4 independent accumulators). `bytes.len() >= n * 4`, `b.len() >= n`.
fn l2_sq_scalar(bytes: &[u8], b: &[f32]) -> f32 {
    let n = b.len();
    let (chunks, rem) = bytes[..n * 4].as_chunks::<16>();
    let mut acc = [0.0f32; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f32::from_le_bytes([c[l * 4], c[l * 4 + 1], c[l * 4 + 2], c[l * 4 + 3]]);
            let d = a - bc[l];
            acc[l] += d * d;
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        let d = read_f32(&bytes[done * 4..], i) - b[done + i];
        sum += d * d;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn l2_sq_avx512(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps, _mm512_sub_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let d0 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
        );
        let d1 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm512_loadu_ps(bp.add(i + 16)),
        );
        acc0 = _mm512_fmadd_ps(d0, d0, acc0);
        acc1 = _mm512_fmadd_ps(d1, d1, acc1);
        i += 32;
    }
    if i + 16 <= n {
        let d = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
        );
        acc0 = _mm512_fmadd_ps(d, d, acc0);
        i += 16;
    }
    let mut sum = _mm512_reduce_add_ps(acc0) + _mm512_reduce_add_ps(acc1);
    while i < n {
        let d = read_f32(bytes, i) - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn l2_sq_avx2(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m256, _mm256_castps256_ps128, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_loadu_ps,
        _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehl_ps,
        _mm_shuffle_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let d0 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        let d1 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
            _mm256_loadu_ps(bp.add(i + 8)),
        );
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
        acc1 = _mm256_fmadd_ps(d1, d1, acc1);
        i += 16;
    }
    if i + 8 <= n {
        let d = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        acc0 = _mm256_fmadd_ps(d, d, acc0);
        i += 8;
    }
    let acc = std::arch::x86_64::_mm256_add_ps(acc0, acc1);
    let lo = _mm256_castps256_ps128(acc);
    let hi = _mm256_extractf128_ps(acc, 1);
    let s4 = _mm_add_ps(lo, hi);
    let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
    let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
    let mut sum = _mm_cvtss_f32(s1);
    while i < n {
        let d = read_f32(bytes, i) - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn l2_sq_neon(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vaddvq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vsubq_f32};
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let d0 = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        let d1 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 4) * 4)),
            vld1q_f32(bp.add(i + 4)),
        );
        acc0 = vfmaq_f32(acc0, d0, d0);
        acc1 = vfmaq_f32(acc1, d1, d1);
        i += 8;
    }
    if i + 4 <= n {
        let d = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        acc0 = vfmaq_f32(acc0, d, d);
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(acc0, acc1));
    while i < n {
        let d = read_f32(bytes, i) - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

/// Dispatches one contiguous range to the selected backend.
#[inline]
fn l2_sq_dispatch(backend: SimdBackend, bytes: &[u8], b: &[f32]) -> f32 {
    debug_assert!(bytes.len() >= b.len() * 4);
    match backend {
        // SAFETY: the backend is only ever selected after the matching CPU feature was
        // detected at runtime, and `bytes.len() >= b.len() * 4` is guaranteed by callers.
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 => unsafe { l2_sq_avx512(bytes, b) },
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx2 => unsafe { l2_sq_avx2(bytes, b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { l2_sq_neon(bytes, b) },
        _ => l2_sq_scalar(bytes, b),
    }
}

/// Squared L2 distance between little-endian `f32` bytes and a float query.
///
/// Processes `min(bytes.len() / 4, b.len())` elements. When `threshold` is given the kernel
/// abandons early (returning the partial sum, which is already `> threshold`) as soon as the
/// accumulated distance exceeds it; the check runs once per 64-float block.
#[inline]
pub fn l2_sq_f32_bytes(bytes: &[u8], b: &[f32], threshold: Option<f32>) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    let (bytes, b) = (&bytes[..n * 4], &b[..n]);
    let backend = backend();
    let Some(t) = threshold else {
        return l2_sq_dispatch(backend, bytes, b);
    };
    if n <= THRESHOLD_BLOCK {
        return l2_sq_dispatch(backend, bytes, b);
    }
    let mut sum = 0.0f32;
    let mut i = 0usize;
    while i < n {
        let end = (i + THRESHOLD_BLOCK).min(n);
        sum += l2_sq_dispatch(backend, &bytes[i * 4..end * 4], &b[i..end]);
        if sum > t {
            return sum;
        }
        i = end;
    }
    sum
}

/// Squared L2 distance between two `f32` slices (`min(len)` elements).
#[inline]
pub fn l2_sq_f32(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    // SAFETY: reinterpreting `&[f32]` as bytes is always valid (no padding, align 1 target).
    let bytes = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
    #[cfg(target_endian = "little")]
    {
        l2_sq_f32_bytes(bytes, &b[..n], None)
    }
    #[cfg(not(target_endian = "little"))]
    {
        let _ = bytes;
        a[..n]
            .iter()
            .zip(&b[..n])
            .map(|(x, y)| (x - y) * (x - y))
            .sum()
    }
}

// ============================================================================
// Dot Product (Inner Product) SIMD Kernels
// ============================================================================

fn dot_scalar(bytes: &[u8], b: &[f32]) -> f32 {
    let n = b.len();
    let (chunks, rem) = bytes[..n * 4].as_chunks::<16>();
    let mut acc = [0.0f32; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f32::from_le_bytes([c[l * 4], c[l * 4 + 1], c[l * 4 + 2], c[l * 4 + 3]]);
            acc[l] += a * bc[l];
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        sum += read_f32(&bytes[done * 4..], i) * b[done + i];
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn dot_avx512(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        acc0 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
            acc0,
        );
        acc1 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm512_loadu_ps(bp.add(i + 16)),
            acc1,
        );
        i += 32;
    }
    if i + 16 <= n {
        acc0 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
            acc0,
        );
        i += 16;
    }
    let mut sum = _mm512_reduce_add_ps(acc0) + _mm512_reduce_add_ps(acc1);
    while i < n {
        sum += read_f32(bytes, i) * b[i];
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn dot_avx2(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_extractf128_ps, _mm256_fmadd_ps,
        _mm256_loadu_ps, _mm256_setzero_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehl_ps,
        _mm_shuffle_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        acc0 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
            acc0,
        );
        acc1 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
            _mm256_loadu_ps(bp.add(i + 8)),
            acc1,
        );
        i += 16;
    }
    if i + 8 <= n {
        acc0 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
            acc0,
        );
        i += 8;
    }
    let acc = _mm256_add_ps(acc0, acc1);
    let lo = _mm256_castps256_ps128(acc);
    let hi = _mm256_extractf128_ps(acc, 1);
    let s4 = _mm_add_ps(lo, hi);
    let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
    let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
    let mut sum = _mm_cvtss_f32(s1);
    while i < n {
        sum += read_f32(bytes, i) * b[i];
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn dot_neon(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vaddvq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32};
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        acc0 = vfmaq_f32(acc0, vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        acc1 = vfmaq_f32(
            acc1,
            vld1q_f32(ap.byte_add((i + 4) * 4)),
            vld1q_f32(bp.add(i + 4)),
        );
        i += 8;
    }
    if i + 4 <= n {
        acc0 = vfmaq_f32(acc0, vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(acc0, acc1));
    while i < n {
        sum += read_f32(bytes, i) * b[i];
        i += 1;
    }
    sum
}

#[inline]
fn dot_dispatch(backend: SimdBackend, bytes: &[u8], b: &[f32]) -> f32 {
    debug_assert!(bytes.len() >= b.len() * 4);
    match backend {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 => unsafe { dot_avx512(bytes, b) },
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx2 => unsafe { dot_avx2(bytes, b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { dot_neon(bytes, b) },
        _ => dot_scalar(bytes, b),
    }
}

/// Inner product between little-endian `f32` bytes and a float query slice.
#[inline]
pub fn dot_f32_bytes(bytes: &[u8], b: &[f32]) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return 0.0;
    }
    dot_dispatch(backend(), &bytes[..n * 4], &b[..n])
}

/// Inner product between two `f32` slices.
#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len().min(b.len());
    let bytes = unsafe { std::slice::from_raw_parts(a.as_ptr().cast::<u8>(), n * 4) };
    #[cfg(target_endian = "little")]
    {
        dot_f32_bytes(bytes, &b[..n])
    }
    #[cfg(not(target_endian = "little"))]
    {
        let _ = bytes;
        a[..n].iter().zip(&b[..n]).map(|(x, y)| x * y).sum()
    }
}

// ============================================================================
// Manhattan (L1) & Chebyshev (L-inf) SIMD Kernels
// ============================================================================

fn l1_scalar(bytes: &[u8], b: &[f32]) -> f32 {
    let n = b.len();
    let (chunks, rem) = bytes[..n * 4].as_chunks::<16>();
    let mut acc = [0.0f32; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f32::from_le_bytes([c[l * 4], c[l * 4 + 1], c[l * 4 + 2], c[l * 4 + 3]]);
            acc[l] += (a - bc[l]).abs();
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        sum += (read_f32(&bytes[done * 4..], i) - b[done + i]).abs();
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn l1_neon(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::aarch64::{vabdq_f32, vaddq_f32, vaddvq_f32, vdupq_n_f32, vld1q_f32};
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        acc0 = vaddq_f32(
            acc0,
            vabdq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i))),
        );
        acc1 = vaddq_f32(
            acc1,
            vabdq_f32(
                vld1q_f32(ap.byte_add((i + 4) * 4)),
                vld1q_f32(bp.add(i + 4)),
            ),
        );
        i += 8;
    }
    if i + 4 <= n {
        acc0 = vaddq_f32(
            acc0,
            vabdq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i))),
        );
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(acc0, acc1));
    while i < n {
        sum += (read_f32(bytes, i) - b[i]).abs();
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2")]
unsafe fn l1_avx2(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m256, _mm256_add_ps, _mm256_andnot_ps, _mm256_castps256_ps128, _mm256_extractf128_ps,
        _mm256_loadu_ps, _mm256_set1_ps, _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss,
        _mm_cvtss_f32, _mm_movehl_ps, _mm_shuffle_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let sign_mask = _mm256_set1_ps(-0.0f32);
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let d0 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        let d1 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
            _mm256_loadu_ps(bp.add(i + 8)),
        );
        acc0 = _mm256_add_ps(acc0, _mm256_andnot_ps(sign_mask, d0));
        acc1 = _mm256_add_ps(acc1, _mm256_andnot_ps(sign_mask, d1));
        i += 16;
    }
    if i + 8 <= n {
        let d = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        acc0 = _mm256_add_ps(acc0, _mm256_andnot_ps(sign_mask, d));
        i += 8;
    }
    let acc = _mm256_add_ps(acc0, acc1);
    let lo = _mm256_castps256_ps128(acc);
    let hi = _mm256_extractf128_ps(acc, 1);
    let s4 = _mm_add_ps(lo, hi);
    let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
    let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
    let mut sum = _mm_cvtss_f32(s1);
    while i < n {
        sum += (read_f32(bytes, i) - b[i]).abs();
        i += 1;
    }
    sum
}

#[inline]
fn l1_dispatch(backend: SimdBackend, bytes: &[u8], b: &[f32]) -> f32 {
    debug_assert!(bytes.len() >= b.len() * 4);
    match backend {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 | SimdBackend::Avx2 => unsafe { l1_avx2(bytes, b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { l1_neon(bytes, b) },
        _ => l1_scalar(bytes, b),
    }
}

/// Manhattan (L1) distance between little-endian `f32` bytes and a float query slice.
#[inline]
pub fn l1_f32_bytes(bytes: &[u8], b: &[f32], threshold: Option<f32>) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    let (bytes, b) = (&bytes[..n * 4], &b[..n]);
    let backend = backend();
    let Some(t) = threshold else {
        return l1_dispatch(backend, bytes, b);
    };
    if n <= THRESHOLD_BLOCK {
        return l1_dispatch(backend, bytes, b);
    }
    let mut sum = 0.0f32;
    let mut i = 0usize;
    while i < n {
        let end = (i + THRESHOLD_BLOCK).min(n);
        sum += l1_dispatch(backend, &bytes[i * 4..end * 4], &b[i..end]);
        if sum > t {
            return sum;
        }
        i = end;
    }
    sum
}

/// Chebyshev (L-infinity) distance between little-endian `f32` bytes and a float query slice.
#[inline]
pub fn linf_f32_bytes(bytes: &[u8], b: &[f32], threshold: Option<f32>) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    let mut max_diff = 0.0f32;
    let t = threshold.unwrap_or(f32::INFINITY);
    let (chunks, rem) = bytes[..n * 4].as_chunks::<16>();
    for (c, bc) in chunks.iter().zip(b[..n].as_chunks::<4>().0) {
        let d0 = (f32::from_le_bytes([c[0], c[1], c[2], c[3]]) - bc[0]).abs();
        let d1 = (f32::from_le_bytes([c[4], c[5], c[6], c[7]]) - bc[1]).abs();
        let d2 = (f32::from_le_bytes([c[8], c[9], c[10], c[11]]) - bc[2]).abs();
        let d3 = (f32::from_le_bytes([c[12], c[13], c[14], c[15]]) - bc[3]).abs();
        max_diff = max_diff.max(d0.max(d1).max(d2.max(d3)));
        if max_diff > t {
            return max_diff;
        }
    }
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        let d = (read_f32(&bytes[done * 4..], i) - b[done + i]).abs();
        if d > max_diff {
            max_diff = d;
        }
    }
    max_diff
}

// ============================================================================
// Fused Single-Pass Norms + Distance Kernels (Poincaré, Lorentz, Cosine, Tanimoto)
// ============================================================================

/// Computes `(l2_sq(a, b), norm_a_sq, norm_b_sq)` in a single fused memory pass over `bytes` and `b`.
#[inline]
pub fn norms_and_l2_f32_bytes(bytes: &[u8], b: &[f32]) -> (f32, f32, f32) {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return (0.0, 0.0, 0.0);
    }
    let bytes = &bytes[..n * 4];
    let b = &b[..n];

    #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
    if backend() == SimdBackend::Neon {
        unsafe {
            use std::arch::aarch64::{vaddvq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vsubq_f32};
            let ap = bytes.as_ptr().cast::<f32>();
            let bp = b.as_ptr();
            let mut acc_d = vdupq_n_f32(0.0);
            let mut acc_a = vdupq_n_f32(0.0);
            let mut acc_b = vdupq_n_f32(0.0);
            let mut i = 0usize;
            while i + 4 <= n {
                let va = vld1q_f32(ap.byte_add(i * 4));
                let vb = vld1q_f32(bp.add(i));
                let vd = vsubq_f32(va, vb);
                acc_d = vfmaq_f32(acc_d, vd, vd);
                acc_a = vfmaq_f32(acc_a, va, va);
                acc_b = vfmaq_f32(acc_b, vb, vb);
                i += 4;
            }
            let mut d_sq = vaddvq_f32(acc_d);
            let mut na_sq = vaddvq_f32(acc_a);
            let mut nb_sq = vaddvq_f32(acc_b);
            while i < n {
                let a = read_f32(bytes, i);
                let bv = b[i];
                let d = a - bv;
                d_sq += d * d;
                na_sq += a * a;
                nb_sq += bv * bv;
                i += 1;
            }
            return (d_sq, na_sq, nb_sq);
        }
    }

    let (chunks, rem) = bytes.as_chunks::<16>();
    let mut d_acc = [0.0f32; 4];
    let mut a_acc = [0.0f32; 4];
    let mut b_acc = [0.0f32; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f32::from_le_bytes([c[l * 4], c[l * 4 + 1], c[l * 4 + 2], c[l * 4 + 3]]);
            let bv = bc[l];
            let d = a - bv;
            d_acc[l] += d * d;
            a_acc[l] += a * a;
            b_acc[l] += bv * bv;
        }
    }
    let mut d_sq = (d_acc[0] + d_acc[1]) + (d_acc[2] + d_acc[3]);
    let mut na_sq = (a_acc[0] + a_acc[1]) + (a_acc[2] + a_acc[3]);
    let mut nb_sq = (b_acc[0] + b_acc[1]) + (b_acc[2] + b_acc[3]);
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        let a = read_f32(&bytes[done * 4..], i);
        let bv = b[done + i];
        let d = a - bv;
        d_sq += d * d;
        na_sq += a * a;
        nb_sq += bv * bv;
    }
    (d_sq, na_sq, nb_sq)
}

/// Zero-copy Poincaré ball distance directly on little-endian `f32` bytes and a query slice.
/// Eliminates all heap allocations (`Vec<f32>` / `Vec<f64>`) on the HNSW traversal hot path.
#[inline]
pub fn poincare_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    let (diff_sq_f32, u_norm_sq_f32, v_norm_sq_f32) = norms_and_l2_f32_bytes(bytes, b);
    let diff_sq = f64::from(diff_sq_f32);
    let u_norm_sq = f64::from(u_norm_sq_f32);
    let v_norm_sq = f64::from(v_norm_sq_f32);
    let eps = 1e-6;
    let alpha = (1.0 - u_norm_sq).max(eps);
    let beta = (1.0 - v_norm_sq).max(eps);
    let arg = 1.0 + 2.0 * diff_sq / (alpha * beta);
    arg.max(1.0).acosh()
}

/// Zero-copy Lorentz hyperboloid (Minkowski) distance directly on little-endian `f32` bytes.
/// Eliminates all heap allocations (`Vec<f32>` / `Vec<f64>`) on the HNSW traversal hot path.
#[inline]
pub fn lorentz_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return 0.0;
    }
    let u0 = f64::from(read_f32(bytes, 0));
    let v0 = f64::from(b[0]);
    let spatial_dot = if n > 1 {
        f64::from(dot_f32_bytes(&bytes[4..n * 4], &b[1..n]))
    } else {
        0.0
    };
    let minkowski_ip = u0 * v0 - spatial_dot;
    minkowski_ip.max(1.0).acosh()
}

/// Zero-copy Cosine distance (`1.0 - cos_sim`) directly on little-endian `f32` bytes.
/// Handles both unit-normalized and unnormalized vectors in a single pass.
#[inline]
pub fn cosine_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    let (diff_sq, na_sq, nb_sq) = norms_and_l2_f32_bytes(bytes, b);
    let na = f64::from(na_sq);
    let nb = f64::from(nb_sq);
    if na <= 1e-24 || nb <= 1e-24 {
        return 1.0;
    }
    let dot = 0.5 * (na + nb - f64::from(diff_sq));
    let cos = (dot / (na * nb).sqrt()).clamp(-1.0, 1.0);
    1.0 - cos
}

/// Zero-copy Tanimoto / Jaccard continuous distance directly on little-endian `f32` bytes.
#[inline]
pub fn jaccard_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    let (diff_sq, na_sq, nb_sq) = norms_and_l2_f32_bytes(bytes, b);
    let na = f64::from(na_sq);
    let nb = f64::from(nb_sq);
    let dot = 0.5 * (na + nb - f64::from(diff_sq));
    let denom = na + nb - dot;
    if denom <= 1e-24 {
        return 0.0;
    }
    (1.0 - (dot / denom)).clamp(0.0, 2.0)
}

/// Zero-copy Hamming distance (fraction of differing sign/non-zero coordinates) on `f32` bytes.
#[inline]
pub fn hamming_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut diff = 0usize;
    for i in 0..n {
        let a = read_f32(bytes, i);
        let bv = b[i];
        if (a > 0.0) != (bv > 0.0) || (a == 0.0) != (bv == 0.0) {
            diff += 1;
        }
    }
    diff as f64 / n as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_bytes(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }

    fn reference(a: &[f32], b: &[f32]) -> f64 {
        a.iter()
            .zip(b)
            .map(|(x, y)| {
                let d = f64::from(*x) - f64::from(*y);
                d * d
            })
            .sum()
    }

    fn sample(n: usize, seed: f32) -> Vec<f32> {
        (0..n)
            .map(|i| ((i as f32 * 0.37 + seed).sin()) * 3.0)
            .collect()
    }

    #[test]
    fn matches_reference_for_all_lengths() {
        for n in 0..=300 {
            let a = sample(n, 0.1);
            let b = sample(n, 1.7);
            let want = reference(&a, &b);
            let got = f64::from(l2_sq_f32_bytes(&to_bytes(&a), &b, None));
            assert!(
                (got - want).abs() <= want.abs() * 1e-4 + 1e-4,
                "n={n} backend={:?} got={got} want={want}",
                backend()
            );
            let got2 = f64::from(l2_sq_f32(&a, &b));
            assert!((got2 - want).abs() <= want.abs() * 1e-4 + 1e-4);
        }
    }

    #[test]
    fn every_supported_backend_agrees() {
        let a = sample(771, 0.3);
        let b = sample(771, 2.9);
        let bytes = to_bytes(&a);
        let want = reference(&a, &b);
        for be in [
            SimdBackend::Scalar,
            SimdBackend::Avx2,
            SimdBackend::Avx512,
            SimdBackend::Neon,
        ] {
            if !supported(be) {
                continue;
            }
            let got = f64::from(l2_sq_dispatch(be, &bytes, &b));
            assert!(
                (got - want).abs() <= want.abs() * 1e-4,
                "{be:?}: got={got} want={want}"
            );
        }
    }

    #[test]
    fn handles_unaligned_byte_offsets() {
        let a = sample(131, 0.5);
        let b = sample(131, 4.1);
        let mut padded = vec![0u8; 1];
        padded.extend_from_slice(&to_bytes(&a));
        let want = reference(&a, &b);
        let got = f64::from(l2_sq_f32_bytes(&padded[1..], &b, None));
        assert!((got - want).abs() <= want.abs() * 1e-4);
    }

    #[test]
    fn early_abandon_exceeds_threshold_and_exact_when_under() {
        let a = sample(512, 0.9);
        let b = sample(512, 3.3);
        let bytes = to_bytes(&a);
        let full = l2_sq_f32_bytes(&bytes, &b, None);
        let partial = l2_sq_f32_bytes(&bytes, &b, Some(full * 0.1));
        assert!(partial > full * 0.1 && partial <= full * 1.0001);
        let exact = l2_sq_f32_bytes(&bytes, &b, Some(full * 2.0));
        assert!((exact - full).abs() <= full * 1e-5);
    }

    #[test]
    fn bench_simd_vs_scalar() {
        let a = sample(1024, 0.5);
        let b = sample(1024, 1.5);
        let bytes = to_bytes(&a);
        let iters = 50_000;

        let t0 = std::time::Instant::now();
        let mut s_scalar = 0.0f32;
        for _ in 0..iters {
            s_scalar += l2_sq_dispatch(SimdBackend::Scalar, &bytes, &b);
        }
        let dur_scalar = t0.elapsed();

        #[cfg(target_arch = "aarch64")]
        let hw_be = SimdBackend::Neon;
        #[cfg(target_arch = "x86_64")]
        let hw_be = SimdBackend::Avx2;
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        let hw_be = SimdBackend::Scalar;

        let t1 = std::time::Instant::now();
        let mut s_hw = 0.0f32;
        for _ in 0..iters {
            s_hw += l2_sq_dispatch(hw_be, &bytes, &b);
        }
        let dur_hw = t1.elapsed();

        let ns_scalar = (dur_scalar.as_nanos() as f64) / iters as f64;
        let ns_hw = (dur_hw.as_nanos() as f64) / iters as f64;
        let speedup = ns_scalar / ns_hw;

        println!("\n=== SIMD Micro-Benchmark (1024D, 50,000 iterations) ===");
        println!(
            "Scalar Backend:   {:.2} ns/vector ({:.2?}) [checksum: {:.2}]",
            ns_scalar, dur_scalar, s_scalar
        );
        println!(
            "Hardware Backend ({:?}): {:.2} ns/vector ({:.2?}) [checksum: {:.2}]",
            hw_be, ns_hw, dur_hw, s_hw
        );
        println!(
            "Hardware Speedup: {:.2}x (+{:.1}% throughput)",
            speedup,
            (speedup - 1.0) * 100.0
        );
    }

    #[test]
    fn multi_metric_bytes_kernels_match_reference() {
        for n in [1usize, 7, 16, 33, 64, 128, 384, 768] {
            let a: Vec<f32> = (0..n)
                .map(|i| ((i as f32 * 0.13 + 0.2).sin()) * 0.05)
                .collect();
            let b: Vec<f32> = (0..n)
                .map(|i| ((i as f32 * 0.29 + 0.7).cos()) * 0.05)
                .collect();
            let bytes = to_bytes(&a);

            // Dot product
            let want_dot: f32 = a.iter().zip(&b).map(|(x, y)| x * y).sum();
            let got_dot = dot_f32_bytes(&bytes, &b);
            assert!(
                (got_dot - want_dot).abs() <= want_dot.abs() * 1e-4 + 1e-5,
                "dot mismatch at n={n}: got={got_dot} want={want_dot}"
            );

            // L1 Manhattan
            let want_l1: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).abs()).sum();
            let got_l1 = l1_f32_bytes(&bytes, &b, None);
            assert!(
                (got_l1 - want_l1).abs() <= want_l1.abs() * 1e-4 + 1e-5,
                "l1 mismatch at n={n}: got={got_l1} want={want_l1}"
            );

            // L-inf Chebyshev
            let want_linf: f32 = a
                .iter()
                .zip(&b)
                .map(|(x, y)| (x - y).abs())
                .fold(0.0f32, f32::max);
            let got_linf = linf_f32_bytes(&bytes, &b, None);
            assert!(
                (got_linf - want_linf).abs() <= 1e-6,
                "linf mismatch at n={n}: got={got_linf} want={want_linf}"
            );

            // Poincare
            let u_norm_sq: f64 = a.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
            let v_norm_sq: f64 = b.iter().map(|&x| f64::from(x) * f64::from(x)).sum();
            let diff_sq: f64 = a
                .iter()
                .zip(&b)
                .map(|(&x, &y)| {
                    let d = f64::from(x) - f64::from(y);
                    d * d
                })
                .sum();
            let want_poincare = (1.0
                + 2.0 * diff_sq / ((1.0 - u_norm_sq).max(1e-6) * (1.0 - v_norm_sq).max(1e-6)))
            .max(1.0)
            .acosh();
            let got_poincare = poincare_f32_bytes(&bytes, &b);
            assert!(
                (got_poincare - want_poincare).abs() <= 1e-4,
                "poincare mismatch at n={n}: got={got_poincare} want={want_poincare}"
            );
        }
    }
}
