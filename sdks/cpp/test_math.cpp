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

int main() {
    try {
        test_mobius_add_identity();
        test_exp_log_roundtrip_small_step();
        test_cognitive_math();
        test_lorentz_product_and_distance();
        std::cout << "All C++ math tests passed!" << std::endl;
        return 0;
    } catch (const std::exception& e) {
        std::cerr << "Test failed with exception: " << e.what() << std::endl;
        return 1;
    }
}
