# HyperspaceDB Go SDK

Official Go client for the **HyperspaceDB** gRPC API. 

This SDK features pre-generated protocol buffers (`proto/`) to interact directly with the high-performance HyperspaceDB server (`v3.1.1`). It is tailored for high-concurrency event-streaming systems, CDC syncs, and microservices managing hyperspatial graph databases.

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

	// Create collection with Schema (Multi-Vector & MRL support)
	_ = client.CreateCollection(ctx, collection, &pb.CollectionSchema{
		Components: []*pb.VectorComponent{
			{Name: "primary", Metric: "cosine", FullDimension: 1024, Weight: 1.0},
		},
	})

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

	err = client.CreateCollection(ctx, collection, schema, secretKey, 0.02)
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

