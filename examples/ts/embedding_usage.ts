import { HyperspaceClient } from '../../sdks/ts/src/client';

async function main() {
    console.log("🚀 HyperspaceDB Built-in Embedding Example (TS)");
    console.log("------------------------------------------");

    // 1. Connect to local instance
    // Make sure server is running with HYPERSPACE_EMBED=true
    const client = new HyperspaceClient('localhost:50051', 'I_LOVE_HYPERSPACEDB');
    const colName = 'embedding_test_ts';

    try {
        // Simple Cleanup
        await client.deleteCollection(colName).catch(() => { });

        // 2. Create a collection (we'll use Cosine for standard text search)
        console.log(`Creating collection '${colName}' with Cosine metric...`);
        // dimension should match the server's embedding model for Cosine (e.g. 1024 for Qwen3)
        await client.createCollection(colName, 1024, 'cosine');
        console.log('Collection created.');

        // 3. Using insertText
        console.log("\n📝 Inserting text documents via client.insertText()...");
        const documents = [
            "HyperspaceDB is a spatial AI engine built on Rust.",
            "Hyperbolic geometry is ideal for hierarchical data structures.",
            "Qwen3-Embedding-0.6B provides high-accuracy 1024d vectors.",
            "Robotics and autonomous agents need low-latency memory.",
            "Vectorization happens entirely on the server side now."
        ];

        for (let i = 0; i < documents.length; i++) {
            const doc = documents[i];
            console.log(` -> Indexing: "${doc.slice(0, 40)}..."`);
            // Metadata support currently pending serialization fix
            // But we can still insert text!
            await client.insertText(i + 1, doc, {}, colName);
        }

        console.log("\n✅ Insertion complete. Waiting for async ingestion...");
        await new Promise(r => setTimeout(r, 1500)); // Wait for HNSW background worker

        // 4. Search via searchText
        console.log("\n🔍 Searching via client.searchText()...");
        const query = "How to handle hierarchies in vectors?";
        console.log(`Query: "${query}"`);

        const results = await client.searchText(query, 5, colName);

        console.log('\nResults:');
        results.forEach(r => console.log(`  [ID: ${r.id}] Score: ${r.distance.toFixed(4)}`));


        // 5. Raw vector lookup via client.vectorize()
        console.log("\n🧠 Manual vectorization via client.vectorize()...");
        const testStr = "Explain spatial reasoning.";
        const vector = await client.vectorize(testStr, 'cosine');

        if (vector && vector.length > 0) {
            console.log(`Vector generated for: "${testStr}"`);
            console.log(`First 5 dimensions: [${vector.slice(0, 5).join(', ')}...]`);
        } else {
            console.log("❌ Vectorization failed. Check if server embedding is enabled.");
        }

        // 6. cleanup
        // await client.deleteCollection(colName);
        console.log("\nDone. You can inspect the results in the Dashboard: http://localhost:50050");

        client.close();
    } catch (e) {
        console.error('Error:', e);
        client.close();
    }
}

main();
