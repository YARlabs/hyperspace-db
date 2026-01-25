#![feature(portable_simd)]

pub mod metric;
pub mod vector;

pub type HyperFloat = f64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantizationMode {
    None,
    ScalarI8,
}
