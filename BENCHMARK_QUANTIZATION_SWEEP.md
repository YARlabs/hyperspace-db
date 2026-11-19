# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (release build, Apple M4 Pro, 12 shards = 1 per core); baseline = v4.2.4 (`2fa9d9f`) with the `flat_l0.bin` deadlock fix back-ported so it can complete the sweep
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-19 17:43:29

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **99.05%** | 808 | 1.22ms | 1.34ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **99.90%** | 698 | 1.41ms | 1.65ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **88.35%** | 1,552 | 0.63ms | 0.84ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **98.75%** | 1,145 | 0.86ms | 1.05ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **87.00%** | 1,408 | 0.70ms | 0.87ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **87.10%** | 1,192 | 0.82ms | 1.09ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **99.40%** | 1,080 | 0.91ms | 1.04ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **99.70%** | 944 | 1.04ms | 1.26ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **87.50%** | 1,011 | 0.97ms | 1.10ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.40%** | 974 | 1.01ms | 1.13ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **99.95%** | 867 | 1.14ms | 1.32ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **99.80%** | 948 | 1.04ms | 1.32ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.70%** | 1,344 | 0.73ms | 0.79ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.85%** | 1,054 | 0.93ms | 1.11ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **99.75%** | 1,036 | 0.95ms | 1.18ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **99.75%** | 917 | 1.07ms | 1.19ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **85.00%** | 1,172 | 0.84ms | 1.06ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **24.20%** | 1,811 | 0.54ms | 0.59ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.25%** | 1,427 | 0.69ms | 0.75ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **81.10%** | 1,254 | 0.78ms | 0.89ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **84.30%** | 1,044 | 0.94ms | 1.14ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **41.00%** | 1,577 | 0.62ms | 0.72ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **26.95%** | 1,719 | 0.57ms | 0.74ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **80.60%** | 1,488 | 0.66ms | 0.74ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **84.80%** | 1,268 | 0.77ms | 0.83ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **42.45%** | 1,694 | 0.58ms | 0.64ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **35.55%** | 1,967 | 0.49ms | 0.63ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **96.45%** | 1,389 | 0.71ms | 0.95ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **99.15%** | 1,240 | 0.79ms | 0.98ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.60%** | 946 | 1.04ms | 1.19ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **62.30%** | 1,776 | 0.55ms | 0.69ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **76.80%** | 1,477 | 0.66ms | 0.90ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.45%** | 2,007 | 0.48ms | 0.52ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **98.50%** | 1,666 | 0.59ms | 0.74ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.60%** | 1,407 | 0.70ms | 0.78ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.20%** | 1,836 | 0.53ms | 0.73ms | 2.67 MB | **29.3×** |
| **pq4** | `pq4` | None | **36.30%** | 1,678 | 0.58ms | 0.64ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **98.15%** | 1,381 | 0.71ms | 0.78ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.65%** | 1,154 | 0.85ms | 0.94ms | 2.52 MB | **31.1×** |

---

## 1b. A/B: v4.2.4 → v4.2.5 (same machine, same dataset, same harness)

Both binaries were run with `benchmarks/run_quantization_sweep.py` (distinct port pair per configuration, calibration via `RebuildIndex` for trained modes). Over the 32 shared configurations: **search QPS ×1.95** (geometric mean), **recall@10 +1.5 pp** on average (up to +6.7 pp; never lower by more than 0.25 pp), **ingest ×1.54** (median).

Where the change comes from:

* **Low-bit kernels**: `extreme` and `ternary` score through a per-query nibble table and `medium_plus` through a NEON block kernel. `ternary` QPS ×10–×14, `extreme` ×2.4–×3.2, `medium_plus` ×2.4–×3.1, with the same recall; ingest of those modes ×1.5–×2.2 because graph construction uses the same kernels.
* **Multi-shard recall fix**: v4.2.4 divided `ef_search` by √(shard count) per shard (ef ≈ 29 with 12 shards). v4.2.5 runs the full beam on every shard: full-precision recall@10 92.90% → 99.05%. At the same `ef_search` that costs QPS for `none` (1,145 → 808); at matched recall v4.2.5 is faster (99.15% needed ef_search = 400 and gave 743 QPS in v4.2.4).
* **Hot-loop work** (flat layer-0 arena, per-query kernel dispatch, adaptive pruning, `Arc` metadata, WAL group commit) — see `CHANGELOG.md`.

A HyperspaceDB Docker container was running on the host during both runs, so differences of a few percent are within noise.

| Configuration | Recall@10 v4.2.4 → v4.2.5 | Search QPS v4.2.4 → v4.2.5 | QPS Δ | P95 v4.2.4 → v4.2.5 | Ingest v/s v4.2.4 → v4.2.5 | Index drain v4.2.4 → v4.2.5 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | 92.90% → **99.05%** | 1,145 → **808** | ×0.71 | 1.03 → 1.34 ms | 13,205 → 16,375 | 0.3 → 0.3 s |
| **none_ef400** | 99.15% → **99.90%** | 743 → **698** | ×0.94 | 1.49 → 1.65 ms | 13,563 → 16,829 | 0.3 → 0.3 s |
| **medium** | 84.65% → **88.35%** | 1,715 → **1,552** | ×0.91 | 0.77 → 0.84 ms | 11,801 → 14,495 | 0.3 → 0.3 s |
| **medium_rerank4** | 97.15% → **98.75%** | 1,269 → **1,145** | ×0.90 | 0.98 → 1.05 ms | 11,390 → 14,919 | 0.3 → 0.3 s |
| **medium_plus** | 83.90% → **87.00%** | 597 → **1,408** | ×2.36 | 1.97 → 0.87 ms | 4,211 → 9,058 | 0.6 → 0.3 s |
| **medium_plus_ef400** | 86.45% → **87.10%** | 383 → **1,192** | ×3.12 | 2.94 → 1.09 ms | 4,085 → 9,047 | 0.6 → 0.3 s |
| **medium_plus_rerank4** | 97.50% → **99.40%** | 372 → **1,080** | ×2.90 | 3.06 → 1.04 ms | 3,983 → 9,035 | 0.7 → 0.3 s |
| **medium_plus_rerank8** | 98.90% → **99.70%** | 353 → **944** | ×2.68 | 3.10 → 1.26 ms | 3,928 → 8,813 | 0.7 → 0.3 s |
| **turbo** | 85.25% → **87.50%** | 1,199 → **1,011** | ×0.84 | 0.90 → 1.10 ms | 5,154 → 7,904 | 0.3 → 0.3 s |
| **turbo_ef400** | 87.45% → **87.40%** | 989 → **974** | ×0.98 | 1.08 → 1.13 ms | 5,266 → 7,991 | 0.3 → 0.3 s |
| **turbo_rerank4** | 99.15% → **99.95%** | 916 → **867** | ×0.95 | 1.21 → 1.32 ms | 5,030 → 7,683 | 0.3 → 0.3 s |
| **turbo_twopass_raw40** | 99.20% → **99.80%** | 983 → **948** | ×0.96 | 1.17 → 1.32 ms | 4,950 → 7,714 | 0.3 → 0.3 s |
| **ternary** | 53.95% → **53.70%** | 97 → **1,344** | ×13.85 | 12.25 → 0.79 ms | 1,052 → 1,737 | 10.0 → 0.7 s |
| **ternary_ef400** | 53.75% → **53.85%** | 89 → **1,054** | ×11.83 | 13.33 → 1.11 ms | 861 → 1,654 | 6.2 → 0.8 s |
| **ternary_rerank4** | 99.15% → **99.75%** | 81 → **1,036** | ×12.82 | 16.50 → 1.18 ms | 914 → 1,646 | 7.9 → 0.9 s |
| **ternary_rerank8** | 99.65% → **99.75%** | 90 → **917** | ×10.16 | 13.76 → 1.19 ms | 1,056 → 1,623 | 10.9 → 0.7 s |
| **ternary_twopass_raw40** | 84.40% → **85.00%** | 90 → **1,172** | ×12.99 | 14.14 → 1.06 ms | 934 → 1,654 | 8.3 → 1.0 s |
| **opq** | 24.00% → **24.20%** | 1,730 → **1,811** | ×1.05 | 0.62 → 0.59 ms | 3,749 → 4,925 | 0.3 → 0.3 s |
| **opq_ef400** | 24.25% → **24.25%** | 1,470 → **1,427** | ×0.97 | 0.76 → 0.75 ms | 3,843 → 5,093 | 0.3 → 0.3 s |
| **opq_rerank4** | 77.05% → **81.10%** | 1,361 → **1,254** | ×0.92 | 0.81 → 0.89 ms | 3,777 → 5,143 | 0.3 → 0.3 s |
| **opq_rerank8** | 82.75% → **84.30%** | 1,315 → **1,044** | ×0.79 | 0.94 → 1.14 ms | 3,743 → 5,225 | 0.3 → 0.3 s |
| **opq_twopass_raw40** | 40.95% → **41.00%** | 1,730 → **1,577** | ×0.91 | 0.64 → 0.72 ms | 3,832 → 5,252 | 0.3 → 0.3 s |
| **pq** | 26.85% → **26.95%** | 1,960 → **1,719** | ×0.88 | 0.62 → 0.74 ms | 6,625 → 7,145 | 0.3 → 0.0 s |
| **pq_rerank4** | 76.00% → **80.60%** | 1,521 → **1,488** | ×0.98 | 0.88 → 0.74 ms | 6,880 → 7,309 | 0.0 → 0.0 s |
| **pq_rerank8** | 82.70% → **84.80%** | 1,307 → **1,268** | ×0.97 | 0.91 → 0.83 ms | 6,836 → 7,284 | 0.3 → 0.0 s |
| **pq_twopass_raw40** | 41.95% → **42.45%** | 1,579 → **1,694** | ×1.07 | 0.88 → 0.64 ms | 6,988 → 7,249 | 0.3 → 0.3 s |
| **extreme** | 34.55% → **35.55%** | 811 → **1,967** | ×2.43 | 1.48 → 0.63 ms | 5,169 → 11,330 | 0.3 → 0.3 s |
| **extreme_rerank4** | 89.75% → **96.45%** | 500 → **1,389** | ×2.78 | 2.16 → 0.95 ms | 5,291 → 11,698 | 0.3 → 0.3 s |
| **extreme_rerank8** | 97.45% → **99.15%** | 447 → **1,240** | ×2.77 | 2.84 → 0.98 ms | 5,152 → 11,204 | 0.3 → 0.3 s |
| **extreme_rerank16** | 99.35% → **99.60%** | 366 → **946** | ×2.59 | 3.34 → 1.19 ms | 5,161 → 11,557 | 0.3 → 0.3 s |
| **extreme_twopass_raw40** | 61.65% → **62.30%** | 563 → **1,776** | ×3.16 | 2.12 → 0.69 ms | 5,155 → 11,657 | 0.3 → 0.3 s |
| **extreme_twopass_raw80** | 76.40% → **76.80%** | 469 → **1,477** | ×3.15 | 2.28 → 0.90 ms | 5,144 → 11,762 | 0.3 → 0.3 s |

`rabitq` and `pq4` are new in v4.2.5 and have no baseline; see section 6.

### Online auto-calibration (no `RebuildIndex`)

The trained modes were also measured without the explicit `RebuildIndex` step (`HS_BENCH_CALIBRATE=0`), i.e. relying on online calibration only. With 12 shards each holds ~1,670 vectors: enough for `rabitq` / `pq4` (threshold 1,024) but below the 4,096-vector floor of `pq` / `opq`, which therefore keep their default codebook here.

| Configuration | Default codebook (v4.2.5 before auto-calibration) | Auto-calibrated online | After `RebuildIndex` |
| :--- | :---: | :---: | :---: |
| **rabitq** | 35.50% @ 1,943 QPS | **36.70%** @ 2,360 QPS | 35.45% @ 2,007 QPS |
| **rabitq_rerank4** | 98.30% @ 1,499 QPS | **98.15%** @ 1,690 QPS | 98.50% @ 1,666 QPS |
| **rabitq_rerank8** | 99.55% @ 1,238 QPS | **99.45%** @ 1,404 QPS | 99.60% @ 1,407 QPS |
| **rabitq_twopass_raw40** | 61.35% @ 1,845 QPS | **63.20%** @ 2,068 QPS | 62.20% @ 1,836 QPS |
| **pq4** | 25.65% @ 1,504 QPS | **35.80%** @ 1,641 QPS | 36.30% @ 1,678 QPS |
| **pq4_rerank4** | 90.85% @ 1,186 QPS | **97.60%** @ 1,315 QPS | 98.15% @ 1,381 QPS |
| **pq4_rerank8** | 96.65% @ 901 QPS | **98.25%** @ 1,139 QPS | 99.65% @ 1,154 QPS |

---

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.22s | 16,375 v/s | 652.1 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 1.19s | 16,829 v/s | 665.7 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 1.38s | 14,495 v/s | 732.5 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.34s | 14,919 v/s | 728.5 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 2.21s | 9,058 v/s | 642.8 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 2.21s | 9,047 v/s | 635.0 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 2.21s | 9,035 v/s | 659.2 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 2.27s | 8,813 v/s | 661.4 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 2.53s | 7,904 v/s | 664.0 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 2.50s | 7,991 v/s | 645.8 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 2.60s | 7,683 v/s | 657.8 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 2.59s | 7,714 v/s | 664.8 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 11.51s | 1,737 v/s | 838.7 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 12.10s | 1,654 v/s | 845.8 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 12.15s | 1,646 v/s | 835.7 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 12.32s | 1,623 v/s | 823.2 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 12.09s | 1,654 v/s | 831.8 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 4.06s | 4,925 v/s | 1137.3 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 3.93s | 5,093 v/s | 1122.8 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 3.89s | 5,143 v/s | 1136.3 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 3.83s | 5,225 v/s | 1109.3 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 3.81s | 5,252 v/s | 1146.8 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 2.80s | 7,145 v/s | 968.3 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 2.74s | 7,309 v/s | 969.5 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 2.75s | 7,284 v/s | 966.3 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 2.76s | 7,249 v/s | 960.3 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 1.77s | 11,330 v/s | 581.2 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 1.71s | 11,698 v/s | 596.6 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 1.79s | 11,204 v/s | 594.5 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 1.73s | 11,557 v/s | 585.6 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 1.72s | 11,657 v/s | 589.4 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 1.70s | 11,762 v/s | 589.7 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 3.23s | 6,191 v/s | 1123.0 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 3.35s | 5,961 v/s | 1130.3 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 3.30s | 6,060 v/s | 1106.1 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 3.27s | 6,123 v/s | 1108.0 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 2.23s | 8,977 v/s | 1078.7 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 2.14s | 9,343 v/s | 1082.0 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 2.16s | 9,255 v/s | 1087.4 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **35.55%** | 1,967 | 0.49ms | 0.85ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **96.45%** | 1,389 | 0.71ms | 1.05ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **99.15%** | 1,240 | 0.79ms | 1.11ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.60%** | 946 | 1.04ms | 1.56ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **62.30%** | 1,776 | 0.55ms | 1.08ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **76.80%** | 1,477 | 0.66ms | 1.19ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **24.20%** | 1,811 | 0.54ms | 0.64ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.25%** | 1,427 | 0.69ms | 0.78ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **81.10%** | 1,254 | 0.78ms | 0.95ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **84.30%** | 1,044 | 0.94ms | 1.21ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **41.00%** | 1,577 | 0.62ms | 0.76ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **26.95%** | 1,719 | 0.57ms | 0.91ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **80.60%** | 1,488 | 0.66ms | 0.79ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **84.80%** | 1,268 | 0.77ms | 0.87ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **42.45%** | 1,694 | 0.58ms | 0.68ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.70%** | 1,344 | 0.73ms | 0.81ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.85%** | 1,054 | 0.93ms | 1.25ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **99.75%** | 1,036 | 0.95ms | 1.38ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **99.75%** | 917 | 1.07ms | 1.23ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **85.00%** | 1,172 | 0.84ms | 1.34ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.45%** | 2,007 | 0.48ms | 0.54ms | 5.4s |
| **rabitq_rerank4** | **98.50%** | 1,666 | 0.59ms | 0.96ms | 5.6s |
| **rabitq_rerank8** | **99.60%** | 1,407 | 0.70ms | 0.83ms | 5.1s |
| **rabitq_twopass_raw40** | **62.20%** | 1,836 | 0.53ms | 0.88ms | 5.3s |
| **pq4** | **36.30%** | 1,678 | 0.58ms | 0.66ms | 2.6s |
| **pq4_rerank4** | **98.15%** | 1,381 | 0.71ms | 0.80ms | 2.6s |
| **pq4_rerank8** | **99.65%** | 1,154 | 0.85ms | 0.99ms | 2.7s |

---

## 7. Architectural Recommendation Matrix

Recall and QPS below are the measured values from section 1.

| Workload / Use Case | Recommended Config | Recall@10 | Search QPS | Compression | Trade-off |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **Max precision (medical / financial / legal)** | `none` (ef_search 100) or `turbo_rerank4` | 99.05 – 99.95% | 808 – 867 | 1× – 7.9× | No calibration. |
| **Memory-optimal L2 / cosine** | `rabitq_rerank4` | 98.5% | 1,666 | 29× | Calibrates online; `RebuildIndex` for best recall. |
| **Write-heavy L2 / cosine** | `pq4_rerank8` | 99.65% | 1,154 | 31× | ~9k vec/s ingest; calibrates online. |
| **Any metric, compact** | `extreme_rerank8` | 99.15% | 1,240 | 31× | No calibration; works on hyperbolic metrics. |
| **Any metric, balanced** | `ternary_rerank4` or `medium_plus_rerank4` | 99.4 – 99.75% | 1,036 – 1,080 | 5.3× – 15.3× | No calibration. |
| **Ultra-dense (L2 / cosine)** | `pq_rerank8` / `opq_rerank8` | 84.3 – 84.8% | 1,044 – 1,268 | 60× | 16-D sub-spaces are too coarse for high recall on this data. |
