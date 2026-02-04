# HyperspaceDB Engineering To-Do List

## CRITICAL (Synchronization Foundation)
- [x] **Implementation of Lamport Logical Clocks**
    - [x] **Goal:** Enable causal ordering of events across distributed nodes.
    - [x] Modify `ClusterState` to safe-guard atomic `logical_clock`.
    - [x] Implement `tick()` method to increment clock on local events.
    - [x] Implement `merge(remote_clock)` method (`max(local, remote) + 1`).
    - [x] Integrate into `Insert` flow: update clock before WAL write.
    - [x] Integrate into `ReplicationLog`: ensure emitted logs carry the new clock value.
    - [x] **Test:** Unit test for `tick` and `merge`.

- [x] **Harden Node Identity & Persistence**
    - [x] **Goal:** Ensure `node_id` is immutable and ubiquitous.
    - [x] Verify `node_id` injection in `CollectionImpl` (Review recent hot-fix).
    - [x] Ensure WAL entries verify `node_id` correctness on recovery (basic check).

## IMPORTANT (Data Integrity & Code Health)
- [x] **Refactor `manager.rs` Collection Instantiation**
    - [x] **Goal:** Reduce code duplication and error prone `match` block.
    - [x] Create a `CollectionFactory` or helper trait to handle the `16..2048` generic dispatch cleanly.
    - [x] Remove the massive 100-line match duplication introduced in Sprint 1.

- [x] **Data Drift Detection (Merkle/Hash)**
    - [x] **Goal:** Efficiently compare data state between nodes.
    - [x] Implement `CollectionDigest`: A rolling hash of all vector IDs + timestamps (or clock).
    - [x] Add `GET /api/collections/{name}/digest` endpoint.
    - [x] Implement Bucket Merkle Tree (256 buckets)
    - [x] Add gRPC `GetDigest` endpoint


## MINOR (Polish & UX)
- [x] **Linter & Compiler Clean-up**
    - [x] Fix `unused_imports` and `dead_code` warnings in `http_server.rs` and `manager.rs`.
- [x] **Dashboard Sync Status**
    - [x] Update "Cluster Nodes" page to show real `logical_clock` ticking in real-time. (Implemented via polling)

## COMPLETED ✅
- [x] **Full Merkle Tree**: Implement hierarchical hashing for faster diffing (Sprint 4).
    - [x] Bucket-based XOR hashing (256 buckets)
    - [x] gRPC `GetDigest` endpoint
    - [x] HTTP `/api/collections/{name}/digest` endpoint
    - [x] SDK support (Rust SDK updated)
    - [x] Cluster simulation test created

## NEXT STEPS
- [ ] **Python & TypeScript SDK Updates**: Add `get_digest` methods
- [ ] **Production Hardening**: Error handling, retries, backpressure
- [ ] **Performance Testing**: Run cluster_test and benchmark latency
- [ ] **Documentation**: Update API docs with sync protocol

---

## Progress Log
*Initial Setup: Creating TODO list.*
