import grpc
from typing import List, Dict, Optional
import sys
import os

# Fix import path for generated proto files
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from .proto import hyperspace_pb2
from .proto import hyperspace_pb2_grpc

class HyperspaceClient:
    def __init__(self, host: str = "localhost:50051", api_key: Optional[str] = None):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)
        self.metadata = (('x-api-key', api_key),) if api_key else None

    def insert(self, id: int, vector: List[float], metadata: Dict[str, str] = None) -> bool:
        if metadata is None:
            metadata = {}
            
        req = hyperspace_pb2.InsertRequest(
            id=id,
            vector=vector,
            metadata=metadata
        )
        try:
            resp = self.stub.Insert(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def search(self, vector: List[float], top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, hybrid_query: str = None, hybrid_alpha: float = None) -> List[Dict]:
        if filter is None:
            filter = {}
            
        proto_filters = []
        if filters:
            for f in filters:
                if f.get("type") == "match":
                    proto_filters.append(hyperspace_pb2.Filter(
                        match=hyperspace_pb2.Match(key=f["key"], value=f["value"])
                    ))
                elif f.get("type") == "range":
                    kwargs = {"key": f["key"]}
                    if "gte" in f: kwargs["gte"] = int(f["gte"])
                    if "lte" in f: kwargs["lte"] = int(f["lte"])
                    proto_filters.append(hyperspace_pb2.Filter(
                        range=hyperspace_pb2.Range(**kwargs)
                    ))

        req = hyperspace_pb2.SearchRequest(
            vector=vector,
            top_k=top_k,
            filter=filter,
            filters=proto_filters,
            hybrid_query=hybrid_query,
            hybrid_alpha=hybrid_alpha
        )
        try:
            resp = self.stub.Search(req, metadata=self.metadata)
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
