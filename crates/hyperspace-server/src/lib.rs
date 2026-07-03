/// Minimal library surface for `hyperspace-server`.
///
/// Currently only exposes sub-modules needed by both the main binary and
/// any future library consumers. The primary implementation lives in `main.rs`.
///
/// NOTE: `Args`, `HyperspaceService`, and `start_server` are intentionally
/// NOT re-exported here — they live in `main.rs` (the binary entry point) and
/// are accessed by `hyperspace-miner` via the DePIN env-var protocol described
/// in `src/bin/hyperspace-miner.rs`.
pub mod audit_log;
pub mod chunk_backend;
pub mod chunk_searcher;
pub mod collection;
pub mod gossip;
pub mod http_server;
pub mod manager;
pub mod meta_router;
pub mod security;
pub mod sync;
pub mod wave;
pub mod write_buffer;
