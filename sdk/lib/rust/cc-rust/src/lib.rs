//! cc-rust — the Cache Controller (Cc*), extracted from ntoskrnl-rust into its
//! own crate so it compiles/caches independently of the kernel monolith
//! (crate-split Phase 1b — see docs/engineering/ntoskrnl-crate-split-plan.md).
//! Pure Cc* logic on the ntkrnl-sys FFI substrate; its `#[no_mangle]` Cc* symbols
//! are pulled into libkernel_rust.a via `extern crate cc_rust` in the kernel-rust
//! umbrella.
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
#![allow(static_mut_refs)]

// Debug-print macros hoisted to the dbgprint-rust leaf. Re-export at this crate's
// root so cc's `crate::dbg_print_ex!(…)` call sites resolve unchanged (`$crate`
// inside the macro still resolves to dbgprint_rust). Same pattern as ntoskrnl-rust.
pub use dbgprint_rust::dbgprint;
pub use dbgprint_rust::{dbg_print, dbg_print_ex};

pub mod global_barrier;

pub mod cacheman;
pub mod copy;
pub mod fs;
pub mod lazywrite;
pub mod mdl;
pub mod pin;
pub mod view;
