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
