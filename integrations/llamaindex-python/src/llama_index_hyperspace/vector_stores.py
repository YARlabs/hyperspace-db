import logging
from typing import Any, List, Optional, cast

import numpy as np
from llama_index.core.schema import BaseNode, NodeWithScore, TextNode
from llama_index.core.vector_stores.types import (
    MetadataFilters,
    VectorStore,
    VectorStoreQuery,
    VectorStoreQueryResult,
)
from llama_index.core.vector_stores.utils import (
    metadata_dict_to_node,
    node_to_metadata_dict,
)

from hyperspace.client import HyperspaceClient

logger = logging.getLogger(__name__)

class HyperspaceVectorStore(VectorStore):
    """Hyperspace Vector Store.

    In this vector store, embeddings are stored on a Hyperspace backend.

    Args:
        collection_name (str): Name of the collection.
        host (str): Host of the Hyperspace server.
        port (int): Port of the Hyperspace server.
        api_key (str): API key for Hyperspace.
        user_id (str): User ID for Hyperspace.
    """

    stores_text: bool = True
    is_embedding_query: bool = True

    def __init__(
        self,
        collection_name: str,
        host: str = "localhost",
        port: int = 50051,
        api_key: Optional[str] = None,
        user_id: Optional[str] = None,
        **kwargs: Any,
    ) -> None:
        """Init params."""
        self._collection_name = collection_name
        self._client = HyperspaceClient(
            host=f"{host}:{port}",
            api_key=api_key,
            user_id=user_id
        )

    @property
    def client(self) -> Any:
        """Get client."""
        return self._client

    def add(
        self,
        nodes: List[BaseNode],
        **add_kwargs: Any,
    ) -> List[str]:
        """Add nodes to index.

        Args:
            nodes: List[BaseNode]: list of nodes with embeddings

        """
        for node in nodes:
            metadata = node_to_metadata_dict(
                node, remove_text=not self.stores_text, flat_metadata=True
            )
            # Hyperspace expects uint32 IDs. We'll try to parse or hash the node_id.
            try:
                hs_id = int(node.node_id)
            except ValueError:
                import hashlib
                hs_id = int(hashlib.md5(node.node_id.encode()).hexdigest(), 16) % (2**32)

            self._client.insert(
                id=hs_id,
                vector=node.get_embedding(),
                collection=self._collection_name,
                metadata={k: str(v) for k, v in metadata.items()}
            )

        return [node.node_id for node in nodes]

    def delete(self, ref_doc_id: str, **delete_kwargs: Any) -> None:
        """
        Delete nodes using with ref_doc_id.

        Args:
            ref_doc_id (str): The doc_id of the reference document.

        """
        # LlamaIndex delete often works by ref_doc_id. 
        # In this simple implementation, we might need a metadata index to find all nodes for a doc.
        # For now, we'll implement single ID deletion if ref_doc_id is numeric.
        try:
            hs_id = int(ref_doc_id)
            self._client.delete(id=hs_id, collection=self._collection_name)
        except ValueError:
            logger.warning("Hyperspace requires uint32 IDs for deletion. Non-numeric ref_doc_id skipped.")

    def query(
        self,
        query: VectorStoreQuery,
        **kwargs: Any,
    ) -> VectorStoreQueryResult:
        """Query index for top k results.

        Args:
            query: VectorStoreQuery: query object

        """
        if query.filters is not None:
             # Process filters into Hyperspace format if needed
             pass

        results = self._client.search(
            vector=query.query_embedding,
            top_k=query.similarity_top_k,
            collection=self._collection_name
        )

        nodes = []
        similarities = []
        ids = []

        for res in results:
            # Reconstruct node from metadata
            metadata = res.get("metadata", {})
            node = metadata_dict_to_node(metadata)
            
            # If text is not in metadata, we might need to fetch it or it's a vector-only store
            if isinstance(node, TextNode) and not node.get_text():
                 # Handle missing text
                 pass
            
            nodes.append(node)
            similarities.append(res.get("score", 0.0))
            ids.append(str(res.get("id")))

        return VectorStoreQueryResult(nodes=nodes, similarities=similarities, ids=ids)
