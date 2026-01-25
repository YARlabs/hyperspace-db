#![feature(portable_simd)]

pub mod config;
pub mod metric;
pub mod vector;

pub use config::GlobalConfig;

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMode {
    None,
    ScalarI8,
    Binary,
}
