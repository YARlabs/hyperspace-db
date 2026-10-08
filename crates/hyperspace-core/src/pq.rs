//! Product Quantization (PQ) and Optimized Product Quantization (OPQ)
//!
//! Provides ultra-compact vector compression (16x-64x) and ultra-fast
//! Asymmetric Distance Computation (ADC) using precomputed lookup tables (LUTs).

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::doc_markdown)]

use serde::{Deserialize, Serialize};

/// Encoded compact Product Quantization vector.
/// Memory usage: `num_subvectors` bytes (e.g. 64 bytes for a 1024D vector = 64x compression).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PQVector {
    pub codes: Vec<u8>,
}

impl PQVector {
    #[must_use]
    pub fn new(codes: Vec<u8>) -> Self {
        Self { codes }
    }

    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.codes.clone()
    }

    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            codes: bytes.to_vec(),
        }
    }
}

/// Precomputed query-to-codebook Asymmetric Distance Computation (ADC) Lookup Table.
/// Enables nanosecond distance evaluations using simple table additions.
#[derive(Debug, Clone)]
pub struct PQLookupTable {
    pub num_subvectors: usize,
    /// Flat array of size `num_subvectors * 256` containing squared Euclidean distances.
    pub table: Vec<f32>,
}

impl PQLookupTable {
    /// Compute Asymmetric Distance Computation (ADC) to a quantized PQ vector.
    /// Executes `num_subvectors` lookups and additions with loop unrolling.
    #[must_use]
    #[inline(always)]
    pub fn distance(&self, pq_vector: &PQVector) -> f32 {
        self.distance_bytes(&pq_vector.codes)
    }

    /// Compute ADC distance directly from raw code bytes.
    #[must_use]
    #[inline(always)]
    pub fn distance_bytes(&self, codes: &[u8]) -> f32 {
        let m = self.num_subvectors.min(codes.len());
        let table = self.table.as_slice();

        let chunks = m / 8;
        let mut acc0 = 0.0f32;
        let mut acc1 = 0.0f32;
        let mut acc2 = 0.0f32;
        let mut acc3 = 0.0f32;

        for ch in 0..chunks {
            let base_m = ch * 8;
            let c0 = codes[base_m] as usize;
            let c1 = codes[base_m + 1] as usize;
            let c2 = codes[base_m + 2] as usize;
            let c3 = codes[base_m + 3] as usize;
            let c4 = codes[base_m + 4] as usize;
            let c5 = codes[base_m + 5] as usize;
            let c6 = codes[base_m + 6] as usize;
            let c7 = codes[base_m + 7] as usize;

            acc0 += table[base_m * 256 + c0] + table[(base_m + 1) * 256 + c1];
            acc1 += table[(base_m + 2) * 256 + c2] + table[(base_m + 3) * 256 + c3];
            acc2 += table[(base_m + 4) * 256 + c4] + table[(base_m + 5) * 256 + c5];
            acc3 += table[(base_m + 6) * 256 + c6] + table[(base_m + 7) * 256 + c7];
        }

        let mut sum = (acc0 + acc1) + (acc2 + acc3);
        for sub_idx in (chunks * 8)..m {
            let code = codes[sub_idx] as usize;
            sum += table[sub_idx * 256 + code];
        }

        sum
    }
}

/// Product Quantizer codebook manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductQuantizer {
    pub dimension: usize,
    pub num_subvectors: usize,
    pub subvector_dim: usize,
    pub is_opq: bool,
    /// Centroids for each subvector: flat vector of shape `[num_subvectors, 256, subvector_dim]`
    pub centroids: Vec<f32>,
}

/// Default number of subvectors for a given dimension (targeting subvector_dim = 16).
#[must_use]
pub fn default_num_subvectors(dimension: usize) -> usize {
    if dimension == 0 {
        return 1;
    }
    if dimension.is_multiple_of(16) {
        dimension / 16
    } else if dimension.is_multiple_of(8) {
        dimension / 8
    } else if dimension.is_multiple_of(4) {
        dimension / 4
    } else {
        dimension
    }
}

impl ProductQuantizer {
    /// Create a new Product Quantizer.
    /// `num_subvectors` must evenly divide `dimension` or will be rounded up.
    #[must_use]
    pub fn new(dimension: usize, num_subvectors: usize, is_opq: bool) -> Self {
        assert!(
            num_subvectors > 0 && dimension > 0,
            "Dimension and num_subvectors must be positive"
        );
        let subvector_dim = dimension.div_ceil(num_subvectors);
        let total_centroids = num_subvectors * 256 * subvector_dim;
        let mut centroids = vec![0.0f32; total_centroids];

        // Deterministic pseudo-Gaussian spread across subvector space
        let scale = 1.0 / (dimension as f32).sqrt().max(1e-4);
        for sub_idx in 0..num_subvectors {
            for k in 0..256 {
                let offset = (sub_idx * 256 + k) * subvector_dim;
                for d in 0..subvector_dim {
                    let seed = ((sub_idx as u64) * 256 + (k as u64)) * (subvector_dim as u64)
                        + (d as u64)
                        + 1;
                    let mut x = seed.wrapping_mul(0x517c_c1b7_2722_0a95);
                    x ^= x >> 32;
                    let u1 = ((x & 0xffff) as f32 + 1.0) / 65537.0;
                    let u2 = (((x >> 16) & 0xffff) as f32 + 1.0) / 65537.0;
                    let norm_val =
                        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
                    centroids[offset + d] = norm_val * scale;
                }
            }
        }

        Self {
            dimension,
            num_subvectors,
            subvector_dim,
            is_opq,
            centroids,
        }
    }

    /// Save ProductQuantizer codebooks to a binary file.
    pub fn save_to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let mut buf = Vec::with_capacity(20 + self.centroids.len() * 4);
        buf.extend_from_slice(b"PQCB");
        buf.extend_from_slice(&(self.dimension as u32).to_le_bytes());
        buf.extend_from_slice(&(self.num_subvectors as u32).to_le_bytes());
        buf.extend_from_slice(&(if self.is_opq { 1u32 } else { 0u32 }).to_le_bytes());
        buf.extend_from_slice(&(self.centroids.len() as u32).to_le_bytes());
        for &val in &self.centroids {
            buf.extend_from_slice(&val.to_le_bytes());
        }
        std::fs::write(path, buf)
    }

    /// Load ProductQuantizer codebooks from a binary file.
    pub fn load_from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        if bytes.len() < 20 || &bytes[0..4] != b"PQCB" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid PQ codebook magic",
            ));
        }
        let dimension = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        let num_subvectors = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let is_opq = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) != 0;
        let centroids_len = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;

        if bytes.len() < 20 + centroids_len * 4 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Incomplete PQ codebook file",
            ));
        }

        let mut centroids = Vec::with_capacity(centroids_len);
        for i in 0..centroids_len {
            let offset = 20 + i * 4;
            let val = f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
            centroids.push(val);
        }

        let subvector_dim = dimension.div_ceil(num_subvectors);
        Ok(Self {
            dimension,
            num_subvectors,
            subvector_dim,
            is_opq,
            centroids,
        })
    }

    #[inline(always)]
    fn centroid_slice(&self, sub_idx: usize, centroid_idx: usize) -> &[f32] {
        let offset = (sub_idx * 256 + centroid_idx) * self.subvector_dim;
        &self.centroids[offset..offset + self.subvector_dim]
    }

    #[inline(always)]
    fn centroid_slice_mut(&mut self, sub_idx: usize, centroid_idx: usize) -> &mut [f32] {
        let offset = (sub_idx * 256 + centroid_idx) * self.subvector_dim;
        &mut self.centroids[offset..offset + self.subvector_dim]
    }

    /// Train codebooks using K-means clustering on a set of training samples.
    pub fn train(&mut self, training_vectors: &[&[f32]], iterations: usize) {
        if training_vectors.is_empty() {
            return;
        }

        let transformed_vectors: Vec<Vec<f32>> = if self.is_opq {
            training_vectors
                .iter()
                .map(|&v| {
                    let mut copy = v.to_vec();
                    crate::turbo_rot::rotate_vector_in_place(&mut copy);
                    copy
                })
                .collect()
        } else {
            training_vectors.iter().map(|&v| v.to_vec()).collect()
        };

        let sub_dim = self.subvector_dim;
        let n_samples = transformed_vectors.len();

        for sub_idx in 0..self.num_subvectors {
            let sub_offset = sub_idx * sub_dim;

            // 1. Initialize 256 centroids using uniform stride across dataset
            for k in 0..256 {
                let sample_idx = if n_samples > 256 {
                    (k * n_samples) / 256
                } else {
                    k % n_samples
                };
                let sample = &transformed_vectors[sample_idx];
                let c_slice = self.centroid_slice_mut(sub_idx, k);
                for d in 0..sub_dim {
                    let v_idx = sub_offset + d;
                    c_slice[d] = if v_idx < sample.len() {
                        sample[v_idx]
                    } else {
                        0.0
                    };
                }
            }

            // 2. Run K-Means Lloyd iterations
            let mut assignments = vec![0u8; n_samples];
            let mut counts = vec![0usize; 256];
            let mut sums = vec![0.0f32; 256 * sub_dim];

            for _ in 0..iterations {
                // E-step: Assign each vector sub-slice to nearest centroid
                for (samp_idx, samp) in transformed_vectors.iter().enumerate() {
                    let mut best_k = 0u8;
                    let mut best_dist = f32::MAX;

                    for k in 0..256 {
                        let c = self.centroid_slice(sub_idx, k);
                        let mut dist = 0.0f32;
                        for d in 0..sub_dim {
                            let v_idx = sub_offset + d;
                            let val = if v_idx < samp.len() { samp[v_idx] } else { 0.0 };
                            let diff = val - c[d];
                            dist += diff * diff;
                        }
                        if dist < best_dist {
                            best_dist = dist;
                            best_k = k as u8;
                        }
                    }
                    assignments[samp_idx] = best_k;
                }

                // M-step: Update centroids
                counts.fill(0);
                sums.fill(0.0);

                for (samp_idx, &k) in assignments.iter().enumerate() {
                    let k_usize = k as usize;
                    counts[k_usize] += 1;
                    let samp = &transformed_vectors[samp_idx];
                    let sum_offset = k_usize * sub_dim;
                    for d in 0..sub_dim {
                        let v_idx = sub_offset + d;
                        let val = if v_idx < samp.len() { samp[v_idx] } else { 0.0 };
                        sums[sum_offset + d] += val;
                    }
                }

                for k in 0..256 {
                    if counts[k] > 0 {
                        let count_inv = 1.0 / (counts[k] as f32);
                        let c_slice = self.centroid_slice_mut(sub_idx, k);
                        let sum_offset = k * sub_dim;
                        for d in 0..sub_dim {
                            c_slice[d] = sums[sum_offset + d] * count_inv;
                        }
                    }
                }
            }
        }
    }

    /// Encode a vector into compact `PQVector`.
    #[must_use]
    pub fn encode(&self, vector: &[f32]) -> PQVector {
        let mut v_work: Vec<f32>;
        let target_v = if self.is_opq {
            v_work = vector.to_vec();
            crate::turbo_rot::rotate_vector_in_place(&mut v_work);
            &v_work[..]
        } else {
            vector
        };

        let mut codes = Vec::with_capacity(self.num_subvectors);
        let sub_dim = self.subvector_dim;

        for sub_idx in 0..self.num_subvectors {
            let sub_offset = sub_idx * sub_dim;
            let mut best_k = 0u8;
            let mut best_dist = f32::MAX;

            for k in 0..256 {
                let c = self.centroid_slice(sub_idx, k);
                let mut dist = 0.0f32;
                for d in 0..sub_dim {
                    let v_idx = sub_offset + d;
                    let val = if v_idx < target_v.len() {
                        target_v[v_idx]
                    } else {
                        0.0
                    };
                    let diff = val - c[d];
                    dist += diff * diff;
                }
                if dist < best_dist {
                    best_dist = dist;
                    best_k = k as u8;
                }
            }
            codes.push(best_k);
        }

        PQVector::new(codes)
    }

    /// Decode `PQVector` back into reconstructed float vector.
    #[must_use]
    pub fn decode(&self, pq: &PQVector) -> Vec<f32> {
        let mut out = vec![0.0f32; self.dimension];
        let sub_dim = self.subvector_dim;

        for (sub_idx, &code) in pq.codes.iter().enumerate() {
            let sub_offset = sub_idx * sub_dim;
            let c = self.centroid_slice(sub_idx, code as usize);
            for d in 0..sub_dim {
                let v_idx = sub_offset + d;
                if v_idx < self.dimension {
                    out[v_idx] = c[d];
                }
            }
        }

        if self.is_opq {
            crate::turbo_rot::unrotate_vector_in_place(&mut out);
        }

        out
    }

    /// Precompute Asymmetric Distance Computation (ADC) Lookup Table for query.
    #[must_use]
    pub fn compute_lut(&self, query: &[f32]) -> PQLookupTable {
        let mut q_work: Vec<f32>;
        let target_q = if self.is_opq {
            q_work = query.to_vec();
            crate::turbo_rot::rotate_vector_in_place(&mut q_work);
            &q_work[..]
        } else {
            query
        };

        let sub_dim = self.subvector_dim;
        let mut table = vec![0.0f32; self.num_subvectors * 256];

        for sub_idx in 0..self.num_subvectors {
            let sub_offset = sub_idx * sub_dim;
            let table_sub_offset = sub_idx * 256;

            for k in 0..256 {
                let c = self.centroid_slice(sub_idx, k);
                let mut dist = 0.0f32;
                for d in 0..sub_dim {
                    let v_idx = sub_offset + d;
                    let val = if v_idx < target_q.len() {
                        target_q[v_idx]
                    } else {
                        0.0
                    };
                    let diff = val - c[d];
                    dist += diff * diff;
                }
                table[table_sub_offset + k] = dist;
            }
        }

        PQLookupTable {
            num_subvectors: self.num_subvectors,
            table,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_encode_decode_roundtrip() {
        let dim = 128;
        let num_sub = 16;
        let mut pq = ProductQuantizer::new(dim, num_sub, false);

        // Train with random synthetic data
        let mut rng_data = Vec::new();
        for i in 0..100 {
            let v: Vec<f32> = (0..dim)
                .map(|j| ((i * 13 + j * 7) % 100) as f32 / 100.0)
                .collect();
            rng_data.push(v);
        }
        let refs: Vec<&[f32]> = rng_data.iter().map(|v| v.as_slice()).collect();
        pq.train(&refs, 3);

        let test_vec = &rng_data[0];
        let encoded = pq.encode(test_vec);
        assert_eq!(encoded.codes.len(), num_sub);

        let decoded = pq.decode(&encoded);
        assert_eq!(decoded.len(), dim);

        // Distance between original and decoded should be bounded
        let diff_sq: f32 = test_vec
            .iter()
            .zip(decoded.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum();
        assert!(diff_sq < 50.0);
    }

    #[test]
    fn test_pq_adc_lookup_table() {
        let dim = 64;
        let num_sub = 8;
        let pq = ProductQuantizer::new(dim, num_sub, false);

        let query: Vec<f32> = (0..dim).map(|i| i as f32 * 0.01).collect();
        let target: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.01) + 0.1).collect();

        let lut = pq.compute_lut(&query);
        assert_eq!(lut.table.len(), num_sub * 256);

        let encoded = pq.encode(&target);
        let adc_dist = lut.distance(&encoded);
        assert!(adc_dist >= 0.0);

        // Verify distance_bytes matches distance
        let bytes_dist = lut.distance_bytes(&encoded.to_bytes());
        assert!((adc_dist - bytes_dist).abs() < 1e-6);
    }

    #[test]
    fn test_opq_fwht_rotation_integration() {
        let dim = 128;
        let num_sub = 16;
        let mut pq_opq = ProductQuantizer::new(dim, num_sub, true);

        let mut samples = Vec::new();
        for i in 0..50 {
            let v: Vec<f32> = (0..dim)
                .map(|j| ((i * 17 + j * 5) % 100) as f32 / 100.0)
                .collect();
            samples.push(v);
        }
        let refs: Vec<&[f32]> = samples.iter().map(|v| v.as_slice()).collect();
        pq_opq.train(&refs, 2);

        let query = &samples[0];
        let lut = pq_opq.compute_lut(query);
        let encoded = pq_opq.encode(query);
        let dist = lut.distance(&encoded);
        assert!(dist >= 0.0);

        let decoded = pq_opq.decode(&encoded);
        assert_eq!(decoded.len(), dim);
    }
}
