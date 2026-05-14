use crate::{HyperVector, Metric, QuantizedHyperVector, BinaryHyperVector};

pub struct HybridQuantizedVector {
    pub lorentz: Vec<f32>,
    pub euclidean: Vec<i8>,
    pub alpha: f32, // Used as a global scale or weight
}

impl HybridQuantizedVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let alpha_offset = bytes.len() - 4;
        let mut alpha_bytes = [0u8; 4];
        alpha_bytes.copy_from_slice(&bytes[alpha_offset..]);
        let alpha = f32::from_le_bytes(alpha_bytes);

        // Assume lorentz = 33, euclidean = 768 for hybrid vector for now
        let lorentz_dim = 33;
        let lorentz_bytes_len = lorentz_dim * 4;
        let euclidean_dim = alpha_offset - lorentz_bytes_len;

        let mut lorentz = Vec::with_capacity(lorentz_dim);
        for i in 0..lorentz_dim {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
            lorentz.push(f32::from_le_bytes(buf));
        }

        let mut euclidean = Vec::with_capacity(euclidean_dim);
        for i in lorentz_bytes_len..alpha_offset {
            euclidean.push(bytes[i] as i8);
        }

        Self { lorentz, euclidean, alpha }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.lorentz.len() * 4 + self.euclidean.len() + 4);
        for &c in &self.lorentz {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        for &c in &self.euclidean {
            bytes.push(c as u8);
        }
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    pub fn from_float(v: &HyperVector, lorentz_dim: usize, euclidean_dim: usize) -> Self {
        let mut lorentz = vec![0.0f32; lorentz_dim];
        for i in 0..lorentz_dim {
            lorentz[i] = v.coords[i] as f32;
        }

        let mut euclidean = vec![0i8; euclidean_dim];
        for i in 0..euclidean_dim {
            let val = v.coords[lorentz_dim + i];
            euclidean[i] = (val * 127.0).clamp(-127.0, 127.0) as i8;
        }

        Self {
            lorentz,
            euclidean,
            alpha: 1.0,
        }
    }

    #[inline(always)]
    pub fn lorentz_distance_to_float(&self, other: &HyperVector) -> f64 {
        let dim = self.lorentz.len();
        let mut inner = -f64::from(self.lorentz[0]) * other.coords[0];
        for i in 1..dim {
            inner += f64::from(self.lorentz[i]) * other.coords[i];
        }
        let arg = (-inner).max(1.0 + 1e-12);
        arg.acosh()
    }

    #[inline(always)]
    pub fn euclidean_distance_sq_to_float(&self, other: &HyperVector) -> f64 {
        self.euclidean_distance_sq_mrl(other, self.euclidean.len())
    }

    #[inline(always)]
    pub fn euclidean_distance_sq_mrl(&self, other: &HyperVector, euc_dim: usize) -> f64 {
        let limit = euc_dim.min(self.euclidean.len());
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
        let offset = self.lorentz.len();
        for i in 0..limit {
            let a = f64::from(self.euclidean[i]) * scale_inv_f64;
            let b = other.coords[offset + i];
            let d = a - b;
            sum += d * d;
        }
        sum
    }

    #[cfg(all(target_feature = "avx2", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    pub unsafe fn euclidean_simd_avx2(&self, other: &HyperVector, limit: usize) -> f64 {
        use std::arch::x86_64::*;
        let mut sum_v = _mm256_setzero_pd();
        let scale_v = _mm256_set1_pd(1.0 / 127.0);
        let offset = self.lorentz.len();
        
        for i in (0..limit).step_by(4) {
            let i8_vals = _mm_cvtsi32_si128(*(self.euclidean.as_ptr().add(i) as *const i32));
            let i32_vals = _mm_cvtepi8_epi32(i8_vals);
            let f64_vals = _mm256_cvtepi32_pd(i32_vals);
            let a = _mm256_mul_pd(f64_vals, scale_v);
            let b = _mm256_loadu_pd(other.coords.as_ptr().add(offset + i));
            let diff = _mm256_sub_pd(a, b);
            sum_v = _mm256_fmadd_pd(diff, diff, sum_v);
        }

        let mut res = [0.0; 4];
        _mm256_storeu_pd(res.as_mut_ptr(), sum_v);
        res[0] + res[1] + res[2] + res[3]
    }

    #[cfg(all(target_feature = "neon", target_arch = "aarch64"))]
    pub unsafe fn euclidean_simd_neon(&self, other: &HyperVector, limit: usize) -> f64 {
        use std::arch::aarch64::*;
        let mut sum_v = vdupq_n_f64(0.0);
        let scale = 1.0 / 127.0;
        let offset = self.lorentz.len();

        for i in (0..limit).step_by(2) {
            let a0 = f64::from(self.euclidean[i]) * scale;
            let a1 = f64::from(self.euclidean[i+1]) * scale;
            let b = vld1q_f64(other.coords.as_ptr().add(offset + i));
            let a = vcombine_f64(vcreate_f64(u64::from_ne_bytes(a0.to_ne_bytes())), vcreate_f64(u64::from_ne_bytes(a1.to_ne_bytes())));
            let diff = vsubq_f64(a, b);
            sum_v = vfmaq_f64(sum_v, diff, diff);
        }
        vgetq_lane_f64(sum_v, 0) + vgetq_lane_f64(sum_v, 1)
    }

    #[inline(always)]
    pub fn distance_mrl(&self, other: &HyperVector, euc_dim: usize) -> f64 {
        let d_lor = self.lorentz_distance_to_float(other);
        let d_euc = self.euclidean_distance_sq_mrl(other, euc_dim);
        d_lor + d_euc
    }
}

pub struct HybridMetric;

impl Metric for HybridMetric {
    fn name() -> &'static str {
        "hybrid"
    }

    fn distance(_a: &[f64], _b: &[f64]) -> f64 {
        // dynamic distance requires dimensions. We panic here.
        panic!("Use distance fn with layout");
    }

    fn distance_quantized(_a: &QuantizedHyperVector, _b: &HyperVector) -> f64 {
        panic!("Not implemented");
    }

    fn distance_binary(_a: &BinaryHyperVector, _b: &HyperVector) -> f64 {
        0.0
    }

    fn extrapolate_momentum(past: &[f64], current: &[f64], steps: f64) -> Result<Vec<f64>, String> {
        // Hybrid vectors have a fixed layout: 33 dims Lorentz + N dims Euclidean
        let lorentz_dim = 33;
        if past.len() < lorentz_dim || current.len() < lorentz_dim {
            return Err("Vector too short for Hybrid layout".into());
        }

        // 1. Extrapolate Lorentz part
        let l_past = &past[..lorentz_dim];
        let l_current = &current[..lorentz_dim];
        let l_next = crate::LorentzMetric::extrapolate_momentum(l_past, l_current, steps)?;

        // 2. Extrapolate Euclidean part
        let e_past = &past[lorentz_dim..];
        let e_current = &current[lorentz_dim..];
        let e_next = crate::EuclideanMetric::extrapolate_momentum(e_past, e_current, steps)?;

        // 3. Combine
        let mut result = Vec::with_capacity(past.len());
        result.extend_from_slice(&l_next);
        result.extend_from_slice(&e_next);
        Ok(result)
    }
}
