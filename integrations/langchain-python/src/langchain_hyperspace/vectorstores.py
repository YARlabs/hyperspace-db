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

logger = logging.getLogger(__name__)

class HyperspaceVectorStore(VectorStore):
    """HyperspaceDB vector store integration for LangChain."""

    def __init__(
        self,
        host: str = "localhost",
        port: int = 50051,
        collection_name: str = "default",
        embedding_function: Optional[Embeddings] = None,
        api_key: Optional[str] = None,
        user_id: Optional[str] = None,
        dimension: int = 1536,
        metric: str = "l2",
        enable_deduplication: bool = True,
        use_server_side_embedding: bool = False,
        **kwargs: Any,
    ):
        from .client import HyperspaceClient
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
        
        self._client = HyperspaceClient(
            host=f"{host}:{port}",
            api_key=api_key,
            user_id=user_id,
        )
        
        if not use_server_side_embedding and embedding_function is None:
            raise ValueError("embedding_function is required when use_server_side_embedding is False")

        self._ensure_collection()
        logger.info(f"Initialized HyperspaceVectorStore: {host}:{port}/{collection_name}")

    def _ensure_collection(self) -> None:
        try:
            # Try to fetch existing collection metadata to avoid mismatch
            collections = self._client.list_collections()
            existing = next((c for c in collections if c["name"] == self.collection_name), None)
            
            if existing:
                self.dimension = int(existing["dimension"])
                self.metric = str(existing["metric"])
                logger.info(f"Using existing collection {self.collection_name}: {self.dimension}d, {self.metric}")
            else:
                self._client.create_collection(
                    self.collection_name, 
                    dimension=self.dimension, 
                    metric=self.metric
                )
        except Exception as e:
            logger.debug(f"Collection creation/fetching skipped: {e}")


    def _compute_content_hash(self, text: str) -> int:
        hash_bytes = hashlib.sha256(text.encode("utf-8")).digest()
        return int.from_bytes(hash_bytes[:4], byteorder="big")

    @property
    def embeddings(self) -> Optional[Embeddings]:
        return self._embedding_function

    def add_texts(
        self,
        texts: Iterable[str],
        metadatas: Optional[List[dict]] = None,
        ids: Optional[List[int]] = None,
        **kwargs: Any,
    ) -> List[str]:
        texts_list = list(texts)
        if not texts_list:
            return []
        
        if metadatas is None:
            metadatas = [{} for _ in texts_list]
        
        for i, text in enumerate(texts_list):
            metadatas[i]["text"] = text
        
        if ids is None:
            if self.enable_deduplication:
                ids = [self._compute_content_hash(text) for text in texts_list]
            else:
                import time, random
                ids = [int(time.time() * 1000) + random.randint(0, 1000) for _ in texts_list]
        
        try:
            if self.use_server_side_embedding:
                for i, text in enumerate(texts_list):
                    self._client.insert_text(
                        id=ids[i],
                        text=text,
                        collection=self.collection_name,
                        metadata={str(k): str(v) for k, v in metadatas[i].items()}
                    )
            else:
                if self._embedding_function is None:
                    raise ValueError("Embedding function is required")
                embeddings_data = self._embedding_function.embed_documents(texts_list)
                self._client.batch_insert(
                    vectors=embeddings_data,
                    ids=ids,
                    metadatas=[{str(k): str(v) for k, v in m.items()} for m in metadatas],
                    collection=self.collection_name,
                )
            return [str(x) for x in ids]
        except Exception as e:
            logger.error(f"Failed to batch insert vectors: {e}")
            raise

    def similarity_search_with_score(
        self,
        query: str,
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Tuple[Document, float]]:
        if self.use_server_side_embedding:
            hits = self._client.search_text(text=query, top_k=k, collection=self.collection_name)
        else:
            if self._embedding_function is None:
                raise ValueError("Embedding function is required")
            embedding = self._embedding_function.embed_query(query)
            hits = self._client.search(vector=embedding, top_k=k, collection=self.collection_name)
        
        return self._parse_hits(hits)

    def _parse_hits(self, hits: List[Any]) -> List[Tuple[Document, float]]:
        results = []
        for hit in hits:
            metadata = getattr(hit, "metadata", {}) if not isinstance(hit, dict) else hit.get("metadata", {})
            distance = getattr(hit, "distance", 0.0) if not isinstance(hit, dict) else hit.get("distance", 0.0)
            text = metadata.get("text", "")
            doc = Document(page_content=text, metadata=dict(metadata))
            results.append((doc, float(distance)))
        return results

    def similarity_search(
        self,
        query: str,
        k: int = 4,
        filter: Optional[dict] = None,
        **kwargs: Any,
    ) -> List[Document]:
        docs_and_scores = self.similarity_search_with_score(query, k=k, filter=filter, **kwargs)
        return [doc for doc, _ in docs_and_scores]

    def delete(self, ids: Optional[List[str]] = None, **kwargs: Any) -> Optional[bool]:
        if ids is None:
            return None
        success = True
        for id_str in ids:
            try:
                id_num = int(id_str) if id_str.isdigit() else self._compute_content_hash(id_str)
                self._client.delete(id=id_num, collection=self.collection_name)
            except Exception as e:
                logger.error(f"Failed to delete {id_str}: {e}")
                success = False
        return success

    def max_marginal_relevance_search(
        self,
        query: str,
        k: int = 4,
        fetch_k: int = 20,
        lambda_mult: float = 0.5,
        **kwargs: Any,
    ) -> List[Document]:
        if self.use_server_side_embedding:
            return self.similarity_search(query, k=k, **kwargs)
        return super().max_marginal_relevance_search(query, k=k, fetch_k=fetch_k, lambda_mult=lambda_mult, **kwargs)

    def __del__(self) -> None:
        if hasattr(self, "_client"):
            self._client.close()

    @classmethod
    def from_texts(
        cls: Type[HyperspaceVectorStore],
        texts: List[str],
        embedding: Embeddings,
        metadatas: Optional[List[dict]] = None,
        **kwargs: Any,
    ) -> HyperspaceVectorStore:
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
        texts = [doc.page_content for doc in documents]
        metadatas = [doc.metadata for doc in documents]
        return cls.from_texts(texts, embedding, metadatas=metadatas, **kwargs)
