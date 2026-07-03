import sys
import os

# Include SDK path
sys.path.append("/Users/sergeyglukhota/Downloads/cursor-tutor/YAR_INK/hyperspace-db/sdks/python")

from hyperspace import HyperspaceClient
import grpc
from hyperspace.proto import hyperspace_pb2, hyperspace_pb2_grpc

def main():
    print("=== STARTING LIVE ENCRYPTION TEST ON HYPERSPACEDB ===")
    
    # 1. Initialize Client
    client = HyperspaceClient(host="localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    collection = "secure_test_collection"
    
    # Delete if exists
    client.delete_collection(collection)
    
    # Create encrypted collection
    client.create_collection(
        name=collection,
        dimension=4,
        metric="l2",
        encryption_key="my-secret-vault-key",
        noise_sigma=0.02
    )
    print(f"Collection '{collection}' created with ZK-privacy.")
    
    # 2. Insert some data
    original_vector = [0.1, 0.2, 0.3, 0.4]
    original_doc = "CLASSIFIED: Operation Hyperspace is a go."
    original_metadata = {"status": "active", "clearance": "top-secret"}
    
    success = client.insert(
        id=101,
        vector=original_vector,
        document=original_doc,
        metadata=original_metadata,
        collection=collection
    )
    print(f"Insertion success: {success}")
    
    # 3. Perform a Search via Client SDK (Transparent Decryption)
    print("\n--- Search via Client SDK (Authorized) ---")
    results = client.search(
        vector=original_vector,
        filter={"status": "active"},
        collection=collection
    )
    for r in results:
        print(f"Found Match - ID: {r['id']}, Distance: {r['distance']:.6f}")
        print(f"Decrypted Payload: '{r['payload']}'")
        print(f"Returned Hashed Metadata: {r['metadata']}")
        
    # 4. Attempt to Eavesdrop / Attack (Simulation of Malicious Miner / No-LUKS compromised disk)
    # We bypass the SDK wrapper and call the raw gRPC Search directly
    print("\n--- Attack Simulation: Eavesdropping raw gRPC (Unauthorized) ---")
    channel = grpc.insecure_channel('localhost:50051')
    raw_stub = hyperspace_pb2_grpc.DatabaseStub(channel)
    
    # Call raw Search request with include_payload=True
    search_req = hyperspace_pb2.SearchRequest(
        vector=original_vector, # raw vector
        top_k=1,
        collection=collection,
        include_payload=True
    )
    search_resp = raw_stub.Search(search_req, metadata=(('x-api-key', 'I_LOVE_HYPERSPACEDB'),))
    
    if not search_resp.results:
        print("No search results found on server!")
        return
        
    raw_result = search_resp.results[0]
    print(f"Raw Search Result ID: {raw_result.id}")
    print(f"Raw Payload (bytes) on Server: {raw_result.payload}")
    
    # Try to decode or read it
    try:
        decoded_payload = raw_result.payload.decode('utf-8')
        print(f"Raw Payload Decoded (plaintext leaking?): '{decoded_payload}'")
    except UnicodeDecodeError:
        print("Raw Payload cannot be decoded as UTF-8 (Encrypted bytes - OK!)")
        
    print(f"Raw Metadata on Server: {dict(raw_result.metadata)}")
    
    # Verify that metadata keys/values are completely hashed
    has_leak = False
    for k, v in raw_result.metadata.items():
        if k == "status" or v == "active" or k == "clearance" or v == "top-secret":
            has_leak = True
            
    if has_leak:
        print("WARNING: Plaintext metadata leaked to the server!")
    else:
        print("SUCCESS: Metadata keys and values are fully obfuscated!")
        
    # Clean up
    client.delete_collection(collection)
    client.close()

if __name__ == "__main__":
    main()
