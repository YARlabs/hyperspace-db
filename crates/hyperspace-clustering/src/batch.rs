//! Batched nearest-neighbour search built on top of SGEMM and the pruning kernel.

use rayon::prelude::*;

use crate::adsampling::ADSamplingPruner;
use crate::common::{X_BATCH_SIZE, Y_BATCH_SIZE};
// MINI_BATCH_SIZE only exists (and is only used) on macOS, where the partial
// GEMM is mini-batched across threads for the AMX units.
#[cfg(target_os = "macos")]
use crate::common::MINI_BATCH_SIZE;
use crate::distance::l2_squared;
use crate::pdxearch::top1_partial_search;

/// Compute `out[i, j] = dot(x[i, :effective_d], y[j, :effective_d])`.
fn blas_dot_products(
    x_batch: &[f32], // batch_n_x × d row-major
    y_batch: &[f32], // batch_n_y × d row-major
    batch_n_x: usize,
    batch_n_y: usize,
    d: usize,
    effective_d: usize,
    out: &mut [f32], // batch_n_x × batch_n_y row-major
) {
    debug_assert!(x_batch.len() >= batch_n_x * d);
    debug_assert!(y_batch.len() >= batch_n_y * d);
    debug_assert!(out.len() >= batch_n_x * batch_n_y);
    // out = x[:, :effective_d] * y[:, :effective_d]^T ; both rows are d-wide.
    let gemm = |rows: usize, x: &[f32], c: &mut [f32]| {
        crate::gemm::sgemm_ld(
            false,
            true,
            rows,
            effective_d,
            batch_n_y,
            x,
            d,
            y_batch,
            d,
            c,
            batch_n_y,
        );
    };

    // On Apple Silicon each thread drives its own AMX unit, so dispatching the
    // (thin-K) partial GEMM as mini-batches across threads beats one big
    // `cblas_sgemm` — matching the C++ pruning kernel's Apple strategy. On other
    // targets a single BLAS call (internally threaded) is best.
    #[cfg(target_os = "macos")]
    out.par_chunks_mut(MINI_BATCH_SIZE * batch_n_y)
        .enumerate()
        .for_each(|(mb, chunk)| {
            let r = mb * MINI_BATCH_SIZE;
            let rows = MINI_BATCH_SIZE.min(batch_n_x - r);
            gemm(rows, &x_batch[r * d..(r + rows) * d], chunk);
        });

    #[cfg(not(target_os = "macos"))]
    gemm(batch_n_x, x_batch, out);
}

/// Grow `buf` so one SGEMM batch (up to `X_BATCH_SIZE × Y_BATCH_SIZE`, clamped
/// to the problem size) fits. The buffer is caller-owned and reused across
/// calls, so this avoids re-allocating (and page-faulting) it every iteration.
/// The GEMM writes the used region with beta=0, so stale contents are fine.
fn ensure_buf(buf: &mut Vec<f32>, n_x: usize, n_y: usize) {
    let required = X_BATCH_SIZE.min(n_x) * Y_BATCH_SIZE.min(n_y);
    if buf.len() < required {
        buf.resize(required, 0.0);
    }
}

/// Brute-force top-1 nearest-centroid search via batched SGEMM.
pub fn find_nearest_neighbor(
    x: &[f32],
    y: &[f32],
    n_x: usize,
    n_y: usize,
    d: usize,
    norms_x: &[f32],
    norms_y: &[f32],
    out_knn: &mut [u32],
    out_distances: &mut [f32],
    buf: &mut Vec<f32>,
) {
    for v in out_distances.iter_mut().take(n_x) {
        *v = f32::MAX;
    }
    for v in out_knn.iter_mut().take(n_x) {
        *v = 0;
    }

    ensure_buf(buf, n_x, n_y);

    let mut i = 0;
    while i < n_x {
        let batch_n_x = X_BATCH_SIZE.min(n_x - i);
        let batch_x_p = &x[i * d..(i + batch_n_x) * d];

        let mut j = 0;
        while j < n_y {
            let batch_n_y = Y_BATCH_SIZE.min(n_y - j);
            let batch_y_p = &y[j * d..(j + batch_n_y) * d];

            blas_dot_products(
                batch_x_p,
                batch_y_p,
                batch_n_x,
                batch_n_y,
                d,
                d,
                &mut buf[..batch_n_x * batch_n_y],
            );

            let dist_window = &mut out_distances[i..i + batch_n_x];
            let knn_window = &mut out_knn[i..i + batch_n_x];
            let nx = norms_x;
            let ny = norms_y;
            let j_off = j;

            buf[..batch_n_x * batch_n_y]
                .par_chunks_mut(batch_n_y)
                .zip(dist_window.par_iter_mut())
                .zip(knn_window.par_iter_mut())
                .enumerate()
                .for_each(|(r, ((row, best_d), best_idx))| {
                    let norm_x_i = nx[i + r];
                    let mut min_v = f32::MAX;
                    let mut min_c: u32 = 0;
                    for c in 0..batch_n_y {
                        let v = -2.0 * row[c] + norm_x_i + ny[j_off + c];
                        row[c] = v;
                        if v < min_v {
                            min_v = v;
                            min_c = c as u32;
                        }
                    }
                    if min_v < *best_d {
                        *best_d = min_v.max(0.0);
                        *best_idx = j_off as u32 + min_c;
                    }
                });

            j += batch_n_y;
        }
        i += batch_n_x;
    }
}

/// k-nearest neighbours brute-force via batched SGEMM.
pub fn find_k_nearest_neighbors(
    x: &[f32],
    y: &[f32],
    n_x: usize,
    n_y: usize,
    d: usize,
    norms_x: &[f32],
    norms_y: &[f32],
    k: usize,
    out_knn: &mut [u32],       // n_x × k
    out_distances: &mut [f32], // n_x × k
) {
    for v in out_distances.iter_mut() {
        *v = f32::MAX;
    }
    for v in out_knn.iter_mut() {
        *v = u32::MAX;
    }

    let mut buf = vec![0.0_f32; X_BATCH_SIZE * Y_BATCH_SIZE];

    let mut i = 0;
    while i < n_x {
        let batch_n_x = X_BATCH_SIZE.min(n_x - i);
        let batch_x_p = &x[i * d..(i + batch_n_x) * d];

        let mut j = 0;
        while j < n_y {
            let batch_n_y = Y_BATCH_SIZE.min(n_y - j);
            let batch_y_p = &y[j * d..(j + batch_n_y) * d];

            blas_dot_products(
                batch_x_p,
                batch_y_p,
                batch_n_x,
                batch_n_y,
                d,
                d,
                &mut buf[..batch_n_x * batch_n_y],
            );

            // Per row: convert dot to L2 then merge into the current top-k for that query.
            // We split out_distances/out_knn into row-sized chunks for safe parallelism.
            let i_base = i;
            let j_base = j;
            let bn_y = batch_n_y;
            let nx = norms_x;
            let ny = norms_y;

            let buf_slice = &mut buf[..batch_n_x * bn_y];
            let dist_window = &mut out_distances[i_base * k..(i_base + batch_n_x) * k];
            let knn_window = &mut out_knn[i_base * k..(i_base + batch_n_x) * k];

            buf_slice
                .par_chunks_mut(bn_y)
                .zip(dist_window.par_chunks_mut(k))
                .zip(knn_window.par_chunks_mut(k))
                .enumerate()
                .for_each(|(r, ((row, dist_row), knn_row))| {
                    let norm_x_i = nx[i_base + r];
                    for c in 0..bn_y {
                        row[c] = -2.0 * row[c] + norm_x_i + ny[j_base + c];
                    }

                    let mut candidates: Vec<(f32, u32)> = Vec::with_capacity(k + bn_y);
                    for ki in 0..k {
                        if dist_row[ki] < f32::MAX {
                            candidates.push((dist_row[ki], knn_row[ki]));
                        }
                    }
                    for c in 0..bn_y {
                        candidates.push((row[c], (j_base + c) as u32));
                    }
                    let actual_k = k.min(candidates.len());
                    candidates
                        .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
                    for ki in 0..actual_k {
                        dist_row[ki] = candidates[ki].0.max(0.0);
                        knn_row[ki] = candidates[ki].1;
                    }
                    for ki in actual_k..k {
                        dist_row[ki] = f32::MAX;
                        knn_row[ki] = u32::MAX;
                    }
                });

            j += batch_n_y;
        }
        i += batch_n_x;
    }
}

/// GEMM + ADSampling/PDX pruning. `norms_x_partial` and `norms_y_partial` are
/// squared L2 norms restricted to the first `partial_d` dimensions.
/// `out_knn` / `out_distances` come in seeded from the previous iteration and
/// are refined in place.
pub fn find_nearest_neighbor_with_pruning(
    x: &[f32],
    y: &[f32],
    n_x: usize,
    n_y: usize,
    d: usize,
    vertical_d: usize,
    horizontal_d: usize,
    norms_x_partial: &[f32],
    norms_y_partial: &[f32],
    out_knn: &mut [u32],
    out_distances: &mut [f32],
    pruner: &ADSamplingPruner,
    partial_d: usize,
    out_not_pruned_counts: &mut [usize],
    buf: &mut Vec<f32>,
) {
    ensure_buf(buf, n_x, n_y);

    let mut i = 0;
    while i < n_x {
        let batch_n_x = X_BATCH_SIZE.min(n_x - i);
        let batch_x_p = &x[i * d..(i + batch_n_x) * d];

        let mut j = 0;
        while j < n_y {
            let batch_n_y = Y_BATCH_SIZE.min(n_y - j);
            let batch_y_p = &y[j * d..(j + batch_n_y) * d];

            blas_dot_products(
                batch_x_p,
                batch_y_p,
                batch_n_x,
                batch_n_y,
                d,
                partial_d,
                &mut buf[..batch_n_x * batch_n_y],
            );

            let i_base = i;
            let j_base = j;
            let bn_y = batch_n_y;
            let nx = norms_x_partial;
            let ny = norms_y_partial;

            let buf_slice = &mut buf[..batch_n_x * bn_y];
            let knn_window = &mut out_knn[i_base..i_base + batch_n_x];
            let dist_window = &mut out_distances[i_base..i_base + batch_n_x];
            let npc_window = &mut out_not_pruned_counts[i_base..i_base + batch_n_x];

            buf_slice
                .par_chunks_mut(bn_y)
                .zip(knn_window.par_iter_mut())
                .zip(dist_window.par_iter_mut())
                .zip(npc_window.par_iter_mut())
                .enumerate()
                // Reuse one `positions` scratch buffer per worker thread instead
                // of allocating a Vec per row; top1_partial_search clears it.
                .for_each_init(
                    || Vec::<u32>::with_capacity(bn_y),
                    |positions, (r, (((row, knn), dist), npc))| {
                        let i_idx = i_base + r;
                        let norm_x_i = nx[i_idx];
                        // Convert raw dot products to partial L2 distances. Iterate
                        // over zipped slices (no bounds checks, no aliasing) so the
                        // autovectorizer emits a SIMD FMA loop — the Rust equivalent
                        // of the C++ `SKM_VECTORIZE_LOOP` conversion pass.
                        for (rv, &ny_c) in row.iter_mut().zip(&ny[j_base..j_base + bn_y]) {
                            *rv = norm_x_i + ny_c - 2.0 * *rv;
                        }
                        let data_p = &x[i_idx * d..i_idx * d + d];
                        let prev_assignment = *knn;
                        let dist_to_prev = if j_base == 0 {
                            let other_start = prev_assignment as usize * d;
                            l2_squared(&y[other_start..other_start + d], data_p)
                        } else {
                            *dist
                        };

                        let (assignment, local_not_pruned) = top1_partial_search(
                            pruner,
                            data_p,
                            batch_y_p,
                            bn_y,
                            d,
                            vertical_d,
                            horizontal_d,
                            row,
                            partial_d,
                            prev_assignment,
                            dist_to_prev,
                            j_base as u32,
                            positions,
                        );
                        *npc += local_not_pruned;
                        *knn = assignment.index;
                        *dist = assignment.distance.max(0.0);
                    },
                );

            j += batch_n_y;
        }
        i += batch_n_x;
    }
}
