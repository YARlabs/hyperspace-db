import grpc
from typing import List, Dict, Optional, Union, Iterator
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

    @staticmethod
    def _to_proto_metadata_value(value):
        mv = hyperspace_pb2.MetadataValue()
        if isinstance(value, bool):
            mv.bool_value = value
        elif isinstance(value, int) and not isinstance(value, bool):
            mv.int_value = value
        elif isinstance(value, float):
            mv.double_value = value
        else:
            mv.string_value = str(value)
        return mv

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

    def insert(self, id: int, vector: List[float] = None, document: str = None, metadata: Dict[str, str] = None, typed_metadata: Dict[str, object] = None, collection: str = "", durability: int = Durability.DEFAULT) -> bool:
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
        if typed_metadata:
            for k, v in typed_metadata.items():
                req.typed_metadata[k].CopyFrom(self._to_proto_metadata_value(v))
        try:
            resp = self.stub.Insert(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def insert_text(self, id: int, text: str, metadata: Dict[str, str] = None, collection: str = "", durability: int = Durability.DEFAULT) -> bool:
        req = hyperspace_pb2.InsertTextRequest(
            id=id,
            text=text,
            collection=collection,
            durability=durability
        )
        if metadata:
            req.metadata.update(metadata)
        try:
            resp = self.stub.InsertText(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def delete(self, id: int, collection: str = "") -> bool:
        req = hyperspace_pb2.DeleteRequest(
            id=id,
            collection=collection
        )
        try:
            resp = self.stub.Delete(req, metadata=self.metadata)
            return resp.success
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def vectorize(self, text: str, metric: str = "l2") -> List[float]:
        req = hyperspace_pb2.VectorizeRequest(text=text, metric=metric)
        try:
            resp = self.stub.Vectorize(req, metadata=self.metadata)
            return list(resp.vector)
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def batch_insert(self, vectors: List[List[float]], ids: List[int], metadatas: List[Dict[str, str]] = None, typed_metadatas: List[Dict[str, object]] = None, collection: str = "", durability: int = Durability.DEFAULT) -> bool:
        if len(vectors) != len(ids):
             raise ValueError("Vectors and IDs length mismatch")
        
        proto_vectors = []
        if metadatas is None and typed_metadatas is None:
            for v, i in zip(vectors, ids):
                proto_vectors.append(hyperspace_pb2.VectorData(
                    vector=self._normalize_vector(v),
                    id=i
                ))
        else:
            if metadatas is None:
                metadatas = [{} for _ in vectors]
            if typed_metadatas is None:
                typed_metadatas = [{} for _ in vectors]
            for v, i, m, tm in zip(vectors, ids, metadatas, typed_metadatas):
                if m:
                    vd = hyperspace_pb2.VectorData(
                        vector=self._normalize_vector(v),
                        id=i,
                        metadata=m
                    )
                else:
                    vd = hyperspace_pb2.VectorData(
                        vector=self._normalize_vector(v),
                        id=i
                    )
                if tm:
                    for k, val in tm.items():
                        vd.typed_metadata[k].CopyFrom(self._to_proto_metadata_value(val))
                proto_vectors.append(vd)

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
                    if "gte" in f:
                        gte_val = f["gte"]
                        if isinstance(gte_val, int):
                            kwargs["gte"] = int(gte_val)
                        else:
                            kwargs["gte_f64"] = float(gte_val)
                    if "lte" in f:
                        lte_val = f["lte"]
                        if isinstance(lte_val, int):
                            kwargs["lte"] = int(lte_val)
                        else:
                            kwargs["lte_f64"] = float(lte_val)
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
                    "metadata": (dict(r.metadata) if r.metadata else {}),
                    "typed_metadata": dict(r.typed_metadata) if r.typed_metadata else {}
                }
                for r in resp.results
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def search_text(self, text: str, top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, collection: str = "") -> List[Dict]:
        proto_filters = []
        if filters:
            for f in filters:
                if f.get("type") == "match":
                    proto_filters.append(hyperspace_pb2.Filter(
                        match=hyperspace_pb2.Match(key=f["key"], value=f["value"])
                    ))
                elif f.get("type") == "range":
                    kwargs = {"key": f["key"]}
                    if "gte" in f:
                        gte_val = f["gte"]
                        if isinstance(gte_val, int):
                            kwargs["gte"] = int(gte_val)
                        else:
                            kwargs["gte_f64"] = float(gte_val)
                    if "lte" in f:
                        lte_val = f["lte"]
                        if isinstance(lte_val, int):
                            kwargs["lte"] = int(lte_val)
                        else:
                            kwargs["lte_f64"] = float(lte_val)
                    proto_filters.append(hyperspace_pb2.Filter(
                        range=hyperspace_pb2.Range(**kwargs)
                    ))

        req = hyperspace_pb2.SearchTextRequest(
            text=text,
            top_k=top_k,
            collection=collection
        )
        if filter:
            req.filter.update(filter)
        if proto_filters:
            req.filters.extend(proto_filters)
            
        try:
            resp = self.stub.SearchText(req, metadata=self.metadata)
            return [
                {
                    "id": r.id,
                    "distance": r.distance,
                    "metadata": (dict(r.metadata) if r.metadata else {}),
                    "typed_metadata": dict(r.typed_metadata) if r.typed_metadata else {}
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
                            "typed_metadata": dict(r.typed_metadata) if r.typed_metadata else {},
                        }
                        for r in search_resp.results
                    ]
                )
            return batch
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def subscribe_to_events(self, types: Optional[List[str]] = None, collection: Optional[str] = None) -> Iterator[Dict]:
        req = hyperspace_pb2.EventSubscriptionRequest()
        if collection:
            req.collection = collection
        if types:
            for t in types:
                if t == "insert":
                    req.types.append(hyperspace_pb2.VECTOR_INSERTED)
                elif t == "delete":
                    req.types.append(hyperspace_pb2.VECTOR_DELETED)
        try:
            stream = self.stub.SubscribeToEvents(req, metadata=self.metadata)
            for ev in stream:
                payload = {}
                if ev.HasField("vector_inserted"):
                    payload = {
                        "id": ev.vector_inserted.id,
                        "collection": ev.vector_inserted.collection,
                        "logical_clock": ev.vector_inserted.logical_clock,
                        "origin_node_id": ev.vector_inserted.origin_node_id,
                        "metadata": dict(ev.vector_inserted.metadata),
                        "typed_metadata": dict(ev.vector_inserted.typed_metadata),
                    }
                elif ev.HasField("vector_deleted"):
                    payload = {
                        "id": ev.vector_deleted.id,
                        "collection": ev.vector_deleted.collection,
                        "logical_clock": ev.vector_deleted.logical_clock,
                        "origin_node_id": ev.vector_deleted.origin_node_id,
                    }
                yield {
                    "type": ev.type,
                    "payload": payload,
                }
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return

    def trigger_vacuum(self) -> bool:
        try:
            self.stub.TriggerVacuum(hyperspace_pb2.Empty(), metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return False

    def rebuild_index(self, collection: str, filter_query: Dict[str, object] = None) -> bool:
        req = hyperspace_pb2.RebuildIndexRequest(name=collection)
        if filter_query:
            fq = hyperspace_pb2.VacuumFilterQuery(
                key=str(filter_query.get("key", "")),
                op=str(filter_query.get("op", "")),
                value=float(filter_query.get("value", 0.0)),
            )
            req.filter_query.CopyFrom(fq)
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

    def get_node(self, id: int, layer: int = 0, collection: str = "") -> Dict:
        req = hyperspace_pb2.GetNodeRequest(collection=collection, id=id, layer=layer)
        try:
            resp = self.stub.GetNode(req, metadata=self.metadata)
            return {
                "id": resp.id,
                "layer": resp.layer,
                "neighbors": list(resp.neighbors),
                "metadata": dict(resp.metadata),
                "typed_metadata": dict(resp.typed_metadata),
            }
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return {}

    def get_neighbors(self, id: int, layer: int = 0, limit: int = 64, offset: int = 0, collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.GetNeighborsRequest(
            collection=collection, id=id, layer=layer, limit=limit, offset=offset
        )
        try:
            resp = self.stub.GetNeighbors(req, metadata=self.metadata)
            return [
                {
                    "id": n.id,
                    "layer": n.layer,
                    "neighbors": list(n.neighbors),
                    "metadata": dict(n.metadata),
                    "typed_metadata": dict(n.typed_metadata),
                }
                for n in resp.neighbors
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def get_concept_parents(self, id: int, layer: int = 0, limit: int = 32, collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.GetConceptParentsRequest(
            collection=collection, id=id, layer=layer, limit=limit
        )
        try:
            resp = self.stub.GetConceptParents(req, metadata=self.metadata)
            return [
                {
                    "id": n.id,
                    "layer": n.layer,
                    "neighbors": list(n.neighbors),
                    "metadata": dict(n.metadata),
                    "typed_metadata": dict(n.typed_metadata),
                }
                for n in resp.parents
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def traverse(self, start_id: int, max_depth: int = 2, max_nodes: int = 256, layer: int = 0, filter: Dict[str, str] = None, filters: List[Dict] = None, collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.TraverseRequest(
            collection=collection,
            start_id=start_id,
            max_depth=max_depth,
            max_nodes=max_nodes,
            layer=layer,
        )
        if filter:
            req.filter.update(filter)
        if filters:
            for f in filters:
                if f.get("type") == "match":
                    req.filters.append(
                        hyperspace_pb2.Filter(
                            match=hyperspace_pb2.Match(key=f["key"], value=f["value"])
                        )
                    )
                elif f.get("type") == "range":
                    kwargs = {"key": f["key"]}
                    if "gte" in f:
                        gte_val = f["gte"]
                        if isinstance(gte_val, int):
                            kwargs["gte"] = int(gte_val)
                        else:
                            kwargs["gte_f64"] = float(gte_val)
                    if "lte" in f:
                        lte_val = f["lte"]
                        if isinstance(lte_val, int):
                            kwargs["lte"] = int(lte_val)
                        else:
                            kwargs["lte_f64"] = float(lte_val)
                    req.filters.append(
                        hyperspace_pb2.Filter(range=hyperspace_pb2.Range(**kwargs))
                    )
        try:
            resp = self.stub.Traverse(req, metadata=self.metadata)
            return [
                {
                    "id": n.id,
                    "layer": n.layer,
                    "neighbors": list(n.neighbors),
                    "metadata": dict(n.metadata),
                    "typed_metadata": dict(n.typed_metadata),
                }
                for n in resp.nodes
            ]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def find_semantic_clusters(self, layer: int = 0, min_cluster_size: int = 3, max_clusters: int = 32, max_nodes: int = 10000, collection: str = "") -> List[List[int]]:
        req = hyperspace_pb2.FindSemanticClustersRequest(
            collection=collection,
            layer=layer,
            min_cluster_size=min_cluster_size,
            max_clusters=max_clusters,
            max_nodes=max_nodes,
        )
        try:
            resp = self.stub.FindSemanticClusters(req, metadata=self.metadata)
            return [list(cluster.node_ids) for cluster in resp.clusters]
        except grpc.RpcError as e:
            print(f"RPC Error: {e}")
            return []

    def sync_handshake(self, collection: str, client_buckets: list, client_logical_clock: int = 0, client_count: int = 0) -> dict:
        """
        Initiate synchronization by sending local bucket hashes to the server.
        """
        if len(client_buckets) != 256:
            raise ValueError("client_buckets must contain exactly 256 elements")
            
        req = hyperspace_pb2.SyncHandshakeRequest(
            collection=collection,
            client_buckets=client_buckets,
            client_logical_clock=client_logical_clock,
            client_count=client_count
        )
        try:
            resp = self.stub.SyncHandshake(req, metadata=self.metadata)
            diff_buckets = [{"bucket_index": b.bucket_index, "server_hash": b.server_hash, "client_hash": b.client_hash} for b in resp.diff_buckets]
            return {
                "diff_buckets": diff_buckets,
                "server_logical_clock": resp.server_logical_clock,
                "server_count": resp.server_count,
                "in_sync": resp.in_sync
            }
        except grpc.RpcError as e:
            print(f"RPC Error in sync_handshake: {e}")
            return {}

    def sync_pull(self, collection: str, bucket_indices: list):
        """
        Pull specific buckets from the remote server.
        Returns a generator yielding SyncVectorData dicts.
        """
        req = hyperspace_pb2.SyncPullRequest(
            collection=collection,
            bucket_indices=bucket_indices
        )
        try:
            for item in self.stub.SyncPull(req, metadata=self.metadata):
                yield {
                    "collection": item.collection,
                    "id": item.id,
                    "vector": list(item.vector),
                    "metadata": dict(item.metadata),
                    "bucket_index": item.bucket_index
                }
        except grpc.RpcError as e:
            print(f"RPC Error in sync_pull: {e}")

    def search_multi_collection_text(self, text: str, collections: List[str], top_k: int = 10) -> Dict[str, List[Dict]]:
        vector = self.vectorize(text)
        return self.search_multi_collection(vector, collections, top_k)

    def close(self):
        self.channel.close()

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

def analyze_delta_hyperbolicity(vectors: List[List[float]], num_samples: int = 1000) -> (float, str):
    """
    Computes Gromov's delta-hyperbolicity of a dataset.
    A metric space is delta-hyperbolic if for any 4 points x,y,u,v:
    d(x,y) + d(u,v) <= max(d(x,u)+d(y,v), d(x,v)+d(y,u)) + 2*delta
    """
    import random
    import math

    if len(vectors) < 4:
        return 0.0, "euclidean"

    def l2_dist(a, b):
        return math.sqrt(sum((ax - bx) ** 2 for ax, bx in zip(a, b)))

    max_delta = 0.0
    for _ in range(num_samples):
        i, j, k, l = random.sample(range(len(vectors)), 4)
        
        d_ij = l2_dist(vectors[i], vectors[j])
        d_kl = l2_dist(vectors[k], vectors[l])
        
        d_ik = l2_dist(vectors[i], vectors[k])
        d_jl = l2_dist(vectors[j], vectors[l])
        
        d_il = l2_dist(vectors[i], vectors[l])
        d_jk = l2_dist(vectors[j], vectors[k])
        
        s1 = d_ij + d_kl
        s2 = d_ik + d_jl
        s3 = d_il + d_jk
        
        sums = sorted([s1, s2, s3], reverse=True)
        delta = (sums[0] - sums[1]) / 2.0
        if delta > max_delta:
            max_delta = delta

    # Recommendation heuristic
    if max_delta < 0.15:
        return max_delta, "lorentz"
    elif max_delta < 0.30:
        return max_delta, "poincare"
    else:
        return max_delta, "l2"
