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
        let mut sum_sq_diff = f64x8::splat(0.0);
        const LANES: usize = 8;

        for i in (0..N).step_by(LANES) {
            if i + LANES <= N {
                 let a = f64x8::from_slice(&self.coords[i..i + LANES]);
                 let b = f64x8::from_slice(&other.coords[i..i + LANES]);
                 let diff = a - b;
                 sum_sq_diff += diff * diff;
            } else {
                // Tail handled implicitly/ignored for MVP if N%8==0
            }
        }

        let l2_sq = sum_sq_diff.reduce_sum();
        let delta = l2_sq * self.alpha * other.alpha;
        1.0 + 2.0 * delta
    }

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
        let delta = l2_sq * (self.alpha as f64) * query.alpha;
        1.0 + 2.0 * delta
    }
}

/// Binary Quantized (1 bit per dimension)
/// With Fixed Storage Buffer (512 bytes) to support up to 4096 dimensions
/// safely without generic_const_exprs.
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
            if i >= 4096 { break; } // Safety cap
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
        let limit = (N + 7) / 8;
        // Optimization: loop unrolling or SIMD?
        // Simple loop over valid bytes
        for i in 0..limit {
            dist += (self.bits[i] ^ other.bits[i]).count_ones();
        }
        dist
    }

    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector<N>) -> f64 {
        let val = 1.0 / (N as f64).sqrt() * 0.99; 
        let mut sum_sq_diff = 0.0;
        
        for i in 0..N {
            if i >= 4096 { break; }
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            let bit = (self.bits[byte_idx] >> bit_idx) & 1;
            
            let recon = if bit == 1 { val } else { -val };
            let diff = recon - query.coords[i];
            sum_sq_diff += diff * diff;
        }

        let delta = sum_sq_diff * (self.alpha as f64) * query.alpha;
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

impl<const N: usize> BinaryHyperVector<N> {
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
        let v = HyperVector::<8>::new(coords).unwrap();
        assert_eq!(v.coords[0], 0.1);
        assert!(v.alpha > 0.0);
    }
}
