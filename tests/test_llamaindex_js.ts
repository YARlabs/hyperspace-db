import { HyperspaceVectorStore } from "../integrations/llamaindex-js/src/index";
import { HyperspaceClient } from "../sdks/ts/src/client";

async function testLlamaIndexJS() {
    console.log("🚀 Starting LlamaIndex JS Smoke Test...");

    const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");

    // Ensure collection exists
    try {
        await client.createCollection("test_llamaindex_js_col", 1024, "cosine");
        console.log("✅ Created collection 'test_llamaindex_js_col'");
    } catch (e) {
        console.log("ℹ️ Collection exists or error:", e);
    }

    const vectorStore = new HyperspaceVectorStore({
        client,
        collectionName: "test_llamaindex_js_col"
    });

    console.log("🧪 Testing VectorStore.add() directly...");
    const node1: any = {
        getContent: () => "Test content 1",
        getMetadata: () => ({}),
        id_: "node1",
        embedding: new Array(1024).fill(0.1),
    };

    await vectorStore.add([node1]);
    console.log("✅ VectorStore.add() passed.");

    console.log("🔍 Testing VectorStore.query()...");
    const queryResult = await vectorStore.query({
        queryEmbedding: new Array(1024).fill(0.1),
        similarityTopK: 1,
        mode: "default" as any
    });

    console.log("✅ Query Result:", queryResult.nodes?.length, "nodes found.");
    if (queryResult.nodes && queryResult.nodes.length > 0) {
        console.log("🎉 LlamaIndex JS smoke test PASSED!");
    } else {
        console.log("❌ LlamaIndex JS smoke test FAILED: No nodes found.");
    }
}

testLlamaIndexJS().catch(console.error);
