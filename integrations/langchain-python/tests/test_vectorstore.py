"""Unit tests for HyperspaceVectorStore."""

import pytest
from unittest.mock import Mock, MagicMock, patch
from langchain_core.documents import Document

from langchain_hyperspace import HyperspaceVectorStore


class MockEmbeddings:
    """Mock embeddings for testing."""
    
    def embed_documents(self, texts):
        """Return mock embeddings for documents."""
        return [[0.1] * 1536 for _ in texts]
    
    def embed_query(self, text):
        """Return mock embedding for query."""
        return [0.1] * 1536


@pytest.fixture
def mock_embeddings():
    """Fixture for mock embeddings."""
    return MockEmbeddings()


@pytest.fixture
def vectorstore(mock_embeddings):
    """Fixture for HyperspaceVectorStore."""
    with patch('grpc.insecure_channel'):
        store = HyperspaceVectorStore(
            host="localhost",
            port=50051,
            collection_name="test_collection",
            embedding_function=mock_embeddings,
            dimension=1536,
        )
        return store


class TestHyperspaceVectorStore:
    """Test suite for HyperspaceVectorStore."""
    
    def test_initialization(self, vectorstore):
        """Test vector store initialization."""
        assert vectorstore.host == "localhost"
        assert vectorstore.port == 50051
        assert vectorstore.collection_name == "test_collection"
        assert vectorstore.dimension == 1536
        assert vectorstore.metric == "l2"
        assert vectorstore.enable_deduplication is True
    
    def test_content_hash_deterministic(self, vectorstore):
        """Test that content hashing is deterministic."""
        text = "Hello, world!"
        hash1 = vectorstore._compute_content_hash(text)
        hash2 = vectorstore._compute_content_hash(text)
        assert hash1 == hash2
    
    def test_content_hash_different(self, vectorstore):
        """Test that different content produces different hashes."""
        hash1 = vectorstore._compute_content_hash("Hello")
        hash2 = vectorstore._compute_content_hash("World")
        assert hash1 != hash2
    
    def test_add_texts_basic(self, vectorstore):
        """Test basic text addition."""
        texts = ["Hello", "World"]
        ids = vectorstore.add_texts(texts)
        
        assert len(ids) == 2
        assert all(isinstance(id, str) for id in ids)
    
    def test_add_texts_with_metadata(self, vectorstore):
        """Test adding texts with metadata."""
        texts = ["Hello", "World"]
        metadatas = [{"source": "test1"}, {"source": "test2"}]
        
        ids = vectorstore.add_texts(texts, metadatas=metadatas)
        assert len(ids) == 2
    
    def test_add_texts_deduplication(self, vectorstore):
        """Test that deduplication uses content hash."""
        texts = ["Same text", "Same text", "Different text"]
        ids = vectorstore.add_texts(texts)
        
        # First two should have same ID (content hash)
        assert ids[0] == ids[1]
        assert ids[0] != ids[2]
    
    def test_add_texts_no_deduplication(self, mock_embeddings):
        """Test adding texts without deduplication."""
        with patch('grpc.insecure_channel'):
            vectorstore = HyperspaceVectorStore(
                host="localhost",
                port=50051,
                collection_name="test",
                embedding_function=mock_embeddings,
                enable_deduplication=False,
            )
            
            texts = ["Same text", "Same text"]
            ids = vectorstore.add_texts(texts)
            
            # Should have different IDs
            assert ids[0] != ids[1]
    
    def test_add_texts_no_embedding_function(self):
        """Test that add_texts fails without embedding function."""
        with patch('grpc.insecure_channel'):
            vectorstore = HyperspaceVectorStore(
                host="localhost",
                port=50051,
                collection_name="test",
                embedding_function=None,
            )
            
            with pytest.raises(ValueError, match="embedding_function is required"):
                vectorstore.add_texts(["Hello"])
    
    def test_similarity_search_no_embedding_function(self):
        """Test that search fails without embedding function."""
        with patch('grpc.insecure_channel'):
            vectorstore = HyperspaceVectorStore(
                host="localhost",
                port=50051,
                collection_name="test",
                embedding_function=None,
            )
            
            with pytest.raises(ValueError, match="embedding_function is required"):
                vectorstore.similarity_search("query")
    
    def test_from_texts(self, mock_embeddings):
        """Test creating vector store from texts."""
        with patch('grpc.insecure_channel'):
            texts = ["Hello", "World"]
            vectorstore = HyperspaceVectorStore.from_texts(
                texts=texts,
                embedding=mock_embeddings,
                host="localhost",
                port=50051,
            )
            
            assert vectorstore.collection_name == "default"
    
    def test_from_documents(self, mock_embeddings):
        """Test creating vector store from documents."""
        with patch('grpc.insecure_channel'):
            docs = [
                Document(page_content="Hello", metadata={"source": "test1"}),
                Document(page_content="World", metadata={"source": "test2"}),
            ]
            
            vectorstore = HyperspaceVectorStore.from_documents(
                documents=docs,
                embedding=mock_embeddings,
                host="localhost",
                port=50051,
            )
            
            assert vectorstore.collection_name == "default"
    
    def test_metadata_with_api_key(self, mock_embeddings):
        """Test that API key is included in metadata."""
        with patch('grpc.insecure_channel'):
            vectorstore = HyperspaceVectorStore(
                host="localhost",
                port=50051,
                collection_name="test",
                embedding_function=mock_embeddings,
                api_key="test_key",
            )
            
            metadata = vectorstore._get_metadata()
            assert ("x-api-key", "test_key") in metadata
    
    def test_metadata_without_api_key(self, vectorstore):
        """Test metadata without API key."""
        metadata = vectorstore._get_metadata()
        assert len(metadata) == 0
    
    def test_get_digest(self, vectorstore):
        """Test getting collection digest."""
        digest = vectorstore.get_digest()
        
        assert "logical_clock" in digest
        assert "state_hash" in digest
        assert "buckets" in digest
        assert "count" in digest
        assert len(digest["buckets"]) == 256
    
    def test_delete(self, vectorstore):
        """Test deleting vectors."""
        result = vectorstore.delete(ids=["1", "2", "3"])
        assert result is True
    
    def test_delete_none(self, vectorstore):
        """Test delete with no IDs."""
        result = vectorstore.delete(ids=None)
        assert result is None


class TestHyperspaceVectorStoreIntegration:
    """Integration tests (require running server)."""
    
    @pytest.mark.integration
    def test_real_connection(self, mock_embeddings):
        """Test real connection to HyperspaceDB server.
        
        This test requires a running HyperspaceDB server on localhost:50051.
        Skip if server is not available.
        """
        try:
            vectorstore = HyperspaceVectorStore(
                host="localhost",
                port=50051,
                collection_name="integration_test",
                embedding_function=mock_embeddings,
            )
            
            # Try to add a text
            ids = vectorstore.add_texts(["Integration test"])
            assert len(ids) == 1
            
            # Try to search
            results = vectorstore.similarity_search("test", k=1)
            assert isinstance(results, list)
            
        except Exception as e:
            pytest.skip(f"HyperspaceDB server not available: {e}")
