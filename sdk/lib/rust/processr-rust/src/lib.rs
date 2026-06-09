#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// processr.sys — Rust translation of the processor/CPU driver
// (drivers/processor/processr/{processr,misc,pnp}.c). crate-type=staticlib →
// libprocessr_rust.a, linked into the cmake-built processr.sys with the .c files
// commented out (kept on disk per the never-delete rule). DeepSeek+c2rust
// translation (Batch 3 driver residue).
//
// One C file → one module (rustconv per-file convention). `shared` holds the
// driver-private DEVICE_EXTENSION (processr.h) used across all three.
pub mod shared;
pub mod misc;
pub mod pnp;
pub mod processr;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
