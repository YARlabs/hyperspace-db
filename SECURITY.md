# Security Policy

## Supported Versions

Only the latest major version is currently actively supported with feature and security updates. Migration support is maintained for the preceding major version.

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 4.x     | :white_check_mark: | Active (Current) |
| 3.x     | :warning:          | Supported Migration Path |
| 2.x     | :x:                | End of Life |
| 1.x     | :x:                | End of Life |

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability in HyperspaceDB, please do not disclose it publicly.

**Please report vulnerabilities directly via email to:**
`hi@yar.ink`

We will acknowledge your report within 48 hours and provide an estimated timeframe for a fix.

## Security Features

*   **Memory Safety**: Built 100% in Rust to prevent buffer overflows, use-after-free, and dangling pointer vulnerabilities.
*   **Thread-per-Core Memory Isolation (v4.0)**: Each shard maintains its own partition in a Shared-Nothing architecture (`ShardedCollection`), eliminating cross-core shared memory mutation and preventing concurrency race conditions or lock-poisoning panics.
*   **Direct I/O Hardware Isolation (v4.0)**: Enforces strict 4096-byte hardware page-aligned DMA buffers (`AlignedBuffer`). Bypassing OS page caches eliminates kernel buffer memory exhaustion and page cache side-channel snooping during heavy NVMe disk scans.
*   **Runtime SIMD Memory Safety (v4.1)**: Unaligned SIMD float loads (`_mm512_loadu_ps`, `_mm256_loadu_ps`, `vld1q_f32`) are bounded by strict slice length checks (`n * 4`), preventing memory overreads on truncated or corrupted vector buffers.
*   **io_uring Lifetime & Bounds Safety (v4.1)**: Batched kernel reads validate buffer slice bounds and keep user buffers pinned until all completion queue entries (CQEs) are reaped, preventing use-after-free vulnerabilities.
*   **NUMA Cgroup Boundary Enforcement (v4.1)**: Socket affinity discovery intersects `/sys/devices/system/node` with `sched_getaffinity`, ensuring threads stay strictly within Kubernetes container CPU limits without leaking onto unallocated host cores.
*   **Migration Integrity & Tamper Protection (v4.0)**: The autonomous `MigrationEngine` computes audit manifests recorded in `.migration_v4_complete.json` with vector counts and commit timestamps prior to committing migrated chunks. Legacy stores are preserved in `legacy_v3_migrated/` to ensure full disaster recovery, idempotency, and auditability.
*   **API Authentication**: Built-in API Key support (SHA-256 hashed storage, Constant-time comparison) for all HTTP & gRPC traffic.
*   **Role-Based Access Control (RBAC)**: Fine-grained user permissions (`Admin`, `ReadWrite`, `ReadOnly`) managed via transaction-safe `redb` v4 security registry (`security.db`).
*   **Cross-Tenant Collection Sharing**: Securely share collections between tenants with read-only or read-write privileges using namespaced routing (`owner/collection`).
*   **Mutual TLS (mTLS)**: Zero-dependency out-of-the-box TLS/mTLS support for both gRPC (Tonic) and HTTP (Axum) endpoints using `HS_TLS_CERT`, `HS_TLS_KEY`, and `HS_TLS_CA` environment variables.
*   **Zero-Knowledge Client-Side Vector Encryption & Inversion Defense (`Vec2Text` Immunity) (v4.1.2)**: 
    *   **Isometric Vector Encryption**: Vectors are projected on the client before transmission using deterministic seed-based orthogonal matrices $O(n)$ for Euclidean/Cosine/Poincaré spaces, and proper orthochronous Lorentz transformations $SO^+(1, d)$ for Minkowski hyperboloids. Top-K search recall is 100.0% preserved with distance distortion at floating point precision limit ($\sim 10^{-16}$).
    *   **Vec2Text & Inversion Attack Resistance**: Calibrated anisotropic perturbation ($\sigma \le 0.02$) prevents Manifold Alignment and Neural Inversion attacks. Ciphertext vectors are statistically indistinguishable from uniform points on the unit sphere (observed cosine similarity $0.052$ vs theoretical random sphere expectation $0.088$).
    *   **Payload Encryption (AES-256-GCM)**: All document bodies, raw chunk texts, and sensitive metadata attributes are encrypted client-side using authenticated AES-256-GCM with PBKDF2 key derivation (100,000 rounds). The server never sees plaintext text or unencrypted embeddings.
    *   **Per-Collection Encryption Granularity**: Encryption is strictly optional and configurable per collection. The database engine enforces an explicit `encrypted` status badge surfaced across Protobuf, REST APIs, and the Admin Dashboard.
*   **Compliance and SOC 2 / HIPAA / GDPR Recommendation**:
    *   For healthcare (HIPAA) and financial (GDPR) workloads, tenant-controlled client encryption ensures Zero-Knowledge storage: cloud providers and database administrators cannot read user prompts or invert vector representations even if server disks or RAM snapshots are compromised.
*   **Plugin Sandbox & Dynamic Library Isolation (`hyperspace-pdk` v4.2.0)**:
    *   **ABI Version Handshake**: Every external plugin binary (`.so`, `.dylib`, `.dll`) compiled with `hs_export_plugin!` exports an immutable `hs_plugin_api_version()` symbol. The host loader validates that the plugin was built against the exact ABI version (`40200`), rejecting mismatched or corrupted libraries prior to execution.
    *   **Server-Side Zero-Knowledge Orthogonal Shield (`hs-shield`)**: Provides transparent in-index protection against `Vec2Text` neural reconstruction attacks. Vectors undergo Fast Walsh-Hadamard transformation with keyed pseudo-random sign permutations directly on the server before graph linkage. Top-K search accuracy is preserved with 100.0% mathematical isometry.
    *   **Ternary BitNet Quantization Defense (v4.2.3)**: 1.58-bit ternary projection {-1, 0, +1} provides inherent geometric discretization against gradient-based feature inversion attacks, preventing raw feature value leakage from stored vector index slices.
    *   **Safe Dynamic Dispatch**: Procedure execution and hook invocations (`before_insert`, `after_search`) are mediated by safe Rust trait boundaries, catching internal procedure errors gracefully without crashing worker threads or poisoning shared collections.
*   **Numerical Stability & Denial-of-Service Defense (v4.2.4)**:
    *   **Catastrophic Cancellation Immunity**: Replaced naive Minkowski quadratic subtraction $t_u t_v - \mathbf{x}_u \cdot \mathbf{x}_v$ with difference-scaled identities $\cosh(d) = 1 + \frac{1}{2}(\|\Delta \mathbf{x}\|^2 - \Delta t^2)$, preventing floating-point underflow/cancellation at large radii $t \gg 1$ and guarding against algorithmic complexity attacks that exploit degenerate distance graphs.
    *   **Bounds-Checked SIMD & PQ Lookups**: All high-throughput ADC lookups, hybrid SIMD routines, and zero-copy byte-to-byte SIMD distance kernels validate slice bounds (`(bytes_a.len() / 4).min(bytes_b.len() / 4)`) before vectorized execution, preventing memory overreads or buffer overflow exploits on malformed vectors.
    *   **Memory Safety in 8-Byte Transmuted Heaps (v4.2.5)**: `Candidate` and `MaxCandidate` enforce strict `#[repr(C, align(8))]` 8-byte layout and little-endian transmutation bounds via `as_u64()`, ensuring type-safe, undefined-behavior-free 1-cycle integer comparisons in `BinaryHeap`.
    *   **Integer Overflow Resistance in SQ8 Accumulation (v4.2.5)**: The fused `sq8_dot_and_qsq` kernel accumulates $\sum q_i^2$ in 32-bit unsigned integers (`vpadalq_s16` / `u32`) where $D \le 4096$ guarantees maximum sum $4096 \times 127^2 = 66,064,384 \ll 2^{32} - 1$, eliminating intermediate arithmetic overflow.
    *   **Bounded KAG Graph Traversal (v4.2.5)**: Beam-guided KAG edge expansion inside `search_layer0` is strictly bounded by active edge flags and maximum neighbor fanout limits (`HS_GRAPH_MAX_FANOUT_PER_NODE`), preventing cyclic graph recursion or unbounded memory expansion during vector search.
*   **Structured Audit Logs**: Structured JSON-formatted audit logging (`HS_AUDIT_LOG_LEVEL`) with tenant-isolated logs streamed securely to authenticated users.
*   **Multi-Tenancy**: Native namespace isolation between users ensuring data privacy in shared environments.
*   **Graph Perimeter**: Graph and Traversal endpoints enforce API Key and User-ID isolation. Recursive traversal is depth-limited to prevent DoS.
*   **Swarm Perimeter**: The UDP Edge-to-Edge Gossip protocol operates without built-in encryption. **It is designed for isolated Local Area Networks (LAN) or WireGuard/Tailscale VPCs.** Do not expose the Gossip port to the public internet.

## Compliance and SOC 2 / HIPAA / GDPR Architecture

For production deployments targeting **SOC 2 Type II, HIPAA, and GDPR** compliance:
1. Enable **Client-Side Zero-Knowledge Encryption** (`encryption_key` parameter) on sensitive collections.
2. Run HyperspaceDB inside an isolated Virtual Private Cloud (VPC) or Kubernetes cluster.
3. Enabling **mTLS** for all node-to-node (gRPC) and client-to-server communications (`HS_TLS_CERT`, `HS_TLS_KEY`).
4. Enabling **Audit Logging** with level `high` or `full` to capture security-relevant access logs.
5. Securing persistent storage (`data/` directory) on **LUKS-encrypted drives** or encrypted cloud volumes to guarantee encryption-at-rest.
