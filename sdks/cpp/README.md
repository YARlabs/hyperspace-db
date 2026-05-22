# HyperspaceDB C++ SDK

Welcome to the HyperspaceDB C++ SDK. This SDK provides gRPC generated clients and Cognitive Math tooling for integrating HyperspaceDB—the ultra-fast hyperbolic vector database—into your C++ applications (such as ROS2 Robotics, High-Frequency Trading, and Spatial AI engines).

## Features

- **Blazing Fast gRPC Client**: Direct byte-level access to HyperspaceDB (`Insert`, `BatchInsert`, `Search`, `SearchBatch`).
- **Advanced Data Ops**: Bulk point retrieval (`GetPoints`), metadata patching (`UpdatePayload`), paginated scanning (`Scroll`), and filtered counting (`Count`).
- **Recursive Filtering**: Full support for complex logical expressions (`AND`, `OR`, `NOT`) in filters.
- **Graph Traversal APIs**: Uncover semantic structures using `GetNode`, `GetNeighbors`, `GetSubsumptionTree`, and `Traverse`.
- **Memory Reconsolidation**: Trigger `TriggerReconsolidation` to start the AI Sleep Mode flow matching optimization directly within the database.
- **System Health**: Active server monitoring via `HealthCheck`.
- **Cross-Feature Metric**: Search geometries using standard metrics as well as the newly integrated $1D$ O(N) Wasserstein CDF distance (`use_wasserstein = true`).
- **Differential Sync**: Delta-sync methods for synchronizing decentralized edge databases.

## Generated Protobufs

All protobufs (`hyperspace.pb.h` / `hyperspace.grpc.pb.h`) are pre-generated and located in the `proto/` directory. They sync perfectly with HyperspaceDB server `v3.1.0`.

## Using the SDK

Build using CMake and link against the provided `grpc++` and `protobuf` libraries.

Example usage:

```cpp
#include "proto/hyperspace.grpc.pb.h"
#include <grpcpp/grpcpp.h>

// 1. Create Collection with Schema
hyperspace::CreateCollectionRequest create_req;
create_req.set_name("docs");
auto* schema = create_req.mutable_schema();
auto* primary = schema->add_components();
primary->set_name("primary");
primary->set_metric("cosine");
primary->set_full_dimension(1024);
primary->set_weight(1.0);

hyperspace::StatusResponse create_res;
grpc::ClientContext create_ctx;
create_ctx.AddMetadata("x-api-key", "I_LOVE_HYPERSPACEDB");
status = stub->CreateCollection(&create_ctx, create_req, &create_res);

// 2. Search
hyperspace::SearchRequest request;
request.set_collection("docs");
request.add_vector(0.12);
request.add_vector(-0.45);
request.set_top_k(10);

hyperspace::SearchResponse response;
grpc::ClientContext context;
context.AddMetadata("x-api-key", "I_LOVE_HYPERSPACEDB");

grpc::Status status = stub->Search(&context, request, &response);

// 2. Geometric Filters (New in v3.0)
hyperspace::SearchRequest geo_req;
geo_req.set_collection("docs");

// 2a. Ball Filter
auto* f = geo_req.add_filters();
auto* ball = f->mutable_in_ball();
ball->add_center(0.1);
ball->add_center(0.2);
ball->set_radius(0.5);

// 2b. Box Filter
auto* f2 = geo_req.add_filters();
auto* box = f2->mutable_in_box();
box->add_min_bounds(-1.0);
box->add_max_bounds(1.0);

// 3. Insert Text (Server-Side Embedding)
hyperspace::InsertTextRequest insert_req;
insert_req.set_collection("docs");
insert_req.set_id(1);
insert_req.set_text("HyperspaceDB is fast!");
hyperspace::InsertResponse insert_res;
grpc::ClientContext insert_ctx;
insert_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->InsertText(&insert_ctx, insert_req, &insert_res);

// 4. Vectorize Text
hyperspace::VectorizeRequest vec_req;
vec_req.set_text("Hello!");
vec_req.set_metric("cosine");
hyperspace::VectorizeResponse vec_res;
grpc::ClientContext vec_ctx;
vec_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->Vectorize(&vec_ctx, vec_req, &vec_res);
// vec_res.vector() -> repeated double

// 5. Search Text
hyperspace::SearchTextRequest search_text_req;
search_text_req.set_collection("docs");
search_text_req.set_text("Is HyperspaceDB fast?");
search_text_req.set_top_k(5);
hyperspace::SearchResponse search_text_res;
grpc::ClientContext search_text_ctx;
search_text_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->SearchText(&search_text_ctx, search_text_req, &search_text_res);

status = stub->Search(&hybrid_ctx, hybrid_req, &hybrid_res);

// 8. Matryoshka Representation Learning (MRL) & Cascading
hyperspace::CreateCollectionRequest mrl_req;
mrl_req.set_name("mrl_docs");
auto* mrl_schema = mrl_req.mutable_schema();
auto* comp = mrl_schema->add_components();
comp->set_name("main");
comp->set_metric("lorentz");
comp->set_full_dimension(1025);

auto* mrl_layer = mrl_schema->add_cascade_pipeline();
mrl_layer->set_component_name("main");
mrl_layer->set_cutoff_dimension(129); // 128D (+1) fast search
mrl_layer->set_store_in_ram(true);
mrl_layer->set_rerank_top_k(100);

grpc::ClientContext mrl_ctx;
mrl_ctx.AddMetadata("x-api-key", "I_LOVE_HYPERSPACEDB");
status = stub->CreateCollection(&mrl_ctx, mrl_req, &create_res);

// 6. List Collections with Metadata
hyperspace::Empty list_req;
hyperspace::ListCollectionsResponse list_res;
grpc::ClientContext list_ctx;
list_ctx.AddMetadata("authorization", "Bearer I_LOVE_HYPERSPACEDB");
status = stub->ListCollections(&list_ctx, list_req, &list_res);
for (const auto& col : list_res.collections()) {
    std::cout << "Collection: " << col.name() 
              << ", Count: " << col.count() 
              << ", Schema: " << col.schema().DebugString() << std::endl;
}
```

## Hyperbolic & Cognitive Math SDK (Spatial AI Engine)

The C++ SDK includes built-in offline mathematical utilities for spatial-AI metrics and Agentic AI workflow management in Hyperbolic space via the header-only `<hyperspace/math.hpp>` library:

```cpp
#include "hyperspace/math.hpp"

// 1. Poincaré Ball & Hyperbolic Math
auto z = hyperspace::math::mobius_add(x, y, 1.0);
auto v = hyperspace::math::exp_map(x, tangent_vec, 1.0);
auto mu = hyperspace::math::frechet_mean(points, 1.0, 32, 1e-6);

// 2. Hallucination Detection (Entropy approaches 1.0)
double entropy = hyperspace::math::local_entropy(candidate_thought, neighbors, 1.0);
if (entropy > 0.8) {
    std::cout << "Potential hallucination detected!" << std::endl;
}

// 3. Proof of Convergence (Lyapunov energy derivative < 0 implies stability)
double stability = hyperspace::math::lyapunov_convergence(trajectory, 1.0);

// 4. Predict Trajectory Momentum (Koopman linearization)
auto next_thought = hyperspace::math::koopman_extrapolate(past, current, 1.0, 1.0);

// 5. Phase-Locked Loop for topic tracking
auto synced_thought = hyperspace::math::context_resonance(thought, global_context, 0.5, 1.0);
```

## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** configured via environment variables on the server side. Each geometry (`l2`, `cosine`, `poincare`, `lorentz`, `hybrid`) can use its own backend.

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
| `hybrid` | None | Lorentz + L2 combined metric |

> **Note:** For `lorentz` geometry, dimension = spatial_dim + 1 (e.g. 129 for 128-dim spatial vectors).
