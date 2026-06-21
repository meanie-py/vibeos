#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! kbdhid.sys — HID keyboard client driver, faithful C→Rust cohab translation
//! of drivers/hid/kbdhid/kbdhid.c. Links as libkbdhid_rust.a; HidP_* resolve from hidparse.sys.

pub mod types;
pub mod glue;
pub mod read;
pub mod ioctl;
pub mod pnp;
pub mod power;
pub mod entry;

// arm64: ntkrnl_sys::inline::ke's KeGetCurrentIrql needs KeGetCurrentPrcb, not an ntoskrnl
// export on aarch64 — forward to the register-reading inline (same shim as npfs-rust).
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
