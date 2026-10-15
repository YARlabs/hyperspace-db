"""
HyperspaceDB legalSDK / patentSDK: Normative Hierarchies & Intellectual Property Engine.

Provides Kelsen's normative pyramid validation (Constitution -> Statute -> Regulation -> Clause)
via Lorentz light cones, patent claim scope & Doctrine of Equivalents evaluation in hyperbolic space,
and precedent consensus synthesis (Stare Decisis) via Fréchet mean.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
from . import math as hs_math


@dataclass
class LegalNorm:
    norm_id: Union[str, int]
    rank: int  # 1: Constitutional, 2: Statutory/Federal, 3: Regulatory, 4: Administrative/Contract
    title: str
    vector: Sequence[float]
    jurisdiction: str = "federal"
    metadata: Dict[str, Any] = field(default_factory=dict)


def norm_to_lorentz_cone(
    norm: LegalNorm,
    rank_scale: float = 2.0
) -> List[float]:
    """
    Embeds a legal norm into Lorentz spacetime according to Kelsen's Stufenbaulehre.
    Superior norms (rank=1) establish foundational validity cones; subordinate norms (rank >= 2)
    must reside inside the superior norm's cone to remain constitutionally authorized.
    """
    if norm.rank < 1:
        raise ValueError("Legal norm rank must be >= 1")
    t = float(norm.rank) * rank_scale
    spatial = list(norm.vector)
    return [t] + spatial


def is_norm_valid_under(
    superior_norm_cone: Sequence[float],
    subordinate_norm_cone: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Verifies that a subordinate law, regulation, or contract clause is valid under the superior norm."""
    return hs_math.is_minkowski_subsumed(superior_norm_cone, subordinate_norm_cone, epsilon=epsilon)


def patent_claim_cone(
    claim_type: int,  # 1: Independent Claim (Broad), 2: Dependent Claim, 3: Accused Device/Product
    claim_vector: Sequence[float],
    scale: float = 2.0
) -> List[float]:
    """Embeds patent claims into light-cone geometry to model claim scope and dependent limitations."""
    if claim_type < 1:
        raise ValueError("Claim type must be >= 1")
    t = float(claim_type) * scale
    spatial = list(claim_vector)
    return [t] + spatial


def is_claim_subsumed(
    independent_claim_cone: Sequence[float],
    target_cone: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Checks whether a dependent claim or accused device is strictly subsumed by the patent claim scope."""
    return hs_math.is_minkowski_subsumed(independent_claim_cone, target_cone, epsilon=epsilon)


def doctrine_of_equivalents(
    claim_vector: Sequence[float],
    accused_product_vector: Sequence[float],
    c: float = 1.0,
    equivalence_threshold: float = 0.5
) -> Dict[str, Any]:
    """
    Evaluates whether an accused product/process infringes a patent claim under the Doctrine of Equivalents
    by calculating the geodesic hyperbolic distance between their functional manifolds.
    """
    dist = hs_math.poincare_dist(claim_vector, accused_product_vector, c=c)
    is_equivalent = dist <= equivalence_threshold

    return {
        "is_equivalent_infringement": is_equivalent,
        "hyperbolic_distance": dist,
        "threshold": equivalence_threshold,
        "verdict": "SUBSTANTIAL_EQUIVALENCE" if is_equivalent else "LACKS_EQUIVALENCE"
    }


def synthesize_precedents(
    precedent_vectors: Sequence[Sequence[float]],
    c: float = 1.0,
    max_iter: int = 64
) -> List[float]:
    """
    Synthesizes the consensus legal rule (*ratio decidendi*) from a series of judicial opinions
    as the Fréchet mean on the hyperbolic jurisprudence manifold.
    """
    if not precedent_vectors:
        raise ValueError("Precedent vectors list cannot be empty")
    return hs_math.frechet_mean(precedent_vectors, c=c, max_iter=max_iter)


class LegalSDK:
    """
    High-level Legal Analytics & Patent Intelligence SDK for HyperspaceDB.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "legal_corpus", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def to_cone(self, norm: LegalNorm) -> List[float]:
        return norm_to_lorentz_cone(norm)

    def verify_validity(self, superior_cone: Sequence[float], subordinate_cone: Sequence[float]) -> bool:
        return is_norm_valid_under(superior_cone, subordinate_cone)

    def test_doctrine_of_equivalents(
        self,
        claim_vec: Sequence[float],
        product_vec: Sequence[float],
        threshold: float = 0.5
    ) -> Dict[str, Any]:
        return doctrine_of_equivalents(claim_vec, product_vec, c=self.c, equivalence_threshold=threshold)

    def consensus_ratio_decidendi(self, precedent_vectors: Sequence[Sequence[float]]) -> List[float]:
        return synthesize_precedents(precedent_vectors, c=self.c)

    def index_norm(self, norm: LegalNorm):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        vec = self.to_cone(norm)
        meta = dict(norm.metadata)
        meta["title"] = norm.title
        meta["rank"] = norm.rank
        meta["jurisdiction"] = norm.jurisdiction
        return self.client.insert(id=int(norm.norm_id), vector=vec, metadata=meta, collection=self.collection)
