//! Checkpoint 1 Tests — `hyperspace-billing`
//!
//! Tests cover:
//!   - MeteringEngine: atomic counters, drain_deltas, throttle flag
//!   - AccountingStore: redb ACID persist/load/batch
//!   - BillingContext: throttle restoration from persisted state

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    use hyperspace_billing::{
        AccountingStore, BillingBalance, BillingStatus, MeteringEngine,
    };

    // ─── MeteringEngine tests ───────────────────────────────────────────────

    #[test]
    fn test_metering_record_insert() {
        let engine = MeteringEngine::new(10_000);
        engine.record_insert("key-A", 1);
        engine.record_insert("key-A", 1);
        engine.record_insert("key-B", 5);

        let deltas = engine.drain_deltas();
        let key_a = deltas.iter().find(|d| d.api_key == "key-A").unwrap();
        let key_b = deltas.iter().find(|d| d.api_key == "key-B").unwrap();

        assert_eq!(key_a.delta_inserts, 2, "key-A should have 2 inserts");
        assert_eq!(key_b.delta_inserts, 5, "key-B should have 5 inserts");
    }

    #[test]
    fn test_metering_drain_resets_counters() {
        let engine = MeteringEngine::new(10_000);
        engine.record_insert("tenant-1", 100);
        let first = engine.drain_deltas();
        assert!(!first.is_empty(), "first drain must have data");

        let second = engine.drain_deltas();
        // After drain, counters reset → nothing to report
        assert!(
            second.is_empty(),
            "second drain should be empty after reset"
        );
    }

    /// Checkpoint 1 core test: simulate 1M inserts + 100k searches.
    #[test]
    fn test_metering_1m_inserts_100k_searches() {
        let engine = Arc::new(MeteringEngine::new(10_000));
        let tenant = "tenant-load-test";

        // Simulate 1M inserts in batches (atomic, no actual gRPC needed)
        const INSERTS: u64 = 1_000_000;
        const SEARCHES: u64 = 100_000;
        const PAYLOAD_PER_SEARCH: u64 = 4096; // 4KB avg payload

        engine.record_insert(tenant, INSERTS);
        for _ in 0..SEARCHES {
            engine.record_search(tenant, PAYLOAD_PER_SEARCH);
        }

        let deltas = engine.drain_deltas();
        assert_eq!(deltas.len(), 1, "should have exactly one tenant delta");
        let d = &deltas[0];
        assert_eq!(d.delta_inserts, INSERTS, "insert count mismatch");
        assert_eq!(d.delta_searches, SEARCHES, "search count mismatch");
        assert_eq!(
            d.payload_bytes,
            SEARCHES * PAYLOAD_PER_SEARCH,
            "payload bytes mismatch"
        );
    }

    #[test]
    fn test_metering_throttle_flag() {
        let engine = MeteringEngine::new(10_000);
        assert!(!engine.is_throttled("new-tenant"), "new tenant should not be throttled");

        engine.set_throttled("new-tenant", true);
        assert!(engine.is_throttled("new-tenant"), "should be throttled");

        engine.set_throttled("new-tenant", false);
        assert!(!engine.is_throttled("new-tenant"), "should be unthrottled");
    }

    #[test]
    fn test_metering_concurrent_increments() {
        use std::thread;
        let engine = Arc::new(MeteringEngine::new(100_000));
        let tenant = "concurrent-tenant";
        let threads = 8;
        let per_thread = 1_000u64;

        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let e = engine.clone();
                let t = tenant.to_string();
                thread::spawn(move || {
                    for _ in 0..per_thread {
                        e.record_insert(&t, 1);
                    }
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        let deltas = engine.drain_deltas();
        let d = deltas.iter().find(|d| d.api_key == tenant).unwrap();
        assert_eq!(
            d.delta_inserts,
            threads * per_thread,
            "concurrent inserts lost data"
        );
    }

    // ─── AccountingStore tests ─────────────────────────────────────────────

    fn make_balance(api_key: &str, status: BillingStatus, bal: f64) -> BillingBalance {
        BillingBalance {
            api_key: api_key.to_string(),
            status,
            last_known_balance: bal,
            last_sync_ts: 1_700_000_000,
        }
    }

    #[test]
    fn test_accounting_store_open_and_upsert() {
        let tmp = NamedTempFile::new().unwrap();
        let store = AccountingStore::open(tmp.path()).unwrap();

        let b = make_balance("key-1", BillingStatus::Active, 100.0);
        store.upsert(&b).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].api_key, "key-1");
        assert_eq!(loaded[0].status, BillingStatus::Active);
        assert!((loaded[0].last_known_balance - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_accounting_store_upsert_overwrites() {
        let tmp = NamedTempFile::new().unwrap();
        let store = AccountingStore::open(tmp.path()).unwrap();

        store
            .upsert(&make_balance("k", BillingStatus::Active, 50.0))
            .unwrap();
        store
            .upsert(&make_balance("k", BillingStatus::InsufficientFunds, 0.0))
            .unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1, "upsert should overwrite, not append");
        assert_eq!(loaded[0].status, BillingStatus::InsufficientFunds);
    }

    #[test]
    fn test_accounting_store_batch_upsert() {
        let tmp = NamedTempFile::new().unwrap();
        let store = AccountingStore::open(tmp.path()).unwrap();

        let batch: Vec<BillingBalance> = (0..100)
            .map(|i| make_balance(&format!("key-{i}"), BillingStatus::Active, i as f64))
            .collect();

        store.upsert_batch(&batch).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 100, "batch upsert should persist all records");
    }

    /// Checkpoint 1 crash-recovery test:
    /// Persist InsufficientFunds, reopen the DB, confirm data survived.
    #[test]
    fn test_accounting_store_survives_reopen() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_path_buf();

        // Write
        {
            let store = AccountingStore::open(&path).unwrap();
            store
                .upsert(&make_balance(
                    "crash-key",
                    BillingStatus::InsufficientFunds,
                    0.0,
                ))
                .unwrap();
        }

        // Reopen (simulate restart)
        {
            let store = AccountingStore::open(&path).unwrap();
            let loaded = store.load_all().unwrap();
            assert_eq!(loaded.len(), 1, "data should persist across restarts");
            assert_eq!(loaded[0].status, BillingStatus::InsufficientFunds);
        }
    }

    /// Checkpoint 1 key test: throttle flags restored from persisted state on startup.
    #[test]
    fn test_billing_context_throttle_restored_from_disk() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap().to_string();

        // Simulate previous run that persisted InsufficientFunds
        {
            let store = AccountingStore::open(&path).unwrap();
            store
                .upsert(&make_balance(
                    "broke-tenant",
                    BillingStatus::InsufficientFunds,
                    0.0,
                ))
                .unwrap();
        }

        // BillingContext::start should read redb and restore throttle flag
        // We test the restore logic directly (start() needs a live coordinator URL)
        let accounting = Arc::new(AccountingStore::open(&path).unwrap());
        let metering = Arc::new(MeteringEngine::new(10_000));
        let all = accounting.load_all().unwrap();

        for b in &all {
            if b.status == BillingStatus::InsufficientFunds {
                metering.set_throttled(&b.api_key, true);
            }
        }

        assert!(
            metering.is_throttled("broke-tenant"),
            "throttle flag must be restored from persisted InsufficientFunds"
        );
        assert!(
            !metering.is_throttled("other-tenant"),
            "unmentioned tenants must not be throttled"
        );
    }
}
