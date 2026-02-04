import grpc
import hyperspace_pb2
import hyperspace_pb2_grpc

def test_l2():
    channel = grpc.insecure_channel('localhost:50051')
    stub = hyperspace_pb2_grpc.DatabaseStub(channel)
    
    COLLECTION = "default" # Enforce creation via ENV vars or API first
    
    # 1. Insert Vector A (Origin) [0.0, ...]
    vec_a = [0.0] * 1024
    stub.insert(hyperspace_pb2.InsertRequest(
        vector=vec_a,
        id=1,
        collection=COLLECTION,
        metadata={"name": "origin"}
    ))
    
    # 2. Insert Vector B (0.5) -> L2 Sq = 0.25
    # Note: We use 0.5 to stay within default ScalarI8 quantization range [-1, 1]
    vec_b = [0.0] * 1024
    vec_b[0] = 0.5
    stub.insert(hyperspace_pb2.InsertRequest(
        vector=vec_b,
        id=2, # Internal ID will be auto-assigned
        collection=COLLECTION,
        metadata={"name": "vec_0.5"}
    ))
    
    # 3. Insert Vector C (0.8) -> L2 Sq = 0.64
    vec_c = [0.0] * 1024
    vec_c[0] = 0.8
    stub.insert(hyperspace_pb2.InsertRequest(
        vector=vec_c,
        id=3,
        collection=COLLECTION,
        metadata={"name": "vec_0.8"}
    ))
    
    print("Vectors inserted.")
    
    # 4. Search for Origin
    response = stub.search(hyperspace_pb2.SearchRequest(
        vector=vec_a,
        top_k=5,
        collection=COLLECTION
    ))
    
    print(f"Search results: {response.results}")
    
    found_b = False
    found_c = False
    
    for res in response.results:
        # Check distance for Vec B (~0.25)
        if abs(res.distance - 0.25) < 0.02:
            print(f"✅ Found Vec B (0.5) with distance {res.distance}")
            found_b = True
        # Check distance for Vec C (~0.64)
        if abs(res.distance - 0.64) < 0.02:
            print(f"✅ Found Vec C (0.8) with distance {res.distance}")
            found_c = True
            
    if found_b and found_c:
        print("🎉 L2 Test Passed!")
    else:
        print("❌ Test Failed to find expected distances.")

if __name__ == "__main__":
    test_l2()
