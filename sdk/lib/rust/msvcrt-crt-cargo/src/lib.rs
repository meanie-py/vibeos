//! Staticlib umbrella for the msvcrt.dll Rust CRT-body cohab.
//!
//! Carries no logic of its own: `extern crate msvcrt_crt_rust` forces that
//! rlib's objects (and therefore its `#[no_mangle]` CRT symbols) into this
//! archive, and the no_std runtime bits below make it a linkable final
//! artifact. cmake links the resulting `libmsvcrt_crt_cargo.a` into the
//! `msvcrt` MODULE under `--whole-archive`.
//!
//! See ../msvcrt-crt-rust for the wrappers themselves and the lane plan
//! (docs/engineering/active-lanes/msvcrt-from-pieces-plan.md) for the flip
//! mechanism.
#![no_std]

// Force the rlib's objects (the #[no_mangle] CRT bodies) into the archive.
extern crate msvcrt_crt_rust;

// The dup vs any other Rust staticlib in the module is absorbed by
// --allow-multiple-definition (mirrors crt-dllmain-rust).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
