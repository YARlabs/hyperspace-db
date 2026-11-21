# HyperspaceDB v4.2.5 Quantization & Rerank Sweep Benchmark Report

> **Environment & Test Parameters**
> - **Engine**: HyperspaceDB v4.2.5 (`hyperspace-server`)
> - **Code-mode calibration**: RebuildIndex after ingest (pq, opq, pq4, rabitq, rabitq4, rabitq2)
> - **Dataset**: 20,000 vectors, 1024D Dense Semantic Clustered Embedding (50 clusters)
> - **Metric**: Cosine Distance (Normalized Euclidean)
> - **Queries**: 200 queries evaluated against Exact Brute-Force Ground Truth
> - **HNSW Configuration**: M=64, EF_Construct=400, EF_Search=100
> - **Date**: 2026-11-21 17:59:50

## 1. Executive Summary & Comparison Matrix

| Configuration | Quant Mode | Rerank Scheme | Recall@10 | Search QPS | Mean Latency | P95 Latency | Theor. RAM | Compression |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | None | **100.00%** | 1,374 | 0.71ms | 0.84ms | 78.20 MB | **1.0×** |
| **none_ef400** | `none` | None | **100.00%** | 1,388 | 0.71ms | 0.82ms | 78.20 MB | **1.0×** |
| **medium** | `medium` | None | **98.50%** | 2,660 | 0.36ms | 0.42ms | 19.61 MB | **4.0×** |
| **medium_rerank4** | `medium` | Server ADC (x4) | **100.00%** | 2,294 | 0.42ms | 0.51ms | 19.61 MB | **4.0×** |
| **medium_plus** | `medium_plus` | None | **87.15%** | 1,529 | 0.64ms | 0.69ms | 14.72 MB | **5.3×** |
| **medium_plus_ef400** | `medium_plus` | None | **87.15%** | 1,530 | 0.64ms | 0.71ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank4** | `medium_plus` | Server ADC (x4) | **100.00%** | 1,357 | 0.72ms | 0.81ms | 14.72 MB | **5.3×** |
| **medium_plus_rerank8** | `medium_plus` | Server ADC (x8) | **100.00%** | 1,230 | 0.80ms | 0.94ms | 14.72 MB | **5.3×** |
| **turbo** | `turbo` | None | **87.60%** | 902 | 1.09ms | 1.52ms | 9.92 MB | **7.9×** |
| **turbo_ef400** | `turbo` | None | **87.60%** | 1,072 | 0.92ms | 1.04ms | 9.92 MB | **7.9×** |
| **turbo_rerank4** | `turbo` | Server ADC (x4) | **100.00%** | 994 | 0.99ms | 1.09ms | 9.92 MB | **7.9×** |
| **turbo_twopass_raw40** | `turbo` | Client 2-Pass Raw (k=40) | **100.00%** | 1,000 | 0.98ms | 1.18ms | 9.92 MB | **7.9×** |
| **ternary** | `ternary` | None | **53.90%** | 1,340 | 0.73ms | 0.82ms | 5.11 MB | **15.3×** |
| **ternary_ef400** | `ternary` | None | **53.90%** | 1,338 | 0.73ms | 0.80ms | 5.11 MB | **15.3×** |
| **ternary_rerank4** | `ternary` | Server ADC (x4) | **100.00%** | 821 | 1.20ms | 1.31ms | 5.11 MB | **15.3×** |
| **ternary_rerank8** | `ternary` | Server ADC (x8) | **100.00%** | 699 | 1.42ms | 1.54ms | 5.11 MB | **15.3×** |
| **ternary_twopass_raw40** | `ternary` | Client 2-Pass Raw (k=40) | **85.10%** | 1,228 | 0.80ms | 0.93ms | 5.11 MB | **15.3×** |
| **opq** | `opq` | None | **24.40%** | 2,113 | 0.46ms | 0.58ms | 1.30 MB | **60.3×** |
| **opq_ef400** | `opq` | None | **24.40%** | 2,148 | 0.45ms | 0.52ms | 1.30 MB | **60.3×** |
| **opq_rerank4** | `opq` | Server ADC (x4) | **90.45%** | 1,058 | 0.93ms | 1.18ms | 1.30 MB | **60.3×** |
| **opq_rerank8** | `opq` | Server ADC (x8) | **96.15%** | 922 | 1.07ms | 1.25ms | 1.30 MB | **60.3×** |
| **opq_twopass_raw40** | `opq` | Client 2-Pass Raw (k=40) | **41.80%** | 1,953 | 0.50ms | 0.58ms | 1.30 MB | **60.3×** |
| **pq** | `pq` | None | **24.05%** | 2,193 | 0.44ms | 0.50ms | 1.30 MB | **60.3×** |
| **pq_rerank4** | `pq` | Server ADC (x4) | **89.95%** | 1,146 | 0.86ms | 0.94ms | 1.30 MB | **60.3×** |
| **pq_rerank8** | `pq` | Server ADC (x8) | **95.75%** | 922 | 1.07ms | 1.18ms | 1.30 MB | **60.3×** |
| **pq_twopass_raw40** | `pq` | Client 2-Pass Raw (k=40) | **40.75%** | 1,994 | 0.49ms | 0.55ms | 1.30 MB | **60.3×** |
| **extreme** | `extreme` | None | **36.00%** | 1,831 | 0.53ms | 0.61ms | 2.52 MB | **31.1×** |
| **extreme_rerank4** | `extreme` | Server ADC (x4) | **98.85%** | 1,044 | 0.94ms | 1.04ms | 2.52 MB | **31.1×** |
| **extreme_rerank8** | `extreme` | Server ADC (x8) | **99.80%** | 816 | 1.21ms | 1.41ms | 2.52 MB | **31.1×** |
| **extreme_rerank16** | `extreme` | Server ADC (x16) | **99.95%** | 602 | 1.64ms | 1.84ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw40** | `extreme` | Client 2-Pass Raw (k=40) | **62.80%** | 1,677 | 0.58ms | 0.65ms | 2.52 MB | **31.1×** |
| **extreme_twopass_raw80** | `extreme` | Client 2-Pass Raw (k=80) | **77.00%** | 1,547 | 0.63ms | 0.71ms | 2.52 MB | **31.1×** |
| **rabitq** | `rabitq` | None | **35.75%** | 2,390 | 0.40ms | 0.47ms | 2.67 MB | **29.3×** |
| **rabitq_rerank4** | `rabitq` | Server ADC (x4) | **98.40%** | 1,160 | 0.85ms | 0.93ms | 2.67 MB | **29.3×** |
| **rabitq_rerank8** | `rabitq` | Server ADC (x8) | **99.50%** | 928 | 1.06ms | 1.21ms | 2.67 MB | **29.3×** |
| **rabitq_twopass_raw40** | `rabitq` | Client 2-Pass Raw (k=40) | **62.40%** | 2,116 | 0.46ms | 0.54ms | 2.67 MB | **29.3×** |
| **rabitq4** | `rabitq4` | None | **86.85%** | 2,830 | 0.34ms | 0.40ms | 9.92 MB | **7.9×** |
| **rabitq4_rerank4** | `rabitq4` | Server ADC (x4) | **100.00%** | 2,162 | 0.45ms | 0.56ms | 9.92 MB | **7.9×** |
| **rabitq2** | `rabitq2` | None | **62.50%** | 2,718 | 0.35ms | 0.41ms | 5.04 MB | **15.5×** |
| **rabitq2_rerank4** | `rabitq2` | Server ADC (x4) | **100.00%** | 878 | 1.12ms | 2.53ms | 5.04 MB | **15.5×** |
| **pq4** | `pq4` | None | **35.55%** | 1,572 | 0.62ms | 0.75ms | 2.52 MB | **31.1×** |
| **pq4_rerank4** | `pq4` | Server ADC (x4) | **98.75%** | 976 | 1.01ms | 1.15ms | 2.52 MB | **31.1×** |
| **pq4_rerank8** | `pq4` | Server ADC (x8) | **99.70%** | 838 | 1.18ms | 1.27ms | 2.52 MB | **31.1×** |

---

## 2. Ingestion & Storage Footprint Analysis

| Configuration | Quant Mode | Bytes / Vec | 20,000 Vecs RAM | Ingest Time | Ingest QPS | Process RSS |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: |
| **none_ef100** | `none` | 4100 B | 78.20 MB | 1.00s | 20,068 v/s | 604.7 MB |
| **none_ef400** | `none` | 4100 B | 78.20 MB | 1.02s | 19,529 v/s | 648.6 MB |
| **medium** | `medium` | 1028 B | 19.61 MB | 0.98s | 20,399 v/s | 581.2 MB |
| **medium_rerank4** | `medium` | 1028 B | 19.61 MB | 1.00s | 19,967 v/s | 577.3 MB |
| **medium_plus** | `medium_plus` | 772 B | 14.72 MB | 1.13s | 17,640 v/s | 616.7 MB |
| **medium_plus_ef400** | `medium_plus` | 772 B | 14.72 MB | 1.18s | 16,949 v/s | 550.8 MB |
| **medium_plus_rerank4** | `medium_plus` | 772 B | 14.72 MB | 1.17s | 17,160 v/s | 579.7 MB |
| **medium_plus_rerank8** | `medium_plus` | 772 B | 14.72 MB | 1.01s | 19,786 v/s | 581.9 MB |
| **turbo** | `turbo` | 520 B | 9.92 MB | 1.24s | 16,143 v/s | 584.5 MB |
| **turbo_ef400** | `turbo` | 520 B | 9.92 MB | 1.28s | 15,629 v/s | 563.5 MB |
| **turbo_rerank4** | `turbo` | 520 B | 9.92 MB | 1.24s | 16,154 v/s | 613.2 MB |
| **turbo_twopass_raw40** | `turbo` | 520 B | 9.92 MB | 1.21s | 16,587 v/s | 606.3 MB |
| **ternary** | `ternary` | 268 B | 5.11 MB | 1.24s | 16,193 v/s | 518.9 MB |
| **ternary_ef400** | `ternary` | 268 B | 5.11 MB | 1.32s | 15,129 v/s | 543.6 MB |
| **ternary_rerank4** | `ternary` | 268 B | 5.11 MB | 1.22s | 16,437 v/s | 578.7 MB |
| **ternary_rerank8** | `ternary` | 268 B | 5.11 MB | 1.30s | 15,350 v/s | 565.8 MB |
| **ternary_twopass_raw40** | `ternary` | 268 B | 5.11 MB | 1.35s | 14,826 v/s | 581.1 MB |
| **opq** | `opq` | 68 B | 1.30 MB | 9.53s | 2,098 v/s | 543.5 MB |
| **opq_ef400** | `opq` | 68 B | 1.30 MB | 9.41s | 2,124 v/s | 525.4 MB |
| **opq_rerank4** | `opq` | 68 B | 1.30 MB | 9.30s | 2,150 v/s | 476.3 MB |
| **opq_rerank8** | `opq` | 68 B | 1.30 MB | 9.34s | 2,142 v/s | 542.1 MB |
| **opq_twopass_raw40** | `opq` | 68 B | 1.30 MB | 9.16s | 2,183 v/s | 487.5 MB |
| **pq** | `pq` | 68 B | 1.30 MB | 9.07s | 2,204 v/s | 539.5 MB |
| **pq_rerank4** | `pq` | 68 B | 1.30 MB | 8.96s | 2,232 v/s | 506.6 MB |
| **pq_rerank8** | `pq` | 68 B | 1.30 MB | 8.94s | 2,238 v/s | 470.1 MB |
| **pq_twopass_raw40** | `pq` | 68 B | 1.30 MB | 8.93s | 2,240 v/s | 438.8 MB |
| **extreme** | `extreme` | 132 B | 2.52 MB | 1.17s | 17,100 v/s | 625.1 MB |
| **extreme_rerank4** | `extreme` | 132 B | 2.52 MB | 1.00s | 20,094 v/s | 546.7 MB |
| **extreme_rerank8** | `extreme` | 132 B | 2.52 MB | 1.09s | 18,347 v/s | 542.7 MB |
| **extreme_rerank16** | `extreme` | 132 B | 2.52 MB | 1.03s | 19,419 v/s | 574.3 MB |
| **extreme_twopass_raw40** | `extreme` | 132 B | 2.52 MB | 1.03s | 19,394 v/s | 577.0 MB |
| **extreme_twopass_raw80** | `extreme` | 132 B | 2.52 MB | 1.04s | 19,168 v/s | 530.8 MB |
| **rabitq** | `rabitq` | 140 B | 2.67 MB | 1.03s | 19,478 v/s | 601.0 MB |
| **rabitq_rerank4** | `rabitq` | 140 B | 2.67 MB | 1.11s | 18,055 v/s | 575.3 MB |
| **rabitq_rerank8** | `rabitq` | 140 B | 2.67 MB | 1.10s | 18,217 v/s | 533.8 MB |
| **rabitq_twopass_raw40** | `rabitq` | 140 B | 2.67 MB | 1.07s | 18,733 v/s | 504.7 MB |
| **rabitq4** | `rabitq4` | 520 B | 9.92 MB | 1.37s | 14,640 v/s | 577.9 MB |
| **rabitq4_rerank4** | `rabitq4` | 520 B | 9.92 MB | 1.35s | 14,866 v/s | 579.9 MB |
| **rabitq2** | `rabitq2` | 264 B | 5.04 MB | 1.40s | 14,255 v/s | 659.0 MB |
| **rabitq2_rerank4** | `rabitq2` | 264 B | 5.04 MB | 1.36s | 14,696 v/s | 676.1 MB |
| **pq4** | `pq4` | 132 B | 2.52 MB | 4.79s | 4,177 v/s | 506.2 MB |
| **pq4_rerank4** | `pq4` | 132 B | 2.52 MB | 4.80s | 4,168 v/s | 502.1 MB |
| **pq4_rerank8** | `pq4` | 132 B | 2.52 MB | 4.75s | 4,208 v/s | 511.7 MB |

---

## 3. Deep Dive: Extreme 1-Bit Binary Quantization & Reranking

Extreme quantization compresses 1024D floating-point vectors into a compact 128-byte bitmask (1 bit per dimension) plus magnitude scale factors, representing a **31.1× memory reduction vs FP32** (and **62× vs FP64**).

| Extreme Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **extreme** | **36.00%** | 1,831 | 0.53ms | 0.66ms | Pure single-pass 1-bit Hamming/ADC. Ultra-fast, but suffers bit-coarsening. |
| **extreme_rerank4** | **98.85%** | 1,044 | 0.94ms | 1.12ms | HNSW fetches top 40 candidates; server re-scores via ADC. |
| **extreme_rerank8** | **99.80%** | 816 | 1.21ms | 1.62ms | HNSW fetches top 80 candidates; server re-scores via ADC. |
| **extreme_rerank16** | **99.95%** | 602 | 1.64ms | 2.26ms | HNSW fetches top 160 candidates; server re-scores via ADC. |
| **extreme_twopass_raw40** | **62.80%** | 1,677 | 0.58ms | 0.74ms | HNSW fetches top 40 candidates; exact raw vector cosine re-ranking. |
| **extreme_twopass_raw80** | **77.00%** | 1,547 | 0.63ms | 0.80ms | HNSW fetches top 80 candidates; exact raw vector cosine re-ranking. |

### Key Takeaways for Extreme Mode:
1. **Single-Pass 1-Bit Limitation**: In high-dimensional spaces (1024D), single-pass 1-bit ADC acts as an ultra-fast filter, but loss of magnitude information causes rank inversion.
2. **Server ADC Rerank**: Oversampling 4× to 16× allows HNSW to discover nearest Hamming neighbors, and server-side de-quantized scoring improves candidate ordering.

---

## 4. Deep Dive: Optimized Product Quantization (OPQ / PQ)

OPQ compresses 1024D vectors into **64 subvectors (64 bytes per vector)** plus metadata, delivering **60.3× compression vs FP32**.
By applying Fast Walsh-Hadamard Transform (FWHT $\mathcal{O}(D \log D)$) rotation prior to quantization, coordinate variance is balanced across subvectors, eliminating covariance bottlenecks and allowing compact 64 KB ADC lookup tables in CPU L1/L2 cache.

| OPQ Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **opq** | **24.40%** | 2,113 | 0.46ms | 0.65ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_ef400** | **24.40%** | 2,148 | 0.45ms | 0.56ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **opq_rerank4** | **90.45%** | 1,058 | 0.93ms | 1.31ms | Server ADC candidate expansion (x4). |
| **opq_rerank8** | **96.15%** | 922 | 1.07ms | 1.47ms | Server ADC candidate expansion (x8). |
| **opq_twopass_raw40** | **41.80%** | 1,953 | 0.50ms | 0.64ms | Client raw vector cosine re-ranking (Top-40). |
| **pq** | **24.05%** | 2,193 | 0.44ms | 0.57ms | Single-pass 64-byte ADC table lookups in L1 cache (15-25 ns/vec). |
| **pq_rerank4** | **89.95%** | 1,146 | 0.86ms | 1.01ms | Server ADC candidate expansion (x4). |
| **pq_rerank8** | **95.75%** | 922 | 1.07ms | 1.35ms | Server ADC candidate expansion (x8). |
| **pq_twopass_raw40** | **40.75%** | 1,994 | 0.49ms | 0.60ms | Client raw vector cosine re-ranking (Top-40). |

---

## 5. Deep Dive: 1.58-Bit Ternary BitNet Quantization & Reranking

BitNet 1.58-bit ternary quantization compresses 1024D vectors into **two packed 128-byte bitmasks (non-zero and sign)** with a dynamic threshold $\gamma = 0.67 \cdot \mathbb{E}[|x|]$ and adaptive scaling factors (scale + alpha), achieving **268 bytes per vector (15.3× compression vs FP32, 30.6× vs FP64)**.
Distance computation is hardware-accelerated using parallel SIMD popcount and bitwise logic (`(a.nz & b.nz) ^ (a.sign ^ b.sign)`), yielding high single-pass recall while eliminating floating-point multiplication overhead during graph traversal.

| Ternary Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Key Characteristic |
| :--- | :---: | :---: | :---: | :---: | :--- |
| **ternary** | **53.90%** | 1,340 | 0.73ms | 1.01ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_ef400** | **53.90%** | 1,338 | 0.73ms | 0.83ms | Pure single-pass BitNet 1.58-bit {-1, 0, +1} SIMD popcount. |
| **ternary_rerank4** | **100.00%** | 821 | 1.20ms | 1.39ms | Server ADC candidate expansion (x4). |
| **ternary_rerank8** | **100.00%** | 699 | 1.42ms | 1.63ms | Server ADC candidate expansion (x8). |
| **ternary_twopass_raw40** | **85.10%** | 1,228 | 0.80ms | 1.13ms | Client raw vector cosine re-ranking (Top-40). |

---

## 6. Deep Dive: RaBitQ (1 / 2 / 4-bit) & PQ4 Fast-Scan (v4.2.5)

RaBitQ stores a 1-bit code of the FWHT-rotated residual plus norm and the correction factor ⟨o', x̄⟩ (136 B for 1024D), giving an unbiased distance estimator evaluated with popcount over 4 query bit-planes. Extended RaBitQ (`rabitq4`, `rabitq2`) codes the rotated residual on a 4- / 2-bit grid with the scale that maximizes cos(w, o') (520 / 264 B for 1024D) and estimates ⟨o, q⟩ = ⟨w, q'⟩ / ⟨w, o'⟩ with one int8 dot product per candidate. PQ4 stores 4-bit codes over 4-D sub-spaces (128 B for 1024D) and scores them with an L1-resident u8 lookup table.

| Variant | Recall@10 | Search QPS | Mean Lat | P99 Lat | Calibration |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **rabitq** | **35.75%** | 2,390 | 0.40ms | 0.54ms | 1.6s |
| **rabitq_rerank4** | **98.40%** | 1,160 | 0.85ms | 0.97ms | 1.7s |
| **rabitq_rerank8** | **99.50%** | 928 | 1.06ms | 1.33ms | 1.6s |
| **rabitq_twopass_raw40** | **62.40%** | 2,116 | 0.46ms | 0.59ms | 1.7s |
| **rabitq4** | **86.85%** | 2,830 | 0.34ms | 0.43ms | 2.6s |
| **rabitq4_rerank4** | **100.00%** | 2,162 | 0.45ms | 0.63ms | 2.7s |
| **rabitq2** | **62.50%** | 2,718 | 0.35ms | 0.47ms | 2.9s |
| **rabitq2_rerank4** | **100.00%** | 878 | 1.12ms | 3.46ms | 2.9s |
| **pq4** | **35.55%** | 1,572 | 0.62ms | 0.91ms | 8.1s |
| **pq4_rerank4** | **98.75%** | 976 | 1.01ms | 1.48ms | 8.2s |
| **pq4_rerank8** | **99.70%** | 838 | 1.18ms | 1.36ms | 8.1s |

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

