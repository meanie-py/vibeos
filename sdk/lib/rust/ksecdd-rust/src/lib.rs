#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! ksecdd.sys — Kernel Security Support Provider. Faithful C→Rust cohab
//! (rustconv opus + c2rust anchor). 5 .c → 5 modules.

pub mod types;
pub mod glue;
pub mod ksecdd;
pub mod dispatch;
pub mod crypt;
pub mod random;
pub mod stubs;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

#[no_mangle]
pub unsafe extern "C" fn TRACE_(_component: u32, _fmt: *const core::ffi::c_char) {}
#[no_mangle]
pub unsafe extern "C" fn WARN_(_component: u32, _fmt: *const core::ffi::c_char) {}
#[no_mangle]
pub unsafe extern "C" fn ERR_(_component: u32, _fmt: *const core::ffi::c_char) {}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! { loop {} }
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
