#pragma once

#include <vector>
#include <cmath>
#include <stdexcept>
#include <numeric>
#include <algorithm>
#include <cstring>
#include <openssl/sha.h>

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
// Lorentz Model Math (Hyperboloid)
// ==========================================

/** Computes the Minkowski inner product (Lorentz product) between two vectors. */
inline double lorentz_product(const std::vector<double>& u, const std::vector<double>& v) {
    if (u.empty() || v.empty()) return 0.0;
    double dot = -u[0] * v[0];
    for (size_t i = 1; i < u.size(); ++i) {
        dot += u[i] * v[i];
    }
    return dot;
}

/** Computes the Lorentz distance between two points on the hyperboloid. */
inline double lorentz_dist(const std::vector<double>& u, const std::vector<double>& v) {
    double inner = -lorentz_product(u, v);
    return std::acosh(std::max(inner, 1.0));
}

/** Converts a point from the Lorentz model (Hyperboloid) to the Poincaré Ball model (129 -> 128). */
inline std::vector<double> lorentz_to_poincare(const std::vector<double>& x) {
    if (x.empty()) return {};
    double denom = std::max(1.0 + x[0], 1e-12);
    std::vector<double> proj;
    proj.reserve(x.size() - 1);
    for (size_t i = 1; i < x.size(); ++i) {
        proj.push_back(x[i] / denom);
    }
    return proj;
}

/** Converts a point from the Poincaré Ball model to the Lorentz model (128 -> 129). */
inline std::vector<double> poincare_to_lorentz(const std::vector<double>& p) {
    double p_sq = norm_sq(p);
    double denom = std::max(1.0 - p_sq, 1e-12);
    std::vector<double> x;
    x.reserve(p.size() + 1);
    x.push_back((1.0 + p_sq) / denom);
    for (double pi : p) {
        x.push_back(2.0 * pi / denom);
    }
    return x;
}

/** Ensures a vector satisfies the Lorentz constraint -x0^2 + |x|^2 = -1 (stabilization). */
inline std::vector<double> project_to_hyperboloid(std::vector<double> v) {
    if (v.empty()) return {};
    double spatial_norm_sq = 0.0;
    for (size_t i = 1; i < v.size(); ++i) {
        spatial_norm_sq += v[i] * v[i];
    }
    v[0] = std::sqrt(1.0 + spatial_norm_sq);
    return v;
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

inline std::vector<uint8_t> sha256_hash(const std::vector<uint8_t>& data) {
    std::vector<uint8_t> hash(SHA256_DIGEST_LENGTH);
    SHA256_CTX sha256;
    SHA256_Init(&sha256);
    SHA256_Update(&sha256, data.data(), data.size());
    SHA256_Final(hash.data(), &sha256);
    return hash;
}

inline std::vector<std::vector<double>> generate_orthogonal_matrix(size_t dimension, const std::vector<uint8_t>& seed_bytes) {
    std::vector<std::vector<double>> matrix(dimension, std::vector<double>(dimension, 0.0));
    std::vector<uint8_t> current_hash = sha256_hash(seed_bytes);
    size_t hash_offset = 0;

    for (size_t i = 0; i < dimension; ++i) {
        for (size_t j = 0; j < dimension; ++j) {
            if (hash_offset >= 32) {
                current_hash = sha256_hash(current_hash);
                hash_offset = 0;
            }
            uint32_t val_uint = 0;
            std::memcpy(&val_uint, &current_hash[hash_offset], 4);
            double val = (static_cast<double>(val_uint) / 4294967296.0) * 2.0 - 1.0;
            matrix[i][j] = val;
            hash_offset += 4;
        }
    }

    // Gram-Schmidt with Reorthogonalization
    for (size_t i = 0; i < dimension; ++i) {
        std::vector<double> v = matrix[i];
        for (size_t j = 0; j < i; ++j) {
            const auto& u = matrix[j];
            double u_dot_v = dot(u, v);
            for (size_t k = 0; k < dimension; ++k) {
                v[k] -= u_dot_v * u[k];
            }
            u_dot_v = dot(u, v);
            for (size_t k = 0; k < dimension; ++k) {
                v[k] -= u_dot_v * u[k];
            }
        }
        double v_norm = std::sqrt(std::max(norm_sq(v), 0.0));
        if (v_norm > 1e-15) {
            for (size_t k = 0; k < dimension; ++k) {
                matrix[i][k] = v[k] / v_norm;
            }
        } else {
            for (size_t k = 0; k < dimension; ++k) {
                matrix[i][k] = 0.0;
            }
            matrix[i][i] = 1.0;
        }
    }

    return matrix;
}

inline std::vector<std::vector<double>> generate_lorentz_matrix(size_t dimension, const std::vector<uint8_t>& seed_bytes) {
    size_t d = dimension - 1;

    std::vector<uint8_t> r_seed = seed_bytes;
    std::string spatial_tag = "spatial";
    r_seed.insert(r_seed.end(), spatial_tag.begin(), spatial_tag.end());
    auto R = generate_orthogonal_matrix(d, r_seed);

    std::vector<uint8_t> b_seed = seed_bytes;
    std::string boost_tag = "boost";
    b_seed.insert(b_seed.end(), boost_tag.begin(), boost_tag.end());
    std::vector<double> beta(d, 0.0);
    std::vector<uint8_t> current_hash = sha256_hash(b_seed);
    size_t hash_offset = 0;

    for (size_t i = 0; i < d; ++i) {
        if (hash_offset >= 32) {
            current_hash = sha256_hash(current_hash);
            hash_offset = 0;
        }
        uint32_t val_uint = 0;
        std::memcpy(&val_uint, &current_hash[hash_offset], 4);
        double val = (static_cast<double>(val_uint) / 4294967296.0) * 2.0 - 1.0;
        beta[i] = val;
        hash_offset += 4;
    }

    double beta_norm = std::sqrt(std::max(norm_sq(beta), 0.0));
    if (beta_norm > 1e-15) {
        for (size_t i = 0; i < d; ++i) {
            beta[i] = (beta[i] / beta_norm) * 0.1;
        }
    } else {
        std::fill(beta.begin(), beta.end(), 0.0);
        beta[0] = 0.1;
    }

    double beta_sq = dot(beta, beta);
    double gamma = 1.0 / std::sqrt(1.0 - beta_sq);

    std::vector<std::vector<double>> Lambda_B(dimension, std::vector<double>(dimension, 0.0));
    Lambda_B[0][0] = gamma;
    for (size_t j = 1; j < dimension; ++j) {
        Lambda_B[0][j] = -gamma * beta[j - 1];
        Lambda_B[j][0] = -gamma * beta[j - 1];
    }

    double factor = (gamma - 1.0) / beta_sq;
    for (size_t i = 1; i < dimension; ++i) {
        for (size_t j = 1; j < dimension; ++j) {
            double delta = i == j ? 1.0 : 0.0;
            Lambda_B[i][j] = delta + factor * beta[i - 1] * beta[j - 1];
        }
    }

    std::vector<std::vector<double>> Lambda_R(dimension, std::vector<double>(dimension, 0.0));
    Lambda_R[0][0] = 1.0;
    for (size_t i = 1; i < dimension; ++i) {
        for (size_t j = 1; j < dimension; ++j) {
            Lambda_R[i][j] = R[i - 1][j - 1];
        }
    }

    std::vector<std::vector<double>> Lambda(dimension, std::vector<double>(dimension, 0.0));
    for (size_t i = 0; i < dimension; ++i) {
        for (size_t j = 0; j < dimension; ++j) {
            double sum = 0.0;
            for (size_t k = 0; k < dimension; ++k) {
                sum += Lambda_B[i][k] * Lambda_R[k][j];
            }
            Lambda[i][j] = sum;
        }
    }

    return Lambda;
}

inline std::vector<double> project_vector(const std::vector<double>& v, const std::vector<std::vector<double>>& matrix) {
    size_t dim = v.size();
    std::vector<double> projected(dim, 0.0);
    for (size_t j = 0; j < dim; ++j) {
        double sum = 0.0;
        for (size_t i = 0; i < dim; ++i) {
            sum += v[i] * matrix[i][j];
        }
        projected[j] = sum;
    }
    return projected;
}

inline std::vector<double> inject_anisotropic_noise(const std::vector<double>& v, const std::vector<uint8_t>& seed_bytes, double sigma) {
    if (v.empty() || sigma <= 0.0) return v;
    size_t dim = v.size();
    std::vector<double> noise(dim, 0.0);
    std::vector<uint8_t> current_hash = sha256_hash(seed_bytes);
    size_t hash_offset = 0;

    for (size_t i = 0; i < dim; ++i) {
        if (hash_offset >= 32) {
            current_hash = sha256_hash(current_hash);
            hash_offset = 0;
        }
        uint32_t val_uint = 0;
        std::memcpy(&val_uint, &current_hash[hash_offset], 4);
        double val = (static_cast<double>(val_uint) / 4294967296.0) * 2.0 - 1.0;
        noise[i] = val;
        hash_offset += 4;
    }

    double v_norm = std::sqrt(std::max(norm_sq(v), 0.0));
    if (v_norm > 1e-15) {
        double proj_length = dot(noise, v) / (v_norm * v_norm);
        for (size_t i = 0; i < dim; ++i) {
            noise[i] -= proj_length * v[i];
        }
    }

    double noise_norm = std::sqrt(std::max(norm_sq(noise), 0.0));
    if (noise_norm > 1e-15) {
        double scale = sigma * v_norm / noise_norm;
        std::vector<double> result(dim);
        for (size_t i = 0; i < dim; ++i) {
            result[i] = v[i] + noise[i] * scale;
        }
        return result;
    } else {
        return v;
    }
}

} // namespace math
} // namespace hyperspace
