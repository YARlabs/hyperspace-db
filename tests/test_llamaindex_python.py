import sys
import os
from llama_index.core import Document, VectorStoreIndex, StorageContext
from llama_index_hyperspace import HyperspaceVectorStore
from llama_index.core.embeddings import MockEmbedding
from llama_index.core.llms import MockLLM
from llama_index.core import Settings

# Use Mock embedding and LLM
Settings.embed_model = MockEmbedding(embed_dim=1024)
Settings.llm = MockLLM()

def test_llamaindex_python():
    print("🚀 Starting LlamaIndex Python Smoke Test...")
    
    # Initialize Hyperspace Vector Store
    vector_store = HyperspaceVectorStore(
        collection_name="test_llamaindex_col",
        host="localhost",
        port=50051,
        api_key="I_LOVE_HYPERSPACEDB"
    )
    
    # Create collection if it doesn't exist
    from hyperspace import HyperspaceClient
    client = HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB")
    try:
        client.create_collection("test_llamaindex_col", dimension=1024, metric="cosine")
        print("✅ Created collection 'test_llamaindex_col'")
    except Exception as e:
        print(f"ℹ️ Collection check: {e}")
    
    # Create storage context
    storage_context = StorageContext.from_defaults(vector_store=vector_store)
    
    # Create simple documents
    documents = [
        Document(text="HyperspaceDB is a high-performance vector database."),
        Document(text="LlamaIndex is a data framework for LLM applications."),
        Document(text="Spatial AI is the next frontier of machine intelligence.")
    ]
    
    # Create index (this will embed and add documents)
    print("📝 Building index...")
    index = VectorStoreIndex.from_documents(
        documents, 
        storage_context=storage_context,
        show_progress=True
    )
    
    # Query index
    print("🔍 Querying index...")
    query_engine = index.as_query_engine()
    response = query_engine.query("What is HyperspaceDB?")
    print(f"✅ Response: {response}")
    
    # Check if response makes sense
    if response:
        print("🎉 Smoke test PASSED!")
    else:
        print("❌ Smoke test FAILED: No response.")

if __name__ == "__main__":
    try:
        test_llamaindex_python()
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
