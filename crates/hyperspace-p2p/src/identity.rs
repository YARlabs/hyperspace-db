/// Node identity for the DePIN network.
///
/// Derives a persistent ed25519 + libp2p identity from a BIP-39 mnemonic.
/// The signing key is wrapped in `Zeroizing` so memory is wiped on drop.

use std::path::Path;

use ed25519_dalek::{SigningKey, VerifyingKey, Signer};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Logical partition of semantic space this node serves (0–255).
pub type SemanticZone = u8;

/// Persisted node identity (stored as JSON on disk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedIdentity {
    /// BIP-39 mnemonic — dev mode plaintext. Phase 3: AES-256-GCM encrypted.
    pub mnemonic: String,
    /// libp2p PeerId (base58).
    pub peer_id: String,
    /// Semantic zone assigned to this node.
    pub semantic_zone: SemanticZone,
}

/// Live node identity — held in memory during runtime.
pub struct NodeIdentity {
    pub peer_id: PeerId,
    /// Raw secret bytes, wiped from RAM on drop via Zeroizing.
    pub secret_bytes: Zeroizing<[u8; 32]>,
    /// Ed25519 verifying key (public key).
    pub verifying_key: VerifyingKey,
    pub semantic_zone: SemanticZone,
}

impl NodeIdentity {
    /// Load from disk or generate a fresh identity.
    pub fn load_or_generate<P: AsRef<Path>>(path: P, zone: SemanticZone) -> anyhow::Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            tracing::info!("Loading node identity from {}", path.display());
            let raw = std::fs::read_to_string(path)?;
            let persisted: PersistedIdentity = serde_json::from_str(&raw)?;
            Self::from_mnemonic(&persisted.mnemonic, persisted.semantic_zone)
        } else {
            tracing::info!("Generating new node identity → {}", path.display());
            let mnemonic = Self::generate_mnemonic()?;
            let identity = Self::from_mnemonic(&mnemonic, zone)?;
            let persisted = PersistedIdentity {
                mnemonic,
                peer_id: identity.peer_id.to_base58(),
                semantic_zone: identity.semantic_zone,
            };
            std::fs::write(path, serde_json::to_string_pretty(&persisted)?)?;
            tracing::info!("✅ New identity saved — PeerId: {}", identity.peer_id);
            Ok(identity)
        }
    }

    /// Derive identity from a BIP-39 mnemonic string.
    pub fn from_mnemonic(mnemonic: &str, semantic_zone: SemanticZone) -> anyhow::Result<Self> {
        use bip39::Mnemonic;
        let m: Mnemonic = mnemonic.parse().map_err(|e| anyhow::anyhow!("{e}"))?;
        let seed = m.to_seed("");
        let secret_bytes: [u8; 32] = seed[..32].try_into()?;

        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Derive libp2p PeerId from the same secret bytes.
        let mut secret_copy = secret_bytes;
        let libp2p_kp = libp2p::identity::Keypair::ed25519_from_bytes(&mut secret_copy)
            .map_err(|e| anyhow::anyhow!("libp2p keypair: {e}"))?;
        let peer_id = PeerId::from_public_key(&libp2p_kp.public());

        Ok(Self {
            peer_id,
            secret_bytes: Zeroizing::new(secret_bytes),
            verifying_key,
            semantic_zone,
        })
    }

    /// Build the libp2p `Keypair` from the stored secret bytes.
    pub fn libp2p_keypair(&self) -> anyhow::Result<libp2p::identity::Keypair> {
        let mut secret_copy = *self.secret_bytes;
        libp2p::identity::Keypair::ed25519_from_bytes(&mut secret_copy)
            .map_err(|e| anyhow::anyhow!("libp2p keypair: {e}"))
    }

    /// Return the raw 32-byte ed25519 public key.
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }

    /// Sign arbitrary bytes with the node's ed25519 key.
    pub fn sign(&self, data: &[u8]) -> [u8; 64] {
        let signing_key = SigningKey::from_bytes(&*self.secret_bytes);
        signing_key.sign(data).to_bytes()
    }

    /// Generate a fresh 12-word BIP-39 English mnemonic.
    fn generate_mnemonic() -> anyhow::Result<String> {
        use bip39::Mnemonic;
        // bip39 2.x: Mnemonic::generate(word_count) returns Result<Mnemonic, Error>
        let m = Mnemonic::generate(12).map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(m.to_string())
    }
}
