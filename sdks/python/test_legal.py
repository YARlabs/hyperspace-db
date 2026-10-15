import unittest
from hyperspace import (
    legal,
    patent,
    LegalSDK,
    patentSDK,
    LegalNorm,
    norm_to_lorentz_cone,
    is_norm_valid_under,
    patent_claim_cone,
    is_claim_subsumed,
    doctrine_of_equivalents,
    synthesize_precedents,
    poincare_dist
)


class TestLegalSDK(unittest.TestCase):
    def setUp(self):
        # Kelsen's Pyramid of Law
        self.constitution = LegalNorm(
            norm_id=1,
            rank=1,  # Constitutional
            title="Fundamental Commerce Clause",
            vector=[0.1, 0.05, 0.02, 0.0]
        )
        self.statute = LegalNorm(
            norm_id=2,
            rank=2,  # Statutory Act
            title="Federal Trade Commission Act",
            vector=[0.12, 0.06, 0.03, 0.01]
        )
        self.regulation = LegalNorm(
            norm_id=3,
            rank=3,  # Regulatory Rule
            title="16 CFR Consumer Protection Standards",
            vector=[0.13, 0.07, 0.035, 0.015]
        )
        self.unconstitutional_rule = LegalNorm(
            norm_id=4,
            rank=3,  # Regulation attempting to overreach
            title="Ultra Vires State Overreach Rule",
            vector=[12.0, -12.0, 8.0, -8.0]
        )

    def test_norm_to_lorentz_cone(self):
        cone = norm_to_lorentz_cone(self.constitution, rank_scale=3.0)
        self.assertEqual(len(cone), 5)
        self.assertAlmostEqual(cone[0], 3.0)
        self.assertEqual(cone[1:], [0.1, 0.05, 0.02, 0.0])

        with self.assertRaises(ValueError):
            norm_to_lorentz_cone(LegalNorm(norm_id=0, rank=0, title="invalid", vector=[0.1]))

    def test_normative_hierarchy_validity(self):
        v_const = norm_to_lorentz_cone(self.constitution, rank_scale=5.0)
        v_statute = norm_to_lorentz_cone(self.statute, rank_scale=5.0)
        v_reg = norm_to_lorentz_cone(self.regulation, rank_scale=5.0)
        v_ultra = norm_to_lorentz_cone(self.unconstitutional_rule, rank_scale=5.0)

        # Constitution authorizes Statute
        self.assertTrue(is_norm_valid_under(v_const, v_statute))

        # Statute authorizes Administrative Regulation
        self.assertTrue(is_norm_valid_under(v_statute, v_reg))

        # Constitution transitively authorizes Regulation
        self.assertTrue(is_norm_valid_under(v_const, v_reg))

        # Reverse is false: subordinate regulation cannot authorize superior constitution
        self.assertFalse(is_norm_valid_under(v_reg, v_const))

        # Ultra Vires regulation fails validity cone check
        self.assertFalse(is_norm_valid_under(v_const, v_ultra))

    def test_patent_claim_scope(self):
        # Claim 1: Independent Claim (Broad)
        claim_1_vec = [0.2, 0.1, 0.05, 0.0]
        cone_claim_1 = patent_claim_cone(claim_type=1, claim_vector=claim_1_vec, scale=4.0)

        # Claim 2: Dependent Claim (Narrower, adds limitation)
        claim_2_vec = [0.22, 0.11, 0.06, 0.01]
        cone_claim_2 = patent_claim_cone(claim_type=2, claim_vector=claim_2_vec, scale=4.0)

        # Accused Product (matches dependent claim specification)
        product_vec = [0.23, 0.12, 0.065, 0.012]
        cone_product = patent_claim_cone(claim_type=3, claim_vector=product_vec, scale=4.0)

        # Independent claim subsumes dependent claim
        self.assertTrue(is_claim_subsumed(cone_claim_1, cone_claim_2))

        # Independent claim subsumes product
        self.assertTrue(is_claim_subsumed(cone_claim_1, cone_product))

        # Reverse does not hold
        self.assertFalse(is_claim_subsumed(cone_product, cone_claim_1))

    def test_doctrine_of_equivalents(self):
        c = 1.0
        # Patented functional claim
        claim_manifold = [0.1, 0.1, 0.05, 0.0]

        # Accused device with trivial substitution (insubstantial differences)
        equivalent_device = [0.12, 0.09, 0.052, 0.008]
        res_equiv = doctrine_of_equivalents(claim_manifold, equivalent_device, c=c, equivalence_threshold=0.3)
        self.assertTrue(res_equiv["is_equivalent_infringement"])
        self.assertEqual(res_equiv["verdict"], "SUBSTANTIAL_EQUIVALENCE")
        self.assertLessEqual(res_equiv["hyperbolic_distance"], 0.3)

        # Distinct device with substantial functional difference
        distinct_device = [-0.4, -0.4, 0.1, 0.0]
        res_distinct = doctrine_of_equivalents(claim_manifold, distinct_device, c=c, equivalence_threshold=0.3)
        self.assertFalse(res_distinct["is_equivalent_infringement"])
        self.assertEqual(res_distinct["verdict"], "LACKS_EQUIVALENCE")
        self.assertGreater(res_distinct["hyperbolic_distance"], 0.3)

    def test_precedent_synthesis(self):
        c = 1.0
        # Judicial opinions from multiple circuits on a single statutory question
        opinions = [
            [0.10, 0.05, 0.02, 0.01],
            [0.12, 0.04, 0.01, 0.02],
            [0.09, 0.06, 0.03, 0.00],
            [0.11, 0.05, 0.02, 0.01],
        ]

        consensus = synthesize_precedents(opinions, c=c)
        self.assertEqual(len(consensus), 4)

        # Consensus must be close to all opinions
        for op in opinions:
            d = poincare_dist(consensus, op, c=c)
            self.assertLess(d, 0.1)

    def test_legal_sdk_class_and_aliases(self):
        self.assertIs(patentSDK, LegalSDK)

        sdk = LegalSDK(c=1.0)
        cone = sdk.to_cone(self.constitution)
        self.assertEqual(len(cone), 5)

        with self.assertRaises(RuntimeError):
            sdk.index_norm(self.constitution)


if __name__ == "__main__":
    unittest.main()
