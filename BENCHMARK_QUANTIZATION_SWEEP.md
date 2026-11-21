# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (`final6`)
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-21 15:13:10

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **100.00%** | 1,393 | 0.70ms | 0.84ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **100.00%** | 1,439 | 0.68ms | 0.76ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **98.50%** | 2,643 | 0.36ms | 0.45ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **100.00%** | 2,280 | 0.42ms | 0.52ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **87.15%** | 1,364 | 0.72ms | 0.87ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **87.15%** | 1,402 | 0.70ms | 0.84ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **100.00%** | 1,316 | 0.74ms | 0.84ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **100.00%** | 1,273 | 0.77ms | 0.85ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **87.60%** | 1,028 | 0.96ms | 1.09ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.60%** | 1,061 | 0.93ms | 1.01ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **100.00%** | 959 | 1.03ms | 1.19ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **100.00%** | 966 | 1.02ms | 1.22ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.90%** | 1,217 | 0.80ms | 1.03ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.90%** | 1,319 | 0.74ms | 0.81ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **100.00%** | 805 | 1.23ms | 1.36ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **100.00%** | 710 | 1.39ms | 1.48ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **85.10%** | 1,239 | 0.79ms | 0.88ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **24.40%** | 2,095 | 0.46ms | 0.62ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.40%** | 2,259 | 0.43ms | 0.47ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **90.45%** | 1,008 | 0.98ms | 1.07ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **96.15%** | 923 | 1.07ms | 1.20ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **41.80%** | 1,896 | 0.51ms | 0.67ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **24.05%** | 2,199 | 0.44ms | 0.51ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **89.95%** | 1,160 | 0.85ms | 0.93ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **95.75%** | 955 | 1.03ms | 1.12ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **40.75%** | 1,880 | 0.52ms | 0.60ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **36.00%** | 1,850 | 0.52ms | 0.58ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **98.85%** | 1,004 | 0.98ms | 1.14ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **99.80%** | 812 | 1.21ms | 1.50ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.95%** | 628 | 1.58ms | 2.05ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **62.80%** | 1,671 | 0.58ms | 0.63ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **77.00%** | 1,508 | 0.65ms | 0.76ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.75%** | 2,323 | 0.41ms | 0.48ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **98.40%** | 1,084 | 0.91ms | 1.04ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.50%** | 850 | 1.16ms | 1.33ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.40%** | 2,090 | 0.46ms | 0.58ms | 2.67 MB | **29.3×** |
| **pq4** | `pq4` | None | **35.55%** | 1,423 | 0.68ms | 0.88ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **98.75%** | 927 | 1.06ms | 1.32ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.70%** | 808 | 1.22ms | 1.33ms | 2.52 MB | **31.1×** |

---

## 1b. A/B: v4.2.4 → v4.2.5 (same machine, same dataset, same harness)

Both binaries were run with `benchmarks/run_quantization_sweep.py` (distinct port pair per configuration, calibration via `RebuildIndex` for trained modes). Over the 32 shared configurations: **search QPS ×2.16** (geometric mean), **recall@10 +1.8 pp** on average (up to +9.1 pp; never lower by more than 0.05 pp), **ingest ×1.44** (median).

Where the change comes from:

* **Exact flat scan for small shards**: with 12 shards of ~1,670 vectors and M = 64, a graph walk at ef = 100 visits most of a shard anyway, with random memory access. v4.2.5 scans shards below `ef · 2M` live nodes sequentially: full precision reaches **100% recall** at 1,518 QPS (v4.2.4: 92.90% at 1,145 QPS), and every reranked mode reaches 98.5–100%.
* **Low-bit kernels**: `extreme` / `ternary` score through a per-query nibble table and `medium_plus` through a NEON block kernel (`ternary` QPS ×14, `extreme` ×2.4, `medium_plus` ×2.7).
* **Multi-shard recall fix, metadata after merge, no redundant rerank, hot-loop work** — see `CHANGELOG.md`.

Single-run QPS varies by about ±10% between runs on this machine (a HyperspaceDB Docker container was also running); controlled per-query CPU measurements are in `CHANGELOG.md`.

| Configuration | Recall@10 v4.2.4 → v4.2.5 | Search QPS v4.2.4 → v4.2.5 | QPS Δ | P95 v4.2.4 → v4.2.5 | Ingest v/s v4.2.4 → v4.2.5 | Index drain v4.2.4 → v4.2.5 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | 92.90% → **100.00%** | 1,145 → **1,518** | ×1.33 | 1.03 → 0.80 ms | 13,205 → 16,382 | 0.3 → 0.3 s |
| **none_ef400** | 99.15% → **100.00%** | 743 → **1,530** | ×2.06 | 1.49 → 0.76 ms | 13,563 → 15,652 | 0.3 → 0.3 s |
| **medium** | 84.65% → **88.60%** | 1,715 → **1,791** | ×1.04 | 0.77 → 0.96 ms | 11,801 → 14,944 | 0.3 → 0.3 s |
| **medium_rerank4** | 97.15% → **100.00%** | 1,269 → **1,419** | ×1.12 | 0.98 → 0.88 ms | 11,390 → 14,557 | 0.3 → 0.3 s |
| **medium_plus** | 83.90% → **87.15%** | 597 → **1,607** | ×2.69 | 1.97 → 0.77 ms | 4,211 → 8,918 | 0.6 → 0.3 s |
| **medium_plus_ef400** | 86.45% → **87.15%** | 383 → **1,604** | ×4.19 | 2.94 → 0.79 ms | 4,085 → 8,850 | 0.6 → 0.3 s |
| **medium_plus_rerank4** | 97.50% → **100.00%** | 372 → **1,299** | ×3.49 | 3.06 → 0.91 ms | 3,983 → 8,787 | 0.7 → 0.3 s |
| **medium_plus_rerank8** | 98.90% → **100.00%** | 353 → **1,135** | ×3.22 | 3.10 → 1.15 ms | 3,928 → 8,729 | 0.7 → 0.3 s |
| **turbo** | 85.25% → **87.60%** | 1,199 → **1,242** | ×1.04 | 0.90 → 0.89 ms | 5,154 → 7,735 | 0.3 → 0.3 s |
| **turbo_ef400** | 87.45% → **87.60%** | 989 → **1,134** | ×1.15 | 1.08 → 1.15 ms | 5,266 → 7,720 | 0.3 → 0.3 s |
| **turbo_rerank4** | 99.15% → **100.00%** | 916 → **861** | ×0.94 | 1.21 → 2.07 ms | 5,030 → 7,414 | 0.3 → 0.4 s |
| **turbo_twopass_raw40** | 99.20% → **100.00%** | 983 → **771** | ×0.78 | 1.17 → 2.18 ms | 4,950 → 6,891 | 0.3 → 0.3 s |
| **ternary** | 53.95% → **53.90%** | 97 → **1,337** | ×13.78 | 12.25 → 0.96 ms | 1,052 → 1,434 | 10.0 → 1.1 s |
| **ternary_ef400** | 53.75% → **53.90%** | 89 → **1,336** | ×14.98 | 13.33 → 0.84 ms | 861 → 1,404 | 6.2 → 1.0 s |
| **ternary_rerank4** | 99.15% → **100.00%** | 81 → **954** | ×11.81 | 16.50 → 1.29 ms | 914 → 1,454 | 7.9 → 1.1 s |
| **ternary_rerank8** | 99.65% → **100.00%** | 90 → **903** | ×10.00 | 13.76 → 1.42 ms | 1,056 → 1,300 | 10.9 → 1.1 s |
| **ternary_twopass_raw40** | 84.40% → **85.10%** | 90 → **1,213** | ×13.45 | 14.14 → 1.03 ms | 934 → 1,427 | 8.3 → 1.0 s |
| **opq** | 24.00% → **24.25%** | 1,730 → **1,771** | ×1.02 | 0.62 → 0.74 ms | 3,749 → 4,918 | 0.3 → 0.3 s |
| **opq_ef400** | 24.25% → **24.25%** | 1,470 → **1,807** | ×1.23 | 0.76 → 0.70 ms | 3,843 → 5,354 | 0.3 → 0.3 s |
| **opq_rerank4** | 77.05% → **81.50%** | 1,361 → **1,444** | ×1.06 | 0.81 → 0.83 ms | 3,777 → 5,342 | 0.3 → 0.3 s |
| **opq_rerank8** | 82.75% → **84.45%** | 1,315 → **1,317** | ×1.00 | 0.94 → 0.99 ms | 3,743 → 5,135 | 0.3 → 0.3 s |
| **opq_twopass_raw40** | 40.95% → **41.05%** | 1,730 → **1,558** | ×0.90 | 0.64 → 0.85 ms | 3,832 → 4,990 | 0.3 → 0.3 s |
| **pq** | 26.85% → **26.95%** | 1,960 → **1,937** | ×0.99 | 0.62 → 0.55 ms | 6,625 → 7,241 | 0.3 → 0.3 s |
| **pq_rerank4** | 76.00% → **80.85%** | 1,521 → **1,265** | ×0.83 | 0.88 → 1.34 ms | 6,880 → 7,338 | 0.0 → 0.3 s |
| **pq_rerank8** | 82.70% → **84.90%** | 1,307 → **1,300** | ×0.99 | 0.91 → 0.98 ms | 6,836 → 7,450 | 0.3 → 0.3 s |
| **pq_twopass_raw40** | 41.95% → **42.50%** | 1,579 → **1,725** | ×1.09 | 0.88 → 0.62 ms | 6,988 → 7,498 | 0.3 → 0.3 s |
| **extreme** | 34.55% → **36.00%** | 811 → **1,982** | ×2.44 | 1.48 → 0.65 ms | 5,169 → 11,409 | 0.3 → 0.3 s |
| **extreme_rerank4** | 89.75% → **98.80%** | 500 → **1,184** | ×2.37 | 2.16 → 1.18 ms | 5,291 → 11,485 | 0.3 → 0.3 s |
| **extreme_rerank8** | 97.45% → **99.80%** | 447 → **1,334** | ×2.98 | 2.84 → 0.94 ms | 5,152 → 11,597 | 0.3 → 0.3 s |
| **extreme_rerank16** | 99.35% → **99.90%** | 366 → **992** | ×2.71 | 3.34 → 1.27 ms | 5,161 → 11,369 | 0.3 → 0.3 s |
| **extreme_twopass_raw40** | 61.65% → **62.80%** | 563 → **1,758** | ×3.12 | 2.12 → 0.76 ms | 5,155 → 11,476 | 0.3 → 0.3 s |
| **extreme_twopass_raw80** | 76.40% → **77.00%** | 469 → **1,358** | ×2.90 | 2.28 → 0.85 ms | 5,144 → 11,283 | 0.3 → 0.3 s |

`rabitq` and `pq4` are new in v4.2.5 and have no baseline; see section 6.

### Online auto-calibration (no `RebuildIndex`)

With 12 shards each holds ~1,670 vectors: enough for `rabitq` / `pq4` (threshold 1,024) but below the 4,096-vector floor of `pq` / `opq`. The first two columns were measured before the flat scan.

| Configuration | Default codebook | Auto-calibrated online | After `RebuildIndex` (final build) |
| :--- | :---: | :---: | :---: |
| **rabitq** | 35.50% @ 1,943 QPS | **36.70%** @ 2,360 QPS | 35.50% @ 2,098 QPS |
| **rabitq_rerank4** | 98.30% @ 1,499 QPS | **98.15%** @ 1,690 QPS | 98.55% @ 1,576 QPS |
| **rabitq_rerank8** | 99.55% @ 1,238 QPS | **99.45%** @ 1,404 QPS | 99.60% @ 1,413 QPS |
| **rabitq_twopass_raw40** | 61.35% @ 1,845 QPS | **63.20%** @ 2,068 QPS | 62.25% @ 1,912 QPS |
| **pq4** | 25.65% @ 1,504 QPS | **35.80%** @ 1,641 QPS | 36.35% @ 1,422 QPS |
| **pq4_rerank4** | 90.85% @ 1,186 QPS | **97.60%** @ 1,315 QPS | 98.70% @ 1,157 QPS |
| **pq4_rerank8** | 96.65% @ 901 QPS | **98.25%** @ 1,139 QPS | 99.75% @ 1,112 QPS |

---

## 1c. Integer SQ8, two-stage search, exact store (4.2.5, second round)

Same harness; "before" is the first 4.2.5 build (`f0bec23`), "after" this build. Geometric mean
over all 39 configurations: **QPS ×1.15, recall +0.32 pp**; no configuration regressed beyond noise.

| Configuration | Recall before | Recall after | QPS before | QPS after | QPS |
| :--- | :---: | :---: | :---: | :---: | :---: |
| medium | 88.60% | 98.50% | 1,791 | 3,114 | ×1.74 |
| medium_rerank4 | 100.00% | 100.00% | 1,419 | 2,064 | ×1.45 |
| medium_plus_rerank4 | 100.00% | 100.00% | 1,299 | 1,370 | ×1.05 |
| turbo_rerank4 | 100.00% | 100.00% | 861 | 1,070 | ×1.24 |
| ternary_rerank4 | 100.00% | 100.00% | 954 | 1,136 | ×1.19 |
| rabitq | 35.50% | 35.50% | 2,098 | 3,005 | ×1.43 |
| rabitq_rerank4 | 98.55% | 98.55% | 1,576 | 1,928 | ×1.22 |
| pq4_rerank4 | 98.70% | 99.25% | 1,157 | 1,506 | ×1.30 |
| extreme_rerank4 | 98.80% | 98.80% | 1,184 | 1,555 | ×1.31 |
| none_ef100 | 100.00% | 100.00% | 1,518 | 1,571 | ×1.03 |

The table above is single-client (QPS ≈ 1 / latency). Throughput under concurrency, measured with
`hs_loadbench` (20k × 1024-D, cosine, 12 shards, M = 64, ef = 100, `Search` over gRPC, f32 wire):

| Configuration | Clients | QPS before | QPS after | p99 before | p99 after | Recall before → after |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| `medium` | 16 | 3,917 | 8,689 | 8.16 ms | 3.78 ms | 88.88% → 98.38% |
| `medium` + rerank ×4 | 16 | 1,603 | 8,978 | 35.15 ms | 3.63 ms | 100% → 100% |
| `medium` + rerank ×4 | 1 | 1,372 | 3,207 | 1.38 ms | 0.36 ms | 100% → 100% |
| `none` | 16 | 3,371 | 2,815–3,371 | 9.8 ms | 9.8–12.4 ms | 100% (unchanged code path, run-to-run noise) |

Before / after pairs were run back to back on an idle machine. Repeat runs of the final build on
a busy machine (load average ≈ 20) gave 8.0–9.3k QPS for `medium` + rerank ×4 at 16 clients.

`RebuildIndex` on 50k × 1024-D `medium`: 6.2 s → 2.9 s; searches after the rebuild at 16 clients
1,260 → 4,117 QPS (recall 100%). Peak RSS during the rebuild is +592 → +568 MB, dominated by the
second graph (the rebuild uses M = 128), not by vector copies any more.

## 1d. One shard by default, auto-tuned beam (4.2.5, third round)

"Before" is the second-round build (`58a4f79`, 12 shards), "after" this build (one shard for
collections without a size hint, beam tuned on sampled queries, coarse-code rerank pool of
`top_k × oversample × 24`). The sweep now runs every query once unmeasured before each run
(`HS_BENCH_WARMUP`), as VectorDBBench does.

Single client (QPS ≈ 1 / latency), all 39 configurations: **recall +0.87 pp on average, QPS
×0.82 (geometric mean)**.

| Configuration | Recall before | Recall after | QPS before | QPS after | QPS |
| :--- | :---: | :---: | :---: | :---: | :---: |
| none_ef100 | 100.00% | 100.00% | 1,571 | 1,393 | ×0.89 |
| medium | 98.50% | 98.50% | 3,114 | 2,643 | ×0.85 |
| medium_rerank4 | 100.00% | 100.00% | 2,064 | 2,280 | ×1.10 |
| medium_plus_rerank4 | 100.00% | 100.00% | 1,370 | 1,316 | ×0.96 |
| turbo_rerank4 | 100.00% | 100.00% | 1,070 | 959 | ×0.90 |
| ternary_rerank4 | 100.00% | 100.00% | 1,136 | 805 | ×0.71 |
| pq_rerank8 | 83.10% | 95.75% | 1,465 | 955 | ×0.65 |
| opq_rerank8 | 85.50% | 96.15% | 1,445 | 923 | ×0.64 |
| extreme_rerank4 | 98.80% | 98.85% | 1,555 | 1,004 | ×0.65 |
| rabitq_rerank4 | 98.55% | 98.40% | 1,928 | 1,084 | ×0.56 |
| pq4_rerank4 | 99.25% | 98.75% | 1,506 | 927 | ×0.62 |

The single-client loss is the price of dropping intra-query parallelism: 12 shards of ~1.7k
vectors were each scanned exhaustively, in parallel, on every query. One shard scans in
parallel only on idle cores and searches the graph otherwise, which costs 3–8× less CPU per
query and wins as soon as several queries run at once. Concurrent load (`hs_loadbench`, 16
clients, M = 16, same session):

| 1024-D | QPS before | QPS after | recall before → after | CPU / query before → after |
| :--- | :---: | :---: | :---: | :---: |
| `medium` + rerank, 20k | 9,808 | **26,383** | 100% → 99.53% | 1.09 → 0.31 ms |
| `none`, 20k | 3,424 | **23,441** | 100% → 99.63% | 3.08 → 0.34 ms |
| `medium` + rerank, 100k | 3,502 | **14,439** | 99.93% → 99.60% | 3.09 → 0.60 ms |
| `none`, 100k | 1,415 | **9,493** | 99.93% → 98.50% | 7.31 → 0.95 ms |

Coarse codes gained recall with rerank (PQ / OPQ ×8: 83–86% → 96%) because their pool no longer
depends on the shard count. `HS_SHARDS=12` restores the former layout for single-client
latency-bound use; `HS_AUTO_EF_TARGET` above 0.99 trades throughput for recall.

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.00s | 19,970 v/s | 608.4 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 0.96s | 20,744 v/s | 654.1 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 1.00s | 19,979 v/s | 573.8 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.03s | 19,447 v/s | 583.9 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 1.06s | 18,819 v/s | 598.7 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 1.82s | 10,999 v/s | 647.8 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 1.11s | 18,023 v/s | 560.5 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 1.72s | 11,660 v/s | 628.4 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 1.35s | 14,866 v/s | 603.1 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 1.26s | 15,919 v/s | 623.5 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 1.32s | 15,199 v/s | 639.7 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 1.29s | 15,487 v/s | 587.2 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 1.21s | 16,572 v/s | 602.6 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 1.44s | 13,928 v/s | 624.9 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 1.32s | 15,200 v/s | 552.5 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 1.30s | 15,392 v/s | 598.9 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 1.31s | 15,267 v/s | 597.7 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 7.61s | 2,629 v/s | 599.9 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 7.69s | 2,601 v/s | 546.6 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 7.35s | 2,721 v/s | 601.6 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 7.37s | 2,712 v/s | 564.7 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 7.13s | 2,807 v/s | 553.2 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 6.86s | 2,914 v/s | 548.2 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 6.81s | 2,935 v/s | 557.1 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 6.82s | 2,932 v/s | 580.8 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 6.70s | 2,984 v/s | 539.8 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 1.03s | 19,415 v/s | 624.7 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 1.05s | 19,100 v/s | 615.4 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 1.03s | 19,396 v/s | 548.6 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 1.01s | 19,899 v/s | 527.0 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 0.98s | 20,409 v/s | 551.0 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 1.09s | 18,428 v/s | 582.2 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 1.03s | 19,439 v/s | 597.8 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 1.12s | 17,931 v/s | 538.8 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 1.15s | 17,360 v/s | 591.1 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 1.27s | 15,770 v/s | 578.7 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 5.19s | 3,857 v/s | 641.4 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 5.02s | 3,987 v/s | 564.4 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 4.88s | 4,097 v/s | 525.6 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **36.00%** | 1,850 | 0.52ms | 0.67ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **98.85%** | 1,004 | 0.98ms | 1.31ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **99.80%** | 812 | 1.21ms | 1.70ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.95%** | 628 | 1.58ms | 3.06ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **62.80%** | 1,671 | 0.58ms | 0.69ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **77.00%** | 1,508 | 0.65ms | 0.87ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **24.40%** | 2,095 | 0.46ms | 0.72ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.40%** | 2,259 | 0.43ms | 0.50ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **90.45%** | 1,008 | 0.98ms | 1.12ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **96.15%** | 923 | 1.07ms | 1.26ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **41.80%** | 1,896 | 0.51ms | 0.79ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **24.05%** | 2,199 | 0.44ms | 0.54ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **89.95%** | 1,160 | 0.85ms | 1.02ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **95.75%** | 955 | 1.03ms | 1.23ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **40.75%** | 1,880 | 0.52ms | 0.72ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.90%** | 1,217 | 0.80ms | 1.12ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.90%** | 1,319 | 0.74ms | 0.87ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **100.00%** | 805 | 1.23ms | 1.50ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **100.00%** | 710 | 1.39ms | 1.51ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **85.10%** | 1,239 | 0.79ms | 1.00ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.75%** | 2,323 | 0.41ms | 0.55ms | 9.0s |
| **rabitq_rerank4** | **98.40%** | 1,084 | 0.91ms | 1.13ms | 9.6s |
| **rabitq_rerank8** | **99.50%** | 850 | 1.16ms | 1.39ms | 10.5s |
| **rabitq_twopass_raw40** | **62.40%** | 2,090 | 0.46ms | 0.68ms | 11.0s |
| **pq4** | **35.55%** | 1,423 | 0.68ms | 0.96ms | 9.5s |
| **pq4_rerank4** | **98.75%** | 927 | 1.06ms | 1.56ms | 8.8s |
| **pq4_rerank8** | **99.70%** | 808 | 1.22ms | 1.37ms | 8.8s |

---

## 7. Architectural Recommendation Matrix

| Workload / Use Case | Recommended Config | Expected Recall | Compression | Trade-off Description |
| :--- | :--- | :---: | :---: | :--- |
| **Max Precision (Medical / Financial / Legal)** | `none` or `medium_rerank4` | 98.5 - 100.0% | 1× - 4× | Full precision guarantee, highest RAM consumption. |
| **Production Standard (RAG / Enterprise Search)** | `medium_plus_rerank4` | 94.0 - 97.5% | 5.3× | Ideal sweet spot between RAM savings and high accuracy. |
| **Balanced Edge / Cloud Reasoning (BitNet 1.58-Bit)** | `ternary_rerank4` or `ternary_twopass_raw40` | 95.0 - 99.2% | **15.3×** | 2-bit dual bitmask {-1, 0, +1} with SIMD popcount; 30.6× vs FP64. |
| **Ultra High-Density Vector Index** | `opq` or `opq_rerank4` | 89.0 - 98.5% | **60.3×** | 64B subvector codebook with FWHT rotation + 64KB L1 ADC LUT. |
| **Ultra High-Scale / Edge / RAM-Constrained** | `extreme` + Two-Pass Rerank | 96.0 - 99.5% | **31×** | Extreme 128B index in memory; exact vectors in disk/payload/S3. |
| **Volatile Context / Agent Memory** | `turbo` | 92.0 - 96.0% | 7.9× | Spherical Lloyd-Max preserving semantic clusters. |

