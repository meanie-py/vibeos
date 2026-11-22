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
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

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
pub mod sectorsize;
pub mod stackovf;
pub mod tunnel;
pub mod unc;
