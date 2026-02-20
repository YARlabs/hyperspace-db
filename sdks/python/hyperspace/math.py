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
        if g_norm <= max(tol, 1e-15):
            break
        mu = exp_map(mu, grad, c=c)
        mu = _project_to_ball(mu, c)
    return mu
