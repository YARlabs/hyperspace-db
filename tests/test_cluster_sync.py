#!/usr/bin/env python3
"""Quick test to insert data and verify replication"""
import grpc
from hyperspace import hyperspace_pb2, hyperspace_pb2_grpc

def test_replication():
    # Connect to Leader
    leader_channel = grpc.insecure_channel('localhost:50051')
    leader_stub = hyperspace_pb2_grpc.DatabaseStub(leader_channel)
    
    # Insert data
    metadata = [('x-api-key', 'I_LOVE_HYPERSPACEDB')]
    request = hyperspace_pb2.InsertRequest(
        collection='default',
        id=999,
        vector=[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
        metadata={'test': 'cluster_demo'}
    )
    
    response = leader_stub.Insert(request, metadata=metadata)
    print(f"✅ Inserted to Leader: {response.success}")
    
    # Query Leader digest
    digest_req = hyperspace_pb2.DigestRequest(collection='default')
    leader_digest = leader_stub.GetDigest(digest_req, metadata=metadata)
    print(f"📊 Leader Digest: clock={leader_digest.logical_clock}, hash={leader_digest.state_hash}, count={leader_digest.count}")
    
    # Wait a bit for replication
    import time
    time.sleep(1)
    
    # Query Follower digest
    follower_channel = grpc.insecure_channel('localhost:50052')
    follower_stub = hyperspace_pb2_grpc.DatabaseStub(follower_channel)
    follower_digest = follower_stub.GetDigest(digest_req, metadata=metadata)
    print(f"📊 Follower Digest: clock={follower_digest.logical_clock}, hash={follower_digest.state_hash}, count={follower_digest.count}")
    
    # Verify sync
    if leader_digest.state_hash == follower_digest.state_hash:
        print("🎉 SUCCESS! Leader and Follower are in sync!")
    else:
        print("❌ MISMATCH! Data drift detected.")
        print(f"   Leader hash: {leader_digest.state_hash}")
        print(f"   Follower hash: {follower_digest.state_hash}")

if __name__ == '__main__':
    test_replication()
