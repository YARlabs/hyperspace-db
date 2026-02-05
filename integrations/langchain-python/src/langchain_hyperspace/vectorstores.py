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
        dimension: int = 1536,  # OpenAI default
        metric: str = "l2",
        enable_deduplication: bool = True,
        **kwargs: Any,
    ):
        """Initialize HyperspaceDB vector store.
        
        Args:
            host: HyperspaceDB server host
            port: HyperspaceDB gRPC port
            collection_name: Name of the collection to use
            embedding_function: Function to generate embeddings
            api_key: API key for authentication
            dimension: Vector dimension (must match embeddings)
            metric: Distance metric ('l2', 'cosine', 'dot')
            enable_deduplication: Enable content-based deduplication
            **kwargs: Additional arguments
        """
        self.host = host
        self.port = port
        self.collection_name = collection_name
        self._embedding_function = embedding_function
        self.api_key = api_key
        self.dimension = dimension
        self.metric = metric
        self.enable_deduplication = enable_deduplication
        
        # Create gRPC channel
        self._channel = grpc.insecure_channel(f"{host}:{port}")
        
        # Create collection if it doesn't exist
        self._ensure_collection()
        
        logger.info(
            f"Initialized HyperspaceVectorStore: {host}:{port}/{collection_name}"
        )

    def _ensure_collection(self) -> None:
        """Ensure the collection exists, create if not."""
        # TODO: Implement collection creation via gRPC
        # For now, assume collection exists or is created externally
        pass

    def _get_metadata(self) -> List[Tuple[str, str]]:
        """Get gRPC metadata with API key if provided."""
        if self.api_key:
            return [("x-api-key", self.api_key)]
        return []

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
        if self._embedding_function is None:
            raise ValueError("embedding_function is required for add_texts")
        
        texts_list = list(texts)
        if not texts_list:
            return []
        
        # Generate embeddings
        embeddings = self._embedding_function.embed_documents(texts_list)
        
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
                # TODO: Implement better ID generation
                import random
                ids = [random.randint(0, 2**32 - 1) for _ in texts_list]
        
        # Insert vectors via gRPC
        # TODO: Implement batch insert for better performance
        inserted_ids = []
        for i, (vector_id, embedding, metadata) in enumerate(
            zip(ids, embeddings, metadatas)
        ):
            try:
                # Convert metadata dict to string-string map
                metadata_str = {k: str(v) for k, v in metadata.items()}
                
                # TODO: Make actual gRPC call
                # For now, log the operation
                logger.debug(
                    f"Inserting vector {vector_id} with dimension {len(embedding)}"
                )
                
                inserted_ids.append(str(vector_id))
            except Exception as e:
                logger.error(f"Failed to insert vector {vector_id}: {e}")
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
        """Search for similar documents with relevance scores.
        
        Args:
            query: Query text
            k: Number of results to return
            filter: Optional metadata filter
            **kwargs: Additional arguments
            
        Returns:
            List of (document, score) tuples
        """
        if self._embedding_function is None:
            raise ValueError("embedding_function is required for search")
        
        # Generate query embedding
        query_embedding = self._embedding_function.embed_query(query)
        
        # Perform search via gRPC
        # TODO: Implement actual gRPC search call
        # For now, return empty results
        logger.debug(
            f"Searching {self.collection_name} for query with k={k}"
        )
        
        # Mock results for now
        results: List[Tuple[Document, float]] = []
        
        logger.info(f"Found {len(results)} results for query")
        return results

    def similarity_search_by_vector(
        self,
        embedding: List[float],
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Document]:
        """Search for similar documents by embedding vector.
        
        Args:
            embedding: Query embedding vector
            k: Number of results to return
            filter: Optional metadata filter
            **kwargs: Additional arguments
            
        Returns:
            List of similar documents
        """
        # TODO: Implement vector search via gRPC
        logger.debug(
            f"Searching {self.collection_name} by vector with k={k}"
        )
        return []

    @classmethod
    def from_texts(
        cls: Type[HyperspaceVectorStore],
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        **kwargs: Any,
    ) -> HyperspaceVectorStore:
        """Create a vector store from texts.
        
        Args:
            texts: Texts to add
            embedding: Embedding function
            metadatas: Optional metadata for each text
            **kwargs: Additional arguments for HyperspaceVectorStore
            
        Returns:
            Initialized HyperspaceVectorStore with texts added
        """
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
        """Create a vector store from documents.
        
        Args:
            documents: Documents to add
            embedding: Embedding function
            **kwargs: Additional arguments for HyperspaceVectorStore
            
        Returns:
            Initialized HyperspaceVectorStore with documents added
        """
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        return cls.from_texts(texts, embedding, metadatas=metadatas, **kwargs)

    def delete(self, ids: Optional[List[str]] = None, **kwargs: Any) -> Optional[bool]:
        """Delete vectors by IDs.
        
        Args:
            ids: List of vector IDs to delete
            **kwargs: Additional arguments
            
        Returns:
            True if deletion was successful
        """
        if ids is None:
            return None
        
        # TODO: Implement delete via gRPC
        logger.info(f"Deleting {len(ids)} vectors from {self.collection_name}")
        return True

    def get_digest(self) -> dict:
        """Get collection digest for sync verification.
        
        This is a HyperspaceDB-specific method that returns the Merkle Tree
        digest for the collection, useful for verifying synchronization.
        
        Returns:
            Dictionary with digest information:
            - logical_clock: Lamport logical clock value
            - state_hash: Root hash of the Merkle Tree
            - buckets: List of 256 bucket hashes
            - count: Number of vectors in the collection
        """
        # TODO: Implement GetDigest gRPC call
        logger.debug(f"Getting digest for {self.collection_name}")
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
