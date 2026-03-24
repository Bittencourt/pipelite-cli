use std::process::Command;

fn main() {
    // Capture rustc version for rich --version output
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|s| {
            // Extract version number from "rustc X.Y.Z (hash date)"
            s.split_whitespace().nth(1).map(String::from)
        })
        .unwrap_or_else(|| "unknown".to_string());

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Set a single BUILD_VERSION env var with the full version string
    println!(
        "cargo:rustc-env=BUILD_VERSION={} (rustc {}, {}-{})",
        env!("CARGO_PKG_VERSION"),
        rustc_version,
        os,
        arch
    );

    // Tell cargo to re-run if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");
}
