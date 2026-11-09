#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
// `n` is used in nightly-simd cfg blocks but appears unused in the scalar fallback path.
#![allow(unused_variables)]

#[cfg(all(feature = "nightly-simd", has_nightly))]
use std::simd::prelude::*;

/// `VectorLayout` tracks the sizes of different components and how they are sliced from RAM/Disk.
#[derive(Debug, Clone, PartialEq)]
pub struct VectorComponentLayout {
    pub name: String,
    pub metric: String,
    pub dimension: usize,
    pub weight: f32,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorLayout {
    pub dimension: usize,
    pub bytes_per_element: usize,
    pub is_quantized: bool,
    pub components: Vec<VectorComponentLayout>,
}

/// Dynamic vector struct.
#[derive(Debug, Clone)]
pub struct HyperVector {
    pub coords: Vec<f64>,
    pub alpha: f64, // Precomputed coefficient: 1 / (1 - ||x||^2)
}

/// Memory-optimized storage vector.
#[derive(Debug, Clone)]
pub struct HyperVectorF32 {
    pub coords: Vec<f32>,
    pub alpha: f32,
}

impl HyperVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let alpha_offset = bytes.len() - 8;
        let mut alpha_bytes = [0u8; 8];
        alpha_bytes.copy_from_slice(&bytes[alpha_offset..]);
        let alpha = f64::from_le_bytes(alpha_bytes);

        let coords_len = alpha_offset / 8;
        let mut coords = Vec::with_capacity(coords_len);
        for i in 0..coords_len {
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&bytes[i * 8..(i + 1) * 8]);
            coords.push(f64::from_le_bytes(buf));
        }
        Self { coords, alpha }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.coords.len() * 8 + 8);
        for c in &self.coords {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    /// Creates a new `HyperVector`, validating it is strictly inside the unit ball.
    pub fn new(coords: Vec<f64>) -> Result<Self, String> {
        let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
        if sq_norm >= 1.0 - 1e-9 {
            return Err("Vector must be strictly inside the Poincaré ball".to_string());
        }
        let alpha = 1.0 / (1.0 - sq_norm);
        Ok(Self { coords, alpha })
    }

    /// Creates a `HyperVector` without validation.
    pub fn new_unchecked(coords: Vec<f64>) -> Self {
        let sq_norm: f64 = coords.iter().map(|&x| x * x).sum();
        let alpha = 1.0 / (1.0 - sq_norm);
        Self { coords, alpha }
    }

    /// Calculates the squared Poincaré distance. Critical performance path.
    #[inline(always)]
    pub fn poincare_distance_sq(&self, other: &Self) -> f64 {
        let n = self.coords.len();
        #[cfg(all(feature = "nightly-simd", has_nightly))]
        {
            const LANES: usize = 8;
            let mut sum_sq_diff = f64x8::splat(0.0);

            for i in (0..n).step_by(LANES) {
                if i + LANES <= n {
                    let a = f64x8::from_slice(&self.coords[i..i + LANES]);
                    let b = f64x8::from_slice(&other.coords[i..i + LANES]);
                    let diff = a - b;
                    sum_sq_diff += diff * diff;
                }
            }

            let l2_sq = sum_sq_diff.reduce_sum();

            let mut tail_sq = 0.0;
            let remainder = n % LANES;
            if remainder != 0 {
                let start = n - remainder;
                for i in start..n {
                    let diff = self.coords[i] - other.coords[i];
                    tail_sq += diff * diff;
                }
            }

            let total_sq = l2_sq + tail_sq;
            let delta = total_sq * self.alpha * other.alpha;
            1.0 + 2.0 * delta
        }

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        {
            let n = self.coords.len().min(other.coords.len());
            let (chunks_a, rem_a) = self.coords[..n].as_chunks::<4>();
            let (chunks_b, rem_b) = other.coords[..n].as_chunks::<4>();
            let mut acc = [0.0; 4];
            for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
                for l in 0..4 {
                    let diff = ca[l] - cb[l];
                    acc[l] += diff * diff;
                }
            }
            let mut sum_sq_diff = acc[0] + acc[1] + acc[2] + acc[3];
            for (&u, &v) in rem_a.iter().zip(rem_b.iter()) {
                let diff = u - v;
                sum_sq_diff += diff * diff;
            }
            let delta = sum_sq_diff * self.alpha * other.alpha;
            1.0 + 2.0 * delta
        }
    }

    pub fn true_distance(&self, other: &Self) -> f64 {
        self.poincare_distance_sq(other).acosh()
    }

    #[must_use]
    pub fn to_klein(&self) -> Self {
        let n = self.coords.len();
        let mut coords_k = vec![0.0f64; n];
        let sq_norm = 1.0 - (1.0 / self.alpha);
        let denom = 1.0 + sq_norm;

        for (k, &p) in coords_k.iter_mut().zip(self.coords.iter()) {
            *k = 2.0 * p / denom;
        }
        Self::new_unchecked(coords_k)
    }

    #[must_use]
    pub fn klein_to_lorentz(&self) -> (f64, Vec<f64>) {
        let n = self.coords.len();
        let sq_norm: f64 = self.coords.iter().map(|&x| x * x).sum();
        let factor = 1.0 / (1.0 - sq_norm).max(1e-12).sqrt();
        let mut spatial = vec![0.0f64; n];
        for (s, &k) in spatial.iter_mut().zip(self.coords.iter()) {
            *s = k * factor;
        }
        (factor, spatial)
    }

    #[inline(always)]
    pub fn klein_chord_distance_sq(&self, other: &Self) -> f64 {
        let n = self.coords.len();
        #[cfg(all(feature = "nightly-simd", has_nightly))]
        {
            const LANES: usize = 8;
            let mut sum_sq_diff = f64x8::splat(0.0);

            for i in (0..n).step_by(LANES) {
                if i + LANES <= n {
                    let a = f64x8::from_slice(&self.coords[i..i + LANES]);
                    let b = f64x8::from_slice(&other.coords[i..i + LANES]);
                    let diff = a - b;
                    sum_sq_diff += diff * diff;
                }
            }

            let mut eucl_sq = sum_sq_diff.reduce_sum();

            let remainder = n % LANES;
            if remainder != 0 {
                let start = n - remainder;
                for i in start..n {
                    let diff = self.coords[i] - other.coords[i];
                    eucl_sq += diff * diff;
                }
            }
            eucl_sq
        }

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        {
            let n = self.coords.len().min(other.coords.len());
            let (chunks_a, rem_a) = self.coords[..n].as_chunks::<4>();
            let (chunks_b, rem_b) = other.coords[..n].as_chunks::<4>();
            let mut acc = [0.0; 4];
            for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
                for l in 0..4 {
                    let diff = ca[l] - cb[l];
                    acc[l] += diff * diff;
                }
            }
            let mut sum_sq_diff = acc[0] + acc[1] + acc[2] + acc[3];
            for (&u, &v) in rem_a.iter().zip(rem_b.iter()) {
                let diff = u - v;
                sum_sq_diff += diff * diff;
            }
            sum_sq_diff
        }
    }
}

impl HyperVectorF32 {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let alpha_offset = bytes.len() - 4;
        let mut alpha_bytes = [0u8; 4];
        alpha_bytes.copy_from_slice(&bytes[alpha_offset..]);
        let alpha = f32::from_le_bytes(alpha_bytes);

        let coords_len = alpha_offset / 4;
        let mut coords = Vec::with_capacity(coords_len);
        for i in 0..coords_len {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(&bytes[i * 4..(i + 1) * 4]);
            coords.push(f32::from_le_bytes(buf));
        }
        Self { coords, alpha }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.coords.len() * 4 + 4);
        for c in &self.coords {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    pub fn from_float64(v: &HyperVector) -> Self {
        let n = v.coords.len();
        let mut coords = vec![0.0f32; n];
        for (dst, src) in coords.iter_mut().zip(v.coords.iter()) {
            *dst = *src as f32;
        }
        Self {
            coords,
            alpha: v.alpha as f32,
        }
    }

    pub fn to_float64(&self) -> HyperVector {
        let n = self.coords.len();
        let mut coords = vec![0.0f64; n];
        for (dst, src) in coords.iter_mut().zip(self.coords.iter()) {
            *dst = f64::from(*src);
        }
        HyperVector {
            coords,
            alpha: f64::from(self.alpha),
        }
    }
}

/// Quantized version (i8 coordinates)
#[derive(Debug, Clone)]
pub struct QuantizedHyperVector {
    pub coords: Vec<i8>,
    pub alpha: f32, // Storing as f32 to save space
}

impl QuantizedHyperVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let alpha_offset = bytes.len() - 4;
        let mut alpha_bytes = [0u8; 4];
        alpha_bytes.copy_from_slice(&bytes[alpha_offset..]);
        let alpha = f32::from_le_bytes(alpha_bytes);

        let coords_len = alpha_offset;
        let mut coords = Vec::with_capacity(coords_len);
        for i in 0..coords_len {
            coords.push(bytes[i].cast_signed());
        }
        Self { coords, alpha }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.coords.len() + 4);
        for c in &self.coords {
            bytes.push((*c).cast_unsigned());
        }
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    pub fn from_float(v: &HyperVector, anisotropic: bool) -> Self {
        let n = v.coords.len();
        let mut coords = vec![0i8; n];

        if n > 128 || !anisotropic {
            for (dst, &src) in coords.iter_mut().zip(v.coords.iter()) {
                let val = (src * 127.0).round().clamp(-127.0, 127.0);
                *dst = val as i8;
            }
            return Self {
                coords,
                alpha: v.alpha as f32,
            };
        }

        let mut float_coords = vec![0.0f64; n];
        let mut sq_norm = 0.0;
        for src in &v.coords {
            sq_norm += src * src;
        }
        let norm = sq_norm.sqrt();
        let inv_norm = if norm > 1e-9 { 1.0 / norm } else { 0.0 };

        for i in 0..n {
            let val = (v.coords[i] * 127.0).round().clamp(-127.0, 127.0);
            coords[i] = val as i8;
            float_coords[i] = val / 127.0;
        }

        if norm > 1e-9 {
            let t_weight = 10.0;
            let mut current_dot_err_x = 0.0;
            let mut current_sq_err = 0.0;
            for j in 0..n {
                let err = float_coords[j] - v.coords[j];
                current_dot_err_x += err * v.coords[j];
                current_sq_err += err * err;
            }

            for i in 0..n {
                let original_q = coords[i];
                let original_f = float_coords[i];
                let original_err = original_f - v.coords[i];

                let base_dot_err = current_dot_err_x - original_err * v.coords[i];
                let base_sq_err = current_sq_err - original_err * original_err;

                let mut best_q = original_q;
                let mut best_f = original_f;
                let mut best_loss = f64::MAX;
                let mut best_dot_err = current_dot_err_x;
                let mut best_sq_err = current_sq_err;

                for delta in [-1i16, 0, 1] {
                    let candidate_q = (original_q as i16 + delta).clamp(-127, 127) as i8;
                    let candidate_f = f64::from(candidate_q) / 127.0;
                    let candidate_err = candidate_f - v.coords[i];

                    let dot_err_x = base_dot_err + candidate_err * v.coords[i];
                    let sq_err = base_sq_err + candidate_err * candidate_err;

                    let e_parallel_mag = dot_err_x * inv_norm;
                    let e_parallel_sq = e_parallel_mag * e_parallel_mag;
                    let e_ortho_sq = (sq_err - e_parallel_sq).max(0.0);

                    let loss = e_parallel_sq + t_weight * e_ortho_sq;

                    if loss < best_loss {
                        best_loss = loss;
                        best_q = candidate_q;
                        best_f = candidate_f;
                        best_dot_err = dot_err_x;
                        best_sq_err = sq_err;
                    }
                }

                coords[i] = best_q;
                float_coords[i] = best_f;
                current_dot_err_x = best_dot_err;
                current_sq_err = best_sq_err;
            }
        }

        Self {
            coords,
            alpha: v.alpha as f32,
        }
    }

    #[inline(always)]
    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector) -> f64 {
        let n = self.coords.len();
        #[cfg(all(feature = "nightly-simd", has_nightly))]
        {
            const LANES: usize = 8;
            const SCALE_INV: f32 = 1.0 / 127.0;
            let mut sum_sq_diff = Simd::<f32, LANES>::splat(0.0);

            for i in (0..n).step_by(LANES) {
                if i + LANES <= n {
                    let a_i8 = Simd::<i8, LANES>::from_slice(&self.coords[i..i + LANES]);
                    let a_f32: Simd<f32, LANES> = a_i8.cast();
                    let a_scaled = a_f32 * Simd::splat(SCALE_INV);

                    let b_f64 = Simd::<f64, LANES>::from_slice(&query.coords[i..i + LANES]);
                    let b_f32: Simd<f32, LANES> = b_f64.cast();

                    let diff = a_scaled - b_f32;
                    sum_sq_diff += diff * diff;
                }
            }

            let l2_sq = sum_sq_diff.reduce_sum() as f64;

            let mut tail_sq = 0.0;
            let remainder = n % LANES;
            if remainder != 0 {
                let start = n - remainder;
                for i in start..n {
                    let a_val = f32::from(self.coords[i]) * SCALE_INV;
                    let diff = a_val - (query.coords[i] as f32);
                    tail_sq += (diff * diff) as f64;
                }
            }

            let total_sq = l2_sq + tail_sq;
            let delta = total_sq * f64::from(self.alpha) * query.alpha;
            1.0 + 2.0 * delta
        }

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        {
            const SCALE_INV: f64 = 1.0 / 127.0;
            let n = self.coords.len().min(query.coords.len());
            let (chunks_a, rem_a) = self.coords[..n].as_chunks::<4>();
            let (chunks_b, rem_b) = query.coords[..n].as_chunks::<4>();
            let mut acc = [0.0; 4];
            for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
                for l in 0..4 {
                    let a_val = f64::from(ca[l]) * SCALE_INV;
                    let diff = a_val - cb[l];
                    acc[l] += diff * diff;
                }
            }
            let mut sum_sq_diff = acc[0] + acc[1] + acc[2] + acc[3];
            for (&a_i8, &b_f64) in rem_a.iter().zip(rem_b.iter()) {
                let a_val = f64::from(a_i8) * SCALE_INV;
                let diff = a_val - b_f64;
                sum_sq_diff += diff * diff;
            }

            let delta = sum_sq_diff * f64::from(self.alpha) * query.alpha;
            1.0 + 2.0 * delta
        }
    }

    pub fn from_float_lorentz(v: &HyperVector) -> Self {
        let n = v.coords.len();
        let scale = v
            .coords
            .iter()
            .map(|&x| x.abs())
            .fold(0.0_f64, f64::max)
            .max(1e-12);

        let inv_scale = 127.0 / scale;
        let mut coords = vec![0i8; n];
        for (dst, &src) in coords.iter_mut().zip(v.coords.iter()) {
            *dst = (src * inv_scale).round().clamp(-127.0, 127.0) as i8;
        }

        Self {
            coords,
            alpha: scale as f32,
        }
    }

    #[inline(always)]
    pub fn lorentz_distance_to_float(&self, query: &HyperVector) -> f64 {
        let n = self.coords.len();
        let scale = f64::from(self.alpha);
        #[cfg(all(feature = "nightly-simd", has_nightly))]
        {
            const LANES: usize = 8;
            let inv_127 = 1.0 / 127.0;
            let scale_simd = Simd::<f64, LANES>::splat(scale * inv_127);
            let mut sum_spatial = Simd::<f64, LANES>::splat(0.0);

            for i in (1..n).step_by(LANES) {
                if i + LANES <= n {
                    let a_i8 = Simd::<i8, LANES>::from_slice(&self.coords[i..i + LANES]);
                    let a_f64: Simd<f64, LANES> = a_i8.cast();
                    let a_scaled = a_f64 * scale_simd;
                    let b = Simd::<f64, LANES>::from_slice(&query.coords[i..i + LANES]);
                    sum_spatial += a_scaled * b;
                }
            }

            let mut spatial_dot = sum_spatial.reduce_sum();

            let remainder = (n - 1) % LANES;
            if remainder != 0 {
                let start = n - remainder;
                for i in start..n {
                    let a_val = (f64::from(self.coords[i]) * inv_127) * scale;
                    spatial_dot += a_val * query.coords[i];
                }
            }

            let a_time = (f64::from(self.coords[0]) * inv_127) * scale;
            let b_time = query.coords[0];
            let minkowski_dot = a_time * b_time - spatial_dot;

            let arg = minkowski_dot.max(1.0);
            arg.acosh()
        }

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        {
            let inv_127 = 1.0 / 127.0;
            let mut spatial_dot = 0.0;
            for i in 1..n {
                let a_val = (f64::from(self.coords[i]) * inv_127) * scale;
                spatial_dot += a_val * query.coords[i];
            }

            let a_time = (f64::from(self.coords[0]) * inv_127) * scale;
            let b_time = query.coords[0];

            let minkowski_dot = a_time * b_time - spatial_dot;
            minkowski_dot.max(1.0).acosh()
        }
    }

    #[inline]
    pub fn distance_bytes_poincare_sq(bytes: &[u8], query: &HyperVector) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[alpha_offset..].try_into().unwrap_or([0; 4]));
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = alpha_offset.min(query.coords.len());
        let (chunks_a, rem_a) = bytes[..n].as_chunks::<4>();
        let (chunks_b, rem_b) = query.coords[..n].as_chunks::<4>();
        let mut acc = [0.0; 4];
        for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
            for l in 0..4 {
                let a_val = f64::from(ca[l].cast_signed()) * SCALE_INV;
                let diff = a_val - cb[l];
                acc[l] += diff * diff;
            }
        }
        let mut sum_sq_diff = acc[0] + acc[1] + acc[2] + acc[3];
        for (&a_u8, &b_f64) in rem_a.iter().zip(rem_b.iter()) {
            let a_val = f64::from(a_u8.cast_signed()) * SCALE_INV;
            let diff = a_val - b_f64;
            sum_sq_diff += diff * diff;
        }
        let delta = sum_sq_diff * f64::from(alpha) * query.alpha;
        1.0 + 2.0 * delta
    }

    #[inline]
    pub fn distance_bytes_euclidean_sq(bytes: &[u8], query: &HyperVector) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        const SCALE_INV: f64 = 1.0 / 127.0;
        let n = alpha_offset.min(query.coords.len());
        let (chunks_a, rem_a) = bytes[..n].as_chunks::<4>();
        let (chunks_b, rem_b) = query.coords[..n].as_chunks::<4>();
        let mut acc = [0.0; 4];
        for (ca, cb) in chunks_a.iter().zip(chunks_b.iter()) {
            for l in 0..4 {
                let a_val = f64::from(ca[l].cast_signed()) * SCALE_INV;
                let diff = a_val - cb[l];
                acc[l] += diff * diff;
            }
        }
        let mut sum_sq_diff = acc[0] + acc[1] + acc[2] + acc[3];
        for (&a_u8, &b_f64) in rem_a.iter().zip(rem_b.iter()) {
            let a_val = f64::from(a_u8.cast_signed()) * SCALE_INV;
            let diff = a_val - b_f64;
            sum_sq_diff += diff * diff;
        }
        sum_sq_diff
    }

    #[inline]
    pub fn distance_bytes_lorentz(bytes: &[u8], query: &HyperVector) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[alpha_offset..].try_into().unwrap_or([0; 4]));
        let n = alpha_offset.min(query.coords.len());
        if n == 0 {
            return 0.0;
        }
        let scale = f64::from(alpha);
        let inv_127 = 1.0 / 127.0;
        let mut spatial_dot = 0.0;
        for i in 1..n {
            spatial_dot += f64::from(bytes[i].cast_signed()) * query.coords[i];
        }
        spatial_dot *= inv_127 * scale;
        let a_time = (f64::from(bytes[0].cast_signed()) * inv_127) * scale;
        let b_time = query.coords[0];
        let minkowski_dot = a_time * b_time - spatial_dot;
        minkowski_dot.max(1.0)
    }

    #[inline]
    pub fn distance_bytes_euclidean_sq_f32(bytes: &[u8], query_f32: &[f32]) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let n = alpha_offset.min(query_f32.len());
        f64::from(crate::simd::sq8_l2_sq(&bytes[..n], &query_f32[..n]))
    }

    #[inline]
    pub fn distance_bytes_poincare_sq_f32(
        bytes: &[u8],
        query_f32: &[f32],
        query_alpha: f64,
    ) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[alpha_offset..].try_into().unwrap_or([0; 4]));
        let n = alpha_offset.min(query_f32.len());
        let sum_sq_diff = f64::from(crate::simd::sq8_l2_sq(&bytes[..n], &query_f32[..n]));
        let delta = sum_sq_diff * f64::from(alpha) * query_alpha;
        1.0 + 2.0 * delta
    }

    #[inline]
    pub fn distance_bytes_lorentz_f32(bytes: &[u8], query_f32: &[f32]) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[alpha_offset..].try_into().unwrap_or([0; 4]));
        let n = alpha_offset.min(query_f32.len());
        if n == 0 {
            return 0.0;
        }
        let scale = f64::from(alpha);
        let spatial_dot = if n > 1 {
            f64::from(crate::simd::sq8_dot(&bytes[1..n], &query_f32[1..n])) * (scale / 127.0)
        } else {
            0.0
        };
        let a_time = (f64::from(bytes[0].cast_signed()) / 127.0) * scale;
        let b_time = f64::from(query_f32[0]);
        let minkowski_dot = a_time * b_time - spatial_dot;
        minkowski_dot.max(1.0)
    }
}

/// Binary vector struct (1 bit per dimension)
#[derive(Debug, Clone)]
pub struct BinaryHyperVector {
    pub bits: Vec<u8>,
    pub alpha: f32, // Stored to preserve the conformal factor (for distances)
}

#[allow(dead_code)]
const SIGN_LUT: [[f64; 4]; 16] = [
    [-1.0, -1.0, -1.0, -1.0], // 0000
    [1.0, -1.0, -1.0, -1.0],  // 0001
    [-1.0, 1.0, -1.0, -1.0],  // 0010
    [1.0, 1.0, -1.0, -1.0],   // 0011
    [-1.0, -1.0, 1.0, -1.0],  // 0100
    [1.0, -1.0, 1.0, -1.0],   // 0101
    [-1.0, 1.0, 1.0, -1.0],   // 0110
    [1.0, 1.0, 1.0, -1.0],    // 0111
    [-1.0, -1.0, -1.0, 1.0],  // 1000
    [1.0, -1.0, -1.0, 1.0],   // 1001
    [-1.0, 1.0, -1.0, 1.0],   // 1010
    [1.0, 1.0, -1.0, 1.0],    // 1011
    [-1.0, -1.0, 1.0, 1.0],   // 1100
    [1.0, -1.0, 1.0, 1.0],    // 1101
    [-1.0, 1.0, 1.0, 1.0],    // 1110
    [1.0, 1.0, 1.0, 1.0],     // 1111
];

impl BinaryHyperVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let alpha_offset = bytes.len() - 4;
        let mut alpha_bytes = [0u8; 4];
        alpha_bytes.copy_from_slice(&bytes[alpha_offset..]);
        let alpha = f32::from_le_bytes(alpha_bytes);

        let mut bits = Vec::with_capacity(alpha_offset);
        bits.extend_from_slice(&bytes[..alpha_offset]);
        Self { bits, alpha }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.bits.len() + 4);
        bytes.extend_from_slice(&self.bits);
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    pub fn from_float(v: &HyperVector) -> Self {
        let n = v.coords.len();
        let bytes_len = n.div_ceil(8);
        let mut bits = vec![0u8; bytes_len];

        let mut rotated_f32: Vec<f32> = v.coords.iter().map(|&x| x as f32).collect();
        crate::turbo_rot::rotate_vector_in_place(&mut rotated_f32);

        for i in 0..n {
            if rotated_f32[i] > 0.0 {
                bits[i / 8] |= 1 << (i % 8);
            }
        }

        Self {
            bits,
            alpha: v.alpha as f32,
        }
    }

    #[inline(always)]
    pub fn poincare_distance_sq_to_float(&self, query: &HyperVector) -> f64 {
        let n = query.coords.len();
        let mut i = 0;

        #[cfg(all(feature = "nightly-simd", has_nightly))]
        let mut sum_sq_diff = {
            use std::simd::f64x4;
            use std::simd::num::SimdFloat;
            let mut sum_vec = f64x4::splat(0.0);

            while i + 8 <= n {
                let byte = self.bits[i / 8];
                let val1 = f64x4::from_array(SIGN_LUT[(byte & 0x0F) as usize]);
                let val2 = f64x4::from_array(SIGN_LUT[(byte >> 4) as usize]);

                let q1 = f64x4::from_slice(&query.coords[i..i + 4]);
                let q2 = f64x4::from_slice(&query.coords[i + 4..i + 8]);

                let diff1 = val1 - q1;
                let diff2 = val2 - q2;

                sum_vec += diff1 * diff1 + diff2 * diff2;
                i += 8;
            }
            sum_vec.reduce_sum()
        };

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        let mut sum_sq_diff = 0.0;

        while i < n {
            let bit = (self.bits[i / 8] >> (i % 8)) & 1;
            let val = (bit as f64) * 2.0 - 1.0;
            let diff = val - query.coords[i];
            sum_sq_diff += diff * diff;
            i += 1;
        }

        let delta = sum_sq_diff * f64::from(self.alpha) * query.alpha;
        1.0 + 2.0 * delta
    }

    #[inline(always)]
    pub fn adc_distance_to_float(&self, query: &HyperVector) -> f64 {
        let n = query.coords.len();
        if n == 0 {
            return 0.0;
        }
        let scale = 1.0 / (n as f64).sqrt();
        let mut i = 0;
        let n_full = (n / 8) * 8;

        #[cfg(all(feature = "nightly-simd", has_nightly))]
        let mut dot = {
            use std::simd::f64x4;
            use std::simd::num::SimdFloat;

            let mut sum_vec = f64x4::splat(0.0);
            while i < n_full {
                let byte = self.bits[i / 8];
                let val1 = f64x4::from_array(SIGN_LUT[(byte & 0x0F) as usize]);
                let val2 = f64x4::from_array(SIGN_LUT[(byte >> 4) as usize]);

                let q1 = f64x4::from_slice(&query.coords[i..i + 4]);
                let q2 = f64x4::from_slice(&query.coords[i + 4..i + 8]);

                sum_vec += val1 * q1 + val2 * q2;
                i += 8;
            }
            sum_vec.reduce_sum()
        };

        #[cfg(not(all(feature = "nightly-simd", has_nightly)))]
        let mut dot = {
            let mut d = 0.0;
            while i < n_full {
                let byte = self.bits[i / 8];
                let s1 = SIGN_LUT[(byte & 0x0F) as usize];
                let s2 = SIGN_LUT[(byte >> 4) as usize];
                d += s1[0] * query.coords[i]
                    + s1[1] * query.coords[i + 1]
                    + s1[2] * query.coords[i + 2]
                    + s1[3] * query.coords[i + 3]
                    + s2[0] * query.coords[i + 4]
                    + s2[1] * query.coords[i + 5]
                    + s2[2] * query.coords[i + 6]
                    + s2[3] * query.coords[i + 7];
                i += 8;
            }
            d
        };

        while i < n {
            let bit = (self.bits[i / 8] >> (i % 8)) & 1;
            let val = if bit == 1 { 1.0 } else { -1.0 };
            dot += val * query.coords[i];
            i += 1;
        }

        let cos_dist = 2.0 * (1.0 - (dot * scale).clamp(-1.0, 1.0));
        cos_dist.max(0.0)
    }

    #[inline(always)]
    pub fn poincare_distance_sq_bytes(bytes: &[u8], query: &HyperVector) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let alpha = f32::from_le_bytes(bytes[alpha_offset..].try_into().unwrap_or([0; 4]));
        let bits = &bytes[..alpha_offset];
        let n = query.coords.len();
        let mut sum_sq_diff = 0.0;
        let mut i = 0;
        while i < n && i / 8 < bits.len() {
            let bit = (bits[i / 8] >> (i % 8)) & 1;
            let val = (bit as f64) * 2.0 - 1.0;
            let diff = val - query.coords[i];
            sum_sq_diff += diff * diff;
            i += 1;
        }
        let delta = sum_sq_diff * f64::from(alpha) * query.alpha;
        1.0 + 2.0 * delta
    }

    #[inline(always)]
    pub fn adc_distance_bytes(bytes: &[u8], query: &HyperVector) -> f64 {
        if bytes.len() < 4 {
            return f64::MAX;
        }
        let alpha_offset = bytes.len() - 4;
        let bits = &bytes[..alpha_offset];
        let n = query.coords.len();
        if n == 0 {
            return 0.0;
        }
        let scale = 1.0 / (n as f64).sqrt();
        let mut dot = 0.0;
        let mut i = 0;
        while i < n && i / 8 < bits.len() {
            let bit = (bits[i / 8] >> (i % 8)) & 1;
            let val = (bit as f64) * 2.0 - 1.0;
            dot += val * query.coords[i];
            i += 1;
        }
        let cos_dist = 2.0 * (1.0 - (dot * scale).clamp(-1.0, 1.0));
        cos_dist.max(0.0)
    }
}

/// Ternary 1.58-bit quantized vector {-1, 0, +1}.
/// Optimal BitNet / Lloyd-Max quantization with adaptive threshold tau.
/// Memory layout:
///   - Pure vector (head_dim == 0):
///     [nz_bytes (D/8 ceil)] [sign_bytes (D/8 ceil)] [scale: f32 (4B)] [alpha: f32 (4B)]
///     Total bytes = 2 * D.div_ceil(8) + 8
///   - Hybrid vector (e.g. head_dim == 33 Lorentz, tail_dim == 768 Ternary):
///     [head_f32 (head_dim * 4B)] [nz_bytes (tail_dim/8 ceil)] [sign_bytes (tail_dim/8 ceil)] [scale: f32 (4B)] [alpha: f32 (4B)]
///     Total bytes = head_dim * 4 + 2 * tail_dim.div_ceil(8) + 8
#[derive(Debug, Clone, PartialEq)]
pub struct TernaryHyperVector {
    pub head: Vec<f32>,
    pub nz: Vec<u8>,
    pub sign: Vec<u8>,
    pub scale: f32,
    pub alpha: f32,
}

impl TernaryHyperVector {
    pub const DEFAULT_TAU_FACTOR: f32 = 0.67;

    pub fn from_float(v: &HyperVector, head_dim: usize) -> Self {
        Self::from_float_with_tau(v, head_dim, Self::DEFAULT_TAU_FACTOR)
    }

    pub fn from_float_with_tau(v: &HyperVector, head_dim: usize, tau_factor: f32) -> Self {
        let n = v.coords.len();
        let head_dim = head_dim.min(n);
        let mut head = Vec::with_capacity(head_dim);
        for i in 0..head_dim {
            head.push(v.coords[i] as f32);
        }

        let tail_dim = n - head_dim;
        let num_bytes = tail_dim.div_ceil(8);
        let mut nz = vec![0u8; num_bytes];
        let mut sign = vec![0u8; num_bytes];

        if tail_dim == 0 {
            return Self {
                head,
                nz,
                sign,
                scale: 1.0,
                alpha: v.alpha as f32,
            };
        }

        let mut tail_f32: Vec<f32> = v.coords[head_dim..].iter().map(|&x| x as f32).collect();
        crate::turbo_rot::rotate_vector_in_place(&mut tail_f32);

        let mean_abs: f32 = tail_f32.iter().map(|x| x.abs()).sum::<f32>() / (tail_dim as f32);
        let tau = mean_abs * tau_factor;

        let mut dot_orig_ternary = 0.0f32;
        let mut ternary_norm_sq = 0.0f32;

        for (i, &x) in tail_f32.iter().enumerate() {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if x > tau {
                nz[byte_idx] |= 1 << bit_idx;
                sign[byte_idx] |= 1 << bit_idx;
                dot_orig_ternary += x;
                ternary_norm_sq += 1.0;
            } else if x < -tau {
                nz[byte_idx] |= 1 << bit_idx;
                dot_orig_ternary -= x;
                ternary_norm_sq += 1.0;
            }
        }

        let scale = if ternary_norm_sq > 0.0 {
            dot_orig_ternary / ternary_norm_sq
        } else {
            1.0
        };

        Self {
            head,
            nz,
            sign,
            scale,
            alpha: v.alpha as f32,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let total_size = self.head.len() * 4 + self.nz.len() + self.sign.len() + 8;
        let mut bytes = Vec::with_capacity(total_size);
        for &h in &self.head {
            bytes.extend_from_slice(&h.to_le_bytes());
        }
        bytes.extend_from_slice(&self.nz);
        bytes.extend_from_slice(&self.sign);
        bytes.extend_from_slice(&self.scale.to_le_bytes());
        bytes.extend_from_slice(&self.alpha.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8], dim: usize, head_dim: usize) -> Self {
        let head_bytes_len = head_dim * 4;
        let mut head = Vec::with_capacity(head_dim);
        for i in 0..head_dim {
            let offset = i * 4;
            if offset + 4 <= bytes.len() {
                let val =
                    f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
                head.push(val);
            } else {
                head.push(0.0);
            }
        }

        let tail_dim = dim.saturating_sub(head_dim);
        let num_bytes = tail_dim.div_ceil(8);
        let nz_start = head_bytes_len;
        let nz_end = (nz_start + num_bytes).min(bytes.len());
        let sign_start = nz_end;
        let sign_end = (sign_start + num_bytes).min(bytes.len());

        let nz = if nz_start < bytes.len() {
            bytes[nz_start..nz_end].to_vec()
        } else {
            vec![0u8; num_bytes]
        };
        let sign = if sign_start < bytes.len() {
            bytes[sign_start..sign_end].to_vec()
        } else {
            vec![0u8; num_bytes]
        };

        let scale_offset = head_bytes_len + 2 * num_bytes;
        let scale = if scale_offset + 4 <= bytes.len() {
            f32::from_le_bytes(
                bytes[scale_offset..scale_offset + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            )
        } else {
            1.0
        };
        let alpha = if scale_offset + 8 <= bytes.len() {
            f32::from_le_bytes(
                bytes[scale_offset + 4..scale_offset + 8]
                    .try_into()
                    .unwrap_or([0; 4]),
            )
        } else {
            0.0
        };

        Self {
            head,
            nz,
            sign,
            scale,
            alpha,
        }
    }

    pub fn reconstruct(&self, dim: usize) -> Vec<f64> {
        let mut coords = vec![0.0; dim];
        let head_len = self.head.len().min(dim);
        for i in 0..head_len {
            coords[i] = f64::from(self.head[i]);
        }
        let tail_len = dim.saturating_sub(head_len);
        let scale = f64::from(self.scale);
        for i in 0..tail_len {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if byte_idx < self.nz.len() && (self.nz[byte_idx] & (1 << bit_idx)) != 0 {
                let is_pos =
                    byte_idx < self.sign.len() && (self.sign[byte_idx] & (1 << bit_idx)) != 0;
                coords[head_len + i] = if is_pos { scale } else { -scale };
            }
        }
        coords
    }

    pub fn reconstruct_unrotated(&self, dim: usize) -> Vec<f64> {
        let mut coords = self.reconstruct(dim);
        let head_len = self.head.len().min(dim);
        let tail_len = dim.saturating_sub(head_len);
        if tail_len > 0 {
            let mut tail_f32: Vec<f32> = coords[head_len..].iter().map(|&x| x as f32).collect();
            crate::turbo_rot::unrotate_vector_in_place(&mut tail_f32);
            for i in 0..tail_len {
                coords[head_len + i] = f64::from(tail_f32[i]);
            }
        }
        coords
    }

    #[inline(always)]
    pub fn asymmetric_dot_bytes(
        bytes: &[u8],
        query_coords: &[f64],
        dim: usize,
        head_dim: usize,
    ) -> f64 {
        let head_bytes_len = head_dim * 4;
        let tail_dim = dim.saturating_sub(head_dim);
        let num_bytes = tail_dim.div_ceil(8);
        let sign_start = head_bytes_len + num_bytes;
        let scale_offset = head_bytes_len + 2 * num_bytes;

        if scale_offset + 4 > bytes.len() {
            return 0.0;
        }

        let scale = f32::from_le_bytes(
            bytes[scale_offset..scale_offset + 4]
                .try_into()
                .unwrap_or([0; 4]),
        );
        let nz_bytes = &bytes[head_bytes_len..head_bytes_len + num_bytes];
        let sign_bytes = &bytes[sign_start..sign_start + num_bytes];
        let q_tail = &query_coords[head_dim..query_coords.len().min(dim)];

        let mut dot = 0.0;
        let n = q_tail.len();
        for b_idx in 0..num_bytes {
            let nz = nz_bytes[b_idx];
            if nz == 0 {
                continue;
            }
            let sign = sign_bytes[b_idx];
            let base = b_idx * 8;
            let limit = (n.saturating_sub(base)).min(8);
            for bit in 0..limit {
                if (nz & (1 << bit)) != 0 {
                    let q = q_tail[base + bit];
                    if (sign & (1 << bit)) != 0 {
                        dot += q;
                    } else {
                        dot -= q;
                    }
                }
            }
        }
        dot * f64::from(scale)
    }

    #[inline(always)]
    pub fn distance_bytes_l2_sq(
        bytes: &[u8],
        query: &HyperVector,
        dim: usize,
        head_dim: usize,
    ) -> f64 {
        let head_bytes_len = head_dim * 4;
        let tail_dim = dim.saturating_sub(head_dim);
        let num_bytes = tail_dim.div_ceil(8);
        let scale_offset = head_bytes_len + 2 * num_bytes;

        if scale_offset + 4 > bytes.len() {
            return f64::MAX;
        }

        let mut head_dist_sq = 0.0;
        for i in 0..head_dim {
            let offset = i * 4;
            let h = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            let diff = f64::from(h) - query.coords[i];
            head_dist_sq += diff * diff;
        }

        let scale = f32::from_le_bytes(
            bytes[scale_offset..scale_offset + 4]
                .try_into()
                .unwrap_or([0; 4]),
        );
        let nz_bytes = &bytes[head_bytes_len..head_bytes_len + num_bytes];
        let sign_bytes = &bytes[head_bytes_len + num_bytes..scale_offset];
        let q_tail = &query.coords[head_dim..query.coords.len().min(dim)];

        let mut dot = 0.0;
        let mut nz_count = 0u32;
        let mut q_norm_sq = 0.0;
        let n = q_tail.len();

        for b_idx in 0..num_bytes {
            let nz = nz_bytes[b_idx];
            nz_count += nz.count_ones();
            if nz == 0 {
                let base = b_idx * 8;
                let limit = (n.saturating_sub(base)).min(8);
                for bit in 0..limit {
                    let q = q_tail[base + bit];
                    q_norm_sq += q * q;
                }
                continue;
            }
            let sign = sign_bytes[b_idx];
            let base = b_idx * 8;
            let limit = (n.saturating_sub(base)).min(8);
            for bit in 0..limit {
                let q = q_tail[base + bit];
                q_norm_sq += q * q;
                if (nz & (1 << bit)) != 0 {
                    if (sign & (1 << bit)) != 0 {
                        dot += q;
                    } else {
                        dot -= q;
                    }
                }
            }
        }

        let scale_f64 = f64::from(scale);
        let v_norm_sq = f64::from(nz_count) * scale_f64 * scale_f64;
        let tail_l2_sq = (v_norm_sq + q_norm_sq - 2.0 * dot * scale_f64).max(0.0);

        head_dist_sq + tail_l2_sq
    }

    #[inline(always)]
    pub fn distance_bytes_cosine(
        bytes: &[u8],
        query: &HyperVector,
        dim: usize,
        head_dim: usize,
    ) -> f64 {
        let head_bytes_len = head_dim * 4;
        let tail_dim = dim.saturating_sub(head_dim);
        let num_bytes = tail_dim.div_ceil(8);
        let scale_offset = head_bytes_len + 2 * num_bytes;

        if scale_offset + 4 > bytes.len() {
            return 2.0;
        }

        let mut dot = 0.0;
        for i in 0..head_dim {
            let offset = i * 4;
            let h = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            dot += f64::from(h) * query.coords[i];
        }

        let scale = f32::from_le_bytes(
            bytes[scale_offset..scale_offset + 4]
                .try_into()
                .unwrap_or([0; 4]),
        );
        let nz_bytes = &bytes[head_bytes_len..head_bytes_len + num_bytes];
        let sign_bytes = &bytes[head_bytes_len + num_bytes..scale_offset];
        let q_tail = &query.coords[head_dim..query.coords.len().min(dim)];

        let mut tail_dot = 0.0;
        let mut nz_count = 0u32;
        let n = q_tail.len();

        for b_idx in 0..num_bytes {
            let nz = nz_bytes[b_idx];
            nz_count += nz.count_ones();
            if nz == 0 {
                continue;
            }
            let sign = sign_bytes[b_idx];
            let base = b_idx * 8;
            let limit = (n.saturating_sub(base)).min(8);
            for bit in 0..limit {
                if (nz & (1 << bit)) != 0 {
                    let q = q_tail[base + bit];
                    if (sign & (1 << bit)) != 0 {
                        tail_dot += q;
                    } else {
                        tail_dot -= q;
                    }
                }
            }
        }

        let scale_f64 = f64::from(scale);
        dot += tail_dot * scale_f64;

        let v_norm_sq = f64::from(nz_count) * scale_f64 * scale_f64;
        let mut q_norm_sq = 0.0;
        for &q in &query.coords[..dim.min(query.coords.len())] {
            q_norm_sq += q * q;
        }

        let denom = (v_norm_sq * q_norm_sq).sqrt();
        if denom < 1e-9 {
            return 2.0;
        }

        let cos_sim = (dot / denom).clamp(-1.0, 1.0);
        (1.0 - cos_sim).max(0.0)
    }

    #[inline(always)]
    pub fn poincare_distance_sq_bytes(bytes: &[u8], query: &HyperVector, dim: usize) -> f64 {
        let num_bytes = dim.div_ceil(8);
        let scale_offset = 2 * num_bytes;
        if scale_offset + 8 > bytes.len() {
            return f64::MAX;
        }
        let alpha = f32::from_le_bytes(
            bytes[scale_offset + 4..scale_offset + 8]
                .try_into()
                .unwrap_or([0; 4]),
        );
        let l2_sq = Self::distance_bytes_l2_sq(bytes, query, dim, 0);
        let delta = l2_sq * f64::from(alpha) * query.alpha;
        1.0 + 2.0 * delta
    }

    #[inline(always)]
    pub fn lorentz_distance_bytes(
        bytes: &[u8],
        query: &HyperVector,
        dim: usize,
        head_dim: usize,
    ) -> f64 {
        if head_dim > 0 {
            // Lorentz head is stored in f32
            let mut h_dot = 0.0;
            let h0 = f32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4]));
            let time_term = f64::from(h0) * query.coords[0];
            for i in 1..head_dim {
                let offset = i * 4;
                let h = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
                h_dot += f64::from(h) * query.coords[i];
            }
            let tail_dot = Self::asymmetric_dot_bytes(bytes, &query.coords, dim, head_dim);
            let minkowski_dot = -time_term + h_dot + tail_dot;
            (-minkowski_dot).max(1.0)
        } else {
            let tail_dim = dim;
            let num_bytes = tail_dim.div_ceil(8);
            let scale_offset = 2 * num_bytes;
            if scale_offset + 4 > bytes.len() {
                return f64::MAX;
            }
            let scale = f64::from(f32::from_le_bytes(
                bytes[scale_offset..scale_offset + 4]
                    .try_into()
                    .unwrap_or([0; 4]),
            ));
            let nz_bytes = &bytes[..num_bytes];
            let sign_bytes = &bytes[num_bytes..2 * num_bytes];

            let mut minkowski_dot = 0.0;
            let n = query.coords.len().min(dim);
            for b_idx in 0..num_bytes {
                let nz = nz_bytes[b_idx];
                if nz == 0 {
                    continue;
                }
                let sign = sign_bytes[b_idx];
                let base = b_idx * 8;
                let limit = (n.saturating_sub(base)).min(8);
                for bit in 0..limit {
                    let idx = base + bit;
                    if (nz & (1 << bit)) != 0 {
                        let is_pos = (sign & (1 << bit)) != 0;
                        let val = if is_pos { scale } else { -scale };
                        if idx == 0 {
                            minkowski_dot -= val * query.coords[0];
                        } else {
                            minkowski_dot += val * query.coords[idx];
                        }
                    }
                }
            }
            (-minkowski_dot).max(1.0)
        }
    }

    #[inline(always)]
    pub fn hybrid_distance_bytes(bytes: &[u8], query: &HyperVector, active_dim: usize) -> f64 {
        let d_lor = {
            let h0 = f32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4]));
            let mut minkowski = -f64::from(h0) * query.coords[0];
            for i in 1..33 {
                let offset = i * 4;
                let h = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
                minkowski += f64::from(h) * query.coords[i];
            }
            (-minkowski).max(1.0).acosh()
        };

        let d_euc = Self::distance_bytes_l2_sq(bytes, query, active_dim, 33);
        d_lor + d_euc
    }
}

#[derive(Debug, Clone)]
pub enum ZonalVector {
    Core(Vec<i8>),
    Boundary(Vec<f64>),
}

impl ZonalVector {
    pub fn new_zonal(arr: &[f64]) -> Self {
        Self::Boundary(arr.to_vec())
    }
}
