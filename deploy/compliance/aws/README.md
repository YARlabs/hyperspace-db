# AWS Compliance Guidelines for HyperspaceDB Deployments

This guide provides configurations and reference templates to ensure HyperspaceDB meets SOC 2 Type II and GDPR requirements when hosted on Amazon Web Services (AWS) EC2 instances (Self-Managed On-Premise model).

---

## 1. Data at Rest Encryption (AWS KMS & EBS)

By default, all EBS volumes containing HyperspaceDB payloads (`/var/lib/hyperspacedb`) must be encrypted using AWS KMS Customer Managed Keys (CMKs) to satisfy SOC 2 encryption-at-rest controls.

### 1.1. Enforcing Default EBS Encryption (Terraform HCL snippet)
Use the following configuration to enforce that all newly created EBS volumes are encrypted by default in your region:

```hcl
resource "aws_ebs_encryption_by_default" "enforce_encryption" {
  enabled = true
}

resource "aws_kms_key" "hyperspacedb_key" {
  description             = "KMS Key for HyperspaceDB EBS Volume Encryption"
  deletion_window_in_days = 30
  enable_key_rotation     = true # Mandatory for SOC 2 Type II
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Sid    = "Enable IAM User Permissions"
        Effect = "Allow"
        Principal = {
          AWS = "arn:aws:iam::${var.account_id}:root"
        }
        Action   = "kms:*"
        Resource = "*"
      }
    ]
  })
}
```

---

## 2. Immutable Backups (S3 Object Lock)

To guarantee backup integrity and protect against ransomware, databases backups should be uploaded to an S3 bucket configured with **Object Lock** in **Compliance Mode** (WORM).

### 2.1. Bucket Creation Template (Terraform)
```hcl
resource "aws_s3_bucket" "immutable_backups" {
  bucket = "hyperspacedb-immutable-backups"

  object_lock_enabled = true # Must be set during bucket creation
}

resource "aws_s3_bucket_object_lock_configuration" "lock_config" {
  bucket = aws_s3_bucket.immutable_backups.id

  rule {
    default_retention {
      mode = "COMPLIANCE" # Cannot be overridden or bypassed, even by root account
      days = 30
    }
  }
}
```

---

## 3. Threat Detection and Audit Logging (Amazon CloudWatch)

To monitor instance state and gather audit logs (equivalent to HIDS/auditd gathering), deploy the **Amazon CloudWatch Unified Agent** on the EC2 instances.

### 3.1. CloudWatch Agent Configuration (`amazon-cloudwatch-agent.json`)
Configure the agent to ingest Linux audit logs (`/var/log/audit/audit.log`) and Nginx access logs:

```json
{
  "agent": {
    "metrics_collection_interval": 60,
    "run_as_user": "cwagent"
  },
  "logs": {
    "logs_collected": {
      "files": {
        "collect_list": [
          {
            "file_path": "/var/log/audit/audit.log",
            "log_group_name": "HyperspaceDB-Auditd-Logs",
            "log_stream_name": "{hostname}",
            "retention_in_days": 365
          },
          {
            "file_path": "/var/log/nginx/hyperspacedb_grpc_access.log",
            "log_group_name": "HyperspaceDB-Nginx-Access",
            "log_stream_name": "{hostname}",
            "retention_in_days": 90
          },
          {
            "file_path": "/var/log/auth.log",
            "log_group_name": "System-Authentication-Logs",
            "log_stream_name": "{hostname}",
            "retention_in_days": 365
          }
        ]
      }
    }
  }
}
```
Deploy the configuration on your host at `/opt/aws/amazon-cloudwatch-agent/etc/amazon-cloudwatch-agent.json` and restart the agent.
