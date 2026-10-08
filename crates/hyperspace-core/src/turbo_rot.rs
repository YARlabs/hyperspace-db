//! Fast Walsh-Hadamard Transform (FWHT) Randomized Vector Rotation.
//!
//! Replaces naive O(D^2) dense matrix rotation with O(D log D) randomized
//! Walsh-Hadamard butterfly transformation.
//!
//! Key properties:
//! 1. Complexity: O(D log D) arithmetic operations (pure additions & subtractions).
//! 2. Memory: Zero dense matrix storage (O(1) memory vs O(D^2) in RAM).
//! 3. Orthogonality: Exactly norm-preserving (||Rx|| = ||x||) and isometry-preserving (<Rx, Ry> = <x, y>).
//! 4. Invertibility: Exact O(D log D) inverse transformation for vector reconstruction.
//! 5. Arbitrary D: Decomposes arbitrary dimensions D into greedy powers-of-two orthogonal blocks.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// Precomputed configuration for a given dimension:
/// - `blocks`: list of (offset, block_size) where each block_size is a power of 2.
/// - `scaled_signs`: sign factors s_i / sqrt(block_size) for each coordinate.
#[derive(Clone, Debug)]
pub struct TransformPlan {
    pub blocks: Vec<(usize, usize)>,
    pub scaled_signs: Arc<Vec<f32>>,
}

static PLANS: OnceLock<RwLock<HashMap<usize, TransformPlan>>> = OnceLock::new();

/// Decompose arbitrary dimension D into a sequence of disjoint power-of-two blocks.
/// e.g. 1024 -> [(0, 1024)]
/// e.g. 768  -> [(0, 512), (512, 256)]
/// e.g. 1536 -> [(0, 1024), (1024, 512)]
pub fn decompose_blocks(dim: usize) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut offset = 0;
    let mut rem = dim;
    while rem > 0 {
        let block_len = 1usize << (usize::BITS - 1 - rem.leading_zeros());
        blocks.push((offset, block_len));
        offset += block_len;
        rem -= block_len;
    }
    blocks
}

/// Generate deterministic pseudorandom signs scaled by 1 / sqrt(block_len).
fn generate_plan(dim: usize) -> TransformPlan {
    let blocks = decompose_blocks(dim);
    let mut scaled_signs = Vec::with_capacity(dim);

    for &(offset, block_len) in &blocks {
        let inv_sqrt = 1.0f32 / (block_len as f32).sqrt();
        let mut state =
            0x517cc1b727220a95u64.wrapping_add((offset as u64).wrapping_mul(0x9e3779b97f4a7c15));
        for _ in 0..block_len {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let sign = if (state >> 63) == 0 {
                inv_sqrt
            } else {
                -inv_sqrt
            };
            scaled_signs.push(sign);
        }
    }

    TransformPlan {
        blocks,
        scaled_signs: Arc::new(scaled_signs),
    }
}

fn get_or_create_plan(dim: usize) -> TransformPlan {
    let rwlock = PLANS.get_or_init(|| RwLock::new(HashMap::new()));
    {
        let map = rwlock.read().unwrap();
        if let Some(plan) = map.get(&dim) {
            return plan.clone();
        }
    }
    let mut map = rwlock.write().unwrap();
    map.entry(dim).or_insert_with(|| generate_plan(dim)).clone()
}

/// In-place Fast Walsh-Hadamard Transform butterfly on a power-of-two slice.
/// Computes H_N * v in O(N log N) using SIMD-vectorizable additions and subtractions.
#[inline]
pub fn fwht_in_place(buf: &mut [f32]) {
    let n = buf.len();
    if n <= 1 {
        return;
    }
    debug_assert!(n.is_power_of_two(), "Slice length must be a power of two");

    let mut len = 1;
    while len < n {
        let step = len * 2;
        for chunk in buf.chunks_exact_mut(step) {
            let (u, v) = chunk.split_at_mut(len);
            for (a, b) in u.iter_mut().zip(v.iter_mut()) {
                let x = *a;
                let y = *b;
                *a = x + y;
                *b = x - y;
            }
        }
        len = step;
    }
}

/// Rotates a vector using Fast Walsh-Hadamard Transform:
/// y = FWHT(x * scaled_signs)
/// Runs in O(D log D) time and O(1) auxiliary memory.
pub fn rotate_vector(vec: &[f32], dim: usize) -> Vec<f32> {
    if vec.len() != dim {
        return vec.to_vec();
    }
    let mut res = vec.to_vec();
    rotate_vector_in_place(&mut res);
    res
}

/// In-place FWHT rotation on a mutable slice of length D.
pub fn rotate_vector_in_place(vec: &mut [f32]) {
    let dim = vec.len();
    if dim <= 1 {
        return;
    }
    let plan = get_or_create_plan(dim);
    let signs = &plan.scaled_signs;

    // Step 1: Pre-multiply coordinates by scaled signs: s_i / sqrt(N)
    for i in 0..dim {
        vec[i] *= signs[i];
    }

    // Step 2: Apply in-place FWHT butterfly to each power-of-two block
    for &(offset, block_len) in &plan.blocks {
        fwht_in_place(&mut vec[offset..offset + block_len]);
    }
}

/// Exact inverse transform (un-rotation) using the transpose/inverse of FWHT:
/// x = FWHT(y) * scaled_signs
/// Reconstructs original coordinates in O(D log D) time.
pub fn unrotate_vector(vec: &[f32], dim: usize) -> Vec<f32> {
    if vec.len() != dim {
        return vec.to_vec();
    }
    let mut res = vec.to_vec();
    unrotate_vector_in_place(&mut res);
    res
}

/// In-place FWHT inverse transform on a mutable slice of length D.
pub fn unrotate_vector_in_place(vec: &mut [f32]) {
    let dim = vec.len();
    if dim <= 1 {
        return;
    }
    let plan = get_or_create_plan(dim);
    let signs = &plan.scaled_signs;

    // Step 1: Apply in-place FWHT butterfly to each block
    for &(offset, block_len) in &plan.blocks {
        fwht_in_place(&mut vec[offset..offset + block_len]);
    }

    // Step 2: Multiply by scaled signs: s_i / sqrt(N)
    for i in 0..dim {
        vec[i] *= signs[i];
    }
}

#[cfg(test)]
#[allow(
    clippy::cast_precision_loss,
    clippy::uninlined_format_args,
    clippy::needless_range_loop
)]
mod tests {
    use super::*;

    #[test]
    fn test_fwht_orthogonality_and_inversion() {
        for dim in [2, 4, 8, 16, 32, 64, 128, 256, 512, 768, 1024, 1536] {
            let x: Vec<f32> = (0..dim).map(|i| ((i as f32) * 0.13).sin()).collect();
            let y: Vec<f32> = (0..dim).map(|i| ((i as f32) * 0.29).cos()).collect();

            let norm_x_orig: f32 = x.iter().map(|&v| v * v).sum();
            let norm_y_orig: f32 = y.iter().map(|&v| v * v).sum();
            let dot_orig: f32 = x.iter().zip(y.iter()).map(|(&a, &b)| a * b).sum();

            let rx = rotate_vector(&x, dim);
            let ry = rotate_vector(&y, dim);

            let norm_rx: f32 = rx.iter().map(|&v| v * v).sum();
            let norm_ry: f32 = ry.iter().map(|&v| v * v).sum();
            let dot_rot: f32 = rx.iter().zip(ry.iter()).map(|(&a, &b)| a * b).sum();

            assert!(
                (norm_x_orig - norm_rx).abs() / norm_x_orig < 1e-4,
                "Norm preservation failed for dim {}: orig={}, rot={}",
                dim,
                norm_x_orig,
                norm_rx
            );
            assert!(
                (norm_y_orig - norm_ry).abs() / norm_y_orig < 1e-4,
                "Norm preservation failed for dim {}: orig={}, rot={}",
                dim,
                norm_y_orig,
                norm_ry
            );
            assert!(
                (dot_orig - dot_rot).abs() / (norm_x_orig * norm_y_orig).sqrt() < 1e-4,
                "Dot product preservation failed for dim {}: orig={}, rot={}",
                dim,
                dot_orig,
                dot_rot
            );

            // Test exact inversion
            let x_inv = unrotate_vector(&rx, dim);
            for i in 0..dim {
                assert!(
                    (x[i] - x_inv[i]).abs() < 1e-5,
                    "Inversion failed at idx {} for dim {}: orig={}, inv={}",
                    i,
                    dim,
                    x[i],
                    x_inv[i]
                );
            }
        }
    }
}
