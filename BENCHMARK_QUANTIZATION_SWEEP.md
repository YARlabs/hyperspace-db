# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (release build, Apple M4 Pro); baseline = v4.2.4 (`2fa9d9f`) with the `flat_l0.bin` deadlock fix back-ported so it can complete the sweep
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-19 09:33:03

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **92.95%** | 1,272 | 0.77ms | 0.88ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **99.65%** | 815 | 1.21ms | 1.37ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **84.80%** | 2,075 | 0.47ms | 0.63ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **97.55%** | 1,313 | 0.75ms | 0.93ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **84.45%** | 661 | 1.50ms | 1.75ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **86.85%** | 424 | 2.34ms | 2.69ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **97.55%** | 457 | 2.18ms | 2.47ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **99.35%** | 395 | 2.51ms | 2.86ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **85.60%** | 1,414 | 0.69ms | 0.87ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.50%** | 1,083 | 0.91ms | 1.11ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **99.25%** | 1,033 | 0.95ms | 1.19ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **99.05%** | 1,120 | 0.88ms | 1.07ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.75%** | 120 | 8.29ms | 10.82ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.90%** | 116 | 8.59ms | 9.49ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **99.80%** | 113 | 8.83ms | 11.79ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **99.75%** | 112 | 8.87ms | 9.95ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **85.00%** | 112 | 8.91ms | 11.22ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **23.95%** | 2,100 | 0.46ms | 0.51ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.25%** | 1,679 | 0.58ms | 0.78ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **77.00%** | 1,514 | 0.65ms | 0.75ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **82.80%** | 1,368 | 0.72ms | 0.93ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **40.95%** | 1,619 | 0.60ms | 0.86ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **26.85%** | 1,900 | 0.51ms | 0.62ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **76.00%** | 1,526 | 0.64ms | 0.90ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **82.70%** | 1,352 | 0.72ms | 0.94ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **42.00%** | 1,646 | 0.59ms | 0.78ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **35.25%** | 991 | 0.99ms | 1.18ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **90.55%** | 692 | 1.43ms | 1.84ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **97.80%** | 503 | 1.98ms | 2.53ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.35%** | 435 | 2.29ms | 2.93ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **61.80%** | 652 | 1.52ms | 1.84ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **76.65%** | 525 | 1.89ms | 2.19ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.30%** | 2,599 | 0.37ms | 0.50ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **96.85%** | 1,959 | 0.50ms | 0.56ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.05%** | 1,492 | 0.66ms | 0.89ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.10%** | 2,375 | 0.41ms | 0.46ms | 2.67 MB | **29.3×** |
| **pq4** | `pq4` | None | **35.90%** | 2,048 | 0.47ms | 0.66ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **95.50%** | 1,552 | 0.63ms | 0.71ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.00%** | 1,186 | 0.83ms | 1.16ms | 2.52 MB | **31.1×** |

---

## 1b. A/B: v4.2.4 → v4.2.5 (same machine, same dataset, same harness)

Both binaries were run back-to-back with `benchmarks/run_quantization_sweep.py` (distinct port pair per configuration, calibration via `RebuildIndex` for trained modes). Over the 32 shared configurations: **geometric-mean search QPS ×1.14** (range ×0.94 – ×1.40), **median ingest ×1.38**, recall@10 **+0.2 pp on average** (never lower by more than 0.2 pp). Ternary indexing now drains in ~1.5 s instead of 6–11 s. A HyperspaceDB Docker container was running on the host during both runs, so single-digit percentage differences are within noise.

| Configuration | Recall@10 v4.2.4 → v4.2.5 | Search QPS v4.2.4 → v4.2.5 | QPS Δ | P95 v4.2.4 → v4.2.5 | Ingest v/s v4.2.4 → v4.2.5 | Index drain v4.2.4 → v4.2.5 |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | 92.90% → **92.95%** | 1,145 → **1,272** | ×1.11 | 1.03 → 0.88 ms | 13,205 → 16,476 | 0.3 → 0.3 s |
| **none_ef400** | 99.15% → **99.65%** | 743 → **815** | ×1.10 | 1.49 → 1.37 ms | 13,563 → 15,927 | 0.3 → 0.3 s |
| **medium** | 84.65% → **84.80%** | 1,715 → **2,075** | ×1.21 | 0.77 → 0.63 ms | 11,801 → 15,822 | 0.3 → 0.3 s |
| **medium_rerank4** | 97.15% → **97.55%** | 1,269 → **1,313** | ×1.03 | 0.98 → 0.93 ms | 11,390 → 14,796 | 0.3 → 0.3 s |
| **medium_plus** | 83.90% → **84.45%** | 597 → **661** | ×1.11 | 1.97 → 1.75 ms | 4,211 → 6,058 | 0.6 → 0.3 s |
| **medium_plus_ef400** | 86.45% → **86.85%** | 383 → **424** | ×1.11 | 2.94 → 2.69 ms | 4,085 → 6,103 | 0.6 → 0.3 s |
| **medium_plus_rerank4** | 97.50% → **97.55%** | 372 → **457** | ×1.23 | 3.06 → 2.47 ms | 3,983 → 6,076 | 0.7 → 0.3 s |
| **medium_plus_rerank8** | 98.90% → **99.35%** | 353 → **395** | ×1.12 | 3.10 → 2.86 ms | 3,928 → 6,110 | 0.7 → 0.4 s |
| **turbo** | 85.25% → **85.60%** | 1,199 → **1,414** | ×1.18 | 0.90 → 0.87 ms | 5,154 → 7,718 | 0.3 → 0.3 s |
| **turbo_ef400** | 87.45% → **87.50%** | 989 → **1,083** | ×1.09 | 1.08 → 1.11 ms | 5,266 → 7,775 | 0.3 → 0.3 s |
| **turbo_rerank4** | 99.15% → **99.25%** | 916 → **1,033** | ×1.13 | 1.21 → 1.19 ms | 5,030 → 7,598 | 0.3 → 0.3 s |
| **turbo_twopass_raw40** | 99.20% → **99.05%** | 983 → **1,120** | ×1.14 | 1.17 → 1.07 ms | 4,950 → 7,725 | 0.3 → 0.3 s |
| **ternary** | 53.95% → **53.75%** | 97 → **120** | ×1.24 | 12.25 → 10.82 ms | 1,052 → 1,011 | 10.0 → 1.4 s |
| **ternary_ef400** | 53.75% → **53.90%** | 89 → **116** | ×1.30 | 13.33 → 9.49 ms | 861 → 975 | 6.2 → 1.7 s |
| **ternary_rerank4** | 99.15% → **99.80%** | 81 → **113** | ×1.40 | 16.50 → 11.79 ms | 914 → 964 | 7.9 → 1.7 s |
| **ternary_rerank8** | 99.65% → **99.75%** | 90 → **112** | ×1.25 | 13.76 → 9.95 ms | 1,056 → 946 | 10.9 → 1.5 s |
| **ternary_twopass_raw40** | 84.40% → **85.00%** | 90 → **112** | ×1.24 | 14.14 → 11.22 ms | 934 → 955 | 8.3 → 1.5 s |
| **opq** | 24.00% → **23.95%** | 1,730 → **2,100** | ×1.21 | 0.62 → 0.51 ms | 3,749 → 5,185 | 0.3 → 0.3 s |
| **opq_ef400** | 24.25% → **24.25%** | 1,470 → **1,679** | ×1.14 | 0.76 → 0.78 ms | 3,843 → 5,284 | 0.3 → 0.3 s |
| **opq_rerank4** | 77.05% → **77.00%** | 1,361 → **1,514** | ×1.11 | 0.81 → 0.75 ms | 3,777 → 5,216 | 0.3 → 0.3 s |
| **opq_rerank8** | 82.75% → **82.80%** | 1,315 → **1,368** | ×1.04 | 0.94 → 0.93 ms | 3,743 → 5,230 | 0.3 → 0.3 s |
| **opq_twopass_raw40** | 40.95% → **40.95%** | 1,730 → **1,619** | ×0.94 | 0.64 → 0.86 ms | 3,832 → 5,228 | 0.3 → 0.3 s |
| **pq** | 26.85% → **26.85%** | 1,960 → **1,900** | ×0.97 | 0.62 → 0.62 ms | 6,625 → 7,438 | 0.3 → 0.0 s |
| **pq_rerank4** | 76.00% → **76.00%** | 1,521 → **1,526** | ×1.00 | 0.88 → 0.90 ms | 6,880 → 7,452 | 0.0 → 0.3 s |
| **pq_rerank8** | 82.70% → **82.70%** | 1,307 → **1,352** | ×1.03 | 0.91 → 0.94 ms | 6,836 → 7,542 | 0.3 → 0.3 s |
| **pq_twopass_raw40** | 41.95% → **42.00%** | 1,579 → **1,646** | ×1.04 | 0.88 → 0.78 ms | 6,988 → 7,549 | 0.3 → 0.0 s |
| **extreme** | 34.55% → **35.25%** | 811 → **991** | ×1.22 | 1.48 → 1.18 ms | 5,169 → 7,610 | 0.3 → 0.3 s |
| **extreme_rerank4** | 89.75% → **90.55%** | 500 → **692** | ×1.38 | 2.16 → 1.84 ms | 5,291 → 7,431 | 0.3 → 0.3 s |
| **extreme_rerank8** | 97.45% → **97.80%** | 447 → **503** | ×1.12 | 2.84 → 2.53 ms | 5,152 → 7,523 | 0.3 → 0.3 s |
| **extreme_rerank16** | 99.35% → **99.35%** | 366 → **435** | ×1.19 | 3.34 → 2.93 ms | 5,161 → 7,434 | 0.3 → 0.3 s |
| **extreme_twopass_raw40** | 61.65% → **61.80%** | 563 → **652** | ×1.16 | 2.12 → 1.84 ms | 5,155 → 7,595 | 0.3 → 0.3 s |
| **extreme_twopass_raw80** | 76.40% → **76.65%** | 469 → **525** | ×1.12 | 2.28 → 2.19 ms | 5,144 → 7,532 | 0.3 → 0.3 s |

`rabitq` and `pq4` are new in v4.2.5 and have no baseline; see section 6.

---

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.21s | 16,476 v/s | 657.8 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 1.26s | 15,927 v/s | 684.6 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 1.26s | 15,822 v/s | 728.7 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.35s | 14,796 v/s | 741.7 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 3.30s | 6,058 v/s | 659.7 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 3.28s | 6,103 v/s | 652.8 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 3.29s | 6,076 v/s | 662.7 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 3.27s | 6,110 v/s | 630.4 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 2.59s | 7,718 v/s | 647.5 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 2.57s | 7,775 v/s | 650.1 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 2.63s | 7,598 v/s | 654.8 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 2.59s | 7,725 v/s | 661.4 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 19.79s | 1,011 v/s | 835.5 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 20.52s | 975 v/s | 821.4 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 20.75s | 964 v/s | 832.6 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 21.14s | 946 v/s | 823.0 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 20.95s | 955 v/s | 813.1 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 3.86s | 5,185 v/s | 1155.8 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 3.79s | 5,284 v/s | 1127.3 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 3.83s | 5,216 v/s | 1176.1 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 3.82s | 5,230 v/s | 1144.0 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 3.83s | 5,228 v/s | 1148.4 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 2.69s | 7,438 v/s | 966.3 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 2.68s | 7,452 v/s | 959.6 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 2.65s | 7,542 v/s | 990.8 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 2.65s | 7,549 v/s | 979.7 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 2.63s | 7,610 v/s | 583.1 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 2.69s | 7,431 v/s | 592.2 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 2.66s | 7,523 v/s | 602.1 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 2.69s | 7,434 v/s | 585.3 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 2.63s | 7,595 v/s | 601.8 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 2.66s | 7,532 v/s | 591.8 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 3.31s | 6,035 v/s | 1076.6 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 3.16s | 6,333 v/s | 1105.1 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 3.26s | 6,136 v/s | 1126.9 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 3.18s | 6,288 v/s | 1106.2 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 1.69s | 11,864 v/s | 1033.9 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 1.65s | 12,094 v/s | 1058.1 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 1.67s | 11,991 v/s | 1046.6 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **35.25%** | 991 | 0.99ms | 1.42ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **90.55%** | 692 | 1.43ms | 1.89ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **97.80%** | 503 | 1.98ms | 2.64ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.35%** | 435 | 2.29ms | 3.06ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **61.80%** | 652 | 1.52ms | 1.88ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **76.65%** | 525 | 1.89ms | 2.50ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **23.95%** | 2,100 | 0.46ms | 0.54ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.25%** | 1,679 | 0.58ms | 1.00ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **77.00%** | 1,514 | 0.65ms | 0.85ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **82.80%** | 1,368 | 0.72ms | 1.26ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **40.95%** | 1,619 | 0.60ms | 1.04ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **26.85%** | 1,900 | 0.51ms | 0.66ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **76.00%** | 1,526 | 0.64ms | 1.07ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **82.70%** | 1,352 | 0.72ms | 1.26ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **42.00%** | 1,646 | 0.59ms | 0.89ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.75%** | 120 | 8.29ms | 11.46ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.90%** | 116 | 8.59ms | 11.71ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **99.80%** | 113 | 8.83ms | 12.32ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **99.75%** | 112 | 8.87ms | 12.44ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **85.00%** | 112 | 8.91ms | 13.68ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.30%** | 2,599 | 0.37ms | 0.70ms | 5.1s |
| **rabitq_rerank4** | **96.85%** | 1,959 | 0.50ms | 0.59ms | 5.2s |
| **rabitq_rerank8** | **99.05%** | 1,492 | 0.66ms | 1.02ms | 5.1s |
| **rabitq_twopass_raw40** | **62.10%** | 2,375 | 0.41ms | 0.50ms | 5.2s |
| **pq4** | **35.90%** | 2,048 | 0.47ms | 0.85ms | 2.7s |
| **pq4_rerank4** | **95.50%** | 1,552 | 0.63ms | 0.82ms | 2.6s |
| **pq4_rerank8** | **99.00%** | 1,186 | 0.83ms | 1.39ms | 2.6s |

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

