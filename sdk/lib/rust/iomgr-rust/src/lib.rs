//! iomgr-rust — the I/O Manager (Io*/Iop*), extracted from ntoskrnl-rust into its
//! own crate so it compiles/caches independently of the kernel monolith
//! (crate-split Phase 1b — see docs/engineering/ntoskrnl-crate-split-plan.md).
//! Pure Io* logic on the ntkrnl-sys FFI substrate; its `#[no_mangle]` Io*/Iop*
//! symbols (344 exports incl. the 7 Io object-type singletons) are pulled into
//! libkernel_rust.a via `extern crate iomgr_rust` in the kernel-rust umbrella.
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// Debug-print macros hoisted to the dbgprint-rust leaf. Re-export at this crate's
// root so iomgr's `crate::dbg_print!(…)` / `crate::wstr!(…)` / `crate::swprintf!(…)`
// call sites (35 total: 23 dbg_print, 8 wstr, 4 swprintf) resolve unchanged
// (`$crate` inside each macro still resolves to dbgprint_rust). Same pattern as
// ntoskrnl-rust / cc-rust. These are the ONLY external macros iomgr depends on;
// its local w!/ARM64_UART_TRACE!/CONTAINING_RECORD!/FIELD_OFFSET! macros are
// defined inside the module files and travel with them.
pub use dbgprint_rust::{dbg_print, swprintf, wstr};

// Emit a fn twice: `extern "fastcall"` on i386, `extern "C"` elsewhere.
// FASTCALL only exists as a distinct ABI on x86; rustc treats "fastcall" as "C"
// on other arches today and makes it a HARD ERROR on newer toolchains
// (unsupported_calling_conventions). Same pattern as ex-rust's fastcall_export!.
macro_rules! fastcall_fn {
    ($(#[$attr:meta])* $vis:vis unsafe fn $name:ident($($params:tt)*) $(-> $ret:ty)? $body:block) => {
        $(#[$attr])*
        #[cfg(target_arch = "x86")]
        $vis unsafe extern "fastcall" fn $name($($params)*) $(-> $ret)? $body

        $(#[$attr])*
        #[cfg(not(target_arch = "x86"))]
        $vis unsafe extern "C" fn $name($($params)*) $(-> $ret)? $body
    };
}

pub mod global_barrier;

pub mod adapter;
pub mod arcname;
pub mod bootlog;
pub mod controller;
pub mod debug;
pub mod device;
pub mod deviface;
pub mod driver;
pub mod error;
pub mod file;
pub mod iocomp;
pub mod cancel_ex;
pub mod io_tables;
pub mod ioevent;
pub mod iofunc;
pub mod iomdl;
pub mod iomgr;
pub mod iorsrce;
pub mod iotimer;
pub mod iowork;
pub mod irp;
pub mod irq;
pub mod ramdisk;
pub mod rawfs;
pub mod remlock;
pub mod symlink;
pub mod util;
pub mod volume;

// Folded back 2026-05-09 from per-subsystem rlibs (pilot validated the
// captures(none) firewall is sufficient).
pub mod inlines;
