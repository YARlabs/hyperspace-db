//! RaBitQ: 1-bit quantization with an unbiased inner-product estimator.
//!
//! Gao & Long, *RaBitQ: Quantizing High-Dimensional Vectors with a Theoretical Error
//! Bound for Approximate Nearest Neighbor Search* (SIGMOD 2024).
//!
//! All vectors are taken relative to a centroid `c` ([`RaBitQ::fit_centroid`]); embeddings
//! share a large common component and the estimator is only accurate on residuals.
//! Encoding of the residual `x = v − c` (dimension `D`):
//! 1. `o = x / ‖x‖`, rotated by the orthonormal randomized FWHT `P` already used by
//!    Turbo/OPQ (`turbo_rot`): `o' = P·o`.
//! 2. Store `b_i = [o'_i ≥ 0]` (the code `x̄ = (2b − 1)/√D`), `‖x‖`, and the correction
//!    factor `⟨o', x̄⟩ = Σ|o'_i| / √D`.
//!
//! Querying `q`: `q' = P·q/‖q‖` is quantized to 4-bit bit-planes so that
//! `Σ_{b_i=1} q'_i` reduces to `lo·popcnt(b) + Δ·Σ_j 2^j·popcnt(b ∧ plane_j)` over `u64`
//! words. Then `⟨o, q/‖q‖⟩ ≈ ⟨x̄, q'⟩ / ⟨o', x̄⟩` (unbiased) and
//! `‖x − q‖² = ‖x‖² + ‖q‖² − 2‖x‖‖q‖·⟨o, q/‖q‖⟩`.
//!
//! Layout: `[bits: ⌈D/64⌉ × u64 LE][‖x‖: f32][⟨o', x̄⟩: f32]`.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::turbo_rot;

/// Number of 4-bit query bit-planes.
const QUERY_BITS: usize = 4;

/// Bytes per encoded vector of dimension `dim`.
#[must_use]
pub fn code_size(dim: usize) -> usize {
    dim.div_ceil(64) * 8 + 8
}

#[inline]
fn norm(v: &[f32]) -> f32 {
    v.iter().map(|x| x * x).sum::<f32>().sqrt()
}

/// Encodes a residual `x` into a RaBitQ code (see module docs for the layout).
fn encode_residual(x: &[f32]) -> Vec<u8> {
    let dim = x.len();
    let words = dim.div_ceil(64);
    let mut out = vec![0u8; code_size(dim)];
    let n = norm(x);
    let mut ip = 1.0f32;
    if n > 0.0 && dim > 0 {
        let mut o: Vec<f32> = x.iter().map(|v| v / n).collect();
        turbo_rot::rotate_vector_in_place(&mut o);
        let mut bits = vec![0u64; words];
        let mut abs_sum = 0.0f32;
        for (i, &v) in o.iter().enumerate() {
            if v >= 0.0 {
                bits[i / 64] |= 1u64 << (i % 64);
            }
            abs_sum += v.abs();
        }
        ip = (abs_sum / (dim as f32).sqrt()).max(1e-6);
        for (w, chunk) in bits.iter().zip(out.as_chunks_mut::<8>().0.iter_mut()) {
            chunk.copy_from_slice(&w.to_le_bytes());
        }
    }
    let tail = words * 8;
    out[tail..tail + 4].copy_from_slice(&n.to_le_bytes());
    out[tail + 4..tail + 8].copy_from_slice(&ip.to_le_bytes());
    out
}

/// Reconstructs the residual estimate `‖x‖ · P⁻¹·x̄` into `out` (`out.len() == dim`).
fn reconstruct_residual_into(code: &[u8], dim: usize, out: &mut Vec<f32>) {
    out.clear();
    let words = dim.div_ceil(64);
    if code.len() < code_size(dim) {
        out.resize(dim, 0.0);
        return;
    }
    let n = f32::from_le_bytes(code[words * 8..words * 8 + 4].try_into().unwrap_or([0; 4]));
    let s = n / (dim.max(1) as f32).sqrt();
    out.extend((0..dim).map(|i| {
        let w = u64::from_le_bytes(
            code[(i / 64) * 8..(i / 64) * 8 + 8]
                .try_into()
                .unwrap_or([0; 8]),
        );
        if (w >> (i % 64)) & 1 == 1 {
            s
        } else {
            -s
        }
    }));
    turbo_rot::unrotate_vector_in_place(out);
}

/// RaBitQ quantizer: the shared centroid plus encode / query / reconstruct helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct RaBitQ {
    pub dim: usize,
    pub centroid: Vec<f32>,
}

impl RaBitQ {
    /// Zero-centroid quantizer (call [`Self::fit_centroid`] once data is available).
    #[must_use]
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            centroid: vec![0.0; dim],
        }
    }

    /// Sets the centroid to the mean of `samples`.
    pub fn fit_centroid(&mut self, samples: &[&[f32]]) {
        if samples.is_empty() {
            return;
        }
        let mut c = vec![0.0f64; self.dim];
        for s in samples {
            for (acc, &v) in c.iter_mut().zip(s.iter()) {
                *acc += f64::from(v);
            }
        }
        let inv = 1.0 / samples.len() as f64;
        self.centroid = c.into_iter().map(|v| (v * inv) as f32).collect();
    }

    fn residual(&self, v: &[f32]) -> Vec<f32> {
        (0..self.dim)
            .map(|i| {
                v.get(i).copied().unwrap_or(0.0) - self.centroid.get(i).copied().unwrap_or(0.0)
            })
            .collect()
    }

    /// Encodes `v` (see module docs for the layout).
    #[must_use]
    pub fn encode(&self, v: &[f32]) -> Vec<u8> {
        encode_residual(&self.residual(v))
    }

    /// Per-query state for distance estimation against codes.
    #[must_use]
    pub fn query(&self, q: &[f32]) -> RaBitQQuery {
        RaBitQQuery::from_residual(&self.residual(q))
    }

    /// Reconstructs the best estimate of the original vector into `out`.
    pub fn reconstruct_into(&self, code: &[u8], out: &mut Vec<f32>) {
        reconstruct_residual_into(code, self.dim, out);
        for (o, c) in out.iter_mut().zip(&self.centroid) {
            *o += c;
        }
    }

    /// Serializes the centroid (`RBQC | dim | f32 × dim`).
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(8 + self.dim * 4);
        buf.extend_from_slice(b"RBQC");
        buf.extend_from_slice(&(self.dim as u32).to_le_bytes());
        for c in &self.centroid {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        std::fs::write(path, buf)
    }

    /// Loads a quantizer written by [`Self::save_to_file`].
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad =
            || std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid RaBitQ centroid");
        if bytes.len() < 8 || &bytes[..4] != b"RBQC" {
            return Err(bad());
        }
        let dim = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        if bytes.len() != 8 + dim * 4 {
            return Err(bad());
        }
        let centroid = bytes[8..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        Ok(Self { dim, centroid })
    }
}

/// Per-query RaBitQ state (rotated, 4-bit bit-plane quantized query).
#[derive(Debug, Clone)]
pub struct RaBitQQuery {
    dim: usize,
    words: usize,
    /// `planes[j * words + w]`: bit `j` of the 4-bit query codes.
    planes: Vec<u64>,
    lo: f32,
    delta: f32,
    /// Σ of the dequantized rotated query (consistent with the plane estimate).
    sum_q: f32,
    q_norm: f32,
    inv_sqrt_dim: f32,
}

impl RaBitQQuery {
    /// Builds the query state for a residual `q − c`.
    fn from_residual(q: &[f32]) -> Self {
        let dim = q.len();
        let words = dim.div_ceil(64);
        let q_norm = norm(q);
        let mut r: Vec<f32> = if q_norm > 0.0 {
            q.iter().map(|v| v / q_norm).collect()
        } else {
            vec![0.0; dim]
        };
        turbo_rot::rotate_vector_in_place(&mut r);
        let lo = r.iter().copied().fold(f32::INFINITY, f32::min);
        let hi = r.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let (lo, hi) = if lo.is_finite() { (lo, hi) } else { (0.0, 0.0) };
        let levels = ((1u32 << QUERY_BITS) - 1) as f32;
        let delta = if hi > lo { (hi - lo) / levels } else { 0.0 };
        let mut planes = vec![0u64; QUERY_BITS * words];
        let mut code_sum = 0u64;
        for (i, &v) in r.iter().enumerate() {
            let qc = if delta > 0.0 {
                ((v - lo) / delta).round().clamp(0.0, levels) as u64
            } else {
                0
            };
            code_sum += qc;
            for j in 0..QUERY_BITS {
                if (qc >> j) & 1 == 1 {
                    planes[j * words + i / 64] |= 1u64 << (i % 64);
                }
            }
        }
        let sum_q = dim as f32 * lo + delta * code_sum as f32;
        Self {
            dim,
            words,
            planes,
            lo,
            delta,
            sum_q,
            q_norm,
            inv_sqrt_dim: 1.0 / (dim.max(1) as f32).sqrt(),
        }
    }

    /// Estimated squared L2 distance between the query and an encoded vector.
    #[must_use]
    #[inline]
    pub fn distance_sq(&self, code: &[u8]) -> f32 {
        let words = self.words;
        if code.len() < words * 8 + 8 {
            return f32::MAX;
        }
        let mut pop_b = 0u32;
        let mut weighted = 0u32;
        for w in 0..words {
            let b = u64::from_le_bytes(code[w * 8..w * 8 + 8].try_into().unwrap_or([0; 8]));
            pop_b += b.count_ones();
            for j in 0..QUERY_BITS {
                weighted += (b & self.planes[j * words + w]).count_ones() << j;
            }
        }
        let tail = words * 8;
        let x_norm = f32::from_le_bytes(code[tail..tail + 4].try_into().unwrap_or([0; 4]));
        let ip_ox = f32::from_le_bytes(code[tail + 4..tail + 8].try_into().unwrap_or([0; 4]));
        let masked_sum = self.lo * pop_b as f32 + self.delta * weighted as f32;
        let xbar_dot_q = (2.0 * masked_sum - self.sum_q) * self.inv_sqrt_dim;
        let cos_est = (xbar_dot_q / ip_ox.max(1e-6)).clamp(-1.0, 1.0);
        (x_norm * x_norm + self.q_norm * self.q_norm - 2.0 * x_norm * self.q_norm * cos_est)
            .max(0.0)
    }

    #[must_use]
    pub fn dim(&self) -> usize {
        self.dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-Gaussian vectors with a large common offset (like real
    /// embeddings), so the centroid path is exercised.
    fn vecs(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 11) as f32 / (1u64 << 53) as f32
        };
        (0..n)
            .map(|_| {
                (0..dim)
                    .map(|d| {
                        let g = (-2.0 * next().max(1e-9).ln()).sqrt()
                            * (std::f32::consts::TAU * next()).cos();
                        g + 3.0 + (d % 7) as f32 * 0.5
                    })
                    .collect()
            })
            .collect()
    }

    fn l2(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum()
    }

    #[test]
    fn code_size_and_layout() {
        assert_eq!(code_size(128), 16 + 8);
        assert_eq!(code_size(100), 16 + 8);
        let c = RaBitQ::new(100).encode(&[0.0; 100]);
        assert_eq!(c.len(), code_size(100));
    }

    #[test]
    fn estimator_tracks_true_distance_and_ranking() {
        let dim = 256;
        let data = vecs(400, dim, 1);
        let mut rq_model = RaBitQ::new(dim);
        let refs: Vec<&[f32]> = data.iter().map(Vec::as_slice).collect();
        rq_model.fit_centroid(&refs);
        let codes: Vec<Vec<u8>> = data.iter().map(|v| rq_model.encode(v)).collect();
        let queries = vecs(20, dim, 2);
        let mut rel_err_sum = 0.0f32;
        let mut hits = 0usize;
        for q in &queries {
            let rq = rq_model.query(q);
            let mut exact: Vec<(usize, f32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (i, l2(v, q)))
                .collect();
            let mut est: Vec<(usize, f32)> = codes
                .iter()
                .enumerate()
                .map(|(i, c)| (i, rq.distance_sq(c)))
                .collect();
            for ((_, e), (_, a)) in exact.iter().zip(&est) {
                rel_err_sum += (e - a).abs() / e.max(1e-3);
            }
            exact.sort_by(|a, b| a.1.total_cmp(&b.1));
            est.sort_by(|a, b| a.1.total_cmp(&b.1));
            // Top-1 true neighbour must be within a 10% re-rank window. Pseudo-Gaussian
            // points are nearly equidistant, the hardest case for a 1-bit code.
            if est[..40].iter().any(|(i, _)| *i == exact[0].0) {
                hits += 1;
            }
        }
        let mean_rel = rel_err_sum / (queries.len() * data.len()) as f32;
        assert!(mean_rel < 0.15, "mean relative error {mean_rel}");
        assert!(hits >= 19, "top-1 in top-40: {hits}/20");
    }

    #[test]
    fn centroid_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("rbq.bin");
        let mut m = RaBitQ::new(4);
        m.centroid = vec![1.0, -2.0, 0.5, 3.25];
        m.save_to_file(&p).unwrap();
        assert_eq!(RaBitQ::load_from_file(&p).unwrap(), m);
    }

    #[test]
    fn reconstruction_points_in_the_right_direction() {
        let dim = 128;
        let data = vecs(64, dim, 3);
        let mut model = RaBitQ::new(dim);
        let refs: Vec<&[f32]> = data.iter().map(Vec::as_slice).collect();
        model.fit_centroid(&refs);
        let x = &data[0];
        let mut r = Vec::new();
        model.reconstruct_into(&model.encode(x), &mut r);
        let rx: Vec<f32> = x.iter().zip(&model.centroid).map(|(a, c)| a - c).collect();
        let rr: Vec<f32> = r.iter().zip(&model.centroid).map(|(a, c)| a - c).collect();
        let dot: f32 = rx.iter().zip(&rr).map(|(a, b)| a * b).sum();
        let cos = dot / (norm(&rx) * norm(&rr));
        assert!(cos > 0.7, "cos={cos}");
    }
}
