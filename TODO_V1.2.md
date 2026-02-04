# HyperspaceDB v1.2.0 Delivery Plan

## CRITICAL TASKS (Documentation & SDKs)

- [x] **Documentation Update** (`docs/book/src`)
    - [x] Update `distributed.md` to reflect Federated Clustering (Logic Clock, Node ID).
    - [x] Update `api.md` with `/api/cluster/status`.
    - [x] Update `intro.md` and `quickstart.md` with new version info.
- [x] **Rust SDK** (`crates/hyperspace-sdk`)
    - [x] Review code structure.
    - [x] Run `cargo test`.
    - [x] Verify integration with v1.2.0 server.
- [x] **Python SDK** (`sdks/python`)
    - [x] Review code.
    - [x] Implement/Verify `create_collection`, `list_collections`, etc.
    - [x] Run integration tests.
- [x] **TypeScript SDK** (`sdks/ts`)
    - [x] Initialize project (package.json).
    - [x] Generate gRPC clients using `grpc-tools` or `ts-proto`.
    - [x] Implement `HyperspaceClient` class.
    - [x] Test against server. (*Note: Metadata map serialization requires dependency fix*)
- [ ] **Go SDK** (`sdks/go`) (**Deferred to Sprint 2**)
- [ ] **C++ SDK** (`sdks/cpp`) (**Deferred to Sprint 2**)
- [x] **CHANGELOG.md**
    - [x] Add entry for v1.2.0.

## IMPORTANT TASKS (Examples & Meta)

- [ ] **Examples** (`examples/`)
    - [x] `rust/basic_usage.rs`
    - [x] `python/basic_usage.py`
    - [x] `ts/basic_usage.ts`
    - [ ] `go/bulk_ingest.go` (Deferred)
    - [ ] `cpp/high_perf_client.cc` (Deferred)
- [ ] **Repository Metadata**
    - [x] Update `README.md` (Check links, badges, version).
    - [x] Update `CONTRIBUTING.md` (Add guidelines for new SDKs).
    - [ ] Update `SECURITY.md`.

## MINOR TASKS

- [ ] Final Linting & Cleanup.
- [ ] Verify all tests pass.
