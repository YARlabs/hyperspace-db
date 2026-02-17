# API Reference

HyperspaceDB operates on a **Dual-API** architecture:
1. **gRPC (Data Plane)**: High-performance ingestion and search.
2. **HTTP (Control Plane)**: Management, monitoring, and dashboard integration.

## 📡 gRPC API (Data Plane)

Defined in `hyperspace.proto`. Used by SDKs (Python, Rust, Go).

### Collection Management

#### `CreateCollection`
Creates a new independent vector index.

```protobuf
rpc CreateCollection (CreateCollectionRequest) returns (StatusResponse);

message CreateCollectionRequest {
  string name = 1;
  uint32 dimension = 2; // e.g. 1536, 1024, 64
  string metric = 3;    // "l2", "cosine", "poincare"
}
```

#### `DeleteCollection`
Drops a collection and all its data.

```protobuf
rpc DeleteCollection (DeleteCollectionRequest) returns (StatusResponse);
```

### Vector Operations

#### `Insert`
Ingests a vector into a specific collection.

```protobuf
rpc Insert (InsertRequest) returns (InsertResponse);

message InsertRequest {
  string collection = 1;      // Collection name
  repeated double vector = 2; // Data point
  uint32 id = 3;              // External ID
  map<string, string> metadata = 4; // Metadata tags
  DurabilityLevel durability = 7; // Durability override
}

enum DurabilityLevel {
  DEFAULT_LEVEL = 0; // Use server config
  ASYNC = 1;         // Flush OS cache (Fastest)
  BATCH = 2;         // Background fsync (Balanced)
  STRICT = 3;        // Fsync every write (High Safety)
}

```

#### `Search`
Finds nearest neighbors.

```protobuf
rpc Search (SearchRequest) returns (SearchResponse);

message SearchRequest {
  string collection = 1;
  repeated double vector = 2;
  uint32 top_k = 3;
  // Metadata string filter (e.g. "category:book")
  map<string, string> filter = 4;
  // Complex filter object
  repeated Filter filters = 5;
  // Hybrid search
  optional string hybrid_query = 6;
  optional float hybrid_alpha = 7;
}
```

#### `SearchBatch`
Finds nearest neighbors for multiple queries in a single RPC call.

```protobuf
rpc SearchBatch (BatchSearchRequest) returns (BatchSearchResponse);

message BatchSearchRequest {
  repeated SearchRequest searches = 1;
}

message BatchSearchResponse {
  repeated SearchResponse responses = 1;
}
```

Recommended for high-concurrency clients and benchmarks to reduce per-request gRPC overhead.

---

## 🌐 HTTP API (Control Plane)

Served on port `50050` (default). All endpoints under `/api`.

### Authentication & Multi-Tenancy

Every request should include:
- `x-api-key`: API Key (optional if disabled, but recommended)
- `x-hyperspace-user-id`: Tenant Identifier (e.g. `client_123`). If omitted, defaults to `default_admin`.

### Cluster Status
`GET /api/cluster/status`

Returns the node's identity and topology role.

```json
{
  "node_id": "uuid...",
  "role": "Leader", // or "Follower"
  "upstream_peer": null,
  "downstream_peers": []
}
```

### Node Status (Compatibility)
`GET /api/status`

Returns runtime status and node configuration. Dashboard uses this endpoint first, with fallback to `/api/cluster/status`.

### System Metrics
`GET /api/metrics`

Real-time system resource usage.

```json
{
    "cpu_usage_percent": 12,
    "ram_usage_mb": 512,
    "disk_usage_mb": 1024,
    "total_collections": 5,
    "total_vectors": 1000000
}
```

### Admin / Billing (Since v2.0)

**Requires `user_id: admin`**

`GET /api/admin/usage`

Returns JSON map of `user_id -> usage_stats`:

```json
{
  "tenant_A": {
    "collection_count": 2,
    "vector_count": 1500,
    "disk_usage_bytes": 1048576
  }
}
```

### List Collections
`GET /api/collections`

Returns summary of all active collections.

```json
[
  {
    "name": "my_docs",
    "count": 1500,
    "dimension": 1536,
    "metric": "l2"
  }
]
```

### Collection Search (HTTP Playground)
`POST /api/collections/{name}/search`

Convenience endpoint for dashboard/manual testing.

```json
{
  "vector": [0.1, 0.2, 0.3],
  "top_k": 5
}
```
