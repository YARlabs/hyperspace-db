import sys
import os
import unittest
from langchain_core.documents import Document
from langchain_core.embeddings import Embeddings
from langchain_hyperspace import HyperspaceVectorStore

class FakeEmbeddings(Embeddings):
    """Fake embeddings for testing."""
    def embed_documents(self, texts: list[str]) -> list[list[float]]:
        return [[0.1] * 1024 for _ in texts]
    def embed_query(self, text: str) -> list[float]:
        return [0.1] * 1024

def test_langchain_python():
    print("🚀 Starting LangChain Python Smoke Test...")
    
    collection_name = "test_langchain_python_col"
    
    # Initialize store
    embeddings = FakeEmbeddings()
    vector_store = HyperspaceVectorStore(
        collection_name=collection_name,
        host="localhost",
        port=50051,
        api_key="I_LOVE_HYPERSPACEDB",
        embedding_function=embeddings,
        dimension=1024,
        metric="cosine"
    )
    
    # Add texts
    print("📝 Adding texts...")
    texts = ["HyperspaceDB is fast.", "LangChain is modular."]
    metadatas = [{"source": "test1"}, {"source": "test2"}]
    vector_store.add_texts(texts, metadatas=metadatas)
    
    # Search
    print("🔍 Searching...")
    results = vector_store.similarity_search("fast database", k=1)
    
    if len(results) > 0:
        print(f"✅ Found: {results[0].page_content}")
        print("🎉 LangChain Python smoke test PASSED!")
    else:
        print("❌ LangChain Python smoke test FAILED: No results.")

if __name__ == "__main__":
    try:
        test_langchain_python()
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
