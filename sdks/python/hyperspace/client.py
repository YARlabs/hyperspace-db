import grpc
from typing import List, Dict, Optional, Union
import sys
import os

# Fix import path for generated proto files
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from .proto import hyperspace_pb2
from .proto import hyperspace_pb2_grpc
from .embedders import BaseEmbedder

class HyperspaceClient:
    def __init__(self, host: str = "localhost:50051", api_key: Optional[str] = None, embedder: Optional[BaseEmbedder] = None):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)
        self.metadata = (('x-api-key', api_key),) if api_key else None
        self.embedder = embedder

    def insert(self, id: int, vector: List[float] = None, document: str = None, metadata: Dict[str, str] = None) -> bool:
        if metadata is None:
            metadata = {}
            
        if vector is None and document is not None:
            if self.embedder is None:
                raise ValueError("No embedder configured. Please pass 'vector' or init client with an embedder.")
            vector = self.embedder.encode(document)
        
        if vector is None:
             raise ValueError("Either 'vector' or 'document' must be provided.")

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

    def search(self, vector: List[float] = None, query_text: str = None, top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, hybrid_query: str = None, hybrid_alpha: float = None) -> List[Dict]:
        if filter is None:
            filter = {}
            
        if vector is None and query_text is not None:
            if self.embedder is None:
                raise ValueError("No embedder configured. Please pass 'vector' or init client with an embedder.")
            # For pure vector search using text query
            vector = self.embedder.encode(query_text)
            
            # Auto-enable hybrid if not specified but meaningful?
            # User might just want vector search by text. 
            # If they want hybrid, they pass hybrid_query explicitly or we could default it to query_text?
            # Keeping it simple: query_text just generates the vector.
            # If user wants hybrid, they should pass hybrid_query or we can be smart:
            if hybrid_query is None and hybrid_alpha is not None:
                 # If user specified alpha but no text, assume they want to use the query text for hybrid too
                 hybrid_query = query_text

        if vector is None:
             raise ValueError("Either 'vector' or 'query_text' must be provided.")

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

    def trigger_snapshot(self) -> bool:
        try:
            resp = self.stub.TriggerSnapshot(hyperspace_pb2.Empty(), metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def configure(self, ef_search: int = None, ef_construction: int = None) -> bool:
        req = hyperspace_pb2.ConfigUpdate()
        if ef_search is not None:
            req.ef_search = ef_search
        if ef_construction is not None:
            req.ef_construction = ef_construction
            
        try:
            resp = self.stub.Configure(req, metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def close(self):
        self.channel.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
