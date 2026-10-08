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
*   **Migration Integrity & Tamper Protection (v4.0)**: The autonomous `MigrationEngine` computes audit manifests recorded in `.migration_v4_complete.json` with vector counts and commit timestamps prior to committing migrated chunks. Legacy stores are preserved in `legacy_v3_migrated/` to ensure full disaster recovery, idempotency, and auditability.
*   **API Authentication**: Built-in API Key support (SHA-256 hashed storage, Constant-time comparison) for all HTTP & gRPC traffic.
*   **Role-Based Access Control (RBAC)**: Fine-grained user permissions (`Admin`, `ReadWrite`, `ReadOnly`) managed via transaction-safe `redb` v4 security registry (`security.db`).
*   **Cross-Tenant Collection Sharing**: Securely share collections between tenants with read-only or read-write privileges using namespaced routing (`owner/collection`).
*   **Mutual TLS (mTLS)**: Zero-dependency out-of-the-box TLS/mTLS support for both gRPC (Tonic) and HTTP (Axum) endpoints using `HS_TLS_CERT`, `HS_TLS_KEY`, and `HS_TLS_CA` environment variables.
*   **Structured Audit Logs**: Structured JSON-formatted audit logging (`HS_AUDIT_LOG_LEVEL`) with tenant-isolated logs streamed securely to authenticated users.
*   **Multi-Tenancy**: Native namespace isolation between users ensuring data privacy in shared environments.
*   **Graph Perimeter**: Graph and Traversal endpoints enforce API Key and User-ID isolation. Recursive traversal is depth-limited to prevent DoS.
*   **Swarm Perimeter**: The UDP Edge-to-Edge Gossip protocol operates without built-in encryption. **It is designed for isolated Local Area Networks (LAN) or WireGuard/Tailscale VPCs.** Do not expose the Gossip port to the public internet.

## Compliance and SOC 2 Recommendation

For production deployments targeting **SOC 2 / ISO 27001** compliance, we recommend:
1. Running HyperspaceDB inside an isolated Virtual Private Cloud (VPC).
2. Enabling **mTLS** for all node-to-node (gRPC) and client-to-server communications.
3. Enabling **Audit Logging** with level `high` or `full` to capture security-relevant access logs.
4. Securing persistent storage (`data/` directory) on **LUKS-encrypted drives** or encrypted cloud volumes to guarantee encryption-at-rest.
