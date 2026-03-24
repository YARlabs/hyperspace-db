import { Embeddings } from "@langchain/core/embeddings";
import { HyperspaceStore } from "../integrations/langchain-js/src/vectorstores";
import { HyperspaceClient } from "../sdks/ts/src/client";

class FakeEmbeddings extends Embeddings {
    async embedDocuments(documents: string[]): Promise<number[][]> {
        return documents.map(() => new Array(1024).fill(0.1));
    }
    async embedQuery(query: string): Promise<number[]> {
        return new Array(1024).fill(0.1);
    }
}

async function testLangChainJS() {
    console.log("🚀 Starting LangChain JS Smoke Test...");

    const client = new (HyperspaceClient as any)("localhost:50051", "I_LOVE_HYPERSPACEDB");
    const collectionName = "test_langchain_js_col";

    // Ensure collection exists
    try {
        await client.createCollection(collectionName, 1024, "cosine");
        console.log(`✅ Created collection '${collectionName}'`);
    } catch (e) {
        console.log("ℹ️ Collection check finished.");
    }

    const embeddings = new FakeEmbeddings();
    const vectorStore = new HyperspaceStore(embeddings as any, {
        client: client as any,
        collectionName,
        dimension: 1024,
        metric: "cosine"
    });

    console.log("📝 Adding documents...");
    await vectorStore.addDocuments([
        { pageContent: "HyperspaceDB is powerful.", metadata: { id: 1 } },
        { pageContent: "LangChain is flexible.", metadata: { id: 2 } },
    ]);

    console.log("🔍 Searching...");
    const results = await vectorStore.similaritySearch("powerful database", 1);

    if (results.length > 0) {
        console.log(`✅ Found: ${results[0].pageContent}`);
        console.log("🎉 LangChain JS smoke test PASSED!");
    } else {
        console.log("❌ LangChain JS smoke test FAILED: No results.");
    }
}

testLangChainJS().catch(console.error);
