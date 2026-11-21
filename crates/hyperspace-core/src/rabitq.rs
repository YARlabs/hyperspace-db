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
//!
//! # Extended (multi-bit) RaBitQ
//!
//! Gao et al., *Practical and Asymptotically Optimal Quantization of High-Dimensional
//! Vectors in Euclidean Space for Approximate Nearest Neighbor Search* (SIGMOD 2025).
//! With `B ∈ {2, 4}` bits per dimension the rotated unit residual `o'` is coded on the
//! uniform grid `w_i = u_i − m`, `u_i ∈ [0, 2^B − 1]`, `m = (2^B − 1)/2`, with the scale
//! that maximizes `cos(w, o')` (searched per vector). The estimator stays unbiased:
//! `⟨o, q/‖q‖⟩ ≈ ⟨w, q'⟩ / ⟨w, o'⟩`. The query `q'` is quantized to int8 once, so
//! `⟨w, q'⟩ = s_q (Σ u_i q̂_i − m Σ q̂_i)` is one integer dot product per candidate (NEON
//! `SDOT`, AVX2 `maddubs`).
//!
//! Layout: `[codes: L bytes][‖x‖: f32][1/⟨w, o'⟩: f32]`. Codes are stored in `P = 8/B`
//! planes of `L` bytes: byte `j` holds dimensions `j, j + L, …, j + (P−1)L` in bit fields
//! `B·p`, so a kernel separates them with one shift and mask, without shuffles. `L` is
//! `⌈D/P⌉` rounded up to 32 (padding dimensions are zero in the query).

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::turbo_rot;

/// Number of 4-bit query bit-planes.
const QUERY_BITS: usize = 4;

/// Bytes per encoded 1-bit vector of dimension `dim`.
#[must_use]
pub fn code_size(dim: usize) -> usize {
    dim.div_ceil(64) * 8 + 8
}

/// Bit widths supported by the extended code.
pub const EX_BITS: [u8; 2] = [2, 4];

/// Bytes per plane of an extended code (`L` in the module docs).
#[inline]
#[must_use]
pub fn ex_plane_len(dim: usize, bits: u8) -> usize {
    let planes = 8 / usize::from(bits.max(1));
    dim.div_ceil(planes).div_ceil(32) * 32
}

/// Bytes per encoded vector of dimension `dim` with `bits` per dimension (1 = classic).
#[must_use]
pub fn code_size_bits(dim: usize, bits: u8) -> usize {
    if bits <= 1 {
        code_size(dim)
    } else {
        ex_plane_len(dim, bits) + 8
    }
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
    decode_rotated_into(code, dim, out);
    turbo_rot::unrotate_vector_in_place(out);
}

/// One-bit residual estimate in the rotated domain (`±‖x‖/√D`, no inverse rotation).
fn decode_rotated_into(code: &[u8], dim: usize, out: &mut Vec<f32>) {
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
}

/// RaBitQ quantizer: the shared centroid plus encode / query / reconstruct helpers.
#[derive(Debug, Clone, PartialEq)]
pub struct RaBitQ {
    pub dim: usize,
    pub centroid: Vec<f32>,
    /// Bits per dimension: 1 (classic sign code) or one of [`EX_BITS`] (extended code).
    pub bits: u8,
}

impl RaBitQ {
    /// Zero-centroid 1-bit quantizer (call [`Self::fit_centroid`] once data is available).
    #[must_use]
    pub fn new(dim: usize) -> Self {
        Self::with_bits(dim, 1)
    }

    /// Zero-centroid quantizer with `bits` per dimension (1, 2 or 4; other values fall
    /// back to the nearest supported width).
    #[must_use]
    pub fn with_bits(dim: usize, bits: u8) -> Self {
        let bits = match bits {
            0 | 1 => 1,
            2 | 3 => 2,
            _ => 4,
        };
        Self {
            dim,
            centroid: vec![0.0; dim],
            bits,
        }
    }

    /// Bytes per encoded vector.
    #[must_use]
    pub fn code_size(&self) -> usize {
        code_size_bits(self.dim, self.bits)
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
        if self.bits <= 1 {
            encode_residual(&self.residual(v))
        } else {
            ex_encode_residual(&self.residual(v), self.bits)
        }
    }

    /// Per-query state for distance estimation against codes.
    #[must_use]
    pub fn query(&self, q: &[f32]) -> RaBitQQuery {
        if self.bits <= 1 {
            RaBitQQuery::OneBit(OneBitQuery::from_residual(&self.residual(q)))
        } else {
            RaBitQQuery::Ex(ExQuery::from_residual(&self.residual(q), self.bits))
        }
    }

    /// Reconstructs the best estimate of the original vector into `out`.
    pub fn reconstruct_into(&self, code: &[u8], out: &mut Vec<f32>) {
        if self.bits <= 1 {
            reconstruct_residual_into(code, self.dim, out);
        } else {
            ex_reconstruct_residual_into(code, self.dim, self.bits, out);
        }
        for (o, c) in out.iter_mut().zip(&self.centroid) {
            *o += c;
        }
    }

    /// Decodes a code into the *rotated, centred* domain: the residual estimate before the
    /// inverse rotation, without the centroid. The rotation is orthonormal and the centroid
    /// shared, so L2 distances between two such decodes equal those between full
    /// reconstructions — at the cost of an unpack instead of an unpack plus an inverse FWHT.
    /// Used by graph construction for pairwise (RNG) distances.
    pub fn reconstruct_rotated_into(&self, code: &[u8], out: &mut Vec<f32>) {
        if self.bits <= 1 {
            decode_rotated_into(code, self.dim, out);
        } else {
            ex_decode_rotated_into(code, self.dim, self.bits, out);
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
        Ok(Self {
            dim,
            centroid,
            bits: 1,
        })
    }
}

/// Per-query RaBitQ state for either code width.
#[derive(Debug, Clone)]
pub enum RaBitQQuery {
    OneBit(OneBitQuery),
    Ex(ExQuery),
}

impl RaBitQQuery {
    /// Estimated squared L2 distance between the query and an encoded vector.
    #[must_use]
    #[inline]
    pub fn distance_sq(&self, code: &[u8]) -> f32 {
        match self {
            Self::OneBit(q) => q.distance_sq(code),
            Self::Ex(q) => q.distance_sq(code),
        }
    }

    #[must_use]
    pub fn dim(&self) -> usize {
        match self {
            Self::OneBit(q) => q.dim(),
            Self::Ex(q) => q.dim,
        }
    }
}

/// Coarse grid of the scale search (`t = f · (m + ½)/max|o|`, `f` from `EX_COARSE_FROM` in
/// `EX_COARSE_STEP` steps) and the fine grid within one coarse step around the best `f`.
const EX_COARSE_FROM: f32 = 0.3;
const EX_COARSE_STEP: f32 = 0.2;
const EX_COARSE_STEPS: usize = 12;
const EX_FINE_STEPS: usize = 10;

/// Packs per-dimension codes `u` (`u.len() ≤ planes · L`) into the plane layout.
fn ex_pack(u: &[u8], dim: usize, bits: u8, out: &mut [u8]) {
    let plane_len = ex_plane_len(dim, bits);
    for (d, &v) in u.iter().enumerate() {
        let (p, j) = (d / plane_len, d % plane_len);
        out[j] |= v << (u32::from(bits) * p as u32);
    }
}

/// Encodes a residual with the extended code (see module docs).
fn ex_encode_residual(x: &[f32], bits: u8) -> Vec<u8> {
    let dim = x.len();
    let plane_len = ex_plane_len(dim, bits);
    let mut out = vec![0u8; plane_len + 8];
    let n = norm(x);
    let mut inv_ip = 0.0f32;
    if n > 0.0 && dim > 0 {
        let mut o: Vec<f32> = x.iter().map(|v| v / n).collect();
        turbo_rot::rotate_vector_in_place(&mut o);
        let levels = f32::from((1u8 << bits) - 1);
        let m = levels / 2.0;
        let max_abs = o.iter().fold(0.0f32, |a, v| a.max(v.abs())).max(1e-12);
        let base = (m + 0.5) / max_abs;
        // cos(w, o) for grid scale t, without materializing the codes.
        let score = |t: f32| -> (f32, f32) {
            let (mut wo, mut ww) = (0.0f32, 0.0f32);
            for &oi in &o {
                let w = (t * oi + m).round().clamp(0.0, levels) - m;
                wo += w * oi;
                ww += w * w;
            }
            (wo / ww.sqrt().max(1e-12), wo)
        };
        // Coarse grid over the scale factor, then a fine grid around the best point.
        let mut best = (f32::MIN, 0.0f32, 1.0f32); // (cos, ⟨w,o⟩, scale factor)
        for k in 0..EX_COARSE_STEPS {
            let f = EX_COARSE_FROM + k as f32 * EX_COARSE_STEP;
            let (c, wo) = score(f * base);
            if c > best.0 {
                best = (c, wo, f);
            }
        }
        let centre = best.2;
        for k in 0..=EX_FINE_STEPS {
            let f =
                centre - EX_COARSE_STEP + 2.0 * EX_COARSE_STEP * k as f32 / EX_FINE_STEPS as f32;
            if f <= 0.0 {
                continue;
            }
            let (c, wo) = score(f * base);
            if c > best.0 {
                best = (c, wo, f);
            }
        }
        let t = best.2 * base;
        let u: Vec<u8> = o
            .iter()
            .map(|&oi| (t * oi + m).round().clamp(0.0, levels) as u8)
            .collect();
        ex_pack(&u, dim, bits, &mut out[..plane_len]);
        inv_ip = if best.1 > 1e-12 { 1.0 / best.1 } else { 0.0 };
    }
    out[plane_len..plane_len + 4].copy_from_slice(&n.to_le_bytes());
    out[plane_len + 4..plane_len + 8].copy_from_slice(&inv_ip.to_le_bytes());
    out
}

/// Squared L2 distance between the vectors two codes reconstruct to.
///
/// The rotation is orthonormal and the centroid is shared, so
/// `‖x̂_a − x̂_b‖² = ‖r̂_a − r̂_b‖²` with `r̂ = ‖x‖·w/‖w‖` in the rotated domain:
/// `n_a² + n_b² − 2·n_a·n_b·⟨w_a,w_b⟩/(‖w_a‖‖w_b‖)`. Everything is integer except the final
/// combination, so graph construction (RNG pruning between stored neighbours) never has to
/// unpack and inverse-rotate a code. Matches `reconstruct_into` + L2 up to float rounding.
#[must_use]
pub fn code_pair_l2_sq(a: &[u8], b: &[u8], dim: usize, bits: u8) -> f32 {
    if bits <= 1 {
        let words = dim.div_ceil(64);
        let need = words * 8 + 4;
        if a.len() < need || b.len() < need || dim == 0 {
            return f32::MAX;
        }
        let mut diff = 0u32;
        for (ca, cb) in a[..words * 8]
            .as_chunks::<8>()
            .0
            .iter()
            .zip(b[..words * 8].as_chunks::<8>().0)
        {
            diff += (u64::from_le_bytes(*ca) ^ u64::from_le_bytes(*cb)).count_ones();
        }
        let na = f64::from(read_f32(a, words * 8));
        let nb = f64::from(read_f32(b, words * 8));
        let cos = (dim as f64 - 2.0 * f64::from(diff)) / dim as f64;
        return (na * na + nb * nb - 2.0 * na * nb * cos).max(0.0) as f32;
    }
    let plane_len = ex_plane_len(dim, bits);
    if a.len() < plane_len + 4 || b.len() < plane_len + 4 || dim == 0 {
        return f32::MAX;
    }
    let (ab, sa, sb, qa, qb) = ex_pair_sums(&a[..plane_len], &b[..plane_len], bits);
    // Padding codes are zero, so the sums cover exactly the `dim` real coordinates.
    let m = f64::from((1u8 << bits) - 1) / 2.0;
    let d = dim as f64;
    let (sa, sb) = (sa as f64, sb as f64);
    let wab = ab as f64 - m * (sa + sb) + d * m * m;
    let waa = (qa as f64 - 2.0 * m * sa + d * m * m).max(1e-12);
    let wbb = (qb as f64 - 2.0 * m * sb + d * m * m).max(1e-12);
    let na = f64::from(read_f32(a, plane_len));
    let nb = f64::from(read_f32(b, plane_len));
    let cos = wab / (waa * wbb).sqrt();
    (na * na + nb * nb - 2.0 * na * nb * cos).max(0.0) as f32
}

#[inline]
fn read_f32(code: &[u8], at: usize) -> f32 {
    f32::from_le_bytes(code[at..at + 4].try_into().unwrap_or([0; 4]))
}

/// `(Σ u_a·u_b, Σ u_a, Σ u_b, Σ u_a², Σ u_b²)` over the packed planes of two codes.
#[inline]
fn ex_pair_sums(a: &[u8], b: &[u8], bits: u8) -> (u64, u64, u64, u64, u64) {
    let mask = (1u16 << bits) - 1;
    let shifts: &[u16] = if bits == 2 { &[0, 2, 4, 6] } else { &[0, 4] };
    let (mut ab, mut sa, mut sb, mut qa, mut qb) = (0u64, 0u64, 0u64, 0u64, 0u64);
    // Blocks of 256 bytes keep the u32 lane sums far from overflow (≤ 256·4·225).
    for (ba, bb) in a.chunks(256).zip(b.chunks(256)) {
        let (mut x, mut y, mut z, mut v, mut w) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for (&ca, &cb) in ba.iter().zip(bb) {
            let (ca, cb) = (u16::from(ca), u16::from(cb));
            for &s in shifts {
                let ua = u32::from((ca >> s) & mask);
                let ub = u32::from((cb >> s) & mask);
                x += ua * ub;
                y += ua;
                z += ub;
                v += ua * ua;
                w += ub * ub;
            }
        }
        ab += u64::from(x);
        sa += u64::from(y);
        sb += u64::from(z);
        qa += u64::from(v);
        qb += u64::from(w);
    }
    (ab, sa, sb, qa, qb)
}

/// Reconstructs `‖x‖ · P⁻¹·w/‖w‖` (the residual estimate) into `out`.
fn ex_reconstruct_residual_into(code: &[u8], dim: usize, bits: u8, out: &mut Vec<f32>) {
    ex_decode_rotated_into(code, dim, bits, out);
    turbo_rot::unrotate_vector_in_place(out);
}

/// `‖x‖ · w/‖w‖` in the rotated domain (no inverse rotation), decoded plane by plane.
fn ex_decode_rotated_into(code: &[u8], dim: usize, bits: u8, out: &mut Vec<f32>) {
    out.clear();
    let plane_len = ex_plane_len(dim, bits);
    if code.len() < plane_len + 8 {
        out.resize(dim, 0.0);
        return;
    }
    out.resize(dim, 0.0);
    let n = read_f32(code, plane_len);
    let mask = (1u8 << bits) - 1;
    let m = f32::from(mask) / 2.0;
    let planes = 8 / usize::from(bits);
    for p in 0..planes {
        let start = p * plane_len;
        if start >= dim {
            break;
        }
        let shift = u32::from(bits) * p as u32;
        let dst = &mut out[start..dim.min(start + plane_len)];
        for (o, &c) in dst.iter_mut().zip(&code[..plane_len]) {
            *o = f32::from((c >> shift) & mask) - m;
        }
    }
    let wn = norm(out).max(1e-12);
    let s = n / wn;
    for v in out.iter_mut() {
        *v *= s;
    }
}

/// Per-query state of the extended code: int8 rotated query in the plane layout.
#[derive(Debug, Clone)]
pub struct ExQuery {
    dim: usize,
    bits: u8,
    plane_len: usize,
    /// `q̂` laid out as `P` consecutive planes of `L` values (padding is zero).
    q: Vec<i8>,
    /// `s_q` and `m · s_q · Σ q̂` of the estimator.
    scale: f32,
    offset: f32,
    q_norm: f32,
}

impl ExQuery {
    fn from_residual(q: &[f32], bits: u8) -> Self {
        let dim = q.len();
        let plane_len = ex_plane_len(dim, bits);
        let planes = 8 / usize::from(bits);
        let q_norm = norm(q);
        let mut r: Vec<f32> = if q_norm > 0.0 {
            q.iter().map(|v| v / q_norm).collect()
        } else {
            vec![0.0; dim]
        };
        turbo_rot::rotate_vector_in_place(&mut r);
        let max_abs = r.iter().fold(0.0f32, |a, v| a.max(v.abs()));
        let scale = if max_abs > 0.0 { max_abs / 127.0 } else { 0.0 };
        let mut qv = vec![0i8; planes * plane_len];
        let mut sum = 0i64;
        if scale > 0.0 {
            for (d, &v) in r.iter().enumerate() {
                let c = (v / scale).round().clamp(-127.0, 127.0) as i8;
                let (p, j) = (d / plane_len, d % plane_len);
                qv[p * plane_len + j] = c;
                sum += i64::from(c);
            }
        }
        let m = f32::from((1u8 << bits) - 1) / 2.0;
        Self {
            dim,
            bits,
            plane_len,
            q: qv,
            scale,
            offset: m * scale * sum as f32,
            q_norm,
        }
    }

    /// Estimated squared L2 distance to an extended code.
    #[must_use]
    #[inline]
    pub fn distance_sq(&self, code: &[u8]) -> f32 {
        let l = self.plane_len;
        if code.len() < l + 8 {
            return f32::MAX;
        }
        let x_norm = f32::from_le_bytes(code[l..l + 4].try_into().unwrap_or([0; 4]));
        let inv_ip = f32::from_le_bytes(code[l + 4..l + 8].try_into().unwrap_or([0; 4]));
        let dot = crate::simd::ex_code_dot(&code[..l], &self.q, self.bits);
        let w_dot_q = self.scale * dot as f32 - self.offset;
        let cos_est = (w_dot_q * inv_ip).clamp(-1.0, 1.0);
        (x_norm * x_norm + self.q_norm * self.q_norm - 2.0 * x_norm * self.q_norm * cos_est)
            .max(0.0)
    }
}

/// Per-query 1-bit RaBitQ state (rotated, 4-bit bit-plane quantized query).
#[derive(Debug, Clone)]
pub struct OneBitQuery {
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

impl OneBitQuery {
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

    fn recall_at(model: &RaBitQ, data: &[Vec<f32>], queries: &[Vec<f32>], pool: usize) -> f64 {
        let codes: Vec<Vec<u8>> = data.iter().map(|v| model.encode(v)).collect();
        let mut hits = 0usize;
        for q in queries {
            let rq = model.query(q);
            let mut exact: Vec<(usize, f32)> = data
                .iter()
                .enumerate()
                .map(|(i, v)| (i, l2(v, q)))
                .collect();
            exact.sort_by(|a, b| a.1.total_cmp(&b.1));
            let mut est: Vec<(usize, f32)> = codes
                .iter()
                .enumerate()
                .map(|(i, c)| (i, rq.distance_sq(c)))
                .collect();
            est.sort_by(|a, b| a.1.total_cmp(&b.1));
            let top: Vec<usize> = est[..pool].iter().map(|e| e.0).collect();
            hits += exact[..10].iter().filter(|e| top.contains(&e.0)).count();
        }
        hits as f64 / (queries.len() * 10) as f64
    }

    #[test]
    fn extended_codes_beat_one_bit_and_grow_with_bits() {
        let dim = 256;
        let data = vecs(1500, dim, 11);
        let queries = vecs(25, dim, 12);
        let refs: Vec<&[f32]> = data.iter().map(Vec::as_slice).collect();
        let mut prev = 0.0;
        for bits in [1u8, 2, 4] {
            let mut m = RaBitQ::with_bits(dim, bits);
            m.fit_centroid(&refs);
            let r = recall_at(&m, &data, &queries, 10);
            assert!(r > prev, "{bits}-bit recall {r:.3} not above {prev:.3}");
            prev = r;
        }
        let mut m4 = RaBitQ::with_bits(dim, 4);
        m4.fit_centroid(&refs);
        assert!(
            recall_at(&m4, &data, &queries, 40) >= 0.97,
            "4-bit top-10 within 40"
        );
    }

    #[test]
    fn extended_code_layout_and_size() {
        for (dim, bits, want) in [
            (1024, 4, 520),
            (1024, 2, 264),
            (100, 4, 64 + 8),
            (100, 2, 32 + 8),
            (1, 2, 32 + 8),
        ] {
            let m = RaBitQ::with_bits(dim, bits);
            assert_eq!(m.code_size(), want, "dim {dim} bits {bits}");
            assert_eq!(m.encode(&vec![0.25; dim]).len(), want);
        }
        assert_eq!(RaBitQ::with_bits(8, 3).bits, 2);
        assert_eq!(RaBitQ::with_bits(8, 7).bits, 4);
        // Zero residual: finite estimate equal to the query norm.
        let m = RaBitQ::with_bits(64, 4);
        let q = m.query(&[0.5; 64]);
        let d = q.distance_sq(&m.encode(&[0.0; 64]));
        assert!((d - 16.0).abs() < 1e-3, "{d}");
        assert_eq!(
            q.distance_sq(&[0u8; 3]).to_bits(),
            f32::MAX.to_bits(),
            "short code"
        );
    }

    #[test]
    fn extended_reconstruction_is_close() {
        let dim = 128;
        let data = vecs(64, dim, 5);
        for bits in [2u8, 4] {
            let mut m = RaBitQ::with_bits(dim, bits);
            let refs: Vec<&[f32]> = data.iter().map(Vec::as_slice).collect();
            m.fit_centroid(&refs);
            let x = &data[3];
            let mut r = Vec::new();
            m.reconstruct_into(&m.encode(x), &mut r);
            let rx: Vec<f32> = x.iter().zip(&m.centroid).map(|(a, c)| a - c).collect();
            let rr: Vec<f32> = r.iter().zip(&m.centroid).map(|(a, c)| a - c).collect();
            let cos = rx.iter().zip(&rr).map(|(a, b)| a * b).sum::<f32>() / (norm(&rx) * norm(&rr));
            assert!(
                cos > if bits == 4 { 0.97 } else { 0.85 },
                "{bits}-bit cos={cos}"
            );
            assert!(
                (norm(&rr) - norm(&rx)).abs() / norm(&rx) < 1e-3,
                "norm kept"
            );
        }
    }

    #[test]
    fn code_pair_distance_matches_reconstruction() {
        for &dim in &[1usize, 37, 100, 128, 1024] {
            for bits in [1u8, 2, 4] {
                let mut rq = RaBitQ::with_bits(dim, bits);
                let pts = vecs(12, dim, 7 + dim as u64);
                let refs: Vec<&[f32]> = pts.iter().map(Vec::as_slice).collect();
                rq.fit_centroid(&refs);
                let codes: Vec<Vec<u8>> = pts.iter().map(|p| rq.encode(p)).collect();
                let (mut ra, mut rb) = (Vec::new(), Vec::new());
                for i in 0..codes.len() {
                    for j in 0..codes.len() {
                        rq.reconstruct_into(&codes[i], &mut ra);
                        rq.reconstruct_into(&codes[j], &mut rb);
                        let want: f32 = ra.iter().zip(&rb).map(|(x, y)| (x - y) * (x - y)).sum();
                        let got = code_pair_l2_sq(&codes[i], &codes[j], dim, bits);
                        assert!(
                            (got - want).abs() <= 1e-3 * want.max(1.0),
                            "dim {dim} bits {bits} ({i},{j}): {got} vs {want}"
                        );
                    }
                }
                assert_eq!(
                    code_pair_l2_sq(&codes[0], &[], dim, bits).to_bits(),
                    f32::MAX.to_bits()
                );
                // Rotated-domain decodes preserve pairwise distances too.
                let (mut ta, mut tb) = (Vec::new(), Vec::new());
                for i in 0..codes.len() {
                    let j = (i + 5) % codes.len();
                    rq.reconstruct_rotated_into(&codes[i], &mut ta);
                    rq.reconstruct_rotated_into(&codes[j], &mut tb);
                    assert_eq!(ta.len(), dim);
                    let got: f32 = ta.iter().zip(&tb).map(|(x, y)| (x - y) * (x - y)).sum();
                    let want = code_pair_l2_sq(&codes[i], &codes[j], dim, bits);
                    assert!(
                        (got - want).abs() <= 1e-3 * want.max(1.0),
                        "rotated dim {dim} bits {bits}: {got} vs {want}"
                    );
                }
            }
        }
    }

    #[test]
    fn simd_dot_matches_scalar() {
        let mut state = 99u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            (state >> 33) as u32
        };
        for bits in [2u8, 4] {
            let planes = 8 / usize::from(bits);
            for l in [32usize, 64, 512] {
                let code: Vec<u8> = (0..l).map(|_| next() as u8).collect();
                let q: Vec<i8> = (0..l * planes)
                    .map(|_| i8::try_from(i64::from(next() % 255) - 127).unwrap_or(0))
                    .collect();
                assert_eq!(
                    crate::simd::ex_code_dot(&code, &q, bits),
                    crate::simd::ex_code_dot_scalar(&code, &q, bits),
                    "bits {bits} len {l}"
                );
            }
        }
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
