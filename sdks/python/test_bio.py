import unittest
import math
from hyperspace import (
    bio,
    BioSDK,
    PhyloNode,
    embed_phylogenetic_tree,
    evolutionary_distance,
    ancestral_state_reconstruction,
    protein_domain_lorentz,
    is_domain_subsumed,
    analyze_differentiation_trajectory,
    poincare_dist
)


class TestBioSDK(unittest.TestCase):
    def setUp(self):
        # Create a sample phylogenetic tree:
        # Primates -> (Hominidae -> (Homo, Pan), Hylobatidae)
        self.homo = PhyloNode(name="Homo", branch_length=0.2)
        self.pan = PhyloNode(name="Pan", branch_length=0.2)
        self.hominidae = PhyloNode(name="Hominidae", branch_length=0.4, children=[self.homo, self.pan])
        self.hylobatidae = PhyloNode(name="Hylobatidae", branch_length=0.6)
        self.primates = PhyloNode(name="Primates", branch_length=0.0, children=[self.hominidae, self.hylobatidae])

    def test_phylogenetic_tree_embedding(self):
        dim = 8
        c = 1.0
        embeddings = embed_phylogenetic_tree(self.primates, dim=dim, c=c)

        self.assertIn("Primates", embeddings)
        self.assertIn("Hominidae", embeddings)
        self.assertIn("Homo", embeddings)
        self.assertIn("Pan", embeddings)
        self.assertIn("Hylobatidae", embeddings)

        # Root at origin
        root_vec = embeddings["Primates"]
        self.assertEqual(len(root_vec), dim)
        self.assertAlmostEqual(math.sqrt(sum(x * x for x in root_vec)), 0.0, places=5)

        # All embedded nodes must be strictly inside the Poincaré Ball (||x|| < 1/sqrt(c))
        for name, vec in embeddings.items():
            self.assertEqual(len(vec), dim)
            norm = math.sqrt(sum(x * x for x in vec))
            self.assertLess(norm, 1.0 / math.sqrt(c), f"{name} is outside Poincaré ball")

    def test_evolutionary_distance_properties(self):
        embeddings = embed_phylogenetic_tree(self.primates, dim=8, c=1.0)
        homo_vec = embeddings["Homo"]
        pan_vec = embeddings["Pan"]
        gibbon_vec = embeddings["Hylobatidae"]

        # Symmetry
        d_hp = evolutionary_distance(homo_vec, pan_vec)
        d_ph = evolutionary_distance(pan_vec, homo_vec)
        self.assertAlmostEqual(d_hp, d_ph, places=5)

        # Homo and Pan are sister taxa (Hominidae); distance between them should be smaller
        # than distance between Homo and Hylobatidae (Gibbon)
        d_hg = evolutionary_distance(homo_vec, gibbon_vec)
        self.assertLess(d_hp, d_hg)

        # Triangle inequality: d(Homo, Hylobatidae) <= d(Homo, Pan) + d(Pan, Hylobatidae)
        d_pg = evolutionary_distance(pan_vec, gibbon_vec)
        self.assertLessEqual(d_hg, d_hp + d_pg + 1e-7)

    def test_ancestral_state_reconstruction(self):
        embeddings = embed_phylogenetic_tree(self.primates, dim=8, c=1.0)
        descendants = [embeddings["Homo"], embeddings["Pan"]]
        ancestor = ancestral_state_reconstruction(descendants, c=1.0)

        self.assertEqual(len(ancestor), 8)
        norm = math.sqrt(sum(x * x for x in ancestor))
        self.assertLess(norm, 1.0)

        # Ancestor (Fréchet mean) should be equidistant or close to both descendants
        d1 = poincare_dist(ancestor, descendants[0])
        d2 = poincare_dist(ancestor, descendants[1])
        self.assertAlmostEqual(d1, d2, places=4)

    def test_protein_domain_hierarchy(self):
        superfamily_emb = [0.1, 0.2, 0.0, 0.1]
        superfamily_cone = protein_domain_lorentz(hierarchy_level=1, sequence_embedding=superfamily_emb, level_scale=5.0)

        # Family belongs to superfamily: level 2, slight feature shift
        family_emb = [0.12, 0.21, 0.01, 0.09]
        family_cone = protein_domain_lorentz(hierarchy_level=2, sequence_embedding=family_emb, level_scale=5.0)

        # Superfamily subsumes family
        self.assertTrue(is_domain_subsumed(superfamily_cone, family_cone))

        # Reverse should not hold
        self.assertFalse(is_domain_subsumed(family_cone, superfamily_cone))

        # Unrelated domain far away
        unrelated_emb = [10.0, -10.0, 5.0, -5.0]
        unrelated_cone = protein_domain_lorentz(hierarchy_level=2, sequence_embedding=unrelated_emb, level_scale=5.0)
        self.assertFalse(is_domain_subsumed(superfamily_cone, unrelated_cone))

    def test_differentiation_trajectory(self):
        dim = 4
        c = 1.0
        # Converging trajectory towards an attractor state [0.0, 0.0, 0.0, 0.0]
        converging_trajectory = [
            [0.4, 0.3, 0.2, 0.1],
            [0.2, 0.15, 0.1, 0.05],
            [0.08, 0.05, 0.03, 0.01],
            [0.02, 0.01, 0.005, 0.002],
        ]

        res = analyze_differentiation_trajectory(converging_trajectory, c=c)
        self.assertTrue(res["is_converging"])
        self.assertLess(res["lyapunov_derivative"], 0.0)
        self.assertEqual(res["fate"], "converging_differentiation")
        self.assertEqual(len(res["predicted_future_state"]), dim)

    def test_bio_sdk_class(self):
        sdk = BioSDK(c=1.0)
        tree_map = sdk.embed_tree(self.primates, dim=6)
        self.assertEqual(len(tree_map), 5)

        with self.assertRaises(RuntimeError):
            sdk.index_tree(self.primates)


if __name__ == "__main__":
    unittest.main()
