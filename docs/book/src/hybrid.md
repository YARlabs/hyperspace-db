# 🧠 Hybrid Search (Dense + Sparse + Multi-Vector)

HyperspaceDB combines **Hyperbolic Vector Search** with state-of-the-art **BM25 Lexical Ranking** to deliver maximum retrieval accuracy.

## Conceptual Flow

HyperspaceDB supports three levels of hybrid search:

1. **Semantic Branch (Dense)**: Finds conceptually similar items using HNSW (L2, Cosine, Poincaré, Lorentz).
2. **Lexical Branch (Sparse)**: Finds exact token matches using a BM25-optimized inverted index.
3. **Multi-Vector Branch**: Fuses multiple vector components (e.g. **Lorentz + L2**) within the same request.

### 1. Vector + Lexical Fusion (BM25)

Scores from dense and sparse branches are fused using **Reciprocal Rank Fusion (RRF)** or **Linear Weighted Fusion**.

`RRF Score = 1/(k + rank_vec) + 1/(k + rank_lex)` (where `k` defaults to 60).

### 2. Multi-Vector Fusion (Hybrid Metrics)

In v3.1.0, you can search across multiple vector components defined in your `CollectionSchema`. Use `component_weights` to balance them.

Example: **Lorentz (Hierarchy) + L2 (Similarity)**.

## BM25 Options

You can tune the lexical scavenger by providing a `bm25` configuration:

- `method`: `"bm25"` (classic), `"bm25plus"` (recommended for long docs), `"lucene"`, `"atire"`.
- `k1`: Term frequency saturation (default 1.2).
- `b`: Length normalization impact (default 0.75).
- `language`: Stemmer choice (e.g. `"english"`, `"russian"`).

## API Usage

### Python

```python
# 1. Full Hybrid (Dense Vector + BM25 Sparse)
# This uses Reciprocal Rank Fusion (RRF) by default
results = client.search(
    vector=query_vector,
    hybrid_query="apple macbook air",
    hybrid_alpha=0.7,      # 70% vector weight, 30% lexical weight
    top_k=10,
    bm25={
        "method": "bm25plus",
        "language": "english"
    }
)

# 2. Multi-Vector Component Weighting (v3.1.0)
# Weighting internal schema components (e.g. Lorentz vs L2)
results = client.search(
    vector=query_vector,
    top_k=10,
    component_weights={"lorentz_part": 0.8, "l2_part": 0.2}
)

# 3. Triple Hybrid (Multi-Vector + BM25)
# The ultimate retrieval pipeline
results = client.search(
    vector=query_vector,
    hybrid_query="macbook pro hierarchy",
    component_weights={"lorentz": 1.0},
    bm25={"method": "lucene"},
    top_k=5
)
```

### TypeScript

```ts
const results = await client.search(vector, 10, "collection", {
  hybridQuery: "apple macbook",
  hybridAlpha: 0.7,
  bm25: { method: "bm25plus" }
});
```

### Rust

```rust
let results = client.search(SearchRequest {
    collection: "docs".into(),
    vector: query_vector,
    top_k: 10,
    hybrid_query: Some("macbook".into()),
    hybrid_alpha: Some(0.7),
    bm25_options: Some(Bm25Options {
        method: "bm25plus".into(),
        ..Default::default()
    }),
    ..Default::default()
}).await?;
```

## Tokenization

The engine uses a built-in multi-lingual tokenizer that performs:
- Case folding (lower-casing).
- Alpha-numeric filtering.
- Stop-word removal (optional).
- Language-specific stemming based on `bm25_options.language`.
