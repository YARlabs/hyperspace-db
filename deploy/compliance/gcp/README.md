# GCP Compliance Guidelines for HyperspaceDB Deployments

This guide provides security configurations and reference templates to ensure HyperspaceDB meets SOC 2 Type II and GDPR requirements when hosted on Google Cloud Platform (GCP) Compute Engine virtual machines.

---

## 1. Data at Rest Encryption (GCP KMS & CMEK)

By default, Google Cloud encrypts all persistent disks at rest using Google-Managed Keys. To achieve strict SOC 2 compliance, deploy persistent disks encrypted with **Customer-Managed Encryption Keys (CMEK)**.

### 1.1. Creating a CMEK Disk (Terraform HCL snippet)
```hcl
resource "google_kms_key_ring" "hyperspacedb_keyring" {
  name     = "hyperspacedb-keyring"
  location = "us-central1"
}

resource "google_kms_crypto_key" "hyperspacedb_disk_key" {
  name            = "disk-encryption-key"
  key_ring        = google_kms_key_ring.hyperspacedb_keyring.id
  rotation_period = "7776000s" # 90 days automatic key rotation (SOC 2 control)
}

resource "google_compute_disk" "hyperspacedb_data" {
  name  = "hyperspacedb-data-disk"
  type  = "pd-ssd"
  zone  = "us-central1-a"
  size  = 100

  disk_encryption_key {
    kms_key_self_link = google_kms_crypto_key.hyperspacedb_disk_key.id
  }
}
```

---

## 2. Immutable Backups (GCP Cloud Storage Bucket Lock)

For business continuity and ransomware protection, backup files must be uploaded to a GCS bucket with a **Retention Policy** (Bucket Lock) enabled in locked mode.

### 2.1. Locked Bucket Configuration (Terraform)
```hcl
resource "google_storage_bucket" "immutable_backups" {
  name     = "hyperspacedb-immutable-backups"
  location = "US"

  # Prevent bucket deletion if it contains locked objects
  force_destroy = false 

  retention_policy {
    is_locked        = true # Once locked, the policy CANNOT be removed or reduced.
    retention_period = 2592000 # 30 days retention (in seconds)
  }
}
```

---

## 3. Logs Collection & Intrusion Monitoring (GCP Ops Agent)

Deploy the unified **Google Cloud Operations Suite Agent** (Ops Agent) to stream OS syslog, authentication logs, and filesystem audit logs (`auditd`) to GCP Cloud Logging.

### 3.1. Ops Agent Configuration (`/etc/google-cloud-ops-agent/config.yaml`)
```yaml
logging:
  receivers:
    auditd_logs:
      type: files
      include_paths:
        - /var/log/audit/audit.log
    nginx_logs:
      type: files
      include_paths:
        - /var/log/nginx/hyperspacedb_grpc_access.log
        - /var/log/nginx/hyperspacedb_http_access.log
    syslog:
      type: syslog
  processors:
    parse_json:
      type: parse_json
  service:
    pipelines:
      audit_pipeline:
        receivers: [auditd_logs, syslog]
      nginx_pipeline:
        receivers: [nginx_logs]
```
Deploy the YAML configuration and reload the service:
```bash
sudo systemctl restart google-cloud-ops-agent
```
All streamed logs can be queried in GCP Logging Console using the Log Explorer.
