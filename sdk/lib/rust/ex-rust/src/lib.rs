//! ex-rust — the Executive (Ex*/Exp*) + pool allocator, extracted from ntoskrnl-rust
//! (crate-split Phase 1b). The largest subsystem (~41K LOC). #[no_mangle] Ex*/Exp*/Nt*
//! symbols link via `extern crate ex_rust`. No per-crate barrier (dead-mechanism).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// dbgprint macros ex uses (dbg_print/dbg_print_ex/kdbp_print/rtl_string_cb_printf_a/
// swprintf/wstr), re-exported so `crate::<macro>!` sites resolve unchanged.
pub use dbgprint_rust::{dbg_print, dbg_print_ex, kdbp_print, rtl_string_cb_printf_a, swprintf, wstr};

// abi_cast! — cast an extern fn to the target callback ABI (crate-local copy).
#[allow(unused_macros)]
macro_rules! abi_cast {
    ($func:expr) => {
        unsafe { core::mem::transmute($func as usize) }
    };
}

pub mod atom;
pub mod callback;
pub mod dbgctrl;
pub mod efi;
pub mod event;
pub mod evtpair;
pub mod exintrin;
pub mod fmutex;
pub mod handle;
pub mod harderr;
pub mod hdlsterm;
pub mod init;
pub mod interlocked;
pub mod keyedevt;
pub mod locale;
pub mod lookas;
pub mod mutant;
pub mod profile;
pub mod pushlock;
pub mod resource;
pub mod rundown;
pub mod sem;
pub mod shutdown;
pub mod sysinfo;
pub mod time;
pub mod timer;
pub mod uuid;
pub mod win32k;
pub mod work;
pub mod xipdisp;
pub mod zone;
pub mod pool;
