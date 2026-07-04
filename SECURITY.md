# Security Policy

## Supported Versions

Only the latest major version is currently supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 3.x     | :white_check_mark: |
| 2.x     | :warning:          |
| 1.x     | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability in HyperspaceDB, please do not disclose it publicly.

**Please report vulnerabilities directly via email to:**
`hi@yar.ink`

We will acknowledge your report within 48 hours and provide an estimated timeframe for a fix.

## Security Features

*   **Memory Safety**: Built 100% in Rust to prevent buffer overflows and use-after-free errors.
**Authentication**: Built-in API Key support (SHA-256 hashed storage, Constant-time comparison) for all HTTP & gRPC traffic.
*   **Swarm Perimeter (v3.0)**: The UDP Edge-to-Edge Gossip protocol (`HS_GOSSIP_PEERS`) operates without built-in encryption. **It is designed for isolated Local Area Networks (LAN) or WireGuard/Tailscale VPCs.** Do not expose the Gossip port (`7946` by default) to the public internet.
*   **Multi-Tenancy**: Native namespace isolation between users ensuring data privacy in shared environments.
*   **Graph Perimeter (v3.1)**: New Graph and Traversal endpoints (`/api/collections/{name}/graph/*`) enforce the same API Key and User-ID isolation as the search data plane. Recursive traversal is depth-limited to prevent DoS.
*   **Role Based Access**: Strict Leader (Read/Write) and Follower (Read-Only) separation.
*   **Dependency Audits**: We regularly audit our crate dependencies.
*   **API Authentication**: Built-in API Key support (SHA-256 hashed storage, Constant-time comparison) for all HTTP & gRPC traffic.
*   **Role-Based Access Control (RBAC)**: Fine-grained user permissions (`Admin`, `ReadWrite`, `ReadOnly`) managed via transaction-safe `redb` v4 security registry (`security.db`).
*   **Cross-Tenant Collection Sharing**: Securely share collections between tenants with read-only or read-write privileges using namespaced routing (`owner/collection`).
*   **Mutual TLS (mTLS)**: Zero-dependency out-of-the-box TLS/mTLS support for both gRPC (Tonic) and HTTP (Axum) endpoints using `HS_TLS_CERT`, `HS_TLS_KEY`, and `HS_TLS_CA` environment variables.
*   **Structured Audit Logs**: Structured JSON-formatted audit logging (`HS_AUDIT_LOG_LEVEL`) with tenant-isolated logs streamed securely to authenticated users.
*   **Swarm Perimeter**: The UDP Edge-to-Edge Gossip protocol operates without built-in encryption. **It is designed for isolated Local Area Networks (LAN) or WireGuard/Tailscale VPCs.** Do not expose the Gossip port to the public internet.

## Compliance and SOC 2 Recommendation

For production deployments targeting **SOC 2 / ISO 27001** compliance, we recommend:
1. Running HyperspaceDB inside an isolated Virtual Private Cloud (VPC).
2. Enabling **mTLS** for all node-to-node (gRPC) and client-to-server communications.
3. Enabling **Audit Logging** with level `high` or `full` to capture security-relevant access logs.
4. Securing persistent storage (`data/` directory) on **LUKS-encrypted drives** or encrypted cloud volumes to guarantee encryption-at-rest.
