#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// pl011.sys — Rust translation of the ARM64 PL011 UART driver
// (drivers/serial/pl011/pl011.c). crate-type=staticlib → libpl011_rust.a, linked
// into the cmake-built pl011.sys with pl011.c commented out (kept on disk per the
// never-delete rule). DeepSeek+c2rust translation (Batch 3 driver residue).
//
// One C file → one module (rustconv per-file convention).
pub mod pl011;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
