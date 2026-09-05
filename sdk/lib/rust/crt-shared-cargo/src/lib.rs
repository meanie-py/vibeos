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

// Absorbed by --allow-multiple-definition against any other Rust staticlib in the module
// (mirrors msvcrt-crt-cargo and crt-dllmain-rust).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
