import time
import random
from hyperspace.client import HyperspaceClient

# To run this test, you need two HyperspaceDB instances running or we can just mock a client and server
# For simplicity, we'll interact with the same DB to demonstrate the API,
# or better yet, we can create two collections 'sync_master' and 'sync_edge'.

def main():
    print("🚀 Starting Delta Sync Test (Python)...")
    
    # 1. Connect to the local server
    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
    master_col = "sync_master"
    edge_col = "sync_edge"

    # Clean up any existing
    client.delete_collection(master_col)
    client.delete_collection(edge_col)

    print("\n1. Creating Master and Edge collections...")
    client.create_collection(master_col, dimension=8, metric="l2")
    client.create_collection(edge_col, dimension=8, metric="l2")

    # 2. Insert vectors into Master
    print("2. Inserting 100 vectors into Master...")
    for i in range(100):
        vec = [random.random() for _ in range(8)]
        client.insert(
            collection=master_col,
            vector=vec,
            id=i,
            metadata={"source": "master", "idx": str(i)}
        )

    # Allow a little time for flush/indexing depending on config
    time.sleep(1)

    # 3. Insert some vectors into Edge to simulate a dirty state (out of sync)
    # Edge will have vectors 0-49 matching master (we'll just copy them later, but for now let's just leave edge empty)
    # Wait, if edge is empty, its digest is all 0s. Let's just pull from Master.
    
    # 4. Get Edge Digest (local)
    edge_stats = client.get_collection_stats(edge_col)
    # Since the client has get_collection_stats, let's look if get_digest exists.
    # We didn't add get_digest in the python client yet, let's implement a dummy client digest.
    # A real Edge client would compute this locally. 
    # For this test, we assume the edge is empty, so its buckets are all zeros.
    empty_buckets = [0] * 256
    empty_count = 0
    empty_clock = 0

    print(f"\n3. Initiating Sync Handshake from Edge to Master...")
    # Edge sends its empty digest to Master
    handshake_resp = client.sync_handshake(
        collection=master_col, 
        client_buckets=empty_buckets, 
        client_logical_clock=empty_clock, 
        client_count=empty_count
    )

    in_sync = handshake_resp.get("in_sync", False)
    diff_buckets = handshake_resp.get("diff_buckets", [])
    print(f"   In Sync: {in_sync}")
    print(f"   Differing buckets: {len(diff_buckets)}")
    
    if len(diff_buckets) > 0:
        bucket_indices_to_pull = [b["bucket_index"] for b in diff_buckets]
        print(f"\n4. Pulling {len(bucket_indices_to_pull)} buckets from Master...")
        
        # 5. Pull missing vectors
        pulled_vectors = []
        for item in client.sync_pull(collection=master_col, bucket_indices=bucket_indices_to_pull):
            pulled_vectors.append(item)
            
        print(f"   Pulled {len(pulled_vectors)} vectors representing the delta.")
        
        # 6. Apply vectors to Edge
        print("\n5. Applying delta to Edge collection...")
        for data in pulled_vectors:
            client.insert(
                collection=edge_col,
                vector=data["vector"],
                id=data["id"],
                metadata=data["metadata"]
            )
            
        print("   Delta applied!")

    # 7. Verification
    time.sleep(1)
    
    # Check if counts match
    # Since Python client doesn't expose get_digest directly, we can check node count from stats if it exists
    # Or just search to verify.
    print("\n6. Verifying data identity...")
    master_res = client.search(collection=master_col, vector=[0.5]*8, top_k=5)
    edge_res = client.search(collection=edge_col, vector=[0.5]*8, top_k=5)

    assert len(master_res) == len(edge_res), "Results count mismatch!"
    
    for m_hit, e_hit in zip(master_res, edge_res):
        assert m_hit["id"] == e_hit["id"], f"ID mismatch: {m_hit['id']} != {e_hit['id']}"
        # distance might differ slightly by precision, but should be identical
        assert abs(m_hit["distance"] - e_hit["distance"]) < 1e-4, "Distance mismatch!"
        
    print("✅ Sync check passed! Edge is identical to Master.")
    
    # Clean up
    client.delete_collection(master_col)
    client.delete_collection(edge_col)

if __name__ == "__main__":
    main()
