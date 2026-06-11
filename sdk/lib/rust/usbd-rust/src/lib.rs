#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

// usbd.sys — Rust translation of the USBD library driver (drivers/usb/usbd/
// usbd.c). crate-type=staticlib → libusbd_rust.a, linked into the cmake-built
// usbd.sys with usbd.c commented out (kept on disk per the never-delete rule).
// Export driver: the usbd.spec USBD_* exports come from the Rust #[no_mangle]
// symbols. Standalone kernel-mode driver on the ntkrnl-sys substrate. Built for
// all 3 arches. DeepSeek+c2rust translation (Batch 2 driver residue).
//
// One C file → one module (rustconv per-file convention).
pub mod usbd;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
