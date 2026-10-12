//! Squared-L2 distance with a SIMD-friendly reduction.
//!
//! A single-accumulator `acc += d*d` loop does NOT auto-vectorize: float
//! addition isn't reassociated without fast-math, so LLVM must keep the
//! sequential dependency and emits scalar code. Using independent per-lane
//! accumulators breaks that chain, letting LLVM emit a vector FMA loop (NEON /
//! AVX / AVX512 via `target-cpu=native`) — the portable equivalent of the C++'s
//! explicit SIMD distance kernels.

const LANES: usize = 8;

#[inline]
pub fn l2_squared(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut acc = [0.0_f32; LANES];
    let (ca, ra) = a.as_chunks::<LANES>();
    let (cb, rb) = b.as_chunks::<LANES>();
    for (av, bv) in ca.iter().zip(cb) {
        // Independent accumulators — no cross-lane dependency, so this vectorizes.
        for l in 0..LANES {
            let d = av[l] - bv[l];
            acc[l] += d * d;
        }
    }
    let mut s = 0.0_f32;
    for l in 0..LANES {
        s += acc[l];
    }
    for (x, y) in ra.iter().zip(rb) {
        let d = x - y;
        s += d * d;
    }
    s
}

/// Squared L2 over the first `len` elements of `a` and `b`.
#[inline]
pub fn l2_squared_range(a: &[f32], b: &[f32], len: usize) -> f32 {
    l2_squared(&a[..len], &b[..len])
}

#[cfg(test)]
mod tests {
    use super::{l2_squared, l2_squared_range, LANES};

    #[test]
    fn distance_matches_scalar_across_chunk_boundaries() {
        for len in [0, 1, LANES - 1, LANES, LANES + 1, 2 * LANES, 2 * LANES + 3] {
            let a: Vec<f32> = (0..len).map(|i| i as f32 - 5.0).collect();
            let b: Vec<f32> = (0..len).map(|i| 2.0 * i as f32 + 1.0).collect();
            let expected: f32 = a.iter().zip(&b).map(|(x, y)| (x - y).powi(2)).sum();
            assert_eq!(l2_squared(&a, &b), expected, "length {len}");
            assert_eq!(l2_squared(&b, &a), expected, "length {len}");
            assert_eq!(l2_squared(&a, &a), 0.0, "length {len}");
        }
    }

    #[test]
    fn range_ignores_values_after_the_prefix() {
        let a = [1.0; LANES + 2];
        let mut b = [3.0; LANES + 3];
        b[LANES + 1] = 100.0;
        for len in [0, LANES - 1, LANES, LANES + 1] {
            assert_eq!(l2_squared_range(&a, &b, len), 4.0 * len as f32);
        }
    }
}
