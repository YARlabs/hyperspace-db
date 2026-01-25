
# 🤝 Distributed Replication

To scale **Read Throughput** and ensure **High Availability**, HyperspaceDB supports a Leader-Follower architecture.

## Architecture

* **Leader**:
  * Accepts Writes (Insert/Delete).
  * Accepts Reads.
  * Streams WAL (Write-Ahead Log) entries to connected followers via gRPC streaming.
* **Follower**:
  * Replicates state from Leader in real-time.
  * Accepts Reads (Scaling).
  * Rejects Writes.

## Configuration

### Leader

Simply start the server (default role is leader).

```bash
./hyperspace-server --port 50051
```

### Follower

Start with `--role follower` and point to the leader's URL.

```bash
./hyperspace-server --port 50052 --role follower --leader http://127.0.0.1:50051
```

> **Note**: Followers store data in memory and can optionally persist snapshots. Currently, followers replicate the in-memory state.
