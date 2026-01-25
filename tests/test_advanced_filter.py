from hyperspace import HyperspaceClient
import time
import sys

def test_filters():
    print("--- Advanced Filter Test ---")
    
    key = "supersecret"
    try:
        client = HyperspaceClient("localhost:50051", api_key=key)
    except Exception as e:
        print(f"Connection failed: {e}")
        sys.exit(1)

    # 1. Insert Data
    print("1. Inserting data...")
    vec = [0.1] * 8
    
    # ID 1: Price 100
    if not client.insert(1, vec, {"price": "100", "cat": "book"}):
        print("Insert 1 failed"); sys.exit(1)
        
    # ID 2: Price 200
    if not client.insert(2, vec, {"price": "200", "cat": "tech"}):
        print("Insert 2 failed"); sys.exit(1)
        
    # ID 3: Price 50
    if not client.insert(3, vec, {"price": "50", "cat": "book"}):
        print("Insert 3 failed"); sys.exit(1)

    time.sleep(1) # wait for indexer

    # 2. Test Range > 80 (Should match 1, 2 -> Internal 0, 1)
    print("2. Search price > 80")
    results = client.search(vec, top_k=10, filters=[
        {"type": "range", "key": "price", "gte": 80}
    ])
    ids = sorted([r["id"] for r in results])
    print(f"   Found IDs: {ids}")
    if ids != [0, 1]:
        print("❌ FAIL: Expected [0, 1]")
        sys.exit(1)
    
    # 3. Test Range < 150 (Should match 1, 3 -> Internal 0, 2)
    print("3. Search price < 150")
    results = client.search(vec, top_k=10, filters=[
        {"type": "range", "key": "price", "lte": 150}
    ])
    ids = sorted([r["id"] for r in results])
    print(f"   Found IDs: {ids}")
    if ids != [0, 2]:
        print("❌ FAIL: Expected [0, 2]")
        sys.exit(1)

    # 4. Test Range [80, 150] (Should match 1 -> Internal 0)
    print("4. Search 80 <= price <= 150")
    results = client.search(vec, top_k=10, filters=[
        {"type": "range", "key": "price", "gte": 80, "lte": 150}
    ])
    ids = sorted([r["id"] for r in results])
    print(f"   Found IDs: {ids}")
    if ids != [0]:
        print("❌ FAIL: Expected [0]")
        sys.exit(1)

    # 5. Test Compound: Price > 80 AND cat=tech (Should match 2 -> Internal 1)
    print("5. Search price > 80 AND cat=tech")
    results = client.search(vec, top_k=10, filters=[
        {"type": "range", "key": "price", "gte": 80},
        {"type": "match", "key": "cat", "value": "tech"}
    ])
    ids = sorted([r["id"] for r in results])
    print(f"   Found IDs: {ids}")
    if ids != [1]:
        print("❌ FAIL: Expected [1]")
        sys.exit(1)

    print("✅ ALL TESTS PASSED")

if __name__ == "__main__":
    test_filters()
