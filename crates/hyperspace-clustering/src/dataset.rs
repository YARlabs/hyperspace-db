//! Replayable sources of [`Matrix`] batches for streaming k-means.

use std::convert::Infallible;

use crate::matrix::Matrix;

/// A sequential scan of row-major [`Matrix`] batches that can be replayed.
///
/// Streaming training walks the data once to pick the starting centroids and
/// once per Lloyd iteration, so every call to [`Dataset::for_each_batch`] must
/// yield the **same vectors in the same order**. Rows are identified by their
/// 0-based position in that order.
///
/// Each batch is `n × d` (`n` may be 1). Training packs small batches into
/// GEMM-sized work batches, so yielding one vector at a time is correct but
/// slower than yielding larger batches.
pub trait Dataset {
    /// Error a scan can fail with, e.g. an IO error or a cancellation.
    type Error;

    /// Call `f` once per batch, in order. Stops at the first error.
    fn for_each_batch(&mut self, f: &mut dyn FnMut(Matrix<'_>)) -> Result<(), Self::Error>;
}

/// A [`Clone`] iterator of [`Matrix`] batches, replayed by cloning it on
/// every pass.
///
/// `clone()` has to produce the same sequence again, which holds for
/// iterators over in-memory data such as [`Matrix::chunks`].
pub struct IterDataset<I> {
    iter: I,
}

impl<I> IterDataset<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'a, I> Dataset for IterDataset<I>
where
    I: Iterator<Item = Matrix<'a>> + Clone,
{
    type Error = Infallible;

    fn for_each_batch(&mut self, f: &mut dyn FnMut(Matrix<'_>)) -> Result<(), Infallible> {
        for matrix in self.iter.clone() {
            if !matrix.is_empty() {
                f(matrix);
            }
        }
        Ok(())
    }
}

impl Dataset for Matrix<'_> {
    type Error = Infallible;

    fn for_each_batch(&mut self, f: &mut dyn FnMut(Matrix<'_>)) -> Result<(), Infallible> {
        if !self.is_empty() {
            f(*self);
        }
        Ok(())
    }
}

/// Walk `data`, coalescing small matrices and splitting large ones so each
/// call to `f` sees about `batch_rows` vectors (the GEMM / rayon width).
///
/// Large batches are split without copying; smaller ones are packed into
/// `buf`.
pub(crate) fn visit_work_batches<D, F>(
    data: &mut D,
    d: usize,
    batch_rows: usize,
    buf: &mut Vec<f32>,
    mut f: F,
) -> Result<(), D::Error>
where
    D: Dataset + ?Sized,
    F: FnMut(Matrix<'_>),
{
    assert!(d > 0, "dimensionality must be positive");
    assert!(batch_rows > 0, "batch_rows must be positive");
    buf.clear();

    data.for_each_batch(&mut |matrix| {
        assert_eq!(
            matrix.d(),
            d,
            "matrix dimensionality {} does not match the model ({d})",
            matrix.d()
        );
        if matrix.is_empty() {
            return;
        }

        if buf.is_empty() && matrix.n() >= batch_rows {
            for chunk in matrix.chunks(batch_rows) {
                if chunk.n() == batch_rows {
                    f(chunk);
                } else {
                    buf.extend_from_slice(chunk.as_slice());
                }
            }
            return;
        }

        for row in matrix.rows() {
            if buf.is_empty() {
                buf.reserve(batch_rows * d);
            }
            buf.extend_from_slice(row);
            if buf.len() == batch_rows * d {
                f(Matrix::new(buf, batch_rows, d));
                buf.clear();
            }
        }
    })?;

    if !buf.is_empty() {
        let n = buf.len() / d;
        f(Matrix::new(buf, n, d));
        buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_unit_matrices_and_splits_large_ones() {
        let data: Vec<f32> = (0..20).map(|i| i as f32).collect();
        let d = 2;
        let matrices: Vec<Matrix<'_>> = data.chunks(d).map(Matrix::vector).collect();
        let mut dataset = IterDataset::new(matrices.into_iter());

        let mut buf = Vec::new();
        let mut seen = Vec::new();
        visit_work_batches(&mut dataset, d, 3, &mut buf, |m| {
            seen.push((m.n(), m.as_slice().to_vec()));
        })
        .unwrap();
        assert_eq!(seen.len(), 4);
        assert_eq!(seen[0].0, 3);
        assert_eq!(seen[3].0, 1);
        assert_eq!(seen[3].1, vec![18.0, 19.0]);

        // A single large matrix is split without changing values.
        let mut whole = Matrix::from_slice(&data, d);
        let mut sizes = Vec::new();
        visit_work_batches(&mut whole, d, 4, &mut buf, |m| {
            sizes.push(m.n());
        })
        .unwrap();
        assert_eq!(sizes, vec![4, 4, 2]);
    }

    /// A short tail of a large matrix is packed together with whatever comes
    /// next rather than handed on as its own tiny batch.
    #[test]
    fn short_tails_are_packed_with_the_next_matrix() {
        let data: Vec<f32> = (0..14).map(|i| i as f32).collect();
        let d = 2;
        let (first, second) = data.split_at(5 * d);
        let mut dataset = IterDataset::new(
            [Matrix::from_slice(first, d), Matrix::from_slice(second, d)].into_iter(),
        );

        let mut buf = Vec::new();
        let mut seen = Vec::new();
        visit_work_batches(&mut dataset, d, 4, &mut buf, |m| {
            seen.push(m.as_slice().to_vec());
        })
        .unwrap();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen.concat(), data);
    }

    #[test]
    fn iter_dataset_replays_in_the_same_order() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let a = Matrix::new(&data[..2], 1, 2);
        let b = Matrix::new(&data[2..], 1, 2);
        let mut dataset = IterDataset::new([a, b].into_iter());

        let mut first = Vec::new();
        dataset
            .for_each_batch(&mut |m| first.push(m.as_slice().to_vec()))
            .unwrap();
        let mut second = Vec::new();
        dataset
            .for_each_batch(&mut |m| second.push(m.as_slice().to_vec()))
            .unwrap();
        assert_eq!(first, second);
    }

    /// A failing scan stops and hands its error back to the caller.
    #[test]
    fn scan_errors_propagate() {
        struct Failing;
        impl Dataset for Failing {
            type Error = &'static str;
            fn for_each_batch(
                &mut self,
                f: &mut dyn FnMut(Matrix<'_>),
            ) -> Result<(), &'static str> {
                f(Matrix::vector(&[1.0, 2.0]));
                Err("disk on fire")
            }
        }
        let mut buf = Vec::new();
        let result = visit_work_batches(&mut Failing, 2, 4, &mut buf, |_| {});
        assert_eq!(result, Err("disk on fire"));
    }
}
