#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, unused_variables, unused_parens, unused_mut)]
#![allow(unused_assignments, improper_ctypes, improper_ctypes_definitions)]
#![allow(clippy::all)]

//! usbhub.sys — USB hub class driver. Faithful C→Rust cohab (rustconv opus; usbhub.c via
//! pure-LLM as c2rust couldn't anchor it, the rest c2rust-anchored). 5 .c -> 5 modules.
//! Enumerates/powers/resets downstream ports. NOT boot-loaded in smoke (boots from CD/ramdisk).
// ASSERT is debug_assert! (ntkrnl_sys::dbgassert): checked in a debug build, never
// evaluated in release -- the kernel's convention. It was an empty fn here, which
// never checked yet still evaluated its argument on every call.
macro_rules! ASSERT { ($($t:tt)*) => { ntkrnl_sys::nt_assert!($($t)*) }; }

// Fold-proof DPRINT/TRACE shims — see drvtrace-shim/src/lib.rs (§9.99).
#[macro_use]
extern crate drvtrace_shim;

pub mod types;
pub mod glue;
pub mod debug;
pub mod ioctl;
pub mod pnp;
pub mod power;
pub mod usbhub;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
