#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// mouclass.sys — Rust translation of the mouse class driver
// (drivers/input/mouclass/{mouclass,misc}.c). crate-type=staticlib →
// libmouclass_rust.a, linked into the cmake-built mouclass.sys with the C
// sources commented out (kept on disk per never-delete rule). Standalone
// kernel-mode driver depending on ntkrnl-sys; mirrors arm64input-rust
// cohabitation. Built for all 3 arches.

pub mod mouclass_h;
pub mod mouclass;
pub mod misc;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
