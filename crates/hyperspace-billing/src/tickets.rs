//! PHASE 3: SignedTicket — Cryptographic billing micro-receipt

use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};

// ─── redb table ──────────────────────────────────────────────────────────────

const PENDING_TICKETS: TableDefinition<&[u8], &[u8]> = TableDefinition::new("pending_tickets");

// ─── Serde helpers for [u8;64] ────────────────────────────────────────────────

/// Serde serialise `[u8; 64]` as a base64 string (serde only auto-impls up to [u8;32]).
mod sig_serde {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(sig: &[u8; 64], s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        STANDARD.encode(sig).serialize(s)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        let bytes = STANDARD.decode(&s).map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes"))
    }
}

// ─── SignedTicket ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SignedTicket {
    pub issuer_pubkey: [u8; 32],
    pub recipient_pubkey: [u8; 32],
    pub amount_microusd: u64,
    pub request_id: [u8; 16],
    pub expires_at: u64,
    #[serde(with = "sig_serde")]
    pub signature: [u8; 64],
}

impl SignedTicket {
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32 + 32 + 8 + 16 + 8);
        buf.extend_from_slice(&self.issuer_pubkey);
        buf.extend_from_slice(&self.recipient_pubkey);
        buf.extend_from_slice(&self.amount_microusd.to_le_bytes());
        buf.extend_from_slice(&self.request_id);
        buf.extend_from_slice(&self.expires_at.to_le_bytes());
        buf
    }

    pub fn sign(
        signing_key: &SigningKey,
        recipient_pubkey: [u8; 32],
        amount_microusd: u64,
        request_id: [u8; 16],
        expires_at: u64,
    ) -> Self {
        let issuer_pubkey = signing_key.verifying_key().to_bytes();
        let mut ticket = SignedTicket {
            issuer_pubkey,
            recipient_pubkey,
            amount_microusd,
            request_id,
            expires_at,
            signature: [0u8; 64],
        };
        let sig = signing_key.sign(&ticket.signing_bytes());
        ticket.signature = sig.to_bytes();
        ticket
    }

    pub fn verify_signature(&self) -> anyhow::Result<()> {
        let vk = VerifyingKey::from_bytes(&self.issuer_pubkey)
            .map_err(|e| anyhow::anyhow!("Invalid issuer pubkey: {e}"))?;
        let sig = Signature::from_bytes(&self.signature);
        vk.verify(&self.signing_bytes(), &sig)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {e}"))
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.expires_at < now
    }
}

// ─── TicketVerifier ───────────────────────────────────────────────────────────

pub struct TicketVerifier {
    seen: std::collections::HashSet<[u8; 16]>,
}

impl TicketVerifier {
    pub fn new() -> Self {
        Self {
            seen: std::collections::HashSet::new(),
        }
    }

    pub fn verify(&mut self, ticket: &SignedTicket) -> anyhow::Result<()> {
        ticket.verify_signature()?;

        if ticket.is_expired() {
            return Err(anyhow::anyhow!("Ticket expired"));
        }

        if self.seen.contains(&ticket.request_id) {
            return Err(anyhow::anyhow!(
                "Replay attack detected: duplicate request_id"
            ));
        }
        self.seen.insert(ticket.request_id);
        Ok(())
    }
}

impl Default for TicketVerifier {
    fn default() -> Self {
        Self::new()
    }
}

// ─── PendingTicketStore ───────────────────────────────────────────────────────

pub struct PendingTicketStore {
    db: Database,
}

impl PendingTicketStore {
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let db = Database::create(path)?;
        let wtxn = db.begin_write()?;
        {
            wtxn.open_table(PENDING_TICKETS)?;
        }
        wtxn.commit()?;
        Ok(Self { db })
    }

    pub fn push(&self, ticket: &SignedTicket) -> anyhow::Result<()> {
        let encoded = serde_json::to_vec(ticket)?;
        let wtxn = self.db.begin_write()?;
        {
            let mut table = wtxn.open_table(PENDING_TICKETS)?;
            table.insert(ticket.request_id.as_slice(), encoded.as_slice())?;
        }
        wtxn.commit()?;
        Ok(())
    }

    pub fn load_all(&self) -> anyhow::Result<Vec<SignedTicket>> {
        let rtxn = self.db.begin_read()?;
        let table = rtxn.open_table(PENDING_TICKETS)?;
        let mut tickets = Vec::new();
        for result in table.iter()? {
            let (_, v) = result?;
            let t: SignedTicket = serde_json::from_slice(v.value())?;
            tickets.push(t);
        }
        Ok(tickets)
    }

    pub fn settle_batch(&self, request_ids: &[[u8; 16]]) -> anyhow::Result<()> {
        let wtxn = self.db.begin_write()?;
        {
            let mut table = wtxn.open_table(PENDING_TICKETS)?;
            for rid in request_ids {
                table.remove(rid.as_slice())?;
            }
        }
        wtxn.commit()?;
        Ok(())
    }

    pub fn len(&self) -> anyhow::Result<u64> {
        let rtxn = self.db.begin_read()?;
        let table = rtxn.open_table(PENDING_TICKETS)?;
        Ok(table.len()?)
    }

    pub fn is_empty(&self) -> anyhow::Result<bool> {
        Ok(self.len()? == 0)
    }
}
