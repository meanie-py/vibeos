//! ex-rust — the Executive (Ex*/Exp*) + pool allocator, extracted from ntoskrnl-rust
//! (crate-split Phase 1b). The largest subsystem (~41K LOC). #[no_mangle] Ex*/Exp*/Nt*
//! symbols link via `extern crate ex_rust`. No per-crate barrier (dead-mechanism).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// dbgprint macros ex uses (dbg_print/dbg_print_ex/kdbp_print/rtl_string_cb_printf_a/
// swprintf/wstr), re-exported so `crate::<macro>!` sites resolve unchanged.
pub use dbgprint_rust::{dbg_print, dbg_print_ex, kdbp_print, rtl_string_cb_printf_a, swprintf, wstr};

// Boot-milestone UART traces, arm64 only (the only arch with a serial console
// this early). ntoskrnl/ex/init.c — now cohab-dead — carries 54 of these across
// ExpInitializeExecutive and Phase1InitializationDiscard; the Rust translation
// dropped every single one, so the arch that flakes in Phase 1 was the arch with
// no Phase 1 visibility at all.
//
// macro_rules!, never a fn. An `#[inline(always)] fn(_: &[u8]) {}` no-op does not
// just discard the message — LLVM then drops the string from .rdata too, so even
// a post-hoc `strings` check on the binary says the probe was never there. That is
// exactly how iomgr's io_trace lost 40 reports on aarch64.
#[allow(unused_macros)]
macro_rules! uart_trace {
    ($msg:expr) => {{
        #[cfg(target_arch = "aarch64")]
        {
            extern "C" {
                fn ARM64_UART_TRACE(msg: *const core::ffi::c_char);
            }
            #[allow(unused_unsafe)]
            unsafe {
                ARM64_UART_TRACE(concat!($msg, "\r\n\0").as_ptr() as *const core::ffi::c_char);
            }
        }
    }};
}

#[allow(unused_macros)]
macro_rules! uart_trace_hex {
    ($prefix:expr, $val:expr) => {{
        #[cfg(target_arch = "aarch64")]
        {
            extern "C" {
                fn ARM64_UART_TRACE_HEX(prefix: *const core::ffi::c_char, val: usize);
            }
            #[allow(unused_unsafe)]
            unsafe {
                ARM64_UART_TRACE_HEX(
                    concat!($prefix, "\0").as_ptr() as *const core::ffi::c_char,
                    ($val) as usize,
                );
            }
        }
    }};
}

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
