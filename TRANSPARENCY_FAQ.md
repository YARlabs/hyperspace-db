# HyperspaceDB Knowledge-Augmented Generation (KAG) Transparency Note & Architecture FAQ

This document provides transparency into the architecture, capabilities, intended use cases, limitations, empirical evaluation, and economic footprint of **HyperspaceDB's Native Vector-Graph (KAG) Engine**, modeled after the Responsible AI (RAI) Transparency standard.

---

## 1. What is HyperspaceDB KAG?

**HyperspaceDB KAG (Knowledge-Augmented Generation)** is an in-engine, unified spatial-vector and property-graph database subsystem. Unlike traditional architectures that orchestrate separate vector databases (e.g., Pinecone, Qdrant) and graph databases (e.g., Neo4j, Amazon Neptune) over inter-process network protocols, HyperspaceDB internalizes topological graph representation directly inside its Thread-per-Core LSM storage tier.

### Core Architectural Pillars:
* **Packed Property Compressed Sparse Row (CSR)**: 12-byte binary edge encoding (`CompactEdge`) stored in contiguous memory arrays (`graph.csr`), delivering **~5 µs** reachability traversal.
* **Dense Node Translation**: Lock-free bidirectional mapping between 32-bit user hash IDs and dense $0 \dots N$ internal CSR nodes (`DashMap<u32, u32>`), consuming under **1 MB RAM for 20,000+ edges**.
* **Declarative Metadata Induction**: $O(N)$ automatic relationship generation (`path_hierarchy`, `foreign_key`, `exact_match`) without external LLM entity extraction passes ($0 token expense).
* **Minkowski Spacetime Subsumption**: 33-dimensional Lorentzian hyperbolic geometry ($\mathbb{R}^{1,32}$) modeling hierarchical parent-child inclusions as causal light cones ($\Delta s^2 \le 0$).
* **Tribunal Anti-Hallucination Router**: Real-time geometric Lyapunov stability evaluation verifying that generative model outputs remain within the causal geodesic boundary of the source context.

---

## 2. What can HyperspaceDB KAG do?

| Capability | Description |
| :--- | :--- |
| **Unified Vector-Graph Traversal** | Seeds retrieval via high-dimensional vector similarity (Cosine, L2, Poincaré, Lorentz, Hybrid) and immediately expands the result through multi-hop topological graph edges in the same CPU cache line. |
| **SIMD Bitmask Pre-Filtering** | Generates an in-engine reachability bitmask during graph traversal to evaluate vector candidate distances only on connected nodes, achieving a **110× speedup (1.02 µs vs 111.91 µs)** over naive post-filtering. |
| **Instant Real-Time Edge Induction** | Materializes thousands of typed, weighted edges per millisecond directly from document metadata without manual Cypher scripting. |
| **Deterministic Prerequisite Retrieval** | Traverses 100% of explicit policy, compliance, or causal containment dependencies (`DEPENDS_ON`, `SUBORDINATE_TO`, `MANDATORY_PRECONDITION`), eliminating structural omissions in RAG pipelines. |
| **Out-of-Core Graph Compaction** | Persists topology to immutable `.csr` snapshots on NVMe/SSD, allowing streaming traversals on commodity hardware with minimal RAM overhead. |

---

## 3. What are the intended uses of HyperspaceDB KAG?

### Primary Intended Uses:
1. **Autonomous Security Operations (SOC)**: Traversal of MITRE ATT&CK tactic trees, D3FEND countermeasure playbooks, and causal kill-chain forensics where missing a containment step causes breach failure.
2. **Regulatory & Statutory Compliance**: Verification of complex legal and financial corpora (SEC, CFTC, OSHA, FAR, FAA, DGCL) where subordinate clauses and statutory exceptions must be retrieved deterministically.
3. **Autonomous Robotics & Avionics**: Edge navigation and operational design domain (ODD) safety rules where decision latencies must stay strictly under **5 milliseconds**.
4. **Pharmaceutical & Medical Taxonomies**: Navigating FDA QSR quality standards, SNOMED-CT clinical hierarchies, and molecular reaction graphs without hallucinated steps.
5. **High-Throughput Enterprise RAG**: Production agent systems serving hundreds of queries per second without paying the latency or infrastructure tax of multi-database Frankenstein stacks.

### Out-of-Scope / Unintended Uses:
* **Unstructured Graph Discovery from Raw Unlabeled Text**: HyperspaceDB's declarative induction relies on document metadata or embeddings. It does not replace open-ended information extraction on unstructured text (though it seamlessly ingests graphs produced by external extractors).
* **Massive OLAP Batch Graph Analytics**: Not designed for graph-wide iterative algorithms (e.g., PageRank over a 10-billion-edge web crawl); optimized for low-latency localized multi-hop traversals ($\le 3$ hops) and sub-graph neighborhood extraction.

---

## 4. How was HyperspaceDB KAG evaluated?

Evaluation was conducted across a full cluster of **12 production domain playbooks** comprising **16,001 ingested vectors** and **18,402 active graph edges**:

### Empirical Benchmark Findings:
* **Causal Context Completeness**: Pure vector search (HNSW Cosine/Hybrid) achieved an average recall of only **22.1%** on connected rule trees, missing **77.9% of structural prerequisites**. Native KAG achieved **100.0% Context Recall** across all 12 collections with **zero lost steps**.
* **Graph Traversal Latency**: Native CSR bitset reachability resolved in **~5 µs** at the CPU core, and **2,363.9 µs (p50)** end-to-end over HTTP (including complete JSON payload reconstitution).
* **End-to-End Decision Latency**: Vector entry match + multi-hop graph expansion completed in **3.37 ms (p50)**.
* **Memory Density**: 18,402 edges consumed less than **1 MB of RAM** and **18 KB of disk storage** via 12-byte packed binary CSR representation.

---

## 5. What are the limitations of HyperspaceDB KAG? How can users minimize their impact?

| Limitation | Impact | Mitigation / Best Practice |
| :--- | :--- | :--- |
| **Idle Collection Sleep (Serverless Unloading)** | Inactive collections are offloaded to disk (`status: "idle"`), causing the first touch to pay a 10–50 ms warm-up penalty. | Keep high-frequency collections active, or configure background keep-alive probes via `/api/collections/{name}/stats`. |
| **Combinatorial Fanout on Dense Graphs** | Traversing highly interconnected clusters (e.g. 100+ edges per node) can cause breadth-first search queues to expand exponentially. | The engine bounds traversal via `HS_GRAPH_MAX_FANOUT_PER_NODE` (default: 64) and limits induction cliques to 16 members. |
| **Metadata Consistency Requirement** | Declarative induction relies on consistent metadata keys (`lorentz_hierarchy_path`, `category`, `technique_id`). | Enforce schema validation upon ingestion using `CollectionSchema` or use explicit `link_batch` API calls if metadata varies. |

---

## 6. Anti-Hallucination Guarantees: How Does KAG Prevent LLM Errors?

LLM hallucinations in enterprise RAG stem predominantly from **context truncation**: the vector search returns 2 or 3 loosely related chunks, but misses the crucial prerequisite or exception clause that lives 2 hops away in the policy manual.

HyperspaceDB employs a **two-tier defense mechanism**:
1. **Topological Prerequisite Guarantee**: KAG deterministically traverses all incoming and outgoing dependency edges (`DEPENDS_ON`, `REQUIRES`, `SUBORDINATE_TO`), ensuring the LLM prompt contains 100% of the operational procedure before generation begins.
2. **Tribunal Router Geometric Verification (`tribunal.rs`)**: Post-generation verification evaluates the generated response against the context embedding inside 33D Lorentzian space. If the generated output diverges beyond the Lyapunov stability limit or escapes the source document's causal light-cone, it is flagged as an ungrounded hallucination (`is_hallucination: true`).

---

## 7. Compute, Economic & Cost Transparency

| Architectural Model | Graph Construction Cost | Ingestion Time | Query Latency | Monthly Infrastructure |
| :--- | :---: | :---: | :---: | :---: |
| **Microsoft GraphRAG** | **$100 – $1,000+** (OpenAI LLM tokens) | Hours to days | 5 – 30+ seconds | High (Compute + API token fees) |
| **Frankenstein Stack (VectorDB + Neo4j)** | $0 (Manual Cypher ETL) | Minutes to hours | 40 – 120 ms | High (3 independent clusters) |
| **HyperspaceDB v4.1 (Native KAG)** | **$0** (In-engine $O(N)$ induction) | **2 – 15 milliseconds** | **3.37 milliseconds** | **Low (Single commodity binary)** |

### Environmental & Efficiency Impact:
* Eliminating inter-process HTTP/TCP hops between separate database clusters slashes CPU idle wait states by up to **85%**.
* In-engine 12-byte packed CSR edges require **16× less RAM** than JVM-based property graphs (12 bytes vs 200+ bytes per edge).
* Real-time telemetry is continuously reported via the built-in `EcoMonitor` agent on the Web Dashboard.

---

*Document Version: 1.0.0 (v4.1.0 Engine)*  
*Maintained by: YAR Labs Architecture & Safety Research Team*
