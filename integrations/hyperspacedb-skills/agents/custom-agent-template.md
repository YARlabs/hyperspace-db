# Custom Agent Template for HyperspaceDB

Copy and adapt this template to add HyperspaceDB awareness to any custom AI agent,
LangChain agent, LlamaIndex agent, or AutoGen agent.

## System Prompt Injection

Add the following to your agent's system prompt:

---

```
You have access to HyperspaceDB — a multi-geometry vector database with built-in
cognitive AI capabilities, accessed via gRPC.

## What you can do with HyperspaceDB:

### Memory & Knowledge Storage
- Store facts, documents, reasoning steps as searchable vectors
- Use `insertText()` to automatically embed and store text
- Retrieve stored knowledge with `searchText()` (semantic similarity)

### Graph Knowledge Navigation
- Navigate multi-hop knowledge paths with `traverse()`
- Explore concept hierarchies with `getSubsumptionTree()`
- Find related concept clusters with `findSemanticClusters()`

### Reasoning Validation
- Check if your reasoning chain is converging: `analyzeThoughtStability()`
- Predict where your reasoning is heading: `predictMomentum()`
- Score your reasoning confidence: `getTrustScore()` (0.0 – 1.0)

## Important constraints:
- Connection via env: HYPERSPACE_HOST and HYPERSPACE_API_KEY
- Choose geometry based on data type (cosine for text, lorentz for hierarchies)
- DePIN features are alpha — use on testnet only
```

---

## LangChain Integration (Python)

```python
from langchain_hyperspacedb import HyperspaceDBVectorStore
from hyperspacedb import HyperspaceClient
import os

client = HyperspaceClient(
    host=os.environ["HYPERSPACE_HOST"],
    api_key=os.environ["HYPERSPACE_API_KEY"]
)

vectorstore = HyperspaceDBVectorStore(
    client=client,
    collection_name="agent_memory",
    embedding=your_embedding_model
)

retriever = vectorstore.as_retriever(search_kwargs={"k": 5})
```

## LlamaIndex Integration (Python)

```python
from llama_index_hyperspacedb import HyperspaceDBVectorIndex
from hyperspacedb import HyperspaceClient
import os

client = HyperspaceClient(
    host=os.environ["HYPERSPACE_HOST"],
    api_key=os.environ["HYPERSPACE_API_KEY"]
)

index = HyperspaceDBVectorIndex.from_client(
    client=client,
    collection_name="knowledge_base"
)

query_engine = index.as_query_engine()
response = query_engine.query("What is quantum entanglement?")
```

## LangChain Integration (TypeScript)

```typescript
import { HyperspaceDBVectorStore } from 'langchain-hyperspacedb';
import { HyperspaceClient } from 'hyperspace-sdk-ts';

const client = new HyperspaceClient(
  process.env.HYPERSPACE_HOST!,
  process.env.HYPERSPACE_API_KEY!
);

const vectorStore = new HyperspaceDBVectorStore({
  client,
  collectionName: 'agent_memory',
  embeddingModel: yourEmbeddingModel,
});
```

## Direct SDK for Custom Agents

```typescript
import { HyperspaceClient, CognitiveMathExport } from 'hyperspace-sdk-ts';

class HyperspaceMemoryAgent {
  private db: HyperspaceClient;
  private cotIds: number[] = [];

  constructor() {
    this.db = new HyperspaceClient(
      process.env.HYPERSPACE_HOST!,
      process.env.HYPERSPACE_API_KEY!
    );
  }

  async think(thought: string): Promise<{ stable: boolean; trust: number }> {
    // Store thought step
    const id = await this.db.insertText(thought, {
      step: String(this.cotIds.length),
      ts: new Date().toISOString()
    }, 'agent_cot');
    this.cotIds.push(id);

    // Validate after every 5 steps
    if (this.cotIds.length >= 5) {
      const window = this.cotIds.slice(-10);
      const stability = await this.db.analyzeThoughtStability(window, 1.0, 'agent_cot');
      const trust = await this.db.getTrustScore(window, 'agent_cot');
      return { stable: stability.is_stable, trust: trust.score };
    }

    return { stable: true, trust: 1.0 };
  }

  async recall(query: string, k = 5) {
    return this.db.searchText(query, k, 'agent_cot');
  }
}
```

## n8n Workflow Node

```json
{
  "type": "n8n-nodes-hyperspacedb.hyperspaceDb",
  "typeVersion": 1,
  "parameters": {
    "operation": "searchText",
    "collection": "{{ $env.COLLECTION_NAME }}",
    "query": "{{ $json.userQuery }}",
    "topK": 5
  },
  "credentials": {
    "hyperspaceDbApi": {
      "host": "{{ $env.HYPERSPACE_HOST }}",
      "apiKey": "{{ $env.HYPERSPACE_API_KEY }}"
    }
  }
}
```
