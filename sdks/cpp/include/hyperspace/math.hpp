#pragma once

#include <vector>
#include <cmath>
#include <stdexcept>
#include <numeric>
#include <algorithm>

namespace hyperspace {
namespace math {

inline double dot(const std::vector<double>& a, const std::vector<double>& b) {
    if (a.size() != b.size()) throw std::invalid_argument("Dimension mismatch");
    return std::inner_product(a.begin(), a.end(), b.begin(), 0.0);
}

inline double norm_sq(const std::vector<double>& v) {
    return dot(v, v);
}

inline std::vector<double> project_to_ball(std::vector<double> x, double c) {
    double n = std::sqrt(std::max(norm_sq(x), 0.0));
    double max_n = (1.0 / std::sqrt(c)) - 1e-9;
    if (n <= max_n || n <= 1e-15) return x;
    double s = max_n / n;
    for (auto& val : x) val *= s;
    return x;
}

inline std::vector<double> mobius_add(const std::vector<double>& x, const std::vector<double>& y, double c = 1.0) {
    if (x.size() != y.size()) throw std::invalid_argument("Dimension mismatch");
    if (c <= 0.0) throw std::invalid_argument("Curvature c must be > 0");

    double xy = dot(x, y);
    double x2 = norm_sq(x);
    double y2 = norm_sq(y);

    double num_left = 1.0 + 2.0 * c * xy + c * y2;
    double num_right = 1.0 - c * x2;
    double den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;

    if (std::abs(den) < 1e-15) throw std::runtime_error("Mobius addition denominator too close to zero");

    std::vector<double> res(x.size());
    for (size_t i = 0; i < x.size(); ++i) {
        res[i] = (num_left * x[i] + num_right * y[i]) / den;
    }
    return res;
}

inline std::vector<double> exp_map(const std::vector<double>& x, const std::vector<double>& v, double c = 1.0) {
    if (x.size() != v.size()) throw std::invalid_argument("Dimension mismatch");
    if (c <= 0.0) throw std::invalid_argument("Curvature c must be > 0");

    double x2 = norm_sq(x);
    double v_norm = std::sqrt(std::max(norm_sq(v), 0.0));
    if (v_norm < 1e-15) return x;

    double lambda_x = 2.0 / std::max(1.0 - c * x2, 1e-15);
    double scale = std::tanh(std::sqrt(c) * lambda_x * v_norm / 2.0) / (std::sqrt(c) * v_norm);

    std::vector<double> step = v;
    for (auto& val : step) val *= scale;

    return mobius_add(x, step, c);
}

inline std::vector<double> log_map(const std::vector<double>& x, const std::vector<double>& y, double c = 1.0) {
    if (x.size() != y.size()) throw std::invalid_argument("Dimension mismatch");
    if (c <= 0.0) throw std::invalid_argument("Curvature c must be > 0");

    std::vector<double> neg_x = x;
    for (auto& val : neg_x) val = -val;

    std::vector<double> delta = mobius_add(neg_x, y, c);
    double delta_norm = std::sqrt(std::max(norm_sq(delta), 0.0));
    if (delta_norm < 1e-15) return std::vector<double>(x.size(), 0.0);

    double x2 = norm_sq(x);
    double lambda_x = 2.0 / std::max(1.0 - c * x2, 1e-15);
    double factor = (2.0 / (lambda_x * std::sqrt(c))) * std::atanh(std::min(std::sqrt(c) * delta_norm, 1.0 - 1e-15));

    for (auto& val : delta) val = factor * val / delta_norm;
    return delta;
}

inline std::vector<double> gyro(const std::vector<double>& u, const std::vector<double>& v, const std::vector<double>& w, double c = 1.0) {
    auto uv = mobius_add(u, v, c);
    auto vw = mobius_add(v, w, c);
    auto left = mobius_add(u, vw, c);
    for (auto& val : uv) val = -val;
    return mobius_add(uv, left, c);
}

inline std::vector<double> parallel_transport(const std::vector<double>& x, const std::vector<double>& y, const std::vector<double>& v, double c = 1.0) {
    if (x.size() != y.size() || x.size() != v.size()) throw std::invalid_argument("Dimension mismatch");
    if (c <= 0.0) throw std::invalid_argument("Curvature c must be > 0");

    std::vector<double> neg_x = x;
    for (auto& val : neg_x) val = -val;

    auto gyr = gyro(y, neg_x, v, c);
    double lambda_x = 2.0 / std::max(1.0 - c * norm_sq(x), 1e-15);
    double lambda_y = 2.0 / std::max(1.0 - c * norm_sq(y), 1e-15);
    double scale = lambda_x / lambda_y;

    for (auto& val : gyr) val *= scale;
    return gyr;
}

inline std::vector<double> frechet_mean(const std::vector<std::vector<double>>& points, double c = 1.0, int max_iter = 32, double tol = 1e-6) {
    if (points.empty()) throw std::invalid_argument("Points set cannot be empty");
    if (c <= 0.0) throw std::invalid_argument("Curvature c must be > 0");

    size_t dim = points[0].size();
    std::vector<double> mu = project_to_ball(points[0], c);

    for (int iter = 0; iter < std::max(1, max_iter); ++iter) {
        std::vector<double> grad(dim, 0.0);
        for (const auto& p : points) {
            auto lg = log_map(mu, p, c);
            for (size_t i = 0; i < dim; ++i) grad[i] += lg[i];
        }

        double inv = 1.0 / points.size();
        for (size_t i = 0; i < dim; ++i) grad[i] *= inv;

        double g_norm = std::sqrt(std::max(norm_sq(grad), 0.0));
        if (g_norm <= std::max(tol, 1e-15)) break;

        mu = exp_map(mu, grad, c);
        mu = project_to_ball(mu, c);
    }
    return mu;
}

// ==========================================
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================

inline double local_entropy(const std::vector<double>& candidate, const std::vector<std::vector<double>>& neighbors, double c = 1.0) {
    if (neighbors.empty()) return 1.0;
    double total_deviation = 0.0;
    for (const auto& neighbor : neighbors) {
        auto diff = log_map(candidate, neighbor, c);
        total_deviation += std::sqrt(std::max(norm_sq(diff), 0.0));
    }
    double mean_deviation = total_deviation / neighbors.size();
    return 1.0 - std::exp(-mean_deviation);
}

inline double lyapunov_convergence(const std::vector<std::vector<double>>& trajectory, double c = 1.0) {
    if (trajectory.size() < 3) throw std::invalid_argument("Need at least 3 points for convergence trend");
    auto attractor = frechet_mean(trajectory, c, 32, 1e-6);
    double v_diff_sum = 0.0;
    for (size_t i = 0; i < trajectory.size() - 1; ++i) {
        double v_t0 = std::sqrt(std::max(norm_sq(log_map(attractor, trajectory[i], c)), 0.0));
        double v_t1 = std::sqrt(std::max(norm_sq(log_map(attractor, trajectory[i + 1], c)), 0.0));
        v_diff_sum += (v_t1 - v_t0);
    }
    return v_diff_sum / (trajectory.size() - 1);
}

inline std::vector<double> koopman_extrapolate(const std::vector<double>& past, const std::vector<double>& current, double steps, double c = 1.0) {
    auto velocity_at_past = log_map(past, current, c);
    auto velocity_at_current = parallel_transport(past, current, velocity_at_past, c);
    for(auto& v : velocity_at_current) v *= steps;
    return exp_map(current, velocity_at_current, c);
}

inline std::vector<double> context_resonance(const std::vector<double>& thought, const std::vector<double>& global_context, double resonance_factor, double c = 1.0) {
    auto pull_dir = log_map(thought, global_context, c);
    double factor = std::max(0.0, std::min(1.0, resonance_factor));
    for(auto& v : pull_dir) v *= factor;
    return exp_map(thought, pull_dir, c);
}

} // namespace math
} // namespace hyperspace
