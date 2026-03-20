# HyperspaceDB C++ SDK

Welcome to the HyperspaceDB C++ SDK. This SDK provides gRPC generated clients and Cognitive Math tooling for integrating HyperspaceDB—the ultra-fast hyperbolic vector database—into your C++ applications (such as ROS2 Robotics, High-Frequency Trading, and Spatial AI engines).

## Features

- **Blazing Fast gRPC Client**: Direct byte-level access to HyperspaceDB (`Insert`, `BatchInsert`, `Search`, `SearchBatch`).
- **Graph Traversal APIs**: Uncover semantic structures using `GetNode`, `GetNeighbors`, and `Traverse`.
- **Memory Reconsolidation**: Trigger `TriggerReconsolidation` to start the AI Sleep Mode flow matching optimization directly within the database.
- **Cross-Feature Metric**: Search geometries using standard metrics as well as the newly integrated $1D$ O(N) Wasserstein CDF distance (`use_wasserstein = true`).
- **Differential Sync**: Delta-sync methods for synchronizing decentralized edge databases.

## Generated Protobufs

All protobufs (`hyperspace.pb.h` / `hyperspace.grpc.pb.h`) are pre-generated and located in the `proto/` directory. They sync perfectly with HyperspaceDB server `v3.0.0-alpha.2`.

## Using the SDK

Build using CMake and link against the provided `grpc++` and `protobuf` libraries.

Example usage:

```cpp
#include "proto/hyperspace.grpc.pb.h"
#include <grpcpp/grpcpp.h>

// Connect and Search
auto channel = grpc::CreateChannel("localhost:50051", grpc::InsecureChannelCredentials());
auto stub = hyperspace::Database::NewStub(channel);

hyperspace::SearchRequest request;
request.set_collection("robots_memory");
request.add_vector(0.12);
request.add_vector(-0.45);
request.set_top_k(10);
request.set_use_wasserstein(false); // Enable for 1D CFM (Wasserstein Metric)

hyperspace::SearchResponse response;
grpc::ClientContext context;
context.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");

grpc::Status status = stub->Search(&context, request, &response);

// 2. Insert Text (Server-Side Embedding)
hyperspace::InsertTextRequest insert_req;
insert_req.set_collection("docs");
insert_req.set_id(1);
insert_req.set_text("HyperspaceDB is fast!");
hyperspace::InsertResponse insert_res;
grpc::ClientContext insert_ctx;
insert_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->InsertText(&insert_ctx, insert_req, &insert_res);

// 3. Vectorize Text
hyperspace::VectorizeRequest vec_req;
vec_req.set_text("Hello!");
vec_req.set_metric("cosine");
hyperspace::VectorizeResponse vec_res;
grpc::ClientContext vec_ctx;
vec_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->Vectorize(&vec_ctx, vec_req, &vec_res);
// vec_res.vector() -> repeated double

// 4. Search Text
hyperspace::SearchTextRequest search_text_req;
search_text_req.set_collection("docs");
search_text_req.set_text("Is HyperspaceDB fast?");
search_text_req.set_top_k(5);
hyperspace::SearchResponse search_text_res;
grpc::ClientContext search_text_ctx;
search_text_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->SearchText(&search_text_ctx, search_text_req, &search_text_res);
```
## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** configured via environment variables on the server side. Each geometry (`l2`, `cosine`, `poincare`, `lorentz`) can use its own backend.

### Server Configuration (`.env`)

```env
HYPERSPACE_EMBED=true

# Cosine via OpenAI
HS_EMBED_COSINE_PROVIDER=openai
HS_EMBED_COSINE_EMBED_MODEL=text-embedding-3-small
HS_EMBED_COSINE_API_KEY=sk-...

# Lorentz via HuggingFace Hub (downloads model.onnx + tokenizer.json)
HS_EMBED_LORENTZ_PROVIDER=huggingface
HS_EMBED_LORENTZ_HF_MODEL_ID=your-org/cde-spatial-lorentz-128d
HS_EMBED_LORENTZ_DIM=129
HF_TOKEN=hf_...  # Optional — required for gated/private models

# Poincaré via local ONNX
HS_EMBED_POINCARE_PROVIDER=local
HS_EMBED_POINCARE_MODEL_PATH=./models/poincare_128d.onnx
HS_EMBED_POINCARE_TOKENIZER_PATH=./models/poincare_128d_tokenizer.json
HS_EMBED_POINCARE_DIM=128
```

### Multi-Geometry Search from C++

```cpp
hyperspace::SearchMultiCollectionRequest req;
req.add_vector(0.12);
req.add_vector(-0.45);
req.set_top_k(10);
req.add_collections("robots_l2");
req.add_collections("robots_cosine");
req.add_collections("robots_lorentz");

hyperspace::SearchMultiCollectionResponse response;
grpc::ClientContext context;
context.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
grpc::Status status = stub->SearchMultiCollection(&context, req, &response);
// response.results_by_collection() → map<string, SearchResultList>
```

### Supported Geometries

| Geometry | Post-Processing | Best For |
|---|---|---|
| `cosine` | Unit normalize | Semantic similarity |
| `l2` | Unit normalize | Euclidean distance / robotics |
| `poincare` | Clamp to unit ball | Hierarchical data (knowledge graphs) |
| `lorentz` | None | Mixed hierarchical + semantic (spatial AI) |

> **Note:** For `lorentz` geometry, dimension = spatial_dim + 1 (e.g. 129 for 128-dim spatial vectors).
