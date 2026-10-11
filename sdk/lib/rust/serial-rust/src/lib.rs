#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! serial.sys — faithful C→Rust cohab (rustconv opus + c2rust anchor).
// Fold-proof DPRINT/TRACE shims — see drvtrace-shim/src/lib.rs (§9.99).
#[macro_use]
extern crate drvtrace_shim;

// ERR_ / WARN_ / TRACE_ (sdk/include/reactos/debug.h). This build is DBG=1, where
// each is `DbgPrintEx(DPFLTR_<ch>_ID, <level>, "(%s:%d) " fmt, __RELFILE__,
// __LINE__, ...)` -- a LIVE print, filtered by the kernel's debug masks (by default
// only ERROR level reaches the port, so ERR_ prints and WARN_/TRACE_ stay quiet
// unless a mask is raised). They were empty functions here: ERR_'s twelve
// "not implemented" reports were dropped, and every argument was still evaluated.
// The "(%s:%d) " prefix is a second call because the format is a runtime pointer.
macro_rules! serial_dbgprintex {
    ($lvl:expr, $ch:expr, $fmt:expr $(, $a:expr)* $(,)?) => {{
        $crate::glue::DbgPrintEx(
            $ch,
            $lvl,
            concat!("(%s:%d) \0").as_ptr() as *const i8,
            concat!(file!(), "\0").as_ptr() as *const i8,
            line!(),
        );
        $crate::glue::DbgPrintEx($ch, $lvl, $fmt $(, $a)*);
    }};
}
macro_rules! ERR_ { ($($t:tt)*) => { serial_dbgprintex!(0, $($t)*) }; }
macro_rules! WARN_ { ($($t:tt)*) => { serial_dbgprintex!(1, $($t)*) }; }
macro_rules! TRACE_ { ($($t:tt)*) => { serial_dbgprintex!(2, $($t)*) }; }

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


#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
