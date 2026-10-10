//! config-rust — the Configuration Manager (registry / CM), extracted from
//! ntoskrnl-rust into its own crate so it compiles/caches independently of the
//! kernel monolith (crate-split Phase 1b). Its `#[no_mangle]` Cm*/Cmp* symbols are
//! pulled into libkernel_rust.a via `extern crate config_rust` in the kernel-rust
//! umbrella.
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

/// ASSERT macro — maps to ntkrnl_sys::ASSERT!(no-op in release). A crate-local convenience
/// (ntoskrnl-rust defines its own identical copy; it isn't exported). Defined before
/// the `pub mod`s so submodules see it via textual macro scope.
#[allow(unused_macros)]
macro_rules! ASSERT {
    ($e:expr) => {
        ntkrnl_sys::ASSERT!($e)
    };
}

/// ke.h `ARM64_UART_TRACE(s)`: `do { } while(0)` / `((void)0)` unless
/// `ARM64_UART_TRACE_ENABLE`, which is commented out -- the string is never evaluated.
/// (cminit.rs instead calls the kernel's exported UART writer by `extern`, on purpose.)
#[allow(unused_macros)]
macro_rules! ARM64_UART_TRACE {
    ($($t:tt)*) => {
        ()
    };
}

// Debug-print / format macros hoisted to the dbgprint-rust leaf. Re-export at this
// crate's root so config's `crate::dbg_print!(…)` / `crate::wstr!(…)` /
// `crate::rtl_string_cb_printf_a!(…)` / `crate::rtl_string_cb_printf_w!(…)` call sites
// resolve unchanged (`$crate` inside each #[macro_export] macro still resolves to
// dbgprint_rust). config has no direct `crate::dbgprint::…` sites.
pub use dbgprint_rust::{dbg_print, rtl_string_cb_printf_a, rtl_string_cb_printf_w, wstr};

pub mod global_barrier;

// HHIVE::HiveFlags bits, transcribed once from sdk/lib/cmlib/hivedata.h. See the
// module header: four of the eight open-coded copies these replace were wrong.
pub mod hiveflags;

// Module tree transcribed from ntoskrnl-rust/src/config/mod.rs (cmboot was already
// hoisted to the shared cmboot-rust crate; not included here).
pub mod cmalloc;
pub mod cmapi;
pub mod cmconfig;
pub mod cmcontrl;
pub mod cmdata;
pub mod cmdelay;
pub mod cmhook;
pub mod cmhvlist;
pub mod cminit;
pub mod cmkcbncb;
pub mod cmlazy;
pub mod cmmapvw;
pub mod cmnotify;
pub mod cmparse;
pub mod cmquota;
pub mod cmse;
mod cmsecurity_persist;
pub mod cmsecach;
pub mod cmsysini;
pub mod cmvalche;
pub mod cmwow64;
pub mod cmwraprs;
pub mod ntapi;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod i386;

pub mod inlines;
