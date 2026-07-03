use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::path::Path;
use tracing::{error, info};

/// `redb` table: api_key (String) → json-encoded BillingBalance
const BALANCES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("balances");

/// Serializable balance record per tenant.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BillingBalance {
    pub api_key: String,
    /// Whether the remote service marked this tenant as active
    pub status: BillingStatus,
    /// Cached token balance (updated from api_identity_rust after each sync)
    pub last_known_balance: f64,
    /// Unix timestamp of the last successful sync
    pub last_sync_ts: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum BillingStatus {
    Active,
    InsufficientFunds,
    Unknown,
}

/// Thread-safe, ACID-transactional persistent billing state backed by `redb`.
pub struct AccountingStore {
    db: Database,
}

impl AccountingStore {
    /// Open (or create) the `redb` billing database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let db = Database::create(path)?;
        // Ensure the balances table exists
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(BALANCES_TABLE)?;
        }
        write_txn.commit()?;
        info!("AccountingStore opened");
        Ok(Self { db })
    }

    /// Load all known billing balances from disk (called at node startup).
    pub fn load_all(&self) -> anyhow::Result<Vec<BillingBalance>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(BALANCES_TABLE)?;
        let mut balances = Vec::new();
        for result in table.iter()? {
            let (_, value) = result?;
            match serde_json::from_slice::<BillingBalance>(value.value()) {
                Ok(b) => balances.push(b),
                Err(e) => error!("Failed to deserialize BillingBalance: {}", e),
            }
        }
        Ok(balances)
    }

    /// Persist a single balance record (upsert).
    pub fn upsert(&self, balance: &BillingBalance) -> anyhow::Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(BALANCES_TABLE)?;
            let encoded = serde_json::to_vec(balance)?;
            table.insert(balance.api_key.as_str(), encoded.as_slice())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Batch upsert — all writes happen inside a single ACID transaction.
    pub fn upsert_batch(&self, balances: &[BillingBalance]) -> anyhow::Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(BALANCES_TABLE)?;
            for b in balances {
                let encoded = serde_json::to_vec(b)?;
                table.insert(b.api_key.as_str(), encoded.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}
