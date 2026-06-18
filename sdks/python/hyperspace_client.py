import grpc
import numpy as np
import sys
import os

# Ensure nested proto package is searchable
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace"))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace", "proto"))

from hyperspace.proto import hyperspace_pb2
from hyperspace.proto import hyperspace_pb2_grpc

class HyperspaceClient:
    def __init__(self, host="localhost:50051"):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)

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
