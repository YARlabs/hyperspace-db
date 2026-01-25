# Security Policy

## Supported Versions

Only the latest major version is currently supported with security updates.

| Version | Supported          |
| ------- | ------------------ |
| 1.x     | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take security seriously. If you discover a vulnerability in HyperspaceDB, please do not disclose it publicly.

**Please report vulnerabilities directly via email to:**
`sglukhota@gmail.com`

We will acknowledge your report within 48 hours and provide an estimated timeframe for a fix.

## Security Features

*   **Memory Safety**: Built 100% in Rust to prevent buffer overflows and use-after-free errors.
*   **Authentication**: Built-in API Key support (SHA-256 hashed storage, Constant-time comparison).
*   **Role Based Access**: Strict Leader (Read/Write) and Follower (Read-Only) separation.
*   **Dependency Audits**: We regularly audit our crate dependencies.
