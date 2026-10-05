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

// Linked for their #[no_mangle] exports (see Cargo.toml): an unreferenced dependency would
// not reach the archive.
extern crate delayimp_rt;
extern crate pseh_rt;
extern crate setjmp_rt;

// Force the rlib's objects (the #[no_mangle] CRT bodies) into the archive.
extern crate msvcrt_crt_rust;

// The math wave: cnt-rust built with ONLY `cabi-math` (see Cargo.toml) —
// forces cabi/math.rs's 66 libm exports (sin/cos/exp/pow/…) into the
// archive, where whole-archive + lld first-match flips msvcrt.dll's math
// exports off crtmath's C/asm onto the kernel-proven Rust engine.
extern crate cnt_rust;

// The dup vs any other Rust staticlib in the module is absorbed by
// --allow-multiple-definition (mirrors crt-dllmain-rust).
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
