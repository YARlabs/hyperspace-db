# 🧠 Hybrid Search (BM25 + RRF)

HyperspaceDB combines **Hyperbolic Vector Search** with state-of-the-art **BM25 Lexical Ranking** to deliver maximum retrieval accuracy.

## Conceptual Flow

1. **Semantic Branch (Dense)**: Finds conceptually similar items using HNSW (L2, Cosine, Poincaré).
2. **Lexical Branch (Sparse)**: Finds exact token matches using a BM25-optimized inverted index.
3. **Fusion Layer**: Scores from both branches are fused using **Reciprocal Rank Fusion (RRF)** or **Linear Weighted Fusion**.

`RRF Score = 1/(k + rank_vec) + 1/(k + rank_lex)` (where `k` defaults to 60).

## BM25 Options

You can tune the lexical scavenger by providing a `bm25` configuration:

- `method`: `"bm25"` (classic), `"bm25plus"` (recommended for long docs), `"lucene"`, `"atire"`.
- `k1`: Term frequency saturation (default 1.2).
- `b`: Length normalization impact (default 0.75).
- `language`: Stemmer choice (e.g. `"english"`, `"russian"`).

## API Usage

### Python

```python
results = client.search(
    vector=query_vector,
    hybrid_query="apple macbook air",
    hybrid_alpha=0.7,  # 70% vector weight
    top_k=10,
    bm25={
        "method": "bm25plus",
        "language": "english"
    }
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
