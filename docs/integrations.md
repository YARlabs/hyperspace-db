# HyperspaceDB Integrations

HyperspaceDB is designed to work seamlessly with modern AI frameworks.

## LangChain (Python)

Official integration for LangChain Python.

### Installation

```bash
pip install langchain-hyperspace
```

### Usage

```python
from langchain_hyperspace import HyperspaceVectorStore
from langchain_openai import OpenAIEmbeddings

# Initialize
vectorstore = HyperspaceVectorStore(
    host="localhost",
    port=50051,
    collection_name="my_docs",
    embedding_function=OpenAIEmbeddings()
)

# Add Documents
vectorstore.add_texts(["Hello world", "HyperspaceDB is fast"])

# Search
results = vectorstore.similarity_search("fast database")
```

### Features
- **Automatic Deduplication**: Uses Merkle hash of content to prevent duplicates.
- **Edge-Cloud Sync**: Local changes automatically sync when connected.

---

## LangChain (JavaScript/TypeScript)

Official integration for LangChain JS.

### Installation

```bash
npm install langchain-hyperspace
```

### Usage

```typescript
import { HyperspaceStore } from "langchain-hyperspace";
import { OpenAIEmbeddings } from "@langchain/openai";

const store = new HyperspaceStore(new OpenAIEmbeddings(), {
  client: grpcClient, // gRPC client instance
  collectionName: "documents"
});

await store.addDocuments([
  { pageContent: "Hello world", metadata: { source: "test" } }
]);

const results = await store.similaritySearch("Hello", 1);
```

---

## Vercel AI SDK (Coming Soon)

Integrate HyperspaceDB as long-term memory for Vercel AI Chat bots.

## n8n (Coming Soon)

No-code node for building AI workflows with HyperspaceDB memory.
