import os
import shutil
import time

import numpy as np
from tqdm import tqdm

from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result
class ChromaPlugin(DatabasePlugin):
    name = "chroma"

    def is_available(self) -> bool:
        try:
            import chromadb  # noqa: F401

            return True
        except Exception:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import run_benchmark_legacy as legacy

        if ctx.doc_vecs_euc is None or ctx.q_vecs_euc is None:
            return Result("ChromaDB", 0, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", "Error: missing vectors")

        import chromadb

        cleanup_local_dir = None
        try:
            name = "bench_semantic"
            client = None
            col = None
            chroma_local_dir = None
            os.environ["ANONYMIZED_TELEMETRY"] = "False"
            os.environ["CHROMA_TELEMETRY_IMPL"] = "chromadb.telemetry.product.noop.NoopTelemetry"

            if hasattr(chromadb, "HttpClient"):
                try:
                    from chromadb.config import Settings

                    client = chromadb.HttpClient(
                        host="localhost",
                        port=8000,
                        settings=Settings(anonymized_telemetry=False),
                    )
                    try:
                        client.delete_collection(name)
                    except Exception:
                        pass
                    col = client.create_collection(name, metadata={"hnsw:space": "cosine"})
                except Exception:
                    client = None
                    col = None

            if col is None:
                if hasattr(chromadb, "PersistentClient"):
                    chroma_local_dir = os.path.join(os.path.dirname(__file__), "..", "..", ".chroma_bench_data")
                    cleanup_local_dir = os.path.abspath(chroma_local_dir)
                    from chromadb.config import Settings

                    client = chromadb.PersistentClient(
                        path=cleanup_local_dir,
                        settings=Settings(anonymized_telemetry=False),
                    )
                else:
                    from chromadb.config import Settings

                    client = chromadb.Client(
                        Settings(
                            chroma_api_impl="rest",
                            chroma_server_host="localhost",
                            chroma_server_http_port="8000",
                            anonymized_telemetry=False,
                        )
                    )
                try:
                    client.delete_collection(name)
                except Exception:
                    pass
                col = client.create_collection(name, metadata={"hnsw:space": "cosine"})

            t0 = time.time()
            c_batch_size = max(10, int(3_000_000 / (ctx.cfg.dim_base * 8)))
            for i in tqdm(range(0, len(ctx.doc_vecs_euc), c_batch_size), desc="Chroma Insert"):
                batch_vecs = ctx.doc_vecs_euc[i : i + c_batch_size]
                batch_ids = ctx.doc_ids[i : i + c_batch_size]
                col.add(ids=batch_ids, embeddings=batch_vecs.tolist())
            v_dur = time.time() - t0

            all_res_ids = []
            all_gt_ids = []
            lats = []
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(ctx.q_vecs_euc, desc="Chroma Search")):
                q_id = ctx.test_query_ids[i]
                all_gt_ids.append(ctx.valid_qrels.get(q_id, []))

                ts = time.time()
                res = col.query(query_embeddings=[q_vec.tolist()], n_results=10)
                lats.append((time.time() - ts) * 1000)
                all_res_ids.append(res["ids"][0])

            search_dur = time.time() - search_t0
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = legacy.calculate_system_recall(all_res_ids, ctx.math_gt_euc, 10)

            q_list = ctx.q_vecs_euc[0].tolist()

            def chroma_query() -> None:
                col.query(query_embeddings=[q_list], n_results=10)

            conc = legacy.run_concurrency_profile(chroma_query)
            if chroma_local_dir:
                disk = legacy.get_local_disk(os.path.abspath(chroma_local_dir))
            else:
                disk = legacy.format_size(legacy.get_docker_disk("chroma"))
            client.delete_collection(name)

            return Result(
                database="ChromaDB",
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
            return Result("ChromaDB", ctx.cfg.dim_base, "Euclidean", "Cosine", 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", f"Error: {exc}")
        finally:
            if cleanup_local_dir and os.path.exists(cleanup_local_dir):
                shutil.rmtree(cleanup_local_dir, ignore_errors=True)


PLUGIN = ChromaPlugin()
