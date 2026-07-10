#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! usbccgp.sys — USB composite-device (generic parent) class driver. Faithful C→Rust cohab
//! (rustconv opus + c2rust anchor). 6 .c → 6 modules. Splits a composite USB device into
//! per-interface child PDOs. NOT boot-loaded in smoke (boots from CD/ramdisk, not USB).

pub mod types;
pub mod glue;
pub mod descriptor;
pub mod fdo;
pub mod function;
pub mod misc;
pub mod pdo;
pub mod usbccgp;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
