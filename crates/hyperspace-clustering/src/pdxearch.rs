//! Progressive ADSampling-based top-1 nearest-neighbour search.
//!
//! Given a batch of partial (GEMM) distances over the first `partial_d`
//! dimensions, this routine adds the remaining dimensions in two phases —
//! horizontal-d in blocks of `H_DIM_SIZE`, then the trailing vertical-d slab
//! — pruning aggressively against the running best with the precomputed
//! pruner ratios.

use crate::adsampling::ADSamplingPruner;
use crate::common::{KnnCandidate, H_DIM_SIZE};
use crate::distance::l2_squared;

use std::mem::MaybeUninit;

/// Fill `positions` with the indices `i` where `distances[i] < threshold`,
/// returning the count. Mirrors the C++ `InitPositionsArray`.
fn collect_survivors(distances: &[f32], threshold: f32, positions: &mut Vec<u32>) -> usize {
    positions.clear();
    positions.reserve(distances.len());
    let count = fill_survivors(distances, threshold, positions.spare_capacity_mut());
    // SAFETY: `fill_survivors` initialized elements [0, count).
    unsafe { positions.set_len(count) };
    count
}

/// Branch-free compress: unconditionally write each index, advance the cursor
/// only when the candidate survives. Avoids the data-dependent `push` (and its
/// branch misprediction in the ~99%-pruned case) and lets the compare vectorize.
/// Portable fallback, and the tail handler for the SIMD paths below.
#[allow(dead_code)]
fn fill_survivors_scalar(distances: &[f32], threshold: f32, out: &mut [MaybeUninit<u32>]) -> usize {
    let mut count = 0usize;
    for (i, &dist) in distances.iter().enumerate() {
        // SAFETY: count <= i < distances.len() <= out.len().
        unsafe { out.get_unchecked_mut(count).write(i as u32) };
        count += (dist < threshold) as usize;
    }
    count
}

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
fn fill_survivors(distances: &[f32], threshold: f32, out: &mut [MaybeUninit<u32>]) -> usize {
    fill_survivors_scalar(distances, threshold, out)
}

/// x86-64: runtime-dispatch to the widest available SIMD compress, mirroring the
/// C++ arch selection (AVX512 → AVX2 → scalar).
///
/// NOTE: written to match the C++ AVX2/AVX512 `InitPositionsArray` but NOT yet
/// validated on x86 hardware — needs a Linux-x86 CI run to confirm recall parity
/// and the speedup before relying on it. The aarch64 NEON path (below) is the
/// one exercised here. The scalar fallback stays correct on any x86 CPU.
#[cfg(target_arch = "x86_64")]
fn fill_survivors(distances: &[f32], threshold: f32, out: &mut [MaybeUninit<u32>]) -> usize {
    if is_x86_feature_detected!("avx512f") {
        // SAFETY: guarded by runtime feature detection.
        unsafe { fill_survivors_avx512(distances, threshold, out) }
    } else if is_x86_feature_detected!("avx2") {
        // SAFETY: guarded by runtime feature detection.
        unsafe { fill_survivors_avx2(distances, threshold, out) }
    } else {
        fill_survivors_scalar(distances, threshold, out)
    }
}

/// AVX2 (8-wide): movemask the compare, and for groups with any survivor do the
/// branch-free collect. Matches the C++ AVX2 `InitPositionsArray`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn fill_survivors_avx2(
    distances: &[f32],
    threshold: f32,
    out: &mut [MaybeUninit<u32>],
) -> usize {
    use std::arch::x86_64::*;
    // SAFETY: caller ensures AVX2; count <= i < n <= out.len() throughout.
    unsafe {
        let n = distances.len();
        let simd_n = n & !7;
        let mut count = 0usize;
        let thr = _mm256_set1_ps(threshold);
        let mut i = 0;
        while i < simd_n {
            let cmp = _mm256_cmp_ps::<_CMP_LT_OQ>(_mm256_loadu_ps(distances.as_ptr().add(i)), thr);
            let mask = _mm256_movemask_ps(cmp);
            if mask != 0 {
                for k in 0..8 {
                    out.get_unchecked_mut(count).write((i + k) as u32);
                    count += ((mask >> k) & 1) as usize;
                }
            }
            i += 8;
        }
        while i < n {
            out.get_unchecked_mut(count).write(i as u32);
            count += (*distances.get_unchecked(i) < threshold) as usize;
            i += 1;
        }
        count
    }
}

/// AVX512 (16-wide): compare to a mask register and use the hardware
/// compress-store (`vpcompressd`) to write only the surviving indices. Matches
/// the C++ AVX512 `InitPositionsArray`.
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn fill_survivors_avx512(
    distances: &[f32],
    threshold: f32,
    out: &mut [MaybeUninit<u32>],
) -> usize {
    use std::arch::x86_64::*;
    // SAFETY: caller ensures AVX512F; count + popcount(mask) <= n <= out.len().
    unsafe {
        let n = distances.len();
        let simd_n = n & !15;
        let mut count = 0usize;
        let thr = _mm512_set1_ps(threshold);
        let iota = _mm512_set_epi32(15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0);
        let mut i = 0;
        while i < simd_n {
            let mask =
                _mm512_cmp_ps_mask::<_CMP_LT_OQ>(_mm512_loadu_ps(distances.as_ptr().add(i)), thr);
            if mask != 0 {
                let indices = _mm512_add_epi32(_mm512_set1_epi32(i as i32), iota);
                _mm512_mask_compressstoreu_epi32(out.as_mut_ptr().add(count).cast(), mask, indices);
                count += mask.count_ones() as usize;
            }
            i += 16;
        }
        while i < n {
            out.get_unchecked_mut(count).write(i as u32);
            count += (*distances.get_unchecked(i) < threshold) as usize;
            i += 1;
        }
        count
    }
}

/// NEON variant (matches the C++ NEON `InitPositionsArray`): compare 4 lanes at
/// a time and skip the branch-free collect entirely for groups where none pass
/// — the overwhelmingly common case, since pruning keeps well under 1%.
#[cfg(target_arch = "aarch64")]
fn fill_survivors(distances: &[f32], threshold: f32, out: &mut [MaybeUninit<u32>]) -> usize {
    use std::arch::aarch64::*;
    let n = distances.len();
    let simd_n = n & !3;
    let mut count = 0usize;
    // SAFETY: NEON is baseline on aarch64; all indices below stay < n <= out.len().
    unsafe {
        let thr = vdupq_n_f32(threshold);
        let mut i = 0;
        while i < simd_n {
            let cmp = vcltq_f32(vld1q_f32(distances.as_ptr().add(i)), thr);
            if vmaxvq_u32(cmp) != 0 {
                let mut mask = [0u32; 4];
                vst1q_u32(mask.as_mut_ptr(), cmp);
                for (k, &m) in mask.iter().enumerate() {
                    out.get_unchecked_mut(count).write((i + k) as u32);
                    count += (m != 0) as usize;
                }
            }
            i += 4;
        }
        while i < n {
            out.get_unchecked_mut(count).write(i as u32);
            count += (*distances.get_unchecked(i) < threshold) as usize;
            i += 1;
        }
    }
    count
}

/// Top-1 search for a single query against `batch_n_y` centroids stored
/// row-major in `centroids[(j..j+batch_n_y) * d]` (slice passed at the right
/// offset). `partial_distances` already holds the GEMM-derived partial L2
/// distances for the first `partial_d` dimensions; this function refines
/// those entries in place.
///
/// Returns `(best_candidate, initial_not_pruned)`.
pub fn top1_partial_search(
    pruner: &ADSamplingPruner,
    query: &[f32],     // length d (already rotated)
    centroids: &[f32], // batch_n_y × d, row-major
    batch_n_y: usize,
    d: usize,
    vertical_d: usize,
    horizontal_d: usize,
    partial_distances: &mut [f32], // length batch_n_y
    partial_d: usize,
    prev_top_1: u32,
    prev_threshold: f32,
    centroid_global_offset: u32,
    positions_buf: &mut Vec<u32>,
) -> (KnnCandidate, usize) {
    let ratios = &pruner.ratios;
    let mut top = KnnCandidate::new(prev_top_1, prev_threshold);

    let init_threshold = top.distance * ratios[partial_d];
    let initial_not_pruned = collect_survivors(
        &partial_distances[..batch_n_y],
        init_threshold,
        positions_buf,
    );

    // Early exit: only candidate is the previous best.
    if positions_buf.len() == 1 {
        let p = positions_buf[0];
        if centroid_global_offset + p == prev_top_1 {
            positions_buf.clear();
            return (top, initial_not_pruned);
        }
    }

    let mut current_dim = partial_d;
    let mut current_horizontal = 0;

    // Horizontal phase — H_DIM_SIZE columns at a time, starting at offset vertical_d.
    while !positions_buf.is_empty() && current_horizontal < horizontal_d {
        let block_size = H_DIM_SIZE.min(horizontal_d - current_horizontal);
        let offset = vertical_d + current_horizontal;
        let q_slice = &query[offset..offset + block_size];

        for &v_idx in positions_buf.iter() {
            let i = v_idx as usize;
            let row = &centroids[i * d + offset..i * d + offset + block_size];
            partial_distances[i] += l2_squared(q_slice, row);
        }

        current_horizontal += block_size;
        current_dim += block_size;
        let threshold = top.distance * ratios[current_dim];
        positions_buf.retain(|&v_idx| partial_distances[v_idx as usize] < threshold);
    }

    // Trailing vertical-d slab — columns [partial_d, vertical_d).
    if !positions_buf.is_empty() && partial_d < vertical_d {
        let dims_left = vertical_d - partial_d;
        let offset = partial_d;
        let q_slice = &query[offset..offset + dims_left];

        for &v_idx in positions_buf.iter() {
            let i = v_idx as usize;
            let row = &centroids[i * d + offset..i * d + offset + dims_left];
            partial_distances[i] += l2_squared(q_slice, row);
        }

        current_dim = d;
        let threshold = top.distance * ratios[current_dim];
        positions_buf.retain(|&v_idx| partial_distances[v_idx as usize] < threshold);
    }

    // Final scan over remaining candidates.
    for &v_idx in positions_buf.iter() {
        let i = v_idx as usize;
        let dist = partial_distances[i];
        if dist < top.distance {
            top.distance = dist;
            top.index = centroid_global_offset + v_idx;
        }
    }

    positions_buf.clear();
    (top, initial_not_pruned)
}
