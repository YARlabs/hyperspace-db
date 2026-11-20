# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (release build, Apple M4 Pro, 12 shards = 1 per core); baseline = v4.2.4 (`2fa9d9f`) with the `flat_l0.bin` deadlock fix back-ported so it can complete the sweep
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-20 13:50:27

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **100.00%** | 1,518 | 0.64ms | 0.80ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **100.00%** | 1,530 | 0.64ms | 0.76ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **88.60%** | 1,791 | 0.54ms | 0.96ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **100.00%** | 1,419 | 0.69ms | 0.88ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **87.15%** | 1,607 | 0.61ms | 0.77ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **87.15%** | 1,604 | 0.61ms | 0.79ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **100.00%** | 1,299 | 0.76ms | 0.91ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **100.00%** | 1,135 | 0.87ms | 1.15ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **87.60%** | 1,242 | 0.79ms | 0.89ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.60%** | 1,134 | 0.87ms | 1.15ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **100.00%** | 861 | 1.15ms | 2.07ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **100.00%** | 771 | 1.28ms | 2.18ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.90%** | 1,337 | 0.73ms | 0.96ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.90%** | 1,336 | 0.73ms | 0.84ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **100.00%** | 954 | 1.03ms | 1.29ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **100.00%** | 903 | 1.09ms | 1.42ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **85.10%** | 1,213 | 0.80ms | 1.03ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **24.25%** | 1,771 | 0.54ms | 0.74ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.25%** | 1,807 | 0.53ms | 0.70ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **81.50%** | 1,444 | 0.68ms | 0.83ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **84.45%** | 1,317 | 0.74ms | 0.99ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **41.05%** | 1,558 | 0.62ms | 0.85ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **26.95%** | 1,937 | 0.50ms | 0.55ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **80.85%** | 1,265 | 0.77ms | 1.34ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **84.90%** | 1,300 | 0.75ms | 0.98ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **42.50%** | 1,725 | 0.56ms | 0.62ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **36.00%** | 1,982 | 0.49ms | 0.65ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **98.80%** | 1,184 | 0.82ms | 1.18ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **99.80%** | 1,334 | 0.74ms | 0.94ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.90%** | 992 | 0.99ms | 1.27ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **62.80%** | 1,758 | 0.55ms | 0.76ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **77.00%** | 1,358 | 0.72ms | 0.85ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.50%** | 2,098 | 0.45ms | 0.53ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **98.55%** | 1,576 | 0.62ms | 0.72ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.60%** | 1,413 | 0.69ms | 0.94ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.25%** | 1,912 | 0.50ms | 0.55ms | 2.67 MB | **29.3×** |
| **pq4** | `pq4` | None | **36.35%** | 1,422 | 0.68ms | 0.74ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **98.70%** | 1,157 | 0.85ms | 0.98ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.75%** | 1,112 | 0.88ms | 1.19ms | 2.52 MB | **31.1×** |

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

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.22s | 16,382 v/s | 623.4 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 1.28s | 15,652 v/s | 649.2 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 1.34s | 14,944 v/s | 695.1 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.37s | 14,557 v/s | 673.1 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 2.24s | 8,918 v/s | 626.3 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 2.26s | 8,850 v/s | 605.8 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 2.28s | 8,787 v/s | 614.0 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 2.29s | 8,729 v/s | 624.8 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 2.59s | 7,735 v/s | 613.6 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 2.59s | 7,720 v/s | 645.2 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 2.70s | 7,414 v/s | 631.0 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 2.90s | 6,891 v/s | 616.8 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 13.95s | 1,434 v/s | 801.9 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 14.24s | 1,404 v/s | 840.7 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 13.75s | 1,454 v/s | 816.2 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 15.38s | 1,300 v/s | 811.4 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 14.01s | 1,427 v/s | 801.9 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 4.07s | 4,918 v/s | 1069.7 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 3.74s | 5,354 v/s | 1092.6 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 3.74s | 5,342 v/s | 1080.9 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 3.89s | 5,135 v/s | 1089.8 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 4.01s | 4,990 v/s | 1096.8 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 2.76s | 7,241 v/s | 901.3 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 2.73s | 7,338 v/s | 897.9 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 2.68s | 7,450 v/s | 887.5 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 2.67s | 7,498 v/s | 902.7 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 1.75s | 11,409 v/s | 557.2 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 1.74s | 11,485 v/s | 576.8 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 1.72s | 11,597 v/s | 550.2 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 1.76s | 11,369 v/s | 577.8 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 1.74s | 11,476 v/s | 549.7 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 1.77s | 11,283 v/s | 555.0 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 3.98s | 5,020 v/s | 1055.3 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 3.89s | 5,143 v/s | 1087.5 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 3.71s | 5,392 v/s | 1051.2 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 3.31s | 6,041 v/s | 1060.8 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 2.15s | 9,309 v/s | 1011.2 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 2.12s | 9,437 v/s | 1033.3 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 2.15s | 9,300 v/s | 1024.2 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **36.00%** | 1,982 | 0.49ms | 0.76ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **98.80%** | 1,184 | 0.82ms | 3.26ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **99.80%** | 1,334 | 0.74ms | 1.14ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.90%** | 992 | 0.99ms | 1.44ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **62.80%** | 1,758 | 0.55ms | 0.91ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **77.00%** | 1,358 | 0.72ms | 1.04ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **24.25%** | 1,771 | 0.54ms | 0.88ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.25%** | 1,807 | 0.53ms | 0.98ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **81.50%** | 1,444 | 0.68ms | 0.92ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **84.45%** | 1,317 | 0.74ms | 1.11ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **41.05%** | 1,558 | 0.62ms | 1.12ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **26.95%** | 1,937 | 0.50ms | 0.59ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **80.85%** | 1,265 | 0.77ms | 3.67ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **84.90%** | 1,300 | 0.75ms | 1.15ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **42.50%** | 1,725 | 0.56ms | 0.68ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.90%** | 1,337 | 0.73ms | 1.16ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.90%** | 1,336 | 0.73ms | 0.90ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **100.00%** | 954 | 1.03ms | 1.44ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **100.00%** | 903 | 1.09ms | 1.70ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **85.10%** | 1,213 | 0.80ms | 1.16ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.50%** | 2,098 | 0.45ms | 0.59ms | 6.9s |
| **rabitq_rerank4** | **98.55%** | 1,576 | 0.62ms | 0.77ms | 6.4s |
| **rabitq_rerank8** | **99.60%** | 1,413 | 0.69ms | 1.11ms | 6.1s |
| **rabitq_twopass_raw40** | **62.25%** | 1,912 | 0.50ms | 0.66ms | 6.2s |
| **pq4** | **36.35%** | 1,422 | 0.68ms | 0.79ms | 3.0s |
| **pq4_rerank4** | **98.70%** | 1,157 | 0.85ms | 1.04ms | 2.9s |
| **pq4_rerank8** | **99.75%** | 1,112 | 0.88ms | 1.30ms | 2.9s |

---

## 7. Architectural Recommendation Matrix

Recall and QPS below are the measured values from section 1.

| Workload / Use Case | Recommended Config | Recall@10 | Search QPS | Compression | Trade-off |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **Max precision** | `none` | 100.00% | 1,518 | 1× | Exact on small shards (flat scan); no calibration. |
| **Enterprise RAG** | `medium_rerank4` | 100.00% | 1,419 | 4× | No calibration, any metric. |
| **Memory-optimal L2 / cosine** | `rabitq_rerank4` | 98.55% | 1,576 | 29× | Calibrates online. |
| **Highest-recall compressed** | `pq4_rerank8` | 99.75% | 1,112 | 31× | Calibrates online. |
| **Any metric, compact** | `extreme_rerank8` | 99.80% | 1,334 | 31× | No calibration; works on hyperbolic metrics. |
| **Any metric, balanced** | `medium_plus_rerank4` / `ternary_rerank4` | 100.00% / 100.00% | 1,299 / 954 | 5.3× / 15.3× | No calibration. |
| **Ultra-dense (L2 / cosine)** | `pq_rerank8` / `opq_rerank8` | 84.90% / 84.45% | 1,300 / 1,317 | 60× | 16-D sub-spaces limit recall. |
