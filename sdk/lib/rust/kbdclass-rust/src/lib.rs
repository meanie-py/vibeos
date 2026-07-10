#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// kbdclass.sys — Rust translation of the keyboard class driver
// (drivers/input/kbdclass/{kbdclass,misc}.c). crate-type=staticlib →
// libkbdclass_rust.a, linked into the cmake-built kbdclass.sys with the C
// sources commented out (kept on disk per never-delete rule). Standalone
// kernel-mode driver depending on ntkrnl-sys; mirrors the arm64input-rust
// cohabitation. Built for all 3 arches.

pub mod kbdclass_h;
pub mod kbdclass;
pub mod misc;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
