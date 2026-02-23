import unittest
from hyperspace.math import local_entropy, lyapunov_convergence, koopman_extrapolate, context_resonance

class TestCognitiveMath(unittest.TestCase):
    def test_local_entropy(self):
        candidate = [0.1, 0.1]
        neighbors = [
            [0.11, 0.1],
            [0.1, 0.12],
            [0.09, 0.09],
        ]
        entropy = local_entropy(candidate, neighbors, c=1.0)
        self.assertLess(entropy, 0.1)

    def test_lyapunov_convergence(self):
        trajectory_converging = [
            [0.5, 0.5],
            [0.3, 0.3],
            [0.1, 0.1],
            [0.05, 0.05]
        ]
        lyapunov = lyapunov_convergence(trajectory_converging, c=1.0)
        self.assertLess(lyapunov, 0.0)

    def test_context_resonance(self):
        thought = [0.5, 0.0]
        global_ctx = [0.0, 0.5]
        pull = context_resonance(thought, global_ctx, resonance_factor=0.5, c=1.0)
        self.assertEqual(len(pull), 2)

    def test_koopman_extrapolate(self):
        past = [0.1, 0.2]
        current = [0.15, 0.25]
        predicted = koopman_extrapolate(past, current, steps=1.0, c=1.0)
        self.assertEqual(len(predicted), 2)

if __name__ == '__main__':
    unittest.main()
