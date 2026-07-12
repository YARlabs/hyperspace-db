/// Minimal library surface for `hyperspace-server`.
///
/// Exposes `Args` and `start_server` so that the `hyperspace-miner` binary
/// can call them directly as a unified single process.
pub mod audit_log;
pub mod chunk_backend;
pub mod chunk_searcher;
pub mod collection;
pub mod gossip;
pub mod http_server;
pub mod manager;
pub mod meta_router;
pub mod security;
pub mod server_args;
pub mod sync;
pub mod wave;
pub mod write_buffer;

// Re-export startup API for hyperspace-miner
pub use server_args::Args;
