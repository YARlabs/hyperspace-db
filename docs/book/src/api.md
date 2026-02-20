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
  string metric = 3;    // "l2", "euclidean", "cosine", "poincare", "lorentz"
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
  map<string, MetadataValue> typed_metadata = 8; // Typed metadata (int/float/bool/string)
}

enum DurabilityLevel {
  DEFAULT_LEVEL = 0; // Use server config
  ASYNC = 1;         // Flush OS cache (Fastest)
  BATCH = 2;         // Background fsync (Balanced)
  STRICT = 3;        // Fsync every write (High Safety)
}

```

`typed_metadata` is the preferred metadata path for new clients. Legacy `metadata` remains for compatibility.

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

`SearchResult` now includes both `metadata` and `typed_metadata`.
Range filters are evaluated with numeric semantics (`f64`) against typed metadata numeric values.
For gRPC clients, decimal thresholds are supported via `Range.gte_f64` / `Range.lte_f64` (legacy `gte/lte` `int64` remains supported).

gRPC `Range` examples:

```protobuf
// Integer threshold (legacy-compatible)
Filter {
  range: {
    key: "depth",
    gte: 2,
    lte: 10
  }
}

// Decimal threshold (recommended for typed numeric metadata)
Filter {
  range: {
    key: "energy",
    gte_f64: 0.8,
    lte_f64: 1.0
  }
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

#### `SubscribeToEvents`
Streams CDC events for post-insert/delete hooks.

```protobuf
rpc SubscribeToEvents (EventSubscriptionRequest) returns (stream EventMessage);

enum EventType {
  EVENT_UNKNOWN = 0;
  VECTOR_INSERTED = 1;
  VECTOR_DELETED = 2;
}

message EventSubscriptionRequest {
  repeated EventType types = 1;
  optional string collection = 2;
}

message EventMessage {
  EventType type = 1;
  oneof payload {
    VectorInsertedEvent vector_inserted = 2;
    VectorDeletedEvent vector_deleted = 3;
  }
}
```

Use this stream to build external pipelines (audit, Elasticsearch sync, graph projections, Neo4j updaters).
SDKs (Python/TypeScript/Rust) expose convenience subscription methods for this stream.

Reliability note:
- stream consumers may lag under burst load; server now handles lagged broadcast reads without dropping the whole stream task;
- tune `HS_EVENT_STREAM_BUFFER` for higher event fan-out pressure.

#### `MetadataValue` (Typed Metadata)
```protobuf
message MetadataValue {
  oneof kind {
    string string_value = 1;
    int64 int_value = 2;
    double double_value = 3;
    bool bool_value = 4;
  }
}
```

#### `Graph Traversal API` (v2.3)
```protobuf
rpc GetNode (GetNodeRequest) returns (GraphNode);
rpc GetNeighbors (GetNeighborsRequest) returns (GetNeighborsResponse);
rpc GetConceptParents (GetConceptParentsRequest) returns (GetConceptParentsResponse);
rpc Traverse (TraverseRequest) returns (TraverseResponse);
rpc FindSemanticClusters (FindSemanticClustersRequest) returns (FindSemanticClustersResponse);
```

Key safety guards:
- `GetNeighborsRequest.limit` and `offset` for bounded pagination.
- `TraverseRequest.max_depth` and `max_nodes` to prevent unbounded graph walks.
- `FindSemanticClustersRequest.max_clusters` and `max_nodes` for bounded connected-component scans.

`TraverseRequest` is filter-aware and supports both:
- `filter` (`map<string,string>`)
- `filters` (`Match` / `Range`)

`GetNeighborsResponse` now includes `edge_weights`, where `edge_weights[i]` is the distance from source node to `neighbors[i]`.

#### `RebuildIndex` with pruning filter (v2.2.1)
```protobuf
message RebuildIndexRequest {
  string name = 1;
  optional VacuumFilterQuery filter_query = 2;
}

message VacuumFilterQuery {
  string key = 1;
  string op = 2; // "lt" | "lte" | "gt" | "gte" | "eq" | "ne"
  double value = 3;
}
```

Use this API for sleep/reconsolidation cycles when you need to rebuild an index and prune low-value vectors in one server-side operation.

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

### Graph HTTP Endpoints (Dashboard / tooling)

- `GET /api/collections/{name}/graph/node?id={id}&layer={layer}`
- `GET /api/collections/{name}/graph/neighbors?id={id}&layer={layer}&limit={limit}&offset={offset}`
- `GET /api/collections/{name}/graph/parents?id={id}&layer={layer}&limit={limit}`
- `POST /api/collections/{name}/graph/traverse`
- `POST /api/collections/{name}/graph/clusters`
