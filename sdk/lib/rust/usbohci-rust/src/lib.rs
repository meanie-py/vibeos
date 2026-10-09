#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]
#![allow(dangerous_implicit_autorefs)]

//! usbohci.sys — USB 1.1 OHCI host-controller miniport. Faithful C→Rust cohab (rustconv
//! opus + c2rust-anchored). 2 .c (roothub/usbohci) -> 2 modules. Registers a
//! USBPORT_REGISTRATION_PACKET with the usbport framework. guid.c stays as a C data TU.
//! NOT boot-loaded in smoke (boots from CD/ramdisk, not USB). Bar: link-clean x3.
// Fold-proof DPRINT/TRACE shims — see drvtrace-shim/src/lib.rs (§9.99).
#[macro_use]
extern crate drvtrace_shim;

pub mod types;
pub mod glue;
pub mod roothub;
pub mod usbohci;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
