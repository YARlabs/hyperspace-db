import grpc
import hyperspace_pb2
import hyperspace_pb2_grpc
import numpy as np

class HyperspaceClient:
    def __init__(self, host="localhost:50051"):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)

    def search(self, vector: np.ndarray, top_k: int = 10):
        # Validate Poincare norm < 1
        if np.linalg.norm(vector) >= 1:
            raise ValueError("Vector must be inside Poincaré ball")
            
        req = hyperspace_pb2.SearchRequest(
            vector=vector.tolist(), 
            top_k=top_k
        )
        return self.stub.Search(req)
