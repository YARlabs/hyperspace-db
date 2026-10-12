//! SGEMM backend. Default: pure-Rust `matrixmultiply`. With a BLAS backend
//! feature the same calls route through `cblas_sgemm`:
//!   * `accelerate`      — Apple Accelerate (AMX-backed on Apple Silicon)
//!   * `openblas`        — OpenBLAS, built from source (Linux / Windows / macOS)
//!   * `openblas-system` — link a preinstalled system OpenBLAS
//! All three enable the internal `blas` marker feature; exactly one backend
//! should be selected. OpenBLAS is linked by the `openblas-src` crate (see
//! `lib.rs`); Accelerate is a system framework linked here.

#[cfg(feature = "blas")]
mod blas_backend {
    // CBLAS constants: RowMajor=101, NoTrans=111, Trans=112. Both Accelerate
    // and OpenBLAS expose this identical symbol/ABI.
    #[cfg_attr(feature = "accelerate", link(name = "Accelerate", kind = "framework"))]
    unsafe extern "C" {
        pub fn cblas_sgemm(
            order: i32,
            transa: i32,
            transb: i32,
            m: i32,
            n: i32,
            k: i32,
            alpha: f32,
            a: *const f32,
            lda: i32,
            b: *const f32,
            ldb: i32,
            beta: f32,
            c: *mut f32,
            ldc: i32,
        );
    }
}

/// General SGEMM: `c = op(a) * op(b)`, row-major, with explicit leading
/// dimensions. `trans_a`/`trans_b` transpose the respective operand.
///   a: (trans_a ? k×m : m×k), leading dim `lda`
///   b: (trans_b ? n×k : k×n), leading dim `ldb`
///   c: m×n, leading dim `ldc`
#[inline]
pub fn sgemm_ld(
    trans_a: bool,
    trans_b: bool,
    m: usize,
    k: usize,
    n: usize,
    a: &[f32],
    lda: usize,
    b: &[f32],
    ldb: usize,
    c: &mut [f32],
    ldc: usize,
) {
    #[cfg(feature = "blas")]
    unsafe {
        blas_backend::cblas_sgemm(
            101,
            if trans_a { 112 } else { 111 },
            if trans_b { 112 } else { 111 },
            m as i32,
            n as i32,
            k as i32,
            1.0,
            a.as_ptr(),
            lda as i32,
            b.as_ptr(),
            ldb as i32,
            0.0,
            c.as_mut_ptr(),
            ldc as i32,
        );
    }
    #[cfg(not(feature = "blas"))]
    unsafe {
        let (rsa, csa) = if trans_a {
            (1isize, lda as isize)
        } else {
            (lda as isize, 1)
        };
        let (rsb, csb) = if trans_b {
            (1isize, ldb as isize)
        } else {
            (ldb as isize, 1)
        };
        matrixmultiply::sgemm(
            m,
            k,
            n,
            1.0,
            a.as_ptr(),
            rsa,
            csa,
            b.as_ptr(),
            rsb,
            csb,
            0.0,
            c.as_mut_ptr(),
            ldc as isize,
            1,
        );
    }
}

/// Computes `c = a * b` (no transposes), all row-major.
///   `a`: m × k, `b`: k × n, `c`: m × n
pub fn sgemm_row_major(m: usize, k: usize, n: usize, a: &[f32], b: &[f32], c: &mut [f32]) {
    debug_assert!(a.len() >= m * k);
    debug_assert!(b.len() >= k * n);
    debug_assert!(c.len() >= m * n);
    sgemm_ld(false, false, m, k, n, a, k, b, n, c, n);
}

/// Computes `c = a * b^T`, all row-major in storage.
///   `a`: m × k, `b`: n × k (accessed transposed), `c`: m × n
pub fn sgemm_row_major_b_transposed(
    m: usize,
    k: usize,
    n: usize,
    a: &[f32],
    b: &[f32],
    c: &mut [f32],
) {
    debug_assert!(a.len() >= m * k);
    debug_assert!(b.len() >= n * k);
    debug_assert!(c.len() >= m * n);
    sgemm_ld(false, true, m, k, n, a, k, b, k, c, n);
}
