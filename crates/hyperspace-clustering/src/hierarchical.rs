//! Hierarchical balanced SuperKMeans clustering (HBC / BKT-style tree).
//!
//! The root is clustered, the training set is reordered in place so each child
//! owns a contiguous block of rows, and any child larger than `max_leaf_size`
//! is split the same way until every leaf fits. Balance comes from each split
//! rebalancing its own undersized clusters (`use_aggressive_split`), not from
//! a global penalty term.
//!
//! By default the root splits into \(\lceil\sqrt{K}\rceil\) children with
//! \(K = \lceil n / \texttt{max\_leaf\_size}\rceil\) and deeper splits use
//! \(k = \lceil n_i / \texttt{max\_leaf\_size}\rceil\). Set
//! [`HierarchicalSuperKMeansConfig::branching_factor`] for a fixed fan-out
//! (e.g. 2 for a BKT).
//!
//! Splits permute rows in place, so peak memory is one copy of the training set.

use std::borrow::Cow;
use std::collections::VecDeque;
use std::ops::Range;
use std::sync::Arc;

use crate::adsampling::ADSamplingPruner;
use crate::common::HIERARCHICAL_PRUNER_INITIAL_THRESHOLD;
use crate::superkmeans::{SuperKMeans, SuperKMeansConfig, SuperKMeansIterationStats};
use crate::utils::squared_norms;

/// Config for hierarchical training: [`SuperKMeansConfig`] plus tree knobs.
#[derive(Clone, Debug)]
pub struct HierarchicalSuperKMeansConfig {
    /// Settings for the k-means run performed at each split.
    pub base: SuperKMeansConfig,
    /// Fixed fan-out for every split, capped by how many children the subset
    /// can fill. [`None`] (default) uses \(\lceil\sqrt{K}\rceil\) at the root
    /// and \(\lceil n_i / \texttt{max\_leaf\_size}\rceil\) below it.
    pub branching_factor: Option<usize>,
    /// Stop splitting once a node has at most this many points. The cluster
    /// count is emergent and lands near `n / max_leaf_size`.
    pub max_leaf_size: usize,
    /// Iterations for each local k-means run.
    pub iters_per_split: u32,
}

impl Default for HierarchicalSuperKMeansConfig {
    fn default() -> Self {
        let mut base = SuperKMeansConfig::default();
        // Rebalancing undersized clusters at every split is what keeps the
        // leaves evenly sized.
        base.use_aggressive_split = true;
        Self {
            base,
            branching_factor: None,
            max_leaf_size: 256,
            iters_per_split: 5,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct HierarchicalSuperKMeansIterationStats {
    /// Flattened stats from every local k-means run during tree construction.
    pub local_runs: Vec<SuperKMeansIterationStats>,
}

/// Index into [`ClusterTree::nodes`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// One node in the hierarchical cluster tree.
///
/// Every node owns a contiguous block of the (permuted) training set —
/// `[data_offset, data_offset + size)` — and one centroid.
#[derive(Clone, Debug)]
pub enum TreeNode {
    Internal {
        /// Row index into [`ClusterTree::centroids`] (`centroid_offset * d`).
        centroid_offset: usize,
        /// Start row of this node's points within the split-order training set.
        data_offset: usize,
        /// Number of training points under this node.
        size: usize,
        /// Index of the first child in [`ClusterTree::nodes`].
        children_offset: usize,
        /// Number of direct children; the child ids are
        /// `child_start .. child_start + child_count`.
        children_size: usize,
    },
    Leaf {
        /// Row index into [`ClusterTree::centroids`] (`centroid_offset * d`).
        centroid_offset: usize,
        /// Start row of this leaf's points within the split-order training set.
        data_offset: usize,
        /// Number of training points in this leaf.
        size: usize,
    },
}

impl TreeNode {
    /// Number of training points beneath this node.
    pub fn size(&self) -> usize {
        match self {
            Self::Internal { size, .. } | Self::Leaf { size, .. } => *size,
        }
    }

    /// Row of this node's centroid within [`ClusterTree::centroids`].
    ///
    /// Prefer [`ClusterTree::centroid`], which resolves the row to a slice.
    pub fn centroid_offset(&self) -> usize {
        match self {
            Self::Internal {
                centroid_offset, ..
            }
            | Self::Leaf {
                centroid_offset, ..
            } => *centroid_offset,
        }
    }

    /// Start row of this node's points in the split-order training set.
    pub fn data_offset(&self) -> usize {
        match self {
            Self::Internal { data_offset, .. } | Self::Leaf { data_offset, .. } => *data_offset,
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::Leaf { .. })
    }

    /// Contiguous range of child indices in [`ClusterTree::nodes`], or `None`
    /// for a leaf.
    pub fn child_range(&self) -> Option<Range<usize>> {
        match self {
            Self::Internal {
                children_offset: child_start,
                children_size: child_count,
                ..
            } => Some(*child_start..*child_start + *child_count),
            Self::Leaf { .. } => None,
        }
    }

    /// Number of direct children; `0` for a leaf.
    pub fn child_count(&self) -> usize {
        match self {
            Self::Internal {
                children_size: child_count,
                ..
            } => *child_count,
            Self::Leaf { .. } => 0,
        }
    }
}

/// Full hierarchy produced by training.
///
/// Every node — internal and leaf alike — owns exactly one centroid row in
/// [`Self::centroids`], so `centroids.len() == nodes.len() * d`, and a
/// contiguous block of the (permuted) training set via `data_offset` / `size`.
#[derive(Clone, Debug, Default)]
pub struct ClusterTree {
    /// All nodes; a node's position is its [`NodeId`].
    pub nodes: Vec<TreeNode>,
    /// All node centroids (internal + leaf), row-major, length `nodes.len() * d`.
    pub centroids: Vec<f32>,
    /// Number of leaves, i.e. the number of clusters training produced.
    pub n_leaves: usize,
    /// Entry point for traversals.
    pub root: NodeId,
}

impl ClusterTree {
    /// Drop any previously built nodes so `train` can rebuild from scratch.
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.centroids.clear();
        self.n_leaves = 0;
        self.root = NodeId(0);
    }

    /// Returns true when no nodes have been built yet.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Vector dimensionality, recovered from the one-centroid-per-node layout.
    ///
    /// Returns `0` for an empty tree.
    pub fn dimensionality(&self) -> usize {
        if self.nodes.is_empty() {
            0
        } else {
            self.centroids.len() / self.nodes.len()
        }
    }

    /// Node at `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    pub fn node(&self, id: NodeId) -> &TreeNode {
        &self.nodes[id.0]
    }

    /// Centroid of the node at `id`, in the rotated training domain.
    ///
    /// # Panics
    ///
    /// Panics if `id` is out of bounds.
    pub fn centroid(&self, id: NodeId) -> &[f32] {
        let d = self.dimensionality();
        let row = self.node(id).centroid_offset() * d;
        &self.centroids[row..row + d]
    }

    /// Direct children of `id`, in contiguous `nodes` order.
    ///
    /// Empty when `id` is a leaf.
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        let range = self.node(id).child_range().unwrap_or(0..0);
        range.map(NodeId)
    }

    /// Iterator over leaf nodes in the order of the flat centroid table from
    /// training (`0..n_leaves`).
    pub fn leaves(&self) -> impl Iterator<Item = &TreeNode> {
        self.nodes.iter().filter(|node| node.is_leaf())
    }
}

/// Balanced hierarchical k-means: grows a tree until every leaf holds at most
/// [`HierarchicalSuperKMeansConfig::max_leaf_size`] points.
///
/// The cluster count is emergent, landing near `n / max_leaf_size`.
/// [`Self::train`] returns the leaf centroids and leaves the hierarchy on
/// [`Self::tree`].
///
/// ```no_run
/// use hyperspace_clustering::{HierarchicalSuperKMeans, HierarchicalSuperKMeansConfig};
///
/// let (n, d) = (10_000, 128);
/// let data = vec![0.0_f32; n * d];
///
/// let mut config = HierarchicalSuperKMeansConfig::default();
/// config.max_leaf_size = 64;
///
/// let mut kmeans = HierarchicalSuperKMeans::with_config(d, config);
/// let centroids = kmeans.train(&data, n);
/// assert_eq!(centroids.len(), kmeans.tree.n_leaves * d);
/// ```
pub struct HierarchicalSuperKMeans {
    /// Model holding the leaf centroids once [`Self::train`] has run.
    pub base: SuperKMeans,
    pub config: HierarchicalSuperKMeansConfig,
    /// Per-iteration statistics from every local k-means run.
    pub iteration_stats: HierarchicalSuperKMeansIterationStats,
    /// Hierarchy produced by the most recent [`Self::train`] call.
    pub tree: ClusterTree,
    /// Shared by every per-split model to avoid repeating the O(d³)
    /// Householder QR per node.
    pruner: Arc<ADSamplingPruner>,
}

impl HierarchicalSuperKMeans {
    /// Build a trainer with [`HierarchicalSuperKMeansConfig::default`].
    pub fn new(dimensionality: usize) -> Self {
        Self::with_config(dimensionality, HierarchicalSuperKMeansConfig::default())
    }

    /// Build a trainer for `dimensionality`-dimensional vectors.
    ///
    /// # Panics
    ///
    /// Panics if `branching_factor` is [`Some`]`(b)` with `b < 2`, or if
    /// `max_leaf_size` or `iters_per_split` is zero.
    pub fn with_config(dimensionality: usize, config: HierarchicalSuperKMeansConfig) -> Self {
        if let Some(bf) = config.branching_factor {
            assert!(bf >= 2, "branching_factor must be >= 2");
        }
        assert!(config.max_leaf_size >= 1, "max_leaf_size must be positive");
        assert!(
            config.iters_per_split > 0,
            "iters_per_split must be positive"
        );

        let pruner = Arc::new(ADSamplingPruner::new(
            dimensionality,
            HIERARCHICAL_PRUNER_INITIAL_THRESHOLD,
            config.base.seed,
        ));

        // Placeholder until `train` replaces it with the leaf-centroid model.
        let base = Self::new_superkmeans(
            2,
            dimensionality,
            &config.base,
            LocalSkmMode::Root,
            Arc::clone(&pruner),
        );

        Self {
            base,
            config,
            iteration_stats: HierarchicalSuperKMeansIterationStats::default(),
            tree: ClusterTree::default(),
            pruner,
        }
    }

    /// Build the hierarchy over `n` row-major vectors and return the row-major
    /// **leaf** centroids (`n_leaves × d`).
    ///
    /// The leaf count is emergent; read it from [`ClusterTree::n_leaves`].
    ///
    /// # Panics
    ///
    /// Panics if `n` is zero or the model has already been trained.
    pub fn train(&mut self, data: &[f32], n: usize) -> Vec<f32> {
        self.train_impl(Cow::Borrowed(data), n)
    }

    /// Like [`Self::train`], but takes ownership of `data` so the buffer can be
    /// rotated and split in place. Peak memory is then one copy of the training
    /// set rather than two.
    pub fn train_owned(&mut self, data: Vec<f32>, n: usize) -> Vec<f32> {
        self.train_impl(Cow::Owned(data), n)
    }

    fn train_impl(&mut self, data: Cow<'_, [f32]>, n: usize) -> Vec<f32> {
        assert!(n > 0, "n must be positive");
        assert!(!self.base.trained, "already trained");

        let d = self.base.d;
        self.iteration_stats = HierarchicalSuperKMeansIterationStats::default();
        self.tree.clear();

        let n_samples = n;

        let rotate = !self.config.base.data_already_rotated;
        // Only the owned path can rotate the caller's buffer in place; borrowing
        // forces a second full-size copy.
        let mut data_to_cluster = match data {
            Cow::Owned(mut owned) => {
                owned.truncate(n * d);
                if rotate {
                    self.pruner.rotate_in_place(&mut owned, n);
                }
                owned
            }
            Cow::Borrowed(borrowed) => {
                let len = n * d;
                if rotate {
                    let mut rotated = vec![0.0_f32; len];
                    self.pruner.rotate(&borrowed[..len], &mut rotated, n);
                    rotated
                } else {
                    borrowed[..len].to_vec()
                }
            }
        };

        let mut norms = squared_norms(&data_to_cluster, n_samples, d);
        let mut scratch = PartitionScratch::new(n_samples, d);

        self.build_tree(&mut data_to_cluster, &mut norms, &mut scratch);

        let n_leaves = self.tree.n_leaves;
        assert!(n_leaves > 0, "tree produced no leaves");

        // Replace placeholder `base` with a model holding the leaf centroids.
        // Training membership is not retained.
        let mut base = self.new_superkmeans_from_config(n_leaves, LocalSkmMode::Root);
        base.n_samples = n_samples;
        base.horizontal_centroids = vec![0.0_f32; n_leaves * d];
        base.cluster_sizes = vec![0_u32; n_leaves];

        for (leaf, node) in self.tree.leaves().enumerate() {
            let src = node.centroid_offset() * d;
            base.horizontal_centroids[leaf * d..(leaf + 1) * d]
                .copy_from_slice(&self.tree.centroids[src..src + d]);
            base.cluster_sizes[leaf] = node.size() as u32;
        }

        if base.config.verbose {
            println!(
                "HierarchicalSuperKMeans: n_leaves={}, tree_nodes={}, max_leaf_size={}, branching_factor={:?}",
                n_leaves,
                self.tree.nodes.len(),
                self.config.max_leaf_size,
                self.config.branching_factor
            );
        }

        base.trained = true;
        let centroids = base.get_output_centroids(base.config.unrotate_centroids);
        self.base = base;
        centroids
    }

    /// Assign each of `n_vectors` vectors to its nearest centroid, returning one
    /// index per vector.
    ///
    /// Pass the centroids returned by [`Self::train`] to get leaf indices.
    pub fn assign(&self, vectors: &[f32], centroids: &[f32], n_vectors: usize) -> Vec<u32> {
        self.base.assign(vectors, centroids, n_vectors)
    }

    fn new_superkmeans_from_config(&self, n_clusters: usize, mode: LocalSkmMode) -> SuperKMeans {
        Self::new_superkmeans(
            n_clusters,
            self.base.d,
            &self.config.base,
            mode,
            Arc::clone(&self.pruner),
        )
    }

    fn new_superkmeans(
        n_clusters: usize,
        dimensionality: usize,
        base_config: &SuperKMeansConfig,
        mode: LocalSkmMode,
        pruner: Arc<ADSamplingPruner>,
    ) -> SuperKMeans {
        let mut cfg = base_config.clone();
        match mode {
            LocalSkmMode::Root => {}
            LocalSkmMode::Split { iters } => {
                // Subsets are already in the rotated domain from the root pass.
                cfg.data_already_rotated = true;
                cfg.iters = iters;
            }
        }
        SuperKMeans::with_shared_pruner(n_clusters, dimensionality, cfg, pruner)
    }

    /// Grow the hierarchy: root k-means outside the loop, then split oversized
    /// leaves from a queue until every leaf is at most `max_leaf_size`.
    ///
    /// Each split reorders that node's rows in place so children become
    /// contiguous `[data_offset, data_offset + size)` blocks.
    fn build_tree(&mut self, data: &mut [f32], norms: &mut [f32], scratch: &mut PartitionScratch) {
        let d = self.base.d;
        let n = norms.len();
        assert!(n > 0, "build_tree requires a non-empty sample");
        debug_assert_eq!(data.len(), n * d);

        let mut queue = VecDeque::new();

        // Small enough already: a single leaf over the whole sample.
        let k_root = self.children_for_split(n, true);
        if k_root < 2 {
            let centroid = mean_rows(data, n, d);
            self.tree.root = self.push_leaf(0, n, &centroid);
            self.count_leaves();
            return;
        }

        // Root split: cluster, reorder into cluster order, then materialize the
        // internal root and its children. Deeper splits reuse the same pattern.
        let (centroids, assignments) = self.local_kmeans(data, norms, k_root);
        let (offsets, repaired) =
            self.partition_and_repair(data, norms, 0..n, &assignments, k_root, scratch);

        // The root centroid is never read (routing compares a query against
        // children), so reserve a zero row to keep the layout dense. Children
        // are appended next, so `child_start` will be `nodes.len()` after this push.
        let root = self.push_internal(0, n, &vec![0.0_f32; d], 0, 0);
        self.tree.root = root;

        let (child_start, child_count) =
            self.push_children_from_split(data, &centroids, 0, &offsets, repaired, &mut queue);
        debug_assert!(child_count >= 2, "root split must produce ≥2 children");
        if let TreeNode::Internal {
            children_offset: start,
            children_size: count,
            ..
        } = &mut self.tree.nodes[root.0]
        {
            *start = child_start;
            *count = child_count;
        }

        while let Some(node_id) = queue.pop_front() {
            let (data_offset, size, parent_centroid_offset) = match self.tree.nodes[node_id.0] {
                TreeNode::Leaf {
                    data_offset,
                    size,
                    centroid_offset,
                } => (data_offset, size, centroid_offset),
                TreeNode::Internal { .. } => {
                    unreachable!("queue only holds provisional leaves")
                }
            };

            let k = self.children_for_split(size, false);
            if k < 2 {
                continue;
            }

            let range = data_offset..data_offset + size;
            let (centroids, assignments) = self.local_kmeans(
                &data[range.start * d..range.end * d],
                &norms[range.start..range.end],
                k,
            );
            let (offsets, repaired) =
                self.partition_and_repair(data, norms, range, &assignments, k, scratch);

            let (child_start, child_count) = self.push_children_from_split(
                data,
                &centroids,
                data_offset,
                &offsets,
                repaired,
                &mut queue,
            );
            debug_assert!(child_count >= 2, "split must produce at least two children");

            self.tree.nodes[node_id.0] = TreeNode::Internal {
                centroid_offset: parent_centroid_offset,
                data_offset,
                size,
                children_offset: child_start,
                children_size: child_count,
            };
        }

        self.count_leaves();
    }

    /// Partition `range` by `assignments`. If the clustering collapsed to fewer
    /// than two non-empty groups, bisect instead and report `repaired = true`.
    fn partition_and_repair(
        &self,
        data: &mut [f32],
        norms: &mut [f32],
        range: Range<usize>,
        assignments: &[u32],
        k: usize,
        scratch: &mut PartitionScratch,
    ) -> (Vec<usize>, bool) {
        let n_sub = range.end - range.start;
        let d = self.base.d;
        let mut offsets = {
            let mut subset = Subset {
                data: &mut data[range.start * d..range.end * d],
                norms: &mut norms[range.start..range.end],
            };
            subset.partition(assignments, k, scratch)
        };

        // The clustering collapsed onto one cluster; bisect so the
        // max_leaf_size invariant stays reachable.
        if offsets.windows(2).filter(|b| b[1] > b[0]).count() < 2 {
            offsets = vec![0, (n_sub / 2).clamp(1, n_sub - 1), n_sub];
            return (offsets, true);
        }
        (offsets, false)
    }

    /// Push a leaf for every non-empty cluster in `offsets` (relative to
    /// `base_offset`), enqueueing any that still exceed `max_leaf_size`.
    ///
    /// Children are appended contiguously to [`ClusterTree::nodes`]; the
    /// returned `(child_start, child_count)` describes that block.
    ///
    /// Child centroids come from the k-means run. When `repaired` (bisect
    /// fallback), the partition no longer matches those labels, so each child
    /// takes the mean of its rows instead.
    fn push_children_from_split(
        &mut self,
        data: &[f32],
        centroids: &[f32],
        base_offset: usize,
        offsets: &[usize],
        repaired: bool,
        queue: &mut VecDeque<NodeId>,
    ) -> (usize, usize) {
        let d = self.base.d;
        let child_start = self.tree.nodes.len();
        for (cluster, bounds) in offsets.windows(2).enumerate() {
            let (start, end) = (bounds[0], bounds[1]);
            if start == end {
                continue;
            }
            let child_offset = base_offset + start;
            let child_len = end - start;
            let mean;
            let centroid = if repaired {
                mean = mean_rows(
                    &data[child_offset * d..(child_offset + child_len) * d],
                    child_len,
                    d,
                );
                mean.as_slice()
            } else {
                &centroids[cluster * d..(cluster + 1) * d]
            };
            let child = self.push_leaf(child_offset, child_len, centroid);
            if child_len > self.config.max_leaf_size {
                queue.push_back(child);
            }
        }
        let child_count = self.tree.nodes.len() - child_start;
        (child_start, child_count)
    }

    /// How many children a split of `n_sub` points should request.
    ///
    /// Returns `0` when the subset cannot usefully split (fewer than two
    /// non-empty children would fit under `max_leaf_size`).
    fn children_for_split(&self, n_sub: usize, is_root: bool) -> usize {
        let max_useful = n_sub.div_ceil(self.config.max_leaf_size);
        if max_useful < 2 {
            return 0;
        }
        match self.config.branching_factor {
            // Fixed fan-out (BKT-style): cap at what the subset can fill.
            Some(bf) => bf.min(max_useful),
            // √K meso split at the root, then each oversized subtree finishes
            // toward the leaf cap in one split.
            None if is_root => {
                let meso = (max_useful as f64).sqrt().ceil() as usize;
                meso.clamp(2, max_useful)
            }
            None => max_useful,
        }
    }

    /// Append a leaf over `[data_offset, data_offset + size)` with `centroid`.
    fn push_leaf(&mut self, data_offset: usize, size: usize, centroid: &[f32]) -> NodeId {
        let d = self.base.d;
        debug_assert!(size > 0);
        debug_assert_eq!(centroid.len(), d);

        let centroid_offset = self.tree.centroids.len() / d;
        self.tree.centroids.extend_from_slice(centroid);

        let node_id = NodeId(self.tree.nodes.len());
        self.tree.nodes.push(TreeNode::Leaf {
            centroid_offset,
            data_offset,
            size,
        });
        node_id
    }

    /// Append an internal node over `[data_offset, data_offset + size)`.
    ///
    /// `child_start` / `child_count` may be filled in after the children are
    /// appended (they must form a contiguous block in `nodes`).
    fn push_internal(
        &mut self,
        data_offset: usize,
        size: usize,
        centroid: &[f32],
        child_start: usize,
        child_count: usize,
    ) -> NodeId {
        let d = self.base.d;
        debug_assert!(size > 0);
        debug_assert_eq!(centroid.len(), d);

        let centroid_offset = self.tree.centroids.len() / d;
        self.tree.centroids.extend_from_slice(centroid);

        let node_id = NodeId(self.tree.nodes.len());
        self.tree.nodes.push(TreeNode::Internal {
            centroid_offset,
            data_offset,
            size,
            children_offset: child_start,
            children_size: child_count,
        });
        node_id
    }

    /// Set [`ClusterTree::n_leaves`] from the finished node list.
    fn count_leaves(&mut self) {
        self.tree.n_leaves = self.tree.nodes.iter().filter(|n| n.is_leaf()).count();
    }

    /// Cluster the `n × d` rows of `subset` into `k` groups, returning the
    /// centroids and one cluster index per row.
    ///
    /// `norms` holds the rows' squared norms, computed once at the root;
    /// splits only reorder rows, so they stay valid. Each call runs a fresh
    /// [`SuperKMeans`] through [`SuperKMeans::run_core_loop`], giving every
    /// split the same GEMM batching, pruning and `d'` adaptation as a flat run.
    fn local_kmeans(&mut self, subset: &[f32], norms: &[f32], k: usize) -> (Vec<f32>, Vec<u32>) {
        let d = self.base.d;
        let n = norms.len();
        let iters = self.config.iters_per_split;

        let mut skm = self.new_superkmeans_from_config(k, LocalSkmMode::Split { iters });
        skm.n_samples = n;
        skm.horizontal_centroids = vec![0.0_f32; k * d];
        skm.prev_centroids = vec![0.0_f32; k * d];
        skm.cluster_sizes = vec![0_u32; k];
        skm.assignments = vec![0_u32; n];
        skm.distances = vec![0.0_f32; n];

        // Forgy init (data already in the rotated domain).
        skm.generate_centroids(subset, n, false);
        skm.prev_centroids
            .copy_from_slice(&skm.horizontal_centroids);

        // Full-width norms; the loop narrows them to `d'` once pruning engages.
        skm.data_norms = norms.to_vec();
        skm.centroid_norms = squared_norms(&skm.prev_centroids, k, d);

        skm.run_core_loop(subset, iters);

        self.iteration_stats
            .local_runs
            .append(&mut skm.iteration_stats);
        (skm.horizontal_centroids, skm.assignments)
    }
}

/// Mean of `n` contiguous row-major vectors of length `d`.
fn mean_rows(data: &[f32], n: usize, d: usize) -> Vec<f32> {
    debug_assert_eq!(data.len(), n * d);
    let mut centroid = vec![0.0_f32; d];
    for row in data.chunks_exact(d) {
        for (acc, v) in centroid.iter_mut().zip(row) {
            *acc += v;
        }
    }
    let inv = 1.0 / n as f32;
    for v in &mut centroid {
        *v *= inv;
    }
    centroid
}

/// The contiguous block of training rows one tree node owns.
///
/// `data` and `norms` are parallel — row `i` of `data` has norm `norms[i]` —
/// and [`Subset::partition`] permutes them together so a split can hand each
/// child a contiguous sub-range.
struct Subset<'a> {
    /// Row-major vectors, `n * d` floats.
    data: &'a mut [f32],
    /// Squared L2 norm of each row.
    norms: &'a mut [f32],
}

impl Subset<'_> {
    /// Number of rows. Never zero — a node always owns at least one point.
    fn len(&self) -> usize {
        self.norms.len()
    }

    fn dim(&self) -> usize {
        self.data.len() / self.len()
    }

    /// Group rows by their entry in `assignments`, in place, returning the
    /// `k + 1` offsets that delimit the clusters.
    ///
    /// Every row moves at most once: the assignment is inverted into a
    /// permutation, which is applied by rotating each of its cycles through a
    /// single held-out row. Rows keep their relative order within a cluster,
    /// which is what makes the resulting tree reproducible.
    fn partition(
        &mut self,
        assignments: &[u32],
        k: usize,
        scratch: &mut PartitionScratch,
    ) -> Vec<usize> {
        let (n, d) = (self.len(), self.dim());
        debug_assert_eq!(assignments.len(), n);

        let mut offsets = vec![0_usize; k + 1];
        for &c in assignments {
            offsets[c as usize + 1] += 1;
        }
        for c in 0..k {
            offsets[c + 1] += offsets[c];
        }

        let cursor = &mut scratch.cursor;
        cursor.clear();
        cursor.extend(offsets[..k].iter().map(|&o| o as u32));
        let dest = &mut scratch.dest[..n];
        for (row, &c) in assignments.iter().enumerate() {
            dest[row] = cursor[c as usize];
            cursor[c as usize] += 1;
        }

        let visited = &mut scratch.visited[..n];
        visited.fill(false);
        let held = &mut scratch.row[..d];
        for start in 0..n {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            if dest[start] as usize == start {
                continue;
            }

            held.copy_from_slice(&self.data[start * d..(start + 1) * d]);
            let mut held_norm = self.norms[start];
            // Walk the cycle, leaving each row at its destination and picking up
            // whatever it displaced, until the chain closes back on `start`.
            let mut cur = start;
            loop {
                let next = dest[cur] as usize;
                if next == start {
                    break;
                }
                visited[next] = true;
                held.swap_with_slice(&mut self.data[next * d..(next + 1) * d]);
                std::mem::swap(&mut held_norm, &mut self.norms[next]);
                cur = next;
            }
            self.data[start * d..(start + 1) * d].copy_from_slice(held);
            self.norms[start] = held_norm;
        }

        offsets
    }
}

/// Buffers shared by every in-place partition.
///
/// One set sized for the root suffices: only one partition runs at a time.
struct PartitionScratch {
    /// Destination row of each row, i.e. the permutation being applied.
    dest: Vec<u32>,
    /// Next free row of each cluster while inverting the assignment.
    cursor: Vec<u32>,
    /// Cycle bookkeeping for the permutation.
    visited: Vec<bool>,
    /// The row held out while a cycle rotates.
    row: Vec<f32>,
}

impl PartitionScratch {
    fn new(n: usize, d: usize) -> Self {
        Self {
            dest: vec![0_u32; n],
            cursor: Vec::new(),
            visited: vec![false; n],
            row: vec![0.0_f32; d],
        }
    }
}

/// How to configure a SuperKMeans instance used by hierarchical training.
enum LocalSkmMode {
    /// Placeholder / final leaf model (respects the caller's rotation flags).
    Root,
    /// Per-split clustering over an already-rotated contiguous subset.
    Split { iters: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::make_blobs;

    /// Default config with only the tree shape overridden.
    fn config(
        max_leaf_size: usize,
        branching_factor: Option<usize>,
    ) -> HierarchicalSuperKMeansConfig {
        HierarchicalSuperKMeansConfig {
            max_leaf_size,
            branching_factor,
            ..Default::default()
        }
    }

    fn train_tree(
        n: usize,
        d: usize,
        cfg: HierarchicalSuperKMeansConfig,
    ) -> HierarchicalSuperKMeans {
        let data = make_blobs(n, d, 8, true, 1.0, 5.0, 42);
        let mut kmeans = HierarchicalSuperKMeans::with_config(d, cfg);
        let _centroids = kmeans.train(&data, n);
        kmeans
    }

    #[test]
    fn leaf_size_invariant_and_full_coverage() {
        let mut cfg = config(32, Some(2));
        cfg.iters_per_split = 4;
        cfg.base.early_termination = false;

        let n = 500usize;
        let d = 16usize;
        let kmeans = train_tree(n, d, cfg);
        let tree = &kmeans.tree;

        assert!(
            tree.n_leaves >= 2,
            "expected the worklist build to produce multiple leaves"
        );
        assert_eq!(tree.leaves().count(), tree.n_leaves);
        assert_eq!(kmeans.base.n_clusters, tree.n_leaves);

        let mut total = 0usize;
        for (leaf, node) in tree.leaves().enumerate() {
            assert_eq!(node.size(), kmeans.base.cluster_sizes[leaf] as usize);
            assert!(node.size() > 0);
            assert!(node.size() <= 32);
            total += node.size();
        }
        assert_eq!(total, n, "leaf sizes must cover the training sample");
        assert_eq!(tree.node(tree.root).size(), n);
    }

    #[test]
    fn tree_depth_exceeds_one() {
        let mut cfg = config(16, Some(2));
        cfg.iters_per_split = 3;
        cfg.base.early_termination = false;

        let kmeans = train_tree(256, 8, cfg);
        let root = kmeans.tree.node(kmeans.tree.root);
        assert!(!root.is_leaf());
        assert!(root.child_count() >= 2);

        // At least one child should itself be internal for n=256, leaf=16.
        let has_internal_child = kmeans
            .tree
            .children(kmeans.tree.root)
            .any(|c| matches!(kmeans.tree.node(c), TreeNode::Internal { .. }));
        assert!(has_internal_child, "expected depth > 1");
    }

    /// HBC default: root uses √K meso children with
    /// \(K = \lceil n / \texttt{max\_leaf\_size}\rceil\).
    #[test]
    fn hbc_root_uses_meso_sqrt_k() {
        let (n, d, max_leaf_size) = (10_000usize, 16usize, 100usize);
        let target_leaves = n.div_ceil(max_leaf_size); // 100
        let meso = (target_leaves as f64).sqrt().ceil() as usize; // 10

        let kmeans = train_tree(n, d, config(max_leaf_size, None));
        let root = kmeans.tree.node(kmeans.tree.root);
        assert!(!root.is_leaf());
        let n_children = root.child_count();
        // Empty clusters may be skipped, so allow a small shortfall.
        assert!(
            n_children >= meso.saturating_sub(1) && n_children <= meso,
            "root should split into about √K = {meso} meso clusters, got {n_children}"
        );
        // Direct children must occupy a contiguous block of `nodes`.
        let range = root.child_range().expect("internal root");
        assert_eq!(range.end - range.start, n_children);
    }

    /// Evenly sized leaves have to fall out of the split loop rather than an
    /// explicit penalty. Coefficient of variation over leaf sizes measures it:
    /// 0 is perfectly even, 1.0 means the spread is as large as the mean.
    #[test]
    fn leaf_sizes_stay_balanced() {
        let (n, d, max_leaf_size) = (20_000usize, 32usize, 100usize);
        let data = make_blobs(n, d, 40, true, 1.0, 10.0, 7);

        let mut kmeans = HierarchicalSuperKMeans::with_config(d, config(max_leaf_size, None));
        let _ = kmeans.train(&data, n);

        let sizes: Vec<f64> = kmeans
            .tree
            .leaves()
            .map(|node| node.size() as f64)
            .collect();
        let mean = sizes.iter().sum::<f64>() / sizes.len() as f64;
        let var = sizes.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / sizes.len() as f64;
        let cv = var.sqrt() / mean;

        assert!(
            cv < 0.6,
            "leaf sizes too uneven: cv={cv:.3} over {} leaves (mean={mean:.1})",
            sizes.len()
        );
    }

    /// One iteration per split is a valid, if crude, setting.
    #[test]
    fn one_iteration_per_split_is_allowed() {
        let mut cfg = config(32, Some(32));
        cfg.iters_per_split = 1;
        let kmeans = train_tree(200, 8, cfg);
        assert!(kmeans.tree.n_leaves >= 200 / 32);
    }

    /// Leaf centroids have to be informative. Unit-norm points average a squared
    /// distance near 1.0 from an uninformative centroid, so nearest-centroid
    /// distortion near that means the tree is partitioning by something other
    /// than distance — which is exactly what an ill-scaled balance penalty did.
    #[test]
    fn leaf_centroids_carry_information() {
        let (n, d) = (2_000usize, 32usize);
        let data = make_blobs(n, d, 40, true, 1.0, 10.0, 3);

        let mut kmeans = HierarchicalSuperKMeans::with_config(d, config(25, Some(8)));
        let centroids = kmeans.train(&data, n);
        let assignments = kmeans.assign(&data, &centroids, n);

        let mut total = 0.0f64;
        for (i, &leaf) in assignments.iter().enumerate() {
            let c = &centroids[leaf as usize * d..(leaf as usize + 1) * d];
            let x = &data[i * d..(i + 1) * d];
            let sq: f32 = x.iter().zip(c).map(|(a, b)| (a - b) * (a - b)).sum();
            total += sq as f64;
        }
        let distortion = total / n as f64;

        assert!(
            distortion < 0.5,
            "distortion {distortion} means leaf centroids carry no information"
        );
    }

    /// Capping the branching factor at `ceil(n / max_leaf_size)` is what keeps
    /// leaf sizes near the cap; a fixed wide fan-out on every node instead
    /// strands near-empty leaves and overshoots the cluster count.
    #[test]
    fn leaf_count_tracks_the_requested_granularity() {
        let (n, d, max_leaf_size) = (2_000usize, 16usize, 20usize);

        let kmeans = train_tree(n, d, config(max_leaf_size, Some(16)));
        let ideal = n.div_ceil(max_leaf_size);

        assert!(
            kmeans.tree.n_leaves >= ideal,
            "cannot cover {n} points with {} leaves of at most {max_leaf_size}",
            kmeans.tree.n_leaves
        );
        assert!(
            kmeans.tree.n_leaves <= ideal * 2,
            "leaf count {} overshoots the {ideal} implied by max_leaf_size",
            kmeans.tree.n_leaves
        );
    }

    /// Every internal node must account for exactly the points its children hold.
    #[test]
    fn node_sizes_agree_with_children() {
        let kmeans = train_tree(600, 12, config(24, Some(4)));
        let tree = &kmeans.tree;

        for (i, node) in tree.nodes.iter().enumerate() {
            if node.is_leaf() {
                continue;
            }
            let children: usize = tree.children(NodeId(i)).map(|c| tree.node(c).size()).sum();
            assert_eq!(
                children,
                node.size(),
                "internal node size disagrees with its children"
            );
            // Sibling block is packed contiguously in `nodes`.
            let range = node.child_range().unwrap();
            assert_eq!(range.len(), node.child_count());
        }
        assert_eq!(
            tree.node(tree.root).size(),
            600,
            "root must cover the input"
        );
    }

    #[test]
    fn subset_at_the_cap_stays_a_single_leaf() {
        let kmeans = train_tree(64, 8, config(64, Some(2)));
        assert_eq!(kmeans.tree.n_leaves, 1);
        assert_eq!(kmeans.tree.nodes.len(), 1);
        assert!(kmeans.tree.node(kmeans.tree.root).is_leaf());
    }

    #[test]
    fn tree_accessors_expose_centroids_and_sizes() {
        let (n, d) = (300usize, 16usize);
        let kmeans = train_tree(n, d, config(32, Some(2)));
        let tree = &kmeans.tree;

        assert_eq!(tree.dimensionality(), d);
        assert!(!tree.is_empty());
        assert_eq!(tree.centroid(tree.root).len(), d);
        assert_eq!(tree.leaves().count(), tree.n_leaves);
        assert_eq!(
            tree.leaves().map(|node| node.size()).sum::<usize>(),
            n,
            "leaves must partition the training sample"
        );
    }

    #[test]
    fn assign_maps_vectors_onto_leaf_centroids() {
        let (n, d) = (400usize, 16usize);
        let data = make_blobs(n, d, 8, true, 1.0, 5.0, 42);
        let mut kmeans = HierarchicalSuperKMeans::with_config(d, config(32, Some(2)));
        let centroids = kmeans.train(&data, n);

        let assignments = kmeans.assign(&data, &centroids, n);
        assert_eq!(assignments.len(), n);
        assert!(
            assignments
                .iter()
                .all(|&a| (a as usize) < kmeans.tree.n_leaves),
            "assignment outside the leaf range"
        );
    }

    #[test]
    fn training_is_deterministic_for_a_fixed_seed() {
        let cfg = config(32, Some(4));
        let (n, d) = (300usize, 16usize);
        let data = make_blobs(n, d, 6, true, 1.0, 5.0, 11);

        let mut first = HierarchicalSuperKMeans::with_config(d, cfg.clone());
        let first_centroids = first.train(&data, n);
        let mut second = HierarchicalSuperKMeans::with_config(d, cfg);
        let second_centroids = second.train(&data, n);

        assert_eq!(first.tree.n_leaves, second.tree.n_leaves);
        assert_eq!(first_centroids, second_centroids);
    }

    #[test]
    fn partition_groups_rows_and_keeps_the_arrays_in_lockstep() {
        let (n, d, k) = (9usize, 2usize, 3usize);
        // Row i is [i, i] with norm 2i², which ties data and norms together:
        // any row that lands with the wrong norm is visible.
        let mut data: Vec<f32> = (0..n).flat_map(|i| [i as f32, i as f32]).collect();
        let mut norms: Vec<f32> = (0..n).map(|i| 2.0 * (i * i) as f32).collect();
        let assignments: Vec<u32> = vec![2, 0, 1, 2, 0, 1, 2, 0, 1];

        let mut subset = Subset {
            data: &mut data,
            norms: &mut norms,
        };
        let mut scratch = PartitionScratch::new(n, d);
        let offsets = subset.partition(&assignments, k, &mut scratch);

        assert_eq!(offsets, vec![0, 3, 6, 9]);
        // Stable within-cluster order: cluster 0 ← rows 1,4,7; 1 ← 2,5,8; 2 ← 0,3,6.
        let expected_src = [1usize, 4, 7, 2, 5, 8, 0, 3, 6];
        for (row, &src) in expected_src.iter().enumerate() {
            assert_eq!(data[row * d..(row + 1) * d], [src as f32, src as f32]);
            assert_eq!(norms[row], 2.0 * (src * src) as f32);
        }
    }

    /// Rows move by rotating the permutation's cycles through a single held-out
    /// row, so a slip loses or duplicates training points rather than merely
    /// misordering them. Interleaved clusters make the cycles long, which is
    /// where such a bug shows up.
    #[test]
    fn partition_preserves_every_row_under_long_cycles() {
        let (n, d, k) = (257usize, 3usize, 5usize);
        let value = |row: usize, j: usize| (row * d + j) as f32 * 0.5;

        let mut data: Vec<f32> = (0..n)
            .flat_map(|i| (0..d).map(move |j| value(i, j)))
            .collect();
        let mut norms: Vec<f32> = (0..n)
            .map(|i| (0..d).map(|j| value(i, j) * value(i, j)).sum())
            .collect();
        let assignments: Vec<u32> = (0..n).map(|i| ((i * 7) % k) as u32).collect();

        let mut subset = Subset {
            data: &mut data,
            norms: &mut norms,
        };
        let mut scratch = PartitionScratch::new(n, d);
        let offsets = subset.partition(&assignments, k, &mut scratch);

        let mut seen = vec![false; n];
        for row in 0..n {
            // Recover the original row id from the encoded payload.
            let src = (data[row * d] / 0.5) as usize / d;
            assert!(!seen[src], "row {src} survived twice");
            seen[src] = true;

            for j in 0..d {
                assert_eq!(data[row * d + j], value(src, j));
            }
            let expected: f32 = (0..d).map(|j| value(src, j) * value(src, j)).sum();
            assert_eq!(norms[row], expected);

            let c = assignments[src] as usize;
            assert!(
                (offsets[c]..offsets[c + 1]).contains(&row),
                "row {src} landed outside cluster {c}"
            );
        }
        assert!(seen.iter().all(|&s| s), "the partition dropped rows");
    }

    /// Building an `ADSamplingPruner` is an O(d³) Householder QR, so every
    /// model created during training must adopt the shared rotation matrix
    /// rather than computing its own.
    #[test]
    fn training_reuses_one_rotation_matrix() {
        let mut cfg = config(16, Some(2));
        cfg.iters_per_split = 2;

        let kmeans = train_tree(256, 8, cfg);
        assert!(
            Arc::ptr_eq(&kmeans.pruner, &kmeans.base.pruner),
            "leaf model rebuilt its own pruner instead of sharing one"
        );
    }

    #[test]
    fn leaf_centroids_match_n_leaves() {
        let mut cfg = config(64, Some(4));
        cfg.iters_per_split = 3;
        cfg.base.unrotate_centroids = true;

        let n = 512usize;
        let d = 32usize;
        let data = make_blobs(n, d, 10, true, 1.0, 5.0, 1);
        let mut kmeans = HierarchicalSuperKMeans::with_config(d, cfg);
        let centroids = kmeans.train(&data, n);
        assert_eq!(centroids.len(), kmeans.tree.n_leaves * d);
        assert!(kmeans.tree.n_leaves >= n.div_ceil(64));
    }
}
