
# 🧠 Hybrid Search

HyperspaceDB combines **Hyperbolic Vector Search** with **Lexical (Keyword) Search** to provide the best of both worlds.

This is powered by **Reciprocal Rank Fusion (RRF)**, which normalizes scores from both engines and merges them.

## Conceptual Flow

1. **Vector Search**: Finds semantically similar items (e.g. "smartphone" finds "iPhone").
2. **Keyword Search**: Finds exact token matches in metadata (e.g. "iphone" finds items with "iphone" in title).
3. **RRF Fusion**: `Score = 1/(k + rank_vec) + 1/(k + rank_lex)`.

## API Usage

### Python

```python
results = client.search(
    vector=query_vector,
    top_k=10,
    hybrid_query="apple macbook",  # Lexical query
    hybrid_alpha=0.5               # Balance factor (default 60.0 in RRF usually, but exposed as alpha here)
)
```

### Rust

```rust
let results = client.search_advanced(
    query_vector,
    10,
    vec![], 
    Some(("apple macbook".to_string(), 0.5))
).await?;
```

## Tokenization

Currently, all string metadata values are automatically tokenized (split by whitespace, lowercase, alphanumeric) and indexed in an inverted index.
