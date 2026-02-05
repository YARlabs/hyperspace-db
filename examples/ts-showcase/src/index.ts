import { HyperspaceStore } from "langchain-hyperspace";
import { HyperspaceClient } from "hyperspace-sdk-ts";
import { Embeddings } from "@langchain/core/embeddings";
import { Document } from "@langchain/core/documents";

// --- Fake Embeddings Implementation ---
// Generates random vectors for demonstration without API keys
class FakeEmbeddings extends Embeddings {
    constructor(private dim: number = 1024) { // Changed default dimension to 1024 to match server default
        super({});
    }

    async embedDocuments(documents: string[]): Promise<number[][]> {
        return documents.map(() => this.generateVector());
    }

    async embedQuery(document: string): Promise<number[]> {
        return this.generateVector();
    }

    private generateVector(): number[] {
        return Array.from({ length: this.dim }, () => Math.random());
    }
}

// --- Main Showcase ---
async function main() {
    console.log("🚀 Starting HiveMind Showcase...");

    // 1. Connect to HyperspaceDB
    const client = new HyperspaceClient("localhost:50051", "I_LOVE_HYPERSPACEDB");
    const collectionName = "hivemind_demo";

    // Create collection (if not exists logic handled by server usually, but let's try creating)
    try {
        console.log(`Creating collection '${collectionName}'...`);
        // We assume L2 metric and 1024 dim
        await client.createCollection(collectionName, 1024, "l2");
    } catch (e) {
        console.log("Collection might already exist, proceeding...");
    }

    // 2. Initialize VectorStore
    const embeddings = new FakeEmbeddings(1024);
    const vectorStore = new HyperspaceStore(embeddings, {
        client,
        collectionName,
        enableDeduplication: true
    });

    // 3. Ingest Data
    console.log("📥 Ingesting notes...");
    const notes = [
        "Project Alpha meeting notes: Discussed the new UI design. Need to focus on dark mode.",
        "Shopping list: Milk, Eggs, Bread, and Coffee.",
        "Idea for blog: 'Why Euclidean Geometry Kills RAG' - discuss hyperbolic spaces.",
        "Reminder: Call Alice about the server migration on Tuesday.",
        "Tech stack: Rust, TypeScript, HyperspaceDB, React."
    ];

    const docs = notes.map((text, i) => new Document({
        pageContent: text,
        metadata: { source: "user_input", id: i, type: "note" }
    }));

    await vectorStore.addDocuments(docs);
    console.log("✅ Notes ingested successfully!");

    // 4. Perform Search
    console.log("\n🔍 Searching for 'geometry'...");
    // Even with fake embeddings, we verify the flow works. 
    // In real life, 'geometry' vector would be close to the blog post vector.
    const query = "geometry";
    const results = await vectorStore.similaritySearch(query, 2);

    console.log("\n📊 Search Results:");
    results.forEach((doc, i) => {
        console.log(`\n[Result ${i + 1}]`);
        console.log(`Content: "${doc.pageContent}"`);
        console.log(`Metadata:`, doc.metadata);
    });

    // 5. Cleanup (Optional)
    // await client.deleteCollection(collectionName);
    // client.close(); // SDK client might not have close? It does in my version.
    if (client.close) client.close();

    console.log("\n✨ Demo Complete!");
}

main().catch(console.error);
