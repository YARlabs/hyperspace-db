#![allow(clippy::pedantic)]
#![allow(clippy::all)]
#![allow(unexpected_cfgs)] // Also silencing this common issue with generated code

pub mod hyperspace {
    tonic::include_proto!("hyperspace");
}

/// Wire-format helper: requests may carry the vector as compact `repeated float`
/// (`vector_f32`) instead of `repeated double` (`vector`). Servers call
/// [`F32Vector::normalize_vector`] once on receipt so the rest of the pipeline only sees `vector`.
pub trait F32Vector {
    /// Moves `vector_f32` into `vector` (as `f64`) when `vector` is empty.
    fn normalize_vector(&mut self);
}

macro_rules! impl_f32_vector {
    ($($t:ty),*) => {$(
        impl F32Vector for $t {
            #[inline]
            fn normalize_vector(&mut self) {
                if self.vector.is_empty() && !self.vector_f32.is_empty() {
                    self.vector = self.vector_f32.drain(..).map(f64::from).collect();
                }
            }
        }
    )*};
}

impl_f32_vector!(
    hyperspace::SearchRequest,
    hyperspace::InsertRequest,
    hyperspace::VectorData,
    hyperspace::SearchMultiCollectionRequest
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_vector_is_used_only_when_f64_is_empty() {
        let mut req = hyperspace::SearchRequest {
            vector_f32: vec![0.5, -1.25],
            ..Default::default()
        };
        req.normalize_vector();
        assert_eq!(req.vector, vec![0.5, -1.25]);
        assert!(req.vector_f32.is_empty());

        let mut both = hyperspace::InsertRequest {
            vector: vec![1.0],
            vector_f32: vec![2.0],
            ..Default::default()
        };
        both.normalize_vector();
        assert_eq!(both.vector, vec![1.0]);
    }
}
