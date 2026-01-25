import grpc
import numpy as np
import typing
from . import hyperspace_pb2
from . import hyperspace_pb2_grpc

class HyperspaceClient:
    def __init__(self, host="localhost:50051"):
        self.channel = grpc.insecure_channel(host)
        self.stub = hyperspace_pb2_grpc.DatabaseStub(self.channel)

    def insert(self, vector: typing.List[float], metadata: typing.Optional[typing.Dict[str, str]] = None):
        """Insert a vector with optional metadata into HyperspaceDB."""
        if len(vector) != 8:
            raise ValueError("HyperspaceDB currently only supports 8D vectors")
            
        req = hyperspace_pb2.InsertRequest(
            vector=vector,
            metadata=metadata or {}
        )
        return self.stub.Insert(req)

    def search(self, vector: typing.List[float], top_k: int = 10, filter: typing.Optional[typing.Dict[str, str]] = None):
        """Perform Approximate Nearest Neighbor search."""
        if len(vector) != 8:
            raise ValueError("Query vector must be 8D")
            
        req = hyperspace_pb2.SearchRequest(
            vector=vector,
            top_k=top_k,
            filter=filter or {}
        )
        return self.stub.Search(req)

    def delete(self, vector_id: int):
        """Soft-delete a vector by its ID."""
        req = hyperspace_pb2.DeleteRequest(id=vector_id)
        return self.stub.Delete(req)

    def monitor(self):
        """Get a stream of system statistics."""
        req = hyperspace_pb2.MonitorRequest()
        return self.stub.Monitor(req)

    def configure(self, ef_search: typing.Optional[int] = None, ef_construction: typing.Optional[int] = None):
        """Dynamically tune database performance parameters."""
        req = hyperspace_pb2.ConfigUpdate(
            ef_search=ef_search,
            ef_construction=ef_construction
        )
        return self.stub.Configure(req)

    def trigger_snapshot(self):
        """Manually trigger an index snapshot to disk."""
        return self.stub.TriggerSnapshot(hyperspace_pb2.Empty())

    def trigger_vacuum(self):
        """Manually trigger vacuum (cleaning soft-deleted records)."""
        return self.stub.TriggerVacuum(hyperspace_pb2.Empty())
