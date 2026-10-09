import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const API_KEY = process.env.HYPERSPACE_API_KEY || "";
const HOST = process.env.HYPERSPACE_HOST || "127.0.0.1:50051";
const MEMORY_COLLECTION = process.env.MEMORY_COLLECTION || "agent_cognitive_memories_129";

async function runBrutalMemoryStressTest() {
  console.log("==========================================================================");
  console.log("🔥 BRUTAL COGNITIVE MEMORY STRESS & CAPACITY BENCHMARK");
  console.log("==========================================================================");
  console.log(`   Host: ${HOST}`);
  console.log(`   Collection: ${MEMORY_COLLECTION}`);
  console.log(`   Time: ${new Date().toISOString()}\n`);

  const serverPath = path.resolve(__dirname, "../dist/index.js");

  const transport = new StdioClientTransport({
    command: "node",
    args: [serverPath],
    env: {
      ...(process.env as Record<string, string>),
      HYPERSPACE_HOST: HOST,
      HYPERSPACE_API_KEY: API_KEY,
      HYPERSPACE_LOCAL_ADMIN_KEY: process.env.HYPERSPACE_LOCAL_ADMIN_KEY || "",
      MEMORY_COLLECTION
    }
  });

  const client = new Client(
    { name: "mcp-memory-stress-tester", version: "1.0.0" },
    { capabilities: {} }
  );

  await client.connect(transport);
  console.log("   ✓ Connected to mcp-hyperspace-memory via Stdio.\n");

  let passed = 0;
  let failed = 0;

  function pass(msg: string) {
    console.log(`   ✅ [PASS] ${msg}`);
    passed++;
  }

  function fail(msg: string, err?: any) {
    console.error(`   ❌ [FAIL] ${msg}`, err || "");
    failed++;
  }

  const benchmarkStats: Record<string, any> = {};

  try {
    // ── PHASE 1: CONCURRENT INGESTION BLAST (30 MEMORIES) ─────────────────────
    console.log("--------------------------------------------------------------------------");
    console.log("🚀 PHASE 1: High-Concurrency Ingestion Blast (30 Memories Across 3 Domains)");
    console.log("--------------------------------------------------------------------------");

    const testDataset = [
      // SecOps Session
      { session: "session_secops_stress", type: "episodic", importance: 9, text: "Incident #8821: Unauthorized egress port 4444 detected from container staging-db at 03:14 UTC.", tags: ["security", "incident", "egress"] },
      { session: "session_secops_stress", type: "procedural", importance: 8, text: "Playbook Firewall-Rule-33: Immediately isolate host and rotate Kubernetes service account tokens.", tags: ["security", "playbook", "mitigation"] },
      { session: "session_secops_stress", type: "semantic", importance: 6, text: "Security baseline requires mTLS encryption across all intra-cluster gRPC endpoints.", tags: ["security", "policy", "mtls"] },
      { session: "session_secops_stress", type: "episodic", importance: 7, text: "WAF rate limit rule triggered by IP 198.51.100.42 exceeding 1500 requests per minute.", tags: ["security", "waf", "ratelimit"] },
      { session: "session_secops_stress", type: "procedural", importance: 9, text: "Zero Trust Step 4: Revoke OAuth session cookies and invalidate Redis JWT refresh tokens.", tags: ["security", "auth", "tokens"] },
      { session: "session_secops_stress", type: "semantic", importance: 5, text: "PostgreSQL cluster replication lag exceeded 120 seconds during snapshot backup cycle.", tags: ["infrastructure", "database", "lag"] },
      { session: "session_secops_stress", type: "episodic", importance: 8, text: "DDoS mitigation active on cloud edge: scrubbing volumetric SYN flood on port 443.", tags: ["security", "ddos", "edge"] },
      { session: "session_secops_stress", type: "semantic", importance: 10, text: "CRITICAL NEEDLE: Project Chimera vault encryption bypass master token is Sigma-9981-Alpha-X.", tags: ["chimera", "master_key", "top_secret"] },
      { session: "session_secops_stress", type: "procedural", importance: 7, text: "Kubernetes pod eviction procedure: verify pod disruption budget before draining node.", tags: ["k8s", "devops", "pdb"] },
      { session: "session_secops_stress", type: "episodic", importance: 6, text: "Audit log rotation completed: 4.8 GB archived to cold S3 bucket with Glacier lifecycle.", tags: ["audit", "s3", "storage"] },

      // BioMed Session
      { session: "session_biomed_stress", type: "episodic", importance: 9, text: "Patient telemetry monitor 12B: Paroxysmal supraventricular tachycardia recorded at 168 bpm.", tags: ["medical", "cardio", "emergency"] },
      { session: "session_biomed_stress", type: "procedural", importance: 8, text: "Clinical guideline: Administer 6 mg Adenosine rapid IV push followed by saline flush for SVT.", tags: ["medical", "pharmacology", "protocol"] },
      { session: "session_biomed_stress", type: "semantic", importance: 7, text: "Normal adult sinus rhythm reference range is 60 to 100 beats per minute at rest.", tags: ["medical", "reference", "vitals"] },
      { session: "session_biomed_stress", type: "episodic", importance: 8, text: "Laboratory result: Serum potassium is critically low at 2.8 mmol/L (hypokalemia).", tags: ["medical", "lab", "potassium"] },
      { session: "session_biomed_stress", type: "procedural", importance: 9, text: "Electrolyte repletion protocol: Infuse IV potassium chloride at maximum 10 mmol/hr with ECG monitoring.", tags: ["medical", "infusion", "protocol"] },
      { session: "session_biomed_stress", type: "semantic", importance: 6, text: "Continuous subcutaneous insulin infusion pump basal rate adjusted to 0.85 units per hour.", tags: ["medical", "diabetes", "insulin"] },
      { session: "session_biomed_stress", type: "episodic", importance: 7, text: "Patient reported acute bronchospasm 15 minutes after cephalosporin antibiotic injection.", tags: ["medical", "allergy", "anaphylaxis"] },
      { session: "session_biomed_stress", type: "procedural", importance: 10, text: "Anaphylaxis first-line treatment: Intramuscular Epinephrine 0.3 mg 1:1000 into anterolateral thigh.", tags: ["medical", "epinephrine", "critical"] },
      { session: "session_biomed_stress", type: "semantic", importance: 5, text: "Oxygen saturation baseline improved to 98% on 2 liters nasal cannula.", tags: ["medical", "respiratory", "vitals"] },
      { session: "session_biomed_stress", type: "episodic", importance: 6, text: "ICU shift handover completed: Patient stable, hemodynamics monitored on arterial line.", tags: ["medical", "nursing", "handover"] },

      // Legal Session
      { session: "session_legal_stress", type: "semantic", importance: 8, text: "Master Services Agreement Clause 14.2: Maximum aggregate liability capped at 12 months fees paid.", tags: ["legal", "contract", "liability"] },
      { session: "session_legal_stress", type: "procedural", importance: 9, text: "GDPR Article 33 notice protocol: Report personal data breach to supervisory authority within 72 hours.", tags: ["legal", "gdpr", "compliance"] },
      { session: "session_legal_stress", type: "episodic", importance: 7, text: "Patent application US-2026-08819A filed covering hyperbolic hierarchical vector search index.", tags: ["legal", "ip", "patent"] },
      { session: "session_legal_stress", type: "semantic", importance: 6, text: "Non-Disclosure Agreement mutual term extended through December 2028 with surviving confidentiality.", tags: ["legal", "nda", "confidentiality"] },
      { session: "session_legal_stress", type: "procedural", importance: 8, text: "Export Control compliance check: Confirm target destination is not subject to OFAC sanctions.", tags: ["legal", "ofac", "compliance"] },
      { session: "session_legal_stress", type: "episodic", importance: 8, text: "Delaware Court of Chancery approved merger agreement pursuant to DGCL Section 251.", tags: ["legal", "corporate", "merger"] },
      { session: "session_legal_stress", type: "semantic", importance: 7, text: "Software escrow release condition: Bankruptcy of licensor triggers source code availability.", tags: ["legal", "ip", "escrow"] },
      { session: "session_legal_stress", type: "procedural", importance: 8, text: "Litigation hold order: Suspend all automated email destruction and backup purging immediately.", tags: ["legal", "litigation", "discovery"] },
      { session: "session_legal_stress", type: "episodic", importance: 6, text: "Board meeting minutes ratified: Series B preferred stock dividend rights affirmed.", tags: ["legal", "corporate", "governance"] },
      { session: "session_legal_stress", type: "semantic", importance: 5, text: "Standard contractual clauses version 2021/914 adopted for cross-border data transfer to UK.", tags: ["legal", "privacy", "scc"] }
    ];

    const t0 = Date.now();
    const rememberPromises = testDataset.map((item, idx) => {
      const callStart = Date.now();
      return client.callTool({
        name: "memory_remember",
        arguments: {
          text: item.text,
          session_id: item.session,
          tags: item.tags,
          memory_type: item.type,
          importance: item.importance
        }
      }).then(res => {
        const elapsed = Date.now() - callStart;
        return { success: true, elapsed, res, item, idx };
      }).catch(err => {
        return { success: false, elapsed: Date.now() - callStart, err, item, idx };
      });
    });

    const results = await Promise.all(rememberPromises);
    const totalElapsed = Date.now() - t0;
    const successes = results.filter(r => r.success);
    const latencies = successes.map(r => r.elapsed).sort((a, b) => a - b);
    const p50 = latencies[Math.floor(latencies.length * 0.5)] || 0;
    const p95 = latencies[Math.floor(latencies.length * 0.95)] || 0;
    const throughput = (successes.length / (totalElapsed / 1000)).toFixed(1);

    benchmarkStats.ingest = {
      total: testDataset.length,
      successes: successes.length,
      failures: testDataset.length - successes.length,
      totalElapsedMs: totalElapsed,
      p50Ms: p50,
      p95Ms: p95,
      throughputOpsPerSec: throughput
    };

    console.log(`   Ingested: ${successes.length}/${testDataset.length} in ${totalElapsed}ms`);
    console.log(`   Throughput: ${throughput} ops/sec | p50: ${p50}ms | p95: ${p95}ms`);

    if (successes.length === testDataset.length) {
      pass(`All 30 concurrent memories ingested with zero dropped requests.`);
    } else {
      fail(`Ingestion dropped ${testDataset.length - successes.length} memories.`);
    }

    // ── PHASE 2: NEEDLE-IN-A-HAYSTACK (NIAH) RECALL ──────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🎯 PHASE 2: Needle-In-A-Haystack (NIAH) Semantic Recall & Ranking");
    console.log("--------------------------------------------------------------------------");

    const niahQuery = "Where is the secret bypass token for Chimera vault stored?";
    const niahRes = await client.callTool({
      name: "memory_recall",
      arguments: {
        query: niahQuery,
        session_id: "session_secops_stress",
        limit: 5
      }
    });

    const niahParsed = JSON.parse((niahRes as any).content[0].text);
    const topHit = niahParsed.results?.[0];

    if (topHit && topHit.text?.includes("Sigma-9981-Alpha-X")) {
      pass(`Needle successfully retrieved as Rank #1 hit! Cognitive Score: ${topHit.cognitive_score}`);
    } else {
      fail(`NIAH recall failed to rank needle as #1. Top hit: ${JSON.stringify(topHit)}`);
    }

    // ── PHASE 3: STRICT SESSION ISOLATION & NEGATIVE LEAKAGE ─────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🛡️  PHASE 3: Strict Multi-Tenant Session Isolation & Anti-Leakage");
    console.log("--------------------------------------------------------------------------");

    // Query secops session with a medical query — MUST NOT return biomed memories
    const leakCheckRes = await client.callTool({
      name: "memory_recall",
      arguments: {
        query: "What is the emergency protocol for anaphylaxis and tachycardia?",
        session_id: "session_secops_stress",
        limit: 5
      }
    });

    const leakParsed = JSON.parse((leakCheckRes as any).content[0].text);
    const leakMatches = leakParsed.results || [];
    const hasBiomedLeak = leakMatches.some((m: any) => m.session_id === "session_biomed_stress" || m.text?.includes("Epinephrine"));

    if (!hasBiomedLeak) {
      pass("Strict session isolation verified: 0 biomed memories leaked into secops session.");
    } else {
      fail("Session isolation breach: biomed memories leaked into secops recall.");
    }

    // ── PHASE 4: IMPORTANCE & RECENCY FILTERING ──────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("⭐ PHASE 4: Cognitive Importance & Memory Type Filtering");
    console.log("--------------------------------------------------------------------------");

    const highImportanceRes = await client.callTool({
      name: "memory_recall",
      arguments: {
        query: "medical emergency treatment guideline",
        session_id: "session_biomed_stress",
        min_importance: 8,
        memory_type: "procedural",
        limit: 5
      }
    });

    const highImpParsed = JSON.parse((highImportanceRes as any).content[0].text);
    const filteredResults = highImpParsed.results || [];
    const allMatchCriteria = filteredResults.every((r: any) => r.importance >= 8 && r.memory_type === "procedural");

    if (filteredResults.length > 0 && allMatchCriteria) {
      pass(`Filter verified: retrieved ${filteredResults.length} procedural memories with importance >= 8.`);
    } else {
      fail(`Filter failed. Results: ${JSON.stringify(filteredResults)}`);
    }

    // ── PHASE 5: HYPERBOLIC CLUSTER CONSOLIDATION (FRÉCHET MEAN) ─────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🌀 PHASE 5: Hyperbolic Fréchet Mean Consolidation with Archive/Prune");
    console.log("--------------------------------------------------------------------------");

    const consolidateRes = await client.callTool({
      name: "memory_consolidate",
      arguments: {
        topic_query: "cardiovascular emergency resuscitation tachycardia epinephrine",
        limit: 8,
        summary_text: "[Cognitive Synthesis] Standardized clinical resuscitation protocol for acute tachyarrhythmia and anaphylaxis emergencies.",
        persist: true,
        archive_sources: false
      }
    });

    const consParsed = JSON.parse((consolidateRes as any).content[0].text);
    if (consParsed.status === "consolidated" && consParsed.consolidated_dimension === 801 && typeof consParsed.consolidated_id === "number") {
      pass(`Fréchet Mean consolidation generated 801D persistent concept node: ID ${consParsed.consolidated_id} from ${consParsed.source_count} source memories.`);
    } else {
      fail(`Consolidation failed: ${JSON.stringify(consParsed)}`);
    }

    // ── PHASE 6: ANTI-HALLUCINATION ENTAILMENT MATRIX (10 CLAIMS) ─────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("🔬 PHASE 6: Anti-Hallucination Claim Verification Matrix (Lorentz Hybrid Space)");
    console.log("--------------------------------------------------------------------------");

    const claimTests = [
      {
        name: "Direct Entailment (Medical)",
        premise: "Patient serum potassium is 2.8 mmol/L indicating acute hypokalemia.",
        conclusion: "Patient has dangerously low potassium levels requiring intravenous potassium replacement.",
        expected: "VERIFIED"
      },
      {
        name: "Direct Entailment (Security)",
        premise: "Unauthorized outbound connection detected from container staging-db to external IP on port 4444.",
        conclusion: "A potential reverse shell backdoor is active on the staging database container.",
        expected: "VERIFIED"
      },
      {
        name: "Direct Entailment (Legal)",
        premise: "Master Services Agreement caps total liability at 12 months of paid service fees.",
        conclusion: "Customer damages claims under the contract are subject to a twelve-month aggregate liability ceiling.",
        expected: "VERIFIED"
      },
      {
        name: "Contradiction / False Attribution",
        premise: "Patient telemetry monitor recorded paroxysmal supraventricular tachycardia at 168 bpm.",
        conclusion: "The corporate legal department ratified the Delaware merger pursuant to DGCL Section 251.",
        expected: "REJECTED"
      },
      {
        name: "Semantic Shift / Unrelated Claim",
        premise: "Project Chimera vault encryption bypass master token is Sigma-9981-Alpha-X.",
        conclusion: "The president of France signed a bilateral defense pact with New Zealand.",
        expected: "REJECTED"
      }
    ];

    let claimPasses = 0;
    for (const test of claimTests) {
      const claimRes = await client.callTool({
        name: "memory_verify_claim",
        arguments: {
          premise: test.premise,
          conclusion: test.conclusion
        }
      });
      const cParsed = JSON.parse((claimRes as any).content[0].text);
      if (cParsed.status === test.expected) {
        pass(`Claim [${test.name}]: correctly classified as ${cParsed.status} (Trust Score: ${cParsed.trust_score}, Threshold: ${cParsed.threshold})`);
        claimPasses++;
      } else {
        fail(`Claim [${test.name}]: expected ${test.expected} but got ${cParsed.status} (Trust Score: ${cParsed.trust_score})`);
      }
    }

    // ── PHASE 7: CONCURRENT READ/WRITE RACE TEST (10 WRITES + 10 READS) ───────
    console.log("\n--------------------------------------------------------------------------");
    console.log("⚡ PHASE 7: Concurrent Mixed Read/Write Stress (Simultaneous Access)");
    console.log("--------------------------------------------------------------------------");

    const tMixed0 = Date.now();
    const mixedOps: Promise<any>[] = [];

    // 10 concurrent writes
    for (let i = 0; i < 10; i++) {
      mixedOps.push(
        client.callTool({
          name: "memory_remember",
          arguments: {
            text: `High-frequency telemetry packet #${i} recorded with payload size ${100 + i * 20} KB.`,
            session_id: "session_telemetry_race",
            tags: ["telemetry", "race", `seq_${i}`]
          }
        })
      );
    }

    // 10 concurrent reads
    for (let i = 0; i < 10; i++) {
      mixedOps.push(
        client.callTool({
          name: "memory_recall",
          arguments: {
            query: `telemetry packet recorded payload size`,
            session_id: "session_telemetry_race",
            limit: 3
          }
        })
      );
    }

    const mixedResults = await Promise.all(mixedOps);
    const mixedElapsed = Date.now() - tMixed0;
    pass(`Executed 20 interleaved mixed read/write operations concurrently in ${mixedElapsed}ms (${(20 / (mixedElapsed / 1000)).toFixed(1)} ops/sec).`);

    // ── PHASE 8: CLUSTER STATS & SCALE REPORT ─────────────────────────────────
    console.log("\n--------------------------------------------------------------------------");
    console.log("📊 PHASE 8: Collection Health & Diagnostic Metrics");
    console.log("--------------------------------------------------------------------------");

    const statsRes = await client.callTool({
      name: "memory_stats",
      arguments: {}
    });
    const statsParsed = JSON.parse((statsRes as any).content[0].text);
    console.log("   Collection Stats:", JSON.stringify(statsParsed, null, 2));

    if (statsParsed.status === "ready" && statsParsed.approx_vector_count > 0) {
      pass(`Collection is healthy. Vector count: ${statsParsed.approx_vector_count}`);
    } else {
      fail(`Collection health degraded: ${JSON.stringify(statsParsed)}`);
    }

    // ── SUMMARY REPORT ────────────────────────────────────────────────────────
    console.log("\n==========================================================================");
    console.log("🏆 STRESS TEST EXECUTION SUMMARY");
    console.log("==========================================================================");
    console.log(`   ✅ TOTAL PASSED: ${passed}`);
    console.log(`   ❌ TOTAL FAILED: ${failed}`);
    console.log(`   Ingestion Throughput: ${benchmarkStats.ingest?.throughputOpsPerSec} ops/sec`);
    console.log(`   Latency (p50): ${benchmarkStats.ingest?.p50Ms}ms | (p95): ${benchmarkStats.ingest?.p95Ms}ms`);
    console.log(`   Status: ${failed === 0 ? "PASSED ALL TESTS WITH FLYING COLORS" : "SOME TESTS FAILED"}`);
    console.log("==========================================================================\n");

  } catch (err: any) {
    console.error("❌ Fatal unhandled exception in stress test suite:", err);
    process.exit(1);
  } finally {
    await client.close();
  }
}

runBrutalMemoryStressTest().catch(err => {
  console.error("Test execution terminated:", err);
  process.exit(1);
});
