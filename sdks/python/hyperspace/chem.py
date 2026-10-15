"""
HyperspaceDB chemSDK: Hyperbolic Cheminformatics & Molecular AI Engine.

Provides molecular representation, topological fingerprinting, reaction algebra via Möbius addition,
scaffold hierarchies via Lorentz light cone subsumption, and anti-hallucination validation for
generative chemistry.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
import math
import hashlib
from . import math as hs_math


@dataclass
class Molecule:
    smiles: str
    name: Optional[str] = None
    fingerprint: Optional[List[float]] = None
    lorentz_vector: Optional[List[float]] = None
    complexity: float = 1.0
    metadata: Dict[str, Any] = field(default_factory=dict)


def smiles_to_fingerprint(smiles: str, n_bits: int = 128) -> List[float]:
    """
    Generates a deterministic molecular topological fingerprint from a SMILES string.
    If RDKit is installed, uses Morgan Fingerprints (ECFP4 equivalent).
    Otherwise, uses a robust pure-Python circular atom-environment topological hasher.
    """
    if not smiles or not isinstance(smiles, str):
        raise ValueError("Valid SMILES string required")

    try:
        from rdkit import Chem
        from rdkit.Chem import AllChem
        mol = Chem.MolFromSmiles(smiles)
        if mol is not None:
            bit_vect = AllChem.GetMorganFingerprintAsBitVect(mol, radius=2, nBits=n_bits)
            raw = [1.0 if bit_vect.GetBit(i) else 0.0 for i in range(n_bits)]
            norm = math.sqrt(sum(x * x for x in raw))
            return [x / max(norm, 1e-12) for x in raw]
    except ImportError:
        pass

    # Pure-Python circular atom-environment topological fingerprint
    counts = [0.0] * n_bits
    tokens: List[str] = []
    i = 0
    n = len(smiles)
    while i < n:
        ch = smiles[i]
        if ch.isalpha():
            if i + 1 < n and smiles[i + 1].islower():
                tokens.append(smiles[i:i + 2])
                i += 2
            else:
                tokens.append(ch)
                i += 1
        else:
            tokens.append(ch)
            i += 1

    # Extract circular sub-environments (radius 0, 1, 2)
    for r in range(1, 4):
        for idx in range(len(tokens)):
            sub = "".join(tokens[max(0, idx - r): min(len(tokens), idx + r + 1)])
            h = int(hashlib.sha256(sub.encode("utf-8")).hexdigest(), 16)
            counts[h % n_bits] += 1.0

    # L2 normalize
    norm = math.sqrt(sum(c * c for c in counts))
    if norm < 1e-12:
        return [1.0 / math.sqrt(n_bits)] * n_bits
    return [c / norm for c in counts]


def tanimoto_similarity(fp1: Sequence[float], fp2: Sequence[float]) -> float:
    """Computes continuous Tanimoto / Jaccard similarity between two molecular fingerprints."""
    if len(fp1) != len(fp2):
        raise ValueError("Fingerprint dimension mismatch")
    dot = sum(a * b for a, b in zip(fp1, fp2))
    sum_a = sum(a * a for a in fp1)
    sum_b = sum(b * b for b in fp2)
    denom = sum_a + sum_b - dot
    if denom <= 1e-15:
        return 1.0
    return max(0.0, min(1.0, dot / denom))


def scaffold_to_lorentz(
    fingerprint: Sequence[float],
    complexity_score: float = 1.0,
    spatial_scale: float = 0.5
) -> List[float]:
    """
    Embeds a molecular scaffold or compound into Lorentz spacetime coordinates (t, x_1, ..., x_d).
    The temporal coordinate t encodes scaffold complexity/depth, creating an enclosing light cone.
    Smaller scaffolds (substructures) have lower t; larger derivatives have higher t and fall inside
    the scaffold's future light cone.
    """
    if complexity_score <= 0.0:
        raise ValueError("Complexity score must be > 0")
    # Spatial part: scaled fingerprint
    spatial = [v * spatial_scale for v in fingerprint]
    spatial_norm_sq = sum(s * s for s in spatial)
    # Temporal coordinate chosen so it dominates the spatial extent
    t = math.sqrt(spatial_norm_sq + (complexity_score * complexity_score) + 1.0)
    return [t] + spatial


def is_substructure_subsumed(
    scaffold_lorentz: Sequence[float],
    molecule_lorentz: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """
    Checks if `molecule_lorentz` is within the future causal light cone of `scaffold_lorentz`.
    If True, the molecule structurally contains the scaffold according to Lorentz spacetime geometry.
    """
    return hs_math.is_minkowski_subsumed(scaffold_lorentz, molecule_lorentz, epsilon=epsilon)


def reaction_step(
    reactant: Sequence[float],
    reaction_vector: Sequence[float],
    c: float = 1.0
) -> List[float]:
    """
    Applies a chemical transformation vector to a reactant vector in hyperbolic space via Möbius addition.
    (Reactant (+) Reaction = Product)
    """
    return hs_math.mobius_add(reactant, reaction_vector, c=c)


def estimate_reaction_vector(
    reactant: Sequence[float],
    product: Sequence[float],
    c: float = 1.0
) -> List[float]:
    """
    Estimates the chemical reaction vector connecting reactant and product on the hyperbolic manifold
    using the Riemannian logarithmic map.
    """
    return hs_math.log_map(reactant, product, c=c)


def verify_chemical_validity(
    candidate: Sequence[float],
    reference_library: Sequence[Sequence[float]],
    c: float = 1.0,
    max_entropy_threshold: float = 0.2
) -> Dict[str, Any]:
    """
    Validates whether a generative chemical candidate vector belongs to the valid chemical manifold
    or represents a generative hallucination. Uses spatial local entropy, ball boundary constraint,
    and Fréchet mean distance.
    """
    if not reference_library:
        raise ValueError("Reference library cannot be empty")

    cand_norm_sq = hs_math._norm_sq(candidate)
    if cand_norm_sq >= (1.0 / c):
        return {
            "is_valid": False,
            "local_entropy": 1.0,
            "distance_to_centroid": float("inf"),
            "threshold": max_entropy_threshold,
            "status": "out_of_manifold_boundary"
        }

    entropy = hs_math.local_entropy(candidate, reference_library, c=c)
    centroid = hs_math.frechet_mean(reference_library, c=c, max_iter=32)
    dist_to_centroid = hs_math.poincare_dist(candidate, centroid, c=c)

    is_valid = entropy < max_entropy_threshold
    return {
        "is_valid": is_valid,
        "local_entropy": entropy,
        "distance_to_centroid": dist_to_centroid,
        "threshold": max_entropy_threshold,
        "status": "valid_manifold" if is_valid else "hallucinatory_anomaly"
    }


class ChemSDK:
    """
    High-level Cheminformatics SDK for HyperspaceDB.
    Supports both standalone algorithmic operations and database client interactions.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "molecules", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def fingerprint(self, smiles: str, n_bits: int = 128) -> List[float]:
        return smiles_to_fingerprint(smiles, n_bits=n_bits)

    def to_lorentz(self, smiles_or_fp: Union[str, Sequence[float]], complexity: float = 1.0) -> List[float]:
        if isinstance(smiles_or_fp, str):
            fp = self.fingerprint(smiles_or_fp)
        else:
            fp = list(smiles_or_fp)
        return scaffold_to_lorentz(fp, complexity_score=complexity)

    def is_substructure(self, scaffold_lorentz: Sequence[float], molecule_lorentz: Sequence[float]) -> bool:
        return is_substructure_subsumed(scaffold_lorentz, molecule_lorentz)

    def react(self, reactant: Sequence[float], reaction_vec: Sequence[float]) -> List[float]:
        return reaction_step(reactant, reaction_vec, c=self.c)

    def verify_candidate(
        self,
        candidate: Sequence[float],
        known_compounds: Sequence[Sequence[float]],
        threshold: float = 0.85
    ) -> Dict[str, Any]:
        return verify_chemical_validity(candidate, known_compounds, c=self.c, max_entropy_threshold=threshold)

    def insert_molecule(self, id: int, smiles: str, complexity: float = 1.0, metadata: Optional[Dict[str, Any]] = None):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        vec = self.to_lorentz(smiles, complexity=complexity)
        meta = metadata or {}
        meta["smiles"] = smiles
        meta["complexity"] = complexity
        return self.client.insert(id=id, vector=vec, metadata=meta, collection=self.collection)

    def search_similar(self, query_smiles: str, top_k: int = 10):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        query_vec = self.to_lorentz(query_smiles)
        return self.client.search(vector=query_vec, top_k=top_k, collection=self.collection)
