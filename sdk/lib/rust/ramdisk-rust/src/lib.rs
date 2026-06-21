#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! ramdisk.sys — faithful C→Rust cohab (rustconv opus mechanical).
//! EASY leaf: pure WDM class driver — no miniport / no ISR / no DPC / no HW register I/O.

pub mod types;
pub mod glue;
pub mod ramdisk;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// ---------------------------------------------------------------------------
// TRACE_ / WARN_ / ERR_ — ReactOS debug.h macros → __noop in retail builds
// (no real symbol). cdecl stubs so the rlib is self-contained.
// ---------------------------------------------------------------------------
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
