import grpc
import numpy as np
import time
import hyperspace_pb2
import hyperspace_pb2_grpc

# Config
DIM = 8
HOST = "localhost:50051"

def generate_poincare_point(dim):
    """Generates a random point inside the Poincaré ball"""
    vec = np.random.uniform(-1, 1, dim)
    norm = np.linalg.norm(vec)
    if norm >= 1:
        vec = vec / (norm + 0.1) # Normalize inside
    return vec.tolist()

def run():
    print(f"Connecting to {HOST}...")
    try:
        channel = grpc.insecure_channel(HOST)
        stub = hyperspace_pb2_grpc.DatabaseStub(channel)
        
        # Check connection implicitly by calling Insert
        pass
    except Exception as e:
        print(f"Connection failed: {e}")
        return

    channel = grpc.insecure_channel(HOST)
    stub = hyperspace_pb2_grpc.DatabaseStub(channel)

    # 1. Insert Data with Metadata
    print("🚀 Inserting vectors...")
    start = time.time()
    for i in range(100):
        vec = generate_poincare_point(DIM)
        # Half "red", Half "blue"
        category = "red" if i % 2 == 0 else "blue"
        meta = {"category": category}
        
        try:
            stub.Insert(hyperspace_pb2.InsertRequest(vector=vec, metadata=meta))
            if i % 10 == 0: print(f".", end="", flush=True)
        except grpc.RpcError as e:
            print(f"Insert failed: {e}")
            break
    
    elapsed = time.time() - start
    print(f"\n✅ Inserted 100 vectors in {elapsed:.4f}s")

    # 2. Search with Filter
    query_vec = generate_poincare_point(DIM)
    
    print(f"\n🔍 Searching nearest neighbors (Filter: category='red')...")
    try:
        # Filter for RED only
        f = {"category": "red"}
        response = stub.Search(hyperspace_pb2.SearchRequest(vector=query_vec, top_k=5, filter=f))
        print("🎯 Search Results (Red Only):")
        for res in response.results:
            print(f"   ID: {res.id}, Dist: {res.distance:.4f}")
            # Note: We can't verify metadata in response because SearchResult doesn't return metadata yet. 
            # But the detailed logic in server ensures filtering.
            # ID check: Even IDs are red.
            if res.id % 2 != 0:
                print(f"   ⚠️ ERROR: Found ID {res.id} which should be blue (odd)!")
    except grpc.RpcError as e:
        print(f"❌ RPC Error: {e.details()}")

    print(f"\n🔍 Searching nearest neighbors (Filter: category='blue')...")
    try:
        # Filter for BLUE only
        f = {"category": "blue"}
        response = stub.Search(hyperspace_pb2.SearchRequest(vector=query_vec, top_k=5, filter=f))
        print("🎯 Search Results (Blue Only):")
        for res in response.results:
            print(f"   ID: {res.id}, Dist: {res.distance:.4f}")
            if res.id % 2 == 0:
                print(f"   ⚠️ ERROR: Found ID {res.id} which should be red (even)!")
    except grpc.RpcError as e:
        print(f"❌ RPC Error: {e.details()}")

if __name__ == "__main__":
    run()
