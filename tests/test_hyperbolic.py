import grpc
import math
import hyperspace_pb2
import hyperspace_pb2_grpc

def test_hyperbolic():
    channel = grpc.insecure_channel('localhost:50051')
    stub = hyperspace_pb2_grpc.DatabaseStub(channel)
    
    COLLECTION = "hyp_16" # Assuming manually created or server started with dim=16.
    
    # 1. Insert Origin
    vec_a = [0.0] * 16
    stub.insert(hyperspace_pb2.InsertRequest(
        vector=vec_a,
        id=1,
        collection=COLLECTION,
        metadata={"name": "origin"}
    ))
    
    # 2. Insert [0.5, 0...]
    vec_b = [0.0] * 16
    vec_b[0] = 0.5
    stub.insert(hyperspace_pb2.InsertRequest(
        vector=vec_b,
        id=2,
        collection=COLLECTION,
        metadata={"name": "vec_0.5"}
    ))
    
    print("Vectors inserted.")
    
    # 3. Search Origin
    response = stub.search(hyperspace_pb2.SearchRequest(
        vector=vec_a,
        top_k=2,
        collection=COLLECTION
    ))
    
    # Calc Expected
    # d(u,v) = acosh(1 + 2*||u-v||^2 / ((1-||u||^2)(1-||v||^2)))
    # u=0, v=0.5 -> ||u||=0, ||v||=0.5, ||u-v||^2=0.25
    # arg = 1 + 2 * 0.25 / (1 * 0.75) = 1 + 0.5/0.75 = 1.666...
    expected = math.acosh(1.0 + 2.0 * 0.25 / 0.75)
    print(f"Expected Distance: {expected:.4f}")
    
    found = False
    for res in response.results:
        print(f"Result ID: {res.id}, Dist: {res.distance}")
        if abs(res.distance - expected) < 0.01:
            print("✅ Found correct Hyperbolic distance!")
            found = True
            
    if found:
        print("🎉 Hyperbolic Test Passed!")
    else:
        print("❌ Test Failed")

if __name__ == "__main__":
    test_hyperbolic()
