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

    # 1. Insert Data
    print("🚀 Inserting vectors...")
    start = time.time()
    for i in range(100):
        vec = generate_poincare_point(DIM)
        try:
            stub.Insert(hyperspace_pb2.InsertRequest(vector=vec))
            if i % 10 == 0: print(f".", end="", flush=True)
        except grpc.RpcError as e:
            print(f"Insert failed: {e}")
            break
    
    elapsed = time.time() - start
    print(f"\n✅ Inserted 100 vectors in {elapsed:.4f}s")

    # 2. Search
    query_vec = generate_poincare_point(DIM)
    print(f"🔍 Searching nearest neighbors...")
    
    try:
        response = stub.Search(hyperspace_pb2.SearchRequest(vector=query_vec, top_k=5))
        print("🎯 Search Results:")
        for res in response.results:
            print(f"   ID: {res.id}, Dist: {res.distance:.4f}")
    except grpc.RpcError as e:
        print(f"❌ RPC Error: {e.details()}")

if __name__ == "__main__":
    run()
