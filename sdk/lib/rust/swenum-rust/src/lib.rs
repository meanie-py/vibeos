#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// swenum.sys — Rust translation of the KS software bus enumerator driver
// (drivers/ksfilter/swenum/swenum.c). crate-type=staticlib →
// libswenum_rust.a, linked into the cmake-built swenum.sys with swenum.c
// commented out (kept on disk per the never-delete rule). Standalone
// kernel-mode driver on the ntkrnl-sys substrate; the Ks* bus-enum entry
// points are ks.sys imports declared in the swenum module. Built for all 3
// arches. DeepSeek+c2rust translation (Batch 2 driver residue).
//
// One C file → one module (rustconv per-file convention).
pub mod swenum;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
