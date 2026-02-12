#![allow(clippy::pedantic)]
#![allow(clippy::all)]
#![allow(unexpected_cfgs)] // Also silencing this common issue with generated code

pub mod hyperspace {
    tonic::include_proto!("hyperspace");
}
