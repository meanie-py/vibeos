#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::all)]

//! hidclass.sys — faithful C→Rust cohab translation of drivers/hid/hidclass/*.c.

pub mod types;
pub mod glue;
pub mod guid;
pub mod fdo;
pub mod pdo;
pub mod hidclass;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
