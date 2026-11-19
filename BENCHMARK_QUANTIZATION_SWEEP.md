# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (release build, Apple M4 Pro, 12 shards = 1 per core); baseline = v4.2.4 (`2fa9d9f`) with the `flat_l0.bin` deadlock fix back-ported so it can complete the sweep
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-19 16:19:45

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **99.00%** | 796 | 1.24ms | 1.42ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **99.20%** | 649 | 1.52ms | 1.74ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **88.15%** | 1,492 | 0.66ms | 0.91ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **99.70%** | 1,120 | 0.88ms | 1.09ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **86.80%** | 435 | 2.29ms | 2.65ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **87.10%** | 381 | 2.61ms | 2.86ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **99.75%** | 365 | 2.72ms | 3.19ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **99.70%** | 367 | 2.71ms | 2.90ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **87.45%** | 1,106 | 0.89ms | 1.09ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.50%** | 940 | 1.05ms | 1.28ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **99.80%** | 919 | 1.07ms | 1.33ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **99.45%** | 1,020 | 0.96ms | 1.15ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.85%** | 116 | 8.61ms | 9.62ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.85%** | 115 | 8.67ms | 9.49ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **99.80%** | 114 | 8.76ms | 10.80ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **99.95%** | 98 | 10.14ms | 12.15ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **84.95%** | 93 | 10.69ms | 12.60ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **24.20%** | 1,244 | 0.78ms | 0.88ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.25%** | 1,220 | 0.80ms | 1.18ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **81.05%** | 934 | 1.05ms | 1.26ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **84.30%** | 791 | 1.24ms | 1.44ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **41.00%** | 1,030 | 0.95ms | 1.15ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **26.95%** | 1,264 | 0.77ms | 0.91ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **80.60%** | 1,042 | 0.94ms | 1.09ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **84.80%** | 889 | 1.10ms | 1.27ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **42.45%** | 1,116 | 0.87ms | 1.06ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **35.95%** | 417 | 2.38ms | 2.74ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **96.35%** | 381 | 2.60ms | 3.09ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **98.65%** | 305 | 3.26ms | 3.90ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.60%** | 292 | 3.41ms | 3.96ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **62.45%** | 377 | 2.63ms | 3.24ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **76.65%** | 358 | 2.77ms | 3.34ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.45%** | 1,534 | 0.63ms | 0.81ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **98.50%** | 1,388 | 0.70ms | 1.15ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.60%** | 1,029 | 0.95ms | 1.38ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.20%** | 1,667 | 0.58ms | 0.95ms | 2.67 MB | **29.3×** |
| **pq4** | `pq4` | None | **36.30%** | 1,388 | 0.70ms | 1.02ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **98.15%** | 1,091 | 0.90ms | 1.13ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.65%** | 1,062 | 0.93ms | 1.05ms | 2.52 MB | **31.1×** |

---

## 1b. A/B: v4.2.4 → v4.2.5 (same machine, same dataset, same harness)

Both binaries were run with `benchmarks/run_quantization_sweep.py` (distinct port pair per configuration, calibration via `RebuildIndex` for trained modes).

**Why recall@10 rose and QPS fell at the same `ef_search`.** v4.2.4 divided `ef_search` by √(shard count) on every shard and asked each shard for only `top_k + 4` candidates. With 12 shards an `ef_search = 100` search ran each shard at ef ≈ 29, which is why full-precision recall@10 was 92.9% (v4.2.3: 99.35%). v4.2.5 runs the full `ef_search` on every shard and returns `max(top_k + 6, 2·top_k)` candidates per shard, so each query does ~3.5× more beam work.

Over the 32 shared configurations: recall@10 **+1.5 pp on average** (up to +6.6 pp for `extreme_rerank4`, never lower by more than 0.1 pp), search QPS **×0.83** (geometric mean), ingest **×1.15** (median). At matched recall v4.2.5 is as fast or faster for most modes:

| Mode | v4.2.4 | v4.2.5 |
| :--- | :--- | :--- |
| `none` | 99.15% @ 743 QPS (needs ef_search = 400) | **99.00% @ 796 QPS** (ef_search = 100) |
| `turbo` + rerank ×4 | 99.15% @ 916 QPS | **99.80% @ 919 QPS** |
| `medium_plus` | 98.90% @ 353 QPS (rerank ×8) | **99.75% @ 365 QPS** (rerank ×4) |
| `medium` + rerank ×4 | 97.15% @ 1,269 QPS | **99.70% @ 1,120 QPS** |
| `extreme` | 99.35% @ 366 QPS (rerank ×16) | 99.60% @ 292 QPS (rerank ×16) |

`extreme` (1-bit ADC) is the exception: its distance kernel is the slowest per candidate, so the wider beam costs it ~20% QPS at the same recall. A HyperspaceDB Docker container was running on the host during both runs, so single-digit percentage differences are within noise.

| Configuration | Recall@10 v4.2.4 → v4.2.5 | Search QPS v4.2.4 → v4.2.5 | QPS Δ | P95 v4.2.4 → v4.2.5 | Ingest v/s v4.2.4 → v4.2.5 | Index drain v4.2.4 → v4.2.5 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | 92.90% → **99.00%** | 1,145 → 796 | ×0.70 | 1.03 → 1.42 ms | 13,205 → 16,351 | 0.3 → 0.3 s |
| **none_ef400** | 99.15% → **99.20%** | 743 → 649 | ×0.87 | 1.49 → 1.74 ms | 13,563 → 16,274 | 0.3 → 0.3 s |
| **medium** | 84.65% → **88.15%** | 1,715 → 1,492 | ×0.87 | 0.77 → 0.91 ms | 11,801 → 14,959 | 0.3 → 0.3 s |
| **medium_rerank4** | 97.15% → **99.70%** | 1,269 → 1,120 | ×0.88 | 0.98 → 1.09 ms | 11,390 → 15,011 | 0.3 → 0.3 s |
| **medium_plus** | 83.90% → **86.80%** | 597 → 435 | ×0.73 | 1.97 → 2.65 ms | 4,211 → 5,914 | 0.6 → 0.3 s |
| **medium_plus_ef400** | 86.45% → **87.10%** | 383 → 381 | ×1.00 | 2.94 → 2.86 ms | 4,085 → 5,953 | 0.6 → 0.3 s |
| **medium_plus_rerank4** | 97.50% → **99.75%** | 372 → 365 | ×0.98 | 3.06 → 3.19 ms | 3,983 → 5,947 | 0.7 → 0.3 s |
| **medium_plus_rerank8** | 98.90% → **99.70%** | 353 → 367 | ×1.04 | 3.10 → 2.90 ms | 3,928 → 5,766 | 0.7 → 0.3 s |
| **turbo** | 85.25% → **87.45%** | 1,199 → 1,106 | ×0.92 | 0.90 → 1.09 ms | 5,154 → 7,783 | 0.3 → 0.3 s |
| **turbo_ef400** | 87.45% → **87.50%** | 989 → 940 | ×0.95 | 1.08 → 1.28 ms | 5,266 → 7,516 | 0.3 → 0.3 s |
| **turbo_rerank4** | 99.15% → **99.80%** | 916 → 919 | ×1.00 | 1.21 → 1.33 ms | 5,030 → 7,391 | 0.3 → 0.3 s |
| **turbo_twopass_raw40** | 99.20% → **99.45%** | 983 → 1,020 | ×1.04 | 1.17 → 1.15 ms | 4,950 → 7,614 | 0.3 → 0.3 s |
| **ternary** | 53.95% → **53.85%** | 97 → 116 | ×1.19 | 12.25 → 9.62 ms | 1,052 → 1,032 | 10.0 → 2.0 s |
| **ternary_ef400** | 53.75% → **53.85%** | 89 → 115 | ×1.29 | 13.33 → 9.49 ms | 861 → 982 | 6.2 → 1.7 s |
| **ternary_rerank4** | 99.15% → **99.80%** | 81 → 114 | ×1.41 | 16.50 → 10.80 ms | 914 → 971 | 7.9 → 1.9 s |
| **ternary_rerank8** | 99.65% → **99.95%** | 90 → 98 | ×1.09 | 13.76 → 12.15 ms | 1,056 → 895 | 10.9 → 1.9 s |
| **ternary_twopass_raw40** | 84.40% → **84.95%** | 90 → 93 | ×1.04 | 14.14 → 12.60 ms | 934 → 841 | 8.3 → 2.4 s |
| **opq** | 24.00% → **24.20%** | 1,730 → 1,244 | ×0.72 | 0.62 → 0.88 ms | 3,749 → 4,789 | 0.3 → 0.3 s |
| **opq_ef400** | 24.25% → **24.25%** | 1,470 → 1,220 | ×0.83 | 0.76 → 1.18 ms | 3,843 → 4,739 | 0.3 → 0.3 s |
| **opq_rerank4** | 77.05% → **81.05%** | 1,361 → 934 | ×0.69 | 0.81 → 1.26 ms | 3,777 → 4,858 | 0.3 → 0.3 s |
| **opq_rerank8** | 82.75% → **84.30%** | 1,315 → 791 | ×0.60 | 0.94 → 1.44 ms | 3,743 → 3,290 | 0.3 → 0.3 s |
| **opq_twopass_raw40** | 40.95% → **41.00%** | 1,730 → 1,030 | ×0.60 | 0.64 → 1.15 ms | 3,832 → 3,160 | 0.3 → 0.3 s |
| **pq** | 26.85% → **26.95%** | 1,960 → 1,264 | ×0.64 | 0.62 → 0.91 ms | 6,625 → 6,521 | 0.3 → 0.3 s |
| **pq_rerank4** | 76.00% → **80.60%** | 1,521 → 1,042 | ×0.69 | 0.88 → 1.09 ms | 6,880 → 6,992 | 0.0 → 0.3 s |
| **pq_rerank8** | 82.70% → **84.80%** | 1,307 → 889 | ×0.68 | 0.91 → 1.27 ms | 6,836 → 7,114 | 0.3 → 0.3 s |
| **pq_twopass_raw40** | 41.95% → **42.45%** | 1,579 → 1,116 | ×0.71 | 0.88 → 1.06 ms | 6,988 → 6,877 | 0.3 → 0.3 s |
| **extreme** | 34.55% → **35.95%** | 811 → 417 | ×0.51 | 1.48 → 2.74 ms | 5,169 → 5,506 | 0.3 → 0.3 s |
| **extreme_rerank4** | 89.75% → **96.35%** | 500 → 381 | ×0.76 | 2.16 → 3.09 ms | 5,291 → 5,837 | 0.3 → 0.3 s |
| **extreme_rerank8** | 97.45% → **98.65%** | 447 → 305 | ×0.68 | 2.84 → 3.90 ms | 5,152 → 5,776 | 0.3 → 0.3 s |
| **extreme_rerank16** | 99.35% → **99.60%** | 366 → 292 | ×0.80 | 3.34 → 3.96 ms | 5,161 → 5,909 | 0.3 → 0.3 s |
| **extreme_twopass_raw40** | 61.65% → **62.45%** | 563 → 377 | ×0.67 | 2.12 → 3.24 ms | 5,155 → 5,896 | 0.3 → 0.3 s |
| **extreme_twopass_raw80** | 76.40% → **76.65%** | 469 → 358 | ×0.76 | 2.28 → 3.34 ms | 5,144 → 5,901 | 0.3 → 0.3 s |

`rabitq` and `pq4` are new in v4.2.5 and have no baseline; see section 6.

---

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.22s | 16,351 v/s | 672.8 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 1.23s | 16,274 v/s | 675.6 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 1.34s | 14,959 v/s | 718.7 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.33s | 15,011 v/s | 726.3 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 3.38s | 5,914 v/s | 672.1 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 3.36s | 5,953 v/s | 648.0 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 3.36s | 5,947 v/s | 650.5 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 3.47s | 5,766 v/s | 628.3 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 2.57s | 7,783 v/s | 648.6 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 2.66s | 7,516 v/s | 650.1 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 2.71s | 7,391 v/s | 641.8 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 2.63s | 7,614 v/s | 634.5 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 19.38s | 1,032 v/s | 828.1 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 20.37s | 982 v/s | 837.4 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 20.60s | 971 v/s | 815.0 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 22.36s | 895 v/s | 813.9 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 23.78s | 841 v/s | 834.3 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 4.18s | 4,789 v/s | 1149.9 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 4.22s | 4,739 v/s | 1113.8 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 4.12s | 4,858 v/s | 1115.0 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 6.08s | 3,290 v/s | 1148.8 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 6.33s | 3,160 v/s | 1120.4 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 3.07s | 6,521 v/s | 977.9 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 2.86s | 6,992 v/s | 971.5 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 2.81s | 7,114 v/s | 970.5 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 2.91s | 6,877 v/s | 981.2 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 3.63s | 5,506 v/s | 568.9 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 3.43s | 5,837 v/s | 573.6 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 3.46s | 5,776 v/s | 592.9 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 3.38s | 5,909 v/s | 586.0 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 3.39s | 5,896 v/s | 596.2 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 3.39s | 5,901 v/s | 587.1 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 4.17s | 4,794 v/s | 1082.9 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 3.84s | 5,211 v/s | 1107.0 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 3.35s | 5,968 v/s | 1079.5 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 3.67s | 5,443 v/s | 1099.0 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 1.76s | 11,386 v/s | 1054.0 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 1.78s | 11,243 v/s | 1057.1 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 1.86s | 10,737 v/s | 1014.6 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **35.95%** | 417 | 2.38ms | 3.11ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **96.35%** | 381 | 2.60ms | 3.58ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **98.65%** | 305 | 3.26ms | 4.49ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.60%** | 292 | 3.41ms | 4.25ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **62.45%** | 377 | 2.63ms | 3.30ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **76.65%** | 358 | 2.77ms | 3.40ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **24.20%** | 1,244 | 0.78ms | 0.96ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.25%** | 1,220 | 0.80ms | 1.49ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **81.05%** | 934 | 1.05ms | 1.34ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **84.30%** | 791 | 1.24ms | 1.51ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **41.00%** | 1,030 | 0.95ms | 1.22ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **26.95%** | 1,264 | 0.77ms | 0.94ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **80.60%** | 1,042 | 0.94ms | 1.20ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **84.80%** | 889 | 1.10ms | 1.41ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **42.45%** | 1,116 | 0.87ms | 1.10ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.85%** | 116 | 8.61ms | 11.94ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.85%** | 115 | 8.67ms | 11.06ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **99.80%** | 114 | 8.76ms | 12.37ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **99.95%** | 98 | 10.14ms | 12.80ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **84.95%** | 93 | 10.69ms | 13.17ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.45%** | 1,534 | 0.63ms | 1.00ms | 7.1s |
| **rabitq_rerank4** | **98.50%** | 1,388 | 0.70ms | 1.30ms | 6.6s |
| **rabitq_rerank8** | **99.60%** | 1,029 | 0.95ms | 1.64ms | 5.5s |
| **rabitq_twopass_raw40** | **62.20%** | 1,667 | 0.58ms | 1.10ms | 6.0s |
| **pq4** | **36.30%** | 1,388 | 0.70ms | 1.21ms | 2.7s |
| **pq4_rerank4** | **98.15%** | 1,091 | 0.90ms | 3.84ms | 2.7s |
| **pq4_rerank8** | **99.65%** | 1,062 | 0.93ms | 1.11ms | 2.8s |

---

## 7. Architectural Recommendation Matrix

Recall and QPS below are the measured values from section 1.

| Workload / Use Case | Recommended Config | Recall@10 | Search QPS | Compression | Trade-off |
| :--- | :--- | :---: | :---: | :---: | :--- |
| **Max precision (medical / financial / legal)** | `none` (ef_search 100) or `medium_rerank4` | 99.0 – 99.7% | 796 – 1,120 | 1× – 4× | Highest RAM; no calibration. |
| **Memory-optimal L2 / cosine** | `rabitq_rerank4` | 98.5% | 1,388 | 29× | Needs one `RebuildIndex` after ingest. |
| **Write-heavy L2 / cosine** | `pq4_rerank8` | 99.65% | 1,062 | 31× | Fastest compressed ingest (~11k vec/s); needs calibration. |
| **Near-lossless at 8× less RAM** | `turbo_rerank4` | 99.8% | 919 | 7.9× | No calibration. |
| **Hyperbolic / hybrid metrics** | `medium_plus_rerank4` or `ternary_rerank4` | 99.75 – 99.8% | 114 – 365 | 5.3× – 15.3× | Work with every metric. |
| **Edge / RAM-constrained, any metric** | `extreme_rerank8` | 98.65% | 305 | 31× | Slowest per-candidate kernel; no calibration. |
| **Ultra-dense (L2 / cosine)** | `pq_rerank8` / `opq_rerank8` | 84.3 – 84.8% | 791 – 889 | 60× | 16-D sub-spaces are too coarse for high recall on this data. |
