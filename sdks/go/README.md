# HyperspaceDB Go SDK

Official Go client for the **HyperspaceDB** gRPC API. 

This SDK features pre-generated protocol buffers (`proto/`) to interact directly with the high-performance HyperspaceDB server (`v3.1.0`). It is tailored for high-concurrency event-streaming systems, CDC syncs, and microservices managing hyperspatial graph databases.

## Integration

To install and use our package, retrieve it via Go Modules:

```bash
go get github.com/yarlabs/hyperspace-sdk-go
```

## Features

- **Vector Searching**: High-performance Batched searches spanning Poincare, Lorentz, Cosine, L2, and Wasserstein cross-feature metrics (`SearchRequest.UseWasserstein`).
- **Memory Reconsolidation (Sleep Mode)**: Optimize datasets using `TriggerReconsolidation` directly inside your database from microservices.
- **CDC Streaming**: React to structural topology changes in real time through `SubscribeToEvents`/Event Streams.
- **Graph Traversal APIs**: Access direct paths via `GetNeighbors` and perform node clustering.

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

	// Create collection
	_ = client.CreateCollection(ctx, collection, 1024, "cosine")

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
			log.Printf("Collection: %s, Count: %d, Dim: %d, Metric: %s", col.Name, col.Count, col.Dimension, col.Metric)
		}
	}
}

```

## Geometric Filters (New in v3.0)

HyperspaceDB v3.0 introduces advanced spatial filters that run on the engine level:

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
## Embedding Pipeline (Optional)

HyperspaceDB supports **per-geometry embeddings** configured via environment variables on the server side. Each geometry (`l2`, `cosine`, `poincare`, `lorentz`) can use its own backend independently.

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
