#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]

// Sidecar Payload Storage: available regardless of vector storage backend.
pub mod payload_store;
pub use payload_store::{PayloadSlot, PayloadStore, DEFAULT_ZSTD_LEVEL};

#[cfg(feature = "mmap")]
pub mod wal;

#[cfg(feature = "mmap")]
mod mmap_impl;
#[cfg(feature = "mmap")]
pub use mmap_impl::VectorStore;

#[cfg(not(feature = "mmap"))]
mod ram_impl;
#[cfg(not(feature = "mmap"))]
pub use ram_impl::VectorStore;
