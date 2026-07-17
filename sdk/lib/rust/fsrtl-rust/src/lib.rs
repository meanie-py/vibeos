//! fsrtl-rust — the File System Run-Time Library (FsRtl*), extracted from
//! ntoskrnl-rust into its own crate so it compiles/caches independently of the
//! kernel monolith (crate-split — see docs/engineering/ntoskrnl-crate-split-plan.md).
//! Pure FsRtl logic on the ntkrnl-sys FFI substrate; its `#[no_mangle]` FsRtl*
//! symbols are pulled into libkernel_rust.a via `extern crate fsrtl_rust` in the
//! kernel-rust umbrella.
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

/// Cast an `extern "system"` fn to a target callback ABI (i386 stdcall vs
/// amd64/arm64 C — identical machine code, distinct Rust types). Copied from
/// ntoskrnl-rust's crate-root macro: 5 fsrtl files invoke it, and crate-root
/// macros are invisible to a `crate::`/`use` extraction audit (hidden-coupling
/// class #2). TODO: hoist the shared kernel macros (abi_cast, dbg_print, …) into
/// ntkrnl-sys or an ntos-macros crate so extracted crates share one definition —
/// see docs/engineering/ntoskrnl-crate-split-plan.md.
#[allow(unused_macros)]
macro_rules! abi_cast {
    ($func:expr) => {
        unsafe { core::mem::transmute($func as usize) }
    };
}

pub mod global_barrier;

pub mod dbcsname;
pub mod fastio;
pub mod faulttol;
pub mod filelock;
pub mod filter;
pub mod filtrctx;
pub mod fsfilter;
pub mod fsrtlpc;
pub mod largemcb;
pub mod mcb;
pub mod name;
pub mod notify;
pub mod oplock;
pub mod pnp;
pub mod stackovf;
pub mod tunnel;
pub mod unc;
