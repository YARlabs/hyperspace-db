import unittest
from hyperspace import HyperspaceClient, VerificationResult

class TestVerification(unittest.TestCase):
    def test_verification_result_structure(self):
        res = VerificationResult(
            trust_score=0.92,
            is_hallucination=False,
            is_valid=True,
            min_distance=0.15,
            avg_distance=0.28,
            geodesic_distance=0.35,
            geodesic_hops=2,
            cone_satisfied=True,
            details="Topologically sound",
            resolved_context_ids=[1, 2]
        )
        self.assertEqual(res.trust_score, 0.92)
        self.assertFalse(res.is_hallucination)
        self.assertTrue(res.is_valid)
        self.assertEqual(res.resolved_context_ids, [1, 2])

    def test_client_has_verify_generation(self):
        self.assertTrue(hasattr(HyperspaceClient, "verify_generation"))

if __name__ == "__main__":
    unittest.main()
