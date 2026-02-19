import grpc
from typing import List, Dict, Optional, Union
import sys
import os

sys.path.append(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "proto"))

from .proto import hyperspace_pb2
from .proto import hyperspace_pb2_grpc
from .embedders import BaseEmbedder

class Durability:
    DEFAULT = 0
    ASYNC = 1
    BATCH = 2
    STRICT = 3

class HyperspaceClient:
    def __init__(self, host: str = "localhost:50051", api_key: Optional[str] = None, embedder: Optional[BaseEmbedder] = None, user_id: Optional[str] = None):
        # Optimized gRPC Channel with KeepAlive and Max Message Size
        options = [
            ('grpc.max_send_message_length', 64 * 1024 * 1024), # 64MB
            ('grpc.max_receive_message_length', 64 * 1024 * 1024), # 64MB
            ('grpc.keepalive_time_ms', 10000),
            ('grpc.keepalive_timeout_ms', 5000),
            ('grpc.keepalive_permit_without_calls', 1),
            ('grpc.http2.max_pings_without_data', 0),
            ('grpc.http2.min_time_between_pings_ms', 10000),
            ('grpc.http2.min_ping_interval_without_data_ms', 5000),
        ]
        self.channel = grpc.insecure_channel(host, options=options)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)
        meta = []
        if api_key:
            meta.append(('x-api-key', api_key))
        if user_id:
            meta.append(('x-hyperspace-user-id', user_id))
        self.metadata = tuple(meta) if meta else None
        self.embedder = embedder

    @staticmethod
    def _normalize_vector(vector: Union[List[float], tuple]) -> List[float]:
        # Fast path: already Python list (protobuf will consume directly).
        if isinstance(vector, list):
            return vector
        # Common path for tuples/numpy arrays/iterables.
        # Keep explicit list conversion once per request.
        return list(vector)

    # ... (create/delete/list unchanged) ...

    def create_collection(self, name: str, dimension: int, metric: str) -> bool:
        req = hyperspace_pb2.CreateCollectionRequest(name=name, dimension=dimension, metric=metric)
        try:
            resp = self.stub.CreateCollection(req, metadata=self.metadata)
            return True
        except grpc.RpcError:
            return False

    def delete_collection(self, name: str) -> bool:
        req = hyperspace_pb2.DeleteCollectionRequest(name=name)
        try:
            resp = self.stub.DeleteCollection(req, metadata=self.metadata)
            return True
        except grpc.RpcError:
            return False

    def list_collections(self) -> List[str]:
        req = hyperspace_pb2.Empty()
        try:
            resp = self.stub.ListCollections(req, metadata=self.metadata)
            return resp.collections
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def get_collection_stats(self, name: str) -> Dict:
        req = hyperspace_pb2.CollectionStatsRequest(name=name)
        try:
            resp = self.stub.GetCollectionStats(req, metadata=self.metadata)
            return {
                "count": resp.count,
                "dimension": resp.dimension,
                "metric": resp.metric,
                "indexing_queue": resp.indexing_queue
            }
        except grpc.RpcError:
            return {}

    def insert(self, id: int, vector: List[float] = None, document: str = None, metadata: Dict[str, str] = None, collection: str = "", durability: int = Durability.DEFAULT) -> bool:
        if vector is None and document is not None:
            if self.embedder is None:
                raise ValueError("No embedder configured. Please pass 'vector' or init client with an embedder.")
            vector = self.embedder.encode(document)
        
        if vector is None:
             raise ValueError("Either 'vector' or 'document' must be provided.")
        vector = self._normalize_vector(vector)

        req = hyperspace_pb2.InsertRequest(
            id=id,
            vector=vector,
            collection=collection,
            origin_node_id="",
            logical_clock=0,
            durability=durability
        )
        if metadata:
            req.metadata.update(metadata)
        try:
            resp = self.stub.Insert(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def batch_insert(self, vectors: List[List[float]], ids: List[int], metadatas: List[Dict[str, str]] = None, collection: str = "", durability: int = Durability.DEFAULT) -> bool:
        if len(vectors) != len(ids):
             raise ValueError("Vectors and IDs length mismatch")
        
        proto_vectors = []
        if metadatas is None:
            for v, i in zip(vectors, ids):
                proto_vectors.append(hyperspace_pb2.VectorData(
                    vector=self._normalize_vector(v),
                    id=i
                ))
        else:
            for v, i, m in zip(vectors, ids, metadatas):
                if m:
                    proto_vectors.append(hyperspace_pb2.VectorData(
                        vector=self._normalize_vector(v),
                        id=i,
                        metadata=m
                    ))
                else:
                    proto_vectors.append(hyperspace_pb2.VectorData(
                        vector=self._normalize_vector(v),
                        id=i
                    ))

        req = hyperspace_pb2.BatchInsertRequest(
            collection=collection,
            vectors=proto_vectors,
            origin_node_id="",
            logical_clock=0,
            durability=durability
        )
        try:
            resp = self.stub.BatchInsert(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def search(self, vector: List[float] = None, query_text: str = None, top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, hybrid_query: str = None, hybrid_alpha: float = None, collection: str = "") -> List[Dict]:
        if vector is None and query_text is not None:
            if self.embedder is None:
                raise ValueError("No embedder configured. Please pass 'vector' or init client with an embedder.")
            # For pure vector search using text query
            vector = self.embedder.encode(query_text)
            
            # Auto-enable hybrid if not specified but meaningful?
            if hybrid_query is None and hybrid_alpha is not None:
                 hybrid_query = query_text
        
        if vector is None:
             raise ValueError("Either 'vector' or 'query_text' must be provided.")
        vector = self._normalize_vector(vector)

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
            collection=collection
        )
        if filter:
            req.filter.update(filter)
        if proto_filters:
            req.filters.extend(proto_filters)
        if hybrid_query is not None:
            req.hybrid_query = hybrid_query
        if hybrid_alpha is not None:
            req.hybrid_alpha = hybrid_alpha
        try:
            resp = self.stub.Search(req, metadata=self.metadata)
            return [
                {
                    "id": r.id,
                    "distance": r.distance,
                    "metadata": (dict(r.metadata) if r.metadata else {})
                }
                for r in resp.results
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def search_batch(
        self,
        vectors: List[List[float]],
        top_k: int = 10,
        collection: str = "",
    ) -> List[List[Dict]]:
        searches = []
        for vector in vectors:
            searches.append(
                hyperspace_pb2.SearchRequest(
                    vector=self._normalize_vector(vector),
                    top_k=top_k,
                    collection=collection,
                )
            )
        req = hyperspace_pb2.BatchSearchRequest(searches=searches)
        try:
            resp = self.stub.SearchBatch(req, metadata=self.metadata)
            batch = []
            for search_resp in resp.responses:
                batch.append(
                    [
                        {
                            "id": r.id,
                            "distance": r.distance,
                            "metadata": (dict(r.metadata) if r.metadata else {}),
                        }
                        for r in search_resp.results
                    ]
                )
            return batch
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def trigger_vacuum(self) -> bool:
        try:
            self.stub.TriggerVacuum(hyperspace_pb2.Empty(), metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def rebuild_index(self, collection: str) -> bool:
        req = hyperspace_pb2.RebuildIndexRequest(name=collection)
        try:
            self.stub.RebuildIndex(req, metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def trigger_snapshot(self) -> bool:
        try:
            resp = self.stub.TriggerSnapshot(hyperspace_pb2.Empty(), metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def configure(self, ef_search: int = None, ef_construction: int = None, collection: str = "") -> bool:
        req = hyperspace_pb2.ConfigUpdate(collection=collection)
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

    def get_digest(self, collection: str = "") -> Dict:
        req = hyperspace_pb2.DigestRequest(collection=collection)
        try:
            resp = self.stub.GetDigest(req, metadata=self.metadata)
            return {
                "logical_clock": resp.logical_clock,
                "state_hash": resp.state_hash,
                "count": resp.count
            }
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return {}

    def close(self):
        self.channel.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
