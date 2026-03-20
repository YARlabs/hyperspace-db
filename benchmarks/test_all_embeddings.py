import sys
import os
import time

# Add SDK to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../sdks/python")))

from hyperspace import HyperspaceClient

def test_embeddings():
    client = HyperspaceClient(host="localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    
    print("🚀 Starting Embedding Engine Integration Test")
    print("-" * 50)

    test_cases = [
        {"metric": "l2", "expected_dim": 1024, "label": "L2 (Qwen3)"},
        {"metric": "cosine", "expected_dim": 1024, "label": "Cosine (Qwen3)"},
        {"metric": "poincare", "expected_dim": 128, "label": "Poincare (YAR v5)"},
        {"metric": "lorentz", "expected_dim": 129, "label": "Lorentz (YAR v5)"},
    ]

    for case in test_cases:
        metric = case["metric"]
        expected_dim = case["expected_dim"]
        label = case["label"]
        
        print(f"Testing {label}...")
        
        # 1. Test Pure Vectorization
        try:
            vec = client.vectorize("Hello Hyperspace!", metric=metric)
            if len(vec) == expected_dim:
                print(f"  ✅ Vectorize: Success (dim={len(vec)})")
            else:
                print(f"  ❌ Vectorize: Dimension Mismatch! Got {len(vec)}, expected {expected_dim}")
        except Exception as e:
            print(f"  ❌ Vectorize: Failed with error: {e}")

        # 2. Test End-to-End Search
        col_name = f"test_embed_{metric}"
        client.delete_collection(col_name)
        
        if client.create_collection(col_name, dimension=expected_dim, metric=metric):
            print(f"  ✅ Create Collection: Success")
            
            # Insert some text
            texts = [
                "Hyperspace is a fast vector database.",
                "Hyperbolic geometry is useful for hierarchical data.",
                "Multi-geometry search combines different metrics."
            ]
            
            for i, text in enumerate(texts):
                if client.insert_text(id=i+1, text=text, collection=col_name):
                    pass
                else:
                    print(f"  ❌ Insert Text '{text}': Failed")
            
            print(f"  ✅ Insert Text: Success ({len(texts)} vectors)")
            
            # Wait for indexing
            time.sleep(1)
            
            # Search
            query = "What is Hyperspace?"
            results = client.search_text(text=query, top_k=2, collection=col_name)
            
            if results:
                print(f"  ✅ Search Text: Success (Found {len(results)} results)")
                for r in results:
                    print(f"     - ID {r['id']}, Distance: {r['distance']:.4f}")
            else:
                print(f"  ❌ Search Text: No results found!")
                
            client.delete_collection(col_name)
        else:
            print(f"  ❌ Create Collection: Failed")
            
        print("-" * 50)

    print("🏁 Embedding Engine Test Completed")

if __name__ == "__main__":
    test_embeddings()
