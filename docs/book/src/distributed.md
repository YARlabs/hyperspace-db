# 🤝 Federated Clustering (v1.2)

HyperspaceDB v1.2 introduces a **Federated Leader-Follower** architecture. This goes beyond simple read-replication, introducing `Node Identity`, `Logical Clocks`, and `Topology Awareness` to support future Edge-Cloud synchronization scenarios.

## Concepts

### Node Identity
Every node in the cluster is assigned a persistent, unique UUID (`node_id`) upon first startup. This ID is used to track the origin of write operations in the replication log.

### Roles
* **Leader** (Coordinator):
  * Accepts Writes (`Insert`, `Delete`, `CreateCollection`).
  * Manages the Cluster Topology.
  * Streams WAL events to connected Followers.
* **Follower** (Replica):
  * Read-Only.
  * Replicates state from the Leader in real-time.
  * Can be promoted to Leader if needed.
* **Edge Node** (Planned v1.4):
  * Offline-first node that accumulates writes and syncs via Merkle Trees when online.

## Configuration

### Leader
Simply start the server. By default, it assumes the **Leader** role.

```bash
./hyperspace-server --port 50051
```

### Follower
Start with `--role follower` and point to the leader's URL.

```bash
./hyperspace-server --port 50052 --role follower --leader http://127.0.0.1:50051
```

## Monitoring Topology

You can inspect the cluster state via the HTTP API on the Dashboard port (default `50050`).

**Request:**
```bash
curl http://localhost:50050/api/cluster/status
```

**Response:**
```json
{
  "node_id": "e8b37fde-6c60-427f-8a09-47103c2da80e",
  "role": "Leader",
  "upstream_peer": null,
  "downstream_peers": [],
  "logical_clock": 1234
}
```

This JSON response tells you:
- The node's unique ID.
- Its current role.
- Who it is following (if Follower).
- Who is following it (if Leader).
- The current logical timestamp of its database state.
