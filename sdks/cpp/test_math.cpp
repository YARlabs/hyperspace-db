#include <iostream>
#include <cassert>
#include <cmath>
#include "hyperspace/math.hpp"

using namespace hyperspace::math;

void test_mobius_add_identity() {
    std::vector<double> x = {0.1, -0.2, 0.05};
    std::vector<double> zero = {0.0, 0.0, 0.0};
    auto out = mobius_add(x, zero, 1.0);
    assert(out.size() == x.size());
    for (size_t i = 0; i < x.size(); ++i) {
        assert(std::abs(out[i] - x[i]) < 1e-12);
    }
    std::cout << "test_mobius_add_identity passed!" << std::endl;
}

void test_exp_log_roundtrip_small_step() {
    std::vector<double> x = {0.05, -0.03};
    std::vector<double> v = {0.001, 0.002};
    auto y = exp_map(x, v, 1.0);
    auto v_back = log_map(x, y, 1.0);
    assert(v.size() == v_back.size());
    for (size_t i = 0; i < v.size(); ++i) {
        assert(std::abs(v[i] - v_back[i]) < 1e-6);
    }
    std::cout << "test_exp_log_roundtrip_small_step passed!" << std::endl;
}

void test_cognitive_math() {
    // 1. Local Entropy
    std::vector<double> candidate = {0.1, 0.1};
    std::vector<std::vector<double>> neighbors = {
        {0.11, 0.1},
        {0.1, 0.12},
        {0.09, 0.09}
    };
    double entropy = local_entropy(candidate, neighbors, 1.0);
    assert(entropy < 0.1);
    std::cout << "local_entropy passed! entropy: " << entropy << std::endl;

    // 2. Lyapunov Convergence
    std::vector<std::vector<double>> trajectory = {
        {0.5, 0.5},
        {0.3, 0.3},
        {0.1, 0.1},
        {0.05, 0.05}
    };
    double lya = lyapunov_convergence(trajectory, 1.0);
    assert(lya < 0.0);
    std::cout << "lyapunov_convergence passed! lyapunov: " << lya << std::endl;

    // 3. Context Resonance
    std::vector<double> thought = {0.5, 0.0};
    std::vector<double> global_ctx = {0.0, 0.5};
    auto pull = context_resonance(thought, global_ctx, 0.5, 1.0);
    assert(pull.size() == 2);
    std::cout << "context_resonance passed!" << std::endl;

    // 4. Koopman Extrapolation
    std::vector<double> past = {0.1, 0.2};
    std::vector<double> current = {0.15, 0.25};
    auto predicted = koopman_extrapolate(past, current, 1.0, 1.0);
    assert(predicted.size() == 2);
    std::cout << "koopman_extrapolate passed!" << std::endl;
}

void test_lorentz_product_and_distance() {
    std::vector<double> u = {2.0, std::sqrt(3.0)};
    std::vector<double> v = {2.0, std::sqrt(3.0)};
    double dist = lorentz_dist(u, v);
    assert(dist < 1e-4);

    std::vector<double> w = {3.0, std::sqrt(8.0)};
    double product = lorentz_product(u, w);
    double expected = -2.0 * 3.0 + std::sqrt(3.0) * std::sqrt(8.0);
    assert(std::abs(product - expected) < 1e-4);
    std::cout << "test_lorentz_product_and_distance passed!" << std::endl;
}

void test_zk_crypto_preservation() {
    std::string secret = "test-secret-key-cpp";
    std::vector<uint8_t> seed(secret.begin(), secret.end());

    // 1. Orthogonal dimension preservation & dot product preservation
    size_t dim = 4;
    auto mat = generate_orthogonal_matrix(dim, seed);

    std::vector<double> v1 = {0.1, 0.2, 0.3, 0.4};
    std::vector<double> v2 = {0.5, 0.6, 0.7, 0.8};

    double orig_dot = dot(v1, v2);
    auto p1 = project_vector(v1, mat);
    auto p2 = project_vector(v2, mat);
    double proj_dot = dot(p1, p2);

    assert(std::abs(orig_dot - proj_dot) < 1e-9);
    std::cout << "Orthogonal dot product preservation passed!" << std::endl;

    // 2. Lorentz distance preservation
    size_t l_dim = 4;
    auto l_mat = generate_lorentz_matrix(l_dim, seed);

    std::vector<double> lv1 = {2.0, 1.0, 1.0, 0.5}; // Lorentz constraints: -x0^2 + |x|^2 = -1
    std::vector<double> lv2 = {3.0, 2.0, 2.0, 0.0};

    // Correct Lorentz vectors: project to hyperboloid first
    lv1 = project_to_hyperboloid(lv1);
    lv2 = project_to_hyperboloid(lv2);

    double orig_l_dist = lorentz_dist(lv1, lv2);
    auto lp1 = project_vector(lv1, l_mat);
    auto lp2 = project_vector(lv2, l_mat);
    double proj_l_dist = lorentz_dist(lp1, lp2);

    assert(std::abs(orig_l_dist - proj_l_dist) < 1e-9);
    std::cout << "Lorentz distance preservation passed!" << std::endl;

    // 3. Poincare distance preservation
    // Scale vectors down to ensure poincare ball boundary is not violated
    std::vector<double> pv1 = {0.05, -0.02, 0.04};
    std::vector<double> pv2 = {-0.01, 0.03, 0.05};

    auto p_mat = generate_lorentz_matrix(4, seed); // Poincare maps to Lorentz (dim + 1)
    
    // Project pv1 and pv2 using poincare projection flow
    auto poincare_proj = [&](const std::vector<double>& p) {
        auto lor = poincare_to_lorentz(p);
        auto proj_lor = project_vector(lor, p_mat);
        return lorentz_to_poincare(proj_lor);
    };

    auto pp1 = poincare_proj(pv1);
    auto pp2 = poincare_proj(pv2);

    // Poincare distance is calculated via mobius addition / norm
    auto dist_p = [](const std::vector<double>& a, const std::vector<double>& b) {
        std::vector<double> neg_a = a;
        for (auto& x : neg_a) x = -x;
        auto sum = mobius_add(neg_a, b, 1.0);
        double s_norm = std::sqrt(norm_sq(sum));
        return 2.0 * std::atanh(s_norm);
    };

    double orig_p_dist = dist_p(pv1, pv2);
    double proj_p_dist = dist_p(pp1, pp2);

    assert(std::abs(orig_p_dist - proj_p_dist) < 1e-9);
    std::cout << "Poincare distance preservation passed!" << std::endl;
}

int main() {
    try {
        test_mobius_add_identity();
        test_exp_log_roundtrip_small_step();
        test_cognitive_math();
        test_lorentz_product_and_distance();
        test_zk_crypto_preservation();
        std::cout << "All C++ math tests passed!" << std::endl;
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "Test failed with exception: " << e.what() << std::endl;
        return 1;
    }
}
