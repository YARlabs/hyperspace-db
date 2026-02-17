import os
os.environ['KMP_DUPLICATE_LIB_OK'] = 'TRUE'

import pathlib
import sys
from typing import Dict, List, Tuple

import numpy as np

import run_benchmark_legacy as legacy
from db_plugins.registry import load_plugins, select_plugins
from plugin_runtime import BenchmarkContext, Result


def _load_case_data(cfg: legacy.Config, case_name: str) -> Tuple[List[str], List[str], List[str], List[str], Dict[str, List[str]], np.ndarray, np.ndarray, List[List[str]]]:
    if not legacy.VB_AVAILABLE:
        raise RuntimeError("vectordb_bench is required for --case mode")

    case_cls = getattr(legacy, case_name, None)
    if case_cls is None:
        raise RuntimeError(f"Unknown case: {case_name}")

    case_inst = case_cls()
    ds = case_inst.dataset
    ds.prepare(source=legacy.DatasetSource.S3)
    data_dir = pathlib.Path(ds.data_dir)
    if not data_dir.exists():
        raise RuntimeError(f"Data directory not found: {data_dir}")

    import pandas as pd

    train_path = data_dir / "train.parquet"
    if not train_path.exists():
        train_path = data_dir / "shuffle_train.parquet"
    if not train_path.exists():
        raise RuntimeError(f"Train data file not found: {train_path}")

    df_train = pd.read_parquet(train_path)
    if "emb" in df_train.columns:
        doc_vecs_euc = np.stack(df_train["emb"].values).astype(np.float32)
    elif "vector" in df_train.columns:
        doc_vecs_euc = np.stack(df_train["vector"].values).astype(np.float32)
    else:
        doc_vecs_euc = np.stack(df_train.iloc[:, 1].values).astype(np.float32)

    limit = cfg.doc_limit if cfg.doc_limit > 0 else len(doc_vecs_euc)
    doc_vecs_euc = doc_vecs_euc[:limit]
    if "id" in df_train.columns:
        doc_ids = [str(x) for x in df_train["id"].values[: len(doc_vecs_euc)]]
    else:
        doc_ids = [str(i) for i in range(len(doc_vecs_euc))]
    docs = [f"Real Doc {i}" for i in range(len(doc_vecs_euc))]

    test_path = data_dir / "test.parquet"
    q_ids_from_data = None
    if test_path.exists():
        df_test = pd.read_parquet(test_path)
        if "emb" in df_test.columns:
            q_vecs_euc = np.stack(df_test["emb"].values).astype(np.float32)
        elif "vector" in df_test.columns:
            q_vecs_euc = np.stack(df_test["vector"].values).astype(np.float32)
        else:
            q_vecs_euc = np.stack(df_test.iloc[:, 1].values).astype(np.float32)
        if "id" in df_test.columns:
            q_ids_from_data = [str(x) for x in df_test["id"].values]
    else:
        q_vecs_euc = doc_vecs_euc[:100]
    q_vecs_euc = q_vecs_euc[: cfg.query_limit]

    test_queries = [f"Query {i}" for i in range(len(q_vecs_euc))]
    if q_ids_from_data is not None:
        test_query_ids = q_ids_from_data[: len(q_vecs_euc)]
    else:
        test_query_ids = [str(i) for i in range(len(q_vecs_euc))]
    valid_qrels: Dict[str, List[str]] = {}
    math_gt_euc: List[List[str]] = []

    neighbors_path = data_dir / "neighbors.parquet"
    if neighbors_path.exists():
        df_gt = pd.read_parquet(neighbors_path)
        col = None
        for candidate in ("neighbors", "labels", "neighbors_id"):
            if candidate in df_gt.columns:
                col = candidate
                break
        if col:
            math_gt_euc = [[str(idx) for idx in list(x)[:10]] for x in df_gt[col].values[: len(q_vecs_euc)]]
            for i, row in enumerate(math_gt_euc):
                valid_qrels[test_query_ids[i]] = row

    cfg.dim_base = int(doc_vecs_euc.shape[1])
    return docs, doc_ids, test_queries, test_query_ids, valid_qrels, doc_vecs_euc, q_vecs_euc, math_gt_euc


def prepare_context(cfg: legacy.Config, target_db: str | None = None) -> BenchmarkContext:
    docs: List[str] = []
    doc_ids: List[str] = []
    test_queries: List[str] = []
    test_query_ids: List[str] = []
    valid_qrels: Dict[str, List[str]] = {}
    doc_vecs_euc = q_vecs_euc = doc_vecs_hyp = q_vecs_hyp = None
    math_gt_euc: List[List[str]] = []
    math_gt_hyp: List[List[str]] = []

    if cfg.target_case:
        docs, doc_ids, test_queries, test_query_ids, valid_qrels, doc_vecs_euc, q_vecs_euc, math_gt_euc = _load_case_data(cfg, cfg.target_case)
    else:
        docs, doc_ids, test_queries, test_query_ids, valid_qrels = legacy.load_data_smart(cfg)
        if not docs:
            raise RuntimeError("Data loading failed")

    ds_slug = cfg.dataset_name.replace("/", "_")
    target_db_norm = (target_db or "").lower()
    run_hyperspace_only = target_db_norm in {"hyper", "hyperspace"}

    need_euc = not (run_hyperspace_only and cfg.HYPER_MODE.lower() == "poincare")
    if need_euc and doc_vecs_euc is None:
        cache_file = f"cache_{ds_slug}_euclidean_1024d_{cfg.doc_limit}.npz"
        if pathlib.Path(cache_file).exists():
            doc_vecs_euc = np.load(cache_file)["embeddings"]
        else:
            model_base = legacy.Vectorizer(cfg.model_path_base, is_hyperbolic=False, target_dim=cfg.dim_base)
            doc_vecs_euc = model_base.encode(docs, batch_size=cfg.batch_size)
            np.savez_compressed(cache_file, embeddings=doc_vecs_euc, doc_ids=np.array(doc_ids))

        q_cache_file = f"cache_{ds_slug}_euclidean_queries_{cfg.query_limit}.npy"
        if pathlib.Path(q_cache_file).exists():
            q_vecs_euc = np.load(q_cache_file)
        else:
            model_base = legacy.Vectorizer(cfg.model_path_base, is_hyperbolic=False, target_dim=cfg.dim_base)
            q_vecs_euc = model_base.encode(test_queries, batch_size=cfg.batch_size)
            np.save(q_cache_file, q_vecs_euc)

    need_hyp = cfg.HYPER_MODE.lower() == "poincare" and (not target_db_norm or run_hyperspace_only)
    if need_hyp:
        cache_file = f"cache_{ds_slug}_hyperbolic_64d_{cfg.doc_limit}.npz"
        if pathlib.Path(cache_file).exists():
            doc_vecs_hyp = np.load(cache_file)["embeddings"]
        else:
            model_hyp = legacy.Vectorizer(cfg.model_path_hyp, is_hyperbolic=True, target_dim=cfg.dim_hyp)
            doc_vecs_hyp = model_hyp.encode(docs, batch_size=cfg.batch_size)
            np.savez_compressed(cache_file, embeddings=doc_vecs_hyp, doc_ids=np.array(doc_ids))

        q_cache_file = f"cache_{ds_slug}_hyperbolic_queries_{cfg.query_limit}.npy"
        if pathlib.Path(q_cache_file).exists():
            q_vecs_hyp = np.load(q_cache_file)
        else:
            model_hyp = legacy.Vectorizer(cfg.model_path_hyp, is_hyperbolic=True, target_dim=cfg.dim_hyp)
            q_vecs_hyp = model_hyp.encode(test_queries, batch_size=cfg.batch_size)
            np.save(q_cache_file, q_vecs_hyp)

    if doc_vecs_euc is not None and q_vecs_euc is not None and not math_gt_euc:
        math_gt_euc = legacy.calculate_brute_force_gt(q_vecs_euc, doc_vecs_euc, doc_ids, k=10, metric="cosine")
        if not valid_qrels:
            for i, q_id in enumerate(test_query_ids):
                valid_qrels[q_id] = math_gt_euc[i]

    if doc_vecs_hyp is not None and q_vecs_hyp is not None:
        math_gt_hyp = legacy.calculate_brute_force_gt(q_vecs_hyp, doc_vecs_hyp, doc_ids, k=10, metric="poincare")

    return BenchmarkContext(cfg, docs, doc_ids, test_queries, test_query_ids, valid_qrels, doc_vecs_euc, q_vecs_euc, doc_vecs_hyp, q_vecs_hyp, math_gt_euc, math_gt_hyp)


def write_story(cfg: legacy.Config, docs: List[str], queries: List[str], final_results: List[Result]) -> None:
    with open("BENCHMARK_STORY_MODULAR.md", "w") as f:
        f.write("# Modular Benchmark Report\n\n")
        f.write(f"Testing on **{cfg.dataset_name}** with **{len(docs):,}** docs and **{len(queries):,}** queries.\n\n")
        f.write("| Database | Dim | Geometry | Metric | Ins QPS | Srch QPS | P99 Lat | Recall(Sem)@10 | Recall(Sys)@10 | MRR | NDCG@10 | C1 | C10 | C30 | Disk |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n")
        for r in final_results:
            if r.status == "Success":
                f.write(f"| **{r.database}** | {r.dimension:,} | {r.geometry} | {r.metric} | {r.insert_qps:,.0f} | {r.search_qps:,.0f} | {r.p99:.2f}ms | {r.recall:.1%} | {r.recall_sys:.1%} | {r.mrr:.2f} | {r.ndcg:.2f} | {r.c1_qps:,.0f} | {r.c10_qps:,.0f} | {r.c30_qps:,.0f} | {r.disk_usage} |\n")


def main() -> None:
    cfg = legacy.Config()
    case_arg = next((arg for arg in sys.argv if arg.startswith("--case=")), None)
    if case_arg:
        cfg.target_case = case_arg.split("=", 1)[1]
        cfg.apply_case(cfg.target_case)

    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    target_db = args[0].lower() if args else None

    ctx = prepare_context(cfg, target_db=target_db)
    plugins = load_plugins()
    selected = select_plugins(plugins, target_db)
    final_results: List[Result] = []
    for plugin in selected:
        if not plugin.is_available():
            continue
        final_results.append(plugin.run(ctx))

    legacy.print_table(final_results)
    write_story(cfg, ctx.docs, ctx.test_queries, final_results)


if __name__ == "__main__":
    main()
