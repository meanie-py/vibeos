//! kd-rust — kernel-debug I/O surface (Kd*), extracted from ntoskrnl-rust
//! (crate-split Phase 1b). #[no_mangle] Kd* symbols link via `extern crate kd_rust`.
//! No per-crate barrier: the monolith master barrier is unwired/dead and extracted
//! barriers are dead-stripped; kd statics survive via their (C/Rust) callers.
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

pub use dbgprint_rust::dbg_print;

pub mod kdio;
pub mod kdmain;
pub mod kdprompt;
pub mod kdps2kbd;
pub mod kdserial;
pub mod kdterminal;
