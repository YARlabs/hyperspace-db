
import time
import numpy as np
from tqdm import tqdm
from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result
import run_benchmark_legacy as legacy

class WeaviatePlugin(DatabasePlugin):
    name = "weaviate"

    def is_available(self) -> bool:
        try:
            import weaviate
            return True
        except ImportError:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import weaviate

        if ctx.doc_vecs_euc is None:
             return Result("Weaviate", 0, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", "missing vectors")

        try:
            # Connect to Weaviate (v3 client)
            client = weaviate.Client(
                url="http://localhost:8080",
                timeout_config=(5, 60),  # (connect, read)
            )
            
            class_name = "BenchmarkVector"
            
            # Cleanup
            if client.schema.exists(class_name):
                client.schema.delete_class(class_name)
            
            # Create Schema
            class_obj = {
                "class": class_name,
                "vectorizer": "none", # We provide vectors
                "vectorIndexType": "hnsw",
                "vectorIndexConfig": {
                    "distance": "cosine", 
                    "efConstruction": 128,
                    "maxConnections": 64
                },
                "properties": [
                    {"name": "doc_id", "dataType": ["string"]}
                ]
            }
            client.schema.create_class(class_obj)
            
            # Insert
            print("   Inserting into Weaviate...")
            t0 = time.time()
            
            # Configure batch
            client.batch.configure(batch_size=1000, dynamic=True)
            
            with client.batch as batch:
                for i, vec in enumerate(tqdm(ctx.doc_vecs_euc, desc="Weaviate Insert")):
                    doc_id = ctx.doc_ids[i]
                    batch.add_data_object(
                        data_object={"doc_id": doc_id},
                        class_name=class_name,
                        vector=vec
                    )
            
            v_dur = time.time() - t0
            
            # Search
            print("   Searching Weaviate...")
            all_res_ids = []
            lats = []
            
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(ctx.q_vecs_euc, desc="Weaviate Search")):
                ts = time.time()
                response = (
                    client.query
                    .get(class_name, ["doc_id"])
                    .with_near_vector({"vector": q_vec, "certainty": 0.0}) # certainty ignored for just ranking usually, but required arg in older v3
                    .with_limit(10)
                    .do()
                )
                lats.append((time.time() - ts) * 1000)
                
                try:
                    hits = response["data"]["Get"][class_name]
                    all_res_ids.append([h["doc_id"] for h in hits])
                except Exception:
                    all_res_ids.append([])

            search_dur = time.time() - search_t0
            
            # Metrics
            all_gt_ids = [ctx.valid_qrels.get(qid, []) for qid in ctx.test_query_ids]
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = legacy.calculate_system_recall(all_res_ids, ctx.math_gt_euc, 10)
            
            # Concurrency
            q_list = ctx.q_vecs_euc[0].tolist()
            def weaviate_query():
                client.query.get(class_name, ["doc_id"]).with_near_vector({"vector": q_list}).with_limit(10).do()
            
            conc = legacy.run_concurrency_profile(weaviate_query)
            disk = legacy.format_size(legacy.get_docker_disk("weaviate"))
            
            client.schema.delete_class(class_name)
            
            return Result(
                database="Weaviate",
                dimension=ctx.cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(ctx.docs) / v_dur,
                search_qps=len(ctx.test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            )

        except Exception as e:
            return Result("Weaviate", ctx.cfg.dim_base, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}")

PLUGIN = WeaviatePlugin()
