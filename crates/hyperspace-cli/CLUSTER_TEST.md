# Cluster Simulation Test

This test validates the full Leader-Follower replication flow with data drift detection and recovery.

## What it tests:
1. **Cluster Bootstrap**: Spawns 1 Leader + 2 Followers
2. **Initial Sync**: Inserts 100 vectors to Leader, verifies Followers receive them
3. **Digest Verification**: Uses Bucket Merkle Tree to verify data consistency
4. **Fault Tolerance**: Kills Follower 2, inserts more data, restarts Follower 2
5. **Recovery**: Verifies Follower 2 catches up after restart

## Running the test:
```bash
cargo run --release --bin cluster_test
```

## Expected Output:
```
🏗️  Building server...
🧪 Starting Cluster Test (Leader + 2 Followers)
✅ Leader started on :50051
✅ Followers started
✅ Collection created on Leader
Please wait, inserting 100 vectors...
✅ Insertion complete
Leader Hash: 12345678
F1 Hash:     12345678
💀 Killing Follower 2...
Inserting 50 more vectors...
♻️  Restarting Follower 2...
Leader Hash (new): 87654321
F2 Hash (restored): 87654321
🎉 CLUSTER TEST PASSED! GOLD MASTER READY.
```

## Architecture:
- Each node runs in a separate temp directory (`tmp_data_<port>`)
- Nodes communicate via gRPC (replication stream)
- Digest API is used to verify sync status
- Test uses 256-bucket Merkle Tree for efficient diffing

## Cleanup:
Temporary directories are automatically cleaned on test start.
To manually clean: `rm -rf tmp_data_*`
