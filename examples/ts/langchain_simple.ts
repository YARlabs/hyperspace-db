import { HyperspaceStore, YARLabsEmbeddings } from "../../integrations/langchain-js/src/index";
import { HyperspaceClient } from "../../sdks/ts/src/client";
import { Document } from "@langchain/core/documents";

async function main() {
    const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");
    
    // 2. Initialize YARLabs Local Embeddings (requires @xenova/transformers)
    const embeddings = new YARLabsEmbeddings();
    
    // 1. Initialize Store
    const store = new HyperspaceStore(embeddings, {
        client,
        collectionName: "langchain_js_lorentz_demo",
        // The YARLabs 0.5B model with targetDim=64 produces 65-dimensional Lorentz vectors (1 + 64)
        dimension: 65,
        metric: "lorentz",
    });

    // 2. Add Documents
    console.log("Adding documents...");
    await store.addDocuments([
        new Document({ 
            pageContent: "HyperspaceDB is a multi-geometry vector database.",
            metadata: { category: "tech", level: "advanced" } 
        }),
        new Document({ 
            pageContent: "LangChain is a framework for developing AI applications.",
            metadata: { category: "ai", level: "beginner" } 
        })
    ]);

    // 3. Similarity Search
    console.log("Searching for 'vector database'...");
    const results = await store.similaritySearch("vector database", 1);
    console.log("Top result:", results[0].pageContent);

    // 4. Search with Metadata Filter
    console.log("Searching for 'AI' with category=ai filter...");
    const filteredResults = await store.similaritySearch("AI", 1, { category: "ai" });
    console.log("Filtered result:", filteredResults[0].pageContent);

    // 5. Delete by ID
    const demoId = "12345";
    await store.addDocuments([new Document({ pageContent: "Temp doc", metadata: { id: demoId } })], { ids: [demoId] });
    console.log("Deleting document...");
    await store.delete({ ids: [demoId] });
    
    client.close();
}

main().catch(console.error);
