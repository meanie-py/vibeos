//! po-rust — the Power Manager (Po*/Pop*), extracted from ntoskrnl-rust into its own
//! crate (crate-split Phase 1b). Its #[no_mangle] Po*/Pop* symbols link into
//! ntoskrnl.exe via `extern crate po_rust` in the kernel-rust umbrella.
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
pub mod events;
pub mod poshtdwn;
pub mod povolume;
pub mod power;
pub mod setting;
