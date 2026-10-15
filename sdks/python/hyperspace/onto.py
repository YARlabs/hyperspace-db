"""
HyperspaceDB ontoSDK / kgSDK: Knowledge Graph Completion & Hyperbolic Ontologies Engine.

Provides Hyperbolic TransE embeddings (Möbius relation translations), triple plausibility scoring,
taxonomic IS-A hierarchies via Lorentz light cones, and cross-ontology alignment via phase-locked
context resonance.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
from . import math as hs_math


@dataclass
class Triple:
    head: str
    relation: str
    tail: str
    head_vec: Sequence[float]
    relation_vec: Sequence[float]
    tail_vec: Sequence[float]
    metadata: Dict[str, Any] = field(default_factory=dict)


def hyperbolic_translate(
    head_vec: Sequence[float],
    relation_vec: Sequence[float],
    c: float = 1.0
) -> List[float]:
    """
    Computes the predicted tail entity embedding in hyperbolic space using Möbius addition:
    tail_hat = head (+) relation.
    """
    return hs_math.mobius_add(head_vec, relation_vec, c=c)


def score_triple(
    head_vec: Sequence[float],
    relation_vec: Sequence[float],
    tail_vec: Sequence[float],
    c: float = 1.0
) -> float:
    """
    Computes the plausibility energy score for a knowledge graph triple (h, r, t).
    Energy is the geodesic hyperbolic distance d_c(h (+) r, t).
    Lower energy indicates higher confidence/truth.
    """
    predicted_tail = hyperbolic_translate(head_vec, relation_vec, c=c)
    return hs_math.poincare_dist(predicted_tail, tail_vec, c=c)


def concept_subsumption_cone(
    taxonomy_depth: int,
    concept_vector: Sequence[float],
    depth_scale: float = 2.0
) -> List[float]:
    """
    Embeds an ontological concept into Lorentz spacetime to model taxonomic IS-A relationships.
    Depth 1: Root concept (e.g., "Living Organism")
    Depth 2: Sub-category (e.g., "Mammal")
    Depth 3: Terminal concept (e.g., "Homo Sapiens")
    """
    if taxonomy_depth < 1:
        raise ValueError("Taxonomy depth must be >= 1")
    t = float(taxonomy_depth) * depth_scale
    spatial = list(concept_vector)
    return [t] + spatial


def is_concept_subsumed(
    parent_concept_cone: Sequence[float],
    child_concept_cone: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Verifies whether child_concept IS-A parent_concept in the taxonomic hierarchy."""
    return hs_math.is_minkowski_subsumed(parent_concept_cone, child_concept_cone, epsilon=epsilon)


def align_ontologies(
    concept_source: Sequence[float],
    concept_target: Sequence[float],
    resonance_factor: float = 0.5,
    c: float = 1.0
) -> List[float]:
    """
    Aligns heterogeneous ontology entities across different knowledge graphs using
    phase-locked hyperbolic context resonance.
    """
    return hs_math.context_resonance(
        concept_source,
        concept_target,
        resonance_factor=resonance_factor,
        c=c
    )


class OntoSDK:
    """
    High-level Knowledge Graph & Ontology SDK for HyperspaceDB.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "knowledge_graph", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def predict_tail(self, head: Sequence[float], relation: Sequence[float]) -> List[float]:
        return hyperbolic_translate(head, relation, c=self.c)

    def evaluate_triple(self, head: Sequence[float], relation: Sequence[float], tail: Sequence[float]) -> float:
        return score_triple(head, relation, tail, c=self.c)

    def to_isa_cone(self, depth: int, vector: Sequence[float]) -> List[float]:
        return concept_subsumption_cone(depth, vector)

    def verify_isa(self, parent_cone: Sequence[float], child_cone: Sequence[float]) -> bool:
        return is_concept_subsumed(parent_cone, child_cone)

    def align(self, source_vec: Sequence[float], target_vec: Sequence[float], factor: float = 0.5) -> List[float]:
        return align_ontologies(source_vec, target_vec, resonance_factor=factor, c=self.c)

    def index_triple(self, triple: Triple, id_start: int = 1):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        h_id = id_start
        t_id = id_start + 1
        res1 = self.client.insert(id=h_id, vector=list(triple.head_vec), metadata={"entity": triple.head}, collection=self.collection)
        res2 = self.client.insert(id=t_id, vector=list(triple.tail_vec), metadata={"entity": triple.tail}, collection=self.collection)
        return res1, res2
