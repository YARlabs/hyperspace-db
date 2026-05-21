#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_lossless)]
// `n` is used in nightly-simd cfg blocks but appears unused in the scalar fallback path.
#![allow(unused_variables)]

#[cfg(feature = "nightly-simd")]
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
        #[cfg(feature = "nightly-simd")]
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

        #[cfg(not(feature = "nightly-simd"))]
        {
            let mut sum_sq_diff = 0.0;
            for (u, v) in self.coords.iter().zip(other.coords.iter()) {
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
        #[cfg(feature = "nightly-simd")]
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

        #[cfg(not(feature = "nightly-simd"))]
        {
            let mut sum_sq_diff = 0.0;
            for (u, v) in self.coords.iter().zip(other.coords.iter()) {
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
        #[cfg(feature = "nightly-simd")]
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

        #[cfg(not(feature = "nightly-simd"))]
        {
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
        #[cfg(feature = "nightly-simd")]
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

        #[cfg(not(feature = "nightly-simd"))]
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
}

/// Binary vector struct (1 bit per dimension)
#[derive(Debug, Clone)]
pub struct BinaryHyperVector {
    pub bits: Vec<u8>,
    pub alpha: f32, // Stored to preserve the conformal factor (for distances)
}

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

        for i in 0..n {
            if v.coords[i] > 0.0 {
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
        let mut sum_sq_diff = 0.0;

        for i in 0..n {
            let bit = (self.bits[i / 8] >> (i % 8)) & 1;
            let val = if bit == 1 { 1.0 } else { -1.0 };
            let diff = val - query.coords[i];
            sum_sq_diff += diff * diff;
        }

        let delta = sum_sq_diff * f64::from(self.alpha) * query.alpha;
        1.0 + 2.0 * delta
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
