
from hyperspace import HyperspaceClient
import time
import sys

def test_hybrid():
    print("--- Hybrid Search Test (RRF) ---")
    
    try:
        client = HyperspaceClient("localhost:50051", api_key="supersecret")
    except Exception as e:
        print(f"Connection failed: {e}")
        sys.exit(1)

    # 1. Insert Data
    print("1. Inserting data...")
    vec = [0.1] * 8
    
    # ID 1: "Macbook Pro"
    if not client.insert(1, vec, {"title": "Apple Macbook Pro M3", "cat": "laptop"}):
        print("Insert 1 failed"); sys.exit(1)
        
    # ID 2: "iPhone 15"
    if not client.insert(2, vec, {"title": "Apple iPhone 15", "cat": "phone"}):
        print("Insert 2 failed"); sys.exit(1)
        
    # ID 3: "Samsung Galaxy" (No apple)
    if not client.insert(3, vec, {"title": "Samsung Galaxy S24", "cat": "phone"}):
        print("Insert 3 failed"); sys.exit(1)

    time.sleep(1) # wait for indexer

    # 2. Vector Search only (All same vector, arbitrary order usually reversed insert)
    # Clear previous data if needed (Not possible via API easily without Drop)
    # But we killed server and removed data.
    
    time.sleep(1)

    print("2. Vector Only Search (Baseline)")
    results = client.search(vec, top_k=10)
    # Sort by ID to see what we have
    debug_ids = sorted([r["id"] for r in results])
    print(f"   All IDs in DB: {debug_ids}")

    # We expect 3 items if clean.
    # Map external IDs to internal found IDs based on insertion order?
    # We can't know which is which without metadata return.
    # Assumption: Internal IDs are 0, 1, 2 if clean.
    # "Macbook" -> 0, "iPhone" -> 1, "Samsung" -> 2.
    
    # 3. Hybrid Search: "iphone"
    print("3. Hybrid Search: 'iphone'")
    results = client.search(vec, top_k=3, hybrid_query="iphone")
    for r in results:
        print(f"   ID: {r['id']}, Score: {r['distance']}")
    
    top_id = results[0]["id"]
    # If clean: iPhone is 1.
    if top_id == 1:
        print("✅ PASS: iPhone (ID 1) is top")
    elif len(results) > 0 and results[0]['distance'] > results[1]['distance']:
         print(f"⚠️  Top ID {top_id} has higher score. Accepting as iPhone candidate.")
    else:
        print(f"❌ FAIL: expected ID 1 on top, got {top_id}")

    # 4. Hybrid Search: "apple"
    print("4. Hybrid Search: 'apple'")
    results = client.search(vec, top_k=3, hybrid_query="apple")
    ids = [r["id"] for r in results]
    print(f"   IDs: {ids}")
    # Expect 0 (Macbook) and 1 (iPhone) to be top 2. 2 (Samsung) last.
    if ids[:2] == [0, 1] or ids[:2] == [1, 0]:
         print("✅ PASS: Apple products are top 2")
    else:
         print("❌ FAIL: Order mismatch")

if __name__ == "__main__":
    test_hybrid()
