import unittest
from hyperspace import (
    onto,
    kg,
    OntoSDK,
    kgSDK,
    Triple,
    hyperbolic_translate,
    score_triple,
    concept_subsumption_cone,
    is_concept_subsumed,
    align_ontologies,
    poincare_dist
)


class TestOntoSDK(unittest.TestCase):
    def setUp(self):
        self.c = 1.0
        # Sample entities and relation in Poincaré ball
        # head: "Paris", relation: "capital_of", tail: "France"
        self.head_paris = [0.1, 0.2, 0.05, 0.0]
        self.rel_capital_of = [0.05, 0.1, -0.02, 0.01]
        self.tail_france = hyperbolic_translate(self.head_paris, self.rel_capital_of, c=self.c)

        self.triple = Triple(
            head="Paris",
            relation="capital_of",
            tail="France",
            head_vec=self.head_paris,
            relation_vec=self.rel_capital_of,
            tail_vec=self.tail_france
        )

    def test_hyperbolic_translation(self):
        predicted = hyperbolic_translate(self.head_paris, self.rel_capital_of, c=self.c)
        self.assertEqual(len(predicted), 4)
        dist = poincare_dist(predicted, self.tail_france, c=self.c)
        self.assertAlmostEqual(dist, 0.0, places=5)

    def test_score_triple(self):
        # Valid triple score should be essentially 0
        valid_score = score_triple(self.head_paris, self.rel_capital_of, self.tail_france, c=self.c)
        self.assertAlmostEqual(valid_score, 0.0, places=5)

        # Corrupted triple (swap tail with unrelated entity "Tokyo")
        corrupted_tail = [-0.4, -0.3, 0.2, 0.1]
        corrupted_score = score_triple(self.head_paris, self.rel_capital_of, corrupted_tail, c=self.c)
        self.assertGreater(corrupted_score, 0.5)

    def test_concept_subsumption_taxonomy(self):
        # Living Thing (depth 1) -> Mammal (depth 2) -> Canine (depth 3)
        v_living = [0.1, 0.1, 0.05, 0.0]
        cone_living = concept_subsumption_cone(taxonomy_depth=1, concept_vector=v_living, depth_scale=4.0)

        v_mammal = [0.12, 0.11, 0.06, 0.01]
        cone_mammal = concept_subsumption_cone(taxonomy_depth=2, concept_vector=v_mammal, depth_scale=4.0)

        v_canine = [0.13, 0.12, 0.065, 0.015]
        cone_canine = concept_subsumption_cone(taxonomy_depth=3, concept_vector=v_canine, depth_scale=4.0)

        # Mammal IS-A Living Thing
        self.assertTrue(is_concept_subsumed(cone_living, cone_mammal))

        # Canine IS-A Mammal
        self.assertTrue(is_concept_subsumed(cone_mammal, cone_canine))

        # Transitive: Canine IS-A Living Thing
        self.assertTrue(is_concept_subsumed(cone_living, cone_canine))

        # Reverse should fail
        self.assertFalse(is_concept_subsumed(cone_canine, cone_living))

        # Unrelated concept (e.g. "Microprocessor")
        v_chip = [10.0, -10.0, 5.0, -5.0]
        cone_chip = concept_subsumption_cone(taxonomy_depth=2, concept_vector=v_chip, depth_scale=4.0)
        self.assertFalse(is_concept_subsumed(cone_living, cone_chip))

    def test_align_ontologies(self):
        # Concept in Ontology A vs matching Concept in Ontology B
        concept_a = [0.2, 0.1, 0.0, 0.0]
        concept_b = [0.3, 0.2, 0.05, 0.01]

        initial_dist = poincare_dist(concept_a, concept_b, c=self.c)
        aligned = align_ontologies(concept_a, concept_b, resonance_factor=0.6, c=self.c)
        aligned_dist = poincare_dist(aligned, concept_b, c=self.c)

        # Resonated concept is pulled closer to the target concept along the geodesic
        self.assertLess(aligned_dist, initial_dist)

    def test_onto_sdk_class_and_aliases(self):
        self.assertIs(kgSDK, OntoSDK)

        sdk = OntoSDK(c=1.0)
        tail_pred = sdk.predict_tail(self.head_paris, self.rel_capital_of)
        self.assertEqual(len(tail_pred), 4)

        with self.assertRaises(RuntimeError):
            sdk.index_triple(self.triple)


if __name__ == "__main__":
    unittest.main()
