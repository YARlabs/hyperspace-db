# Azure Compliance Guidelines for HyperspaceDB Deployments

This guide provides configurations and reference templates to ensure HyperspaceDB meets SOC 2 Type II and GDPR requirements when hosted on Microsoft Azure Virtual Machines.

---

## 1. Data at Rest Encryption (Azure Disk Encryption & Customer-Managed Keys)

Azure VM OS and data disks are encrypted by default with Platform-Managed Keys (PMK). To satisfy enterprise-grade SOC 2 auditing, configure Virtual Machine disks to use **Disk Encryption Sets** with **Customer-Managed Keys (CMK)** managed in Azure Key Vault.

### 1.1. Disk Encryption Set Template (Terraform HCL snippet)
```hcl
resource "azurerm_key_vault" "vault" {
  name                = "hyperspace-kv"
  location            = "eastus"
  resource_group_name = var.rg_name
  tenant_id           = var.tenant_id
  sku_name            = "standard"

  purge_protection_enabled = true # Crucial for SOC 2 (prevents accidental key purges)
}

resource "azurerm_key_vault_key" "disk_key" {
  name         = "db-disk-key"
  key_vault_id = azurerm_key_vault.vault.id
  key_type     = "RSA"
  key_size     = 2048

  key_opts = [
    "decrypt", "encrypt", "unwrapKey", "wrapKey"
  ]
}

resource "azurerm_disk_encryption_set" "des" {
  name                = "hyperspace-des"
  location            = "eastus"
  resource_group_name = var.rg_name
  key_vault_key_id    = azurerm_key_vault_key.disk_key.id

  identity {
    type = "SystemAssigned"
  }
}
```

---

## 2. Immutable Backups (Azure Blob Storage Immutable Policy)

Backup archives containing HyperspaceDB payloads should be stored in Azure Storage Accounts using **immutable storage** (WORM - Write Once, Read Many).

### 2.1. Blob Retention Policy Template (Terraform)
```hcl
resource "azurerm_storage_account" "backups" {
  name                     = "hyperspacebackups"
  resource_group_name      = var.rg_name
  location                 = "eastus"
  account_tier             = "Standard"
  account_replication_type = "GRS" # Geo-Redundant for DR requirements
}

resource "azurerm_storage_container" "backup_container" {
  name                  = "immutable-backups"
  storage_account_name  = azurerm_storage_account.backups.name
  container_access_type = "private"
}

resource "azurerm_storage_container_immutability_policy" "worm_policy" {
  storage_container_id = azurerm_storage_container.backup_container.id
  
  allow_saved_asset_creation = true
  period_since_creation_in_days = 30 # Once locked, this cannot be deleted or shortened
}
```

---

## 3. Log Ingestion & Threat Detection (Azure Monitor & AMA)

For host auditing, install the **Azure Monitor Agent (AMA)** on the Linux VMs. Configure it to stream syslog and `auditd` output files into an **Azure Log Analytics Workspace**.

### 3.1. Data Collection Rules (DCR) Configuration (Reference)
Configure Azure Data Collection Rules to target these log paths:
* `/var/log/audit/audit.log` (Custom Text Log)
* `/var/log/nginx/hyperspacedb_grpc_access.log` (Custom Text Log)
* Syslog (`auth.log`, `daemon.log`)

Through Log Analytics, you can configure Sentinel alert rules for unauthorized access to `/var/lib/hyperspacedb`.
