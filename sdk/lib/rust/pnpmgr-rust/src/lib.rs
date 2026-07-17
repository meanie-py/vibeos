//! pnpmgr-rust — the Plug-and-Play manager (Pnp*/Iop*/Pi* PnP device tree),
//! extracted from ntoskrnl-rust into its own crate so it compiles/caches
//! independently of the kernel monolith (crate-split Phase 1b — see
//! docs/engineering/ntoskrnl-crate-split-plan.md). Pure PnP logic on the
//! ntkrnl-sys FFI substrate; its `#[no_mangle]` Pnp*/Iop*/Pi* symbols are pulled
//! into libkernel_rust.a via `extern crate pnpmgr_rust` in the kernel-rust
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

// Debug-print / format macros hoisted to the dbgprint-rust leaf. Re-export at this
// crate's root so pnpmgr's `crate::dbg_print!(…)` / `crate::wstr!(…)` /
// `crate::rtl_string_cb_printf_w!(…)` / `crate::swprintf!(…)` call sites resolve
// unchanged (`$crate` inside each macro still resolves to dbgprint_rust, so their
// `$crate::dbgprint::…` bodies stay valid — no `dbgprint` module re-export needed,
// pnpmgr has no direct `crate::dbgprint::…` sites). Same pattern as cc-rust.
pub use dbgprint_rust::{dbg_print, rtl_string_cb_printf_w, swprintf, wstr};

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

// Module tree transcribed from ntoskrnl-rust/src/pnpmgr/mod.rs (same order). The
// nested `arb` bus/irq/port/dma/mem arbiters keep their subdir (arb/mod.rs moved to
// src/arb/mod.rs).
pub mod pnputil;
pub mod pnpirp;
pub mod pnpnotify;
pub mod pnpdma;
pub mod pnpmap;
pub mod devnode;
pub mod plugplay;
pub mod pnpreport;
pub mod pnproot;
pub mod pnpinit;
pub mod pnpmgr;
pub mod pnpres;
pub mod devaction;
pub mod arb;
