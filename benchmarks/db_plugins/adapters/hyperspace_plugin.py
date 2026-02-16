import time

import numpy as np
from tqdm import tqdm

from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result
class HyperspacePlugin(DatabasePlugin):
    name = "hyper"

    def is_available(self) -> bool:
        try:
            from hyperspace import HyperspaceClient  # noqa: F401

            return True
        except Exception:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import run_benchmark_legacy as legacy

        from hyperspace import HyperspaceClient

        mode = ctx.cfg.HYPER_MODE.lower()
        use_hyp = mode == "poincare"
        target_vecs = ctx.doc_vecs_hyp if use_hyp else ctx.doc_vecs_euc
        target_q_vecs = ctx.q_vecs_hyp if use_hyp else ctx.q_vecs_euc
        target_dim = ctx.cfg.dim_hyp if use_hyp else ctx.cfg.dim_base
        geom_name = "Poincaré" if use_hyp else "Euclidean"

        if target_vecs is None or target_q_vecs is None:
            return Result("Hyperspace", target_dim, geom_name, mode.capitalize(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", "Error: missing vectors")

        try:
            server_metric = legacy.detect_hyperspace_metric()
            if server_metric and server_metric != mode:
                return Result(
                    "Hyperspace",
                    target_dim,
                    geom_name,
                    mode.capitalize(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    "0",
                    f"Skipped: mode mismatch ({server_metric})",
                )

            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB")
            coll_name = "bench_semantic"
            try:
                client.delete_collection(coll_name)
            except Exception:
                pass

            if not client.create_collection(coll_name, dimension=target_dim, metric=mode):
                raise RuntimeError("Collection creation failed")

            t0 = time.time()
            h_batch_size = max(10, int(4_000_000 / (target_dim * 8)))
            for i in tqdm(range(0, len(target_vecs), h_batch_size), desc="Hyperspace Insert"):
                batch_vecs = target_vecs[i : i + h_batch_size]
                batch_ids = ctx.doc_ids[i : i + h_batch_size]
                int_ids = list(range(i, i + len(batch_ids)))
                metas = [{"doc_id": did} for did in batch_ids]
                client.batch_insert(batch_vecs.tolist(), int_ids, metas, collection=coll_name)
            v_dur = time.time() - t0

            legacy.wait_for_indexing(collection=coll_name)
            all_res_ids = []
            all_gt_ids = []
            lats = []
            search_t0 = time.time()
            for i, q_vec in enumerate(tqdm(target_q_vecs, desc="Hyperspace Search")):
                q_id = ctx.test_query_ids[i]
                all_gt_ids.append(ctx.valid_qrels.get(q_id, []))
                ts = time.time()
                res = client.search(q_vec.tolist(), top_k=10, collection=coll_name)
                lats.append((time.time() - ts) * 1000)
                all_res_ids.append(legacy.extract_ids(res))

            search_dur = time.time() - search_t0
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            gt_for_mode = ctx.math_gt_hyp if use_hyp else ctx.math_gt_euc
            recall_sys = legacy.calculate_system_recall(all_res_ids, gt_for_mode, 10)

            q_list = target_q_vecs[0].tolist()

            def hyperspace_query() -> None:
                client.search(q_list, top_k=10, collection=coll_name)

            conc = legacy.run_concurrency_profile(hyperspace_query)
            disk = legacy.get_hyperspace_disk_api() or legacy.get_local_disk("../data")
            disk = legacy.format_size(disk)
            client.delete_collection(coll_name)

            return Result(
                database="Hyperspace",
                dimension=target_dim,
                geometry=geom_name,
                metric=mode.capitalize(),
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
            return Result("Hyperspace", target_dim, geom_name, mode.capitalize(), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0", f"Error: {exc}")


PLUGIN = HyperspacePlugin()
