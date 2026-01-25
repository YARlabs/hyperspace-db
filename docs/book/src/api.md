# gRPC API Reference

HyperspaceDB exposes a simple yet powerful gRPC interface defined in `hyperspace.proto`.

## Service: `Database`

### `Insert`
Ingests a vector into the database. This is an asynchronous operation regarding the index, but synchronous regarding durability (WAL).

```protobuf
rpc Insert (InsertRequest) returns (InsertResponse);

message InsertRequest {
  uint32 id = 1;             // External unique ID
  repeated float vector = 2; // The data point (must be norm < 1)
  map<string, string> metadata = 3; // Tags for filtering
}
```

### `Search`

Finds the nearest neighbors using the Poincaré metric.

```protobuf
rpc Search (SearchRequest) returns (SearchResponse);

message SearchRequest {
  repeated float vector = 1; // Query vector
  uint32 top_k = 2;          // Number of results
  map<string, string> filter = 3; // Optional metadata filter (AND logic)
}
```

### `Configure`

Updates runtime parameters.

```protobuf
rpc Configure (ConfigUpdate) returns (StatusResponse);

message ConfigUpdate {
  optional uint32 ef_search = 1;       // Search beam width (higher = more accurate)
  optional uint32 ef_construction = 2; // Indexing beam width (higher = better graph)
}
```
