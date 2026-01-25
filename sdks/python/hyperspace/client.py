import grpc
from typing import List, Dict, Optional
import sys
import os

# Fix import path for generated proto files
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from .proto import hyperspace_pb2
from .proto import hyperspace_pb2_grpc

class HyperspaceClient:
    def __init__(self, host: str = "localhost:50051"):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)

    def insert(self, id: int, vector: List[float], metadata: Dict[str, str] = None) -> bool:
        if metadata is None:
            metadata = {}
            
        req = hyperspace_pb2.InsertRequest(
            id=id,
            vector=vector,
            metadata=metadata
        )
        try:
            resp = self.stub.Insert(req)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def search(self, vector: List[float], top_k: int = 10, filter: Dict[str, str] = None) -> List[Dict]:
        if filter is None:
            filter = {}
            
        req = hyperspace_pb2.SearchRequest(
            vector=vector,
            top_k=top_k,
            filter=filter
        )
        try:
            resp = self.stub.Search(req)
            return [
                {"id": r.id, "distance": r.distance}
                for r in resp.results
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def close(self):
        self.channel.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
