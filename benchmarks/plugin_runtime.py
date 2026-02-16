from dataclasses import dataclass
from typing import Any, Dict, List, Optional

import numpy as np


@dataclass
class Result:
    database: str
    dimension: int
    geometry: str
    metric: str
    insert_qps: float
    search_qps: float
    p50: float
    p95: float
    p99: float
    recall: float
    recall_sys: float
    mrr: float
    ndcg: float
    c1_qps: float
    c10_qps: float
    c30_qps: float
    disk_usage: str
    status: str


@dataclass
class BenchmarkContext:
    cfg: Any
    docs: List[str]
    doc_ids: List[str]
    test_queries: List[str]
    test_query_ids: List[str]
    valid_qrels: Dict[str, List[str]]
    doc_vecs_euc: Optional[np.ndarray]
    q_vecs_euc: Optional[np.ndarray]
    doc_vecs_hyp: Optional[np.ndarray]
    q_vecs_hyp: Optional[np.ndarray]
    math_gt_euc: List[List[str]]
    math_gt_hyp: List[List[str]]
