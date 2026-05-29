#!/usr/bin/env python3
import os
import sys
import time
import json
import hashlib
import numpy as np
import pandas as pd
import argparse
from tqdm import tqdm
from concurrent.futures import ThreadPoolExecutor

# Add python SDK to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdks", "python")))

# Database client imports
try:
    from hyperspace import HyperspaceClient
    HYPERSPACE_AVAILABLE = True
except ImportError:
    HYPERSPACE_AVAILABLE = False

try:
    from qdrant_client import QdrantClient
    from qdrant_client.models import Distance, PointStruct, VectorParams
    QDRANT_AVAILABLE = True
except ImportError:
    QDRANT_AVAILABLE = False

try:
    import chromadb
    from chromadb.config import Settings
    CHROMA_AVAILABLE = True
except ImportError:
    CHROMA_AVAILABLE = False

try:
    from pymilvus import connections, Collection, CollectionSchema, DataType, FieldSchema, utility
    MILVUS_AVAILABLE = True
except ImportError:
    MILVUS_AVAILABLE = False

try:
    import weaviate
    WEAVIATE_AVAILABLE = True
except ImportError:
    WEAVIATE_AVAILABLE = False

try:
    from sentence_transformers import SentenceTransformer
    EMBEDDER_AVAILABLE = True
except ImportError:
    EMBEDDER_AVAILABLE = False

try:
    from datasets import load_dataset
    DATASETS_AVAILABLE = True
except ImportError:
    DATASETS_AVAILABLE = False


class RAGStandardsEvaluator:
    def __init__(self, limit=1000, query_limit=200, model_name="all-MiniLM-L6-v2"):
        self.limit = limit
        self.query_limit = query_limit
        self.model_name = model_name
        
        # Load local embedder
        if EMBEDDER_AVAILABLE:
            print(f"⚙️ Loading local embedding model '{model_name}'...")
            self.model = SentenceTransformer(model_name)
            self.dim = self.model.get_sentence_embedding_dimension()
            print(f"✅ Embedder loaded. Dimension: {self.dim}")
        else:
            print("❌ sentence-transformers package is missing. Cannot proceed.")
            sys.exit(1)

    def load_domain_dataset(self, domain):
        print(f"\n📦 Loading Galileo RAGBench ({domain} subset)...")
        loaded_real = False
        corpus = []
        queries = []
        
        if DATASETS_AVAILABLE:
            try:
                # Attempt to stream dataset from Hugging Face
                dataset = load_dataset("rungalileo/ragbench", domain, split="test", streaming=True)
                
                passages_set = set()
                query_list = []
                
                print("   Streaming and building corpus...")
                count = 0
                for row in tqdm(dataset, desc="Ingesting rows"):
                    if count >= self.query_limit * 3:
                        break
                    
                    q_text = row["question"]
                    doc_list = row["documents"]
                    relevant_keys = row.get("all_relevant_sentence_keys", [])
                    
                    # Parse relevant document indices from keys (e.g. '0c' -> index 0)
                    relevant_indices = set()
                    for key in relevant_keys:
                        digits = []
                        for char in key:
                            if char.isdigit():
                                digits.append(char)
                            else:
                                break
                        if digits:
                            relevant_indices.add(int("".join(digits)))
                            
                    # If no relevant indices marked, assume first document is gold
                    if not relevant_indices and doc_list:
                        relevant_indices.add(0)
                        
                    gold_passages = []
                    for idx in relevant_indices:
                        if idx < len(doc_list):
                            gold_passages.append(doc_list[idx])
                            
                    # Add all passages to main corpus
                    for doc in doc_list:
                        passages_set.add(doc)
                        
                    query_list.append({
                        "question": q_text,
                        "gold_passages": gold_passages,
                        "all_passages": doc_list
                    })
                    count += 1
                
                corpus = list(passages_set)[:self.limit]
                corpus_set = set(corpus)
                
                queries = []
                for q in query_list:
                    valid_gold = [p for p in q["gold_passages"] if p in corpus_set]
                    if valid_gold and len(queries) < self.query_limit:
                        queries.append({
                            "question": q["question"],
                            "gold_passages": valid_gold
                        })
                
                if len(corpus) > 0 and len(queries) > 0:
                    loaded_real = True
                    print(f"   Successfully loaded {len(corpus)} unique documents and {len(queries)} RAG queries from Galileo HF Hub.")
            except Exception as e:
                print(f"⚠️ Could not load remote Hugging Face dataset (likely offline/timeout): {e}")
                
        if not loaded_real:
            print("   Falling back to High-Fidelity local simulated RAGBench dataset...")
            corpus, queries = self.generate_simulated_dataset(domain)
            
        return corpus, queries

    def generate_simulated_dataset(self, domain):
        # High fidelity simulated RAGBench
        topics = {
            "covidqa": [
                ("interferon response", "Type I Interferons (IFN-a/b) are critical for viral clearance. Deficiencies in IFN receptors in dendritic cells facilitate norovirus persistence."),
                ("vaccine efficacy", "Clinical trials show the new mRNA vaccine achieves 94% efficacy against symptomatic infection and 99% against severe hospitalization."),
                ("norovirus replication", "Noroviruses replicate in the cytoplasm of enterocytes. Strong induction of interferon-stimulated genes (ISGs) prevents prolonged inflammation.")
            ],
            "finqa": [
                ("revenue growth", "The company reported a 14% year-over-year revenue increase, driven primarily by enterprise software cloud subscriptions."),
                ("operating margin", "Operating margin expanded by 240 basis points to 28.5%, reflecting strong cost controls and higher hardware pricing power."),
                ("cash flow", "Free cash flow reached $4.2 billion, up from $3.1 billion in the prior fiscal year, enabling a major stock buyback.")
            ],
            "cuad": [
                ("indemnification clause", "The seller agrees to indemnify and hold harmless the buyer from any liabilities, damages, or claims arising from patent infringement."),
                ("force majeure", "Neither party shall be liable for failures to perform resulting from acts of God, strikes, natural disasters, or government orders."),
                ("termination terms", "This agreement may be terminated by either party upon 30 days written notice in the event of a material breach.")
            ],
            "msmarco": [
                ("customer support", "To reset your password, click on the profile icon, select settings, click on security, and choose update password options."),
                ("device pairing", "Enable Bluetooth on both devices, select discoverable mode, and enter the 6-digit pin displayed on the primary console screen."),
                ("shipping policy", "Standard shipping takes 3-5 business days. Express shipping is delivered within 1-2 business days with flat rate pricing.")
            ]
        }
        
        domain_topics = topics.get(domain, topics["covidqa"])
        passages = []
        queries = []
        
        # Expand corpus with topic distractors
        for i in range(self.limit):
            topic = domain_topics[i % len(domain_topics)]
            passage_text = f"Title: {domain.upper()} Research Paper {i}\nPassage: {topic[1]} Supplementary data indicates this study represents cohort {i}."
            passages.append(passage_text)
            
        # Build queries
        queries_raw = {
            "covidqa": [
                ("How do type I interferons contribute to viral clearance?", "Type I Interferons (IFN-a/b) are critical for viral clearance"),
                ("What is the efficacy rate of the new mRNA vaccine?", "clinical trials show the new mRNA vaccine achieves 94% efficacy"),
                ("Where do noroviruses replicate in host cells?", "Noroviruses replicate in the cytoplasm of enterocytes")
            ],
            "finqa": [
                ("What was the revenue growth driver for the company?", "enterprise software cloud subscriptions"),
                ("How much did the operating margin expand?", "margin expanded by 240 basis points"),
                ("What was the free cash flow for the fiscal year?", "Free cash flow reached $4.2 billion")
            ],
            "cuad": [
                ("Who is held harmless under the indemnification clause?", "indemnify and hold harmless the buyer"),
                ("What causes fall under force majeure terms?", "acts of God, strikes, natural disasters"),
                ("How many days notice is required for termination?", "terminated by either party upon 30 days")
            ],
            "msmarco": [
                ("What are the steps to reset your account password?", "click on the profile icon, select settings"),
                ("How do you pair two Bluetooth devices?", "select discoverable mode, and enter the 6-digit pin"),
                ("What is the shipping time for standard orders?", "Standard shipping takes 3-5 business days")
            ]
        }
        
        domain_queries = queries_raw.get(domain, queries_raw["covidqa"])
        for q_text, gold_snippet in domain_queries:
            gold_passages = []
            for p in passages:
                if gold_snippet.lower() in p.lower():
                    gold_passages.append(p)
            queries.append({
                "question": q_text,
                "gold_passages": gold_passages
            })
            
        return passages, queries[:self.query_limit]

    def run_eval(self, db_name, corpus, queries, setup_fn, insert_fn, search_fn, cleanup_fn):
        print(f"\n🚀 Evaluating Database: {db_name}")
        
        # 1. Setup
        db_context = setup_fn()
        
        # 2. Embedding Corpus
        print("   Embedding corpus documents...")
        t0 = time.time()
        embeddings = self.model.encode(corpus, show_progress_bar=False, batch_size=64).tolist()
        embed_time = time.time() - t0
        print(f"   Embedded {len(corpus)} docs in {embed_time:.2f}s (Speed: {len(corpus)/embed_time:.1f} docs/s)")
        
        # 3. Inserting
        print("   Inserting into database...")
        t0 = time.time()
        insert_fn(db_context, embeddings, corpus)
        insert_time = time.time() - t0
        print(f"   Indexed in {insert_time:.2f}s (Ingest QPS: {len(corpus)/insert_time:.1f})")
        
        # Settle indexing
        time.sleep(2)
        
        # 4. Search and Accuracy Metrics
        print("   Embedding queries...")
        q_texts = [q["question"] for q in queries]
        q_embeddings = self.model.encode(q_texts, show_progress_bar=False).tolist()
        
        print("   Executing RAG retrieval queries...")
        latencies = []
        precisions = []
        recalls = []
        ndcgs = []
        mrrs = []
        context_precisions = []
        
        for i, q_emb in enumerate(q_embeddings):
            gold_set = set(queries[i]["gold_passages"])
            
            # Execute search
            t_start = time.time()
            hits = search_fn(db_context, q_emb, limit=10)
            latency = (time.time() - t_start) * 1000
            latencies.append(latency)
            
            retrieved = [h["text"] for h in hits]
            
            # ── Calculate Precision@10 & Recall@10 ──
            hits_in_gold = [p for p in retrieved if p in gold_set]
            precision = len(hits_in_gold) / len(retrieved) if retrieved else 0.0
            recall = len(hits_in_gold) / len(gold_set) if gold_set else 0.0
            precisions.append(precision)
            recalls.append(recall)
            
            # ── Calculate MRR ──
            mrr = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    mrr = 1.0 / (rank + 1)
                    break
            mrrs.append(mrr)
            
            # ── Calculate NDCG@10 ──
            dcg = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    dcg += 1.0 / np.log2(rank + 2)
            ideal_hits = min(10, len(gold_set))
            idcg = sum(1.0 / np.log2(r + 2) for r in range(ideal_hits))
            ndcgs.append(dcg / idcg if idcg > 0.0 else 0.0)
            
            # ── Calculate Context Precision ──
            num_relevant = 0
            precision_sum = 0.0
            for rank, p in enumerate(retrieved):
                if p in gold_set:
                    num_relevant += 1
                    precision_sum += num_relevant / (rank + 1)
            context_precision = precision_sum / num_relevant if num_relevant > 0 else 0.0
            context_precisions.append(context_precision)
            
        avg_precision = np.mean(precisions)
        avg_recall = np.mean(recalls)
        avg_ndcg = np.mean(ndcgs)
        avg_mrr = np.mean(mrrs)
        avg_context_precision = np.mean(context_precisions)
        
        p50 = np.percentile(latencies, 50)
        p99 = np.percentile(latencies, 99)
        overall_qps = len(queries) / (sum(latencies) / 1000.0)
        
        print(f"📊 Results for {db_name}:")
        print(f"   Precision@10: {avg_precision:.1%}")
        print(f"   Recall@10:    {avg_recall:.1%}")
        print(f"   NDCG@10:      {avg_ndcg:.3f}")
        print(f"   MRR:          {avg_mrr:.3f}")
        print(f"   Context Prec: {avg_context_precision:.1%}")
        print(f"   Latency p50:  {p50:.2f}ms | p99: {p99:.2f}ms (QPS: {overall_qps:.0f})")
        
        # 5. Cleanup
        cleanup_fn(db_context)
        
        return {
            "db_name": db_name,
            "precision": avg_precision,
            "recall": avg_recall,
            "ndcg": avg_ndcg,
            "mrr": avg_mrr,
            "context_precision": avg_context_precision,
            "p50": p50,
            "p99": p99,
            "qps": overall_qps,
            "insert_qps": len(corpus) / insert_time
        }

    def generate_html_report(self, domain_results):
        html_path = "RAG_STANDARDS_REPORT.html"
        
        # Render Javascript and Tabs
        tabs_html = ""
        tables_html = ""
        js_charts_data = {}
        
        for idx, (domain, results) in enumerate(domain_results.items()):
            active_class = "active" if idx == 0 else ""
            display_style = "block" if idx == 0 else "none"
            
            tabs_html += f"""
            <button class="tab-link {active_class}" onclick="openDomain(event, '{domain}')">{domain.upper()}</button>
            """
            
            rows = ""
            js_labels = []
            js_qps = []
            js_recall = []
            js_c_precision = []
            
            for r in results:
                js_labels.append(r["db_name"])
                js_qps.append(r["qps"])
                js_recall.append(r["recall"] * 100)
                js_c_precision.append(r["context_precision"] * 100)
                
                rows += f"""
                <tr>
                    <td style="font-weight: bold; color: #38bdf8;">{r['db_name']}</td>
                    <td>{r['insert_qps']:.0f}</td>
                    <td>{r['qps']:.0f}</td>
                    <td>{r['p50']:.2f} ms</td>
                    <td>{r['p99']:.2f} ms</td>
                    <td>{r['precision']:.1%}</td>
                    <td style="font-weight: bold; color: #10b981;">{r['recall']:.1%}</td>
                    <td>{r['ndcg']:.3f}</td>
                    <td style="font-weight: bold; color: #a78bfa;">{r['context_precision']:.1%}</td>
                </tr>
                """
                
            js_charts_data[domain] = {
                "labels": js_labels,
                "qps": js_qps,
                "recall": js_recall,
                "c_precision": js_c_precision
            }
            
            tables_html += f"""
            <div id="{domain}" class="tab-content" style="display: {display_style};">
                <div class="grid">
                    <div class="card">
                        <h2>Retrieval Throughput ({domain.upper()} QPS)</h2>
                        <canvas id="qpsChart_{domain}"></canvas>
                    </div>
                    <div class="card">
                        <h2>Retrieval Quality ({domain.upper()} Recall & Context Precision)</h2>
                        <canvas id="qualityChart_{domain}"></canvas>
                    </div>
                </div>

                <div class="card" style="overflow-x: auto;">
                    <h2>Performance Leaderboard ({domain.upper()})</h2>
                    <table>
                        <thead>
                            <tr>
                                <th>Database</th>
                                <th>Insert QPS</th>
                                <th>Search QPS</th>
                                <th>Latency p50</th>
                                <th>Latency p99</th>
                                <th>Precision@10</th>
                                <th>Recall@10</th>
                                <th>NDCG@10</th>
                                <th>Context Precision</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows}
                        </tbody>
                    </table>
                </div>
            </div>
            """
            
        html_content = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Standardized RAG Retrieval Benchmark Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        :root {{
            --bg: #0b0f19;
            --card-bg: #111827;
            --text: #f3f4f6;
            --accent: #38bdf8;
            --border: #1f2937;
        }}
        body {{
            font-family: 'Inter', sans-serif;
            background: var(--bg);
            color: var(--text);
            margin: 0;
            padding: 2rem;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}
        header {{
            text-align: center;
            margin-bottom: 2rem;
        }}
        h1 {{
            color: var(--accent);
            font-size: 2.5rem;
            margin-bottom: 0.5rem;
            text-shadow: 0 0 15px rgba(56, 189, 248, 0.3);
        }}
        p.subtitle {{
            color: #9ca3af;
            font-size: 1.1rem;
        }}
        .tab-bar {{
            display: flex;
            justify-content: center;
            gap: 1rem;
            margin-bottom: 2rem;
            border-bottom: 1px solid var(--border);
            padding-bottom: 1rem;
        }}
        .tab-link {{
            background: #1f2937;
            border: 1px solid var(--border);
            color: #9ca3af;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
            cursor: pointer;
            font-weight: 600;
            transition: all 0.2s ease;
        }}
        .tab-link.active, .tab-link:hover {{
            background: var(--accent);
            color: var(--bg);
            border-color: var(--accent);
            box-shadow: 0 0 10px rgba(56, 189, 248, 0.4);
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 2rem;
            margin-bottom: 3rem;
        }}
        .card {{
            background: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 1rem;
            padding: 1.5rem;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.2);
        }}
        .card h2 {{
            font-size: 1.2rem;
            margin-top: 0;
            color: #94a3b8;
            border-bottom: 1px solid var(--border);
            padding-bottom: 0.5rem;
        }}
        canvas {{
            max-height: 300px;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 1rem;
            border-radius: 0.5rem;
            overflow: hidden;
        }}
        th, td {{
            padding: 1rem;
            text-align: left;
            border-bottom: 1px solid var(--border);
        }}
        th {{
            background: #1f2937;
            color: #9ca3af;
            font-weight: 600;
        }}
        tr:hover {{
            background: rgba(31, 41, 55, 0.5);
        }}
        .badge {{
            background: rgba(56, 189, 248, 0.1);
            color: var(--accent);
            padding: 0.25rem 0.5rem;
            border-radius: 0.375rem;
            font-size: 0.85rem;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Standardized RAG Benchmark</h1>
            <p class="subtitle">Verifiable Retrieval Quality Evaluation across Galileo RAGBench domains</p>
        </header>

        <div class="tab-bar">
            {tabs_html}
        </div>

        {tables_html}
    </div>

    <script>
        function openDomain(evt, domainName) {{
            var i, tabcontent, tablinks;
            tabcontent = document.getElementsByClassName("tab-content");
            for (i = 0; i < tabcontent.length; i++) {{
                tabcontent[i].style.display = "none";
            }}
            tablinks = document.getElementsByClassName("tab-link");
            for (i = 0; i < tablinks.length; i++) {{
                tablinks[i].className = tablinks[i].className.replace(" active", "");
            }}
            document.getElementById(domainName).style.display = "block";
            evt.currentTarget.className += " active";
        }}

        const chartData = {json.dumps(js_charts_data)};
        
        // Render charts dynamically for all domains
        for (const [domain, data] of Object.entries(chartData)) {{
            // QPS Chart
            new Chart(document.getElementById('qpsChart_' + domain), {{
                type: 'bar',
                data: {{
                    labels: data.labels,
                    datasets: [{{
                        label: 'Query QPS',
                        data: data.qps,
                        backgroundColor: 'rgba(56, 189, 248, 0.5)',
                        borderColor: '#38bdf8',
                        borderWidth: 1
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }},
                        x: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }}
                    }}
                }}
            }});

            // Quality Chart
            new Chart(document.getElementById('qualityChart_' + domain), {{
                type: 'bar',
                data: {{
                    labels: data.labels,
                    datasets: [
                        {{
                            label: 'Recall@10 (%)',
                            data: data.recall,
                            backgroundColor: 'rgba(16, 185, 129, 0.5)',
                            borderColor: '#10b981',
                            borderWidth: 1
                        }},
                        {{
                            label: 'Context Precision (%)',
                            data: data.c_precision,
                            backgroundColor: 'rgba(167, 139, 250, 0.5)',
                            borderColor: '#a78bfa',
                            borderWidth: 1
                        }}
                    ]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{ min: 0, max: 100, grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }},
                        x: {{ grid: {{ color: '#1f2937' }}, ticks: {{ color: '#9ca3af' }} }}
                    }}
                }}
            }});
        }}
    </script>
</body>
</html>
"""
        with open(html_path, "w") as f:
            f.write(html_content)
        print(f"\n✅ Standardized RAG Master Report Generated: {os.path.abspath(html_path)}")


def main():
    parser = argparse.ArgumentParser(description="Standardized RAG Database Evaluator")
    parser.add_argument("--db", nargs="+", help="Specific DBs to test (hyperspace qdrant chroma milvus weaviate)")
    parser.add_argument("--limit", type=int, default=1000, help="Document corpus limit")
    parser.add_argument("--query-limit", type=int, default=200, help="RAG query limit")
    parser.add_argument("--domain", type=str, default="covidqa", help="Hugging Face Galileo RAGBench subset (covidqa, finqa, cuad, msmarco, or all)")
    args = parser.parse_args()

    evaluator = RAGStandardsEvaluator(
        limit=args.limit,
        query_limit=args.query_limit
    )
    
    if args.domain.lower() == "all":
        domains = ["covidqa", "finqa", "cuad", "msmarco"]
    else:
        domains = [args.domain]
        
    target_dbs = [d.lower() for d in args.db] if args.db else None
    
    # Domain mapped results
    domain_results = {}
    
    for dom in domains:
        print(f"\n================================================================================")
        print(f"🔥 STARTING EVALUATION SWEEP FOR DOMAIN: {dom.upper()}")
        print(f"================================================================================")
        corpus, queries = evaluator.load_domain_dataset(dom)
        
        results = []
        
        # ── HYPERSPACE ──
        if (not target_dbs or "hyperspace" in target_dbs) and HYPERSPACE_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                def hs_setup():
                    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.create_collection(coll_name, dimension=evaluator.dim, metric="cosine")
                    # Match high speed defaults by letting ef_search be configured
                    client.configure(ef_search=32, ef_construction=100, collection=coll_name)
                    return client
                
                def hs_ins(client, embs, texts):
                    ids = list(range(len(embs)))
                    metas = [{"text": t} for t in texts]
                    batch_size = 2000
                    for i in range(0, len(embs), batch_size):
                        client.batch_insert(
                            embs[i:i+batch_size],
                            ids[i:i+batch_size],
                            metas[i:i+batch_size],
                            collection=coll_name
                        )
                
                def hs_srch(client, q_emb, limit):
                    res = client.search(q_emb, top_k=limit, collection=coll_name)
                    return [{"text": h["metadata"]["text"]} for h in res]
                
                def hs_cleanup(client):
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.close()
                    
                res = evaluator.run_eval("HyperspaceDB", corpus, queries, hs_setup, hs_ins, hs_srch, hs_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Hyperspace evaluation failed: {e}")

        # ── HYPERSPACE (WAVE) ──
        if (not target_dbs or "hyperspace-wave" in target_dbs or "hyperspace_wave" in target_dbs) and HYPERSPACE_AVAILABLE:
            try:
                coll_name = f"rag_standards_wave_{dom}"
                def hs_setup():
                    client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.create_collection(coll_name, dimension=evaluator.dim, metric="cosine")
                    client.configure(ef_search=32, ef_construction=100, collection=coll_name)
                    return client
                
                def hs_ins(client, embs, texts):
                    ids = list(range(len(embs)))
                    metas = [{"text": t} for t in texts]
                    batch_size = 2000
                    for i in range(0, len(embs), batch_size):
                        client.batch_insert(
                            embs[i:i+batch_size],
                            ids[i:i+batch_size],
                            metas[i:i+batch_size],
                            collection=coll_name
                        )
                
                def hs_srch_wave(client, q_emb, limit):
                    # 1. Fetch top-5 global seeds
                    seeds = client.search(q_emb, top_k=5, collection=coll_name)
                    if not seeds:
                        return []
                    
                    node_scores = {}
                    node_metas = {}
                    
                    # 2. Run wave traversals from each seed and aggregate consensus energy
                    for seed in seeds:
                        seed_dist = seed.get("distance", 1.0)
                        seed_weight = 1.0 / (1.0 + seed_dist)
                        
                        traversed = client.traverse(
                            start_id=seed["id"],
                            max_depth=3,
                            max_nodes=limit * 3,
                            traversal_mode=1,
                            collection=coll_name
                        )
                        
                        for rank_n, node in enumerate(traversed):
                            node_id = node["id"]
                            if "text" not in node["metadata"]:
                                continue
                            
                            # Aggregate score: seed similarity * local traversal decay
                            node_score = seed_weight * (1.0 / (1.0 + rank_n))
                            node_scores[node_id] = node_scores.get(node_id, 0.0) + node_score
                            node_metas[node_id] = node["metadata"]
                            
                    # 3. Sort globally by consensus energy score descending
                    sorted_nodes = sorted(node_scores.items(), key=lambda x: x[1], reverse=True)
                    
                    return [{"text": node_metas[node_id]["text"]} for node_id, _ in sorted_nodes[:limit]]
                
                def hs_cleanup(client):
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.close()
                    
                res = evaluator.run_eval("HyperspaceDB-Wave", corpus, queries, hs_setup, hs_ins, hs_srch_wave, hs_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ HyperspaceDB-Wave evaluation failed: {e}")

        # ── QDRANT ──
        if (not target_dbs or "qdrant" in target_dbs) and QDRANT_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                def qd_setup():
                    client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.create_collection(
                        coll_name,
                        vectors_config=VectorParams(size=evaluator.dim, distance=Distance.COSINE)
                    )
                    return client
                
                def qd_ins(client, embs, texts):
                    points = [PointStruct(id=i, vector=v, payload={"text": texts[i]}) for i, v in enumerate(embs)]
                    batch_size = 500
                    for i in range(0, len(points), batch_size):
                        client.upsert(coll_name, points[i:i+batch_size], wait=True)
                
                def qd_srch(client, q_emb, limit):
                    res = client.search(coll_name, query_vector=q_emb, limit=limit)
                    return [{"text": h.payload["text"]} for h in res]
                
                def qd_cleanup(client):
                    try: client.delete_collection(coll_name)
                    except: pass
                    client.close()
                    
                res = evaluator.run_eval("Qdrant", corpus, queries, qd_setup, qd_ins, qd_srch, qd_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Qdrant evaluation failed: {e}")

        # ── CHROMADB ──
        if (not target_dbs or "chroma" in target_dbs) and CHROMA_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                
                class ChromaContext:
                    def __init__(self, col, client):
                        self.col = col
                        self.client = client
                        
                def chr_setup():
                    client = None
                    col = None
                    # Fallback patch telemetry
                    class UniversalNoopTelemetry:
                        def __init__(self, *args, **kwargs): pass
                        def capture(self, *args, **kwargs): pass
                        def context(self, *args, **kwargs): pass
                        def dependencies(self): return set()
                        def start(self): pass
                        def stop(self): pass

                    try:
                        client = chromadb.HttpClient(
                            host="localhost",
                            port=8000,
                            settings=Settings(anonymized_telemetry=False),
                        )
                        if hasattr(client, "_telemetry"): client._telemetry = UniversalNoopTelemetry()
                        try: client.delete_collection(coll_name)
                        except: pass
                        col = client.create_collection(coll_name, metadata={"hnsw:space": "cosine"})
                    except Exception:
                        client = None
                        col = None

                    if col is None:
                        db_dir = os.path.abspath(".chroma_rag_data")
                        client = chromadb.PersistentClient(
                            path=db_dir,
                            settings=Settings(anonymized_telemetry=False),
                        )
                        if hasattr(client, "_telemetry"): client._telemetry = UniversalNoopTelemetry()
                        try: client.delete_collection(coll_name)
                        except: pass
                        col = client.create_collection(coll_name, metadata={"hnsw:space": "cosine"})
                    
                    return ChromaContext(col, client)
                
                def chr_ins(ctx, embs, texts):
                    ids = [str(i) for i in range(len(embs))]
                    batch_size = 500
                    for i in range(0, len(embs), batch_size):
                        ctx.col.add(
                            embeddings=embs[i:i+batch_size],
                            ids=ids[i:i+batch_size],
                            documents=texts[i:i+batch_size]
                        )
                
                def chr_srch(ctx, q_emb, limit):
                    res = ctx.col.query(query_embeddings=[q_emb], n_results=limit)
                    return [{"text": doc} for doc in res["documents"][0]]
                
                def chr_cleanup(ctx):
                    try:
                        ctx.client.delete_collection(coll_name)
                    except: pass
                    
                res = evaluator.run_eval("ChromaDB", corpus, queries, chr_setup, chr_ins, chr_srch, chr_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ ChromaDB evaluation failed: {e}")

        # ── MILVUS ──
        if (not target_dbs or "milvus" in target_dbs) and MILVUS_AVAILABLE:
            try:
                coll_name = f"rag_standards_{dom}"
                def mil_setup():
                    connections.connect(host="localhost", port="19530")
                    if utility.has_collection(coll_name):
                        utility.drop_collection(coll_name)
                    schema = CollectionSchema([
                        FieldSchema("id", DataType.INT64, is_primary=True, auto_id=False),
                        FieldSchema("vec", DataType.FLOAT_VECTOR, dim=evaluator.dim),
                        FieldSchema("text", DataType.VARCHAR, max_length=65535)
                    ])
                    col = Collection(coll_name, schema)
                    col.create_index("vec", {"metric_type": "COSINE", "index_type": "IVF_FLAT", "params": {"nlist": 64}})
                    col.load()
                    return col
                
                def mil_ins(col, embs, texts):
                    ids = list(range(len(embs)))
                    def truncate_bytes(s, max_bytes=65530):
                        b = s.encode('utf-8')
                        return b[:max_bytes].decode('utf-8', errors='ignore') if len(b) > max_bytes else s
                    truncated_texts = [truncate_bytes(t) for t in texts]
                    col.insert([ids, embs, truncated_texts])
                    col.flush()
                
                def mil_srch(col, q_emb, limit):
                    res = col.search([q_emb], "vec", {"metric_type": "COSINE", "params": {"nprobe": 10}}, limit=limit, output_fields=["text"])
                    return [{"text": h.entity.get("text")} for h in res[0]]
                
                def mil_cleanup(col):
                    try: utility.drop_collection(coll_name)
                    except: pass
                    
                res = evaluator.run_eval("Milvus", corpus, queries, mil_setup, mil_ins, mil_srch, mil_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Milvus evaluation failed: {e}")

        # ── WEAVIATE ──
        if (not target_dbs or "weaviate" in target_dbs) and WEAVIATE_AVAILABLE:
            try:
                coll_name = f"RagStandards{dom.capitalize()}"
                def weav_setup():
                    client = weaviate.Client(url="http://localhost:8080")
                    if client.schema.exists(coll_name):
                        client.schema.delete_class(coll_name)
                    client.schema.create_class({
                        "class": coll_name,
                        "vectorizer": "none",
                        "vectorIndexConfig": {"distance": "cosine"},
                        "properties": [
                            {"name": "text", "dataType": ["text"]}
                        ]
                    })
                    return client
                
                def weav_ins(client, embs, texts):
                    client.batch.configure(batch_size=100)
                    with client.batch as batch:
                        for i, (emb, text) in enumerate(zip(embs, texts)):
                            batch.add_data_object(
                                data_object={"text": text},
                                class_name=coll_name,
                                vector=emb
                            )
                
                def weav_srch(client, q_emb, limit):
                    res = client.query.get(coll_name, ["text"]).with_near_vector({"vector": q_emb}).with_limit(limit).do()
                    if "data" in res and "Get" in res["data"] and coll_name in res["data"]["Get"] and res["data"]["Get"][coll_name]:
                        return [{"text": obj["text"]} for obj in res["data"]["Get"][coll_name] if obj]
                    return []
                
                def weav_cleanup(client):
                    try: client.schema.delete_class(coll_name)
                    except: pass
                    
                res = evaluator.run_eval("Weaviate", corpus, queries, weav_setup, weav_ins, weav_srch, weav_cleanup)
                results.append(res)
            except Exception as e:
                print(f"❌ Weaviate evaluation failed: {e}")
                
        if results:
            domain_results[dom] = results
            
    if domain_results:
        evaluator.generate_html_report(domain_results)
    else:
        print("\n❌ No databases were successfully evaluated across any domains. Please check database server statuses.")


if __name__ == "__main__":
    main()
