//! Link a system `OpenBLAS` when the `openblas` feature is enabled.
//!
//! Discovery order:
//!   1. `OPENBLAS_LIB_DIR` env var — explicit override (Windows / custom installs).
//!   2. `pkg-config` for `openblas` (Linux, macOS/Homebrew with `PKG_CONFIG_PATH` set).
//!   3. Bare `-lopenblas`, trusting the linker's default search path.
//!
//! Accelerate needs nothing here — it is linked via a framework attribute.

#![allow(clippy::pedantic)]

fn main() {
    println!("cargo:rerun-if-env-changed=OPENBLAS_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_OPENBLAS").is_none() {
        return;
    }
    if let Some(dir) = std::env::var_os("OPENBLAS_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
        println!("cargo:rustc-link-lib=openblas");
        return;
    }
    if pkg_config::Config::new().probe("openblas").is_ok() {
        return; // pkg-config emitted the search path + link directives.
    }
    println!("cargo:rustc-link-lib=openblas");
}
