"""HyperspaceDB Python gRPC client.

This module provides a simple Python client for HyperspaceDB using gRPC.
It is used by the LangChain integration.
"""

from typing import Dict, List, Optional, Tuple, Any
import grpc
try:
    from .generated import hyperspace_pb2, hyperspace_pb2_grpc
except ImportError:
    # Fallback for direct scripts
    from langchain_hyperspace.generated import hyperspace_pb2, hyperspace_pb2_grpc


class HyperspaceClient:
    """Simple Python client for HyperspaceDB.
    
    This is a lightweight wrapper around the gRPC API.
    """
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 50051,
        api_key: Optional[str] = None,
        user_id: Optional[str] = None,
    ):
        target = f"{host}:{port}" if ":" not in host else host
        self._channel = grpc.insecure_channel(target)
        self._stub = hyperspace_pb2_grpc.DatabaseStub(self._channel)
        self.api_key = api_key
        self.user_id = user_id
        
    def _get_metadata(self) -> List[Tuple[str, str]]:
        meta = []
        if self.api_key:
            meta.append(("x-api-key", self.api_key))
        if self.user_id:
            meta.append(("x-hyperspace-user-id", self.user_id))
        return meta
    
    def create_collection(self, name: str, dimension: int, metric: str = "l2") -> bool:
        req = hyperspace_pb2.CreateCollectionRequest(name=name, dimension=dimension, metric=metric)
        self._stub.CreateCollection(req, metadata=self._get_metadata())
        return True
    
    def insert(self, collection: str, id: int, vector: List[float], metadata: Dict[str, str]) -> bool:
        req = hyperspace_pb2.InsertRequest(collection=collection, id=id, vector=vector, metadata=metadata)
        res = self._stub.Insert(req, metadata=self._get_metadata())
        return res.success

    def insert_text(self, id: int, text: str, collection: str, metadata: Dict[str, str] = None) -> bool:
        req = hyperspace_pb2.InsertTextRequest(collection=collection, id=id, text=text, metadata=metadata or {})
        res = self._stub.InsertText(req, metadata=self._get_metadata())
        return res.success

    def batch_insert(self, vectors: List[List[float]], ids: List[int], metadatas: List[Dict[str, str]], collection: str) -> bool:
        items = []
        for i in range(len(vectors)):
            vdata = hyperspace_pb2.VectorData(id=ids[i], vector=vectors[i], metadata=metadatas[i])
            items.append(vdata)
        req = hyperspace_pb2.BatchInsertRequest(collection=collection, vectors=items)
        res = self._stub.BatchInsert(req, metadata=self._get_metadata())
        return res.success

    def search(self, vector: List[float], top_k: int, collection: str, filters: List[Any] = None) -> List[Any]:
        req = hyperspace_pb2.SearchRequest(collection=collection, vector=vector, top_k=top_k)
        if filters:
             # Logic to add filters to the req
             pass
        res = self._stub.Search(req, metadata=self._get_metadata())
        return res.results

    def search_text(self, text: str, top_k: int, collection: str) -> List[Any]:
        req = hyperspace_pb2.SearchTextRequest(collection=collection, text=text, top_k=top_k)
        res = self._stub.SearchText(req, metadata=self._get_metadata())
        return res.results

    def get_digest(self, collection: str) -> Dict[str, Any]:
        req = hyperspace_pb2.DigestRequest(collection=collection)
        res = self._stub.GetDigest(req, metadata=self._get_metadata())
        return {"count": res.count, "logical_clock": res.logical_clock}

    def delete(self, collection: str, id: int) -> bool:
        req = hyperspace_pb2.DeleteRequest(collection=collection, id=id)
        res = self._stub.Delete(req, metadata=self._get_metadata())
        return res.success

    def close(self):
        self._channel.close()
