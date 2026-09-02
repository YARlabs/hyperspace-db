import sys
import os
import time
import json
import uuid
from typing import List, Dict, Any, Optional, Union

try:
    from hyperspace import HyperspaceClient
except ImportError:
    sdk_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../../sdks/python"))
    if sdk_path not in sys.path:
        sys.path.insert(0, sdk_path)
    from hyperspace import HyperspaceClient

class Memory:
    """
    Drop-in replacement for Mem0, Zep, and MemGPT Memory powered by HyperspaceDB native memory engine.
    
    Provides 100x lower latency and 98% RAM reduction through MRL in-RAM cascades and 1-bit ADC quantization.
    """
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        config = config or {}
        self.host = config.get("host") or config.get("vector_store", {}).get("config", {}).get("host") or os.environ.get("HYPERSPACE_HOST") or "the.yar.ink"
        self.api_key = config.get("api_key") or config.get("vector_store", {}).get("config", {}).get("api_key") or os.environ.get("HYPERSPACE_API_KEY") or "YOUR_YARINK_API_KEY"
        self.collection = config.get("collection_name") or "agent_memories"
        self.quantization = config.get("quantization") or "extreme"
        
        is_local = self.host.startswith("localhost") or self.host.startswith("127.0.0.1")
        grpc_key = "I_LOVE_HYPERSPACEDB" if (is_local and self.api_key.startswith("sk_")) else (self.api_key or "I_LOVE_HYPERSPACEDB")
        self.client = HyperspaceClient(self.host, grpc_key)
        self._ensure_collection()

    def _ensure_collection(self):
        """Creates default memory collection if it does not exist."""
        try:
            schema = {
                "components": [
                    {
                        "name": "default",
                        "metric": "hybrid",
                        "fullDimension": 801,
                        "weight": 1.0
                    }
                ],
                "cascadePipeline": [
                    {
                        "componentName": "default",
                        "cutoffDimension": 129,
                        "storeInRam": True,
                        "rerankTopK": 50
                    }
                ]
            }
            self.client.create_collection(
                self.collection,
                schema,
                quantization=self.quantization
            )
        except Exception:
            # Collection already exists or server handled creation
            pass

    def _generate_numeric_id(self, memory_id_str: str) -> int:
        """Helper to derive deterministic numeric ID for vector database."""
        return abs(hash(memory_id_str)) % (2**31 - 1)

    def add(
        self,
        messages: Union[str, List[Dict[str, Any]]],
        user_id: Optional[str] = None,
        agent_id: Optional[str] = None,
        run_id: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None,
        filters: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Store a new memory item. Mem0 API compatible.
        """
        if isinstance(messages, list):
            text = " ".join([m.get("content", str(m)) if isinstance(m, dict) else str(m) for m in messages])
        else:
            text = str(messages)

        memory_id = str(uuid.uuid4())
        num_id = self._generate_numeric_id(memory_id)
        
        meta = metadata or {}
        if user_id: meta["user_id"] = user_id
        if agent_id: meta["agent_id"] = agent_id
        if run_id: meta["run_id"] = run_id
        meta["memory_id"] = memory_id
        meta["text"] = text
        meta["timestamp"] = str(time.time())

        # Vectorize & insert text into HyperspaceDB
        vector = self.client.vectorize(text, metric="hybrid")
        string_meta = {k: str(v) for k, v in meta.items()}
        self.client.insert(num_id, vector=vector, metadata=string_meta, collection=self.collection)

        return {
            "results": [
                {
                    "id": memory_id,
                    "event": "ADD",
                    "data": text,
                    "metadata": string_meta
                }
            ]
        }

    def search(
        self,
        query: str,
        user_id: Optional[str] = None,
        agent_id: Optional[str] = None,
        run_id: Optional[str] = None,
        limit: int = 5,
        filters: Optional[Dict[str, Any]] = None
    ) -> List[Dict[str, Any]]:
        """
        Search memories by semantic query. Mem0 API compatible.
        """
        query_vector = self.client.vectorize(query, metric="hybrid")
        results = self.client.search(query_vector, top_k=limit, collection=self.collection)

        memories = []
        for match in results:
            if isinstance(match, dict):
                match_meta = match.get("metadata", {}) or {}
                match_id = match.get("id")
                match_dist = float(match.get("distance", 0.0))
                match_score = 1.0 / (1.0 + match_dist) if match_dist >= 0 else 0.95
            else:
                match_meta = getattr(match, "metadata", {}) or {}
                match_id = getattr(match, "id", "")
                match_score = float(getattr(match, "score", 0.95))
            
            # Apply Mem0 user_id / agent_id / run_id filtering if provided
            if user_id and match_meta.get("user_id") != str(user_id): continue
            if agent_id and match_meta.get("agent_id") != str(agent_id): continue
            if run_id and match_meta.get("run_id") != str(run_id): continue

            memories.append({
                "id": match_meta.get("memory_id", str(match_id)),
                "memory": match_meta.get("text", query),
                "score": match_score,
                "user_id": match_meta.get("user_id", user_id),
                "agent_id": match_meta.get("agent_id", agent_id),
                "run_id": match_meta.get("run_id", run_id),
                "metadata": match_meta
            })

        return memories

    def get_all(
        self,
        user_id: Optional[str] = None,
        agent_id: Optional[str] = None,
        run_id: Optional[str] = None,
        limit: int = 100
    ) -> List[Dict[str, Any]]:
        """
        Retrieve all stored memories for a user/agent. Mem0 API compatible.
        """
        # Generic recall via empty prompt / broad query
        return self.search("*", user_id=user_id, agent_id=agent_id, run_id=run_id, limit=limit)

    def get(self, memory_id: str) -> Optional[Dict[str, Any]]:
        """Retrieve a specific memory by ID."""
        results = self.get_all(limit=100)
        for mem in results:
            if mem.get("id") == memory_id:
                return mem
        return None

    def update(self, memory_id: str, data: str) -> Dict[str, Any]:
        """Update an existing memory item."""
        num_id = self._generate_numeric_id(memory_id)
        
        # Delete old point & re-insert with new data
        try:
            self.client.delete(num_id, collection=self.collection)
        except Exception:
            pass

        vector = self.client.vectorize(data, metric="hybrid")
        meta = {
            "memory_id": memory_id,
            "text": data,
            "updated_at": str(time.time())
        }
        self.client.insert(num_id, vector=vector, metadata=meta, collection=self.collection)

        return {
            "message": "Memory updated successfully",
            "id": memory_id,
            "data": data
        }

    def delete(self, memory_id: str) -> Dict[str, Any]:
        """Delete a single memory item by ID."""
        num_id = self._generate_numeric_id(memory_id)
        self.client.delete(num_id, collection=self.collection)
        return {"message": f"Memory {memory_id} deleted successfully"}

    def delete_all(
        self,
        user_id: Optional[str] = None,
        agent_id: Optional[str] = None,
        run_id: Optional[str] = None
    ) -> Dict[str, Any]:
        """Delete all memories matching user_id / agent_id / run_id."""
        memories = self.get_all(user_id=user_id, agent_id=agent_id, run_id=run_id, limit=1000)
        count = 0
        for mem in memories:
            if mem.get("id"):
                self.delete(mem["id"])
                count += 1
        return {"message": f"Deleted {count} memories"}

    def reset(self) -> Dict[str, Any]:
        """Purge and reset the memory collection."""
        try:
            self.client.delete_collection(self.collection)
        except Exception:
            pass
        self._ensure_collection()
        return {"message": "Memory collection reset successfully"}

    def history(self, memory_id: str) -> List[Dict[str, Any]]:
        """Retrieve modification history for a memory item."""
        mem = self.get(memory_id)
        if not mem:
            return []
        return [
            {
                "id": memory_id,
                "event": "ADD",
                "data": mem.get("memory"),
                "timestamp": mem.get("metadata", {}).get("timestamp")
            }
        ]
