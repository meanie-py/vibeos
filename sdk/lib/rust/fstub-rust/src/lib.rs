//! fstub-rust — the I/O disk-stub / partition layer (Io* partition API, Fstub*,
//! xHal* bus/mirror/pnp stubs, HalExamineMBR, Halp*, and the HalDispatchTable /
//! HalPrivateDispatchTable globals), extracted from ntoskrnl-rust into its own
//! crate so it compiles/caches independently of the kernel monolith (crate-split
//! Phase 1b — see docs/engineering/ntoskrnl-crate-split-plan.md). Pure fstub logic
//! on the ntkrnl-sys FFI substrate; its `#[no_mangle]` Io*/Fstub*/xHal*/Halp*
//! symbols are pulled into libkernel_rust.a via `extern crate fstub_rust` in the
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
#![allow(static_mut_refs)]

// Debug-print macro hoisted to the dbgprint-rust leaf. Re-export at this crate's
// root so fstub's `crate::swprintf!(…)` call sites (17× in disksup.rs) resolve
// unchanged (`$crate` inside the macro still resolves to dbgprint_rust). Same
// pattern as cc-rust / ntoskrnl-rust. fstub uses ONLY swprintf! (verified: no
// abi_cast!, no ASSERT!, no other crate-root macros).
pub use dbgprint_rust::swprintf;

pub mod global_barrier;

pub mod disksup;
pub mod fstubex;
pub mod halstub;
pub mod translate;
