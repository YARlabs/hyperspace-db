# HyperspaceDB Compliance & Security Guide

This compliance and security guide outlines the shared responsibility model, application-level security features, and infrastructure hardening templates required to run **HyperspaceDB** in enterprise environments (complying with SOC 2 Type II, ISO 27001, and GDPR).

---

## 1. Shared Responsibility Model

When deploying HyperspaceDB, security responsibilities are split based on the deployment model:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Shared Responsibility Model                         │
├──────────────────────────────────────┬──────────────────────────────────┤
│           SaaS Deployments           │     On-Premise / Self-Hosted     │
├──────────────────────────────────────┼──────────────────────────────────┤
│ 🔵 User & Access Control (RBAC)      │ 🔵 User & Access Control (RBAC)  │
│ 🔵 Application Audit Logs Logging    │ 🔵 Application Audit Logs Logging│
│ ──────────────────────────────────── │ ──────────────────────────────── │
│ 🔶 OS-Level Hardening (sysctl, UFW)  │ 🔴 OS-Level Hardening (Customer) │
│ 🔶 Disk Encryption (LUKS + TPM 2.0)  │ 🔴 Disk Encryption (Customer)    │
│ 🔶 Host Intrusion Detection (HIDS)  │ 🔴 Host Intrusion Detection (Cust)│
│ 🔶 Immutable Offsite Backups (S3)    │ 🔴 Immutable Offsite Backups (Cust)│
└──────────────────────────────────────┴──────────────────────────────────┘
🔵 Database Level (HyperspaceDB Core) | 🔶 SaaS Host (We manage) | 🔴 Customer Managed
```

### 1.1. On-Premise Deployments
For On-Premise/Self-Hosted installations, the customer is fully responsible for securing the host operating system, firewall, physical/virtual disk encryption, and backup destinations. HyperspaceDB provides the necessary application-level knobs (RBAC, Audit Logging, TLS) to integrate seamlessly with the customer’s security stack.

### 1.2. SaaS Deployments
For our SaaS offering, HyperspaceDB manages both the database engine security and the host infrastructure (Bare-metal server hardening, LUKS encryption, Wazuh HIDS monitoring, and immutable S3 backups).

---

## 2. Database Core Security Features (Audit-Ready Core)

HyperspaceDB includes built-in security features designed to satisfy corporate security audits out of the box:

### 2.1. Application-Level Audit Logs
HyperspaceDB writes a structured, JSON-formatted audit log to `stdout` or a dedicated log file (`audit.log`). Any critical action (e.g., dropping a collection, updating indexes, altering permissions) generates an immutable log entry.

*Example Audit Log Entry:*
```json
{
  "timestamp": "2026-07-01T10:45:00Z",
  "actor": "admin_user",
  "action": "DROP_COLLECTION",
  "collection": "spatial_ai_nodes",
  "status": "success",
  "client_ip": "192.168.1.45"
}
```

### 2.2. Role-Based Access Control (RBAC)
Database connections are authenticated via API keys or client certificates. Permissions are strictly enforced:
* **Read-Only:** Allows candidate search and vector distance queries.
* **Read-Write:** Allows inserting vectors, creating graphs, and modifying metadata.
* **Admin:** Full administrative access including WAL compaction, backups, and user management.

### 2.3. Native mTLS Support
The HyperspaceDB binary natively supports certificate-based authentication. In addition to running Nginx in front of the database, the engine can be configured with native TLS using environment variables:
```bash
export HS_TLS_CERT=/path/to/server.crt
export HS_TLS_KEY=/path/to/server.key
export HS_TLS_CA=/path/to/ca.crt
./hyperspace-server
```

### 2.4. GDPR Data Segregation (Tenant Isolation)
To comply with GDPR "Right to be Forgotten" and multi-tenant data isolation, HyperspaceDB HNSW graphs support pre-filtering by `tenant_id` at the index traversal layer.

---

## 3. Infrastructure Compliance Templates

To assist you with infrastructure setup, this directory contains production-ready templates for various environments:

* **[bare-metal/](/hyperspace-db/deploy/compliance/bare-metal/)**  
  Hardening configurations for physical servers, including automated LUKS2 disk encryption with **TPM 2.0 / Clevis** integration and system-level `auditd` FIM policies.
* **[mTLS/](/hyperspace-db/deploy/compliance/mTLS/)**  
  Certificates generation scripts for private Root CA setup and template proxies for Nginx and Envoy.
* **[k8s/](/hyperspace-db/deploy/compliance/k8s/)**  
  Hardened Kubernetes manifests, containing secure StatefulSet templates (running as non-root UID 1000, read-only root FS, owner-only data access) and NetworkPolicies for namespace isolation.
* **[aws/](/hyperspace-db/deploy/compliance/aws/)**  
  AWS guidelines for KMS disk encryption, Amazon CloudWatch Logs collection, and S3 Immutable Object Lock policy.
* **[gcp/](/hyperspace-db/deploy/compliance/gcp/)**  
  Google Cloud templates for Customer-Managed Encryption Keys (CMEK), GCS Bucket Lock, and Cloud Operations Agent (Ops Agent).
* **[azure/](/hyperspace-db/deploy/compliance/azure/)**  
  Microsoft Azure VM templates for Key Vault, Disk Encryption Sets (CMK), Azure Storage Immutability policies, and Log Analytics.
