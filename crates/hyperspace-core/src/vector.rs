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
            let a = f64x8::from_slice(&self.coords[i..i+LANES]);
            let b = f64x8::from_slice(&other.coords[i..i+LANES]);
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
