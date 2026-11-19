//! 4-bit Product Quantization with a fast-scan lookup table (PQ4 / "PQ fast-scan").
//!
//! Each sub-space has 16 centroids, so a code is a nibble and a vector of `m` sub-spaces
//! packs into `⌈m/2⌉` bytes. The per-query ADC table is `m × 16` entries: quantized to
//! `u8` it is only `16·m` bytes (768 B for 768-D with `sub_dim = 4`) and stays resident
//! in L1, versus `256·m·4` bytes (48 KB+) for classic 8-bit PQ whose random lookups miss
//! L1 on every sub-space. Distances accumulate in integers:
//! `d ≈ Σ_s bias_s + scale · Σ_s lut_u8[s][code_s]`.
//!
//! [`Pq4Lut::scan_block`] additionally implements the batched fast-scan kernel for
//! sequential scans (brute-force filtered search): 32 codes are evaluated per sub-space
//! table, which LLVM vectorizes into byte shuffles on AVX2 / NEON.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_range_loop)]

const KSUB: usize = 16;

/// Default sub-space count for PQ4 (`sub_dim = 4` when possible).
#[must_use]
pub fn default_num_subvectors(dimension: usize) -> usize {
    if dimension == 0 {
        1
    } else if dimension.is_multiple_of(4) {
        dimension / 4
    } else if dimension.is_multiple_of(2) {
        dimension / 2
    } else {
        dimension
    }
}

/// Bytes per PQ4 code for `num_subvectors` sub-spaces.
#[must_use]
pub fn code_size(num_subvectors: usize) -> usize {
    num_subvectors.div_ceil(2)
}

/// PQ4 codebook (`[m][16][sub_dim]` centroids).
#[derive(Debug, Clone)]
pub struct Pq4Quantizer {
    pub dimension: usize,
    pub num_subvectors: usize,
    pub subvector_dim: usize,
    pub centroids: Vec<f32>,
}

impl Pq4Quantizer {
    /// Creates a codebook with a deterministic spread (usable before training).
    #[must_use]
    pub fn new(dimension: usize, num_subvectors: usize) -> Self {
        let num_subvectors = num_subvectors.max(1);
        let subvector_dim = dimension.max(1).div_ceil(num_subvectors);
        let mut centroids = vec![0.0f32; num_subvectors * KSUB * subvector_dim];
        let scale = 1.0 / (dimension.max(1) as f32).sqrt();
        for (i, c) in centroids.iter_mut().enumerate() {
            let mut x = (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
            x ^= x >> 29;
            let u = ((x & 0xFFFF) as f32 / 65535.0) * 2.0 - 1.0;
            *c = u * 2.0 * scale;
        }
        Self {
            dimension,
            num_subvectors,
            subvector_dim,
            centroids,
        }
    }

    #[inline]
    fn centroid(&self, sub: usize, k: usize) -> &[f32] {
        let o = (sub * KSUB + k) * self.subvector_dim;
        &self.centroids[o..o + self.subvector_dim]
    }

    #[inline]
    fn sub_dist(&self, v: &[f32], sub: usize, k: usize) -> f32 {
        let off = sub * self.subvector_dim;
        let c = self.centroid(sub, k);
        let mut d = 0.0f32;
        for (j, &cv) in c.iter().enumerate() {
            let x = v.get(off + j).copied().unwrap_or(0.0);
            d += (x - cv) * (x - cv);
        }
        d
    }

    /// Lloyd k-means (16 centroids per sub-space).
    pub fn train(&mut self, samples: &[&[f32]], iterations: usize) {
        if samples.is_empty() {
            return;
        }
        let n = samples.len();
        let sd = self.subvector_dim;
        for sub in 0..self.num_subvectors {
            let off = sub * sd;
            for k in 0..KSUB {
                let s = samples[(k * n) / KSUB.max(1) % n];
                for j in 0..sd {
                    self.centroids[(sub * KSUB + k) * sd + j] =
                        s.get(off + j).copied().unwrap_or(0.0);
                }
            }
            let mut assign = vec![0usize; n];
            for _ in 0..iterations.max(1) {
                for (i, s) in samples.iter().enumerate() {
                    assign[i] = (0..KSUB)
                        .min_by(|&a, &b| {
                            self.sub_dist(s, sub, a)
                                .total_cmp(&self.sub_dist(s, sub, b))
                        })
                        .unwrap_or(0);
                }
                let mut sums = vec![0.0f32; KSUB * sd];
                let mut counts = [0usize; KSUB];
                for (i, s) in samples.iter().enumerate() {
                    counts[assign[i]] += 1;
                    for j in 0..sd {
                        sums[assign[i] * sd + j] += s.get(off + j).copied().unwrap_or(0.0);
                    }
                }
                for k in 0..KSUB {
                    if counts[k] > 0 {
                        for j in 0..sd {
                            self.centroids[(sub * KSUB + k) * sd + j] =
                                sums[k * sd + j] / counts[k] as f32;
                        }
                    }
                }
            }
        }
    }

    /// Encodes `v` into `⌈m/2⌉` bytes (even sub-space in the low nibble).
    #[must_use]
    pub fn encode(&self, v: &[f32]) -> Vec<u8> {
        let mut out = vec![0u8; code_size(self.num_subvectors)];
        for sub in 0..self.num_subvectors {
            let best = (0..KSUB)
                .min_by(|&a, &b| {
                    self.sub_dist(v, sub, a)
                        .total_cmp(&self.sub_dist(v, sub, b))
                })
                .unwrap_or(0) as u8;
            out[sub / 2] |= if sub % 2 == 0 { best } else { best << 4 };
        }
        out
    }

    #[inline]
    fn code_at(codes: &[u8], sub: usize) -> usize {
        let b = codes.get(sub / 2).copied().unwrap_or(0);
        (if sub.is_multiple_of(2) {
            b & 0x0F
        } else {
            b >> 4
        }) as usize
    }

    /// Reconstructs a vector from its code into `out`.
    pub fn decode_into(&self, codes: &[u8], out: &mut Vec<f32>) {
        out.clear();
        out.resize(self.dimension, 0.0);
        for sub in 0..self.num_subvectors {
            let c = self.centroid(sub, Self::code_at(codes, sub));
            let off = sub * self.subvector_dim;
            for (j, &cv) in c.iter().enumerate() {
                if off + j < self.dimension {
                    out[off + j] = cv;
                }
            }
        }
    }

    /// Builds the per-query fast-scan lookup table.
    #[must_use]
    pub fn compute_lut(&self, q: &[f32]) -> Pq4Lut {
        let m = self.num_subvectors;
        let mut table = vec![0.0f32; m * KSUB];
        for sub in 0..m {
            for k in 0..KSUB {
                table[sub * KSUB + k] = self.sub_dist(q, sub, k);
            }
        }
        Pq4Lut::from_f32(m, &table)
    }

    /// Serializes the codebook.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(16 + self.centroids.len() * 4);
        buf.extend_from_slice(b"PQ4C");
        buf.extend_from_slice(&(self.dimension as u32).to_le_bytes());
        buf.extend_from_slice(&(self.num_subvectors as u32).to_le_bytes());
        buf.extend_from_slice(&(self.centroids.len() as u32).to_le_bytes());
        for c in &self.centroids {
            buf.extend_from_slice(&c.to_le_bytes());
        }
        std::fs::write(path, buf)
    }

    /// Loads a codebook written by [`Self::save_to_file`].
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid PQ4 codebook");
        if bytes.len() < 16 || &bytes[..4] != b"PQ4C" {
            return Err(bad());
        }
        let rd = |o: usize| {
            u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]) as usize
        };
        let (dimension, num_subvectors, len) = (rd(4), rd(8), rd(12));
        if bytes.len() != 16 + len * 4 || num_subvectors == 0 {
            return Err(bad());
        }
        let centroids = bytes[16..]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        Ok(Self {
            dimension,
            num_subvectors,
            subvector_dim: dimension.max(1).div_ceil(num_subvectors),
            centroids,
        })
    }
}

/// Per-query PQ4 table quantized to `u8` (L1-resident).
#[derive(Debug, Clone)]
pub struct Pq4Lut {
    m: usize,
    lut: Vec<u8>,
    scale: f32,
    bias_sum: f32,
}

impl Pq4Lut {
    fn from_f32(m: usize, table: &[f32]) -> Self {
        let mut biases = vec![0.0f32; m];
        let mut max_range = 0.0f32;
        for sub in 0..m {
            let row = &table[sub * KSUB..(sub + 1) * KSUB];
            let lo = row.iter().copied().fold(f32::INFINITY, f32::min);
            let hi = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            biases[sub] = lo;
            max_range = max_range.max(hi - lo);
        }
        let scale = if max_range > 0.0 {
            max_range / 255.0
        } else {
            1.0
        };
        let lut = (0..m * KSUB)
            .map(|i| {
                ((table[i] - biases[i / KSUB]) / scale)
                    .round()
                    .clamp(0.0, 255.0) as u8
            })
            .collect();
        Self {
            m,
            lut,
            scale,
            bias_sum: biases.iter().sum(),
        }
    }

    /// ADC distance for one packed code.
    #[must_use]
    #[inline]
    pub fn distance(&self, codes: &[u8]) -> f32 {
        let mut acc = 0u32;
        let pairs = self.m / 2;
        let lut = &self.lut[..self.m * KSUB];
        for (p, &byte) in codes.iter().take(pairs).enumerate() {
            let base = p * 2 * KSUB;
            acc += u32::from(lut[base + (byte & 0x0F) as usize]);
            acc += u32::from(lut[base + KSUB + (byte >> 4) as usize]);
        }
        if self.m % 2 == 1 {
            let byte = codes.get(pairs).copied().unwrap_or(0);
            acc += u32::from(lut[(self.m - 1) * KSUB + (byte & 0x0F) as usize]);
        }
        self.bias_sum + self.scale * acc as f32
    }

    /// Batched fast-scan: distances of `codes.len() / code_size` consecutive codes.
    /// Sub-space-major loop over a block of codes keeps one 16-byte table row hot and lets
    /// the compiler emit byte-shuffle lookups.
    pub fn scan_block(&self, codes: &[u8], out: &mut Vec<f32>) {
        let cs = code_size(self.m);
        let n = codes.len().checked_div(cs).unwrap_or(0);
        let mut acc = vec![0u16; n];
        out.clear();
        for sub in 0..self.m {
            let row = &self.lut[sub * KSUB..(sub + 1) * KSUB];
            let (byte_idx, hi) = (sub / 2, sub % 2 == 1);
            for (i, a) in acc.iter_mut().enumerate() {
                let b = codes[i * cs + byte_idx];
                let nib = if hi { b >> 4 } else { b & 0x0F };
                *a = a.saturating_add(u16::from(row[nib as usize]));
            }
        }
        out.extend(
            acc.iter()
                .map(|&a| self.bias_sum + self.scale * f32::from(a)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(n: usize, dim: usize, seed: f32) -> Vec<Vec<f32>> {
        (0..n)
            .map(|i| {
                (0..dim)
                    .map(|d| (i as f32 * 0.21 + d as f32 * 0.9 + seed).sin())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn encode_decode_and_lut_agree() {
        let dim = 64;
        let mut pq = Pq4Quantizer::new(dim, default_num_subvectors(dim));
        let train = data(300, dim, 0.0);
        let refs: Vec<&[f32]> = train.iter().map(Vec::as_slice).collect();
        pq.train(&refs, 6);
        assert_eq!(code_size(pq.num_subvectors), 8);

        let q = &data(1, dim, 0.3)[0];
        let lut = pq.compute_lut(q);
        let codes: Vec<Vec<u8>> = train.iter().map(|v| pq.encode(v)).collect();
        let mut rec = Vec::new();
        let mut flat = Vec::new();
        for c in &codes {
            pq.decode_into(c, &mut rec);
            let exact_adc: f32 = rec.iter().zip(q).map(|(a, b)| (a - b) * (a - b)).sum();
            let fast = lut.distance(c);
            assert!(
                (fast - exact_adc).abs() <= 0.05 * exact_adc.max(1.0),
                "{fast} vs {exact_adc}"
            );
            flat.extend_from_slice(c);
        }
        let mut block = Vec::new();
        lut.scan_block(&flat, &mut block);
        for (c, b) in codes.iter().zip(&block) {
            assert!((lut.distance(c) - b).abs() < 1e-3);
        }
    }

    #[test]
    fn codebook_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("pq4.bin");
        let pq = Pq4Quantizer::new(32, 8);
        pq.save_to_file(&p).unwrap();
        let back = Pq4Quantizer::load_from_file(&p).unwrap();
        assert_eq!(back.centroids, pq.centroids);
        assert_eq!(back.num_subvectors, 8);
    }
}
