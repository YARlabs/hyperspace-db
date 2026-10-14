import grpc
import numpy as np
import sys
import os

# Ensure nested proto package is searchable
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace"))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace", "proto"))

from hyperspace.proto import hyperspace_pb2
from hyperspace.proto import hyperspace_pb2_grpc

from hyperspace.client import VerificationResult

class HyperspaceClient:
    def __init__(self, host="localhost:50051"):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)

    def verify_generation(self, llm_answer: str, context_ids: list, collection: str = "") -> VerificationResult:
        """
        Mathematically verifies if the LLM's answer semantically breaks the topological/geometric
        boundaries of the source context.

        The server vectorizes the drafted LLM answer on the fly and evaluates geodesic paths
        and manifold boundaries relative to the provided source context chunks in the index.
        Geometric distance calculations and boundary verifications are performed opaquely
        by the server engine.

        Args:
            llm_answer (str): The response text drafted by the LLM.
            context_ids (List[str]): List of source context chunk identifiers (string IDs or UUIDs)
                used by the LLM to generate the answer.
            collection (str, optional): Target collection name. If empty, the server's default collection is used.

        Returns:
            VerificationResult: A typed object containing `trust_score` (float 0.0-1.0) and
                `is_hallucination` (boolean) flag.
        """
        req = hyperspace_pb2.VerifyGenerationRequest(
            collection=collection,
            llm_answer=llm_answer,
            context_ids=context_ids,
            check_geodesic=True,
        )
        resp = self.stub.VerifyGeneration(req)
        return VerificationResult(
            trust_score=float(resp.trust_score),
            is_hallucination=not resp.is_valid,
            is_valid=resp.is_valid,
            min_distance=float(resp.min_distance),
            avg_distance=float(resp.avg_distance),
            geodesic_distance=float(resp.geodesic_distance),
            geodesic_hops=int(resp.geodesic_hops),
            cone_satisfied=resp.cone_satisfied,
            details=resp.details,
            resolved_context_ids=list(resp.resolved_context_ids),
        )

    def search(self, vector: np.ndarray, top_k: int = 10, filters=None):
        # Validate Poincare norm < 1
        if np.linalg.norm(vector) >= 1:
            raise ValueError("Vector must be inside Poincaré ball")
            
        req = hyperspace_pb2.SearchRequest(
            vector=vector.tolist(), 
            top_k=top_k,
            filters=filters or []
        )
        return self.stub.Search(req)

    @staticmethod
    def filter_ball(center: list, radius: float):
        return hyperspace_pb2.Filter(in_ball=hyperspace_pb2.InBall(center=center, radius=radius))

    @staticmethod
    def filter_box(min_bounds: list, max_bounds: list):
        return hyperspace_pb2.Filter(in_box=hyperspace_pb2.InBox(min_bounds=min_bounds, max_bounds=max_bounds))

    @staticmethod
    def filter_cone(axes: list, apertures: list, cen: float = 0.0):
        return hyperspace_pb2.Filter(in_cone=hyperspace_pb2.InCone(axes=axes, apertures=apertures, cen=cen))

