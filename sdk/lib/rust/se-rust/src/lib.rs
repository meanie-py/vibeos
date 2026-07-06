//! se-rust — the Security Reference Monitor (Se*/Sep*), extracted from ntoskrnl-rust
//! into its own crate so it compiles/caches independently of the kernel monolith
//! (crate-split Phase 1b — see docs/engineering/ntoskrnl-crate-split-plan.md).
//! Pure Se* logic on the ntkrnl-sys FFI substrate; its `#[no_mangle]` Se*/Sep* symbols
//! are pulled into libkernel_rust.a via `extern crate se_rust` in the kernel-rust
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

// Debug-print macro hoisted to the dbgprint-rust leaf. Re-export at this crate's
// root so se's `crate::dbg_print!(…)` call sites (59× in se/debug.rs) resolve
// unchanged (`$crate` inside the macro still resolves to dbgprint_rust). Same
// pattern as ntoskrnl-rust / cc-rust. se uses ONLY dbg_print! (verified: the only
// `crate::<macro>!` in se/ is `crate::dbg_print!`). The `dbgprint` module re-export
// mirrors cc-rust (harmless — the macro body is already self-contained via `$crate`).
pub use dbgprint_rust::dbgprint;
pub use dbgprint_rust::dbg_print;

pub mod global_barrier;

// The 20 module files from se/mod.rs (mod.rs itself becomes this lib.rs).
// se-local macros `w!` (srm.rs, token.rs) and `VALID_IMPERSONATION_LEVEL!`
// (client.rs) are defined INSIDE their files and travel with them — not redefined here.
pub mod access;
pub mod accesschk;
pub mod acl;
pub mod audit;
pub mod client;
pub mod debug;
pub mod objtype;
pub mod priv_;
pub mod sd;
pub mod semgr;
pub mod sid;
pub mod sqos;
pub mod srm;
pub mod subject;
pub mod token;
pub mod tokenadj;
pub mod tokencls;
pub mod tokenlif;
pub mod inlines;
