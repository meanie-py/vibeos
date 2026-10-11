#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! fltmgr.sys — Filesystem Filter Manager. Faithful C→Rust cohab
//! (rustconv opus + c2rust anchor). 10 .c → 10 modules.

pub mod types;
pub mod glue;
pub mod Context;
pub mod Dispatch;
pub mod Filter;
pub mod Interface;
pub mod flt_lib;
pub mod Messaging;
pub mod Misc;
pub mod Object;
pub mod Registry;
pub mod Volume;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
