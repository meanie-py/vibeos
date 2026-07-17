//! dbgk-rust — the kernel Debugging subsystem (Dbgk*: debug objects, ports,
//! process/thread debug notifications), extracted from ntoskrnl-rust (crate-split
//! — see docs/engineering/ntoskrnl-crate-split-plan.md). #[no_mangle] Dbgk*
//! symbols linked via `extern crate dbgk_rust` in the kernel-rust umbrella.
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

pub mod dbgkobj;
pub mod dbgkutil;
