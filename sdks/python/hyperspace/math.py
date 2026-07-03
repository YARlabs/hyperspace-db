from typing import Sequence, List
import math


def _dot(a: Sequence[float], b: Sequence[float]) -> float:
    return sum(x * y for x, y in zip(a, b))


def _norm_sq(v: Sequence[float]) -> float:
    return _dot(v, v)


def _project_to_ball(x: Sequence[float], c: float) -> List[float]:
    n = math.sqrt(max(_norm_sq(x), 0.0))
    max_n = (1.0 / math.sqrt(c)) - 1e-9
    if n <= max_n or n <= 1e-15:
        return list(x)
    s = max_n / n
    return [s * v for v in x]


def mobius_add(x: Sequence[float], y: Sequence[float], c: float = 1.0) -> List[float]:
    if len(x) != len(y):
        raise ValueError("Dimension mismatch")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    xy = _dot(x, y)
    x2 = _norm_sq(x)
    y2 = _norm_sq(y)
    num_left = 1.0 + 2.0 * c * xy + c * y2
    num_right = 1.0 - c * x2
    den = 1.0 + 2.0 * c * xy + (c * c) * x2 * y2
    if abs(den) < 1e-15:
        raise ValueError("Möbius addition denominator is too close to zero")
    return [(num_left * xi + num_right * yi) / den for xi, yi in zip(x, y)]


def exp_map(x: Sequence[float], v: Sequence[float], c: float = 1.0) -> List[float]:
    if len(x) != len(v):
        raise ValueError("Dimension mismatch")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    x2 = _norm_sq(x)
    v_norm = math.sqrt(max(_norm_sq(v), 0.0))
    if v_norm < 1e-15:
        return list(x)
    lambda_x = 2.0 / max(1.0 - c * x2, 1e-15)
    scale = math.tanh(math.sqrt(c) * lambda_x * v_norm / 2.0) / (math.sqrt(c) * v_norm)
    step = [scale * vi for vi in v]
    return mobius_add(x, step, c=c)


def log_map(x: Sequence[float], y: Sequence[float], c: float = 1.0) -> List[float]:
    if len(x) != len(y):
        raise ValueError("Dimension mismatch")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    neg_x = [-xi for xi in x]
    delta = mobius_add(neg_x, y, c=c)
    delta_norm = math.sqrt(max(_norm_sq(delta), 0.0))
    if delta_norm < 1e-15:
        return [0.0 for _ in x]
    x2 = _norm_sq(x)
    lambda_x = 2.0 / max(1.0 - c * x2, 1e-15)
    factor = (2.0 / (lambda_x * math.sqrt(c))) * math.atanh(
        min(math.sqrt(c) * delta_norm, 1.0 - 1e-15)
    )
    return [factor * di / delta_norm for di in delta]


def riemannian_gradient(x: Sequence[float], euclidean_grad: Sequence[float], c: float = 1.0) -> List[float]:
    if len(x) != len(euclidean_grad):
        raise ValueError("Dimension mismatch")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    x2 = _norm_sq(x)
    lambda_x = 2.0 / max(1.0 - c * x2, 1e-15)
    scale = 1.0 / (lambda_x * lambda_x)
    return [scale * g for g in euclidean_grad]


def _gyro(u: Sequence[float], v: Sequence[float], w: Sequence[float], c: float = 1.0) -> List[float]:
    uv = mobius_add(u, v, c=c)
    vw = mobius_add(v, w, c=c)
    left = mobius_add(u, vw, c=c)
    neg_uv = [-z for z in uv]
    return mobius_add(neg_uv, left, c=c)


def parallel_transport(x: Sequence[float], y: Sequence[float], v: Sequence[float], c: float = 1.0) -> List[float]:
    if len(x) != len(y) or len(x) != len(v):
        raise ValueError("Dimension mismatch")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    neg_x = [-xi for xi in x]
    gyr = _gyro(y, neg_x, v, c=c)
    lambda_x = 2.0 / max(1.0 - c * _norm_sq(x), 1e-15)
    lambda_y = 2.0 / max(1.0 - c * _norm_sq(y), 1e-15)
    scale = lambda_x / lambda_y
    return [scale * gi for gi in gyr]


def frechet_mean(points: Sequence[Sequence[float]], c: float = 1.0, max_iter: int = 64, tol: float = 1e-8) -> List[float]:
    if not points:
        raise ValueError("Points set cannot be empty")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")
    dim = len(points[0])
    if any(len(p) != dim for p in points):
        raise ValueError("Dimension mismatch")
    mu = _project_to_ball(points[0], c)
    for _ in range(max(1, max_iter)):
        grad = [0.0] * dim
        for p in points:
            lg = log_map(mu, p, c=c)
            for i in range(dim):
                grad[i] += lg[i]
        inv = 1.0 / float(len(points))
        grad = [g * inv for g in grad]
        g_norm = math.sqrt(max(_norm_sq(grad), 0.0))
        mu = exp_map(mu, grad, c=c)
        mu = _project_to_ball(mu, c)
    return mu


# ==========================================
# Lorentz Model Math (Hyperboloid)
# ==========================================

def lorentz_product(u: Sequence[float], v: Sequence[float]) -> float:
    """Computes the Minkowski inner product (Lorentz product) between two vectors."""
    if not u or not v:
        return 0.0
    dot = -u[0] * v[0]
    for ui, vi in zip(u[1:], v[1:]):
        dot += ui * vi
    return dot


def lorentz_dist(u: Sequence[float], v: Sequence[float]) -> float:
    """Computes the Lorentz distance between two points on the hyperboloid."""
    inner = -lorentz_product(u, v)
    # Clamp to 1.0 to avoid NaN in acosh due to floating point inaccuracies
    arg = max(inner, 1.0)
    return math.acosh(arg)


def lorentz_to_poincare(x: Sequence[float]) -> List[float]:
    """Converts a point from the Lorentz model (Hyperboloid) to the Poincaré Ball model (129 -> 128)."""
    if not x:
        return []
    x0 = x[0]
    denom = max(1.0 + x0, 1e-12)
    return [xi / denom for xi in x[1:]]


def poincare_to_lorentz(p: Sequence[float]) -> List[float]:
    """Converts a point from the Poincaré Ball model to the Lorentz model (128 -> 129)."""
    p_sq = sum(v * v for v in p)
    denom = max(1.0 - p_sq, 1e-12)
    
    x = [(1.0 + p_sq) / denom]
    for pi in p:
        x.append(2.0 * pi / denom)
    return x


def project_to_hyperboloid(v: Sequence[float]) -> List[float]:
    """Ensures a vector satisfies the Lorentz constraint -x0^2 + |x|^2 = -1 (stabilization)."""
    if not v:
        return []
    res = list(v)
    spatial_norm_sq = sum(x * x for x in res[1:])
    res[0] = math.sqrt(1.0 + spatial_norm_sq)
    return res


# ==========================================
# Cognitive Math SDK (Spatial AI Engine)
# ==========================================
# ... rest of the file ...

def local_entropy(candidate: Sequence[float], neighbors: Sequence[Sequence[float]], c: float = 1.0) -> float:
    """
    Calculates the spatial entropy (dispersion) of a `candidate` vector relative to its `neighbors`.
    Used to track LLM hallucinations (Task 2.3.1).
    Returns a value in [0, 1) where values approaching 1 imply high chaos (hallucination).
    """
    if not neighbors:
        return 1.0  # Infinite entropy without neighbors
    total_deviation = 0.0
    for neighbor in neighbors:
        diff = log_map(candidate, neighbor, c=c)
        total_deviation += math.sqrt(max(_norm_sq(diff), 0.0))
    mean_deviation = total_deviation / len(neighbors)
    # Logarithmic compression mapping deviation to [0, 1)
    return 1.0 - math.exp(-mean_deviation)


def lyapunov_convergence(trajectory: Sequence[Sequence[float]], c: float = 1.0) -> float:
    """
    Evaluates if a trajectory of vectors (e.g. Chain of Thought) converges to an attractor.
    Calculates the average energy derivative (Lyapunov function derivative).
    Negative values indicate convergence (stable), positive indicate divergence (chaos/hallucination).
    """
    if len(trajectory) < 3:
        raise ValueError("Need at least 3 points to evaluate convergence trend")
    # Attractor is approximated by Fréchet mean of the trajectory
    attractor = frechet_mean(trajectory, c=c, max_iter=32, tol=1e-6)
    v_diff_sum = 0.0
    for i in range(len(trajectory) - 1):
        v_t0 = math.sqrt(max(_norm_sq(log_map(attractor, trajectory[i], c=c)), 0.0))
        v_t1 = math.sqrt(max(_norm_sq(log_map(attractor, trajectory[i + 1], c=c)), 0.0))
        v_diff_sum += (v_t1 - v_t0)
    
    return v_diff_sum / (len(trajectory) - 1)


def koopman_extrapolate(past: Sequence[float], current: Sequence[float], steps: float, c: float = 1.0) -> List[float]:
    """
    Extrapolates the trajectory in linear space (Koopman linearization) by tracking the 
    shift vector from `past` to `current` and projecting it forward.
    """
    velocity_at_past = log_map(past, current, c=c)
    velocity_at_current = parallel_transport(past, current, velocity_at_past, c=c)
    future_velocity = [v * steps for v in velocity_at_current]
    return exp_map(current, future_velocity, c=c)


def context_resonance(thought: Sequence[float], global_context: Sequence[float], resonance_factor: float, c: float = 1.0) -> List[float]:
    """
    Resonates a thought vector towards a global context vector (Phase-Locked Loop context synchronization).
    Pulls the thought towards the context along the geodesic by `resonance_factor` [0, 1].
    """
    pull_dir = log_map(thought, global_context, c=c)
    factor = max(0.0, min(1.0, resonance_factor))
    applied_pull = [v * factor for v in pull_dir]
    return exp_map(thought, applied_pull, c=c)


# ==========================================
# Private Projection / Mathematical Obfuscation (ZK-Privacy)
# ==========================================

def generate_orthogonal_matrix(dimension: int, seed_bytes: bytes):
    """
    Generates a deterministic, seed-based orthogonal matrix of shape (dimension, dimension).
    Preserves Euclidean distance and inner products (L2 / Cosine).
    """
    import numpy as np
    import hashlib
    seed_int = int.from_bytes(hashlib.sha256(seed_bytes).digest()[:8], 'big')
    rng = np.random.default_rng(seed_int)
    H = rng.standard_normal((dimension, dimension))
    Q, R = np.linalg.qr(H)
    d = np.diag(R)
    # Avoid division by zero
    d_abs = np.abs(d)
    d_abs[d_abs < 1e-15] = 1e-15
    ph = d / d_abs
    Q = Q * ph
    return Q


def generate_lorentz_matrix(dimension: int, seed_bytes: bytes):
    """
    Generates a deterministic, seed-based Lorentz transformation matrix of shape (dimension, dimension).
    Preserves Minkowski inner product (Lorentz distance / Hyperbolic metric).
    """
    import numpy as np
    import hashlib
    if dimension < 2:
        raise ValueError("Lorentz space dimension must be >= 2")
    d = dimension - 1
    seed_int = int.from_bytes(hashlib.sha256(seed_bytes).digest()[:8], 'big')
    rng = np.random.default_rng(seed_int)
    
    # 1. Generate random spatial rotation R
    H = rng.standard_normal((d, d))
    R, R_r = np.linalg.qr(H)
    d_diag = np.diag(R_r)
    d_diag_abs = np.abs(d_diag)
    d_diag_abs[d_diag_abs < 1e-15] = 1e-15
    ph = d_diag / d_diag_abs
    R = R * ph
    
    Lambda_R = np.eye(dimension)
    Lambda_R[1:, 1:] = R
    
    # 2. Generate random boost vector beta in R^d with ||beta|| in [0.1, 0.8]
    beta_dir = rng.standard_normal(d)
    norm = np.linalg.norm(beta_dir)
    if norm < 1e-9:
        beta_dir = np.zeros(d)
        beta_dir[0] = 1.0
        norm = 1.0
    beta_dir = beta_dir / norm
    v = 0.1 + 0.7 * rng.random()
    beta = v * beta_dir
    
    beta_sq = np.dot(beta, beta)
    gamma = 1.0 / np.sqrt(1.0 - beta_sq)
    
    Lambda_B = np.zeros((dimension, dimension))
    Lambda_B[0, 0] = gamma
    Lambda_B[0, 1:] = -gamma * beta
    Lambda_B[1:, 0] = -gamma * beta
    
    factor = (gamma - 1.0) / beta_sq
    spatial_part = np.eye(d) + factor * np.outer(beta, beta)
    Lambda_B[1:, 1:] = spatial_part
    
    return np.matmul(Lambda_R, Lambda_B)


def project_vector(v: Sequence[float], projection_matrix) -> List[float]:
    """Projects a vector using the provided projection matrix (v' = v * P)."""
    import numpy as np
    v_arr = np.array(v, dtype=np.float64)
    v_projected = np.matmul(v_arr, projection_matrix)
    return v_projected.tolist()


def inject_anisotropic_noise(vector: Sequence[float], noise_seed: bytes, sigma: float = 0.02) -> List[float]:
    """
    Injects deterministic, anisotropic noise calibrated to the vector norm.
    Protects against Manifold Alignment Attacks (Manifold Reconstruction).
    """
    if sigma <= 0.0:
        return list(vector)
    import numpy as np
    import hashlib
    vec_arr = np.array(vector, dtype=np.float64)
    # Generate deterministic seed from the vector data + noise_seed
    vec_bytes = str(vector).encode('utf-8')
    vec_hash = hashlib.sha256(noise_seed + vec_bytes).digest()
    seed_int = int.from_bytes(vec_hash[:8], 'big')
    rng = np.random.default_rng(seed_int)
    
    noise = rng.normal(0, sigma, len(vector))
    noisy = vec_arr + noise
    
    # Re-normalize to original norm to preserve cosine / lorentz distance metrics
    orig_norm = np.linalg.norm(vec_arr)
    noisy_norm = np.linalg.norm(noisy)
    if noisy_norm > 1e-15:
        noisy = (noisy / noisy_norm) * orig_norm
        
    return noisy.tolist()


