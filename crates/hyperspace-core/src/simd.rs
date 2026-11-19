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
        _mm512_add_ps, _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps,
        _mm512_sub_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut acc2 = _mm512_setzero_ps();
    let mut acc3 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 64 <= n {
        let d0 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
        );
        let d1 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm512_loadu_ps(bp.add(i + 16)),
        );
        let d2 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 32) * 4)),
            _mm512_loadu_ps(bp.add(i + 32)),
        );
        let d3 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 48) * 4)),
            _mm512_loadu_ps(bp.add(i + 48)),
        );
        acc0 = _mm512_fmadd_ps(d0, d0, acc0);
        acc1 = _mm512_fmadd_ps(d1, d1, acc1);
        acc2 = _mm512_fmadd_ps(d2, d2, acc2);
        acc3 = _mm512_fmadd_ps(d3, d3, acc3);
        i += 64;
    }
    while i + 16 <= n {
        let d = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
        );
        acc0 = _mm512_fmadd_ps(d, d, acc0);
        i += 16;
    }
    let sum01 = _mm512_add_ps(acc0, acc1);
    let sum23 = _mm512_add_ps(acc2, acc3);
    let mut sum = _mm512_reduce_add_ps(_mm512_add_ps(sum01, sum23));
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
        __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_extractf128_ps, _mm256_fmadd_ps,
        _mm256_loadu_ps, _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32,
        _mm_movehl_ps, _mm_shuffle_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut acc2: __m256 = _mm256_setzero_ps();
    let mut acc3: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let d0 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        let d1 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
            _mm256_loadu_ps(bp.add(i + 8)),
        );
        let d2 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm256_loadu_ps(bp.add(i + 16)),
        );
        let d3 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 24) * 4)),
            _mm256_loadu_ps(bp.add(i + 24)),
        );
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
        acc1 = _mm256_fmadd_ps(d1, d1, acc1);
        acc2 = _mm256_fmadd_ps(d2, d2, acc2);
        acc3 = _mm256_fmadd_ps(d3, d3, acc3);
        i += 32;
    }
    while i + 8 <= n {
        let d = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
        );
        acc0 = _mm256_fmadd_ps(d, d, acc0);
        i += 8;
    }
    let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
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
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let d0 = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        let d1 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 4) * 4)),
            vld1q_f32(bp.add(i + 4)),
        );
        let d2 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 8) * 4)),
            vld1q_f32(bp.add(i + 8)),
        );
        let d3 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 12) * 4)),
            vld1q_f32(bp.add(i + 12)),
        );
        acc0 = vfmaq_f32(acc0, d0, d0);
        acc1 = vfmaq_f32(acc1, d1, d1);
        acc2 = vfmaq_f32(acc2, d2, d2);
        acc3 = vfmaq_f32(acc3, d3, d3);
        i += 16;
    }
    while i + 4 <= n {
        let d = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        acc0 = vfmaq_f32(acc0, d, d);
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)));
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

/// `eps0` for the ADSampling kernels on **unrotated** storage: only the exact lower-bound
/// test (`partial_sum > threshold`) is applied. The statistical `0.65·t` / `0.92·t`
/// checkpoints assume energy is spread uniformly over dimensions, which only holds after
/// a random orthogonal rotation; on raw (e.g. Matryoshka/MRL) embeddings, where leading
/// dimensions carry most of the variance, they discard true neighbors.
pub const EXACT_EARLY_ABANDON: f32 = 0.0;

/// Computes the ADSampling statistical hypothesis threshold at prefix length `visited` out of `total`:
/// `threshold * (visited / total) * (1 + eps0 / sqrt(visited))^2`.
#[inline(always)]
pub fn adsampling_threshold(threshold: f32, visited: usize, total: usize, eps0: f32) -> f32 {
    if visited >= total || total == 0 {
        return threshold;
    }
    let ratio = (visited as f32) / (total as f32);
    let slack = 1.0 + eps0 / (visited as f32).sqrt();
    threshold * ratio * slack * slack
}

/// Squared L2 distance with explicit ADSampling progressive statistical early-abandon.
/// Uses at most 2 checkpoints (50% and 75% of dimension, aligned to 32 floats) with
/// precomputed constant bounds (`0.65 * t` at 50%, `0.92 * t` at 75% when `eps0 > 0.0`)
/// to eliminate per-64-float dispatch, horizontal reduction, and `sqrt` overhead.
#[inline]
pub fn l2_sq_f32_bytes_adsampling(
    bytes: &[u8],
    b: &[f32],
    threshold: Option<f32>,
    eps0: f32,
) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    let (bytes, b) = (&bytes[..n * 4], &b[..n]);
    let backend = backend();
    let Some(t) = threshold else {
        return l2_sq_dispatch(backend, bytes, b);
    };
    if n < 128 || !t.is_finite() {
        return l2_sq_dispatch(backend, bytes, b);
    }
    let c1 = (n / 2) & !31;
    let mut sum = l2_sq_dispatch(backend, &bytes[..c1 * 4], &b[..c1]);
    if n >= 256 {
        let bound1 = if eps0 > 0.0 { t * 0.65 } else { t };
        if sum > bound1 {
            return sum.max(t + 1e-4);
        }
        let c2 = ((n * 3) / 4) & !31;
        if c2 > c1 {
            sum += l2_sq_dispatch(backend, &bytes[c1 * 4..c2 * 4], &b[c1..c2]);
            let bound2 = if eps0 > 0.0 { t * 0.92 } else { t };
            if sum > bound2 {
                return sum.max(t + 1e-4);
            }
            sum += l2_sq_dispatch(backend, &bytes[c2 * 4..], &b[c2..]);
            return sum;
        }
    } else if sum > t {
        return sum;
    }
    sum += l2_sq_dispatch(backend, &bytes[c1 * 4..], &b[c1..]);
    sum
}

#[inline]
fn l2_sq_pair_scalar(bytes_a: &[u8], bytes_b: &[u8]) -> f32 {
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    let (ca, _) = bytes_a[..n * 4].as_chunks::<16>();
    let (cb, _) = bytes_b[..n * 4].as_chunks::<16>();
    let mut acc = [0.0f32; 4];
    for (a, b) in ca.iter().zip(cb.iter()) {
        for l in 0..4 {
            let va = f32::from_le_bytes([a[l * 4], a[l * 4 + 1], a[l * 4 + 2], a[l * 4 + 3]]);
            let vb = f32::from_le_bytes([b[l * 4], b[l * 4 + 1], b[l * 4 + 2], b[l * 4 + 3]]);
            let d = va - vb;
            acc[l] += d * d;
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    let done = ca.len() * 4;
    for i in done..n {
        let d = read_f32(bytes_a, i) - read_f32(bytes_b, i);
        sum += d * d;
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn l2_sq_pair_neon(bytes_a: &[u8], bytes_b: &[u8]) -> f32 {
    use std::arch::aarch64::{vaddq_f32, vaddvq_f32, vdupq_n_f32, vfmaq_f32, vld1q_f32, vsubq_f32};
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    let ap = bytes_a.as_ptr().cast::<f32>();
    let bp = bytes_b.as_ptr().cast::<f32>();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let d0 = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.byte_add(i * 4)));
        let d1 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 4) * 4)),
            vld1q_f32(bp.byte_add((i + 4) * 4)),
        );
        let d2 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 8) * 4)),
            vld1q_f32(bp.byte_add((i + 8) * 4)),
        );
        let d3 = vsubq_f32(
            vld1q_f32(ap.byte_add((i + 12) * 4)),
            vld1q_f32(bp.byte_add((i + 12) * 4)),
        );
        acc0 = vfmaq_f32(acc0, d0, d0);
        acc1 = vfmaq_f32(acc1, d1, d1);
        acc2 = vfmaq_f32(acc2, d2, d2);
        acc3 = vfmaq_f32(acc3, d3, d3);
        i += 16;
    }
    while i + 4 <= n {
        let d = vsubq_f32(vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.byte_add(i * 4)));
        acc0 = vfmaq_f32(acc0, d, d);
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)));
    while i < n {
        let d = read_f32(bytes_a, i) - read_f32(bytes_b, i);
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn l2_sq_pair_avx2(bytes_a: &[u8], bytes_b: &[u8]) -> f32 {
    use std::arch::x86_64::{
        __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_extractf128_ps, _mm256_fmadd_ps,
        _mm256_loadu_ps, _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32,
        _mm_movehl_ps, _mm_shuffle_ps,
    };
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    let ap = bytes_a.as_ptr().cast::<f32>();
    let bp = bytes_b.as_ptr().cast::<f32>();
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut acc2: __m256 = _mm256_setzero_ps();
    let mut acc3: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let d0 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.byte_add(i * 4)),
        );
        let d1 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
            _mm256_loadu_ps(bp.byte_add((i + 8) * 4)),
        );
        let d2 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm256_loadu_ps(bp.byte_add((i + 16) * 4)),
        );
        let d3 = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add((i + 24) * 4)),
            _mm256_loadu_ps(bp.byte_add((i + 24) * 4)),
        );
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
        acc1 = _mm256_fmadd_ps(d1, d1, acc1);
        acc2 = _mm256_fmadd_ps(d2, d2, acc2);
        acc3 = _mm256_fmadd_ps(d3, d3, acc3);
        i += 32;
    }
    while i + 8 <= n {
        let d = _mm256_sub_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.byte_add(i * 4)),
        );
        acc0 = _mm256_fmadd_ps(d, d, acc0);
        i += 8;
    }
    let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
    let lo = _mm256_castps256_ps128(acc);
    let hi = _mm256_extractf128_ps(acc, 1);
    let s4 = _mm_add_ps(lo, hi);
    let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
    let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
    let mut sum = _mm_cvtss_f32(s1);
    while i < n {
        let d = read_f32(bytes_a, i) - read_f32(bytes_b, i);
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn l2_sq_pair_avx512(bytes_a: &[u8], bytes_b: &[u8]) -> f32 {
    use std::arch::x86_64::{
        _mm512_add_ps, _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps,
        _mm512_sub_ps,
    };
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    let ap = bytes_a.as_ptr().cast::<f32>();
    let bp = bytes_b.as_ptr().cast::<f32>();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut acc2 = _mm512_setzero_ps();
    let mut acc3 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 64 <= n {
        let d0 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.byte_add(i * 4)),
        );
        let d1 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm512_loadu_ps(bp.byte_add((i + 16) * 4)),
        );
        let d2 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 32) * 4)),
            _mm512_loadu_ps(bp.byte_add((i + 32) * 4)),
        );
        let d3 = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add((i + 48) * 4)),
            _mm512_loadu_ps(bp.byte_add((i + 48) * 4)),
        );
        acc0 = _mm512_fmadd_ps(d0, d0, acc0);
        acc1 = _mm512_fmadd_ps(d1, d1, acc1);
        acc2 = _mm512_fmadd_ps(d2, d2, acc2);
        acc3 = _mm512_fmadd_ps(d3, d3, acc3);
        i += 64;
    }
    while i + 16 <= n {
        let d = _mm512_sub_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.byte_add(i * 4)),
        );
        acc0 = _mm512_fmadd_ps(d, d, acc0);
        i += 16;
    }
    let mut sum = _mm512_reduce_add_ps(_mm512_add_ps(
        _mm512_add_ps(acc0, acc1),
        _mm512_add_ps(acc2, acc3),
    ));
    while i < n {
        let d = read_f32(bytes_a, i) - read_f32(bytes_b, i);
        sum += d * d;
        i += 1;
    }
    sum
}

#[inline]
fn l2_sq_pair_dispatch(backend: SimdBackend, bytes_a: &[u8], bytes_b: &[u8]) -> f32 {
    match backend {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 => unsafe { l2_sq_pair_avx512(bytes_a, bytes_b) },
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx2 => unsafe { l2_sq_pair_avx2(bytes_a, bytes_b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { l2_sq_pair_neon(bytes_a, bytes_b) },
        _ => l2_sq_pair_scalar(bytes_a, bytes_b),
    }
}

/// Zero-copy squared L2 distance directly between two little-endian `f32` byte slices (`&[u8]`)
/// with 2-checkpoint ADSampling progressive early-abandon (`50%` and `75%` of dimension).
/// Eliminates all intermediate `Vec<f32>` unpacking and `memcpy` in `select_neighbors` and `prune_connections`.
#[inline]
pub fn l2_sq_f32_bytes_pair_adsampling(
    bytes_a: &[u8],
    bytes_b: &[u8],
    threshold: Option<f32>,
    eps0: f32,
) -> f32 {
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    let (bytes_a, bytes_b) = (&bytes_a[..n * 4], &bytes_b[..n * 4]);
    let backend = backend();
    let Some(t) = threshold else {
        return l2_sq_pair_dispatch(backend, bytes_a, bytes_b);
    };
    if n < 128 || !t.is_finite() {
        return l2_sq_pair_dispatch(backend, bytes_a, bytes_b);
    }
    let c1 = (n / 2) & !31;
    let mut sum = l2_sq_pair_dispatch(backend, &bytes_a[..c1 * 4], &bytes_b[..c1 * 4]);
    if n >= 256 {
        let bound1 = if eps0 > 0.0 { t * 0.65 } else { t };
        if sum > bound1 {
            return sum.max(t + 1e-4);
        }
        let c2 = ((n * 3) / 4) & !31;
        if c2 > c1 {
            sum += l2_sq_pair_dispatch(backend, &bytes_a[c1 * 4..c2 * 4], &bytes_b[c1 * 4..c2 * 4]);
            let bound2 = if eps0 > 0.0 { t * 0.92 } else { t };
            if sum > bound2 {
                return sum.max(t + 1e-4);
            }
            sum += l2_sq_pair_dispatch(backend, &bytes_a[c2 * 4..], &bytes_b[c2 * 4..]);
            return sum;
        }
    } else if sum > t {
        return sum;
    }
    sum += l2_sq_pair_dispatch(backend, &bytes_a[c1 * 4..], &bytes_b[c1 * 4..]);
    sum
}

/// Zero-copy Lorentz hyperboloid distance proxy directly between two little-endian `f32` byte slices.
#[inline]
pub fn lorentz_f32_bytes_pair_thresh(
    bytes_a: &[u8],
    bytes_b: &[u8],
    threshold: Option<f64>,
) -> f64 {
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    if n == 0 {
        return 1.0;
    }
    let u0 = f64::from(read_f32(bytes_a, 0));
    let v0 = f64::from(read_f32(bytes_b, 0));
    let dt = u0 - v0;
    let dt_sq = dt * dt;
    let sp_thresh = threshold.and_then(|thr| {
        if thr.is_finite() && thr > 1.0 {
            let max_sp = (thr - 1.0) * 2.0 + dt_sq;
            if max_sp < f64::from(f32::MAX) {
                Some(max_sp as f32)
            } else {
                None
            }
        } else {
            None
        }
    });
    let sp = if n > 1 {
        f64::from(l2_sq_f32_bytes_pair_adsampling(
            &bytes_a[4..n * 4],
            &bytes_b[4..n * 4],
            sp_thresh,
            0.0,
        ))
    } else {
        0.0
    };
    1.0 + ((sp - dt_sq) * 0.5).max(0.0)
}

/// Zero-copy **squared** 801D Hybrid (`H^33 x R^768`) distance `d_lor² + ‖Δe‖²` between two
/// little-endian `f32` byte slices. `threshold_sq` is in squared units. No `sqrt`, and no
/// `ln`/`sqrt` for near-field Lorentz heads (see [`crate::acosh_sq_from_cosh`]).
#[inline]
pub fn hybrid_sq_f32_bytes_pair(bytes_a: &[u8], bytes_b: &[u8], threshold_sq: Option<f64>) -> f64 {
    let n = (bytes_a.len() / 4).min(bytes_b.len() / 4);
    if n < 33 {
        return 0.0;
    }
    let cosh_lor = lorentz_f32_bytes_pair_thresh(&bytes_a[..132], &bytes_b[..132], None);
    let d_lor_sq = crate::acosh_sq_from_cosh(cosh_lor);
    if let Some(t) = threshold_sq {
        if d_lor_sq > t {
            return d_lor_sq;
        }
    }
    let euc_thresh = threshold_sq.and_then(|t| {
        let rem = t - d_lor_sq;
        (rem.is_finite() && rem >= 0.0 && rem < f64::from(f32::MAX)).then_some(rem as f32)
    });
    let d_euc = if n > 33 {
        f64::from(l2_sq_f32_bytes_pair_adsampling(
            &bytes_a[132..n * 4],
            &bytes_b[132..n * 4],
            euc_thresh,
            EXACT_EARLY_ABANDON,
        ))
    } else {
        0.0
    };
    d_lor_sq + d_euc.max(0.0)
}

/// Squared L2 distance between little-endian `f32` bytes and a float query.
///
/// Processes `min(bytes.len() / 4, b.len())` elements. When `threshold` is given, checks at
/// at most 2 checkpoints (50% and 75% of dimension, aligned to 32 floats) to avoid per-64-float
/// horizontal reduction and dispatch overhead.
#[inline]
pub fn l2_sq_f32_bytes(bytes: &[u8], b: &[f32], threshold: Option<f32>) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    let (bytes, b) = (&bytes[..n * 4], &b[..n]);
    let backend = backend();
    let Some(t) = threshold else {
        return l2_sq_dispatch(backend, bytes, b);
    };
    if n < 128 {
        return l2_sq_dispatch(backend, bytes, b);
    }
    let c1 = (n / 2) & !31;
    let mut sum = l2_sq_dispatch(backend, &bytes[..c1 * 4], &b[..c1]);
    if sum > t {
        return sum;
    }
    if n >= 256 {
        let c2 = ((n * 3) / 4) & !31;
        if c2 > c1 {
            sum += l2_sq_dispatch(backend, &bytes[c1 * 4..c2 * 4], &b[c1..c2]);
            if sum > t {
                return sum;
            }
            sum += l2_sq_dispatch(backend, &bytes[c2 * 4..], &b[c2..]);
            return sum;
        }
    }
    sum += l2_sq_dispatch(backend, &bytes[c1 * 4..], &b[c1..]);
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
// Zero-copy f64 kernels (legacy `f64` storage, no intermediate Vec<f64>)
// ============================================================================

#[inline(always)]
fn read_f64(bytes: &[u8], i: usize) -> f64 {
    let o = i * 8;
    f64::from_le_bytes([
        bytes[o],
        bytes[o + 1],
        bytes[o + 2],
        bytes[o + 3],
        bytes[o + 4],
        bytes[o + 5],
        bytes[o + 6],
        bytes[o + 7],
    ])
}

/// Squared L2 distance between little-endian `f64` bytes and an `f64` query
/// (`min(bytes.len() / 8, b.len())` elements), reading the storage in place.
/// Four independent accumulators let LLVM vectorize the loop on the target ISA.
#[inline]
pub fn l2_sq_f64_bytes(bytes: &[u8], b: &[f64]) -> f64 {
    let n = (bytes.len() / 8).min(b.len());
    let (chunks, _) = bytes[..n * 8].as_chunks::<32>();
    let mut acc = [0.0f64; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f64::from_le_bytes([
                c[l * 8],
                c[l * 8 + 1],
                c[l * 8 + 2],
                c[l * 8 + 3],
                c[l * 8 + 4],
                c[l * 8 + 5],
                c[l * 8 + 6],
                c[l * 8 + 7],
            ]);
            let d = a - bc[l];
            acc[l] += d * d;
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    for i in (chunks.len() * 4)..n {
        let d = read_f64(bytes, i) - b[i];
        sum += d * d;
    }
    sum
}

/// Lorentz `cosh(d)` between little-endian `f64` bytes and an `f64` query using the
/// cancellation-free difference identity `1 + (‖Δx‖² − Δt²) / 2`.
#[inline]
pub fn lorentz_cosh_f64_bytes(bytes: &[u8], b: &[f64]) -> f64 {
    let n = (bytes.len() / 8).min(b.len());
    if n == 0 {
        return 1.0;
    }
    let dt = read_f64(bytes, 0) - b[0];
    let sp = if n > 1 {
        l2_sq_f64_bytes(&bytes[8..n * 8], &b[1..n])
    } else {
        0.0
    };
    1.0 + ((sp - dt * dt) * 0.5).max(0.0)
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
        _mm512_add_ps, _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut acc2 = _mm512_setzero_ps();
    let mut acc3 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 64 <= n {
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
        acc2 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add((i + 32) * 4)),
            _mm512_loadu_ps(bp.add(i + 32)),
            acc2,
        );
        acc3 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add((i + 48) * 4)),
            _mm512_loadu_ps(bp.add(i + 48)),
            acc3,
        );
        i += 64;
    }
    while i + 16 <= n {
        acc0 = _mm512_fmadd_ps(
            _mm512_loadu_ps(ap.byte_add(i * 4)),
            _mm512_loadu_ps(bp.add(i)),
            acc0,
        );
        i += 16;
    }
    let sum01 = _mm512_add_ps(acc0, acc1);
    let sum23 = _mm512_add_ps(acc2, acc3);
    let mut sum = _mm512_reduce_add_ps(_mm512_add_ps(sum01, sum23));
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
    let mut acc2: __m256 = _mm256_setzero_ps();
    let mut acc3: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
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
        acc2 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add((i + 16) * 4)),
            _mm256_loadu_ps(bp.add(i + 16)),
            acc2,
        );
        acc3 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add((i + 24) * 4)),
            _mm256_loadu_ps(bp.add(i + 24)),
            acc3,
        );
        i += 32;
    }
    while i + 8 <= n {
        acc0 = _mm256_fmadd_ps(
            _mm256_loadu_ps(ap.byte_add(i * 4)),
            _mm256_loadu_ps(bp.add(i)),
            acc0,
        );
        i += 8;
    }
    let acc = _mm256_add_ps(_mm256_add_ps(acc0, acc1), _mm256_add_ps(acc2, acc3));
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
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        acc0 = vfmaq_f32(acc0, vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        acc1 = vfmaq_f32(
            acc1,
            vld1q_f32(ap.byte_add((i + 4) * 4)),
            vld1q_f32(bp.add(i + 4)),
        );
        acc2 = vfmaq_f32(
            acc2,
            vld1q_f32(ap.byte_add((i + 8) * 4)),
            vld1q_f32(bp.add(i + 8)),
        );
        acc3 = vfmaq_f32(
            acc3,
            vld1q_f32(ap.byte_add((i + 12) * 4)),
            vld1q_f32(bp.add(i + 12)),
        );
        i += 16;
    }
    while i + 4 <= n {
        acc0 = vfmaq_f32(acc0, vld1q_f32(ap.byte_add(i * 4)), vld1q_f32(bp.add(i)));
        i += 4;
    }
    let mut sum = vaddvq_f32(vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)));
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
    if n < 128 {
        return l1_dispatch(backend, bytes, b);
    }
    let c1 = (n / 2) & !31;
    let mut sum = l1_dispatch(backend, &bytes[..c1 * 4], &b[..c1]);
    if sum > t {
        return sum;
    }
    if n >= 256 {
        let c2 = ((n * 3) / 4) & !31;
        if c2 > c1 {
            sum += l1_dispatch(backend, &bytes[c1 * 4..c2 * 4], &b[c1..c2]);
            if sum > t {
                return sum;
            }
            sum += l1_dispatch(backend, &bytes[c2 * 4..], &b[c2..]);
            return sum;
        }
    }
    sum += l1_dispatch(backend, &bytes[c1 * 4..], &b[c1..]);
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

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn norms_and_l2_avx512(bytes: &[u8], b: &[f32]) -> (f32, f32, f32) {
    use std::arch::x86_64::{
        _mm512_fmadd_ps, _mm512_loadu_ps, _mm512_reduce_add_ps, _mm512_setzero_ps, _mm512_sub_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc_d = _mm512_setzero_ps();
    let mut acc_a = _mm512_setzero_ps();
    let mut acc_b = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let va = _mm512_loadu_ps(ap.byte_add(i * 4));
        let vb = _mm512_loadu_ps(bp.add(i));
        let vd = _mm512_sub_ps(va, vb);
        acc_d = _mm512_fmadd_ps(vd, vd, acc_d);
        acc_a = _mm512_fmadd_ps(va, va, acc_a);
        acc_b = _mm512_fmadd_ps(vb, vb, acc_b);
        i += 16;
    }
    let mut d_sq = _mm512_reduce_add_ps(acc_d);
    let mut na_sq = _mm512_reduce_add_ps(acc_a);
    let mut nb_sq = _mm512_reduce_add_ps(acc_b);
    while i < n {
        let a = read_f32(bytes, i);
        let bv = b[i];
        let d = a - bv;
        d_sq += d * d;
        na_sq += a * a;
        nb_sq += bv * bv;
        i += 1;
    }
    (d_sq, na_sq, nb_sq)
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn norms_and_l2_avx2(bytes: &[u8], b: &[f32]) -> (f32, f32, f32) {
    use std::arch::x86_64::{
        __m256, _mm256_castps256_ps128, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_loadu_ps,
        _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehl_ps,
        _mm_shuffle_ps,
    };
    let n = b.len();
    let ap = bytes.as_ptr().cast::<f32>();
    let bp = b.as_ptr();
    let mut acc_d = _mm256_setzero_ps();
    let mut acc_a = _mm256_setzero_ps();
    let mut acc_b = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 8 <= n {
        let va = _mm256_loadu_ps(ap.byte_add(i * 4));
        let vb = _mm256_loadu_ps(bp.add(i));
        let vd = _mm256_sub_ps(va, vb);
        acc_d = _mm256_fmadd_ps(vd, vd, acc_d);
        acc_a = _mm256_fmadd_ps(va, va, acc_a);
        acc_b = _mm256_fmadd_ps(vb, vb, acc_b);
        i += 8;
    }
    let hsum = |v: __m256| -> f32 {
        let low = _mm256_castps256_ps128(v);
        let high = _mm256_extractf128_ps(v, 1);
        let sum128 = _mm_add_ps(low, high);
        let shuf = _mm_movehl_ps(sum128, sum128);
        let sums = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_shuffle_ps(sums, sums, 1);
        _mm_cvtss_f32(_mm_add_ss(sums, shuf2))
    };
    let mut d_sq = hsum(acc_d);
    let mut na_sq = hsum(acc_a);
    let mut nb_sq = hsum(acc_b);
    while i < n {
        let a = read_f32(bytes, i);
        let bv = b[i];
        let d = a - bv;
        d_sq += d * d;
        na_sq += a * a;
        nb_sq += bv * bv;
        i += 1;
    }
    (d_sq, na_sq, nb_sq)
}

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

    #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
    match backend() {
        SimdBackend::Avx512 => unsafe { return norms_and_l2_avx512(bytes, b) },
        SimdBackend::Avx2 => unsafe { return norms_and_l2_avx2(bytes, b) },
        _ => {}
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

/// Zero-copy Lorentz hyperboloid (Minkowski) distance proxy directly on little-endian `f32` bytes.
///
/// Uses the numerically stable difference identity:
/// `cosh(d) = 1 + 0.5 * (||u_spatial - v_spatial||^2 - (u0 - v0)^2)`
/// which avoids catastrophic cancellation `u0 * v0 - <u_spatial, v_spatial>` at large radii `t`,
/// and reuses the hardware-accelerated `l2_sq_f32_bytes` kernel.
#[inline]
pub fn lorentz_f32_bytes(bytes: &[u8], b: &[f32]) -> f64 {
    lorentz_f32_bytes_thresh(bytes, b, None)
}

/// Zero-copy Lorentz hyperboloid distance proxy with optional early-abandon threshold on `cosh(d)`.
#[inline]
pub fn lorentz_f32_bytes_thresh(bytes: &[u8], b: &[f32], threshold: Option<f64>) -> f64 {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return 1.0;
    }
    let u0 = f64::from(read_f32(bytes, 0));
    let v0 = f64::from(b[0]);
    let dt = u0 - v0;
    let dt_sq = dt * dt;
    let sp_thresh = threshold.and_then(|thr| {
        if thr.is_finite() && thr > 1.0 {
            let max_sp = (thr - 1.0) * 2.0 + dt_sq;
            if max_sp < f64::from(f32::MAX) {
                Some(max_sp as f32)
            } else {
                None
            }
        } else {
            None
        }
    });
    let sp = if n > 1 {
        f64::from(l2_sq_f32_bytes(&bytes[4..n * 4], &b[1..n], sp_thresh))
    } else {
        0.0
    };
    1.0 + ((sp - dt_sq) * 0.5).max(0.0)
}

/// Zero-copy **squared** 801D Hybrid (`H^33 x R^768`) distance `d_lor² + ‖Δe‖²` on
/// little-endian `f32` bytes and query slice `b`.
///
/// Ranking by the squared product-space distance is equivalent to ranking by the distance,
/// so the final `sqrt` is deferred to result output, and the Lorentz head uses
/// [`crate::acosh_sq_from_cosh`] (no `sqrt`/`ln` in the near field). `threshold_sq` is in
/// squared units; the cross-manifold early-abandon checks it right after the 33D head.
#[inline]
pub fn hybrid_sq_f32_bytes(bytes: &[u8], b: &[f32], threshold_sq: Option<f64>) -> f64 {
    let n = (bytes.len() / 4).min(b.len());
    if n < 33 {
        return 0.0;
    }
    let cosh_lor = lorentz_f32_bytes_thresh(&bytes[..132], &b[..33], None);
    let d_lor_sq = crate::acosh_sq_from_cosh(cosh_lor);
    if let Some(t) = threshold_sq {
        if d_lor_sq > t {
            return d_lor_sq;
        }
    }
    let euc_thresh = threshold_sq.and_then(|t| {
        let rem = t - d_lor_sq;
        (rem.is_finite() && rem >= 0.0 && rem < f64::from(f32::MAX)).then_some(rem as f32)
    });
    let d_euc = if n > 33 {
        f64::from(l2_sq_f32_bytes_adsampling(
            &bytes[132..n * 4],
            &b[33..n],
            euc_thresh,
            EXACT_EARLY_ABANDON,
        ))
    } else {
        0.0
    };
    d_lor_sq + d_euc.max(0.0)
}

/// Computes `sum_i (a_i * scale - b_i)^2` over little-endian `f32` bytes `a` and slice `b`.
/// Used for SIMD-accelerated Klein chord distance on upper HNSW layers.
#[inline]
pub fn scaled_l2_sq_f32_bytes(bytes: &[u8], scale: f32, b: &[f32]) -> f32 {
    let n = (bytes.len() / 4).min(b.len());
    if n == 0 {
        return 0.0;
    }
    let bytes = &bytes[..n * 4];
    let b = &b[..n];

    #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
    if backend() == SimdBackend::Neon {
        unsafe {
            use std::arch::aarch64::{
                vaddq_f32, vaddvq_f32, vdupq_n_f32, vfmaq_f32, vfmsq_f32, vld1q_f32, vnegq_f32,
            };
            let ap = bytes.as_ptr().cast::<f32>();
            let bp = b.as_ptr();
            let vscale = vdupq_n_f32(scale);
            let mut acc0 = vdupq_n_f32(0.0);
            let mut acc1 = vdupq_n_f32(0.0);
            let mut i = 0usize;
            while i + 8 <= n {
                // vfmsq_f32(b, a, scale) = b - a * scale; squared is identical to (a * scale - b)^2
                let d0 = vfmsq_f32(vld1q_f32(bp.add(i)), vld1q_f32(ap.byte_add(i * 4)), vscale);
                let d1 = vfmsq_f32(
                    vld1q_f32(bp.add(i + 4)),
                    vld1q_f32(ap.byte_add((i + 4) * 4)),
                    vscale,
                );
                let _ = vnegq_f32;
                acc0 = vfmaq_f32(acc0, d0, d0);
                acc1 = vfmaq_f32(acc1, d1, d1);
                i += 8;
            }
            let mut sum = vaddvq_f32(vaddq_f32(acc0, acc1));
            while i < n {
                let d = read_f32(bytes, i) * scale - b[i];
                sum += d * d;
                i += 1;
            }
            return sum;
        }
    }

    #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
    if matches!(backend(), SimdBackend::Avx2 | SimdBackend::Avx512) {
        unsafe {
            use std::arch::x86_64::{
                __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_extractf128_ps,
                _mm256_fmadd_ps, _mm256_fmsub_ps, _mm256_loadu_ps, _mm256_set1_ps,
                _mm256_setzero_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_movehl_ps,
                _mm_shuffle_ps,
            };
            let ap = bytes.as_ptr().cast::<f32>();
            let bp = b.as_ptr();
            let vscale = _mm256_set1_ps(scale);
            let mut acc0: __m256 = _mm256_setzero_ps();
            let mut acc1: __m256 = _mm256_setzero_ps();
            let mut i = 0usize;
            while i + 16 <= n {
                let d0 = _mm256_fmsub_ps(
                    _mm256_loadu_ps(ap.byte_add(i * 4)),
                    vscale,
                    _mm256_loadu_ps(bp.add(i)),
                );
                let d1 = _mm256_fmsub_ps(
                    _mm256_loadu_ps(ap.byte_add((i + 8) * 4)),
                    vscale,
                    _mm256_loadu_ps(bp.add(i + 8)),
                );
                acc0 = _mm256_fmadd_ps(d0, d0, acc0);
                acc1 = _mm256_fmadd_ps(d1, d1, acc1);
                i += 16;
            }
            let acc = _mm256_add_ps(acc0, acc1);
            let lo = _mm256_castps256_ps128(acc);
            let hi = _mm256_extractf128_ps(acc, 1);
            let s4 = _mm_add_ps(lo, hi);
            let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
            let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
            let mut sum = _mm_cvtss_f32(s1);
            while i < n {
                let d = read_f32(bytes, i) * scale - b[i];
                sum += d * d;
                i += 1;
            }
            return sum;
        }
    }

    let (chunks, rem) = bytes.as_chunks::<16>();
    let mut acc = [0.0f32; 4];
    for (c, bc) in chunks.iter().zip(b.as_chunks::<4>().0) {
        for l in 0..4 {
            let a = f32::from_le_bytes([c[l * 4], c[l * 4 + 1], c[l * 4 + 2], c[l * 4 + 3]]);
            let d = a * scale - bc[l];
            acc[l] += d * d;
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]);
    let done = chunks.len() * 4;
    for i in 0..(rem.len() / 4) {
        let d = read_f32(&bytes[done * 4..], i) * scale - b[done + i];
        sum += d * d;
    }
    sum
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

// ============================================================================
// SQ8 (Scalar Quantization 8-bit i8) SIMD Kernels
// ============================================================================

const SQ8_SCALE_INV: f32 = 1.0 / 127.0;

fn sq8_l2_sq_scalar(bytes: &[u8], b: &[f32]) -> f32 {
    let n = bytes.len().min(b.len());
    let (chunks_a, rem_a) = bytes[..n].as_chunks::<8>();
    let (chunks_b, rem_b) = b[..n].as_chunks::<8>();
    let mut acc = [0.0f32; 8];
    for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
        for l in 0..8 {
            let a = f32::from(ca[l].cast_signed()) * SQ8_SCALE_INV;
            let d = a - cb[l];
            acc[l] += d * d;
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]) + (acc[4] + acc[5]) + (acc[6] + acc[7]);
    for (&a_u8, &b_f32) in rem_a.iter().zip(rem_b.iter()) {
        let a = f32::from(a_u8.cast_signed()) * SQ8_SCALE_INV;
        let d = a - b_f32;
        sum += d * d;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn sq8_l2_sq_avx512(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m128i, _mm512_cvtepi32_ps, _mm512_cvtepi8_epi32, _mm512_fmadd_ps, _mm512_loadu_ps,
        _mm512_mul_ps, _mm512_reduce_add_ps, _mm512_set1_ps, _mm512_setzero_ps, _mm512_sub_ps,
        _mm_loadu_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let v_scale = _mm512_set1_ps(SQ8_SCALE_INV);
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let raw0 = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let raw1 = _mm_loadu_si128(ap.add(i + 16).cast::<__m128i>());
        let a0 = _mm512_mul_ps(_mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw0)), v_scale);
        let a1 = _mm512_mul_ps(_mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw1)), v_scale);
        let d0 = _mm512_sub_ps(a0, _mm512_loadu_ps(bp.add(i)));
        let d1 = _mm512_sub_ps(a1, _mm512_loadu_ps(bp.add(i + 16)));
        acc0 = _mm512_fmadd_ps(d0, d0, acc0);
        acc1 = _mm512_fmadd_ps(d1, d1, acc1);
        i += 32;
    }
    if i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let a = _mm512_mul_ps(_mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw)), v_scale);
        let d = _mm512_sub_ps(a, _mm512_loadu_ps(bp.add(i)));
        acc0 = _mm512_fmadd_ps(d, d, acc0);
        i += 16;
    }
    let mut sum = _mm512_reduce_add_ps(acc0) + _mm512_reduce_add_ps(acc1);
    while i < n {
        let a = (f32::from(bytes[i].cast_signed())) * SQ8_SCALE_INV;
        let d = a - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn sq8_l2_sq_avx2(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m128i, __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_cvtepi32_ps,
        _mm256_cvtepi8_epi32, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_loadu_ps,
        _mm256_mul_ps, _mm256_set1_ps, _mm256_setzero_ps, _mm256_sub_ps, _mm_add_ps, _mm_add_ss,
        _mm_cvtss_f32, _mm_loadl_epi64, _mm_loadu_si128, _mm_movehl_ps, _mm_shuffle_ps,
        _mm_srli_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let v_scale = _mm256_set1_ps(SQ8_SCALE_INV);
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let a0 = _mm256_mul_ps(_mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw)), v_scale);
        let a1 = _mm256_mul_ps(
            _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(_mm_srli_si128(raw, 8))),
            v_scale,
        );
        let d0 = _mm256_sub_ps(a0, _mm256_loadu_ps(bp.add(i)));
        let d1 = _mm256_sub_ps(a1, _mm256_loadu_ps(bp.add(i + 8)));
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
        acc1 = _mm256_fmadd_ps(d1, d1, acc1);
        i += 16;
    }
    if i + 8 <= n {
        let raw = _mm_loadl_epi64(ap.add(i).cast::<__m128i>());
        let a0 = _mm256_mul_ps(_mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw)), v_scale);
        let d0 = _mm256_sub_ps(a0, _mm256_loadu_ps(bp.add(i)));
        acc0 = _mm256_fmadd_ps(d0, d0, acc0);
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
        let a = f32::from(bytes[i].cast_signed()) * SQ8_SCALE_INV;
        let d = a - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn sq8_l2_sq_neon(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::aarch64::{
        vaddq_f32, vaddvq_f32, vcvtq_f32_s32, vdupq_n_f32, vfmaq_f32, vfmsq_f32, vget_high_s16,
        vget_high_s8, vget_low_s16, vget_low_s8, vld1_s8, vld1q_f32, vld1q_s8, vmovl_s16, vmovl_s8,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr().cast::<i8>();
    let bp = b.as_ptr();
    let v_scale = vdupq_n_f32(SQ8_SCALE_INV);
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = vld1q_s8(ap.add(i));
        let s16_lo = vmovl_s8(vget_low_s8(raw));
        let s16_hi = vmovl_s8(vget_high_s8(raw));
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_lo)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_lo)));
        let q2 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_hi)));
        let q3 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_hi)));
        let d0 = vfmsq_f32(vld1q_f32(bp.add(i)), q0, v_scale);
        let d1 = vfmsq_f32(vld1q_f32(bp.add(i + 4)), q1, v_scale);
        let d2 = vfmsq_f32(vld1q_f32(bp.add(i + 8)), q2, v_scale);
        let d3 = vfmsq_f32(vld1q_f32(bp.add(i + 12)), q3, v_scale);
        acc0 = vfmaq_f32(acc0, d0, d0);
        acc1 = vfmaq_f32(acc1, d1, d1);
        acc2 = vfmaq_f32(acc2, d2, d2);
        acc3 = vfmaq_f32(acc3, d3, d3);
        i += 16;
    }
    if i + 8 <= n {
        let raw = vld1_s8(ap.add(i));
        let s16 = vmovl_s8(raw);
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16)));
        let d0 = vfmsq_f32(vld1q_f32(bp.add(i)), q0, v_scale);
        let d1 = vfmsq_f32(vld1q_f32(bp.add(i + 4)), q1, v_scale);
        acc0 = vfmaq_f32(acc0, d0, d0);
        acc1 = vfmaq_f32(acc1, d1, d1);
        i += 8;
    }
    let mut sum = vaddvq_f32(vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)));
    while i < n {
        let a = f32::from(bytes[i].cast_signed()) * SQ8_SCALE_INV;
        let d = a - b[i];
        sum += d * d;
        i += 1;
    }
    sum
}

/// Squared Euclidean (L2) distance between SQ8 i8 bytes and a float query slice.
#[inline]
pub fn sq8_l2_sq(bytes: &[u8], b: &[f32]) -> f32 {
    let backend = backend();
    match backend {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 => unsafe { sq8_l2_sq_avx512(bytes, b) },
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx2 => unsafe { sq8_l2_sq_avx2(bytes, b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { sq8_l2_sq_neon(bytes, b) },
        _ => sq8_l2_sq_scalar(bytes, b),
    }
}

fn sq8_scaled_l2_and_norm_sq_scalar(bytes: &[u8], scale: f32, b: &[f32]) -> (f32, f32) {
    let n = bytes.len().min(b.len());
    let (chunks_a, rem_a) = bytes[..n].as_chunks::<4>();
    let (chunks_b, rem_b) = b[..n].as_chunks::<4>();
    let mut d_acc = [0.0f32; 4];
    let mut q_sq_sum: u32 = 0;
    for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
        for l in 0..4 {
            let qi = i32::from(ca[l].cast_signed());
            q_sq_sum += (qi * qi).cast_unsigned();
            let a = (qi as f32) * scale;
            let d = a - cb[l];
            d_acc[l] += d * d;
        }
    }
    let mut d_sum = (d_acc[0] + d_acc[1]) + (d_acc[2] + d_acc[3]);
    for (&a_u8, &b_f32) in rem_a.iter().zip(rem_b.iter()) {
        let qi = i32::from(a_u8.cast_signed());
        q_sq_sum += (qi * qi).cast_unsigned();
        let a = (qi as f32) * scale;
        let d = a - b_f32;
        d_sum += d * d;
    }
    let n_sum = (q_sq_sum as f32) * (scale * scale);
    (d_sum, n_sum)
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn sq8_scaled_l2_and_norm_sq_neon(bytes: &[u8], scale: f32, b: &[f32]) -> (f32, f32) {
    use std::arch::aarch64::{
        vaddq_f32, vaddvq_f32, vaddvq_s32, vcvtq_f32_s32, vdupq_n_f32, vdupq_n_s32, vfmaq_f32,
        vfmsq_f32, vget_high_s16, vget_high_s8, vget_low_s16, vget_low_s8, vld1_s8, vld1q_f32,
        vld1q_s8, vmlal_s8, vmovl_s16, vmovl_s8, vmull_s8, vpadalq_s16,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr().cast::<i8>();
    let bp = b.as_ptr();
    let v_scale = vdupq_n_f32(scale);
    let mut acc_d0 = vdupq_n_f32(0.0);
    let mut acc_d1 = vdupq_n_f32(0.0);
    let mut acc_d2 = vdupq_n_f32(0.0);
    let mut acc_d3 = vdupq_n_f32(0.0);
    let mut acc_qsq = vdupq_n_s32(0);
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = vld1q_s8(ap.add(i));
        let lo8 = vget_low_s8(raw);
        let hi8 = vget_high_s8(raw);
        // Integer pipeline: accumulate sum of squares of 16 i8 elements in 2 instructions
        let sq16 = vmlal_s8(vmull_s8(lo8, lo8), hi8, hi8);
        acc_qsq = vpadalq_s16(acc_qsq, sq16);

        let s16_lo = vmovl_s8(lo8);
        let s16_hi = vmovl_s8(hi8);
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_lo)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_lo)));
        let q2 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_hi)));
        let q3 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_hi)));

        // Fused multiply-subtract: d = b - q * scale; d^2 == (q * scale - b)^2
        let d0 = vfmsq_f32(vld1q_f32(bp.add(i)), q0, v_scale);
        let d1 = vfmsq_f32(vld1q_f32(bp.add(i + 4)), q1, v_scale);
        let d2 = vfmsq_f32(vld1q_f32(bp.add(i + 8)), q2, v_scale);
        let d3 = vfmsq_f32(vld1q_f32(bp.add(i + 12)), q3, v_scale);
        acc_d0 = vfmaq_f32(acc_d0, d0, d0);
        acc_d1 = vfmaq_f32(acc_d1, d1, d1);
        acc_d2 = vfmaq_f32(acc_d2, d2, d2);
        acc_d3 = vfmaq_f32(acc_d3, d3, d3);
        i += 16;
    }
    if i + 8 <= n {
        let raw = vld1_s8(ap.add(i));
        acc_qsq = vpadalq_s16(acc_qsq, vmull_s8(raw, raw));
        let s16 = vmovl_s8(raw);
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16)));
        let d0 = vfmsq_f32(vld1q_f32(bp.add(i)), q0, v_scale);
        let d1 = vfmsq_f32(vld1q_f32(bp.add(i + 4)), q1, v_scale);
        acc_d0 = vfmaq_f32(acc_d0, d0, d0);
        acc_d1 = vfmaq_f32(acc_d1, d1, d1);
        i += 8;
    }
    let mut d_sum = vaddvq_f32(vaddq_f32(
        vaddq_f32(acc_d0, acc_d1),
        vaddq_f32(acc_d2, acc_d3),
    ));
    let mut q_sq_sum = vaddvq_s32(acc_qsq);
    while i < n {
        let qi = i32::from(bytes[i].cast_signed());
        q_sq_sum += qi * qi;
        let a = (qi as f32) * scale;
        let d = a - b[i];
        d_sum += d * d;
        i += 1;
    }
    let n_sum = (q_sq_sum as f32) * (scale * scale);
    (d_sum, n_sum)
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn sq8_scaled_l2_and_norm_sq_avx2(bytes: &[u8], scale: f32, b: &[f32]) -> (f32, f32) {
    use std::arch::x86_64::{
        __m128i, __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_cvtepi32_ps,
        _mm256_cvtepi8_epi32, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_fmsub_ps,
        _mm256_loadu_ps, _mm256_set1_ps, _mm256_setzero_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32,
        _mm_loadl_epi64, _mm_loadu_si128, _mm_movehl_ps, _mm_shuffle_ps, _mm_srli_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let v_scale = _mm256_set1_ps(scale);
    let mut acc_d0: __m256 = _mm256_setzero_ps();
    let mut acc_d1: __m256 = _mm256_setzero_ps();
    let mut acc_n0: __m256 = _mm256_setzero_ps();
    let mut acc_n1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let q0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        let q1 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(_mm_srli_si128(raw, 8)));
        let d0 = _mm256_fmsub_ps(q0, v_scale, _mm256_loadu_ps(bp.add(i)));
        let d1 = _mm256_fmsub_ps(q1, v_scale, _mm256_loadu_ps(bp.add(i + 8)));
        acc_d0 = _mm256_fmadd_ps(d0, d0, acc_d0);
        acc_d1 = _mm256_fmadd_ps(d1, d1, acc_d1);
        acc_n0 = _mm256_fmadd_ps(q0, q0, acc_n0);
        acc_n1 = _mm256_fmadd_ps(q1, q1, acc_n1);
        i += 16;
    }
    if i + 8 <= n {
        let raw = _mm_loadl_epi64(ap.add(i).cast::<__m128i>());
        let q0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        let d0 = _mm256_fmsub_ps(q0, v_scale, _mm256_loadu_ps(bp.add(i)));
        acc_d0 = _mm256_fmadd_ps(d0, d0, acc_d0);
        acc_n0 = _mm256_fmadd_ps(q0, q0, acc_n0);
        i += 8;
    }
    let hsum = |v: __m256| -> f32 {
        let lo = _mm256_castps256_ps128(v);
        let hi = _mm256_extractf128_ps(v, 1);
        let s4 = _mm_add_ps(lo, hi);
        let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
        let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
        _mm_cvtss_f32(s1)
    };
    let mut d_sum = hsum(_mm256_add_ps(acc_d0, acc_d1));
    let mut q_sq_sum = hsum(_mm256_add_ps(acc_n0, acc_n1));
    while i < n {
        let q = f32::from(bytes[i].cast_signed());
        q_sq_sum += q * q;
        let d = q * scale - b[i];
        d_sum += d * d;
        i += 1;
    }
    (d_sum, q_sq_sum * (scale * scale))
}

/// Simultaneously computes `(sum (a_i * scale - b_i)^2, sum (a_i * scale)^2)` for SQ8 `i8` bytes.
/// Used for exact hyperboloid-constrained Lorentz distance on `ScalarI8` vectors without
/// catastrophic cancellation or time-coordinate quantization drift.
#[inline]
pub fn sq8_scaled_l2_and_norm_sq(bytes: &[u8], scale: f32, b: &[f32]) -> (f32, f32) {
    match backend() {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 | SimdBackend::Avx2 => unsafe {
            sq8_scaled_l2_and_norm_sq_avx2(bytes, scale, b)
        },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { sq8_scaled_l2_and_norm_sq_neon(bytes, scale, b) },
        _ => sq8_scaled_l2_and_norm_sq_scalar(bytes, scale, b),
    }
}

/// Simultaneously computes `(sum (q_i * b_i), sum (q_i^2))` for SQ8 `i8` bytes and `f32` query.
/// Eliminates the per-lane `vfmsq_f32` scale multiplication inside the loop (cutting FP FMA
/// instructions in half) while accumulating exact integer `sum(q_i^2)` in `u32`.
#[inline]
pub fn sq8_dot_and_qsq(bytes: &[u8], b: &[f32]) -> (f32, u32) {
    #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
    {
        if matches!(backend(), SimdBackend::Neon) {
            // SAFETY: aarch64 baseline guarantees NEON.
            return unsafe { sq8_dot_and_qsq_neon(bytes, b) };
        }
    }
    #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
    {
        if matches!(backend(), SimdBackend::Avx512 | SimdBackend::Avx2) {
            // SAFETY: checked runtime feature detection.
            return unsafe { sq8_dot_and_qsq_avx2(bytes, b) };
        }
    }
    let n = bytes.len().min(b.len());
    let (chunks_a, rem_a) = bytes[..n].as_chunks::<4>();
    let (chunks_b, rem_b) = b[..n].as_chunks::<4>();
    let mut dot_acc = [0.0f32; 4];
    let mut q_sq_sum: u32 = 0;
    for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
        for l in 0..4 {
            let qi = i32::from(ca[l].cast_signed());
            q_sq_sum += (qi * qi).cast_unsigned();
            dot_acc[l] += (qi as f32) * cb[l];
        }
    }
    let mut dot_sum = (dot_acc[0] + dot_acc[1]) + (dot_acc[2] + dot_acc[3]);
    for (&a_u8, &b_f32) in rem_a.iter().zip(rem_b.iter()) {
        let qi = i32::from(a_u8.cast_signed());
        q_sq_sum += (qi * qi).cast_unsigned();
        dot_sum += (qi as f32) * b_f32;
    }
    (dot_sum, q_sq_sum)
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn sq8_dot_and_qsq_neon(bytes: &[u8], b: &[f32]) -> (f32, u32) {
    use std::arch::aarch64::{
        vaddq_f32, vaddvq_f32, vaddvq_s32, vcvtq_f32_s32, vdupq_n_f32, vdupq_n_s32, vfmaq_f32,
        vget_high_s16, vget_high_s8, vget_low_s16, vget_low_s8, vld1_s8, vld1q_f32, vld1q_s8,
        vmlal_s8, vmovl_s16, vmovl_s8, vmull_s8, vpadalq_s16,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr().cast::<i8>();
    let bp = b.as_ptr();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut acc2 = vdupq_n_f32(0.0);
    let mut acc3 = vdupq_n_f32(0.0);
    let mut acc_qsq = vdupq_n_s32(0);
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = vld1q_s8(ap.add(i));
        let lo8 = vget_low_s8(raw);
        let hi8 = vget_high_s8(raw);
        let sq16 = vmlal_s8(vmull_s8(lo8, lo8), hi8, hi8);
        acc_qsq = vpadalq_s16(acc_qsq, sq16);

        let s16_lo = vmovl_s8(lo8);
        let s16_hi = vmovl_s8(hi8);
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_lo)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_lo)));
        let q2 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16_hi)));
        let q3 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16_hi)));

        acc0 = vfmaq_f32(acc0, q0, vld1q_f32(bp.add(i)));
        acc1 = vfmaq_f32(acc1, q1, vld1q_f32(bp.add(i + 4)));
        acc2 = vfmaq_f32(acc2, q2, vld1q_f32(bp.add(i + 8)));
        acc3 = vfmaq_f32(acc3, q3, vld1q_f32(bp.add(i + 12)));
        i += 16;
    }
    if i + 8 <= n {
        let raw = vld1_s8(ap.add(i));
        acc_qsq = vpadalq_s16(acc_qsq, vmull_s8(raw, raw));
        let s16 = vmovl_s8(raw);
        let q0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16)));
        let q1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16)));
        acc0 = vfmaq_f32(acc0, q0, vld1q_f32(bp.add(i)));
        acc1 = vfmaq_f32(acc1, q1, vld1q_f32(bp.add(i + 4)));
        i += 8;
    }
    let mut dot_sum = vaddvq_f32(vaddq_f32(vaddq_f32(acc0, acc1), vaddq_f32(acc2, acc3)));
    let mut q_sq_sum = vaddvq_s32(acc_qsq).cast_unsigned();
    while i < n {
        let qi = i32::from(bytes[i].cast_signed());
        q_sq_sum += (qi * qi).cast_unsigned();
        dot_sum += (qi as f32) * b[i];
        i += 1;
    }
    (dot_sum, q_sq_sum)
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn sq8_dot_and_qsq_avx2(bytes: &[u8], b: &[f32]) -> (f32, u32) {
    use std::arch::x86_64::{
        __m128i, __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_cvtepi32_ps,
        _mm256_cvtepi8_epi32, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_loadu_ps,
        _mm256_setzero_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_loadl_epi64, _mm_loadu_si128,
        _mm_movehl_ps, _mm_shuffle_ps, _mm_srli_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let mut acc_dot0: __m256 = _mm256_setzero_ps();
    let mut acc_dot1: __m256 = _mm256_setzero_ps();
    let mut acc_n0: __m256 = _mm256_setzero_ps();
    let mut acc_n1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let q0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        let q1 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(_mm_srli_si128(raw, 8)));
        acc_dot0 = _mm256_fmadd_ps(q0, _mm256_loadu_ps(bp.add(i)), acc_dot0);
        acc_dot1 = _mm256_fmadd_ps(q1, _mm256_loadu_ps(bp.add(i + 8)), acc_dot1);
        acc_n0 = _mm256_fmadd_ps(q0, q0, acc_n0);
        acc_n1 = _mm256_fmadd_ps(q1, q1, acc_n1);
        i += 16;
    }
    if i + 8 <= n {
        let raw = _mm_loadl_epi64(ap.add(i).cast::<__m128i>());
        let q0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        acc_dot0 = _mm256_fmadd_ps(q0, _mm256_loadu_ps(bp.add(i)), acc_dot0);
        acc_n0 = _mm256_fmadd_ps(q0, q0, acc_n0);
        i += 8;
    }
    let hsum = |v: __m256| -> f32 {
        let lo = _mm256_castps256_ps128(v);
        let hi = _mm256_extractf128_ps(v, 1);
        let s4 = _mm_add_ps(lo, hi);
        let s2 = _mm_add_ps(s4, _mm_movehl_ps(s4, s4));
        let s1 = _mm_add_ss(s2, _mm_shuffle_ps(s2, s2, 0b01));
        _mm_cvtss_f32(s1)
    };
    let mut dot_sum = hsum(_mm256_add_ps(acc_dot0, acc_dot1));
    let mut q_sq_sum = (hsum(_mm256_add_ps(acc_n0, acc_n1)) as i32).cast_unsigned();
    while i < n {
        let qi = i32::from(bytes[i].cast_signed());
        q_sq_sum += (qi * qi).cast_unsigned();
        dot_sum += (qi as f32) * b[i];
        i += 1;
    }
    (dot_sum, q_sq_sum)
}

/// Zero-copy Lorentz hyperboloid distance proxy between two `ScalarI8` byte slices in storage.
/// Uses pure integer `i8 -> i32` dot/norm accumulation inside the loop and a single `f64` evaluation at the end.
#[inline]
pub fn sq8_lorentz_bytes_pair(bytes_a: &[u8], bytes_b: &[u8]) -> f64 {
    if bytes_a.len() < 4 || bytes_b.len() < 4 {
        return f64::MAX;
    }
    let off_a = bytes_a.len() - 4;
    let off_b = bytes_b.len() - 4;
    let n = off_a.min(off_b);
    if n <= 1 {
        return 1.0;
    }
    let alpha_a = f32::from_le_bytes(bytes_a[off_a..].try_into().unwrap_or([0; 4]));
    let alpha_b = f32::from_le_bytes(bytes_b[off_b..].try_into().unwrap_or([0; 4]));
    let scale_a = f64::from(alpha_a) * (1.0 / 127.0);
    let scale_b = f64::from(alpha_b) * (1.0 / 127.0);
    let mut dot_ab: i32 = 0;
    let mut qa_sq: u32 = 0;
    let mut qb_sq: u32 = 0;
    for (&ba, &bb) in bytes_a[1..n].iter().zip(bytes_b[1..n].iter()) {
        let qa = i32::from(ba.cast_signed());
        let qb = i32::from(bb.cast_signed());
        dot_ab += qa * qb;
        qa_sq += (qa * qa).cast_unsigned();
        qb_sq += (qb * qb).cast_unsigned();
    }
    let ta = (1.0 + f64::from(qa_sq) * (scale_a * scale_a)).sqrt();
    let tb = (1.0 + f64::from(qb_sq) * (scale_b * scale_b)).sqrt();
    let sp_dot = f64::from(dot_ab) * (scale_a * scale_b);
    (ta * tb - sp_dot).max(1.0)
}

fn sq8_dot_scalar(bytes: &[u8], b: &[f32]) -> f32 {
    let n = bytes.len().min(b.len());
    let (chunks_a, rem_a) = bytes[..n].as_chunks::<8>();
    let (chunks_b, rem_b) = b[..n].as_chunks::<8>();
    let mut acc = [0.0f32; 8];
    for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
        for l in 0..8 {
            let a = f32::from(ca[l].cast_signed());
            acc[l] += a * cb[l];
        }
    }
    let mut sum = (acc[0] + acc[1]) + (acc[2] + acc[3]) + (acc[4] + acc[5]) + (acc[6] + acc[7]);
    for (&a_u8, &b_f32) in rem_a.iter().zip(rem_b.iter()) {
        let a = f32::from(a_u8.cast_signed());
        sum += a * b_f32;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx512f")]
unsafe fn sq8_dot_avx512(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m128i, _mm512_cvtepi32_ps, _mm512_cvtepi8_epi32, _mm512_fmadd_ps, _mm512_loadu_ps,
        _mm512_reduce_add_ps, _mm512_setzero_ps, _mm_loadu_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let mut acc0 = _mm512_setzero_ps();
    let mut acc1 = _mm512_setzero_ps();
    let mut i = 0usize;
    while i + 32 <= n {
        let raw0 = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let raw1 = _mm_loadu_si128(ap.add(i + 16).cast::<__m128i>());
        let a0 = _mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw0));
        let a1 = _mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw1));
        acc0 = _mm512_fmadd_ps(a0, _mm512_loadu_ps(bp.add(i)), acc0);
        acc1 = _mm512_fmadd_ps(a1, _mm512_loadu_ps(bp.add(i + 16)), acc1);
        i += 32;
    }
    if i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let a = _mm512_cvtepi32_ps(_mm512_cvtepi8_epi32(raw));
        acc0 = _mm512_fmadd_ps(a, _mm512_loadu_ps(bp.add(i)), acc0);
        i += 16;
    }
    let mut sum = _mm512_reduce_add_ps(acc0) + _mm512_reduce_add_ps(acc1);
    while i < n {
        sum += (f32::from(bytes[i].cast_signed())) * b[i];
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "x86_64", target_endian = "little"))]
#[target_feature(enable = "avx2,fma")]
unsafe fn sq8_dot_avx2(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::x86_64::{
        __m128i, __m256, _mm256_add_ps, _mm256_castps256_ps128, _mm256_cvtepi32_ps,
        _mm256_cvtepi8_epi32, _mm256_extractf128_ps, _mm256_fmadd_ps, _mm256_loadu_ps,
        _mm256_setzero_ps, _mm_add_ps, _mm_add_ss, _mm_cvtss_f32, _mm_loadl_epi64, _mm_loadu_si128,
        _mm_movehl_ps, _mm_shuffle_ps, _mm_srli_si128,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let mut acc0: __m256 = _mm256_setzero_ps();
    let mut acc1: __m256 = _mm256_setzero_ps();
    let mut i = 0usize;
    while i + 16 <= n {
        let raw = _mm_loadu_si128(ap.add(i).cast::<__m128i>());
        let a0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        let a1 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(_mm_srli_si128(raw, 8)));
        acc0 = _mm256_fmadd_ps(a0, _mm256_loadu_ps(bp.add(i)), acc0);
        acc1 = _mm256_fmadd_ps(a1, _mm256_loadu_ps(bp.add(i + 8)), acc1);
        i += 16;
    }
    if i + 8 <= n {
        let raw = _mm_loadl_epi64(ap.add(i).cast::<__m128i>());
        let a0 = _mm256_cvtepi32_ps(_mm256_cvtepi8_epi32(raw));
        acc0 = _mm256_fmadd_ps(a0, _mm256_loadu_ps(bp.add(i)), acc0);
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
        sum += f32::from(bytes[i].cast_signed()) * b[i];
        i += 1;
    }
    sum
}

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
#[target_feature(enable = "neon")]
unsafe fn sq8_dot_neon(bytes: &[u8], b: &[f32]) -> f32 {
    use std::arch::aarch64::{
        vaddq_f32, vaddvq_f32, vcvtq_f32_s32, vdupq_n_f32, vfmaq_f32, vget_high_s16, vget_low_s16,
        vld1_s8, vld1q_f32, vmovl_s16, vmovl_s8,
    };
    let n = bytes.len().min(b.len());
    let ap = bytes.as_ptr();
    let bp = b.as_ptr();
    let mut acc0 = vdupq_n_f32(0.0);
    let mut acc1 = vdupq_n_f32(0.0);
    let mut i = 0usize;
    while i + 8 <= n {
        let raw = vld1_s8(ap.add(i).cast::<i8>());
        let s16 = vmovl_s8(raw);
        let a0 = vcvtq_f32_s32(vmovl_s16(vget_low_s16(s16)));
        let a1 = vcvtq_f32_s32(vmovl_s16(vget_high_s16(s16)));
        acc0 = vfmaq_f32(acc0, a0, vld1q_f32(bp.add(i)));
        acc1 = vfmaq_f32(acc1, a1, vld1q_f32(bp.add(i + 4)));
        i += 8;
    }
    let mut sum = vaddvq_f32(vaddq_f32(acc0, acc1));
    while i < n {
        sum += f32::from(bytes[i].cast_signed()) * b[i];
        i += 1;
    }
    sum
}

/// Dot product between SQ8 i8 bytes and a float query slice.
/// Computes sum of (a[i] as f32 * b[i]).
#[inline]
pub fn sq8_dot(bytes: &[u8], b: &[f32]) -> f32 {
    let backend = backend();
    match backend {
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx512 => unsafe { sq8_dot_avx512(bytes, b) },
        #[cfg(all(target_arch = "x86_64", target_endian = "little"))]
        SimdBackend::Avx2 => unsafe { sq8_dot_avx2(bytes, b) },
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        SimdBackend::Neon => unsafe { sq8_dot_neon(bytes, b) },
        _ => sq8_dot_scalar(bytes, b),
    }
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

    #[test]
    fn sq8_kernels_match_reference() {
        for n in [1usize, 7, 8, 15, 16, 31, 32, 64, 128, 768] {
            let bytes: Vec<u8> = (0..n).map(|i| ((i * 17) % 255) as u8).collect();
            let q: Vec<f32> = (0..n).map(|i| ((i as f32 * 0.1).sin()) * 0.8).collect();

            let want_l2 = sq8_l2_sq_scalar(&bytes, &q);
            let got_l2 = sq8_l2_sq(&bytes, &q);
            assert!(
                (got_l2 - want_l2).abs() <= want_l2.abs() * 1e-4 + 1e-5,
                "sq8_l2 mismatch at n={n}: got={got_l2} want={want_l2}"
            );

            let want_dot = sq8_dot_scalar(&bytes, &q);
            let got_dot = sq8_dot(&bytes, &q);
            assert!(
                (got_dot - want_dot).abs() <= want_dot.abs() * 1e-4 + 1e-5,
                "sq8_dot mismatch at n={n}: got={got_dot} want={want_dot}"
            );
        }
    }

    #[test]
    fn f64_byte_kernels_match_reference() {
        for n in [0usize, 1, 3, 4, 7, 33, 128, 801] {
            let a: Vec<f64> = (0..n).map(|i| (i as f64 * 0.37).sin()).collect();
            let b: Vec<f64> = (0..n).map(|i| (i as f64 * 0.11).cos()).collect();
            let bytes: Vec<u8> = a.iter().flat_map(|x| x.to_le_bytes()).collect();
            let want: f64 = a.iter().zip(&b).map(|(x, y)| (x - y) * (x - y)).sum();
            let got = l2_sq_f64_bytes(&bytes, &b);
            assert!((got - want).abs() <= 1e-9 * want.max(1.0), "n={n}");

            if n >= 2 {
                let dt = a[0] - b[0];
                let sp: f64 = a[1..]
                    .iter()
                    .zip(&b[1..])
                    .map(|(x, y)| (x - y) * (x - y))
                    .sum();
                let want_cosh = 1.0 + ((sp - dt * dt) * 0.5).max(0.0);
                let got_cosh = lorentz_cosh_f64_bytes(&bytes, &b);
                assert!((got_cosh - want_cosh).abs() <= 1e-9 * want_cosh, "n={n}");
            }
        }
    }

    #[test]
    fn pair_dispatch_matches_scalar() {
        for n in [1usize, 15, 16, 63, 64, 65, 200, 768] {
            let a: Vec<u8> = (0..n)
                .flat_map(|i| ((i as f32) * 0.5).to_le_bytes())
                .collect();
            let b: Vec<u8> = (0..n)
                .flat_map(|i| ((i as f32) * 0.25 - 1.0).to_le_bytes())
                .collect();
            let want = l2_sq_pair_scalar(&a, &b);
            let got = l2_sq_pair_dispatch(backend(), &a, &b);
            assert!(
                (got - want).abs() <= 1e-3 * want.max(1.0),
                "n={n}: {got} vs {want}"
            );
        }
    }
}
