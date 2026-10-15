import unittest
import math
from hyperspace import (
    chem,
    ChemSDK,
    Molecule,
    smiles_to_fingerprint,
    tanimoto_similarity,
    scaffold_to_lorentz,
    is_substructure_subsumed,
    reaction_step,
    estimate_reaction_vector,
    verify_chemical_validity,
    poincare_dist
)


class TestChemSDK(unittest.TestCase):
    def setUp(self):
        self.benzene_smiles = "c1ccccc1"
        self.toluene_smiles = "Cc1ccccc1"
        self.phenol_smiles = "Oc1ccccc1"
        self.aspirin_smiles = "CC(=O)Oc1ccccc1C(=O)O"
        self.caffeine_smiles = "CN1C=NC2=C1C(=O)N(C(=O)N2C)C"

    def test_smiles_to_fingerprint(self):
        fp_benzene = smiles_to_fingerprint(self.benzene_smiles, n_bits=64)
        self.assertEqual(len(fp_benzene), 64)
        norm = math.sqrt(sum(x * x for x in fp_benzene))
        self.assertAlmostEqual(norm, 1.0, places=4)

        # Invalid SMILES
        with self.assertRaises(ValueError):
            smiles_to_fingerprint("")
        with self.assertRaises(ValueError):
            smiles_to_fingerprint(None)  # type: ignore

    def test_tanimoto_similarity(self):
        fp1 = smiles_to_fingerprint(self.benzene_smiles, n_bits=64)
        fp2 = smiles_to_fingerprint(self.toluene_smiles, n_bits=64)
        fp_caffeine = smiles_to_fingerprint(self.caffeine_smiles, n_bits=64)

        # Identity
        self.assertAlmostEqual(tanimoto_similarity(fp1, fp1), 1.0, places=5)

        # Toluene is structurally closer to benzene than caffeine is
        sim_tol = tanimoto_similarity(fp1, fp2)
        sim_caff = tanimoto_similarity(fp1, fp_caffeine)
        self.assertGreater(sim_tol, sim_caff)
        self.assertTrue(0.0 <= sim_tol <= 1.0)

    def test_scaffold_to_lorentz_and_subsumption(self):
        # Benzene is the core scaffold (complexity=1.0)
        # Toluene has a methyl group added (complexity=1.8, close spatial)
        fp_benzene = smiles_to_fingerprint(self.benzene_smiles, n_bits=32)
        scaffold_cone = scaffold_to_lorentz(fp_benzene, complexity_score=1.0, spatial_scale=0.5)

        # Child molecule (toluene): higher complexity, slight spatial perturbation
        toluene_fp = list(fp_benzene)
        toluene_fp[0] += 0.05
        molecule_cone = scaffold_to_lorentz(toluene_fp, complexity_score=2.2, spatial_scale=0.5)

        # Subsumption holds: benzene scaffold subsumes toluene
        self.assertTrue(is_substructure_subsumed(scaffold_cone, molecule_cone))

        # Reverse should NOT hold: toluene does not subsume its parent scaffold
        self.assertFalse(is_substructure_subsumed(molecule_cone, scaffold_cone))

        # Non-related molecule far away
        unrelated_fp = [-x for x in fp_benzene]
        unrelated_cone = scaffold_to_lorentz(unrelated_fp, complexity_score=1.2, spatial_scale=0.5)
        self.assertFalse(is_substructure_subsumed(scaffold_cone, unrelated_cone))

    def test_reaction_algebra(self):
        dim = 16
        c = 1.0
        reactant = [0.1] * dim
        # Reaction vector in tangent space / hyperbolic displacement
        reaction = [0.05] * dim

        product = reaction_step(reactant, reaction, c=c)
        self.assertEqual(len(product), dim)
        self.assertNotEqual(reactant, product)

        # Estimate reaction vector back
        estimated_reaction = estimate_reaction_vector(reactant, product, c=c)
        self.assertEqual(len(estimated_reaction), dim)
        # Reaction step along estimated vector should recover product
        from hyperspace import exp_map
        recovered = exp_map(reactant, estimated_reaction, c=c)
        dist = poincare_dist(recovered, product, c=c)
        self.assertAlmostEqual(dist, 0.0, places=5)

    def test_verify_chemical_validity(self):
        dim = 8
        c = 1.0
        # Reference library of valid chemical embeddings in Poincaré ball
        library = [
            [0.1, 0.1, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0],
            [0.12, 0.09, 0.01, -0.01, 0.04, 0.06, 0.0, 0.01],
            [0.08, 0.11, -0.01, 0.01, 0.06, 0.04, 0.01, -0.01],
            [0.1, 0.13, 0.02, 0.0, 0.05, 0.05, -0.01, 0.0],
        ]

        # Valid candidate close to the manifold
        valid_candidate = [0.11, 0.10, 0.0, 0.0, 0.05, 0.05, 0.0, 0.0]
        res_valid = verify_chemical_validity(valid_candidate, library, c=c, max_entropy_threshold=0.1)
        self.assertTrue(res_valid["is_valid"])
        self.assertEqual(res_valid["status"], "valid_manifold")
        self.assertLess(res_valid["local_entropy"], 0.1)

        # Outlier candidate (generative hallucination on opposite side of manifold)
        hallucination = [-0.6, -0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        res_invalid = verify_chemical_validity(hallucination, library, c=c, max_entropy_threshold=0.1)
        self.assertFalse(res_invalid["is_valid"])
        self.assertEqual(res_invalid["status"], "hallucinatory_anomaly")

        # Candidate beyond Poincaré ball boundary
        boundary_violator = [1.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        res_boundary = verify_chemical_validity(boundary_violator, library, c=c)
        self.assertFalse(res_boundary["is_valid"])
        self.assertEqual(res_boundary["status"], "out_of_manifold_boundary")

    def test_chem_sdk_class(self):
        sdk = ChemSDK(c=1.0)
        fp = sdk.fingerprint("c1ccccc1", n_bits=32)
        self.assertEqual(len(fp), 32)
        cone = sdk.to_lorentz(fp, complexity=1.5)
        self.assertEqual(len(cone), 33)

        # Test insert without client raises RuntimeError
        with self.assertRaises(RuntimeError):
            sdk.insert_molecule(1, "c1ccccc1")


if __name__ == "__main__":
    unittest.main()
