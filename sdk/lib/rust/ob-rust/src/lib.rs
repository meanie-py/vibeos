//! ob-rust — the Object Manager (Ob*/Obp*), extracted from ntoskrnl-rust
//! (crate-split Phase 1b). #[no_mangle] Ob*/Obp* symbols link via `extern crate
//! ob_rust`. No per-crate barrier (dead-mechanism; statics survive via callers).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

pub use dbgprint_rust::dbg_print;

pub mod devicemap;
pub mod obdir;
pub mod obhandle;
pub mod obinit;
pub mod oblife;
pub mod oblink;
pub mod obname;
pub mod obref;
pub mod obsdcach;
pub mod obsecure;
pub mod obwait;
pub mod inlines;
