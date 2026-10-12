//! Rust port of SuperKMeans — a fast k-means clustering library for
//! high-dimensional vector embeddings using BLAS+ADSampling pruning.
//!
//! The C++ original lives in `SuperKMeans/`; this crate re-implements the
//! C++ public API (no Python surface) in pure Rust with no FFI dependencies.

// The numeric kernels favor explicit index arithmetic and wide, positional
// argument lists that mirror the C++ original, so a handful of Clippy style
// lints are relaxed crate-wide rather than obscuring the ports.
#![allow(
    clippy::pedantic,
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::manual_memcpy,
    clippy::field_reassign_with_default,
    clippy::doc_lazy_continuation,
    clippy::float_cmp,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::items_after_statements,
    clippy::unreadable_literal,
    clippy::default_trait_access,
    clippy::wildcard_imports,
    clippy::module_name_repetitions
)]

// OpenBLAS is linked by `build.rs` (via pkg-config or OPENBLAS_LIB_DIR);
// Accelerate is linked by the framework attribute in `gemm.rs`.
#[cfg(all(
    feature = "blas",
    not(any(feature = "accelerate", feature = "openblas"))
))]
compile_error!(
    "the `blas` feature is a marker — enable a concrete backend instead: \
     `accelerate` (macOS) or `openblas` (Linux/Windows/macOS)"
);

pub mod adsampling;
pub mod batch;
pub mod common;
pub mod distance;
pub mod gemm;
pub mod hierarchical;
pub mod layout;
pub mod pdxearch;
pub mod superkmeans;
pub mod utils;

pub use common::{DistanceFunction, KnnCandidate};
pub use hierarchical::{
    ClusterTree, HierarchicalSuperKMeans, HierarchicalSuperKMeansConfig,
    HierarchicalSuperKMeansIterationStats, NodeId, TreeNode,
};
pub use superkmeans::{
    ClusterBalanceStats, SuperKMeans, SuperKMeansConfig, SuperKMeansIterationStats,
};
pub use utils::{
    centroid_shift, compute_l2_squared, find_nearest_neighbor_brute_force, generate_random_vectors,
    make_blobs, squared_norms, TicToc,
};

/// Clusters high-dimensional vectors (flat row-major f32 slice) into `k` centroids using SuperKMeans.
/// Returns `(centroids_flat, assignments)`.
pub fn cluster_flat_f32(data: &[f32], n: usize, d: usize, k: usize) -> (Vec<f32>, Vec<u32>) {
    let actual_k = k.min(n).max(1);
    let mut kmeans = SuperKMeans::with_config(actual_k, d, SuperKMeansConfig::default());
    let centroids = kmeans.train(data, n);
    let assignments = kmeans.assign(data, &centroids, n);
    (centroids, assignments)
}

/// Clusters vectors represented as `&[Vec<f64>]` into `k` clusters.
/// Returns `Some((centroids, assignments))` where assignments contains centroid indices.
pub fn cluster_vectors_f64(vectors: &[Vec<f64>], k: usize) -> Option<(Vec<Vec<f64>>, Vec<usize>)> {
    if vectors.is_empty() || k == 0 {
        return None;
    }
    let n = vectors.len();
    let d = vectors[0].len();
    if d == 0 {
        return None;
    }
    let mut flat = Vec::with_capacity(n * d);
    for v in vectors {
        if v.len() != d {
            return None;
        }
        for &x in v {
            flat.push(x as f32);
        }
    }
    let actual_k = k.min(n).max(1);
    let (centroids_flat, assignments) = cluster_flat_f32(&flat, n, d, actual_k);
    let mut centroids = Vec::with_capacity(actual_k);
    for i in 0..actual_k {
        let mut c = Vec::with_capacity(d);
        for j in 0..d {
            c.push(centroids_flat[i * d + j] as f64);
        }
        centroids.push(c);
    }
    let assignments_usize = assignments.into_iter().map(|a| a as usize).collect();
    Some((centroids, assignments_usize))
}

/// Hierarchically clusters vectors into bounded clusters (e.g. for agent memory consolidation).
/// Returns list of `(centroid, member_indices)`.
pub fn hierarchical_cluster_f64(
    vectors: &[Vec<f64>],
    max_leaf_size: usize,
) -> Option<Vec<(Vec<f64>, Vec<usize>)>> {
    if vectors.is_empty() {
        return None;
    }
    let n = vectors.len();
    let d = vectors[0].len();
    if d == 0 {
        return None;
    }
    let mut flat = Vec::with_capacity(n * d);
    for v in vectors {
        if v.len() != d {
            return None;
        }
        for &x in v {
            flat.push(x as f32);
        }
    }
    let cfg = HierarchicalSuperKMeansConfig {
        max_leaf_size: max_leaf_size.max(2),
        ..Default::default()
    };
    let mut hkmeans = HierarchicalSuperKMeans::with_config(d, cfg);
    let centroids_flat = hkmeans.train(&flat, n);
    let assignments = hkmeans.assign(&flat, &centroids_flat, n);

    let n_leaves = centroids_flat.len() / d;
    let mut cluster_members: Vec<Vec<usize>> = vec![Vec::new(); n_leaves];
    for (idx, &clust) in assignments.iter().enumerate() {
        let c_idx = clust as usize;
        if c_idx < n_leaves {
            cluster_members[c_idx].push(idx);
        }
    }

    let mut result = Vec::with_capacity(n_leaves);
    for (i, members) in cluster_members.into_iter().enumerate() {
        if members.is_empty() {
            continue;
        }
        let mut c = Vec::with_capacity(d);
        for j in 0..d {
            c.push(centroids_flat[i * d + j] as f64);
        }
        result.push((c, members));
    }
    Some(result)
}
