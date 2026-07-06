//! lpc-rust — the LPC (Local Procedure Call) subsystem, extracted from
//! ntoskrnl-rust into its own crate so it compiles/caches independently of the
//! 450K-LOC kernel monolith (crate-split PoC — see
//! docs/engineering/ntoskrnl-crate-split-plan.md). Pure NT LPC logic on the
//! ntkrnl-sys FFI substrate; its `#[no_mangle]` Nt*/Lpcp* symbols are pulled
//! into libkernel_rust.a via `extern crate lpc_rust` in the kernel-rust umbrella.
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

pub mod global_barrier;

pub mod close;
pub mod complete;
pub mod connect;
pub mod create;
pub mod listen;
pub mod port;
pub mod reply;
pub mod send;
