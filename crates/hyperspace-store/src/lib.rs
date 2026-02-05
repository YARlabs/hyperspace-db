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
