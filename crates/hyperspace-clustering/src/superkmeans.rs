//! Core SuperKMeans algorithm: BLAS+pruning k-means.

use std::borrow::Cow;
use std::sync::Arc;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Uniform};

use crate::adsampling::ADSamplingPruner;
use crate::batch;
use crate::common::{
    CENTROID_PERTURBATION_EPS, DIMENSION_THRESHOLD_FOR_PRUNING, MIN_PARTIAL_D,
    N_CLUSTERS_THRESHOLD_FOR_PRUNING, PRUNER_INITIAL_THRESHOLD, RECALL_CONVERGENCE_PATIENCE,
    X_BATCH_SIZE, Y_BATCH_SIZE,
};
use crate::layout;
use crate::utils::{
    centroid_shift, mean_rows_by_count, normalize_rows_l2, squared_norms, squared_norms_partial,
    sum_rows_by_assignment,
};

/// Configuration parameters for SuperKMeans clustering.
///
/// There is no sampling knob: [`SuperKMeans::train`] clusters exactly the rows
/// it is handed. Subsample before calling if you want to train on less data.
#[derive(Clone, Debug)]
pub struct SuperKMeansConfig {
    pub iters: u32,
    pub n_threads: u32,
    pub seed: u64,
    pub use_blas_only: bool,
    pub tol: f32,
    pub recall_tol: f32,
    pub early_termination: bool,
    pub sample_queries: bool,
    pub objective_k: usize,
    pub ann_explore_fraction: f32,
    pub min_not_pruned_pct: f32,
    pub max_not_pruned_pct: f32,
    pub adjustment_factor_for_partial_d: f32,
    pub unrotate_centroids: bool,
    pub verbose: bool,
    pub angular: bool,
    pub suppress_warnings: bool,
    pub data_already_rotated: bool,
    /// Use the cuVS-style aggressive small-cluster rebalancing during
    /// consolidation. Hierarchical training opts into this.
    pub use_aggressive_split: bool,
}

impl Default for SuperKMeansConfig {
    fn default() -> Self {
        Self {
            iters: 10,
            n_threads: 0,
            seed: 42,
            use_blas_only: false,
            tol: 1.0e-4,
            recall_tol: 0.005,
            early_termination: true,
            sample_queries: false,
            objective_k: 100,
            ann_explore_fraction: 0.01,
            min_not_pruned_pct: 0.03,
            max_not_pruned_pct: 0.05,
            adjustment_factor_for_partial_d: 0.20,
            unrotate_centroids: true,
            verbose: false,
            angular: false,
            suppress_warnings: false,
            data_already_rotated: false,
            use_aggressive_split: false,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SuperKMeansIterationStats {
    pub iteration: usize,
    pub objective: f32,
    pub shift: f32,
    pub split: usize,
    pub recall: f32,
    pub not_pruned_pct: f32,
    pub partial_d: u32,
    pub is_gemm_only: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ClusterBalanceStats {
    pub mean: f32,
    pub geometric_mean: f32,
    pub stdev: f32,
    pub cv: f32,
    pub min: usize,
    pub max: usize,
}

/// Trained SuperKMeans state.
pub struct SuperKMeans {
    pub d: usize,
    pub n_clusters: usize,
    pub config: SuperKMeansConfig,
    pub n_threads: usize,

    /// Shared because building one runs an O(d³) Householder QR; callers that
    /// need many instances at the same `d` reuse a single rotation matrix.
    pub pruner: Arc<ADSamplingPruner>,

    // Row-major centroids (this iteration & previous iteration).
    pub horizontal_centroids: Vec<f32>,
    pub prev_centroids: Vec<f32>,

    // Per-cluster size and per-sample assignment/distance.
    pub cluster_sizes: Vec<u32>,
    pub assignments: Vec<u32>,
    pub distances: Vec<f32>,

    // Pre-computed norms; may be either full or partial-d depending on the
    // most recent kernel that wrote them.
    pub data_norms: Vec<f32>,
    pub centroid_norms: Vec<f32>,

    // Geometry.
    pub vertical_d: usize,
    pub horizontal_d: usize,
    pub partial_d: u32,

    // Iteration state.
    pub trained: bool,
    pub n_split: usize,
    pub n_samples: usize,
    pub centroids_to_explore: usize,
    pub prev_cost: f32,
    pub cost: f32,
    pub shift: f32,
    pub recall: f32,

    pub iteration_stats: Vec<SuperKMeansIterationStats>,

    /// Reusable SGEMM output scratch, grown once and reused across iterations
    /// by `batch::find_nearest_neighbor*` to avoid per-iteration allocation.
    gemm_buf: Vec<f32>,
}

impl SuperKMeans {
    pub fn new(n_clusters: usize, dimensionality: usize) -> Self {
        Self::with_config(n_clusters, dimensionality, SuperKMeansConfig::default())
    }

    pub fn with_config(
        n_clusters: usize,
        dimensionality: usize,
        config: SuperKMeansConfig,
    ) -> Self {
        assert!(dimensionality > 0, "dimensionality must be positive");
        let pruner = Arc::new(ADSamplingPruner::new(
            dimensionality,
            PRUNER_INITIAL_THRESHOLD,
            config.seed,
        ));
        Self::with_shared_pruner(n_clusters, dimensionality, config, pruner)
    }

    /// Like [`Self::with_config`], but adopts an existing rotation matrix.
    ///
    /// [`ADSamplingPruner::new`] runs an O(d³) Householder QR, which dominates
    /// construction at large `d`. Callers that build many instances for the
    /// same dimensionality should build the pruner once and share it.
    pub fn with_shared_pruner(
        n_clusters: usize,
        dimensionality: usize,
        mut config: SuperKMeansConfig,
        pruner: Arc<ADSamplingPruner>,
    ) -> Self {
        assert!(n_clusters > 0, "n_clusters must be positive");
        assert!(dimensionality > 0, "dimensionality must be positive");
        assert!(config.iters > 0, "iters must be positive");
        assert_eq!(
            pruner.num_dimensions, dimensionality,
            "pruner dimensionality must match"
        );

        if config.data_already_rotated {
            config.unrotate_centroids = false;
        }
        let n_threads = if config.n_threads == 0 {
            rayon::current_num_threads()
        } else {
            config.n_threads as usize
        };
        let split = layout::get_dimension_split(dimensionality);
        Self {
            d: dimensionality,
            n_clusters,
            config,
            n_threads,
            pruner,
            horizontal_centroids: Vec::new(),
            prev_centroids: Vec::new(),
            cluster_sizes: Vec::new(),
            assignments: Vec::new(),
            distances: Vec::new(),
            data_norms: Vec::new(),
            centroid_norms: Vec::new(),
            vertical_d: split.vertical_d,
            horizontal_d: split.horizontal_d,
            partial_d: 0,
            trained: false,
            n_split: 0,
            n_samples: 0,
            centroids_to_explore: 0,
            prev_cost: 0.0,
            cost: 0.0,
            shift: 0.0,
            recall: 0.0,
            iteration_stats: Vec::new(),
            gemm_buf: Vec::new(),
        }
    }

    /// Train the model and return row-major centroids (n_clusters × d).
    pub fn train(&mut self, data: &[f32], n: usize) -> Vec<f32> {
        self.train_with_queries(data, n, &[], 0)
    }

    /// Like [`Self::train`], but takes ownership of `data` so the buffer can be
    /// rotated in place. Peak memory is then one copy of the training set
    /// rather than two.
    pub fn train_owned(&mut self, data: Vec<f32>, n: usize) -> Vec<f32> {
        self.train_impl(Cow::Owned(data), n, &[], 0)
    }

    pub fn train_with_queries(
        &mut self,
        data: &[f32],
        n: usize,
        queries: &[f32],
        n_queries: usize,
    ) -> Vec<f32> {
        self.train_impl(Cow::Borrowed(data), n, queries, n_queries)
    }

    fn train_impl(
        &mut self,
        data: Cow<'_, [f32]>,
        n: usize,
        queries: &[f32],
        n_queries: usize,
    ) -> Vec<f32> {
        assert!(n > 0, "n must be positive");
        assert!(!self.trained, "The clustering has already been trained");
        assert!(
            n >= self.n_clusters,
            "n must be >= n_clusters ({} < {})",
            n,
            self.n_clusters
        );
        if n_queries > 0 && queries.is_empty() && !self.config.sample_queries {
            panic!("Queries must be provided if n_queries > 0 and sample_queries is false");
        }

        self.iteration_stats.clear();
        // Every row handed in is clustered; sampling belongs to the caller.
        self.n_samples = n;

        let d = self.d;
        let n_clusters = self.n_clusters;
        let n_samples = self.n_samples;

        self.horizontal_centroids = vec![0.0_f32; n_clusters * d];
        self.prev_centroids = vec![0.0_f32; n_clusters * d];
        self.cluster_sizes = vec![0_u32; n_clusters];
        self.assignments = vec![0_u32; n_samples];
        self.distances = vec![0.0_f32; n_samples];
        self.data_norms = vec![0.0_f32; n_samples];
        self.centroid_norms = vec![0.0_f32; n_clusters];

        if self.config.verbose {
            println!("Front dimensions (d') = {}", self.initial_partial_d());
            println!("Trailing dimensions (d'') = {}", d - self.vertical_d);
        }

        // Sample initial centroids (Forgy) and rotate them into prev_centroids/horizontal_centroids.
        let rotate = !self.config.data_already_rotated;
        self.generate_centroids(&data, n, rotate);

        let data_to_cluster = match data {
            Cow::Borrowed(borrowed) => self.rotate_vectors(borrowed, n, rotate),
            Cow::Owned(owned) => self.rotate_vectors_owned(owned, n, rotate),
        };

        // horizontal_centroids currently holds the unrotated Forgy samples.
        // Rotate (or copy) into prev_centroids.
        self.rotate_or_copy_into_prev_centroids(rotate);

        // Compute full norms for first GEMM iteration.
        self.data_norms = squared_norms(&data_to_cluster, n_samples, d);
        self.centroid_norms = squared_norms(&self.prev_centroids, n_clusters, d);

        // Optional recall tracking — skipped for the minimal port; the algorithm runs
        // identically without queries (early-termination on shift/cost still works).
        let _ = queries;
        let _ = n_queries;

        self.run_core_loop(&data_to_cluster, self.config.iters);

        self.trained = true;
        self.get_output_centroids(self.config.unrotate_centroids)
    }

    /// The Lloyd driver: alternate assignment and centroid update for `iters`
    /// passes, widening `d'` as the pruning bound tightens and stopping early
    /// on convergence.
    ///
    /// Callers own the setup: per-cluster and per-sample buffers sized for
    /// `self.n_samples` / `self.n_clusters`, initial centroids in
    /// `prev_centroids` (rotated domain), and full-width norms in `data_norms`
    /// and `centroid_norms`. Both [`Self::train`] and the hierarchical
    /// splitter go through here.
    pub(crate) fn run_core_loop(&mut self, data: &[f32], iters: u32) {
        let d = self.d;
        let n_clusters = self.n_clusters;
        let n_samples = self.n_samples;

        self.partial_d = self.initial_partial_d();

        let always_gemm_only = d < DIMENSION_THRESHOLD_FOR_PRUNING
            || self.config.use_blas_only
            || n_clusters <= N_CLUSTERS_THRESHOLD_FOR_PRUNING;
        let mut partial_norms_computed = false;
        let mut best_recall = 0.0_f32;
        let mut iters_without_improvement: usize = 0;

        let mut not_pruned_counts = vec![0_usize; n_samples];

        for iter_idx in 0..iters {
            let use_gemm_only = (iter_idx == 0) || always_gemm_only;
            if !use_gemm_only && !partial_norms_computed {
                self.data_norms =
                    squared_norms_partial(data, n_samples, d, self.partial_d as usize);
                partial_norms_computed = true;
            }
            self.run_iteration(
                data,
                iter_idx,
                iter_idx == 0,
                use_gemm_only,
                &mut not_pruned_counts,
            );

            if self.config.early_termination
                && self.should_stop_early(
                    false,
                    &mut best_recall,
                    &mut iters_without_improvement,
                    iter_idx,
                )
            {
                break;
            }
        }
    }

    /// Brute-force assignment: returns the index of the nearest centroid for
    /// each vector (data and centroids assumed to share the same domain).
    pub fn assign(&self, vectors: &[f32], centroids: &[f32], n_vectors: usize) -> Vec<u32> {
        let d = self.d;
        let n_centroids = centroids.len() / d;
        let mut result_assignments = vec![0_u32; n_vectors];
        let mut result_distances = vec![0.0_f32; n_vectors];
        let vector_norms = squared_norms(vectors, n_vectors, d);
        let centroid_norms = squared_norms(centroids, n_centroids, d);
        let mut buf = Vec::new();
        batch::find_nearest_neighbor(
            vectors,
            centroids,
            n_vectors,
            n_centroids,
            d,
            &vector_norms,
            &centroid_norms,
            &mut result_assignments,
            &mut result_distances,
            &mut buf,
        );
        result_assignments
    }

    /// Re-assign the training set to its nearest centroid using the pruning path.
    ///
    /// Each point's pruning threshold is seeded with the centroid it landed on
    /// during training, which is what makes pruning pay: starting from that
    /// tight bound rather than a full GEMM over the first centroid batch is
    /// worth roughly 1.8x in the paper's ablation.
    ///
    /// A *different* set of vectors has no such seed available, so it needs a
    /// cold-start strategy and is deliberately not handled here — use
    /// [`Self::assign`], which is exact but unpruned.
    ///
    /// # Panics
    ///
    /// Panics if the model is untrained, or if `n_vectors` differs from the row
    /// count passed to training.
    pub fn assign_training_points(
        &mut self,
        vectors: &[f32],
        centroids: &[f32],
        n_vectors: usize,
    ) -> Vec<u32> {
        assert!(
            self.trained,
            "assign_training_points requires training first"
        );
        assert_eq!(
            n_vectors, self.n_samples,
            "assign_training_points re-assigns the training set; pass the {} rows given to \
             train, or use assign() for a different set",
            self.n_samples
        );
        let d = self.d;
        let n_centroids = centroids.len() / d;

        // Fall-back path: when pruning isn't beneficial, defer to brute force.
        if self.config.use_blas_only
            || d < DIMENSION_THRESHOLD_FOR_PRUNING
            || n_centroids <= N_CLUSTERS_THRESHOLD_FOR_PRUNING
        {
            if !self.config.suppress_warnings {
                eprintln!(
                    "WARNING: AssignTrainingPoints cannot use pruning, falling back to brute force"
                );
            }
            return self.assign(vectors, centroids, n_vectors);
        }

        self.partial_d = self.initial_partial_d();

        let mut not_pruned_counts = vec![0_usize; n_vectors];
        let mut data_buffer = vec![0.0_f32; n_vectors * d];
        let data_p: &[f32] = if self.config.data_already_rotated {
            vectors
        } else {
            self.pruner.rotate(vectors, &mut data_buffer, n_vectors);
            &data_buffer
        };

        // Norms from the rotated centroids (the pruning space), not the passed-in
        // `centroids` (unrotated; used only by the brute-force fallback).
        let centroid_partial_norms = squared_norms_partial(
            &self.horizontal_centroids,
            n_centroids,
            d,
            self.partial_d as usize,
        );

        let data_partial_norms =
            squared_norms_partial(data_p, n_vectors, d, self.partial_d as usize);

        // Training row i is input row i, so the trained assignments seed directly.
        assert!(
            self.assignments.len() >= n_vectors,
            "training assignments were not retained; use assign() instead"
        );
        let mut result_assignments = self.assignments[..n_vectors].to_vec();
        let mut result_distances = vec![0.0_f32; n_vectors];
        batch::find_nearest_neighbor_with_pruning(
            data_p,
            &self.horizontal_centroids,
            n_vectors,
            n_centroids,
            d,
            self.vertical_d,
            self.horizontal_d,
            &data_partial_norms,
            &centroid_partial_norms,
            &mut result_assignments,
            &mut result_distances,
            &self.pruner,
            self.partial_d as usize,
            &mut not_pruned_counts,
            &mut self.gemm_buf,
        );

        result_assignments
    }

    // ----------- internals -----------

    pub(crate) fn run_iteration(
        &mut self,
        data: &[f32],
        iter_idx: u32,
        is_first_iter: bool,
        gemm_only: bool,
        not_pruned_counts: &mut [usize],
    ) {
        let d = self.d;
        let n_clusters = self.n_clusters;
        let n_samples = self.n_samples;

        if !is_first_iter {
            std::mem::swap(&mut self.horizontal_centroids, &mut self.prev_centroids);
        }

        if gemm_only {
            self.centroid_norms = squared_norms(&self.prev_centroids, n_clusters, d);
            batch::find_nearest_neighbor(
                data,
                &self.prev_centroids,
                n_samples,
                n_clusters,
                d,
                &self.data_norms,
                &self.centroid_norms,
                &mut self.assignments,
                &mut self.distances,
                &mut self.gemm_buf,
            );
        } else {
            // Partial norms for pruning.
            self.centroid_norms =
                squared_norms_partial(&self.prev_centroids, n_clusters, d, self.partial_d as usize);
            for v in not_pruned_counts.iter_mut() {
                *v = 0;
            }
            batch::find_nearest_neighbor_with_pruning(
                data,
                &self.prev_centroids,
                n_samples,
                n_clusters,
                d,
                self.vertical_d,
                self.horizontal_d,
                &self.data_norms,
                &self.centroid_norms,
                &mut self.assignments,
                &mut self.distances,
                &self.pruner,
                self.partial_d as usize,
                not_pruned_counts,
                &mut self.gemm_buf,
            );
        }

        // M-step: total each cluster, which `consolidate_centroids` then averages.
        sum_rows_by_assignment(
            data,
            &self.assignments[..n_samples],
            &mut self.horizontal_centroids,
            &mut self.cluster_sizes[..n_clusters],
            d,
        );

        let mut avg_not_pruned_pct = -1.0_f32;
        let old_partial_d = self.partial_d;
        if !gemm_only {
            let (avg, changed) = self.tune_partial_d(not_pruned_counts, n_samples, n_clusters);
            avg_not_pruned_pct = avg;
            if changed {
                self.data_norms =
                    squared_norms_partial(data, n_samples, d, self.partial_d as usize);
            }
        }

        self.consolidate_centroids(n_samples, n_clusters);
        self.compute_cost();
        self.shift = centroid_shift(
            &self.horizontal_centroids,
            &self.prev_centroids,
            n_clusters,
            d,
        );

        let stats = SuperKMeansIterationStats {
            iteration: (iter_idx + 1) as usize,
            objective: self.cost,
            shift: self.shift,
            split: self.n_split,
            recall: self.recall,
            not_pruned_pct: if gemm_only { -1.0 } else { avg_not_pruned_pct },
            partial_d: if gemm_only { 0 } else { old_partial_d },
            is_gemm_only: gemm_only,
        };
        if self.config.verbose {
            let improvement = if iter_idx > 0 {
                1.0 - (self.cost / self.prev_cost.max(f32::EPSILON))
            } else {
                0.0
            };
            print!(
                "Iteration {}/{} | Objective: {:.4} | Objective improvement: {:.4} | Shift: {:.4} | Split: {}",
                iter_idx + 1,
                self.config.iters,
                self.cost,
                improvement,
                self.shift,
                self.n_split
            );
            if gemm_only {
                println!(" [BLAS-only]");
            } else {
                println!(
                    " | Not Pruned %: {:.4} | d': {} -> {}",
                    avg_not_pruned_pct * 100.0,
                    old_partial_d,
                    self.partial_d
                );
            }
        }
        self.iteration_stats.push(stats);
    }

    /// Turn the sums left by [`sum_rows_by_assignment`] into usable centroids:
    /// take each cluster's mean, reseed the empty ones, and — for angular
    /// distance — put the result back on the unit sphere.
    ///
    /// Normalizing last matters: [`Self::split_clusters`] perturbs and blends
    /// rows, so those have to land on the sphere too.
    pub(crate) fn consolidate_centroids(&mut self, n_samples: usize, n_clusters: usize) {
        let d = self.d;
        mean_rows_by_count(
            &mut self.horizontal_centroids,
            &self.cluster_sizes[..n_clusters],
            d,
        );

        self.split_clusters(n_samples, n_clusters);

        if self.config.angular {
            normalize_rows_l2(&mut self.horizontal_centroids, n_clusters, d);
        }
    }

    pub(crate) fn split_clusters(&mut self, n_samples: usize, n_clusters: usize) {
        self.n_split = 0;
        let d = self.d;
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed);
        let uniform = Uniform::new(0.0_f32, 1.0_f32);

        // Empty-cluster handling: find a donor and copy + symmetric-perturb.
        for ci in 0..n_clusters {
            if self.cluster_sizes[ci] != 0 {
                continue;
            }
            let mut cj = 0usize;
            loop {
                let size_j = self.cluster_sizes[cj] as f32;
                let denom = (n_samples as f32 - n_clusters as f32).max(1.0);
                let p = (size_j - 1.0) / denom;
                let r = uniform.sample(&mut rng);
                if r < p {
                    break;
                }
                cj = (cj + 1) % n_clusters;
            }
            let (left, right) = split_two_rows_mut(&mut self.horizontal_centroids, ci, cj, d);
            left.copy_from_slice(right);
            for j in 0..d {
                if j % 2 == 0 {
                    left[j] *= 1.0 + CENTROID_PERTURBATION_EPS;
                    right[j] *= 1.0 - CENTROID_PERTURBATION_EPS;
                } else {
                    left[j] *= 1.0 - CENTROID_PERTURBATION_EPS;
                    right[j] *= 1.0 + CENTROID_PERTURBATION_EPS;
                }
            }
            let half = self.cluster_sizes[cj] / 2;
            self.cluster_sizes[ci] = half;
            self.cluster_sizes[cj] -= half;
            self.n_split += 1;
        }

        // Optional aggressive cuVS-style balancing: pull small clusters towards
        // points from large ones.
        if !self.config.use_aggressive_split {
            return;
        }
        const CENTER_ADJUSTMENT_WEIGHT: f32 = 7.0;
        const BALANCING_THRESHOLD: f32 = 0.25;
        let average_size = (n_samples / n_clusters.max(1)) as f32;
        let threshold_size = (average_size * BALANCING_THRESHOLD) as u32;
        for ci in 0..n_clusters {
            let csize = self.cluster_sizes[ci];
            if csize == 0 || csize as f32 > threshold_size as f32 {
                continue;
            }
            let mut large_idx = 0usize;
            loop {
                let large_size = self.cluster_sizes[large_idx];
                if (large_size as f32) < average_size {
                    large_idx = (large_idx + 1) % n_clusters;
                    continue;
                }
                let p = (large_size as f32 - average_size + 1.0)
                    / ((n_samples as f32) - average_size * (n_clusters as f32) + n_clusters as f32);
                let r = uniform.sample(&mut rng);
                if r < p {
                    break;
                }
                large_idx = (large_idx + 1) % n_clusters;
            }
            let (small_row, large_row) =
                split_two_rows_mut(&mut self.horizontal_centroids, ci, large_idx, d);
            let wc = (csize as f32).min(CENTER_ADJUSTMENT_WEIGHT);
            let wd = 1.0_f32;
            for j in 0..d {
                let v = (wc * small_row[j] + wd * large_row[j]) / (wc + wd);
                small_row[j] = v;
            }
            self.n_split += 1;
        }
    }

    /// The starting `d'`: half the vertically-stored dimensions, floored at
    /// [`MIN_PARTIAL_D`] and capped at what is actually stored vertically.
    /// [`Self::tune_partial_d`] widens or narrows it from here.
    fn initial_partial_d(&self) -> u32 {
        MIN_PARTIAL_D
            .max(self.vertical_d as u32 / 2)
            .min(self.vertical_d as u32)
    }

    pub(crate) fn compute_cost(&mut self) {
        self.prev_cost = self.cost;
        self.cost = self.distances.iter().sum::<f32>();
    }

    pub(crate) fn tune_partial_d(
        &mut self,
        not_pruned_counts: &[usize],
        n_samples: usize,
        n_y: usize,
    ) -> (f32, bool) {
        let sum: f64 = not_pruned_counts
            .iter()
            .take(n_samples)
            .map(|&v| v as f64)
            .sum();
        let avg = (sum / (n_samples as f64 * n_y as f64)) as f32;
        let old_partial_d = self.partial_d;
        if avg > self.config.max_not_pruned_pct {
            let increase = ((self.partial_d as f32)
                * self.config.adjustment_factor_for_partial_d
                * 2.0) as u32;
            self.partial_d = (self.partial_d + increase.max(1)).min(self.vertical_d as u32);
        } else if avg < self.config.min_not_pruned_pct {
            let decrease =
                ((self.partial_d as f32) * self.config.adjustment_factor_for_partial_d) as u32;
            self.partial_d = (self.partial_d.saturating_sub(decrease.max(1))).max(MIN_PARTIAL_D);
        }
        (avg, old_partial_d != self.partial_d)
    }

    pub(crate) fn should_stop_early(
        &mut self,
        tracking_recall: bool,
        best_recall: &mut f32,
        iters_without_improvement: &mut usize,
        iter_idx: u32,
    ) -> bool {
        if self.shift < self.config.tol {
            if self.config.verbose {
                println!(
                    "Converged at iteration {} (shift {} < tol {})",
                    iter_idx + 1,
                    self.shift,
                    self.config.tol
                );
            }
            return true;
        }
        if iter_idx > 0 {
            let cost_delta = self.cost / self.prev_cost.max(f32::EPSILON);
            if cost_delta > 1.0 - self.config.tol {
                if self.config.verbose {
                    println!(
                        "Converged at iteration {} (cost improved by only {})",
                        iter_idx + 1,
                        1.0 - cost_delta
                    );
                }
                return true;
            }
        }
        if tracking_recall {
            let improvement = self.recall - *best_recall;
            if improvement > self.config.recall_tol {
                *best_recall = self.recall;
                *iters_without_improvement = 0;
            } else {
                *iters_without_improvement += 1;
                if *iters_without_improvement >= RECALL_CONVERGENCE_PATIENCE {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn generate_centroids(&mut self, data: &[f32], n: usize, rotate: bool) {
        let d = self.d;
        let n_clusters = self.n_clusters;
        if self.horizontal_centroids.len() < n_clusters * d {
            self.horizontal_centroids.resize(n_clusters * d, 0.0);
        }
        let mut rng = ChaCha8Rng::seed_from_u64(self.config.seed);
        let mut indices: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = rng.gen_range(0..=i);
            indices.swap(i, j);
        }
        for i in 0..n_clusters {
            let src = indices[i] * d;
            let dst = i * d;
            self.horizontal_centroids[dst..dst + d].copy_from_slice(&data[src..src + d]);
        }

        if rotate {
            let mut rotated = vec![0.0_f32; n_clusters * d];
            self.pruner
                .rotate(&self.horizontal_centroids, &mut rotated, n_clusters);
            self.horizontal_centroids[..n_clusters * d].copy_from_slice(&rotated);
        }
    }

    /// Copy the `n` rows to cluster into an owned buffer, rotating into the
    /// pruning domain on the way when `rotate` is set.
    fn rotate_vectors(&self, data: &[f32], n: usize, rotate: bool) -> Vec<f32> {
        let len = n * self.d;
        if rotate {
            let mut rotated = vec![0.0_f32; len];
            self.pruner.rotate(&data[..len], &mut rotated, n);
            rotated
        } else {
            data[..len].to_vec()
        }
    }

    /// Owned variant: rotates `data` in place instead of allocating a second
    /// full-size buffer.
    fn rotate_vectors_owned(&self, mut data: Vec<f32>, n: usize, rotate: bool) -> Vec<f32> {
        data.truncate(n * self.d);
        if rotate {
            self.pruner.rotate_in_place(&mut data, n);
        }
        data
    }

    /// Copy horizontal_centroids -> prev_centroids, applying rotation if needed.
    fn rotate_or_copy_into_prev_centroids(&mut self, rotate: bool) {
        let d = self.d;
        let n_clusters = self.n_clusters;
        if rotate {
            // horizontal_centroids is already rotated (generate_centroids did it).
            // We just need to mirror into prev_centroids.
            self.prev_centroids[..n_clusters * d]
                .copy_from_slice(&self.horizontal_centroids[..n_clusters * d]);
        } else {
            self.prev_centroids[..n_clusters * d]
                .copy_from_slice(&self.horizontal_centroids[..n_clusters * d]);
        }
    }

    pub(crate) fn get_output_centroids(&self, unrotate: bool) -> Vec<f32> {
        let d = self.d;
        let n_clusters = self.n_clusters;
        if unrotate {
            let mut out = vec![0.0_f32; n_clusters * d];
            self.pruner
                .unrotate(&self.horizontal_centroids, &mut out, n_clusters);
            out
        } else {
            self.horizontal_centroids[..n_clusters * d].to_vec()
        }
    }

    /// Cluster-balance summary stats over the assignments.
    pub fn cluster_balance_stats(
        assignments: &[u32],
        n_samples: usize,
        n_clusters: usize,
    ) -> ClusterBalanceStats {
        let mut sizes = vec![0_usize; n_clusters];
        for i in 0..n_samples {
            sizes[assignments[i] as usize] += 1;
        }
        let mean = sizes.iter().sum::<usize>() as f32 / sizes.len() as f32;
        let mut log_sum = 0.0_f32;
        let mut non_zero = 0;
        for &s in &sizes {
            if s > 0 {
                log_sum += (s as f32).ln();
                non_zero += 1;
            }
        }
        let geometric_mean = if non_zero > 0 {
            (log_sum / non_zero as f32).exp()
        } else {
            0.0
        };
        let sq_sum: usize = sizes.iter().map(|&s| s * s).sum();
        let stdev = (sq_sum as f32 / sizes.len() as f32 - mean * mean)
            .max(0.0)
            .sqrt();
        let cv = if mean != 0.0 { stdev / mean } else { 0.0 };
        let min = *sizes.iter().min().unwrap_or(&0);
        let max = *sizes.iter().max().unwrap_or(&0);
        ClusterBalanceStats {
            mean,
            geometric_mean,
            stdev,
            cv,
            min,
            max,
        }
    }
}

/// Borrow two disjoint rows of a row-major matrix mutably.
fn split_two_rows_mut(data: &mut [f32], a: usize, b: usize, d: usize) -> (&mut [f32], &mut [f32]) {
    assert!(a != b, "split_two_rows_mut requires distinct rows");
    if a < b {
        let (left, right) = data.split_at_mut(b * d);
        (&mut left[a * d..(a + 1) * d], &mut right[..d])
    } else {
        let (left, right) = data.split_at_mut(a * d);
        (&mut right[..d], &mut left[b * d..(b + 1) * d])
    }
}

// Suppress unused-constant warnings for re-export crumbs.
const _: () = {
    let _ = X_BATCH_SIZE;
    let _ = Y_BATCH_SIZE;
};
