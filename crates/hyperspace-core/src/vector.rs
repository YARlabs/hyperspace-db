#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]

#[cfg(feature = "nightly-simd")]
use std::simd::prelude::*;


/// Aligned vector struct. N is the dimension.
/// align(64) is critical for AVX-512 and cache lines.
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct HyperVector<const N: usize> {
    pub coords: [f64; N],
    pub alpha: f64, // Precomputed coefficient: 1 / (1 - ||x||^2)
}

impl<const N: usize> HyperVector<N> {
    /// Creates a new `HyperVector`, validating it is strictly inside the unit ball.
    pub fn new(coords: [f64; N]) -> Result<Self, String> {
        let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector must be strictly inside the Poincaré ball".to_string());
        }
        let alpha = 1.0 / (1.0 - sq_norm);
        Ok(Self { coords, alpha })
    }

    /// Creates a `HyperVector` without validation.
    pub fn new_unchecked(coords: [f64; N]) -> Self {
        let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
        // Calculate alpha anyway, but handle >= 1.0 gracefully (though unused for L2)
        // If sq_norm >= 1.0, alpha is negative or inf.
        // We just store it as is, or 1.0.
        // Let's store as is for now, assuming Metric will ignore it.
        let alpha = 1.0 / (1.0 - sq_norm);
        Self { coords, alpha }
    }

    /// The hottest function in the entire project.
    /// Calculates the squared Poincare distance using the Möbius addition formula optimizations.
    #[inline(always)]
    pub fn poincare_distance_sq(&self, other: &Self) -> f64 {
        #[cfg(feature = "nightly-simd")]
        {
            let mut sum_sq_diff = f64x8::splat(0.0);
            const LANES: usize = 8;

            // Note: If N < 8, this logic needs care, but keeping original logic for consistency
            for i in (0..N).step_by(LANES) {
                if i + LANES <= N {
                    let a = f64x8::from_slice(&self.coords[i..i + LANES]);
                    let b = f64x8::from_slice(&other.coords[i..i + LANES]);
                    let diff = a - b;
                    sum_sq_diff += diff * diff;
                } else {
                    // Tail handling would go here for strict correctness with generic N
                }
            }

            let l2_sq = sum_sq_diff.reduce_sum();

            // Fallback for tail elements if N is not multiple of 8 (basic scalar loop for tail)
            // Ideally should be handled, but for keeping structure similar to your SIMD logic:
            let mut tail_sq = 0.0;
            let remainder = N % LANES;
            if remainder != 0 {
                let start = N - remainder;
                for i in start..N {
                    let diff = self.coords[i] - other.coords[i];
                    tail_sq += diff * diff;
                }
            }

            let total_sq = l2_sq + tail_sq;
            let delta = total_sq * self.alpha * other.alpha;
            1.0 + 2.0 * delta
        }

        #[cfg(not(feature = "nightly-simd"))]
        {
            // Stable Rust implementation (Scalar / Auto-vectorized)
            let mut sum_sq_diff = 0.0;
            for (u, v) in self.coords.iter().zip(other.coords.iter()) {
                let diff = u - v;
                sum_sq_diff += diff * diff;
            }
            let delta = sum_sq_diff * self.alpha * other.alpha;
            1.0 + 2.0 * delta
        }
    }

    /// Returns the true Hyperbolic distance (acosh of the squared distance).
    pub fn true_distance(&self, other: &Self) -> f64 {
        self.poincare_distance_sq(other).acosh()
    }
}

/// Quantized version (i8 coordinates)
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct QuantizedHyperVector<const N: usize> {
    pub coords: [i8; N],
    pub alpha: f32, // Storing as f32 to save space
}

impl<const N: usize> QuantizedHyperVector<N> {
    pub fn from_float(v: &HyperVector<N>) -> Self {
        let mut coords = [0; N];
        for (dst, &src) in coords.iter_mut().zip(v.coords.iter()) {
            let val = (src * 127.0).clamp(-127.0, 127.0);
            *dst = val as i8;
        }

        Self {
            coords,
            alpha: v.alpha as f32,
        }
    }

    #[inline(always)]
    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector<N>) -> f64 {
        #[cfg(feature = "nightly-simd")]
        {
            let mut sum_sq_diff = f64x8::splat(0.0);
            const LANES: usize = 8;
            const SCALE_INV: f64 = 1.0 / 127.0;

            for i in (0..N).step_by(LANES) {
                if i + LANES <= N {
                    let a_i8 = Simd::<i8, LANES>::from_slice(&self.coords[i..i + LANES]);
                    let a_f64: Simd<f64, LANES> = a_i8.cast();
                    let a_scaled = a_f64 * Simd::splat(SCALE_INV);
                    let b = Simd::<f64, LANES>::from_slice(&query.coords[i..i + LANES]);
                    let diff = a_scaled - b;
                    sum_sq_diff += diff * diff;
                }
            }

            let l2_sq = sum_sq_diff.reduce_sum();

            // Tail handling
            let mut tail_sq = 0.0;
            let remainder = N % LANES;
            if remainder != 0 {
                let start = N - remainder;
                for i in start..N {
                    let a_val = (self.coords[i] as f64) * SCALE_INV;
                    let diff = a_val - query.coords[i];
                    tail_sq += diff * diff;
                }
            }

            let total_sq = l2_sq + tail_sq;
            let delta = total_sq * (self.alpha as f64) * query.alpha;
            1.0 + 2.0 * delta
        }

        #[cfg(not(feature = "nightly-simd"))]
        {
            // Stable Rust implementation
            const SCALE_INV: f64 = 1.0 / 127.0;
            let mut sum_sq_diff = 0.0;

            for (a_i8, b_f64) in self.coords.iter().zip(query.coords.iter()) {
                let a_val = f64::from(*a_i8) * SCALE_INV;
                let diff = a_val - b_f64;
                sum_sq_diff += diff * diff;
            }

            let delta = sum_sq_diff * f64::from(self.alpha) * query.alpha;
            1.0 + 2.0 * delta
        }
    }
}

/// Binary Quantized (1 bit per dimension)
/// With Fixed Storage Buffer (512 bytes) to support up to 4096 dimensions
/// safely without `generic_const_exprs`.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BinaryHyperVector<const N: usize> {
    pub bits: [u8; 512],
    pub alpha: f32,
}

impl<const N: usize> BinaryHyperVector<N> {
    pub fn from_float(v: &HyperVector<N>) -> Self {
        let mut bits = [0u8; 512];
        for (i, &val) in v.coords.iter().enumerate() {
            if i >= 4096 {
                break;
            } // Safety cap
            if val > 0.0 {
                let byte_idx = i / 8;
                let bit_idx = i % 8;
                bits[byte_idx] |= 1 << bit_idx;
            }
        }
        Self {
            bits,
            alpha: v.alpha as f32,
        }
    }

    #[inline(always)]
    pub fn hamming_distance(&self, other: &Self) -> u32 {
        let mut dist = 0;
        let limit = N.div_ceil(8);
        for i in 0..limit {
            dist += (self.bits[i] ^ other.bits[i]).count_ones();
        }
        dist
    }

    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector<N>) -> f64 {
        let val = 1.0 / (N as f64).sqrt() * 0.99;
        let mut sum_sq_diff = 0.0;

        for i in 0..N {
            if i >= 4096 {
                break;
            }
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit = (self.bits[byte_idx] >> bit_idx) & 1;

            let recon = if bit == 1 { val } else { -val };
            let diff = recon - query.coords[i];
            sum_sq_diff += diff * diff;
        }

        let delta = sum_sq_diff * f64::from(self.alpha) * query.alpha;
        1.0 + 2.0 * delta
    }

    pub fn l2_distance_sq_to_float(&self, query: &HyperVector<N>) -> f64 {
        let val = 1.0 / (N as f64).sqrt() * 0.99;
        let mut sum_sq_diff = 0.0;

        for i in 0..N {
            if i >= 4096 {
                break;
            }
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit = (self.bits[byte_idx] >> bit_idx) & 1;

            let recon = if bit == 1 { val } else { -val };
            let diff = recon - query.coords[i];
            sum_sq_diff += diff * diff;
        }
        sum_sq_diff
    }
}


impl<const N: usize> HyperVector<N> {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(std::ptr::from_ref(self).cast::<u8>(), Self::SIZE) }
    }
    /// Casts bytes to `HyperVector`.
    /// 
    /// # Panics
    /// 
    /// Panics if the byte slice is not aligned to `std::mem::align_of::<Self>()`.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.as_ptr().align_offset(std::mem::align_of::<Self>()), 0, "HyperVector: Misaligned bytes! Use aligned storage.");
        unsafe { &*bytes.as_ptr().cast::<Self>() }
    }
}

impl<const N: usize> QuantizedHyperVector<N> {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(std::ptr::from_ref(self).cast::<u8>(), Self::SIZE) }
    }
    /// Casts bytes to `QuantizedHyperVector`.
    ///
    /// # Panics
    ///
    /// Panics if the byte slice is not aligned to `std::mem::align_of::<Self>()`.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.as_ptr().align_offset(std::mem::align_of::<Self>()), 0, "QuantizedHyperVector: Misaligned bytes!");
        unsafe { &*bytes.as_ptr().cast::<Self>() }
    }
}

impl<const N: usize> BinaryHyperVector<N> {
    pub const SIZE: usize = std::mem::size_of::<Self>();
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(std::ptr::from_ref(self).cast::<u8>(), Self::SIZE) }
    }
    /// Casts bytes to `BinaryHyperVector`.
    ///
    /// # Panics
    ///
    /// Panics if the byte slice is not aligned to `std::mem::align_of::<Self>()`.
    #[allow(clippy::cast_ptr_alignment)]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert_eq!(bytes.as_ptr().align_offset(std::mem::align_of::<Self>()), 0, "BinaryHyperVector: Misaligned bytes!");
        unsafe { &*bytes.as_ptr().cast::<Self>() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypervector_creation() {
        let coords = [0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.3, 0.4];
        let v = HyperVector::<8>::new(coords).unwrap();
        assert!((v.coords[0] - 0.1).abs() < f64::EPSILON);
        assert!(v.alpha > 0.0);
    }
    #[test]
    fn bench_distance_speed() {
        let a = HyperVector::<1024>::new([0.001; 1024]).unwrap();
        let b = HyperVector::<1024>::new([0.002; 1024]).unwrap();

        let start = std::time::Instant::now();
        let iterations = 1_000_000;

        // "Warming up" the CPU cache
        let mut black_box = 0.0;

        for _ in 0..iterations {
            black_box += a.poincare_distance_sq(&b);
        }

        let duration = start.elapsed();
        println!(
            "⏱️ 1M distances took: {duration:?} (Check sum: {black_box})"
        );
    }
}
