//! Compiles cnt-rust's cabi/streamout.rs in its `_USER32_WSPRINTF` configuration.
fn main() {
    println!("cargo:rustc-check-cfg=cfg(streamout_user32)");
    println!("cargo:rustc-cfg=streamout_user32");
    println!("cargo:rerun-if-changed=../cnt-rust/src/cabi/streamout.rs");
}
