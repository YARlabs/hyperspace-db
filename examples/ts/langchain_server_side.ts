import { HyperspaceStore } from "../../integrations/langchain-js/src/vectorstores";
import { HyperspaceClient } from "hyperspace-sdk-ts";
import { Document } from "@langchain/core/documents";

/**
 * This example demonstrates using HyperspaceDB without any external embedding provider
 * (like OpenAI or HuggingFace). The server handles vectorization automatically.
 */
async function main() {
    // 1. Initialize Client
    const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");
    
    // 2. Initialize Store with useServerSideEmbedding: true
    // Note: We don't provide an 'embeddings' object (passing undefined/dummy)
    const store = new HyperspaceStore(undefined, {
        client,
        collectionName: "server_side_demo",
        useServerSideEmbedding: true,
    });

    // 3. Add Documents (Vectors will be generated on the server)
    console.log("Adding documents (server-side embedding)...");
    await store.addDocuments([
        new Document({ 
            pageContent: "Quantum computing uses qubits instead of bits.",
            metadata: { topic: "physics" } 
        }),
        new Document({ 
            pageContent: "Artificial intelligence is shifting the technology landscape.",
            metadata: { topic: "tech" } 
        })
    ]);

    // 4. Search (Server will also vectorize the query)
    console.log("Searching for 'science stuff'...");
    const results = await store.similaritySearch("science stuff", 1);
    console.log("Result:", results[0].pageContent);
    console.log("Metadata:", results[0].metadata);

    client.close();
    console.log("Done!");
}

main().catch(console.error);
