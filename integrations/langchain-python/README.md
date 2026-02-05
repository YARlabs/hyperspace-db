# LangChain HyperspaceDB Integration

[![PyPI version](https://badge.fury.io/py/langchain-hyperspace.svg)](https://badge.fury.io/py/langchain-hyperspace)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

Official LangChain integration for [HyperspaceDB](https://github.com/yourusername/hyperspace-db) - a hyperbolic vector database with Edge-Cloud Federation.

## Features

- 🌐 **Hyperbolic Geometry**: Poincaré ball model for hierarchical embeddings
- 🔄 **Edge-Cloud Federation**: Offline-first with automatic sync
- 🌳 **Merkle Tree Sync**: Efficient data replication and verification
- 🗜️ **1-bit Quantization**: 64x memory reduction with minimal accuracy loss
- 🔍 **Built-in Deduplication**: Content-based hashing prevents duplicates
- ⚡ **High Performance**: Written in Rust for maximum speed

## Installation

```bash
pip install langchain-hyperspace
```

## Quick Start

### Basic Usage

```python
from langchain_hyperspace import HyperspaceVectorStore
from langchain_openai import OpenAIEmbeddings
from langchain.text_splitter import CharacterTextSplitter
from langchain.document_loaders import TextLoader

# Initialize embeddings
embeddings = OpenAIEmbeddings()

# Create vector store
vectorstore = HyperspaceVectorStore(
    host="localhost",
    port=50051,
    collection_name="my_documents",
    embedding_function=embeddings,
    api_key="your_api_key"  # Optional
)

# Load and split documents
loader = TextLoader("path/to/document.txt")
documents = loader.load()
text_splitter = CharacterTextSplitter(chunk_size=1000, chunk_overlap=0)
docs = text_splitter.split_documents(documents)

# Add documents to vector store
vectorstore.add_documents(docs)

# Search for similar documents
query = "What is the main topic?"
results = vectorstore.similarity_search(query, k=4)

for doc in results:
    print(doc.page_content)
```

### RAG (Retrieval-Augmented Generation) Example

```python
from langchain_hyperspace import HyperspaceVectorStore
from langchain_openai import OpenAIEmbeddings, ChatOpenAI
from langchain.chains import RetrievalQA

# Setup
embeddings = OpenAIEmbeddings()
vectorstore = HyperspaceVectorStore(
    host="localhost",
    port=50051,
    collection_name="knowledge_base",
    embedding_function=embeddings
)

# Create RAG chain
llm = ChatOpenAI(model_name="gpt-4")
qa_chain = RetrievalQA.from_chain_type(
    llm=llm,
    chain_type="stuff",
    retriever=vectorstore.as_retriever(search_kwargs={"k": 3})
)

# Ask questions
response = qa_chain.run("What are the key features of HyperspaceDB?")
print(response)
```

### Content Deduplication

HyperspaceDB automatically deduplicates content using SHA-256 hashing:

```python
vectorstore = HyperspaceVectorStore(
    host="localhost",
    port=50051,
    collection_name="deduplicated_docs",
    embedding_function=embeddings,
    enable_deduplication=True  # Default
)

# Adding the same text twice will only store it once
vectorstore.add_texts([
    "This is a unique document",
    "This is a unique document",  # Duplicate - will be skipped
    "This is another document"
])
```

### Sync Verification (Edge-Cloud Federation)

Check synchronization status using Merkle Tree digest:

```python
# Get collection digest
digest = vectorstore.get_digest()

print(f"Logical Clock: {digest['logical_clock']}")
print(f"State Hash: {digest['state_hash']}")
print(f"Vector Count: {digest['count']}")
print(f"Bucket Hashes: {len(digest['buckets'])} buckets")
```

## Configuration

### Connection Options

```python
vectorstore = HyperspaceVectorStore(
    host="localhost",          # Server host
    port=50051,                # gRPC port
    collection_name="default", # Collection name
    embedding_function=embeddings,
    api_key=None,              # Optional API key
    dimension=1536,            # Vector dimension (must match embeddings)
    metric="l2",               # Distance metric: 'l2', 'cosine', 'dot'
    enable_deduplication=True  # Enable content-based deduplication
)
```

### Distance Metrics

- `l2`: Euclidean distance (default)
- `cosine`: Cosine similarity
- `dot`: Dot product

## Advanced Usage

### Metadata Filtering

```python
# Add documents with metadata
vectorstore.add_texts(
    texts=["Document 1", "Document 2"],
    metadatas=[
        {"source": "web", "category": "tech"},
        {"source": "pdf", "category": "science"}
    ]
)

# Search with metadata filter (coming soon)
results = vectorstore.similarity_search(
    "technology trends",
    k=5,
    filter={"category": "tech"}
)
```

### Batch Operations

```python
# Add large batches efficiently
texts = [f"Document {i}" for i in range(10000)]
metadatas = [{"index": i} for i in range(10000)]

vectorstore.add_texts(texts, metadatas=metadatas)
```

## Running HyperspaceDB Server

### Using Docker

```bash
docker run -p 50051:50051 -p 50050:50050 \
  -e HYPERSPACE_API_KEY=your_secret_key \
  hyperspacedb/hyperspace-server:latest
```

### From Source

```bash
git clone https://github.com/yourusername/hyperspace-db
cd hyperspace-db
cargo build --release
HYPERSPACE_API_KEY=your_secret_key ./target/release/hyperspace-server
```

## Development

### Setup

```bash
git clone https://github.com/yourusername/hyperspace-db
cd hyperspace-db/integrations/langchain-python

# Install in development mode
pip install -e ".[dev]"

# Generate protobuf files
./generate_proto.sh
```

### Running Tests

```bash
pytest tests/
```

### Code Quality

```bash
# Format code
black src/ tests/

# Lint
ruff check src/ tests/

# Type check
mypy src/
```

## Examples

See the [examples/](examples/) directory for complete examples:

- `rag_chatbot.py`: RAG chatbot with memory
- `document_qa.py`: Document Q&A system
- `semantic_search.py`: Semantic search engine
- `edge_sync.py`: Edge-Cloud synchronization demo

## Documentation

- [HyperspaceDB Documentation](https://hyperspacedb.io/docs)
- [LangChain Documentation](https://python.langchain.com/docs)
- [API Reference](https://hyperspacedb.io/docs/api)

## Performance

HyperspaceDB is optimized for:

- **Insert**: 10K+ vectors/second
- **Search**: <10ms p99 latency
- **Memory**: 64x reduction with 1-bit quantization
- **Sync**: Merkle Tree-based differential sync

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

Apache License 2.0 - see [LICENSE](../../LICENSE) for details.

## Support

- GitHub Issues: [Report bugs](https://github.com/yourusername/hyperspace-db/issues)
- Discord: [Join community](https://discord.gg/hyperspacedb)
- Email: support@hyperspacedb.io

## Citation

If you use HyperspaceDB in your research, please cite:

```bibtex
@software{hyperspacedb2024,
  title = {HyperspaceDB: Hyperbolic Vector Database with Edge-Cloud Federation},
  author = {HyperspaceDB Team},
  year = {2024},
  url = {https://github.com/yourusername/hyperspace-db}
}
```
