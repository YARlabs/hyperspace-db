use crate::{Metric, HyperVector, QuantizedHyperVector, BinaryHyperVector};

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy)]
pub struct Hybrid801QuantizedVector {
    pub lorentz: [f32; 33],
    pub euclidean: [i8; 768],
    pub alpha: f32, // Used as a global scale or weight
}

impl Hybrid801QuantizedVector {
    pub const SIZE: usize = std::mem::size_of::<Self>();

    pub fn from_bytes(bytes: &[u8]) -> &Self {
        assert!(bytes.len() >= Self::SIZE);
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                Self::SIZE,
            )
        }
    }

    pub fn from_float(v: &HyperVector<801>) -> Self {
        let mut lorentz = [0.0f32; 33];
        for i in 0..33 {
            lorentz[i] = v.coords[i] as f32;
        }

        let mut euclidean = [0i8; 768];
        // Standard scalar quantization for Euclidean part
        for i in 0..768 {
            let val = v.coords[33 + i];
            euclidean[i] = (val * 127.0).clamp(-127.0, 127.0) as i8;
        }

        Self {
            lorentz,
            euclidean,
            alpha: 1.0, // Default weight
        }
    }

    #[inline(always)]
    pub fn lorentz_distance_to_float(&self, other: &HyperVector<801>) -> f64 {
        let mut inner = -f64::from(self.lorentz[0]) * other.coords[0];
        for i in 1..33 {
            inner += f64::from(self.lorentz[i]) * other.coords[i];
        }
        let arg = (-inner).max(1.0 + 1e-12);
        arg.acosh()
    }

    #[inline(always)]
    pub fn euclidean_distance_sq_to_float(&self, other: &HyperVector<801>) -> f64 {
        self.euclidean_distance_sq_mrl(other, 768)
    }

    #[inline(always)]
    pub fn euclidean_distance_sq_mrl(&self, other: &HyperVector<801>, euc_dim: usize) -> f64 {
        let limit = euc_dim.min(768);
        const SCALE_INV: f32 = 1.0 / 127.0;

        #[cfg(all(target_feature = "avx2", target_arch = "x86_64"))]
        {
            if limit % 8 == 0 {
                return unsafe { self.euclidean_simd_avx2(other, limit) };
            }
        }

        #[cfg(all(target_feature = "neon", target_arch = "aarch64"))]
        {
            if limit % 4 == 0 {
                return unsafe { self.euclidean_simd_neon(other, limit) };
            }
        }

        // Fallback
        let mut sum = 0.0;
        let scale_inv_f64 = SCALE_INV as f64;
        for i in 0..limit {
            let a = f64::from(self.euclidean[i]) * scale_inv_f64;
            let b = other.coords[33 + i];
            let d = a - b;
            sum += d * d;
        }
        sum
    }

    #[cfg(all(target_feature = "avx2", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn euclidean_simd_avx2(&self, other: &HyperVector<801>, limit: usize) -> f64 {
        use std::arch::x86_64::*;
        let mut sum_v = _mm256_setzero_pd();
        let scale_v = _mm256_set1_pd(1.0 / 127.0);
        
        for i in (0..limit).step_by(4) {
            // Load 4 i8 values
            let i8_vals = _mm_cvtsi32_si128(*(self.euclidean.as_ptr().add(i) as *const i32));
            // Convert i8 to i32
            let i32_vals = _mm_cvtepi8_epi32(i8_vals);
            // Convert i32 to f64
            let f64_vals = _mm256_cvtepi32_pd(i32_vals);
            // Scale
            let a = _mm256_mul_pd(f64_vals, scale_v);
            // Load query
            let b = _mm256_loadu_pd(other.coords.as_ptr().add(33 + i));
            // Diff
            let diff = _mm256_sub_pd(a, b);
            // Square and add
            sum_v = _mm256_fmadd_pd(diff, diff, sum_v);
        }

        let mut res = [0.0; 4];
        _mm256_storeu_pd(res.as_mut_ptr(), sum_v);
        res[0] + res[1] + res[2] + res[3]
    }

    #[cfg(all(target_feature = "neon", target_arch = "aarch64"))]
    pub unsafe fn euclidean_simd_neon(&self, other: &HyperVector<801>, limit: usize) -> f64 {
        use std::arch::aarch64::*;
        let mut sum_v = vdupq_n_f64(0.0);
        let scale = 1.0 / 127.0;

        for i in (0..limit).step_by(2) {
            // Process 2 elements at a time (f64 NEON is 128-bit)
            let a0 = f64::from(self.euclidean[i]) * scale;
            let a1 = f64::from(self.euclidean[i+1]) * scale;
            let b = vld1q_f64(other.coords.as_ptr().add(33 + i));
            let a = vcombine_f64(vcreate_f64(u64::from_ne_bytes(a0.to_ne_bytes())), vcreate_f64(u64::from_ne_bytes(a1.to_ne_bytes())));
            let diff = vsubq_f64(a, b);
            sum_v = vfmaq_f64(sum_v, diff, diff);
        }
        vgetq_lane_f64(sum_v, 0) + vgetq_lane_f64(sum_v, 1)
    }

    #[inline(always)]
    pub fn distance_mrl(&self, other: &HyperVector<801>, euc_dim: usize) -> f64 {
        let d_lor = self.lorentz_distance_to_float(other);
        let d_euc = self.euclidean_distance_sq_mrl(other, euc_dim);
        d_lor + d_euc
    }
}

pub struct Hybrid801Metric;

impl Metric<801> for Hybrid801Metric {
    fn name() -> &'static str {
        "hybrid"
    }

    fn distance(a: &[f64; 801], b: &[f64; 801]) -> f64 {
        // d_total = d_lor + d_euc_sq
        // Lorentz (33D)
        let mut inner = -a[0] * b[0];
        for i in 1..33 {
            inner += a[i] * b[i];
        }
        let d_lor = (-inner).max(1.0 + 1e-12).acosh();

        // Euclidean (768D)
        let mut d_euc_sq = 0.0;
        for i in 33..801 {
            let d = a[i] - b[i];
            d_euc_sq += d * d;
        }

        d_lor + d_euc_sq
    }

    fn distance_quantized(a: &QuantizedHyperVector<801>, b: &HyperVector<801>) -> f64 {
        // Fallback if someone uses standard quantization for hybrid metric
        let mut inner = -f64::from(a.coords[0]) * b.coords[0] / 127.0; // Very rough
        for i in 1..33 {
            inner += f64::from(a.coords[i]) * b.coords[i] / 127.0;
        }
        let d_lor = (-inner).max(1.0 + 1e-12).acosh();

        let mut d_euc_sq = 0.0;
        for i in 33..801 {
            let d = f64::from(a.coords[i]) / 127.0 - b.coords[i];
            d_euc_sq += d * d;
        }
        d_lor + d_euc_sq
    }

    fn distance_binary(_a: &BinaryHyperVector<801>, _b: &HyperVector<801>) -> f64 {
        0.0
    }
}
