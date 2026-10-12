//! ADSampling-style random rotation + per-dimension pruning thresholds.
//!
//! The original C++ uses Eigen's HouseholderQR. We re-implement Householder QR
//! directly in pure Rust (it runs once at construction).

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

use crate::common::KnnCandidate;
use crate::gemm;

pub struct ADSamplingPruner {
    pub num_dimensions: usize,
    pub ratios: Vec<f32>,
    pub epsilon0: f32,
    /// Random orthonormal d × d rotation matrix, row-major.
    matrix: Vec<f32>,
}

impl ADSamplingPruner {
    pub fn new(num_dimensions: usize, epsilon0: f32, seed: u64) -> Self {
        let matrix = random_orthonormal_matrix(num_dimensions, seed);
        let mut me = Self {
            num_dimensions,
            ratios: Vec::new(),
            epsilon0,
            matrix,
        };
        me.initialize_ratios();
        me
    }

    pub fn set_epsilon0(&mut self, eps0: f32) {
        self.epsilon0 = eps0;
        self.initialize_ratios();
    }

    fn initialize_ratios(&mut self) {
        let d = self.num_dimensions;
        let mut ratios = vec![0.0_f32; d + 1];
        for i in 0..=d {
            ratios[i] = compute_ratio(i, d, self.epsilon0);
        }
        self.ratios = ratios;
    }

    /// Threshold = best_distance * ratio(current_dim).
    #[inline]
    pub fn pruning_threshold(&self, best: &KnnCandidate, current_dim: usize) -> f32 {
        best.distance * self.ratios[current_dim]
    }

    /// out = vectors * matrix^T
    pub fn rotate(&self, vectors: &[f32], out: &mut [f32], n: usize) {
        let d = self.num_dimensions;
        debug_assert!(vectors.len() >= n * d);
        debug_assert!(out.len() >= n * d);
        gemm::sgemm_ld(false, true, n, d, d, vectors, d, &self.matrix, d, out, d);
    }

    /// In-place `rotate`: rewrites `data` block-by-block through a bounded
    /// scratch buffer instead of a second full-size allocation.
    pub fn rotate_in_place(&self, data: &mut [f32], n: usize) {
        self.rotate_in_place_blocked(data, n, 8 << 20);
    }

    fn rotate_in_place_blocked(&self, data: &mut [f32], n: usize, scratch_bytes: usize) {
        let d = self.num_dimensions;
        debug_assert!(data.len() >= n * d);
        let block_rows = (scratch_bytes / (d * 4)).clamp(1, n.max(1));
        let mut scratch = vec![0.0_f32; block_rows * d];
        for chunk in data[..n * d].chunks_mut(block_rows * d) {
            let rows = chunk.len() / d;
            gemm::sgemm_ld(
                false,
                true,
                rows,
                d,
                d,
                chunk,
                d,
                &self.matrix,
                d,
                &mut scratch[..rows * d],
                d,
            );
            chunk.copy_from_slice(&scratch[..rows * d]);
        }
    }

    /// out = rotated * matrix   (inverse of `rotate` for orthonormal matrix).
    pub fn unrotate(&self, rotated: &[f32], out: &mut [f32], n: usize) {
        let d = self.num_dimensions;
        gemm::sgemm_row_major(n, d, d, rotated, &self.matrix, out);
    }
}

fn compute_ratio(visited: usize, total: usize, epsilon0: f32) -> f32 {
    if visited == 0 || visited == total {
        return 1.0;
    }
    let v = visited as f64;
    let t = total as f64;
    let eps = epsilon0 as f64;
    let r = (v / t) * (1.0 + eps / v.sqrt()) * (1.0 + eps / v.sqrt());
    r as f32
}

fn random_orthonormal_matrix(d: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let normal = Normal::new(0.0_f32, 1.0).unwrap();

    // Column-major working storage: a[c][r] is element (r, c).
    let mut a: Vec<Vec<f32>> = (0..d)
        .map(|_| (0..d).map(|_| normal.sample(&mut rng)).collect())
        .collect();

    let mut q: Vec<Vec<f32>> = (0..d)
        .map(|c| {
            let mut col = vec![0.0_f32; d];
            col[c] = 1.0;
            col
        })
        .collect();

    for j in 0..d {
        // Reflector for the sub-column A[j..d, j].
        let mut norm_sq = 0.0_f32;
        for i in j..d {
            norm_sq += a[j][i] * a[j][i];
        }
        let norm = norm_sq.sqrt();
        if norm < 1.0e-12 {
            continue;
        }
        let alpha = if a[j][j] >= 0.0 { -norm } else { norm };

        let m = d - j;
        let mut v = vec![0.0_f32; m];
        v[0] = a[j][j] - alpha;
        for i in 1..m {
            v[i] = a[j][i + j];
        }
        let mut v_norm_sq = 0.0_f32;
        for &vi in &v {
            v_norm_sq += vi * vi;
        }
        if v_norm_sq < 1.0e-30 {
            continue;
        }
        let inv = 2.0 / v_norm_sq;

        // A := H A  (H acts on rows j..d).
        for c in j..d {
            let mut dot = 0.0_f32;
            for i in 0..m {
                dot += v[i] * a[c][i + j];
            }
            let scale = inv * dot;
            for i in 0..m {
                a[c][i + j] -= scale * v[i];
            }
        }

        // Q := Q H  (H acts on cols j..d, all rows).
        for r in 0..d {
            let mut dot = 0.0_f32;
            for i in 0..m {
                dot += q[j + i][r] * v[i];
            }
            let scale = inv * dot;
            for i in 0..m {
                q[j + i][r] -= scale * v[i];
            }
        }
    }

    let mut out = vec![0.0_f32; d * d];
    for c in 0..d {
        for r in 0..d {
            out[r * d + c] = q[c][r];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::generate_random_vectors;

    fn norm(v: &[f32]) -> f32 {
        v.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    fn inner(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }

    fn rotate_roundtrip_errors(d: usize, n: usize, seed: u64) -> (f64, f64) {
        let pruner = ADSamplingPruner::new(d, 2.1, seed);
        let original = generate_random_vectors(n, d, -1.0, 1.0, 42);
        let mut rotated = vec![0.0_f32; n * d];
        let mut recovered = vec![0.0_f32; n * d];
        pruner.rotate(&original, &mut rotated, n);
        pruner.unrotate(&rotated, &mut recovered, n);
        let mut max_err = 0.0_f64;
        let mut sum_err = 0.0_f64;
        for i in 0..n * d {
            let e = (original[i] - recovered[i]).abs() as f64;
            if e > max_err {
                max_err = e;
            }
            sum_err += e;
        }
        (max_err, sum_err / (n * d) as f64)
    }

    #[test]
    fn rotation_matrix_is_orthonormal() {
        let d = 16;
        let pruner = ADSamplingPruner::new(d, 1.5, 42);
        for r in 0..d {
            for c in 0..d {
                let mut s = 0.0_f32;
                for k in 0..d {
                    s += pruner.matrix[r * d + k] * pruner.matrix[c * d + k];
                }
                let expected = if r == c { 1.0 } else { 0.0 };
                assert!(
                    (s - expected).abs() < 1.0e-4,
                    "QQ^T[{r}, {c}] = {s} (expected {expected})"
                );
            }
        }
    }

    #[test]
    fn rotate_unrotate_inverse_low_dim() {
        let (max_err, avg_err) = rotate_roundtrip_errors(128, 100, 7);
        assert!(max_err < 1e-4, "max error {max_err} too large for d=128");
        assert!(avg_err < 1e-5, "avg error {avg_err} too large for d=128");
    }

    #[test]
    fn rotate_unrotate_inverse_multiple_dimensions() {
        for &d in &[50_usize, 128, 256, 512] {
            let (max_err, avg_err) = rotate_roundtrip_errors(d, 50, 7);
            assert!(max_err < 1e-3, "max error {max_err} too large for d={d}");
            assert!(avg_err < 1e-5, "avg error {avg_err} too large for d={d}");
        }
    }

    #[test]
    fn rotate_in_place_matches_rotate() {
        let d = 64;
        let n = 100;
        let pruner = ADSamplingPruner::new(d, 2.1, 42);
        let original = generate_random_vectors(n, d, -1.0, 1.0, 7);
        let mut expected = vec![0.0_f32; n * d];
        pruner.rotate(&original, &mut expected, n);

        // Small scratch forces multiple blocks (8 rows per block).
        let mut in_place = original.clone();
        pruner.rotate_in_place_blocked(&mut in_place, n, 8 * d * 4);
        for i in 0..n * d {
            let e = (expected[i] - in_place[i]).abs();
            assert!(e < 1e-5, "mismatch at {i}: {e}");
        }

        // Default scratch (single block at this size).
        let mut single_block = original.clone();
        pruner.rotate_in_place(&mut single_block, n);
        assert_eq!(single_block, expected);
    }

    #[test]
    fn rotation_preserves_norms() {
        for &d in &[64_usize, 128, 256] {
            let n = 50;
            let pruner = ADSamplingPruner::new(d, 2.1, 42);
            let original = generate_random_vectors(n, d, -1.0, 1.0, 42);
            let mut rotated = vec![0.0_f32; n * d];
            pruner.rotate(&original, &mut rotated, n);
            for i in 0..n {
                let on = norm(&original[i * d..(i + 1) * d]);
                let rn = norm(&rotated[i * d..(i + 1) * d]);
                let rel = (on - rn).abs() / on;
                assert!(
                    rel < 1e-4,
                    "norm not preserved for vector {i} at d={d} (orig={on}, rot={rn}, rel={rel})"
                );
            }
        }
    }

    #[test]
    fn rotation_preserves_inner_products() {
        for &d in &[64_usize, 128, 256] {
            let n = 20;
            let pruner = ADSamplingPruner::new(d, 2.1, 42);
            let vectors = generate_random_vectors(n, d, -1.0, 1.0, 42);
            let mut rotated = vec![0.0_f32; n * d];
            pruner.rotate(&vectors, &mut rotated, n);
            for i in 0..n {
                for j in i + 1..n {
                    let vi = &vectors[i * d..(i + 1) * d];
                    let vj = &vectors[j * d..(j + 1) * d];
                    let ri = &rotated[i * d..(i + 1) * d];
                    let rj = &rotated[j * d..(j + 1) * d];
                    let orig = inner(vi, vj);
                    let rot = inner(ri, rj);
                    let abs_err = (orig - rot).abs();
                    let rel_err = abs_err / orig.abs().max(1.0);
                    assert!(
                        abs_err < 1e-3 || rel_err < 1e-3,
                        "inner product not preserved for ({i}, {j}) at d={d}: orig={orig}, rot={rot}"
                    );
                }
            }
        }
    }

    #[test]
    fn rotation_preserves_distances() {
        for &d in &[128_usize, 256] {
            let n = 20;
            let pruner = ADSamplingPruner::new(d, 2.1, 42);
            let vectors = generate_random_vectors(n, d, -1.0, 1.0, 42);
            let mut rotated = vec![0.0_f32; n * d];
            pruner.rotate(&vectors, &mut rotated, n);
            for i in 0..n {
                for j in i + 1..n {
                    let vi = &vectors[i * d..(i + 1) * d];
                    let vj = &vectors[j * d..(j + 1) * d];
                    let ri = &rotated[i * d..(i + 1) * d];
                    let rj = &rotated[j * d..(j + 1) * d];
                    let orig_d2: f32 = vi.iter().zip(vj).map(|(a, b)| (a - b) * (a - b)).sum();
                    let rot_d2: f32 = ri.iter().zip(rj).map(|(a, b)| (a - b) * (a - b)).sum();
                    let rel = (orig_d2 - rot_d2).abs() / orig_d2;
                    assert!(
                        rel < 1e-4,
                        "distance not preserved for ({i}, {j}) at d={d}: orig={orig_d2}, rot={rot_d2}"
                    );
                }
            }
        }
    }

    #[test]
    fn single_vector_roundtrip() {
        for &d in &[64_usize, 256] {
            let (max_err, _) = rotate_roundtrip_errors(d, 1, 13);
            assert!(
                max_err < 1e-4,
                "single vector roundtrip failed at d={d}: max_err={max_err}"
            );
        }
    }

    #[test]
    fn unrotate_inverts_rotate() {
        let d = 16;
        let n = 4;
        let pruner = ADSamplingPruner::new(d, 1.5, 7);
        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let normal = Normal::new(0.0_f32, 1.0).unwrap();
        let input: Vec<f32> = (0..n * d).map(|_| normal.sample(&mut rng)).collect();
        let mut rotated = vec![0.0_f32; n * d];
        let mut restored = vec![0.0_f32; n * d];
        pruner.rotate(&input, &mut rotated, n);
        pruner.unrotate(&rotated, &mut restored, n);
        for i in 0..n * d {
            assert!(
                (input[i] - restored[i]).abs() < 1.0e-3,
                "mismatch at {i}: {} vs {}",
                input[i],
                restored[i]
            );
        }
    }

    #[test]
    fn different_seeds_produce_different_rotations() {
        let d = 128;
        let n = 5;
        let original = generate_random_vectors(n, d, -1.0, 1.0, 42);
        let p1 = ADSamplingPruner::new(d, 2.1, 42);
        let p2 = ADSamplingPruner::new(d, 2.1, 123);
        let mut r1 = vec![0.0_f32; n * d];
        let mut r2 = vec![0.0_f32; n * d];
        p1.rotate(&original, &mut r1, n);
        p2.rotate(&original, &mut r2, n);
        let any_diff = r1.iter().zip(&r2).any(|(a, b)| (a - b).abs() > 1e-6);
        assert!(
            any_diff,
            "different seeds should produce different rotations"
        );
    }

    #[test]
    fn same_seed_produces_identical_rotations() {
        let d = 128;
        let n = 5;
        let original = generate_random_vectors(n, d, -1.0, 1.0, 42);
        let p1 = ADSamplingPruner::new(d, 2.1, 42);
        let p2 = ADSamplingPruner::new(d, 2.1, 42);
        let mut r1 = vec![0.0_f32; n * d];
        let mut r2 = vec![0.0_f32; n * d];
        p1.rotate(&original, &mut r1, n);
        p2.rotate(&original, &mut r2, n);
        for i in 0..n * d {
            assert_eq!(
                r1[i], r2[i],
                "same seed should produce bit-identical rotations at {i}"
            );
        }
    }
}
