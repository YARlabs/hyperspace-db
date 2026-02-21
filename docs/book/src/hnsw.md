# Zero-Copy Hyperbolic HNSW

Our implementation of [Hierarchical Navigable Small Worlds](https://arxiv.org/abs/1603.09320) is unique in two ways:

1.  **Metric**: It natively speaks hyperbolic geometry.
2.  **Concurrency**: It uses fine-grained locking (`parking_lot::RwLock`) on every node.

## Graph Structure

The graph consists of Layers (0..max).
*   **Layer 0**: Contains ALL vectors. This is the base ground truth.
*   **Layer N**: Contains a random subset of vectors from Layer N-1.

This creates a skip-list-like structure for navigation.

## The "Select Neighbors" Heuristic

When connecting a new node $U$ to neighbors in HNSW, we use a heuristic to ensure diversity.

Standard Euclidean HNSW checks:
*   Add neighbor $V$ if $dist(U, V)$ is minimal.
*   Skip $V$ if it is closer to an *already selected* neighbor than to $U$.

**Hyperbolic Adaptation**:
We use the Poincaré distance for this check. Because the space expands exponentially, "diversity" is easier to achieve, but "closeness" is tricky because points near the boundary (norm $\approx$ 1) have massive distances even if they look close in Euclidean space.

Our heuristic strictly respects the Poincaré metric, preventing "short-circuiting" through the center of the ball unless mathematically valid.

## Locking Strategy

We do not use a global lock.
*   **Reading**: Search traverses nodes acquiring brief Read Locks.
*   **Writing**: Indexer acquires Write Locks only on the specific adjacency lists (layers) it is modifying.

This allows `insert` and `search` to run in parallel with high throughput.

### Batch Search Acceleration
For high-throughput batch search operations, HNSW can offload Minkowski distance computations to the GPU using WGSL compute shaders. This is particularly effective when combined with [Lorentz SQ8 Quantization](lorentz_quantization.md).
