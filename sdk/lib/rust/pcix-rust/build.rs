// pcix-rust contains C-variadic FFI (pci/id.rs: PciIdPrintf / PciIdPrintfAppend,
// faithful translations of id.c's va_start/vsprintf functions). Those require the
// nightly-only `c_variadic` feature. The real driver build is nightly (build-std;
// see sdk/cmake/gcc.cmake RUSTUP_TOOLCHAIN=nightly), and RUSTC_BOOTSTRAP=1 also
// unlocks the feature on a stable toolchain. Set the `pcix_nightly` cfg whenever
// unstable features are available so lib.rs can gate `feature(c_variadic)` on it,
// which keeps a plain-stable `cargo check` from failing with E0554 while still
// enabling the feature on the real (nightly / bootstrap) build path.

use std::process::Command;

fn main() {
    println!("cargo:rustc-check-cfg=cfg(pcix_nightly)");
    // Re-run when the bootstrap env flips, else cargo caches a stale (no-cfg) result.
    println!("cargo:rerun-if-env-changed=RUSTC_BOOTSTRAP");
    println!("cargo:rerun-if-env-changed=RUSTUP_TOOLCHAIN");

    // RUSTC_BOOTSTRAP=1 forces stable to accept unstable features.
    let bootstrap = std::env::var("RUSTC_BOOTSTRAP").map(|v| v == "1").unwrap_or(false);

    // Detect a nightly channel from `rustc -vV`.
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let nightly = Command::new(&rustc)
        .arg("-vV")
        .output()
        .ok()
        .map(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.contains("-nightly") || s.contains("-dev")
        })
        .unwrap_or(false);

    if bootstrap || nightly {
        println!("cargo:rustc-cfg=pcix_nightly");
    }
}
