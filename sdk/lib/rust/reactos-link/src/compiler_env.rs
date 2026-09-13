//! Switch nested Cargo to the selected compiler without disabling its cache.

use std::process::Command;

pub fn scrub_outer_compiler(command: &mut Command) {
    // The outer build may use nightly/build-std while this child uses stable.
    // RUSTC_WRAPPER is a compiler-independent cache wrapper, not a compiler pin:
    // retain it so the CLI's sccache configuration reaches nested Rust builds.
    for variable in [
        "RUSTC",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTDOC",
        "RUSTUP_TOOLCHAIN",
        "CARGO",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_BUILD_RUSTFLAGS",
        "CARGO_BUILD_RUSTC",
        "CARGO_UNSTABLE_BUILD_STD",
        "CARGO_UNSTABLE_BUILD_STD_FEATURES",
    ] {
        command.env_remove(variable);
    }
}
