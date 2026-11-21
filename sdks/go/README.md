# HyperspaceDB Go SDK

Official Go client for the **HyperspaceDB** gRPC API. 

This SDK features pre-generated protocol buffers (`proto/`) to interact directly with the high-performance HyperspaceDB server (`v4.1.0`). It is tailored for high-concurrency event-streaming systems, CDC syncs, and microservices managing hyperspatial graph databases.

## Integration

To install and use our package, retrieve it via Go Modules:

```bash
go get github.com/yarlabs/hyperspace-sdk-go
```

## Features

- **Vector Searching**: High-performance Batched searches spanning Poincare, Lorentz, Cosine, L2, and Wasserstein cross-feature metrics (`SearchRequest.UseWasserstein`).
- **Data Operations**: Bulk point retrieval (`GetPoints`), metadata patching (`UpdatePayload`), paginated scanning (`Scroll`), and filtered counting (`Count`).
- **Recursive Filtering**: Complex logical expressions (`AND`, `OR`, `NOT`) in filters.
- **Memory Reconsolidation (Sleep Mode)**: Optimize datasets using `TriggerReconsolidation` directly inside your database from microservices.
- **System Health**: Active server monitoring via `HealthCheck`.
- **Tribunal Anti-Hallucination Verification**: Post-generation mathematical verification (`VerifyGeneration`) ensuring LLM answers respect source context geometric boundaries.
- **CDC Streaming**: React to structural topology changes in real time through `SubscribeToEvents`/Event Streams.
- **Graph Traversal APIs**: Access direct paths via `GetNeighbors` and perform node clustering.
- **Implicit Graph Engine**: Extract light-cone hierarchies with `GetSubsumptionTree` and navigate using advanced `Traverse` modes (Greedy, Diffusive, Momentum).


## Usage Example

```go
package main

import (
	"context"
	"log"

	"github.com/yarlabs/hyperspace-sdk-go"
)

func main() {
	client, err := hyperspace.NewHyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB")
	if err != nil {
		log.Fatalf("failed to connect: %v", err)
	}
	defer client.Close()

	ctx := context.Background()
	collection := "docs_go"

	// Create collection with Schema and Quantization ("none", "medium", "medium_plus", "turbo", "ternary", "extreme", "rabitq", "rabitq4", "rabitq2", "pq4", "pq", "opq")
	_ = client.CreateCollectionWithQuantization(ctx, collection, &pb.CollectionSchema{
		Components: []*pb.VectorComponent{
			{Name: "primary", Metric: "cosine", FullDimension: 1024, Weight: 1.0},
		},
	}, "", 0.02, "ternary")

	// Insert text (server-side vectorization)
	err = client.InsertText(ctx, 1, "HyperspaceDB is awesome!", collection)
	if err != nil {
		log.Fatalf("Insert failed: %v", err)
	}
    
	// Hybrid Search (Semantic + BM25 Lexical)
	results, err := client.SearchText(ctx, "What is HyperspaceDB?", 5, collection, 0.7, &pb.Bm25Options{
		Method:   "bm25plus",
		Language: "english",
	})
	if err != nil {
		log.Fatalf("Search failed: %v", err)
	}

	log.Printf("Found %d vectors via hybrid search", len(results))

	// List collections with metadata
	collections, err := client.ListCollections(ctx)
	if err == nil {
		for _, col := range collections {
			log.Printf("Collection: %s, Count: %d, Schema: %v", col.Name, col.Count, col.Schema)
		}
	}
}

```

## Quantization Levels & Compact Vectors (v4.2.5)

Pass one of these values as `quantization` when creating a collection:

| Value | Mode | Bytes / vector (1024-D) | Metrics | Needs calibration |
| :--- | :--- | ---: | :--- | :---: |
| `none` | Full precision (f32 storage) | 4,100 | all | — |
| `medium` | ScalarI8 (8-bit, int8 SDOT kernel) — **default** | 1,028 | all | — |
| `medium_plus` | ScalarI4 (4-bit block-wise) | 772 | all | — |
| `turbo` | TurboQuant 4-bit spherical | 520 | l2, cosine, hybrid | — |
| `ternary` | BitNet 1.58-bit {-1, 0, +1} | 264 | all | — |
| `extreme` | Binary 1-bit | 132 | all | — |
| `rabitq` | **RaBitQ** 1-bit + unbiased estimator (new in 4.2.5) | 136 | l2, cosine | ✔ |
| `rabitq4` | **Extended RaBitQ** 4-bit grid code + unbiased estimator, int8 SDOT kernel (new in 4.2.5) | 520 | l2, cosine | ✔ |
| `rabitq2` | **Extended RaBitQ** 2-bit grid code + unbiased estimator (new in 4.2.5) | 264 | l2, cosine | ✔ |
| `pq4` | **PQ4 fast-scan** 4-bit PQ, L1-resident LUT (new in 4.2.5) | 128 | l2, cosine | ✔ |
| `opq` / `pq` | 8-bit product quantization | 64 | l2, cosine | ✔ |

Trained modes (`rabitq`, `rabitq4`, `rabitq2`, `pq4`, `pq`, `opq`) learn their centroid / codebooks from real data. Since server 4.2.5 each shard calibrates them online once it holds 1,024 vectors (`HS_AUTO_CALIBRATE_MIN`; 4,096 for `pq` / `opq`) and re-encodes stored codes in place; **rebuild index** after a large initial ingest additionally rebuilds the graph on the trained codes. `rabitq*` / `pq4` requested on other metrics fall back to `medium`. Server 4.2.5 reranks quantized candidates against exact vectors by default, so `medium` and `rabitq4` return exact top-k; `rabitq4` does it with half the memory of `medium`.

**Large collections (server ≥ 4.2.5).** A new collection gets one shard unless you pass the expected size: `CreateCollectionWithOptions(ctx, name, schema, "", 0.02, "medium", 5_000_000)` sends `x-expected-vectors`, and the server creates one shard per `HS_SHARD_TARGET_VECTORS` (2M by default). The layout is fixed at creation; leave it unset below a few million vectors. Older servers ignore it.

**Compact wire vectors.** Since server 4.2.5 requests may carry vectors as `repeated float` (`vector_f32`) instead of `repeated double` — half the bytes on the wire and half the decode cost (the server stores f32 anyway). It is opt-in because older servers ignore the new field:

```go
client, _ := hyperspace.NewClient("localhost:50051", apiKey)
client.SetCompactVectors(true) // send vector_f32 (server >= 4.2.5)
```

**Per-request `ef_search` (server ≥ 4.2.5).** Searches use the collection's beam width (`ef_search`, default 100) unless a request sets its own; larger values raise recall at the cost of latency (capped by `HS_MAX_EF_SEARCH`, 4096):

```go
hits, err := client.Search(ctx, query, 10, "docs", &hyperspace.SearchParams{EfSearch: 256})
multi, err := client.SearchMultiCollection(ctx, []string{"a", "b"}, query, 10, hyperspace.WithEfSearch(128))
```

## Matryoshka Representation Learning (MRL) & Cascading

HyperspaceDB supports MRL through its **Cascade Pipeline**. This allows you to perform initial fast search on a truncated low-dimensional vector (e.g., 64D) and then rerank the results using the full vector (e.g., 1024D).

```go
schema := &pb.CollectionSchema{
    Components: []*pb.VectorComponent{
        {Name: "primary", Metric: "lorentz", FullDimension: 1025, Weight: 1.0},
    },
    CascadePipeline: []*pb.MrlLayer{
        {
            ComponentName:   "primary",
            CutoffDimension: 129, // Initial search on 128D (+1)
            StoreInRam:      true,
            RerankTopK:      100,
        },
    },
}
client.CreateCollection(ctx, "mrl_collection", schema)
```

## Geometric Filters

HyperspaceDB introduces advanced spatial filters that run on the engine level:

```go
// 1. Proximity Search (Ball)
ballFilter := &pb.Filter{
    Condition: &pb.Filter_InBall{
        InBall: &pb.InBall{
            Center: []float64{0.1, 0.2, 0.3},
            Radius: 0.5,
        },
    },
}

// 2. Workspace Constraints (Box)
boxFilter := &pb.Filter{
    Condition: &pb.Filter_InBox{
        InBox: &pb.InBox{
            MinBounds: []float64{-1, -1, -1},
            MaxBounds: []float64{1, 1, 1},
        },
    },
}

// 3. Field of View / Angular Search (Cone)
coneFilter := &pb.Filter{
    Condition: &pb.Filter_InCone{
        InCone: &pb.InCone{
            Axes:      []float64{1.0, 0.0, 0.0},
            Apertures: []float64{0.5},
            Cen:       0.01,
        },
    },
}

req := &pb.SearchRequest{
    Vector:  []float64{0.1, 0.2, 0.3},
    TopK:    10,
    Filters: []*pb.Filter{ballFilter, boxFilter},
}
res, err := client.Search(ctx, req)
```

## Hyperbolic & Cognitive Math SDK (Spatial AI Engine)

The Go SDK includes built-in offline mathematical utilities for spatial-AI metrics and Agentic AI workflow management in Hyperbolic space:

```go
import "github.com/yarlabs/hyperspace-sdk-go"

// 1. Poincaré Ball & Hyperbolic Math
z, err := hyperspace.MobiusAdd(x, y, 1.0)
v, err := hyperspace.ExpMap(x, tangentVec, 1.0)
mu, err := hyperspace.FrechetMean(points, 1.0, 32, 1e-6)

// 2. Hallucination Detection (Entropy approaches 1.0)
entropy, err := hyperspace.LocalEntropy(candidateThought, neighbors, 1.0)
if entropy > 0.8 {
    log.Println("Potential hallucination detected!")
}

// 3. Proof of Convergence (Lyapunov energy derivative < 0 implies stability)
stability, err := hyperspace.LyapunovConvergence(trajectory, 1.0)

// 4. Predict Trajectory Momentum (Koopman linearization)
nextThought, err := hyperspace.KoopmanExtrapolate(past, current, 1.0, 1.0)

// 5. Phase-Locked Loop for topic tracking
syncedThought, err := hyperspace.ContextResonance(thought, globalContext, 0.5, 1.0)
```

## Embedding Pipeline (Optional)


HyperspaceDB supports **per-geometry embeddings** configured via environment variables on the server side. Each geometry (`l2`, `cosine`, `poincare`, `lorentz`, `hybrid`) can use its own backend independently.

### Quick Setup (Server `.env`)

```env
HYPERSPACE_EMBED=true

# L2 via OpenAI
HS_EMBED_L2_PROVIDER=openai
HS_EMBED_L2_EMBED_MODEL=text-embedding-3-small
HS_EMBED_L2_API_KEY=sk-...

# Cosine via Cohere
HS_EMBED_COSINE_PROVIDER=cohere
HS_EMBED_COSINE_EMBED_MODEL=embed-english-v3.0
HS_EMBED_COSINE_API_KEY=...

# Poincaré via HuggingFace Hub (auto-downloads model.onnx + tokenizer.json)
HS_EMBED_POINCARE_PROVIDER=huggingface
HS_EMBED_POINCARE_HF_MODEL_ID=your-org/cde-spatial-poincare-128d
HS_EMBED_POINCARE_DIM=128
HF_TOKEN=hf_...  # Optional — required for gated/private models

# Lorentz via local ONNX file
HS_EMBED_LORENTZ_PROVIDER=local
HS_EMBED_LORENTZ_MODEL_PATH=./models/lorentz_128d.onnx
HS_EMBED_LORENTZ_TOKENIZER_PATH=./models/lorentz_128d_tokenizer.json
HS_EMBED_LORENTZ_DIM=129  # spatial_dim + 1 for the time component x₀
```

### Multi-Collection Search (Benchmark Across Geometries)

Use `SearchMultiCollection` to compare the same query across all geometry types simultaneously:

```go
req := &pb.SearchMultiCollectionRequest{
    Vector:      []float64{0.1, 0.2, -0.3},
    Collections: []string{"docs_l2", "docs_cosine", "docs_poincare", "docs_lorentz"},
    TopK:        10,
}
res, err := client.SearchMultiCollection(ctx, req)
// Results: map collection_name → []SearchResult
```

### Supported Geometries

| Geometry | Post-Processing | Best For |
|---|---|---|
| `cosine` | Unit normalize | Semantic similarity |
| `l2` | Unit normalize | Euclidean distance tasks |
| `poincare` | Clamp to unit ball | Hierarchical data (ontologies, taxonomies) |
| `lorentz` | None | Mixed hierarchical + semantic (knowledge graphs) |
| `hybrid` | None | Lorentz + L2 combined metric |

## Zero-Knowledge Client-Side Encryption (ZK-Privacy)

HyperspaceDB v3.1.1 supports Zero-Knowledge client-side encryption (ZK-Privacy) for Go. Private vectors, metadata, and sidecar payloads are projected, noise-injected, and encrypted *before* transmitting over the wire. The database server only processes the projected vectors and obfuscated metadata.

### Usage Example

```go
package main

import (
	"context"
	"fmt"
	"log"

	"github.com/yarlabs/hyperspace-sdk-go"
	pb "github.com/yarlabs/hyperspace-sdk-go/proto"
)

func main() {
	client, err := hyperspace.NewClient("localhost:50051", "I_LOVE_HYPERSPACEDB")
	if err != nil {
		log.Fatalf("failed to connect: %v", err)
	}
	defer client.Close()

	ctx := context.Background()
	collection := "secure_docs"
	secretKey := "my-secret-key-go"

	// Create schema
	schema := &pb.CollectionSchema{
		Components: []*pb.VectorComponent{
			{Name: "primary", Metric: "cosine", FullDimension: 3, Weight: 1.0},
		},
	}

	// Register the key to enable ZK client-side encryption
	// noiseSigma defaults to 0.02 (anisotropic noise fraction)
	client.RegisterCollectionKey(collection, secretKey, "cosine", 0.02, schema)

	err = client.CreateCollectionWithQuantization(ctx, collection, schema, secretKey, 0.02, "turbo")
	if err != nil {
		log.Fatalf("failed to create collection: %v", err)
	}

	// 1. Insert (Vector will be projected, noise injected, payload encrypted, and metadata hashed)
	insertParams := &hyperspace.InsertParams{
		Metadata: map[string]string{
			"category": "secret",
		},
		Payload: []byte("Top secret data payload"),
	}
	err = client.Insert(ctx, 1, []float64{0.1, 0.2, 0.3}, collection, insertParams)
	if err != nil {
		log.Fatalf("failed to insert: %v", err)
	}

	// 2. Search (Query vector projected and filters hashed; payloads decrypted locally)
	searchParams := &hyperspace.SearchParams{
		Filters: []*pb.Filter{
			{
				Condition: &pb.Filter_Match{
					Match: &pb.Match{
						Key:   "category",
						Value: "secret",
					},
				},
			},
		},
	}
	results, err := client.Search(ctx, []float64{0.1, 0.2, 0.3}, 5, collection, searchParams)
	if err != nil {
		log.Fatalf("failed to search: %v", err)
	}

	for _, res := range results {
		fmt.Printf("ID: %d, Distance: %f\n", res.Id, res.Distance)
		if len(res.Payload) > 0 {
			fmt.Printf("Decrypted Payload: %s\n", string(res.Payload))
		}
	}
}
```

## Native Vector-Graph (KAG) & Topology Linking

HyperspaceDB 4.1+ internalizes explicit directed labeled graph relations alongside vector embeddings.

### 1. Linking Nodes (Explicit Directed Edges)

```go
// Create a directed edge with relation label and weight
err := client.Link(ctx, "my_collection", 1, 2, "OWNS", 1.0)
if err != nil {
    log.Fatalf("Link failed: %v", err)
}

// Batch link multiple edges in a single atomic RPC
edges := []hyperspace.EdgeInput{
    {SourceID: 1, TargetID: 2, Relation: "OWNS", Weight: 1.0},
    {SourceID: 2, TargetID: 3, Relation: "DEPENDS_ON", Weight: 0.95},
}
count, err := client.LinkBatch(ctx, "my_collection", edges)
if err != nil {
    log.Fatalf("LinkBatch failed: %v", err)
}
fmt.Printf("Linked %d edges in CSR graph\n", count)

// Delete an edge with zero-lag invalidation
err = client.Unlink(ctx, "my_collection", 1, 2, "OWNS")
if err != nil {
    log.Fatalf("Unlink failed: %v", err)
}
```

### 2. Declarative Metadata-to-Edge Induction

Generate topological graphs from existing metadata for 100k+ documents without rebuilding HNSW indices:

```go
rules := []hyperspace.MetadataInductionRule{
    // 1. Directory/file path hierarchy: /a/b/c -> /a/b
    {
        Field:    "file_path",
        RuleType: "path_hierarchy",
        Relation: "CONTAINS",
    },
    // 2. Foreign-key entity links: parent_id -> target document id
    {
        Field:    "parent_id",
        RuleType: "foreign_key",
        Relation: "OWNS",
    },
}

created, err := client.InduceEdgesFromMetadata(ctx, "my_collection", rules)
if err != nil {
    log.Fatalf("Induce failed: %v", err)
}
fmt.Printf("Induced %d explicit edges directly from metadata\n", created)
```

### 3. Collection Stats & Encryption Verification

```go
stats, err := client.GetCollectionStats(ctx, "secure_docs")
if err != nil {
    log.Fatalf("Stats failed: %v", err)
}
fmt.Printf("Collection %s: VectorCount=%d, Encrypted=%v\n", stats.Name, stats.VectorCount, stats.Encrypted)
```


