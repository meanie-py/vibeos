#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! serial.sys — faithful C→Rust cohab (rustconv opus + c2rust anchor).
// Fold-proof DPRINT/TRACE shims — see drvtrace-shim/src/lib.rs (§9.99).
#[macro_use]
extern crate drvtrace_shim;

pub mod types;
pub mod glue;
pub mod circularbuffer;
pub mod cleanup;
pub mod close;
pub mod create;
pub mod devctrl;
pub mod info;
pub mod legacy;
pub mod misc;
pub mod pnp;
pub mod power;
pub mod rw;
pub mod serial;

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentPrcb() -> ntkrnl_sys::ddk::PKPRCB {
    ntkrnl_sys::inline::ke::KeGetCurrentPrcb()
}

// ---------------------------------------------------------------------------
// TRACE_ / WARN_ / ERR_ — ReactOS debug.h macros → __noop in retail builds
// (no real symbol).  Provide #[no_mangle] cdecl stubs so the rlib is
// self-contained.  cdecl caller-cleans-up: a 2-arg body is safe even when
// callers supply extra variadic args — the caller pops them off the stack.
// ---------------------------------------------------------------------------
// TRACE_ def relocated to drvtrace-shim (§9.99 fold-proofing)
#[no_mangle]
pub unsafe extern "C" fn WARN_(_component: u32, _fmt: *const core::ffi::c_char) {}
#[no_mangle]
pub unsafe extern "C" fn ERR_(_component: u32, _fmt: *const core::ffi::c_char) {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
