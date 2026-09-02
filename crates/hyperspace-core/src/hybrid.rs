use crate::{BinaryHyperVector, HyperVector, Metric, QuantizedHyperVector};

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
            euclidean.push(bytes[i].cast_signed());
        }

        Self {
            lorentz,
            euclidean,
            alpha,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.lorentz.len() * 4 + self.euclidean.len() + 4);
        for &c in &self.lorentz {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        for &c in &self.euclidean {
            bytes.push(c.cast_unsigned());
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
            if limit.is_multiple_of(8) {
                return unsafe { self.euclidean_simd_avx2(other, limit) };
            }
        }

        #[cfg(all(target_feature = "neon", target_arch = "aarch64"))]
        {
            if limit.is_multiple_of(4) {
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
    /// # Safety
    /// Caller must ensure:
    /// - `limit` is a multiple of 4.
    /// - `limit` is within the bounds of `other.coords` and `self.euclidean`.
    /// - The target architecture supports AVX2.
    pub unsafe fn euclidean_simd_avx2(&self, other: &HyperVector, limit: usize) -> f64 {
        use std::arch::x86_64::{
            _mm256_cvtepi32_pd, _mm256_fmadd_pd, _mm256_loadu_pd, _mm256_mul_pd, _mm256_set1_pd,
            _mm256_setzero_pd, _mm256_storeu_pd, _mm256_sub_pd, _mm_cvtepi8_epi32,
            _mm_cvtsi32_si128,
        };
        let mut sum_v = _mm256_setzero_pd();
        let scale_v = _mm256_set1_pd(1.0 / 127.0);
        let offset = self.lorentz.len();

        for i in (0..limit).step_by(4) {
            let val_bytes = self
                .euclidean
                .as_ptr()
                .add(i)
                .cast::<[u8; 4]>()
                .read_unaligned();
            let i8_vals = _mm_cvtsi32_si128(i32::from_ne_bytes(val_bytes));
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
    /// # Safety
    /// Caller must ensure `limit` is a multiple of 2 and within bounds of `other.coords`.
    pub unsafe fn euclidean_simd_neon(&self, other: &HyperVector, limit: usize) -> f64 {
        #[allow(clippy::wildcard_imports)]
        use std::arch::aarch64::*;
        let mut sum_v = vdupq_n_f64(0.0);
        let scale = 1.0 / 127.0;
        let offset = self.lorentz.len();

        for i in (0..limit).step_by(2) {
            let a0 = f64::from(self.euclidean[i]) * scale;
            let a1 = f64::from(self.euclidean[i + 1]) * scale;
            let b = vld1q_f64(other.coords.as_ptr().add(offset + i));
            let a = vcombine_f64(
                vcreate_f64(u64::from_ne_bytes(a0.to_ne_bytes())),
                vcreate_f64(u64::from_ne_bytes(a1.to_ne_bytes())),
            );
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

    fn distance(a: &[f64], b: &[f64]) -> f64 {
        if a.len() < 33 || b.len() < 33 {
            return 0.0;
        }
        let d_lor = <crate::LorentzMetric as crate::Metric>::distance(&a[..33], &b[..33]);
        let d_euc = <crate::EuclideanMetric as crate::Metric>::distance(&a[33..], &b[33..]);
        d_lor + d_euc
    }

    fn distance_quantized(a: &QuantizedHyperVector, b: &HyperVector) -> f64 {
        if a.coords.len() < 33 || b.coords.len() < 33 {
            return 0.0;
        }
        let inv_127 = 1.0 / 127.0;
        let scale = f64::from(a.alpha);

        // 1. Dequantize Lorentz part and compute Lorentz distance
        let mut a_deq_lor = [0.0f64; 33];
        for i in 0..33 {
            a_deq_lor[i] = (f64::from(a.coords[i]) * inv_127) * scale;
        }
        let d_lor = <crate::LorentzMetric as crate::Metric>::distance(&a_deq_lor, &b.coords[..33]);

        // 2. Compute Euclidean distance on the quantized Euclidean part
        let mut d_euc = 0.0;
        for i in 33..a.coords.len() {
            if i >= b.coords.len() {
                break;
            }
            let a_val = f64::from(a.coords[i]) * inv_127;
            let diff = a_val - b.coords[i];
            d_euc += diff * diff;
        }

        d_lor + d_euc
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

pub struct HybridLowBitQuantizedVector {
    pub lorentz: Vec<f32>,
    pub euclidean_packed: Vec<u8>,
    pub scales: Vec<f32>,
}

impl HybridLowBitQuantizedVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let lorentz_dim = 33;
        let lorentz_bytes_len = lorentz_dim * 4;

        let mut lorentz = Vec::with_capacity(lorentz_dim);
        for i in 0..lorentz_dim {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
            lorentz.push(f32::from_le_bytes(buf));
        }

        let mut scales_count_bytes = [0u8; 4];
        scales_count_bytes.copy_from_slice(&bytes[lorentz_bytes_len..lorentz_bytes_len + 4]);
        let scales_count = u32::from_le_bytes(scales_count_bytes) as usize;

        let scales_offset = lorentz_bytes_len + 4;
        let mut scales = Vec::with_capacity(scales_count);
        for i in 0..scales_count {
            let mut buf = [0u8; 4];
            let offset = scales_offset + i * 4;
            buf.copy_from_slice(&bytes[offset..offset + 4]);
            scales.push(f32::from_le_bytes(buf));
        }

        let packed_offset = scales_offset + scales_count * 4;
        let euclidean_packed = bytes[packed_offset..].to_vec();

        Self {
            lorentz,
            euclidean_packed,
            scales,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes =
            Vec::with_capacity(132 + 4 + self.scales.len() * 4 + self.euclidean_packed.len());
        for &c in &self.lorentz {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        let scales_count = self.scales.len() as u32;
        bytes.extend_from_slice(&scales_count.to_le_bytes());
        for &s in &self.scales {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes.extend_from_slice(&self.euclidean_packed);
        bytes
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn from_float(v: &HyperVector, lorentz_dim: usize, euclidean_dim: usize) -> Self {
        let mut lorentz = vec![0.0f32; lorentz_dim];
        for i in 0..lorentz_dim {
            lorentz[i] = v.coords[i] as f32;
        }

        let block_size = 16;
        let num_blocks = euclidean_dim.div_ceil(block_size);
        let mut scales = Vec::with_capacity(num_blocks);
        let mut euclidean_packed = Vec::with_capacity(num_blocks * 8);

        for b in 0..num_blocks {
            let start = b * block_size;
            let end = (start + block_size).min(euclidean_dim);

            let mut max_val = 0.0f64;
            for i in start..end {
                let val = v.coords[lorentz_dim + i].abs();
                if val > max_val {
                    max_val = val;
                }
            }

            let scale = if max_val > 1e-9 { max_val / 7.0 } else { 1.0 };
            scales.push(scale as f32);

            let scale_inv = 1.0 / scale;
            let mut block_q = vec![0i8; block_size];
            for i in 0..block_size {
                let idx = start + i;
                if idx < euclidean_dim {
                    let val = v.coords[lorentz_dim + idx];
                    let q = (val * scale_inv).round().clamp(-7.0, 7.0) as i8;
                    block_q[i] = q;
                } else {
                    block_q[i] = 0;
                }
            }

            for i in (0..block_size).step_by(2) {
                let q1 = block_q[i];
                let q2 = block_q[i + 1];
                let u1 = (q1 + 8) as u8;
                let u2 = (q2 + 8) as u8;
                let packed = (u1 << 4) | (u2 & 0x0F);
                euclidean_packed.push(packed);
            }
        }

        Self {
            lorentz,
            euclidean_packed,
            scales,
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
    #[allow(clippy::cast_possible_wrap)]
    pub fn euclidean_distance_sq_mrl(&self, other: &HyperVector, euc_dim: usize) -> f64 {
        let offset = self.lorentz.len();
        let block_size = 16;
        let mut sum = 0.0;

        let num_blocks = euc_dim.div_ceil(block_size);
        for b in 0..num_blocks {
            let start = b * block_size;
            let end = (start + block_size).min(euc_dim);
            let scale = f64::from(self.scales[b]);

            let packed_block_offset = b * 8;
            for i in start..end {
                let relative_idx = i - start;
                let pair_idx = relative_idx / 2;
                let is_second = relative_idx % 2 == 1;

                let packed_byte = self.euclidean_packed[packed_block_offset + pair_idx];
                let u_val = if is_second {
                    packed_byte & 0x0F
                } else {
                    packed_byte >> 4
                };
                let q = (u_val as i8) - 8;
                let a = f64::from(q) * scale;
                let b_val = other.coords[offset + i];
                let d = a - b_val;
                sum += d * d;
            }
        }
        sum
    }

    #[inline(always)]
    pub fn distance_mrl(&self, other: &HyperVector, euc_dim: usize) -> f64 {
        let d_lor = self.lorentz_distance_to_float(other);
        let d_euc = self.euclidean_distance_sq_mrl(other, euc_dim);
        d_lor + d_euc
    }
}

pub struct ScalarI4Vector {
    pub head: Vec<f32>,
    pub scales: Vec<f32>,
    pub packed_tail: Vec<u8>,
}

impl ScalarI4Vector {
    pub fn from_bytes(bytes: &[u8], _dim: usize, head_dim: usize) -> Self {
        let head_bytes_len = head_dim * 4;
        let mut head = Vec::with_capacity(head_dim);
        for i in 0..head_dim {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
            head.push(f32::from_le_bytes(buf));
        }

        let mut scales_count_bytes = [0u8; 4];
        scales_count_bytes.copy_from_slice(&bytes[head_bytes_len..head_bytes_len + 4]);
        let scales_count = u32::from_le_bytes(scales_count_bytes) as usize;

        let scales_offset = head_bytes_len + 4;
        let mut scales = Vec::with_capacity(scales_count);
        for i in 0..scales_count {
            let mut buf = [0u8; 4];
            let offset = scales_offset + i * 4;
            buf.copy_from_slice(&bytes[offset..offset + 4]);
            scales.push(f32::from_le_bytes(buf));
        }

        let packed_offset = scales_offset + scales_count * 4;
        let packed_tail = bytes[packed_offset..].to_vec();

        Self {
            head,
            scales,
            packed_tail,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            self.head.len() * 4 + 4 + self.scales.len() * 4 + self.packed_tail.len(),
        );
        for &c in &self.head {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        let scales_count = self.scales.len() as u32;
        bytes.extend_from_slice(&scales_count.to_le_bytes());
        for &s in &self.scales {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        bytes.extend_from_slice(&self.packed_tail);
        bytes
    }

    #[allow(clippy::cast_sign_loss)]
    pub fn from_float(v: &HyperVector, head_dim: usize) -> Self {
        let head_dim = head_dim.min(v.coords.len());
        let mut head = vec![0.0f32; head_dim];
        for i in 0..head_dim {
            head[i] = v.coords[i] as f32;
        }

        let tail_dim = v.coords.len().saturating_sub(head_dim);
        let block_size = 16;
        let num_blocks = tail_dim.div_ceil(block_size);
        let mut scales = Vec::with_capacity(num_blocks);
        let mut packed_tail = Vec::with_capacity(num_blocks * (block_size / 2));

        for b in 0..num_blocks {
            let start = b * block_size;
            let end = (start + block_size).min(tail_dim);

            let mut max_val = 0.0f64;
            for i in start..end {
                let val = v.coords[head_dim + i].abs();
                if val > max_val {
                    max_val = val;
                }
            }

            let scale = if max_val > 1e-9 { max_val / 7.0 } else { 1.0 };
            scales.push(scale as f32);

            let scale_inv = 1.0 / scale;
            let mut block_q = vec![0i8; block_size];
            for i in 0..block_size {
                let idx = start + i;
                if idx < tail_dim {
                    let val = v.coords[head_dim + idx];
                    let q = (val * scale_inv).round().clamp(-7.0, 7.0) as i8;
                    block_q[i] = q;
                }
            }

            for pair in 0..(block_size / 2) {
                let q1 = (block_q[pair * 2] + 8) as u8 & 0x0F;
                let q2 = (block_q[pair * 2 + 1] + 8) as u8 & 0x0F;
                let packed_byte = (q1 << 4) | q2;
                packed_tail.push(packed_byte);
            }
        }

        Self {
            head,
            scales,
            packed_tail,
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    pub fn reconstruct(&self, total_dim: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(total_dim);
        let head_len = self.head.len().min(total_dim);
        for &h in &self.head[..head_len] {
            out.push(f64::from(h));
        }

        let tail_dim = total_dim.saturating_sub(head_len);
        let block_size = 16;
        let num_blocks = self.scales.len();
        for b in 0..num_blocks {
            let scale = f64::from(self.scales[b]);
            let start = b * block_size;
            let end = (start + block_size).min(tail_dim);
            let packed_block_offset = b * 8;

            for i in start..end {
                let relative_idx = i - start;
                let pair_idx = relative_idx / 2;
                let is_second = relative_idx % 2 == 1;

                if packed_block_offset + pair_idx >= self.packed_tail.len() {
                    break;
                }
                let packed_byte = self.packed_tail[packed_block_offset + pair_idx];
                let u_val = if is_second {
                    packed_byte & 0x0F
                } else {
                    packed_byte >> 4
                };
                let q = (u_val as i8) - 8;
                out.push(f64::from(q) * scale);
            }
        }
        out
    }

    #[inline(always)]
    #[allow(clippy::cast_possible_wrap)]
    pub fn distance_l2_sq_mrl(&self, other: &HyperVector, active_dim: usize) -> f64 {
        let mut sum = 0.0;
        let head_len = self.head.len().min(active_dim);
        for i in 0..head_len {
            if i >= other.coords.len() {
                break;
            }
            let diff = f64::from(self.head[i]) - other.coords[i];
            sum += diff * diff;
        }

        if active_dim <= head_len {
            return sum;
        }

        let tail_active = active_dim - head_len;
        let block_size = 16;
        let num_blocks = self.scales.len();

        for b in 0..num_blocks {
            let start = b * block_size;
            if start >= tail_active {
                break;
            }
            let end = (start + block_size).min(tail_active);
            let scale = f64::from(self.scales[b]);
            let packed_block_offset = b * 8;

            for i in start..end {
                let relative_idx = i - start;
                let pair_idx = relative_idx / 2;
                let is_second = relative_idx % 2 == 1;
                let other_idx = head_len + i;
                if other_idx >= other.coords.len()
                    || packed_block_offset + pair_idx >= self.packed_tail.len()
                {
                    break;
                }

                let packed_byte = self.packed_tail[packed_block_offset + pair_idx];
                let u_val = if is_second {
                    packed_byte & 0x0F
                } else {
                    packed_byte >> 4
                };
                let q = (u_val as i8) - 8;
                let a = f64::from(q) * scale;
                let b_val = other.coords[other_idx];
                let d = a - b_val;
                sum += d * d;
            }
        }
        sum
    }
}

pub const TURBOQUANT_CENTROIDS_4BIT: [f64; 16] = [
    -2.401, -1.844, -1.437, -1.099, -0.800, -0.524, -0.262, -0.066, 0.066, 0.262, 0.524, 0.800,
    1.099, 1.437, 1.844, 2.401,
];

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurboVector {
    pub norm: f32,
    pub head: Vec<f32>,
    pub packed_tail: Vec<u8>,
}

pub type TurboQuantVector = TurboVector;

impl TurboVector {
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_float(vec: &HyperVector, head_dim: usize) -> Self {
        let coords = &vec.coords;
        let total_dim = coords.len();
        let head_len = head_dim.min(total_dim);

        let head: Vec<f32> = coords[..head_len].iter().map(|&x| x as f32).collect();
        let tail = &coords[head_len..];
        let tail_dim = tail.len();

        let mut norm_sq = 0.0;
        for &x in tail {
            norm_sq += x * x;
        }
        let norm = norm_sq.sqrt() as f32;

        if tail_dim == 0 || norm < 1e-9 {
            let packed_tail_len = tail_dim.div_ceil(2);
            return Self {
                norm: 0.0,
                head,
                packed_tail: vec![0; packed_tail_len],
            };
        }

        let scale = 1.0 / (tail_dim as f64).sqrt();
        let inv_norm = 1.0 / (f64::from(norm));

        let packed_tail_len = tail_dim.div_ceil(2);
        let mut packed_tail = vec![0u8; packed_tail_len];

        for i in 0..tail_dim {
            let val = (tail[i] * inv_norm) / scale;
            let mut best_idx = 0;
            let mut best_diff = (val - TURBOQUANT_CENTROIDS_4BIT[0]).abs();

            for (c_idx, &c_val) in TURBOQUANT_CENTROIDS_4BIT.iter().enumerate().skip(1) {
                let diff = (val - c_val).abs();
                if diff < best_diff {
                    best_diff = diff;
                    best_idx = c_idx;
                }
            }

            let pair_idx = i / 2;
            let is_second = i % 2 == 1;
            let bin = (best_idx & 0x0F) as u8;

            if is_second {
                packed_tail[pair_idx] |= bin;
            } else {
                packed_tail[pair_idx] |= bin << 4;
            }
        }

        Self {
            norm,
            head,
            packed_tail,
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(4 + 4 + self.head.len() * 4 + self.packed_tail.len());
        bytes.extend_from_slice(&self.norm.to_le_bytes());
        bytes.extend_from_slice(&(self.head.len() as u32).to_le_bytes());
        for &h in &self.head {
            bytes.extend_from_slice(&h.to_le_bytes());
        }
        bytes.extend_from_slice(&self.packed_tail);
        bytes
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn from_bytes(bytes: &[u8], tail_dim: usize) -> Option<Self> {
        if bytes.len() < 8 {
            return None;
        }
        let norm = f32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let head_len = u32::from_le_bytes(bytes[4..8].try_into().ok()?) as usize;

        let expected_head_bytes = head_len * 4;
        let expected_packed_len = tail_dim.div_ceil(2);

        if bytes.len() < 8 + expected_head_bytes + expected_packed_len {
            return None;
        }

        let mut offset = 8;
        let mut head = Vec::with_capacity(head_len);
        for _ in 0..head_len {
            head.push(f32::from_le_bytes(
                bytes[offset..offset + 4].try_into().ok()?,
            ));
            offset += 4;
        }

        let packed_tail = bytes[offset..offset + expected_packed_len].to_vec();

        Some(Self {
            norm,
            head,
            packed_tail,
        })
    }

    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn reconstruct(&self, tail_dim: usize) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.head.len() + tail_dim);
        for &h in &self.head {
            out.push(f64::from(h));
        }

        if tail_dim == 0 || self.norm < 1e-9 {
            out.resize(self.head.len() + tail_dim, 0.0);
            return out;
        }

        let scale = f64::from(self.norm) / (tail_dim as f64).sqrt();

        for i in 0..tail_dim {
            let pair_idx = i / 2;
            let is_second = i % 2 == 1;

            if pair_idx >= self.packed_tail.len() {
                out.push(0.0);
                continue;
            }

            let packed_byte = self.packed_tail[pair_idx];
            let bin = if is_second {
                packed_byte & 0x0F
            } else {
                packed_byte >> 4
            } as usize;

            let centroid = TURBOQUANT_CENTROIDS_4BIT[bin & 0x0F];
            out.push(centroid * scale);
        }

        out
    }
}
