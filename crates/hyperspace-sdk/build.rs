fn main() {
    println!("cargo::rustc-check-cfg=cfg(has_nightly)");
    println!("cargo:rerun-if-changed=build.rs");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    if let Ok(output) = std::process::Command::new(rustc).arg("--version").output() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        if version_str.contains("nightly") || version_str.contains("dev") {
            println!("cargo:rustc-cfg=has_nightly");
        }
    }
}
