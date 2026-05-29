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
    def __init__(self, host: str = "localhost:50051", api_key: Optional[str] = None, embedder: Optional[BaseEmbedder] = None, user_id: Optional[str] = None, pool_size: int = 8):
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
        self.host = host
        self.api_key = api_key
        self.user_id = user_id
        
        import threading
        self.num_channels = pool_size
        self._lock = threading.Lock()
        self._thread_local = threading.local()
        self.channels = [grpc.insecure_channel(host, options=options) for _ in range(self.num_channels)]
        self.stubs = [hyperspace_pb2_grpc.DatabaseStub(c) for c in self.channels]
        self._stub_index = 0
        
        # Backwards compatibility channel reference
        self.channel = self.channels[0]
        
        meta = []
        if api_key:
            meta.append(('x-api-key', api_key))
        if user_id:
            meta.append(('x-hyperspace-user-id', user_id))
        self.metadata = tuple(meta) if meta else None
        self.embedder = embedder

    @property
    def stub(self):
        # Lock-free thread-local fast path
        if hasattr(self._thread_local, "stub"):
            return self._thread_local.stub
            
        with self._lock:
            self._stub_index = (self._stub_index + 1) % self.num_channels
            selected_stub = self.stubs[self._stub_index]
            self._thread_local.stub = selected_stub
            return selected_stub

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

    def _to_proto_filter(self, f: Dict) -> hyperspace_pb2.Filter:
        pf = hyperspace_pb2.Filter()
        # Support both 'type' field and direct key check
        f_type = f.get("type")
        
        if f_type == "match" or "match" in f:
            match_data = f.get("match", f)
            pf.match.key = match_data["key"]
            pf.match.value = str(match_data["value"])
        elif f_type == "prefix" or "prefix" in f:
            prefix_data = f.get("prefix", f)
            pf.prefix.key = prefix_data["key"]
            pf.prefix.prefix = str(prefix_data["prefix"])
        elif f_type == "range" or "range" in f:
            range_data = f.get("range", f)
            pf.range.key = range_data["key"]
            if "gte" in range_data:
                val = range_data["gte"]
                if isinstance(val, int): pf.range.gte = val
                else: pf.range.gte_f64 = float(val)
            if "lte" in range_data:
                val = range_data["lte"]
                if isinstance(val, int): pf.range.lte = val
                else: pf.range.lte_f64 = float(val)
        elif f_type == "in_cone" or "in_cone" in f:
            cone_data = f.get("in_cone", f)
            pf.in_cone.axes.extend(cone_data.get("axes", []))
            pf.in_cone.apertures.extend(cone_data.get("apertures", []))
            pf.in_cone.cen = cone_data.get("cen", [0.0])[0] if isinstance(cone_data.get("cen"), list) else float(cone_data.get("cen", 0.0))
        elif f_type == "in_ball" or "in_ball" in f:
            ball_data = f.get("in_ball", f)
            pf.in_ball.center.extend(ball_data.get("center", []))
            pf.in_ball.radius = float(ball_data.get("radius", 0.0))
        elif f_type == "in_box" or "in_box" in f:
            box_data = f.get("in_box", f)
            pf.in_box.min_bounds.extend(box_data.get("min_bounds", []))
            pf.in_box.max_bounds.extend(box_data.get("max_bounds", []))
        elif f_type == "and" or "and" in f:
            and_data = f.get("and", []) if "and" in f else f.get("conditions", [])
            pf.and_op.conditions.extend([self._to_proto_filter(cond) for cond in and_data])
        elif f_type == "or" or "or" in f:
            or_data = f.get("or", []) if "or" in f else f.get("conditions", [])
            pf.or_op.conditions.extend([self._to_proto_filter(cond) for cond in or_data])
        elif f_type == "not" or "not" in f:
            not_data = f.get("not") if "not" in f else f.get("condition")
            pf.not_op.condition.CopyFrom(self._to_proto_filter(not_data))
        return pf

    # ... (create/delete/list unchanged) ...

    def create_collection(self, name: str, schema: Union[Dict, hyperspace_pb2.CollectionSchema] = None, dimension: int = None, metric: str = None) -> bool:
        if schema is None:
            if dimension is not None and metric is not None:
                schema = {
                    "components": [
                        {
                            "name": "default",
                            "metric": metric,
                            "full_dimension": dimension,
                            "weight": 1.0
                        }
                    ]
                }
            else:
                raise ValueError("Either schema or both dimension and metric must be provided")

        if isinstance(schema, dict):
            proto_schema = hyperspace_pb2.CollectionSchema()
            if "components" in schema:
                for c in schema["components"]:
                    comp = hyperspace_pb2.VectorComponent(
                        name=c["name"],
                        metric=c["metric"],
                        full_dimension=c["full_dimension"],
                        weight=c.get("weight", 1.0)
                    )
                    proto_schema.components.append(comp)
            if "cascade_pipeline" in schema:
                for p in schema["cascade_pipeline"]:
                    layer = hyperspace_pb2.MrlLayer(
                        component_name=p["component_name"],
                        cutoff_dimension=p["cutoff_dimension"],
                        store_in_ram=p.get("store_in_ram", True),
                        rerank_top_k=p.get("rerank_top_k", 100)
                    )
                    proto_schema.cascade_pipeline.append(layer)
            schema = proto_schema

        req = hyperspace_pb2.CreateCollectionRequest(name=name, schema=schema)
        try:
            self.stub.CreateCollection(req, metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error in create_collection: {e}")
            return False

    def delete_collection(self, name: str) -> bool:
        req = hyperspace_pb2.DeleteCollectionRequest(name=name)
        try:
            resp = self.stub.DeleteCollection(req, metadata=self.metadata)
            return True
        except grpc.RpcError:
            return False

    def freeze_collection(self, name: str) -> str:
        req = hyperspace_pb2.FreezeCollectionRequest(name=name)
        try:
            resp = self.stub.FreezeCollection(req, metadata=self.metadata)
            return resp.status
        except grpc.RpcError as e:
            print(f"RPC Error in freeze_collection: {e}")
            return f"Error: {e}"

    def unfreeze_collection(self, name: str) -> str:
        req = hyperspace_pb2.UnfreezeCollectionRequest(name=name)
        try:
            resp = self.stub.UnfreezeCollection(req, metadata=self.metadata)
            return resp.status
        except grpc.RpcError as e:
            print(f"RPC Error in unfreeze_collection: {e}")
            return f"Error: {e}"

    def list_collections(self) -> List[Dict]:
        req = hyperspace_pb2.Empty()
        try:
            resp = self.stub.ListCollections(req, metadata=self.metadata)
            return [
                {
                    "name": c.name,
                    "count": c.count,
                    "schema": self._schema_to_dict(c.schema) if c.HasField("schema") else None
                }
                for c in resp.collections
            ]
        except grpc.RpcError as e:
            print(f"RPC Error in list_collections: {e}")
            return []

    def _schema_to_dict(self, schema: hyperspace_pb2.CollectionSchema) -> Dict:
        return {
            "components": [
                {
                    "name": c.name,
                    "metric": c.metric,
                    "full_dimension": c.full_dimension,
                    "weight": c.weight
                }
                for c in schema.components
            ],
            "cascade_pipeline": [
                {
                    "component_name": p.component_name,
                    "cutoff_dimension": p.cutoff_dimension,
                    "store_in_ram": p.store_in_ram,
                    "rerank_top_k": p.rerank_top_k
                }
                for p in schema.cascade_pipeline
            ]
        }


    def get_collection_stats(self, name: str) -> Dict:
        req = hyperspace_pb2.CollectionStatsRequest(name=name)
        try:
            resp = self.stub.GetCollectionStats(req, metadata=self.metadata)
            return {
                "count": resp.count,
                "indexing_queue": resp.indexing_queue,
                "disk_usage_bytes": resp.disk_usage_bytes,
                "ram_usage_bytes": resp.ram_usage_bytes,
                "active_tasks": resp.active_tasks,
                "schema": self._schema_to_dict(resp.schema) if resp.HasField("schema") else None
            }
        except grpc.RpcError as e:
            print(f"RPC Error in get_collection_stats: {e}")
            return {}

    def configure(self, collection: str, ef_search: Optional[int] = None, ef_construction: Optional[int] = None, m: Optional[int] = None) -> bool:
        req = hyperspace_pb2.ConfigUpdate(collection=collection)
        if ef_search is not None: req.ef_search = ef_search
        if ef_construction is not None: req.ef_construction = ef_construction
        if m is not None: req.m = m
        try:
            resp = self.stub.Configure(req, metadata=self.metadata)
            return "success" in resp.status.lower() or "updated" in resp.status.lower()
        except grpc.RpcError:
            return False

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

    def search(self, vector: List[float] = None, query_text: str = None, top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, hybrid_query: str = None, hybrid_alpha: float = None, bm25: Dict = None, mrl_dimension: int = None, use_wasserstein: bool = None, collection: str = "", options: Dict = None) -> List[Dict]:
        if vector is None and query_text is not None:
            if self.embedder is None:
                raise ValueError("No embedder configured. Please pass 'vector' or init client with an embedder.")
            # For pure vector search using text query
            vector = self.embedder.encode(query_text)
            
        if hybrid_query is None and hybrid_alpha is not None and query_text is not None:
             hybrid_query = query_text
        
        if vector is None:
             raise ValueError("Either 'vector' or 'query_text' must be provided.")
        vector = self._normalize_vector(vector)

        proto_filters = [self._to_proto_filter(f) for f in filters] if filters else []

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
        if mrl_dimension is not None:
            req.mrl_dimension = mrl_dimension
        if use_wasserstein is not None:
            req.use_wasserstein = use_wasserstein
        if options and "component_weights" in options:
             for k, v in options["component_weights"].items():
                 req.component_weights[k] = float(v)
            
        if bm25 is not None:
            bm25_msg = hyperspace_pb2.Bm25Options()
            if "method" in bm25: bm25_msg.method = bm25["method"]
            if "k1" in bm25: bm25_msg.k1 = bm25["k1"]
            if "b" in bm25: bm25_msg.b = bm25["b"]
            if "delta" in bm25: bm25_msg.delta = bm25["delta"]
            if "language" in bm25: bm25_msg.language = bm25["language"]
            if "ngrams" in bm25: bm25_msg.ngrams = bm25["ngrams"]
            if "fusion_method" in bm25: bm25_msg.fusion_method = bm25["fusion_method"]
            req.bm25_options.CopyFrom(bm25_msg)
            
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

    def search_text(self, text: str, top_k: int = 10, filter: Dict[str, str] = None, filters: List[Dict] = None, hybrid_alpha: float = None, bm25: Dict = None, collection: str = "") -> List[Dict]:
        proto_filters = [self._to_proto_filter(f) for f in filters] if filters else []

        req = hyperspace_pb2.SearchTextRequest(
            text=text,
            top_k=top_k,
            collection=collection
        )
        if filter:
            req.filter.update(filter)
        if proto_filters:
            req.filters.extend(proto_filters)
            
        if hybrid_alpha is not None:
            req.hybrid_alpha = hybrid_alpha
            
        if bm25 is not None:
            bm25_msg = hyperspace_pb2.Bm25Options()
            if "method" in bm25: bm25_msg.method = bm25["method"]
            if "k1" in bm25: bm25_msg.k1 = bm25["k1"]
            if "b" in bm25: bm25_msg.b = bm25["b"]
            if "delta" in bm25: bm25_msg.delta = bm25["delta"]
            if "language" in bm25: bm25_msg.language = bm25["language"]
            if "ngrams" in bm25: bm25_msg.ngrams = bm25["ngrams"]
            if "fusion_method" in bm25: bm25_msg.fusion_method = bm25["fusion_method"]
            req.bm25_options.CopyFrom(bm25_msg)
            
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

    def traverse(self, start_id: int, max_depth: int = 2, max_nodes: int = 256, layer: int = 0, traversal_mode: int = 0, filter: Dict[str, str] = None, filters: List[Dict] = None, collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.TraverseRequest(
            collection=collection,
            start_id=start_id,
            max_depth=max_depth,
            max_nodes=max_nodes,
            layer=layer,
            traversal_mode=traversal_mode,
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
                elif f.get("type") == "prefix":
                    req.filters.append(
                        hyperspace_pb2.Filter(
                            prefix=hyperspace_pb2.Prefix(key=f["key"], prefix=str(f["prefix"]))
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

    def exists(self, name: str) -> bool:
        try:
            self.get_collection_stats(name)
            return True
        except grpc.RpcError as e:
            if e.code() == grpc.StatusCode.NOT_FOUND or "not found" in str(e).lower():
                return False
            raise

    def get_metrics(self) -> Iterator[Dict]:
        req = hyperspace_pb2.MonitorRequest()
        try:
            stream = self.stub.Monitor(req, metadata=self.metadata)
            for ev in stream:
                yield {
                    "total_collections": ev.total_collections,
                    "total_vectors": ev.total_vectors,
                    "qps": ev.qps,
                    "cpu_usage": ev.cpu_usage,
                    "ram_usage_mb": ev.ram_usage_mb,
                    "latency_p99_ms": ev.latency_p99_ms
                }
        except grpc.RpcError as e:
            print(f"RPC Error in get_metrics: {e}")
            return

    def search_multi_collection(self, query: List[float], collections: List[str], top_k: int = 10) -> Dict[str, List[Dict]]:
        req = hyperspace_pb2.SearchMultiCollectionRequest(
            collections=collections,
            vector=self._normalize_vector(query),
            top_k=top_k
        )
        try:
            resp = self.stub.SearchMultiCollection(req, metadata=self.metadata)
            result = {}
            for col_name, search_resp in resp.responses.items():
                result[col_name] = [
                    {
                        "id": r.id,
                        "distance": r.distance,
                        "metadata": (dict(r.metadata) if r.metadata else {}),
                        "typed_metadata": dict(r.typed_metadata) if r.typed_metadata else {}
                    }
                    for r in search_resp.results
                ]
            return result
        except grpc.RpcError as e:
            print(f"RPC Error in search_multi_collection: {e}")
            return {}

    def search_multi_collection_text(self, text: str, collections: List[str], top_k: int = 10) -> Dict[str, List[Dict]]:
        vector = self.vectorize(text)
        return self.search_multi_collection(vector, collections, top_k)

    def trigger_reconsolidation(self, collection: str, target_vector: List[float], learning_rate: float = 0.01) -> bool:
        req = hyperspace_pb2.ReconsolidationRequest(
            collection=collection,
            target_vector=self._normalize_vector(target_vector),
            learning_rate=learning_rate
        )
        try:
            self.stub.TriggerReconsolidation(req, metadata=self.metadata)
            return True
        except grpc.RpcError as e:
            print(f"RPC Error in trigger_reconsolidation: {e}")
            return False


    def predict_relation(self, id_a: int, id_b: int, collection: str = "", curvature: float = 1.0) -> List[float]:
        """
        Computes the semantic relation vector R such that A + R ≈ B.
        """
        pts = self.get_points([id_a, id_b], collection=collection)
        if len(pts) < 2:
            return []
        
        id_to_vec = {p["id"]: p["vector"] for p in pts}
        vec_a = id_to_vec[id_a]
        vec_b = id_to_vec[id_b]
        
        from .math import log_map
        return log_map(vec_a, vec_b, c=curvature)

    def close(self):
        for c in self.channels:
            try:
                c.close()
            except:
                pass

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()

    def get_points(self, ids: List[int], collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.GetPointsRequest(collection=collection, ids=ids)
        try:
            resp = self.stub.GetPoints(req, metadata=self.metadata)
            return [
                {
                    "id": p.id,
                    "vector": list(p.vector),
                    "metadata": dict(p.metadata),
                    "typed_metadata": dict(p.typed_metadata)
                }
                for p in resp.points
            ]
        except grpc.RpcError as e:
            print(f"RPC Error in get_points: {e}")
            return []

    def get_subsumption_tree(self, root_id: int, max_depth: int = 3, collection: str = "") -> List[Dict]:
        """
        Returns a directed hierarchy tree starting from root_id.
        """
        req = hyperspace_pb2.GetSubsumptionTreeRequest(
            collection=collection,
            root_id=root_id,
            max_depth=max_depth
        )
        try:
            resp = self.stub.GetSubsumptionTree(req, metadata=self.metadata)
            return [
                {
                    "id": n.id,
                    "metadata": dict(n.metadata),
                    "edge_types": n.edge_types
                }
                for n in resp.nodes
            ]
        except grpc.RpcError as e:
            print(f"RPC Error in get_subsumption_tree: {e}")
            return []

    def explore_graph(self, start_id: int, max_depth: int = 2, max_nodes: int = 256, collection: str = "") -> Dict:
        """
        High-level wrapper that returns a JSON Ego-Graph for visualization.
        """
        req = hyperspace_pb2.TraverseRequest(
            collection=collection,
            start_id=start_id,
            max_depth=max_depth,
            max_nodes=max_nodes,
            traversal_mode=1, # DIFFUSIVE
            breadth_limit=10
        )
        try:
            resp = self.stub.Traverse(req, metadata=self.metadata)
            nodes = [
                {
                    "id": n.id,
                    "metadata": dict(n.metadata),
                    "edge_types": n.edge_types,
                    "neighbors": list(n.neighbors)
                }
                for n in resp.nodes
            ]
            return {
                "nodes": nodes,
                "center_id": start_id,
                "count": len(nodes)
            }
        except grpc.RpcError as e:
            print(f"RPC Error in explore_graph: {e}")
            return {}

    def predict_momentum(self, trajectory_ids: List[int], steps: float = 1.0, collection: str = "", curvature: float = 1.0) -> List[float]:
        """
        Client-side trajectory prediction using Koopman/Riemannian math.
        """
        if len(trajectory_ids) < 2:
            return []
            
        pts = self.get_points(trajectory_ids, collection=collection)
        id_to_vec = {p["id"]: p["vector"] for p in pts}
        vectors = [id_to_vec[i] for i in trajectory_ids if i in id_to_vec]
        
        if len(vectors) < 2:
            return []
            
        from .math import koopman_extrapolate
        return koopman_extrapolate(vectors[-2], vectors[-1], steps, c=curvature)

    def get_trust_score(self, trajectory_ids: List[int], collection: str = "", curvature: float = 1.0) -> float:
        """
        Evaluates the Lyapunov stability of a trajectory.
        Lower values mean more stable/reliable path.
        """
        pts = self.get_points(trajectory_ids, collection=collection)
        id_to_vec = {p["id"]: p["vector"] for p in pts}
        vectors = [id_to_vec[i] for i in trajectory_ids if i in id_to_vec]
        
        if len(vectors) < 3:
            return 0.0
            
        from .math import lyapunov_convergence
        try:
            return lyapunov_convergence(vectors, c=curvature)
        except Exception:
            return 1.0 # Max chaos on error

    def update_payload(self, id: int, metadata: Dict[str, str], collection: str = "") -> bool:
        req = hyperspace_pb2.UpdatePayloadRequest(id=id, metadata=metadata, collection=collection)
        try:
            resp = self.stub.UpdatePayload(req, metadata=self.metadata)
            return resp.code == 0
        except grpc.RpcError:
            return False

    def scroll(self, limit: int, offset: int = 0, filters: Optional[List[Dict]] = None, collection: str = "") -> List[Dict]:
        req = hyperspace_pb2.ScrollRequest(limit=limit, offset=offset, collection=collection)
        if filters:
            req.filters.extend([self._to_proto_filter(f) for f in filters])
        try:
            resp = self.stub.Scroll(req, metadata=self.metadata)
            return [
                {
                    "id": p.id,
                    "vector": list(p.vector),
                    "metadata": dict(p.metadata),
                    "typed_metadata": dict(p.typed_metadata)
                }
                for p in resp.points
            ]
        except grpc.RpcError:
            return []

    def count(self, filters: Optional[List[Dict]] = None, collection: str = "") -> int:
        req = hyperspace_pb2.CountRequest(collection=collection)
        if filters:
            req.filters.extend([self._to_proto_filter(f) for f in filters])
        try:
            resp = self.stub.Count(req, metadata=self.metadata)
            return resp.count
        except grpc.RpcError:
            return 0

    def health_check(self) -> str:
        req = hyperspace_pb2.Empty()
        try:
            resp = self.stub.HealthCheck(req, metadata=self.metadata)
            return resp.status
        except grpc.RpcError:
            return "unreachable"

    def get_cache_stats(self, name: str) -> dict:
        import urllib.request
        import json
        ip = self.host.split(':')[0]
        url = f"http://{ip}:50050/api/collections/{name}/cache/stats"
        headers = {}
        if self.api_key:
            headers['x-api-key'] = self.api_key
        if self.user_id:
            headers['x-hyperspace-user-id'] = self.user_id
            
        req = urllib.request.Request(url, headers=headers, method='GET')
        try:
            with urllib.request.urlopen(req) as response:
                return json.loads(response.read().decode('utf-8'))
        except Exception as e:
            print(f"Error in get_cache_stats: {e}")
            return {}

    def clear_cache(self, name: str) -> bool:
        import urllib.request
        import json
        ip = self.host.split(':')[0]
        url = f"http://{ip}:50050/api/collections/{name}/cache/clear"
        headers = {'Content-Type': 'application/json'}
        if self.api_key:
            headers['x-api-key'] = self.api_key
        if self.user_id:
            headers['x-hyperspace-user-id'] = self.user_id
            
        data = json.dumps({}).encode('utf-8')
        req = urllib.request.Request(url, data=data, headers=headers, method='POST')
        try:
            with urllib.request.urlopen(req) as response:
                res_data = json.loads(response.read().decode('utf-8'))
                return res_data.get("status") == "success"
        except Exception as e:
            print(f"Error in clear_cache: {e}")
            return False

    def update_cache_config(self, name: str, policy: str, ann_threshold: Optional[float] = None) -> bool:
        import urllib.request
        import json
        ip = self.host.split(':')[0]
        url = f"http://{ip}:50050/api/collections/{name}/cache/config"
        headers = {'Content-Type': 'application/json'}
        if self.api_key:
            headers['x-api-key'] = self.api_key
        if self.user_id:
            headers['x-hyperspace-user-id'] = self.user_id
            
        payload = {"policy": policy}
        if ann_threshold is not None:
            payload["ann_threshold"] = ann_threshold
            
        data = json.dumps(payload).encode('utf-8')
        req = urllib.request.Request(url, data=data, headers=headers, method='POST')
        try:
            with urllib.request.urlopen(req) as response:
                res_data = json.loads(response.read().decode('utf-8'))
                return res_data.get("status") == "success"
        except Exception as e:
            print(f"Error in update_cache_config: {e}")
            return False

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
