import time

import numpy as np
from tqdm import tqdm

from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result
class MilvusPlugin(DatabasePlugin):
    name = "milvus"

    def is_available(self) -> bool:
        try:
            from pymilvus import Collection  # noqa: F401

            return True
        except Exception:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import run_benchmark_legacy as legacy

        if ctx.doc_vecs_euc is None or ctx.q_vecs_euc is None:
            return Result("Milvus", 0, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", "Error: missing vectors")

        from pymilvus import Collection, CollectionSchema, DataType, FieldSchema, connections, utility

        try:
            connections.connect(host="localhost", port="19530")
            if utility.has_collection("bench_semantic"):
                utility.drop_collection("bench_semantic")

            schema = CollectionSchema(
                [
                    FieldSchema("id", DataType.INT64, is_primary=True, auto_id=True),
                    FieldSchema("doc_id", DataType.VARCHAR, max_length=128),
                    FieldSchema("vec", DataType.FLOAT_VECTOR, dim=ctx.cfg.dim_base),
                ],
                "",
            )
            col = Collection("bench_semantic", schema)

            t0 = time.time()
            m_batch_size = max(10, int(3_000_000 / (ctx.cfg.dim_base * 8)))
            for i in tqdm(range(0, len(ctx.doc_vecs_euc), m_batch_size), desc="Milvus Insert"):
                batch_vecs = ctx.doc_vecs_euc[i : i + m_batch_size]
                batch_ids = ctx.doc_ids[i : i + m_batch_size]
                col.insert([batch_ids, batch_vecs.tolist()])
            v_dur = time.time() - t0

            col.flush()
            col.create_index("vec", {"metric_type": "COSINE", "index_type": "IVF_FLAT", "params": {"nlist": 128}})
            col.load()
            time.sleep(5)

            all_res_ids = []
            all_gt_ids = []
            lats = []
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(ctx.q_vecs_euc, desc="Milvus Search")):
                q_id = ctx.test_query_ids[i]
                all_gt_ids.append(ctx.valid_qrels.get(q_id, []))

                ts = time.time()
                res = col.search(
                    [q_vec.tolist()],
                    "vec",
                    {"metric_type": "COSINE", "params": {"nprobe": 10}},
                    limit=10,
                    output_fields=["doc_id"],
                )
                lats.append((time.time() - ts) * 1000)
                all_res_ids.append([hit.entity.get("doc_id") for hit in res[0]])

            search_dur = time.time() - search_t0
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = legacy.calculate_system_recall(all_res_ids, ctx.math_gt_euc, 10)

            q_list = ctx.q_vecs_euc[0].tolist()

            def milvus_query() -> None:
                col.search([q_list], "vec", {"metric_type": "COSINE", "params": {"nprobe": 10}}, limit=10)

            conc = legacy.run_concurrency_profile(milvus_query)
            disk = legacy.format_size(legacy.get_docker_disk("milvus"))
            utility.drop_collection("bench_semantic")

            return Result(
                database="Milvus",
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
            return Result("Milvus", ctx.cfg.dim_base, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", f"Error: {exc}")


PLUGIN = MilvusPlugin()
