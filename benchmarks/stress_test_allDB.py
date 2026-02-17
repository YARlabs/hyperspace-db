import sys
import os
import time
import numpy as np
import threading
import requests
import argparse
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from typing import List, Dict, Any

# Paths
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "sdks", "python")))

@dataclass
class BenchResult:
    db_name: str
    concurrency: int
    ins_qps: float
    srch_qps: float
    ins_eff: float
    srch_eff: float

class StressTestRunner:
    def __init__(self, dim=1024, count_ins=10000, count_srch=2000):
        self.dim = dim
        self.count_ins = count_ins
        self.count_srch = count_srch
        self.results = []
        self.concurrencies = [1, 10, 50, 100, 500, 1000]

    def gen_vecs(self, count):
        return np.random.uniform(-0.1, 0.1, (count, self.dim)).astype(np.float32).tolist()

    def run_concurrency(self, name, setup_fn, insert_fn, search_fn, cleanup_fn, wait_fn=None):
        print(f"\n--- Testing {name} ---")
        base_ins = 0
        base_srch = 0
        
        for c in self.concurrencies:
            coll = f"stress_{name.lower()}_{c}"
            try:
                # Setup handles connection and collection creation
                db_context = setup_fn(coll)
                
                # Inserts
                print(f"  [{name}] C={c:4} | Phase: Inserts...", end="", flush=True)
                vecs = self.gen_vecs(self.count_ins)
                t0 = time.time()
                with ThreadPoolExecutor(max_workers=c) as ex:
                    batch_per_thread = max(1, self.count_ins // c)
                    for i in range(0, self.count_ins, batch_per_thread):
                        end_idx = min(i + batch_per_thread, self.count_ins)
                        ex.submit(insert_fn, db_context, coll, vecs[i:end_idx], i)
                ins_qps = self.count_ins / (time.time() - t0)
                if c == 1: base_ins = ins_qps
                print(f" Done. QPS: {ins_qps:8.0f}")

                if wait_fn: wait_fn(db_context, coll)
                else: time.sleep(2)

                # Search
                print(f"  [{name}] C={c:4} | Phase: Searches...", end="", flush=True)
                q_vecs = self.gen_vecs(self.count_srch)
                t0 = time.time()
                with ThreadPoolExecutor(max_workers=c) as ex:
                    if name.lower() == "hyperspace":
                        batch_size = 64
                        for i in range(0, len(q_vecs), batch_size):
                            ex.submit(search_fn, db_context, coll, q_vecs[i:i + batch_size])
                    else:
                        for v in q_vecs:
                            ex.submit(search_fn, db_context, coll, v)
                srch_qps = self.count_srch / (time.time() - t0)
                if c == 1: base_srch = srch_qps
                print(f" Done. QPS: {srch_qps:8.0f}")

                self.results.append(BenchResult(
                    db_name=name,
                    concurrency=c,
                    ins_qps=ins_qps,
                    srch_qps=srch_qps,
                    ins_eff=(ins_qps / (base_ins * c)) * 100 if (base_ins * c) > 0 else 0,
                    srch_eff=(srch_qps / (base_srch * c)) * 100 if (base_srch * c) > 0 else 0
                ))
                cleanup_fn(db_context, coll)
            except Exception as e:
                print(f"  ❌ Error at C={c}: {e}")
                # import traceback
                # traceback.print_exc()
                break

    def print_final_report(self):
        if not self.results:
            print("\n❌ No results to show.")
            return

        print("\n\n" + "!" * 40)
        print("!!! FINAL STRESS PERFORMANCE REPORT !!!")
        print("!" * 40)
        
        dbs = list(dict.fromkeys([r.db_name for r in self.results]))
        
        for metric in ["Search", "Insert"]:
            print(f"\nRANKING BY {metric.upper()} PERFORMANCE (Total QPS)")
            print("-" * 120)
            header = f"{'DB Name':<15} |"
            for c in self.concurrencies: header += f" C={c:<7} |"
            print(header)
            print("-" * 120)
            
            db_peak = {}
            for db in dbs:
                qps_vals = [r.srch_qps if metric=="Search" else r.ins_qps for r in self.results if r.db_name == db]
                db_peak[db] = max(qps_vals) if qps_vals else 0
            
            sorted_dbs = sorted(dbs, key=lambda x: db_peak[x], reverse=True)
            for db in sorted_dbs:
                row = f"{db:<15} |"
                for c in self.concurrencies:
                    val = next((r.srch_qps if metric=="Search" else r.ins_qps for r in self.results if r.db_name == db and r.concurrency == c), 0)
                    row += f" {val:8.0f} |"
                print(row)

        self.generate_html_report()

    def generate_html_report(self):
        dbs = list(dict.fromkeys([r.db_name for r in self.results]))
        html_path = os.path.abspath("STRESS_TEST_REPORT.html")
        js_search_datasets = []
        js_insert_datasets = []
        js_eff_datasets = []
        colors = ["#38bdf8", "#fac05e", "#818cf8", "#f472b6", "#10b981", "#6366f1"]
        
        # Concurrencies for efficiency (adding empty 1 and 1100 as padding)
        eff_concs = [1] + self.concurrencies[1:] + [1100]
        
        for i, db in enumerate(dbs):
            color = colors[i % len(colors)]
            db_res = [r for r in self.results if r.db_name == db]
            
            # Throughput data (all C)
            js_search_datasets.append({
                "label": db, 
                "data": [next((r.srch_qps for r in db_res if r.concurrency == c), 0) for c in self.concurrencies], 
                "borderColor": color, 
                "tension": 0.1
            })
            js_insert_datasets.append({
                "label": db, 
                "data": [next((r.ins_qps for r in db_res if r.concurrency == c), 0) for c in self.concurrencies], 
                "borderColor": color, 
                "tension": 0.1
            })
            
            # Efficiency data: force 1 and 1100 to be empty (0)
            eff_data = [0] # Empty placeholder for C=1
            eff_data.extend([next((r.srch_eff for r in db_res if r.concurrency == c), 0) for c in self.concurrencies[1:]])
            eff_data.append(0) # Empty placeholder for C=1100
            
            js_eff_datasets.append({
                "label": db, 
                "data": eff_data, 
                "backgroundColor": color,
                "hoverBackgroundColor": "#fff"
            })

        import json
        with open(html_path, "w") as f:
            f.write(f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Global Stress Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        :root {{ --bg: #0f172a; --card-bg: #1e293b; --text: #f8fafc; --accent: #38bdf8; }}
        body {{ font-family: 'Inter', sans-serif; background: var(--bg); color: var(--text); padding: 2rem; margin: 0; }}
        .grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 2rem; }}
        .card {{ background: var(--card-bg); padding: 1.5rem; border-radius: 1rem; border: 1px solid #334155; margin-bottom: 2rem; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.1); }}
        canvas {{ max-height: 450px; width: 100% !important; }}
        h1 {{ text-align: center; color: var(--accent); margin-bottom: 2rem; }}
        h2 {{ font-size: 1.1rem; color: #94a3b8; margin-top: 0; border-bottom: 1px solid #334155; padding-bottom: 0.5rem; }}
    </style>
</head>
<body>
    <h1>Vector DB Stress Test Metrics</h1>
    <div class="grid">
        <div class="card"><h2>Search Throughput (Total QPS)</h2><canvas id="sChart"></canvas></div>
        <div class="card"><h2>Insert Throughput (Total QPS)</h2><canvas id="iChart"></canvas></div>
    </div>
    <div class="card">
        <h2>Scalability Efficiency (%) - Search</h2>
        <p style="color: #64748b; font-size: 0.9rem; margin-bottom: 1rem;">(Percentage of linear speedup. Higher is better. Labels show % of core utilization and diff from leader)</p>
        <canvas id="eChart"></canvas>
    </div>
    <script>
        const concs = {json.dumps(self.concurrencies)};
        const effConcs = {json.dumps(eff_concs)};
        const commonOptions = {{ 
            responsive: true, 
            maintainAspectRatio: false,
            plugins: {{ legend: {{ labels: {{ color: '#f1f5f9', font: {{ family: 'Inter' }} }} }} }},
            scales: {{ 
                y: {{ grid: {{ color: '#334155' }}, ticks: {{ color: '#94a3b8' }} }}, 
                x: {{ grid: {{ color: '#334155' }}, ticks: {{ color: '#94a3b8' }} }} 
            }} 
        }};

        // Data Labels Plugin
        const topLabelsPlugin = {{
            id: 'topLabels',
            afterDatasetsDraw(chart) {{
                if (chart.canvas.id !== 'eChart') return;
                const {{ ctx, data }} = chart;
                ctx.save();
                ctx.textAlign = 'center';
                ctx.textBaseline = 'bottom';
                
                data.datasets.forEach((dataset, i) => {{
                    const meta = chart.getDatasetMeta(i);
                    meta.data.forEach((bar, index) => {{
                        const val = dataset.data[index];
                        if (val <= 0) return;

                        // Find leader for this concurrency point
                        let leaderVal = 0;
                        data.datasets.forEach(ds => {{ leaderVal = Math.max(leaderVal, ds.data[index]); }});
                        
                        // Value label
                        ctx.font = 'bold 11px Inter';
                        ctx.fillStyle = '#fff';
                        ctx.fillText(val.toFixed(1) + '%', bar.x, bar.y - 18);
                        
                        // Comparison label
                        if (leaderVal > 0) {{
                            const isLeader = val === leaderVal;
                            const diff = isLeader ? 'Leader' : '-' + ((leaderVal - val) / leaderVal * 100).toFixed(0) + '%';
                            ctx.font = '9px Inter';
                            ctx.fillStyle = isLeader ? '#10b981' : '#f87171';
                            ctx.fillText(diff, bar.x, bar.y - 6);
                        }}
                    }});
                }});
                ctx.restore();
            }}
        }};

        new Chart(document.getElementById('sChart'), {{ 
            type: 'line', 
            data: {{ labels: concs, datasets: {json.dumps(js_search_datasets)} }}, 
            options: commonOptions 
        }});
        new Chart(document.getElementById('iChart'), {{ 
            type: 'line', 
            data: {{ labels: concs, datasets: {json.dumps(js_insert_datasets)} }}, 
            options: commonOptions 
        }});
        new Chart(document.getElementById('eChart'), {{ 
            type: 'bar', 
            data: {{ labels: effConcs, datasets: {json.dumps(js_eff_datasets)} }}, 
            options: {{
                ...commonOptions,
                plugins: {{ ...commonOptions.plugins }},
                barPercentage: 0.8,
                categoryPercentage: 0.8
            }},
            plugins: [topLabelsPlugin]
        }});
    </script>
</body>
</html>""")
        print(f"\n✅ Visual report updated: {{html_path}}")

def run_stress_test():
    parser = argparse.ArgumentParser(description="Multi-DB Stress Test Runner")
    parser.add_argument("--db", nargs="+", help="Specific DBs to test (hyperspace qdrant milvus chroma weaviate)")
    parser.add_argument("--dim", type=int, default=1024, help="Vector dimension")
    args = parser.parse_args()

    runner = StressTestRunner(dim=args.dim)
    target_dbs = [d.lower() for d in args.db] if args.db else None

    # --- MONKEYPATCH CHROMA (Legacy logic) ---
    try:
        class UniversalNoopTelemetry:
            def __init__(self, *args, **kwargs): pass
            def capture(self, *args, **kwargs): pass
            def context(self, *args, **kwargs): pass
            def dependencies(self): return set()
            def start(self): pass
            def stop(self): pass
        
        import chromadb.telemetry.product.posthog
        chromadb.telemetry.product.posthog.Posthog = UniversalNoopTelemetry
        import chromadb.telemetry.product.noop
        chromadb.telemetry.product.noop.NoopTelemetry = UniversalNoopTelemetry
    except: pass

    # --- HYPERSPACE ---
    if not target_dbs or "hyperspace" in target_dbs:
        try:
            from hyperspace import HyperspaceClient
            def hs_setup(c):
                client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
                try: client.delete_collection(c)
                except: pass
                client.create_collection(c, dimension=args.dim, metric="cosine")
                return client
            def hs_ins(client, c, vecs, start_id): 
                chunk_size = 4000
                for i in range(0, len(vecs), chunk_size):
                    chunk = vecs[i:i+chunk_size]
                    ids = list(range(start_id + i, start_id + i + len(chunk)))
                    client.batch_insert(chunk, ids, collection=c)
            def hs_srch(client, c, vectors):
                if callable(getattr(client, "search_batch", None)):
                    payload = [v.tolist() if hasattr(v, "tolist") else v for v in vectors]
                    client.search_batch(payload, top_k=10, collection=c)
                    return
                for v in vectors:
                    client.search(v, top_k=10, collection=c)
            def hs_wait(client, coll):
                url = f"http://localhost:50050/api/collections/{coll}/stats"
                headers = {"x-api-key": "I_LOVE_HYPERSPACEDB"}
                for _ in range(60):
                    try:
                        r = requests.get(url, headers=headers).json()
                        if r.get("indexing_queue", 0) == 0 and r.get("count", 0) > 0: return
                    except: pass
                    time.sleep(1)
            runner.run_concurrency("Hyperspace", hs_setup, hs_ins, hs_srch, lambda cl, c: cl.delete_collection(c), hs_wait)
        except Exception as e: print(f"Skipping Hyperspace: {e}")

    # --- QDRANT ---
    if not target_dbs or "qdrant" in target_dbs:
        try:
            from qdrant_client import QdrantClient
            from qdrant_client.models import Distance, PointStruct, VectorParams
            def qd_setup(c):
                client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)
                try: client.delete_collection(c)
                except: pass
                client.create_collection(c, vectors_config=VectorParams(size=args.dim, distance=Distance.COSINE))
                return client
            def qd_ins(client, c, vecs, start_id):
                chunk_size = 1000
                for i in range(0, len(vecs), chunk_size):
                    chunk = vecs[i:i+chunk_size]
                    points = [PointStruct(id=start_id + i + k, vector=v, payload={}) for k, v in enumerate(chunk)]
                    client.upsert(c, points, wait=True)
            def qd_srch(client, c, v): client.search(c, query_vector=v, limit=10)
            runner.run_concurrency("Qdrant", qd_setup, qd_ins, qd_srch, lambda cl, c: cl.delete_collection(c))
        except Exception as e: print(f"Skipping Qdrant: {e}")

    # --- MILVUS ---
    if not target_dbs or "milvus" in target_dbs:
        try:
            from pymilvus import connections, Collection, CollectionSchema, DataType, FieldSchema, utility
            connections.connect(host="localhost", port="19530")
            def mil_setup(c):
                if utility.has_collection(c): utility.drop_collection(c)
                schema = CollectionSchema([
                    FieldSchema("id", DataType.INT64, is_primary=True, auto_id=False),
                    FieldSchema("vec", DataType.FLOAT_VECTOR, dim=args.dim)
                ])
                col = Collection(c, schema)
                col.create_index("vec", {"metric_type": "COSINE", "index_type": "IVF_FLAT", "params": {"nlist": 128}})
                col.load()
                return col
            def mil_ins(col, c, vecs, start_id):
                ids = list(range(start_id, start_id + len(vecs)))
                col.insert([ids, vecs])
            def mil_srch(col, c, v):
                col.search([v], "vec", {"metric_type": "COSINE", "params": {"nprobe": 10}}, limit=10)
            runner.run_concurrency("Milvus", mil_setup, mil_ins, mil_srch, lambda cl, c: utility.drop_collection(c))
        except Exception as e: print(f"Skipping Milvus: {e}")

    # --- CHROMA ---
    # if not target_dbs or "chroma" in target_dbs:
    #     try:
    #         import chromadb
    #         from chromadb.config import Settings
    #         def chr_setup(c):
    #             client = chromadb.HttpClient(host="localhost", port=8000, settings=Settings(anonymized_telemetry=False))
    #             try: client.delete_collection(c)
    #             except: pass
    #             col = client.create_collection(c, metadata={"hnsw:space": "cosine"})
    #             return col
    #         def chr_ins(col, c, vecs, start_id):
    #             ids = [str(start_id + i) for i in range(len(vecs))]
    #             for k in range(0, len(vecs), 500):
    #                 col.add(embeddings=vecs[k:k+500], ids=ids[k:k+500])
    #         def chr_srch(col, c, v):
    #             col.query(query_embeddings=[v], n_results=10)
    #         runner.run_concurrency("Chroma", chr_setup, chr_ins, chr_srch, lambda col, c: None) # Cleanup can be added if needed
    #     except Exception as e: print(f"Skipping Chroma: {e}")

    # --- WEAVIATE ---
    if not target_dbs or "weaviate" in target_dbs:
        try:
            import weaviate
            def weav_setup(c):
                client = weaviate.Client(url="http://localhost:8080")
                if client.schema.exists(c): client.schema.delete_class(c)
                client.schema.create_class({"class": c, "vectorizer": "none", "vectorIndexConfig": {"distance": "cosine"}})
                return client
            def weav_ins(client, c, vecs, start_id):
                client.batch.configure(batch_size=min(len(vecs), 1000))
                with client.batch as b:
                    for v in vecs: b.add_data_object({}, c, vector=v)
            def weav_srch(client, c, v):
                client.query.get(c, ["_additional { id }"]).with_near_vector({"vector": v}).with_limit(10).do()
            runner.run_concurrency("Weaviate", weav_setup, weav_ins, weav_srch, lambda cl, c: cl.schema.delete_class(c))
        except Exception as e: print(f"Skipping Weaviate: {e}")

    runner.print_final_report()

if __name__ == "__main__":
    run_stress_test()
