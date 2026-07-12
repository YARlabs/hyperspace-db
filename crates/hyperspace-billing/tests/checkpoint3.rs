//! Phase 3 Checkpoint Tests — SignedTicket Cryptographic Layer
//!
//! Tests cover:
//!   - sign/verify roundtrip
//!   - tampered message detection
//!   - expiry check
//!   - replay protection
//!   - PendingTicketStore: push/load/settle_batch (ACID)
//!   - Crash recovery: tickets survive process restart

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    use tempfile::NamedTempFile;

    use hyperspace_billing::{PendingTicketStore, SignedTicket, TicketVerifier};

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn fresh_key() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    fn make_ticket(
        issuer_key: &SigningKey,
        recipient_pubkey: [u8; 32],
        amount: u64,
        ttl_secs: i64,
    ) -> SignedTicket {
        let request_id = rand::random::<[u8; 16]>();
        let expires_at = (now_secs() as i64 + ttl_secs) as u64;
        SignedTicket::sign(issuer_key, recipient_pubkey, amount, request_id, expires_at)
    }

    // ─── Sign / Verify ─────────────────────────────────────────────────────

    #[test]
    fn test_ticket_sign_and_verify() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 1000, 30);
        assert!(
            ticket.verify_signature().is_ok(),
            "Valid ticket should verify"
        );
    }

    #[test]
    fn test_ticket_tampered_amount_fails() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let mut ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 1000, 30);
        ticket.amount_microusd = 999_999; // tamper
        assert!(
            ticket.verify_signature().is_err(),
            "Tampered amount should fail verification"
        );
    }

    #[test]
    fn test_ticket_tampered_pubkey_fails() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let mut ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 500, 30);
        ticket.recipient_pubkey[0] ^= 0xFF; // tamper
        assert!(
            ticket.verify_signature().is_err(),
            "Tampered pubkey should fail verification"
        );
    }

    #[test]
    fn test_ticket_wrong_key_fails() {
        let issuer = fresh_key();
        let attacker = fresh_key();
        let recipient = fresh_key();

        let mut ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, 30);
        // Replace issuer pubkey with attacker's pubkey (signature still belongs to issuer)
        ticket.issuer_pubkey = attacker.verifying_key().to_bytes();
        assert!(
            ticket.verify_signature().is_err(),
            "Wrong issuer pubkey should fail verification"
        );
    }

    // ─── Expiry ────────────────────────────────────────────────────────────

    #[test]
    fn test_ticket_not_expired() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, 300); // 5 min TTL
        assert!(!ticket.is_expired(), "Fresh ticket should not be expired");
    }

    #[test]
    fn test_ticket_expired() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, -1); // expired 1 sec ago
        assert!(ticket.is_expired(), "Ticket with past expires_at should be expired");
    }

    // ─── TicketVerifier + Replay Guard ─────────────────────────────────────

    #[test]
    fn test_verifier_accepts_valid_ticket() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, 30);

        let mut verifier = TicketVerifier::new();
        assert!(
            verifier.verify(&ticket).is_ok(),
            "Valid ticket should be accepted"
        );
    }

    #[test]
    fn test_verifier_rejects_replay() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, 30);

        let mut verifier = TicketVerifier::new();
        verifier.verify(&ticket).unwrap();

        // Try to replay the same ticket
        let result = verifier.verify(&ticket);
        assert!(
            result.is_err(),
            "Replayed ticket should be rejected"
        );
        assert!(
            result.unwrap_err().to_string().contains("Replay"),
            "Error should mention replay attack"
        );
    }

    #[test]
    fn test_verifier_rejects_expired() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, -5);

        let mut verifier = TicketVerifier::new();
        let result = verifier.verify(&ticket);
        // Note: signature is valid, but the ticket is expired
        assert!(result.is_err(), "Expired ticket must be rejected");
    }

    #[test]
    fn test_verifier_accepts_multiple_different_tickets() {
        let issuer = fresh_key();
        let recipient = fresh_key();
        let mut verifier = TicketVerifier::new();

        for _ in 0..100 {
            let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 1, 30);
            assert!(
                verifier.verify(&ticket).is_ok(),
                "Each unique ticket should be accepted"
            );
        }
    }

    // ─── PendingTicketStore (redb) ─────────────────────────────────────────

    #[test]
    fn test_pending_store_push_and_load() {
        let tmp = NamedTempFile::new().unwrap();
        let store = PendingTicketStore::open(tmp.path()).unwrap();

        let issuer = fresh_key();
        let recipient = fresh_key();
        let t1 = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 100, 30);
        let t2 = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 200, 30);

        store.push(&t1).unwrap();
        store.push(&t2).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 2, "Should have 2 pending tickets");
    }

    #[test]
    fn test_pending_store_settle_batch() {
        let tmp = NamedTempFile::new().unwrap();
        let store = PendingTicketStore::open(tmp.path()).unwrap();

        let issuer = fresh_key();
        let recipient = fresh_key();
        let tickets: Vec<SignedTicket> = (0..5)
            .map(|_| make_ticket(&issuer, recipient.verifying_key().to_bytes(), 50, 30))
            .collect();

        for t in &tickets {
            store.push(t).unwrap();
        }
        assert_eq!(store.len().unwrap(), 5);

        // Settle first 3
        let to_settle: Vec<[u8; 16]> = tickets[..3].iter().map(|t| t.request_id).collect();
        store.settle_batch(&to_settle).unwrap();

        assert_eq!(store.len().unwrap(), 2, "Should have 2 remaining after settling 3");
    }

    /// Checkpoint 3 crash recovery test:
    /// Ticket written to redb must survive simulated restart (reopen DB).
    #[test]
    fn test_pending_store_crash_recovery() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        let issuer = fresh_key();
        let recipient = fresh_key();
        let ticket = make_ticket(&issuer, recipient.verifying_key().to_bytes(), 999, 3600);
        let expected_id = ticket.request_id;

        // Write
        {
            let store = PendingTicketStore::open(&path).unwrap();
            store.push(&ticket).unwrap();
        }

        // "Crash and restart" — reopen
        {
            let store = PendingTicketStore::open(&path).unwrap();
            let loaded = store.load_all().unwrap();
            assert_eq!(loaded.len(), 1, "Ticket must survive restart");
            assert_eq!(loaded[0].request_id, expected_id);
            assert_eq!(loaded[0].amount_microusd, 999);
        }
    }

    /// ACID test: partial settle must not corrupt remaining tickets.
    #[test]
    fn test_pending_store_settle_is_atomic() {
        let tmp = NamedTempFile::new().unwrap();
        let store = PendingTicketStore::open(tmp.path()).unwrap();

        let issuer = fresh_key();
        let recipient = fresh_key();
        let tickets: Vec<SignedTicket> = (0..10)
            .map(|_| make_ticket(&issuer, recipient.verifying_key().to_bytes(), 1, 30))
            .collect();

        for t in &tickets {
            store.push(t).unwrap();
        }

        // Settle even-indexed tickets (5 out of 10)
        let even_ids: Vec<[u8; 16]> = tickets
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, t)| t.request_id)
            .collect();

        store.settle_batch(&even_ids).unwrap();

        let remaining = store.load_all().unwrap();
        assert_eq!(remaining.len(), 5, "Exactly 5 odd-indexed tickets should remain");
    }
}
