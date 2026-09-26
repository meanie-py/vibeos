//! Staticlib umbrella for ucrtbase.dll's Rust cohab.
//!
//! Carries no logic: `extern crate crt_shared_rust` forces that rlib's objects (and so its
//! `#[no_mangle]` symbols) into this archive, and the no_std runtime bits below make it a
//! linkable final artifact. cmake links the resulting `libcrt_shared_cargo.a` into the
//! `ucrtbase` MODULE ahead of `msvcrt_shared`, so lld first-match hands those symbols to
//! Rust and the C copies go dead in place — the same mechanism msvcrt.dll uses, but over a
//! deliberately tiny surface.
//!
//! See ../crt-shared-rust for why that surface is a separate crate rather than a feature.
#![no_std]

extern crate crt_shared_rust;

// The ucrtbase half of the math wave, and the reason it needs an `extern crate` rather than
// just a manifest entry: a dependency nothing references contributes NO objects to a
// staticlib, so the feature would compile cabi/math.rs and the 66 `#[no_mangle]` libm bodies
// would never reach the archive. That failure is silent -- the build is green, the gate is
// green, and the flip simply has not happened. (Confirmed the hard way: with the manifest
// edit alone, `libcrt_shared_cargo.a` defined 268 symbols and not one of them was `tan`.)
//
// ucrtbase.dll does not take its math from the vendored `sdk/lib/ucrt`; it links OUR
// `crtmath`, so these flip crtmath's C by lld first-match exactly as they already do in
// msvcrt.dll. Mirrors msvcrt-crt-cargo's `extern crate cnt_rust`.
extern crate cnt_rust;

// Same reason as cnt_rust above, and the same silent failure if omitted: a
// dependency nothing references contributes no objects to a staticlib, so the
// manifest entry alone would build green, gate green, and simply not flip.
extern crate ucrt_rust;

// Absorbed by --allow-multiple-definition against any other Rust staticlib in the module
// (mirrors msvcrt-crt-cargo and crt-dllmain-rust).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
