//! ps-rust — the Process/Thread Manager (Ps*/Psp*), extracted from ntoskrnl-rust
//! (crate-split Phase 1b). Its #[no_mangle] Ps*/Psp* symbols link into ntoskrnl.exe
//! via `extern crate ps_rust` in the kernel-rust umbrella. No per-crate barrier
//! (monolith master barrier is dead + extracted barriers dead-stripped; statics
//! survive via real callers).
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// dbg_print! macro + the dbgprint runtime module (IntoSlot / rust_dbg_print_cstr),
// re-exported so ps's `crate::dbg_print!` / `crate::dbgprint::…` sites resolve.
pub use dbgprint_rust::dbgprint;
pub use dbgprint_rust::dbg_print;

#[cfg(target_arch = "x86")]
pub mod i386;
#[cfg(target_arch = "x86_64")]
pub mod amd64;

pub mod apphelp;
pub mod debug;
pub mod info_class;
pub mod job;
pub mod kill;
pub mod process;
pub mod ps_globals;
pub mod psmgr;
pub mod psnotify;
pub mod query;
pub mod quota;
pub mod security;
pub mod state;
pub mod thread;
pub mod trace;
pub mod win32;
pub mod inlines;
