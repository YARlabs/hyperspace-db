"""
HyperspaceDB bioSDK / genomicsSDK: Hyperbolic Bioinformatics & Genomic Dynamics Engine.

Provides hyperbolic phylogenetic tree embedding, ancestral state reconstruction via Fréchet mean,
protein domain hierarchy modeling (CATH/SCOP) via Lorentz light cones, and single-cell trajectory
stability analysis on Waddington's epigenetic landscape via Lyapunov exponents and Koopman operators.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
import math
from . import math as hs_math


@dataclass
class PhyloNode:
    name: str
    branch_length: float = 1.0
    children: List['PhyloNode'] = field(default_factory=list)
    metadata: Dict[str, Any] = field(default_factory=dict)


def embed_phylogenetic_tree(
    root: PhyloNode,
    dim: int = 16,
    c: float = 1.0
) -> Dict[str, List[float]]:
    """
    Embeds a phylogenetic tree into the Poincaré Ball manifold.
    Hyperbolic distances directly preserve evolutionary divergence times along the tree.
    Children branches are distributed angularly across hyperbolic geodesics.
    """
    if dim < 2:
        raise ValueError("Poincaré embedding dimension must be >= 2")
    if c <= 0.0:
        raise ValueError("Curvature c must be > 0")

    result: Dict[str, List[float]] = {}
    origin = [0.0] * dim
    result[root.name] = list(origin)

    def _embed_children(
        parent_node: PhyloNode,
        parent_vec: List[float],
        base_angle: float,
        angle_range: float
    ):
        num_children = len(parent_node.children)
        if num_children == 0:
            return

        step_angle = angle_range / max(num_children, 1)
        for idx, child in enumerate(parent_node.children):
            theta = base_angle - (angle_range / 2.0) + (idx + 0.5) * step_angle
            # Map branch length to hyperbolic step tangent magnitude
            d = max(child.branch_length, 0.01)
            # Tangent direction vector in the first two dimensions
            tangent = [0.0] * dim
            tangent[0] = math.cos(theta) * d
            tangent[1] = math.sin(theta) * d
            # Step along geodesic from parent
            child_vec = hs_math.exp_map(parent_vec, tangent, c=c)
            # Project to ball interior to ensure numerical stability
            child_vec = hs_math._project_to_ball(child_vec, c=c)
            result[child.name] = child_vec

            # Subtree gets a narrower angular sector
            _embed_children(child, child_vec, theta, step_angle * 0.9)

    _embed_children(root, origin, 0.0, 2.0 * math.pi)
    return result


def evolutionary_distance(vec_a: Sequence[float], vec_b: Sequence[float], c: float = 1.0) -> float:
    """Computes exact evolutionary divergence distance between two taxa in the Poincaré Ball."""
    return hs_math.poincare_dist(vec_a, vec_b, c=c)


def ancestral_state_reconstruction(
    descendant_vectors: Sequence[Sequence[float]],
    c: float = 1.0,
    max_iter: int = 64
) -> List[float]:
    """
    Reconstructs the ancestral genomic/phenotypic state vector from a set of descendant taxa
    as the Fréchet mean on the hyperbolic manifold.
    """
    if not descendant_vectors:
        raise ValueError("Descendant vectors cannot be empty")
    return hs_math.frechet_mean(descendant_vectors, c=c, max_iter=max_iter)


def protein_domain_lorentz(
    hierarchy_level: int,
    sequence_embedding: Sequence[float],
    level_scale: float = 2.0
) -> List[float]:
    """
    Embeds a protein domain (CATH/SCOP hierarchy) into Lorentz coordinates.
    Root Superfamily (level=1) forms an encompassing light cone that subsumes
    Family (level=2) and Domain (level=3).
    """
    if hierarchy_level < 1:
        raise ValueError("Hierarchy level must be >= 1")
    t = float(hierarchy_level) * level_scale
    spatial = list(sequence_embedding)
    return [t] + spatial


def is_domain_subsumed(
    superfamily_lorentz: Sequence[float],
    domain_lorentz: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Checks whether a protein domain structurally belongs to a superfamily light cone."""
    return hs_math.is_minkowski_subsumed(superfamily_lorentz, domain_lorentz, epsilon=epsilon)


def analyze_differentiation_trajectory(
    cell_trajectory: Sequence[Sequence[float]],
    c: float = 1.0,
    future_steps: float = 1.0
) -> Dict[str, Any]:
    """
    Evaluates single-cell development (e.g. scRNA-seq) on Waddington's epigenetic landscape.
    - Uses Lyapunov exponent to detect convergence towards a terminal cell type attractor vs instability.
    - Extrapolates future differentiation state via Koopman operator linearization.
    """
    if len(cell_trajectory) < 3:
        raise ValueError("Trajectory must contain at least 3 points along pseudotime")

    lyapunov_trend = hs_math.lyapunov_convergence(cell_trajectory, c=c)
    future_state = hs_math.koopman_extrapolate(
        cell_trajectory[-2],
        cell_trajectory[-1],
        steps=future_steps,
        c=c
    )

    is_stable = lyapunov_trend < 0.0
    fate_description = "converging_differentiation" if is_stable else "chaotic_or_oncogenic_destabilization"

    return {
        "is_converging": is_stable,
        "lyapunov_derivative": lyapunov_trend,
        "predicted_future_state": future_state,
        "fate": fate_description
    }


class BioSDK:
    """
    High-level Genomics & Bio-Dynamical SDK for HyperspaceDB.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "genomics", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def embed_tree(self, root: PhyloNode, dim: int = 16) -> Dict[str, List[float]]:
        return embed_phylogenetic_tree(root, dim=dim, c=self.c)

    def taxa_distance(self, vec_a: Sequence[float], vec_b: Sequence[float]) -> float:
        return evolutionary_distance(vec_a, vec_b, c=self.c)

    def reconstruct_ancestor(self, descendants: Sequence[Sequence[float]]) -> List[float]:
        return ancestral_state_reconstruction(descendants, c=self.c)

    def analyze_trajectory(self, trajectory: Sequence[Sequence[float]], future_steps: float = 1.0) -> Dict[str, Any]:
        return analyze_differentiation_trajectory(trajectory, c=self.c, future_steps=future_steps)

    def index_tree(self, root: PhyloNode, dim: int = 16):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        tree_map = self.embed_tree(root, dim=dim)
        results = []
        for idx, (name, vec) in enumerate(tree_map.items()):
            res = self.client.insert(id=idx + 1, vector=vec, metadata={"taxon": name}, collection=self.collection)
            results.append(res)
        return results
