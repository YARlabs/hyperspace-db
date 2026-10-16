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
}
