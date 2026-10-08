#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::len_without_is_empty)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::uninlined_format_args)]
#![allow(unknown_lints)]
#![allow(clippy::assert_is_empty)]
#![allow(clippy::needless_bool)]

// Direct I/O storage engine (v4.0.0)
#[cfg(not(target_arch = "wasm32"))]
pub mod direct_io;
#[cfg(not(target_arch = "wasm32"))]
pub use direct_io::{AlignedBuffer, DirectFile, DirectVectorStore, DIRECT_IO_ALIGNMENT};

// Sidecar Payload Storage: available regardless of vector storage backend.
#[cfg(not(target_arch = "wasm32"))]
pub mod payload_store;
#[cfg(not(target_arch = "wasm32"))]
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
