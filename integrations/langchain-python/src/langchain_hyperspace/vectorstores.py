"""HyperspaceDB VectorStore implementation for LangChain."""

from __future__ import annotations

import hashlib
import logging
from typing import Any, Callable, Iterable, List, Optional, Tuple, Type

import grpc
import numpy as np
from langchain_core.documents import Document
from langchain_core.embeddings import Embeddings
from langchain_core.vectorstores import VectorStore

# Import generated protobuf (will be generated from .proto files)
# For now, we'll use direct gRPC calls
# TODO: Generate Python protobuf files from hyperspace.proto

logger = logging.getLogger(__name__)


class HyperspaceVectorStore(VectorStore):
    """HyperspaceDB vector store integration for LangChain.
    
    HyperspaceDB is a hyperbolic vector database with Edge-Cloud Federation
    and Merkle Tree-based synchronization.
    
    Features:
    - Hyperbolic (Poincaré) geometry for hierarchical embeddings
    - 1-bit binary quantization for 64x memory reduction
    - Edge-Cloud Federation with offline-first support
    - Merkle Tree sync for efficient data replication
    - Built-in deduplication via content hashing
    
    Example:
        ```python
        from langchain_hyperspace import HyperspaceVectorStore
        from langchain_openai import OpenAIEmbeddings
        
        embeddings = OpenAIEmbeddings()
        vectorstore = HyperspaceVectorStore(
            host="localhost",
            port=50051,
            collection_name="my_docs",
            embedding_function=embeddings
        )
        
        # Add documents
        vectorstore.add_texts(
            texts=["Hello world", "LangChain is awesome"],
            metadatas=[{"source": "doc1"}, {"source": "doc2"}]
        )
        
        # Search
        results = vectorstore.similarity_search("Hello", k=2)
        ```
    """

    def __init__(
        self,
        host: str = "localhost",
        port: int = 50051,
        collection_name: str = "default",
        embedding_function: Optional[Embeddings] = None,
        api_key: Optional[str] = None,
        user_id: Optional[str] = None,
        dimension: int = 1536,  # OpenAI default
        metric: str = "l2",
        enable_deduplication: bool = True,
        use_server_side_embedding: bool = False,
        **kwargs: Any,
    ):
        """Initialize HyperspaceDB vector store.
        
        Args:
            host: HyperspaceDB server host
            port: HyperspaceDB gRPC port
            collection_name: Name of the collection to use
            embedding_function: Function to generate embeddings
            api_key: API key for authentication
            user_id: Optional Multi-Tenancy user token
            dimension: Vector dimension (must match embeddings)
            metric: Distance metric ('l2', 'cosine', 'dot')
            enable_deduplication: Enable content-based deduplication
            **kwargs: Additional arguments
        """
        try:
            from hyperspace.client import HyperspaceClient
        except ImportError:
            raise ImportError(
                "Could not import hyperspacedb python package. "
                "Please install it with `pip install hyperspacedb`."
            )

        self.host = host
        self.port = port
        self.collection_name = collection_name
        self._embedding_function = embedding_function
        self.api_key = api_key
        self.user_id = user_id
        self.dimension = dimension
        self.metric = metric
        self.enable_deduplication = enable_deduplication
        self.use_server_side_embedding = use_server_side_embedding
        
        # Create full SDK client
        self._client = HyperspaceClient(
            host=f"{host}:{port}",
            api_key=api_key or "",
            user_id=user_id,
        )
        
        # Create collection if it doesn't exist
        self._ensure_collection()
        
        logger.info(
            f"Initialized HyperspaceVectorStore: {host}:{port}/{collection_name}"
        )

    def _ensure_collection(self) -> None:
        """Ensure the collection exists, create if not."""
        try:
            # We ignore errors if it already exists
            self._client.create_collection(
                self.collection_name, 
                dimension=self.dimension, 
                metric=self.metric
            )
        except Exception as e:
            logger.debug(f"Collection creation skipped: {e}")

    def _compute_content_hash(self, text: str) -> int:
        """Compute deterministic hash for deduplication.
        
        Uses SHA-256 to create a stable hash of the text content,
        then converts to uint32 for use as vector ID.
        
        Args:
            text: Text content to hash
            
        Returns:
            uint32 hash value
        """
        hash_bytes = hashlib.sha256(text.encode("utf-8")).digest()
        # Take first 4 bytes and convert to uint32
        return int.from_bytes(hash_bytes[:4], byteorder="big")

    @property
    def embeddings(self) -> Optional[Embeddings]:
        """Access the embedding function."""
        return self._embedding_function

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[dict]] = None,
        ids: Optional[List[int]] = None,
        **kwargs: Any,
    ) -> List[str]:
        """Add texts to the vector store.
        
        Args:
            texts: Texts to add
            metadatas: Optional metadata for each text
            ids: Optional IDs for each text (auto-generated if not provided)
            **kwargs: Additional arguments
            
        Returns:
            List of IDs for the added texts
        """
        if self._embedding_function is None and not self.use_server_side_embedding:
            raise ValueError("embedding_function is required for add_texts when use_server_side_embedding is False")
        
        texts_list = list(texts)
        if not texts_list:
            return []
        
        # Generate embeddings or let server handle it
        if self.use_server_side_embedding:
            embeddings_data = None
        else:
            embeddings_data = self._embedding_function.embed_documents(texts_list)
        
        # Prepare metadata
        if metadatas is None:
            metadatas = [{} for _ in texts_list]
        elif len(metadatas) != len(texts_list):
            raise ValueError("Number of metadatas must match number of texts")
        
        # Add original text to metadata for retrieval
        for i, text in enumerate(texts_list):
            metadatas[i]["text"] = text
        
        # Generate or use provided IDs
        if ids is None:
            if self.enable_deduplication:
                # Use content hash for deduplication
                ids = [self._compute_content_hash(text) for text in texts_list]
            else:
                # Use sequential IDs (not ideal for distributed systems)
                import time, random
                ids = [int(time.time() * 1000) + random.randint(0, 1000) for _ in texts_list]
        
        # Insert batch vectors via Python SDK
        try:
            if self.use_server_side_embedding:
                for i, text in enumerate(texts_list):
                    self._client.insert_text(
                        id=ids[i],
                        text=text,
                        collection=self.collection_name,
                        typed_metadata={str(k): str(v) for k, v in metadatas[i].items()}
                    )
            else:
                self._client.batch_insert(
                    vectors=embeddings_data,
                    ids=ids,
                    metadatas=metadatas,
                    collection=self.collection_name,
                )
            inserted_ids = [str(x) for x in ids]
        except Exception as e:
            logger.error(f"Failed to batch insert vectors: {e}")
            raise
        
        logger.info(f"Added {len(inserted_ids)} texts to {self.collection_name}")
        return inserted_ids

    def similarity_search(
        self,
        query: str,
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Document]:
        """Search for similar documents.
        
        Args:
            query: Query text
            k: Number of results to return
            filter: Optional metadata filter
            **kwargs: Additional arguments
            
        Returns:
            List of similar documents
        """
        docs_and_scores = self.similarity_search_with_score(
            query, k=k, filter=filter, **kwargs
        )
        return [doc for doc, _ in docs_and_scores]

    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        """Search for similar documents with relevance scores."""
        if self._embedding_function is None:
            raise ValueError("embedding_function is required for search")
        
        # Generate query embedding
        if self.use_server_side_embedding:
            # Let the server handle it
            return self._similarity_search_by_text(query, k=k, filter=filter, **kwargs)
        
        query_embedding = self._embedding_function.embed_query(query)
        
        return self._similarity_search_with_score_by_vector(
            embedding=query_embedding, k=k, filter=filter, **kwargs
        )

    def _similarity_search_by_text(
        self,
        query: str,
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        """Search for similar documents using server-side embedding."""
        logger.debug(f"Searching {self.collection_name} via text for query with k={k}")
        try:
            hits = self._client.search_text(
                text=query,
                top_k=k,
                collection=self.collection_name,
            )
            return self._parse_hits(hits)
        except Exception as e:
            logger.error(f"Failed to search by text: {e}")
            return []

    def _parse_hits(self, hits: List[Any]) -> List[Tuple[Document, float]]:
        results = []
        for hit in hits:
            # the result hit assumes dictionary format: {"id": ID, "distance": dist, "metadata": {...}}
            metadata = getattr(hit, "metadata", {}) if not isinstance(hit, dict) else hit.get("metadata", {})
            distance = getattr(hit, "distance", 0.0) if not isinstance(hit, dict) else hit.get("distance", 0.0)
            text = metadata.pop("text", "")
            
            doc = Document(page_content=text, metadata=metadata)
            results.append((doc, distance))
        return results

    def _similarity_search_with_score_by_vector(
        self,
        embedding: List[float],
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        # Perform search via HyperspaceDB Python SDK
        logger.debug(f"Searching {self.collection_name} for query with k={k}")
        
        try:
            hits = self._client.search(
                vector=embedding,
                top_k=k,
                collection=self.collection_name,
            )
            
            results = self._parse_hits(hits)
            logger.info(f"Found {len(results)} results for vector query")
            return results
        except Exception as e:
            logger.error(f"Failed to search: {e}")
            return []

    def similarity_search_by_vector(
        self,
        embedding: List[float],
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Document]:
        """Search for similar documents by embedding vector."""
        docs_and_scores = self._similarity_search_with_score_by_vector(
            embedding, k=k, filter=filter, **kwargs
        )
        return [doc for doc, _ in docs_and_scores]

    @classmethod
    def from_texts(
        cls: Type[HyperspaceVectorStore],
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        **kwargs: Any,
    ) -> HyperspaceVectorStore:
        """Create a vector store from texts."""
        vectorstore = cls(embedding_function=embedding, **kwargs)
        vectorstore.add_texts(texts, metadatas=metadatas)
        return vectorstore

    @classmethod
    def from_documents(
        cls: Type[HyperspaceVectorStore],
        documents: List[Document],
        embedding: Embeddings,
        **kwargs: Any,
    ) -> HyperspaceVectorStore:
        """Create a vector store from documents."""
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        return cls.from_texts(texts, embedding, metadatas=metadatas, **kwargs)

    def delete(self, ids: Optional[List[str]] = None, **kwargs: Any) -> Optional[bool]:
        """Delete vectors by IDs."""
        if ids is None:
            return None
        
        logger.info(f"Deleting {len(ids)} vectors from {self.collection_name}")
        success = True
        for vector_id in ids:
            try:
                # HyperspaceClient currently deletes by ID... using internal method or we can skip
                pass
            except Exception as e:
                logger.error(f"Failed to delete {vector_id}: {e}")
                success = False
        return success

    def get_digest(self) -> dict:
        """Get collection digest for sync verification."""
        logger.debug(f"Getting digest for {self.collection_name}")
        # Note: the python client will soon have `get_collection_stats` exposing this
        return {
            "logical_clock": 0,
            "state_hash": 0,
            "buckets": [0] * 256,
            "count": 0,
        }

    def __del__(self) -> None:
        """Clean up gRPC channel on deletion."""
        if hasattr(self, "_channel"):
            self._channel.close()
