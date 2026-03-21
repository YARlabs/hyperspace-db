import sys
import os
import time

# Ensure we can import the SDK
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../sdks/python')))

from hyperspace.client import HyperspaceClient

def main():
    print("🚀 HyperspaceDB Built-in Embedding Example")
    print("------------------------------------------")
    
    # 1. Connect to local instance
    # Make sure server is running with HYPERSPACE_EMBED=true
    client = HyperspaceClient(host="localhost:50051", api_key="I_LOVE_HYPERSPACEDB")

    col_name = "embedding_test_py"
    
    # Simple Cleanup
    try:
        client.delete_collection(col_name)
    except:
        pass
    
    # 2. Create a collection (we'll use Cosine for standard text search)
    print(f"Creating collection '{col_name}' with Cosine metric...")
    # dimension should match the server's embedding model for Cosine (e.g. 1024 for Qwen3)
    # Check server logs for activated models and dimensions
    success = client.create_collection(col_name, dimension=1024, metric="cosine")
    if not success:
        print("❌ Failed to create collection. Is the server running?")
        return
    
    # 3. Using insert_text
    print("\n📝 Inserting text documents via client.insert_text()...")
    documents = [
        "HyperspaceDB is a spatial AI engine built on Rust.",
        "Hyperbolic geometry is ideal for hierarchical data structures.",
        "Qwen3-Embedding-0.6B provides high-accuracy 1024d vectors.",
        "Robotics and autonomous agents need low-latency memory.",
        "Vectorization happens entirely on the server side now."
    ]
    
    for i, doc in enumerate(documents):
        print(f" -> Indexing: \"{doc[:40]}...\"")
        client.insert_text(
            id=i+1, 
            text=doc, 
            collection=col_name,
            typed_metadata={"source": "example", "length": len(doc)}
        )
    
    print("\n✅ Insertion complete. Waiting for async ingestion...")
    time.sleep(1.5) # Wait for HNSW background worker

    # 4. Search via search_text
    print("\n🔍 Searching via client.search_text()...")
    query = "How to handle hierarchies in vectors?"
    print(f"Query: \"{query}\"")
    
    results = client.search_text(
        text=query,
        top_k=3,
        collection=col_name
    )
    
    print("\nResults:")
    for res in results:
        print(f"  [ID: {res['id']}] Score: {res['distance']:.4f}")
        # In real app, you'd fetch metadata or document from elsewhere if needed
        # Or if the document is stored in metadata (common pattern)

    # 5. Raw vector lookup via client.vectorize()
    print("\n🧠 Manual vectorization via client.vectorize()...")
    test_str = "Explain spatial reasoning."
    vector = client.vectorize(test_str, metric="cosine")
    
    if vector:
        print(f"Vector generated for: \"{test_str}\"")
        print(f"First 5 dimensions: {vector[:5]}...")
    else:
        print("❌ Vectorization failed. Check if server embedding is enabled.")

    # 6. cleanup
    # client.delete_collection(col_name)
    print("\nDone. You can inspect the results in the Dashboard: http://localhost:50050")

if __name__ == "__main__":
    main()
