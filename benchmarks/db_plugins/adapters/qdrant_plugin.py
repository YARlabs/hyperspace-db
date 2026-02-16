
import time

import numpy as np
from tqdm import tqdm

from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result


class QdrantPlugin(DatabasePlugin):
    name = "qdrant"

    def is_available(self) -> bool:
        try:
            from qdrant_client import QdrantClient  # noqa: F401

            return True
        except Exception:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import run_benchmark_legacy as legacy

        if ctx.doc_vecs_euc is None or ctx.q_vecs_euc is None:
            return Result("Qdrant", 0, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", "missing vectors")

        from qdrant_client import QdrantClient
        from qdrant_client.models import Distance, PointStruct, VectorParams

        try:
            client = QdrantClient(host="localhost", port=6334, prefer_grpc=True)
            name = "bench_semantic"
            try:
                client.delete_collection(name)
            except Exception:
                pass

            client.create_collection(name, vectors_config=VectorParams(size=ctx.cfg.dim_base, distance=Distance.COSINE))
            t0 = time.time()
            q_batch_size = max(10, int(3_000_000 / (ctx.cfg.dim_base * 8)))
            for i in tqdm(range(0, len(ctx.doc_vecs_euc), q_batch_size), desc="Qdrant Insert"):
                batch_vecs = ctx.doc_vecs_euc[i : i + q_batch_size]
                batch_ids = ctx.doc_ids[i : i + q_batch_size]
                points = [PointStruct(id=i + j, vector=v.tolist(), payload={"doc_id": batch_ids[j]}) for j, v in enumerate(batch_vecs)]
                client.upsert(collection_name=name, points=points, wait=True)
            v_dur = time.time() - t0
            time.sleep(5)

            all_res_ids = []
            all_gt_ids = []
            lats = []
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(ctx.q_vecs_euc, desc="Qdrant Search")):
                q_id = ctx.test_query_ids[i]
                all_gt_ids.append(ctx.valid_qrels.get(q_id, []))

                ts = time.time()
                # Use search for newer clients instead of query_points
                if hasattr(client, "search"):
                     res = client.search(collection_name=name, query_vector=q_vec.tolist(), limit=10)
                else: 
                     # Fallback for slightly older versions, though search is preferred in v1.7+
                     res = client.query_points(collection_name=name, query=q_vec.tolist(), limit=10).points

                lats.append((time.time() - ts) * 1000)
                
                # Handling different response structures depending on method used
                if hasattr(res, "points"): # unlikely if search() used properly, but safeguard
                     hits = res.points
                else:
                     hits = res
                     
                all_res_ids.append([hit.payload.get("doc_id") for hit in hits])

            search_dur = time.time() - search_t0
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = legacy.calculate_system_recall(all_res_ids, ctx.math_gt_euc, 10)

            q_list = ctx.q_vecs_euc[0].tolist()

            def qdrant_query() -> None:
                if hasattr(client, "search"):
                    client.search(collection_name=name, query_vector=q_list, limit=10)
                else:
                    client.query_points(collection_name=name, query=q_list, limit=10)

            conc = legacy.run_concurrency_profile(qdrant_query)
            disk = legacy.format_size(legacy.get_docker_disk("qdrant"))
            client.delete_collection(name)

            return Result(
                database="Qdrant",
                dimension=ctx.cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(ctx.docs) / v_dur,
                search_qps=len(ctx.test_queries) / search_dur,
                p50=float(np.percentile(lats, 50)),
                p95=float(np.percentile(lats, 95)),
                p99=float(np.percentile(lats, 99)),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success",
            )
        except Exception as exc:
            return Result("Qdrant", ctx.cfg.dim_base, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", f"Error: {exc}")


PLUGIN = QdrantPlugin()
