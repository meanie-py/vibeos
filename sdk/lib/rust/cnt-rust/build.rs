//! `cnt_libm`: set when the floating-point crates (`libm`, `musl-math`) can build for this
//! target -- everything except the BUILTIN i686 triple with -sse,-sse2, where rustc rejects
//! their Rust-ABI f64 functions (i386 ntdll/win32k cells and libcntpr). The i386 kernel
//! cells' x86-softfloat json reports `soft-float` and qualifies. Cargo.toml's
//! `[target.'cfg(...)'.dependencies]` states the same condition; this names it once for
//! the source.
fn main() {
    println!("cargo:rustc-check-cfg=cfg(cnt_libm)");
    // set by user32-wsprintf-rust, which compiles cabi/streamout.rs as its own module
    println!("cargo:rustc-check-cfg=cfg(streamout_user32)");
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let feats = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let has = |f: &str| feats.split(',').any(|x| x == f);
    if !(arch == "x86" && !has("sse2") && !has("soft-float")) {
        println!("cargo:rustc-cfg=cnt_libm");
    }
}
