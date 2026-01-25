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
    pub fn new(coords: [f64; N]) -> Result<Self, String> {
        let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector must be strictly inside the Poincaré ball".to_string());
        }
        let alpha = 1.0 / (1.0 - sq_norm);
        Ok(Self { coords, alpha })
    }

    /// The hottest function in the entire project.
    #[inline(always)]
    pub fn poincare_distance_sq(&self, other: &Self) -> f64 {
        // 1. Calculate Squared Euclidean distance (L2 squared) using SIMD
        let mut sum_sq_diff = f64x8::splat(0.0);

        // LANES = 8 for f64x8
        const LANES: usize = 8;

        // Assert at compile time (ideal) or runtime that N is a multiple of LANES for this MVP.
        // In production, loop tail handling is needed here.
        for i in (0..N).step_by(LANES) {
            // Unsafe load - we guarantee align(64) and boundaries via struct definition
            // Note: In real production code, bounds checks or unsafe assurances should be rigorous.
            // Since N is const generic, let's assume valid slices for this MVP step.
            let a = f64x8::from_slice(&self.coords[i..i + LANES]);
            let b = f64x8::from_slice(&other.coords[i..i + LANES]);
            let diff = a - b;
            // Fused Multiply-Add
            sum_sq_diff += diff * diff;
        }

        let l2_sq = sum_sq_diff.reduce_sum();

        // 2. Formula: delta = ||u-v||^2 / ((1-||u||^2)(1-||v||^2))
        // We store alpha = 1/(1-||u||^2), so:
        let delta = l2_sq * self.alpha * other.alpha;

        // 3. Return 1 + 2*delta.
        // We do NOT take Acosh for sorting/comparing (monotonicity).
        // Acosh is taken only when returning the result to the user.
        1.0 + 2.0 * delta
    }

    /// Real distance for user output
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
            // Scale [-1, 1] mapped to [-127, 127]
            let val = (src * 127.0).clamp(-127.0, 127.0);
            *dst = val as i8;
        }

        Self {
            coords,
            alpha: v.alpha as f32,
        }
    }

    /// Calculate distance between Quantized Vector (Storage) and Float Vector (Query)
    /// We dequantize 'self' on the fly.
    #[inline(always)]
    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector<N>) -> f64 {
        let mut sum_sq_diff = f64x8::splat(0.0);
        const LANES: usize = 8;
        const SCALE_INV: f64 = 1.0 / 127.0;

        for i in (0..N).step_by(LANES) {
            // Load 8 i8s
            let a_i8 = Simd::<i8, LANES>::from_slice(&self.coords[i..i + LANES]);
            // Cast to f64 (via cast -> i32 -> f64? or direct?)
            // Simd::cast is available for primitive numeric types
            let a_f64: Simd<f64, LANES> = a_i8.cast();

            // Dequantize: a_f64 / 127.0
            let a_scaled = a_f64 * Simd::splat(SCALE_INV);

            let b = Simd::<f64, LANES>::from_slice(&query.coords[i..i + LANES]);

            let diff = a_scaled - b;
            sum_sq_diff += diff * diff;
        }

        let l2_sq = sum_sq_diff.reduce_sum();
        let delta = l2_sq * (self.alpha as f64) * query.alpha;

        1.0 + 2.0 * delta
    }
}

impl<const N: usize> HyperVector<N> {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, Self::SIZE) }
    }

    pub fn from_bytes(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
}

impl<const N: usize> QuantizedHyperVector<N> {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const _ as *const u8, Self::SIZE) }
    }

    pub fn from_bytes(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hypervector_creation() {
        let coords = [0.1, 0.2, 0.3, 0.4, 0.1, 0.2, 0.3, 0.4];
        let v = HyperVector::new(coords).unwrap();
        assert_eq!(v.coords[0], 0.1);
        assert!(v.alpha > 0.0);
    }

    #[test]
    fn test_quantization_roundtrip() {
        let coords = [0.1, -0.1, 0.1, -0.1, 0.1, -0.1, 0.1, -0.1];
        let v = HyperVector::new(coords).unwrap();
        let q = QuantizedHyperVector::from_float(&v);

        // Check if coords are roughly preserved in sign
        assert!(q.coords[0] > 0);
        assert!(q.coords[1] < 0);
    }
}
