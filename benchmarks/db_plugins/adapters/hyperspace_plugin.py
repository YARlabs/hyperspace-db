import os
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
        use_hyp = mode in ["poincare", "lorentz"]
        target_vecs = ctx.doc_vecs_hyp if use_hyp else ctx.doc_vecs_euc
        target_q_vecs = ctx.q_vecs_hyp if use_hyp else ctx.q_vecs_euc
        target_dim = ctx.cfg.dim_hyp if use_hyp else ctx.cfg.dim_base

        if mode == "poincare":
            geom_name = "Poincaré"
        elif mode == "lorentz":
            geom_name = "Lorentz"
        else:
            geom_name = "Euclidean"

        if target_vecs is None or target_q_vecs is None:
            return Result("Hyperspace", target_dim, geom_name, mode.capitalize(),
                          0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0",
                          "Error: missing vectors")

        try:
            server_metric = legacy.detect_hyperspace_metric()
            if server_metric and server_metric != mode:
                return Result(
                    "Hyperspace", target_dim, geom_name, mode.capitalize(),
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0",
                    f"Skipped: mode mismatch ({server_metric})",
                )

            client = HyperspaceClient("localhost:50051", api_key="I_LOVE_HYPERSPACEDB", pool_size=16)
            coll_name = f"bench_semantic_{int(time.time())}"

            try:
                client.delete_collection(coll_name)
                time.sleep(0.5)
            except Exception:
                pass

            print(f"   Creating collection '{coll_name}' [dim={target_dim}, metric={mode}]...")
            if not client.create_collection(coll_name, dimension=target_dim, metric=mode):
                from hyperspace.proto import hyperspace_pb2
                req = hyperspace_pb2.CreateCollectionRequest(
                    name=coll_name, dimension=target_dim, metric=mode
                )
                try:
                    client.stub.CreateCollection(req, metadata=client.metadata)
                except Exception as e:
                    print(f"❌ Critical Create Error: {e}")
                    raise RuntimeError(f"Hyperspace collection creation failed: {e}")

            # ── Tune HNSW for best quality search ─────────────────────────────
            # ef_search=200 yields higher recall, especially for Cosine 1024d.
            # This is applied BEFORE inserts so the background indexer uses it.
            client.configure(ef_search=200, ef_construction=200, collection=coll_name)

            # ── Batch Insert ───────────────────────────────────────────────────
            t0 = time.time()
            # gRPC limit is 64MB; keep batches under 48MB with headroom.
            h_batch_size = max(10, int(48_000_000 / (target_dim * 8)))
            print(f"   Using batch size: {h_batch_size} (dim={target_dim})")
            for i in tqdm(range(0, len(target_vecs), h_batch_size), desc="Hyperspace Insert"):
                batch_vecs = target_vecs[i: i + h_batch_size]
                batch_ids = ctx.doc_ids[i: i + h_batch_size]
                int_ids = list(range(i, i + len(batch_ids)))
                metas = [{"doc_id": did} for did in batch_ids]
                client.batch_insert(batch_vecs.tolist(), int_ids, metas,
                                    collection=coll_name)
            v_dur = time.time() - t0

            # ── Wait for HNSW indexing to settle ──────────────────────────────
            legacy.wait_for_indexing(collection=coll_name)

            # ── Accuracy phase (search_batch for throughput) ───────────────────
            all_res_ids = []
            all_gt_ids = []
            lats = []
            search_t0 = time.time()

            # Use search_batch (single gRPC call per batch_size queries).
            # The server's HS_SEARCH_BATCH_INNER_CONCURRENCY env var controls
            # how many of these run in parallel server-side (default=1 sequential).
            query_batch_size = 64
            for i in tqdm(range(0, len(target_q_vecs), query_batch_size),
                          desc="Hyperspace Search"):
                batch_vecs = target_q_vecs[i: i + query_batch_size]
                for j in range(len(batch_vecs)):
                    q_id = ctx.test_query_ids[i + j]
                    all_gt_ids.append(ctx.valid_qrels.get(q_id, []))

                batch_ids, batch_lats = legacy.hyperspace_search_many(
                    client=client,
                    vectors=batch_vecs,
                    top_k=10,
                    collection=coll_name,
                    batch_size=query_batch_size,
                )
                all_res_ids.extend(batch_ids)
                lats.extend(batch_lats)

            search_dur = time.time() - search_t0
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            gt_for_mode = ctx.math_gt_hyp if use_hyp else ctx.math_gt_euc
            recall_sys = legacy.calculate_system_recall(all_res_ids, gt_for_mode, 10)

            # ── Concurrency profile ────────────────────────────────────────────
            # NOTE: HS_SEARCH_BATCH_INNER_CONCURRENCY is intentionally left at
            # default (1) for the concurrency profile: run_concurrency_profile()
            # already spawns 1/10/30 client threads, each sending a search_batch
            # of 32 queries. Adding server-side inner concurrency *on top* of
            # 30 client threads would create 30×32=960 simultaneous HNSW tasks,
            # massively over-saturating the CPU and hurting throughput.
            # For single-connection workloads set HS_SEARCH_BATCH_INNER_CONCURRENCY
            # to the number of CPU cores for maximum throughput.
            q_list = target_q_vecs[0].tolist()

            def hyperspace_query() -> None:
                client.search(q_list, top_k=10, collection=coll_name)

            conc = legacy.run_concurrency_profile(
                hyperspace_query,
                queries_per_call=1,
            )

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
            return Result(
                "Hyperspace", target_dim, geom_name, mode.capitalize(),
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "0",
                f"Error: {exc}",
            )


PLUGIN = HyperspacePlugin()
