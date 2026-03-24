# The Hyperbolic Geometry

HyperspaceDB operates in the **Poincaré Ball Model** & **Lorentz (hyperboloid)** of hyperbolic geometry. This space is uniquely suited for hierarchical data (trees, graphs, taxonomies) because the amount of "space" available grows exponentially with the radius, similar to how the number of nodes in a tree grows with depth.

## The Distance Formula

The distance $d(u, v)$ between two vectors $u, v$ in the Poincaré ball ($\mathbb{D}^n$) is defined as:

$$
d(u, v) = \text{arccosh}\left( 1 + 2 \frac{\|u - v\|^2}{(1 - \|u\|^2)(1 - \|v\|^2)} \right)
$$

Where:
* $\|u\|$ is the Euclidean norm of vector $u$.
* The vectors must satisfy $\|u\| < 1$.

## Optimization: The "Alpha" Trick

Calculating `arccosh` and divisions for every distance check in HNSW is expensive. HyperspaceDB optimizes this by pre-computing the curvature factors.

For every vector $x$, we store an additional scalar $\alpha_x$:

$$
\alpha_x = \frac{1}{1 - \|x\|^2}
$$

This is stored alongside the quantized vector in our memory-mapped storage.

## The Monotonicity Trick

Since $f(x) = \text{arccosh}(x)$ is a monotonically increasing function for $x \ge 1$, we do not need to compute the full `arccosh` during the **Nearest Neighbor Search** phase. We only need to compare the arguments:

$$
\delta(u, v) = \|u - v\|^2 \cdot \alpha_u \cdot \alpha_v
$$

If $\delta(A) < \delta(B)$, then $d(A) < d(B)$.

HyperspaceDB performs all internal graph traversals using only $\delta$ (SIMD-optimized), and applies the heavy `arccosh` only when required by final ranking/output.

## Lorentz Model (Hyperboloid)

For Lorentz vectors `x = (t, x1, ..., xn)` and `y = (s, y1, ..., yn)`:

$$
\langle x, y \rangle_L = -ts + \sum_i x_i y_i
$$

Distance:

$$
d(x, y) = \operatorname{arcosh}\left(-\langle x, y \rangle_L\right)
$$

Validation constraints:

- upper sheet: `t > 0`
- unit hyperboloid: `-t^2 + x_1^2 + ... + x_n^2 = -1`

### Optimization: SQ8 Quantization
For the Lorentz model, HyperspaceDB implements a specialized 8-bit scalar quantization (SQ8) with dynamic range scaling and GPU/SIMD acceleration. 
See [Lorentz Quantization Details](lorentz_quantization.md).

## SDK Hyperbolic Utilities (v2.2.1)

To keep core DB focused and still support geometry-heavy clients, SDKs include helpers:

- Python: `hyperspace.mobius_add`, `hyperspace.exp_map`, `hyperspace.log_map`
- Rust: `hyperspace_sdk::math::{mobius_add, exp_map, log_map, parallel_transport, riemannian_gradient, frechet_mean}`
- TypeScript: `HyperbolicMath.mobiusAdd/expMap/logMap/parallelTransport/riemannianGradient/frechetMean`

Fréchet mean support is useful for reconsolidation workflows where multiple nearby hyperbolic embeddings should be merged into one robust centroid.

These functions are useful for L-system growth, manifold transforms, and pre-insert vector shaping pipelines.

## Geometric Search (Spatial Filters)

HyperspaceDB v3.0 introduces native geometric predicates. Unlike metadata filters, these are based on the vector's position in the embedding space.

### 1. The Ball Filter (Proximity)
Mathematical definition: $\{ v \in \mathbb{D}^n \mid d(c, v) \le r \}$.
Used for finding all entities within a semantic radius of a concept center $c$.

### 2. The Box Filter (Constraints)
Mathematical definition: $\{ v \in \mathbb{R}^n \mid \forall i, \min_i \le v_i \le \max_i \}$.
Used for bounding reasoning to a specific workspace (e.g., "only consider nodes in the 1st quadrant").

### 3. The Cone Filter (Angular Logic)
Mathematical definition (Angular distance): $\{ v \in \mathbb{R}^n \mid \text{angle}(\text{axes}, v) \le \text{aperture} \}$.
Inspired by **ConE (Zhang & Wang, 2021)**, this filter allows for modeling logical entailment and hierarchy-aware FOV.
In HyperspaceDB, this is implemented as an $O(N)$ dot-product check against the aperture thresholds.

## Performance: Sequential Bitset Pruning

To ensure these filters don't slow down the engine, geometric intersection is performed efficiently during the candidate selection phase. We use a **Bitset Pruning** pattern:
1.  Generate a bitset of candidates satisfying the geometric query.
2.  Perform HNSW bitwise-AND intersection during the search phase.
3.  This allows for $O(1)$ rejection of candidates outside the region of interest.
