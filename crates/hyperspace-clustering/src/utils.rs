//! Timing, blob data generation, brute-force reference routines, and small
//! shared numeric kernels.

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal, Uniform};
use rayon::prelude::*;
use std::time::Instant;

/// Simple stopwatch matching the C++ TicToc API.
pub struct TicToc {
    accum_ns: u128,
    start: Instant,
}

impl Default for TicToc {
    fn default() -> Self {
        Self::new()
    }
}

impl TicToc {
    pub fn new() -> Self {
        Self {
            accum_ns: 0,
            start: Instant::now(),
        }
    }

    pub fn reset(&mut self) {
        self.accum_ns = 0;
        self.start = Instant::now();
    }

    pub fn tic(&mut self) {
        self.start = Instant::now();
    }

    pub fn toc(&mut self) {
        self.accum_ns += self.start.elapsed().as_nanos();
    }

    pub fn milliseconds(&self) -> f64 {
        self.accum_ns as f64 / 1.0e6
    }
}

/// Generate synthetic clusterable data (scikit-learn-style make_blobs).
pub fn make_blobs(
    n_samples: usize,
    n_features: usize,
    n_centers: usize,
    normalize: bool,
    cluster_std: f32,
    center_spread: f32,
    random_state: u64,
) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(random_state);
    let center_dist = Normal::new(0.0_f32, center_spread).unwrap();

    let mut centers = vec![0.0_f32; n_centers * n_features];
    for value in centers.iter_mut() {
        *value = center_dist.sample(&mut rng);
    }

    let mut data = vec![0.0_f32; n_samples * n_features];
    let point_dist = Normal::new(0.0_f32, cluster_std).unwrap();
    let cluster_dist = Uniform::new(0_usize, n_centers);

    data.par_chunks_mut(n_features)
        .enumerate()
        .for_each(|(i, row)| {
            let mut local_rng = ChaCha8Rng::seed_from_u64(random_state.wrapping_add(i as u64 + 1));
            let center_idx = cluster_dist.sample(&mut local_rng) * n_features;
            let center = &centers[center_idx..center_idx + n_features];
            for j in 0..n_features {
                row[j] = center[j] + point_dist.sample(&mut local_rng);
            }
        });

    if normalize {
        data.par_chunks_mut(n_features).for_each(|row| {
            let mut norm_sq = 0.0_f32;
            for &v in row.iter() {
                norm_sq += v * v;
            }
            let inv = 1.0 / norm_sq.sqrt().max(f32::EPSILON);
            for v in row.iter_mut() {
                *v *= inv;
            }
        });
    }

    data
}

pub fn generate_random_vectors(
    n: usize,
    d: usize,
    min_val: f32,
    max_val: f32,
    seed: u64,
) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let dist = Uniform::new(min_val, max_val);
    let mut output = vec![0.0_f32; n * d];
    for v in output.iter_mut() {
        *v = dist.sample(&mut rng);
    }
    output
}

pub fn ceil_to_multiple(x: u32, m: u32) -> u32 {
    if m == 0 {
        x
    } else {
        x.div_ceil(m) * m
    }
}

pub fn is_power_of_two(x: u32) -> bool {
    x > 0 && (x & (x - 1)) == 0
}

/// Reference brute-force search used in tests and for validation.
pub fn compute_l2_squared(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len());
    let mut s = 0.0_f32;
    for i in 0..a.len() {
        let diff = a[i] - b[i];
        s += diff * diff;
    }
    s
}

/// Squared L2 norm of each of the first `n_vectors` rows of an `n_vectors × d`
/// row-major matrix.
///
/// These are the `‖x‖²` terms of the expansion `‖x − y‖² = ‖x‖² − 2x·y + ‖y‖²`,
/// which is how the GEMM distance kernels avoid materializing differences: given
/// both norm vectors, a single matrix product supplies every `x·y`. Squared, not
/// rooted, because that is the form the expansion needs.
pub fn squared_norms(vectors: &[f32], n_vectors: usize, d: usize) -> Vec<f32> {
    squared_norms_partial(vectors, n_vectors, d, d)
}

/// [`squared_norms`] over just the leading `partial_d` dimensions of each row.
///
/// A prefix norm bounds the full one from below, which is what lets ADSampling
/// reject a candidate after inspecting `partial_d` of its `d` dimensions. Values
/// of `partial_d` above `d` are clamped.
pub fn squared_norms_partial(
    vectors: &[f32],
    n_vectors: usize,
    d: usize,
    partial_d: usize,
) -> Vec<f32> {
    if n_vectors * partial_d.min(d) < NORM_PARALLEL_MIN_ELEMENTS {
        squared_norms_partial_sequential(vectors, n_vectors, d, partial_d)
    } else {
        squared_norms_partial_parallel(vectors, n_vectors, d, partial_d)
    }
}

/// Single-threaded [`squared_norms_partial`].
pub fn squared_norms_partial_sequential(
    vectors: &[f32],
    n_vectors: usize,
    d: usize,
    partial_d: usize,
) -> Vec<f32> {
    debug_assert!(vectors.len() >= n_vectors * d);
    let dims = partial_d.min(d);
    let mut norms = vec![0.0_f32; n_vectors];
    vectors
        .chunks_exact(d)
        .zip(norms.iter_mut())
        .for_each(|(vector, norm)| *norm = squared_norm(&vector[..dims]));
    norms
}

/// One rayon work item per vector [`squared_norms_partial`].
pub fn squared_norms_partial_parallel(
    vectors: &[f32],
    n_vectors: usize,
    d: usize,
    partial_d: usize,
) -> Vec<f32> {
    debug_assert!(vectors.len() >= n_vectors * d);
    let dims = partial_d.min(d);
    let mut norms = vec![0.0_f32; n_vectors];
    vectors
        .par_chunks_exact(d)
        .zip(norms.par_iter_mut())
        .for_each(|(vector, norm)| *norm = squared_norm(&vector[..dims]));
    norms
}

#[inline]
fn squared_norm(vector: &[f32]) -> f32 {
    let mut sum = 0.0_f32;
    for &v in vector {
        sum += v * v;
    }
    sum
}

/// Matrix size below which a sequential pass beats spreading the rows over rayon,
/// for the kernels that *reduce* each row.
///
/// `cargo bench --bench utils` puts rayon's fork-join at 13-15 µs when entered
/// from a non-worker thread, which is what training does, while a sequential row
/// reduction holds a steady ~2.2 Gelem/s at every size. Parallelism therefore only
/// pays once a matrix is large enough to cover that fixed cost, measured at
/// somewhere between 49K and 98K elements.
///
/// Calibrated for those non-worker calls. Crossing into the pool once at the top
/// of training instead would move the crossover down.
const REDUCTION_PARALLEL_MIN_ELEMENTS: usize = 64 * 1024;

/// The same crossover for the per-vector squared norm, at twice the bar the other
/// row reductions clear.
///
/// It reads one matrix where [`centroid_shift`] walks two, so an element costs
/// half the bandwidth and sequential throughput runs at 2-5.5 Gelem/s depending on
/// the row length. A given element count therefore buys less time to cover the
/// dispatch with. `cargo bench --bench utils` has the parallel form still behind
/// at 64K elements summed (0.2-0.4x) and ahead from 128K on (1.2-1.6x), with only
/// one shape in between — 64 rows of 1536 — worth parallelizing.
///
/// Measured against the elements *summed*, `n_vectors * partial_d`, not the
/// footprint walked: prefix norms stride over a matrix several times that size
/// and still track the summed count.
const NORM_PARALLEL_MIN_ELEMENTS: usize = 128 * 1024;

/// The same crossover for kernels that only *scale* each element, which sits 16x
/// higher.
///
/// Two effects compound. A scale runs sequentially at ~17 Gelem/s, so a given
/// element count buys far less time to cover the dispatch with; and it is memory
/// bound, so spreading it over rayon returns ~1.5x rather than the ~5x a reduction
/// gets. Recovering 13 µs out of a 1.5x saving takes ~800K elements, which is
/// where the measured crossover sits.
const SCALE_PARALLEL_MIN_ELEMENTS: usize = 1024 * 1024;

/// Input size below which a sequential pass wins for the per-cluster scatter,
/// which unlike the row-wise kernels reads a matrix far larger than it writes.
///
/// Its work scales with the rows it reads, so this is the term that has to cover
/// rayon's dispatch. `cargo bench --bench utils` puts the crossover between 256K
/// and 512K elements read: at 192K the parallel form still loses (0.6-0.7x), at
/// 384K it is within noise (1.0-1.2x), and by 768K it is clearly ahead
/// (1.4-1.7x).
const SCATTER_PARALLEL_MIN_INPUT_ELEMENTS: usize = 512 * 1024;

/// Output size below which a sequential pass wins for the per-cluster scatter,
/// which splits its *output* over rayon rather than its input.
///
/// A band of clusters is the unit of parallel work, so a small `n_clusters × d`
/// leaves nothing worth dividing however much data is read: measured at 12K
/// elements the parallel form never gets ahead (0.9-1.2x) even over 8M elements
/// of input, while at 16K and up it holds 1.1-2.0x. Both ends reproduce at
/// d=128 and d=768, so the product is what matters, not either factor.
const SCATTER_PARALLEL_MIN_OUTPUT_ELEMENTS: usize = 16 * 1024;

/// Total squared distance the centroids moved this iteration, summed over the
/// first `n_clusters` rows of two `n_clusters × d` row-major matrices.
///
/// This is k-means' convergence signal: callers compare it against their
/// tolerance to decide whether to stop.
///
/// Dispatches on matrix size between [`centroid_shift_sequential`] and
/// [`centroid_shift_parallel`], which agree only to within f32 reduction order.
/// Both are public so benchmarks can measure where the crossover lies.
pub fn centroid_shift(
    new_centroids: &[f32],
    prev_centroids: &[f32],
    n_clusters: usize,
    d: usize,
) -> f32 {
    if n_clusters * d < REDUCTION_PARALLEL_MIN_ELEMENTS {
        centroid_shift_sequential(new_centroids, prev_centroids, n_clusters, d)
    } else {
        centroid_shift_parallel(new_centroids, prev_centroids, n_clusters, d)
    }
}

/// Single-threaded [`centroid_shift`].
pub fn centroid_shift_sequential(
    new_centroids: &[f32],
    prev_centroids: &[f32],
    n_clusters: usize,
    d: usize,
) -> f32 {
    new_centroids
        .chunks(d)
        .zip(prev_centroids.chunks(d))
        .take(n_clusters)
        .map(|(new_row, prev_row)| row_shift(new_row, prev_row))
        .sum()
}

/// One rayon work item per centroid [`centroid_shift`].
pub fn centroid_shift_parallel(
    new_centroids: &[f32],
    prev_centroids: &[f32],
    n_clusters: usize,
    d: usize,
) -> f32 {
    new_centroids
        .par_chunks(d)
        .zip(prev_centroids.par_chunks(d))
        .take(n_clusters)
        .map(|(new_row, prev_row)| row_shift(new_row, prev_row))
        .sum()
}

/// Total the vectors assigned to each cluster, and count how many landed there.
///
/// This is k-means' M-step, and it stops one step short of a centroid: each row
/// of `centroids` comes back holding its cluster's *sum*, not its mean, since the
/// count to divide by is only known once the pass is over. [`mean_rows_by_count`]
/// finishes the job.
///
/// `assignments[i]` names the cluster owning vector `i`, so the vector count
/// comes from `assignments` and the cluster count from `cluster_sizes`. Both
/// outputs are overwritten rather than accumulated into, and any row of
/// `centroids` past `cluster_sizes.len()` is left alone. A vector assigned
/// outside that range is skipped.
///
/// # Panics
///
/// If `centroids` is shorter than `cluster_sizes.len() * d`.
pub fn sum_rows_by_assignment(
    vectors: &[f32],
    assignments: &[u32],
    centroids: &mut [f32],
    cluster_sizes: &mut [u32],
    d: usize,
) {
    let enough_input = assignments.len() * d >= SCATTER_PARALLEL_MIN_INPUT_ELEMENTS;
    let enough_output = cluster_sizes.len() * d >= SCATTER_PARALLEL_MIN_OUTPUT_ELEMENTS;
    if enough_input && enough_output {
        sum_rows_by_assignment_parallel(vectors, assignments, centroids, cluster_sizes, d);
    } else {
        sum_rows_by_assignment_sequential(vectors, assignments, centroids, cluster_sizes, d);
    }
}

/// Single-threaded [`sum_rows_by_assignment`]: one pass, scattering each vector
/// into whichever cluster owns it.
pub fn sum_rows_by_assignment_sequential(
    vectors: &[f32],
    assignments: &[u32],
    centroids: &mut [f32],
    cluster_sizes: &mut [u32],
    d: usize,
) {
    let n_clusters = cluster_sizes.len();
    sum_rows_in_band(
        vectors,
        assignments,
        &mut centroids[..n_clusters * d],
        cluster_sizes,
        0,
        d,
    );
}

/// One rayon work item per band of clusters [`sum_rows_by_assignment`].
pub fn sum_rows_by_assignment_parallel(
    vectors: &[f32],
    assignments: &[u32],
    centroids: &mut [f32],
    cluster_sizes: &mut [u32],
    d: usize,
) {
    let n_clusters = cluster_sizes.len();
    if n_clusters == 0 {
        return;
    }

    // Splitting the output rather than the input is what keeps the scatter safe:
    // each task owns a disjoint band of clusters, so no two tasks ever write the
    // same centroid and no private accumulators are needed. The price is that
    // every task walks the whole assignment vector to pick out the vectors it
    // owns, so the bands are made as wide as the pool allows instead of being
    // left to rayon to subdivide further.
    let per_task = n_clusters.div_ceil(rayon::current_num_threads().max(1));
    centroids[..n_clusters * d]
        .par_chunks_mut(per_task * d)
        .zip(cluster_sizes.par_chunks_mut(per_task))
        .enumerate()
        .for_each(|(task, (centroids, cluster_sizes))| {
            sum_rows_in_band(
                vectors,
                assignments,
                centroids,
                cluster_sizes,
                task * per_task,
                d,
            );
        });
}

/// Sum the vectors belonging to the `cluster_sizes.len()` clusters starting at
/// `first_cluster`, ignoring every other vector.
fn sum_rows_in_band(
    vectors: &[f32],
    assignments: &[u32],
    centroids: &mut [f32],
    cluster_sizes: &mut [u32],
    first_cluster: usize,
    d: usize,
) {
    centroids.fill(0.0);
    cluster_sizes.fill(0);

    let band = first_cluster..first_cluster + cluster_sizes.len();
    for (vector, &cluster) in vectors.chunks_exact(d).zip(assignments) {
        let cluster = cluster as usize;
        if !band.contains(&cluster) {
            continue;
        }
        let local = cluster - first_cluster;
        cluster_sizes[local] += 1;
        accumulate_vector(&mut centroids[local * d..(local + 1) * d], vector);
    }
}

#[inline]
fn accumulate_vector(centroid: &mut [f32], vector: &[f32]) {
    for (sum, &v) in centroid.iter_mut().zip(vector) {
        *sum += v;
    }
}

/// Replace each row of an `n_rows × d` row-major matrix of sums with its
/// arithmetic mean, dividing row `i` by `counts[i]`.
///
/// Rows with a zero count keep their accumulated value — there is no mean to
/// take — so a caller that cares about empty clusters has to repair them.
///
/// Dispatches on matrix size between [`mean_rows_by_count_sequential`] and
/// [`mean_rows_by_count_parallel`].
pub fn mean_rows_by_count(rows: &mut [f32], counts: &[u32], d: usize) {
    if counts.len() * d < SCALE_PARALLEL_MIN_ELEMENTS {
        mean_rows_by_count_sequential(rows, counts, d);
    } else {
        mean_rows_by_count_parallel(rows, counts, d);
    }
}

/// Single-threaded [`mean_rows_by_count`].
pub fn mean_rows_by_count_sequential(rows: &mut [f32], counts: &[u32], d: usize) {
    rows.chunks_mut(d)
        .zip(counts.iter())
        .for_each(|(row, &count)| mean_row(row, count));
}

/// One rayon work item per row [`mean_rows_by_count`].
pub fn mean_rows_by_count_parallel(rows: &mut [f32], counts: &[u32], d: usize) {
    rows.par_chunks_mut(d)
        .zip(counts.par_iter())
        .for_each(|(row, &count)| mean_row(row, count));
}

#[inline]
fn mean_row(row: &mut [f32], count: u32) {
    if count == 0 {
        return;
    }
    // One divide and `d` multiplies rather than `d` divides.
    let mult = 1.0 / count as f32;
    for v in row.iter_mut() {
        *v *= mult;
    }
}

/// Scale each of the first `n_rows` rows of an `n_rows × d` row-major matrix to
/// unit L2 norm, projecting it onto the unit sphere.
///
/// An all-zero row stays zero rather than becoming NaN.
///
/// Dispatches on matrix size between [`normalize_rows_l2_sequential`] and
/// [`normalize_rows_l2_parallel`].
pub fn normalize_rows_l2(rows: &mut [f32], n_rows: usize, d: usize) {
    if n_rows * d < REDUCTION_PARALLEL_MIN_ELEMENTS {
        normalize_rows_l2_sequential(rows, n_rows, d);
    } else {
        normalize_rows_l2_parallel(rows, n_rows, d);
    }
}

/// Single-threaded [`normalize_rows_l2`].
pub fn normalize_rows_l2_sequential(rows: &mut [f32], n_rows: usize, d: usize) {
    rows.chunks_mut(d).take(n_rows).for_each(normalize_row_l2);
}

/// One rayon work item per row [`normalize_rows_l2`].
pub fn normalize_rows_l2_parallel(rows: &mut [f32], n_rows: usize, d: usize) {
    rows.par_chunks_mut(d)
        .take(n_rows)
        .for_each(normalize_row_l2);
}

#[inline]
fn normalize_row_l2(row: &mut [f32]) {
    let mut sum = 0.0_f32;
    for v in row.iter() {
        sum += v * v;
    }
    let norm = 1.0 / sum.sqrt().max(f32::EPSILON);
    for v in row.iter_mut() {
        *v *= norm;
    }
}

#[inline]
fn row_shift(new_row: &[f32], prev_row: &[f32]) -> f32 {
    let mut acc = 0.0_f32;
    for k in 0..new_row.len() {
        let diff = new_row[k] - prev_row[k];
        acc += diff * diff;
    }
    acc
}

pub fn find_nearest_neighbor_brute_force(
    x: &[f32],
    y: &[f32],
    n_x: usize,
    n_y: usize,
    d: usize,
    out_knn: &mut [u32],
    out_distances: &mut [f32],
) {
    for i in 0..n_x {
        let mut best_dist = f32::MAX;
        let mut best_idx = 0u32;
        for j in 0..n_y {
            let mut dist = 0.0_f32;
            for k in 0..d {
                let diff = x[i * d + k] - y[j * d + k];
                dist += diff * diff;
            }
            if dist < best_dist {
                best_dist = dist;
                best_idx = j as u32;
            }
        }
        out_knn[i] = best_idx;
        out_distances[i] = best_dist;
    }
}

pub fn random_vec_seeded(rng: &mut impl Rng, n: usize, low: f32, high: f32) -> Vec<f32> {
    let dist = Uniform::new(low, high);
    (0..n).map(|_| dist.sample(rng)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centroid_shift_sums_squared_row_displacement() {
        // Two rows of d=2 moving by (1,2) and (0,1): 5 + 1.
        let new = [1.0, 2.0, 5.0, 5.0];
        let prev = [0.0, 0.0, 5.0, 4.0];
        assert_eq!(centroid_shift_sequential(&new, &prev, 2, 2), 6.0);
        assert_eq!(centroid_shift_parallel(&new, &prev, 2, 2), 6.0);
    }

    #[test]
    fn centroid_shift_ignores_rows_past_n_clusters() {
        // Callers pass whole centroid buffers, which outlive the cluster count
        // they are currently training: the tail must not reach the sum.
        let new = [1.0, 2.0, 100.0, 100.0];
        let prev = [0.0, 0.0, 0.0, 0.0];
        assert_eq!(centroid_shift_sequential(&new, &prev, 1, 2), 5.0);
        assert_eq!(centroid_shift_parallel(&new, &prev, 1, 2), 5.0);
    }

    #[test]
    fn squared_norms_squares_without_rooting() {
        // (3,4) and (0,2): 25 and 4, not 5 and 2.
        let vectors = [3.0, 4.0, 0.0, 2.0];
        assert_eq!(squared_norms(&vectors, 2, 2), [25.0, 4.0]);
    }

    #[test]
    fn squared_norms_ignores_vectors_past_the_count() {
        // Callers pass whole data buffers, which outlive the sample count they are
        // currently training.
        let vectors = [3.0, 4.0, 100.0, 100.0];
        assert_eq!(squared_norms(&vectors, 1, 2), [25.0]);
    }

    #[test]
    fn squared_norms_partial_sums_only_the_leading_dimensions() {
        // Each row keeps its full stride of 4; only the first 2 are summed, so the
        // trailing pair must not reach the result.
        let vectors = [3.0, 4.0, 90.0, 90.0, 1.0, 0.0, 70.0, 70.0];

        for norms in [
            squared_norms_partial_sequential,
            squared_norms_partial_parallel,
        ] {
            assert_eq!(norms(&vectors, 2, 4, 2), [25.0, 1.0]);
        }
    }

    #[test]
    fn squared_norms_partial_clamps_a_prefix_wider_than_the_row() {
        // `partial_d` is tuned at runtime and can be asked to exceed `d`.
        let vectors = [3.0, 4.0];

        for norms in [
            squared_norms_partial_sequential,
            squared_norms_partial_parallel,
        ] {
            assert_eq!(norms(&vectors, 1, 2, 99), [25.0]);
        }
    }

    #[test]
    fn sum_rows_by_assignment_totals_the_members_of_each_cluster() {
        // Vectors 0 and 2 go to cluster 0, vector 1 to cluster 1.
        let vectors = [1.0, 2.0, 10.0, 20.0, 3.0, 4.0];
        let assignments = [0, 1, 0];

        for sum_rows in [
            sum_rows_by_assignment_sequential,
            sum_rows_by_assignment_parallel,
        ] {
            let mut centroids = [0.0; 4];
            let mut cluster_sizes = [0; 2];
            sum_rows(
                &vectors,
                &assignments,
                &mut centroids,
                &mut cluster_sizes,
                2,
            );
            assert_eq!(centroids, [4.0, 6.0, 10.0, 20.0]);
            assert_eq!(cluster_sizes, [2, 1]);
        }
    }

    #[test]
    fn sum_rows_by_assignment_overwrites_whatever_the_last_iteration_left() {
        // Training reuses one buffer per iteration, so the previous sums must not
        // survive into this one — including for a cluster that drew no vectors.
        let vectors = [1.0, 2.0];
        let assignments = [0];

        for sum_rows in [
            sum_rows_by_assignment_sequential,
            sum_rows_by_assignment_parallel,
        ] {
            let mut centroids = [100.0; 4];
            let mut cluster_sizes = [7; 2];
            sum_rows(
                &vectors,
                &assignments,
                &mut centroids,
                &mut cluster_sizes,
                2,
            );
            assert_eq!(centroids, [1.0, 2.0, 0.0, 0.0]);
            assert_eq!(cluster_sizes, [1, 0]);
        }
    }

    #[test]
    fn sum_rows_by_assignment_ignores_vectors_past_the_assignments() {
        // Callers pass whole data buffers, which outlive the sample count they are
        // currently training.
        let vectors = [1.0, 2.0, 100.0, 100.0];
        let assignments = [0];

        for sum_rows in [
            sum_rows_by_assignment_sequential,
            sum_rows_by_assignment_parallel,
        ] {
            let mut centroids = [0.0; 4];
            let mut cluster_sizes = [0; 2];
            sum_rows(
                &vectors,
                &assignments,
                &mut centroids,
                &mut cluster_sizes,
                2,
            );
            assert_eq!(centroids, [1.0, 2.0, 0.0, 0.0]);
            assert_eq!(cluster_sizes, [1, 0]);
        }
    }

    #[test]
    fn sum_rows_by_assignment_leaves_centroids_past_the_cluster_count_alone() {
        // The cluster count comes from `cluster_sizes`, and centroid buffers are
        // sized for the widest split a caller will train, not the current one.
        let vectors = [1.0, 2.0];
        let assignments = [0];

        for sum_rows in [
            sum_rows_by_assignment_sequential,
            sum_rows_by_assignment_parallel,
        ] {
            let mut centroids = [0.0, 0.0, 9.0, 9.0];
            let mut cluster_sizes = [0; 1];
            sum_rows(
                &vectors,
                &assignments,
                &mut centroids,
                &mut cluster_sizes,
                2,
            );
            assert_eq!(centroids, [1.0, 2.0, 9.0, 9.0]);
        }
    }

    #[test]
    fn sum_rows_by_assignment_variants_agree_bit_for_bit() {
        // Each cluster is totalled in vector order either way, so dispatching on
        // size cannot move a result — worth pinning, since the reduction kernels
        // only manage to agree approximately.
        let (n, d, k) = (2_000, 24, 7);
        let vectors = make_blobs(n, d, k, false, 1.0, 10.0, 3);
        let assignments: Vec<u32> = (0..n).map(|i| (i * 7 % k) as u32).collect();

        let mut sequential = vec![0.0_f32; k * d];
        let mut sequential_sizes = vec![0_u32; k];
        sum_rows_by_assignment_sequential(
            &vectors,
            &assignments,
            &mut sequential,
            &mut sequential_sizes,
            d,
        );

        let mut parallel = vec![0.0_f32; k * d];
        let mut parallel_sizes = vec![0_u32; k];
        sum_rows_by_assignment_parallel(
            &vectors,
            &assignments,
            &mut parallel,
            &mut parallel_sizes,
            d,
        );

        assert_eq!(sequential, parallel);
        assert_eq!(sequential_sizes, parallel_sizes);
    }

    #[test]
    fn mean_rows_by_count_divides_each_row_by_its_own_count() {
        let sums = [3.0, 6.0, 8.0, 12.0];
        let counts = [3, 4];

        let mut rows = sums;
        mean_rows_by_count_sequential(&mut rows, &counts, 2);
        assert_eq!(rows, [1.0, 2.0, 2.0, 3.0]);

        let mut parallel = sums;
        mean_rows_by_count_parallel(&mut parallel, &counts, 2);
        assert_eq!(parallel, [1.0, 2.0, 2.0, 3.0]);
    }

    #[test]
    fn mean_rows_by_count_leaves_uncounted_rows_untouched() {
        // An empty cluster has no mean; `split_clusters` reseeds it instead.
        let sums = [3.0, 6.0, 7.0, 9.0];
        let counts = [3, 0];

        let mut rows = sums;
        mean_rows_by_count_sequential(&mut rows, &counts, 2);
        assert_eq!(rows, [1.0, 2.0, 7.0, 9.0]);

        let mut parallel = sums;
        mean_rows_by_count_parallel(&mut parallel, &counts, 2);
        assert_eq!(parallel, [1.0, 2.0, 7.0, 9.0]);
    }

    #[test]
    fn normalize_rows_l2_scales_each_row_to_unit_norm() {
        let rows = [3.0, 4.0, 0.0, 2.0];

        let mut sequential = rows;
        normalize_rows_l2_sequential(&mut sequential, 2, 2);
        assert_eq!(sequential, [0.6, 0.8, 0.0, 1.0]);

        let mut parallel = rows;
        normalize_rows_l2_parallel(&mut parallel, 2, 2);
        assert_eq!(parallel, [0.6, 0.8, 0.0, 1.0]);
    }

    #[test]
    fn normalize_rows_l2_leaves_zero_rows_at_zero() {
        // A cluster that summed to nothing must not turn into NaN.
        let rows = [0.0, 0.0, 3.0, 4.0];

        let mut sequential = rows;
        normalize_rows_l2_sequential(&mut sequential, 1, 2);
        assert_eq!(sequential, [0.0, 0.0, 3.0, 4.0]);

        let mut parallel = rows;
        normalize_rows_l2_parallel(&mut parallel, 1, 2);
        assert_eq!(parallel, [0.0, 0.0, 3.0, 4.0]);
    }
}
