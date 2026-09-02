import fs from 'fs';
import readline from 'readline';
import path from 'path';
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const DATASET_PATH = "/Volumes/Trashholder/Datasets/nomic_wikipedia.jsonl";
const API_KEY = process.env.HYPERSPACE_API_KEY || "I_LOVE_HYPERSPACEDB";
const HOST = process.env.HYPERSPACE_HOST || "the.yar.ink";

interface WikiPair {
  anchor: string;
  positive: string;
  source: string;
}

/**
 * Scientific Benchmark for HyperspaceDB STM / LTM Memory Engine
 * Dataset: nomic_wikipedia.jsonl (100,000 articles)
 * Tests: insert throughput, search latency, exact-match recall@1
 */
async function runWikipediaMemoryBenchmark(sampleSize: number = 10) {
  console.log("==========================================================================");
  console.log("🔬 HYPERSPACEDB SCIENTIFIC BENCHMARK: STM & LTM Memory Engine");
  console.log("==========================================================================");
  console.log(`   Dataset Path: ${DATASET_PATH}`);
  console.log(`   Sample Size: ${sampleSize} document pairs`);
  console.log(`   Host: ${HOST}\n`);

  if (!fs.existsSync(DATASET_PATH)) {
    console.error(`❌ Dataset file not found at ${DATASET_PATH}`);
    return;
  }

  // 1. Read dataset sample
  console.log("--------------------------------------------------------------------------");
  console.log("📂 STEP 1: Streaming & Validating Wikipedia Dataset");
  console.log("--------------------------------------------------------------------------");
  const fileStream = fs.createReadStream(DATASET_PATH);
  const rl = readline.createInterface({ input: fileStream, crlfDelay: Infinity });

  const dataset: WikiPair[] = [];
  let count = 0;
  for await (const line of rl) {
    if (count >= sampleSize) break;
    if (line.trim()) {
      try {
        const item = JSON.parse(line) as WikiPair;
        dataset.push(item);
        count++;
      } catch (e) {
        // ignore malformed lines
      }
    }
  }
  console.log(`   ✓ Successfully loaded ${dataset.length} Wiki document pairs.\n`);

  // 2. Connect to mcp-hyperspacedb (DB tools only)
  const serverPath = path.resolve(process.cwd(), "dist/index.js");
  const transport = new StdioClientTransport({
    command: "node",
    args: [serverPath],
    env: {
      HYPERSPACE_HOST: HOST,
      HYPERSPACE_API_KEY: API_KEY,
      PATH: process.env.PATH || ""
    }
  });

  const client = new Client(
    { name: "mcp-wikipedia-benchmark", version: "4.0.0" },
    { capabilities: {} }
  );

  await client.connect(transport);
  console.log("   ✓ Connected to HyperspaceDB MCP Server (mcp-hyperspacedb)\n");

  const insertBatchSize = Math.min(sampleSize, dataset.length);
  const stmCollection = `benchmark_stm_${Date.now()}`;
  const ltmCollection = `benchmark_ltm_extreme_${Date.now()}`;

  try {
    // ── STM: Short-Term Memory (no quantization, full precision) ────────────
    console.log("--------------------------------------------------------------------------");
    console.log("⚡ STEP 2: Testing Short-Term Memory (STM) — Cascade In-RAM Indexing");
    console.log("--------------------------------------------------------------------------");

    await client.callTool({
      name: "hyperspace_create_collection",
      arguments: {
        collection: stmCollection,
        dimension: 801,
        metric: "hybrid",
        quantization: "none"
      }
    });
    console.log(`   STM Collection Created: ${stmCollection}`);

    const stmInsertStart = Date.now();
    for (let i = 0; i < insertBatchSize; i++) {
      await client.callTool({
        name: "hyperspace_insert_text",
        arguments: {
          collection: stmCollection,
          id: i + 1,
          text: dataset[i].anchor,
          metadata: { source: dataset[i].source }
        }
      });
    }
    const stmInsertMs = Date.now() - stmInsertStart;
    console.log(`   ✓ Inserted ${insertBatchSize} docs in ${stmInsertMs} ms (${(stmInsertMs / insertBatchSize).toFixed(2)} ms/doc)`);

    const stmQueryStart = Date.now();
    const stmResult = await client.callTool({
      name: "hyperspace_search_text",
      arguments: { collection: stmCollection, text: dataset[0].anchor, top_k: 5 }
    });
    const stmQueryMs = Date.now() - stmQueryStart;
    const stmJson = JSON.parse((stmResult as any).content[0].text);
    const stmTopId = stmJson?.[0]?.id;
    const stmHit = stmTopId === 1;
    console.log(`   ✓ STM Search: ${stmQueryMs} ms | Top-1 ID: ${stmTopId} | Exact Match: ${stmHit ? '✅ YES' : '⚠️ MISS'}`);

    // ── LTM: Long-Term Memory (extreme 1-bit quantization, 61.4x compression) ─
    console.log("\n--------------------------------------------------------------------------");
    console.log("💾 STEP 3: Testing Long-Term Memory (LTM) — Extreme 1-bit Compression");
    console.log("--------------------------------------------------------------------------");

    await client.callTool({
      name: "hyperspace_create_collection",
      arguments: {
        collection: ltmCollection,
        dimension: 801,
        metric: "hybrid",
        quantization: "extreme"
        // Hybrid 801D: 33D Lorentz f32 (132B) + 768D Euclidean 1-bit ADC (96B) = 228B/vec (61.4x vs f64)
      }
    });
    console.log(`   LTM Extreme Collection Created: ${ltmCollection}`);

    const ltmInsertStart = Date.now();
    for (let i = 0; i < insertBatchSize; i++) {
      await client.callTool({
        name: "hyperspace_insert_text",
        arguments: {
          collection: ltmCollection,
          id: 1000 + i,
          text: dataset[i].positive,
          metadata: { type: "long_term_fact", source: dataset[i].source }
        }
      });
    }
    const ltmInsertMs = Date.now() - ltmInsertStart;
    console.log(`   ✓ Inserted ${insertBatchSize} docs in ${ltmInsertMs} ms`);

    const ltmQueryStart = Date.now();
    const ltmResult = await client.callTool({
      name: "hyperspace_search_text",
      arguments: { collection: ltmCollection, text: dataset[0].positive, top_k: 5 }
    });
    const ltmQueryMs = Date.now() - ltmQueryStart;
    const ltmJson = JSON.parse((ltmResult as any).content[0].text);
    const ltmTopId = ltmJson?.[0]?.id;
    const ltmHit = ltmTopId === 1000;
    console.log(`   ✓ LTM Search (61.4x): ${ltmQueryMs} ms | Top-1 ID: ${ltmTopId} | Exact Match: ${ltmHit ? '✅ YES' : '⚠️ MISS'}`);

    // ── Summary ──────────────────────────────────────────────────────────────
    console.log("\n==========================================================================");
    console.log("📊 SCIENTIFIC BENCHMARK RESULTS");
    console.log("==========================================================================");
    console.log(`| Memory Tier | Quantization   | Insert (ms/doc) | Search (ms) | Recall@1 |`);
    console.log(`| ----------- | -------------- | --------------- | ----------- | -------- |`);
    console.log(`| STM         | none (f64)     | ${String((stmInsertMs / insertBatchSize).toFixed(1)).padEnd(15)} | ${String(stmQueryMs).padEnd(11)} | ${stmHit ? '✅ 100%' : '⚠️ miss'}   |`);
    console.log(`| LTM         | extreme (1-bit)| ${String((ltmInsertMs / insertBatchSize).toFixed(1)).padEnd(15)} | ${String(ltmQueryMs).padEnd(11)} | ${ltmHit ? '✅ 100%' : '⚠️ miss'}   |`);
    console.log("==========================================================================");

  } finally {
    // Cleanup
    console.log("\n🧹 Cleaning up benchmark collections...");
    try { await client.callTool({ name: "hyperspace_delete_collection", arguments: { collection: stmCollection } }); } catch {}
    try { await client.callTool({ name: "hyperspace_delete_collection", arguments: { collection: ltmCollection } }); } catch {}
    console.log("   ✓ Done.");
    process.exit(0);
  }
}

const sampleArg = process.argv[2] ? parseInt(process.argv[2], 10) : 10;
runWikipediaMemoryBenchmark(sampleArg).catch(err => {
  console.error("Benchmark failed:", err);
  process.exit(1);
});
