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
# Cognitive Math SDK (Spatial AI Engine)
# ==========================================

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
