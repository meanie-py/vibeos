//! kdbg-rust — the integrated kernel debugger (KDBG), extracted from ntoskrnl-rust
//! into its own crate so it compiles/caches independently of the kernel monolith
//! (crate-split Phase 1b — see docs/engineering/ntoskrnl-crate-split-plan.md).
//! Its `#[no_mangle]` Kdb*/Kdp* symbols are pulled into libkernel_rust.a via
//! `extern crate kdbg_rust` in the kernel-rust umbrella.
//!
//! KDBG IS x86-ONLY (disabled on ARM64). The crate-level `#![cfg(...)]` below empties
//! the ENTIRE crate on arm64: it compiles to an empty rlib, and kernel-rust's
//! `extern crate kdbg_rust` is itself cfg-gated out, so arm64 never references any
//! kdbg symbol. This mirrors ntoskrnl-rust/src/lib.rs, which gated `pub mod kdbg;`
//! the same way (`#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]`).
//! The crate stays an unconditional workspace member — on arm64 `cargo build` just
//! produces an empty rlib (which compiles cleanly), so no per-arch member juggling.
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]

// Debug-print macros + the `dbgprint` runtime module were hoisted to the
// dbgprint-rust leaf crate (crate-split Phase 1b). Re-export them at this crate's
// root so kdbg's `crate::kdbp_print!(…)` / `crate::kdb_printf!(…)` /
// `crate::rtl_string_cb_printf_a!(…)` macro sites (256 total) and the 5
// `crate::dbgprint::rust_kdbp_print_cstr(…)` calls resolve UNCHANGED — `$crate`
// inside each #[macro_export] macro still resolves to dbgprint_rust, so their
// `$crate::dbgprint::…` bodies stay valid. Same pattern as ntoskrnl-rust/cc-rust.
// (Only these three macros + the dbgprint module are used by kdbg; the wider
// dbg_print/kdp_dprintf/swprintf set is not.)
pub use dbgprint_rust::dbgprint;
pub use dbgprint_rust::{kdb_printf, kdbp_print, rtl_string_cb_printf_a};

pub mod global_barrier;

// Module tree — transcribed verbatim from ntoskrnl-rust/src/kdbg/mod.rs, including
// its internal x86 cfg gates on i386_dis + kdb_help (redundant under the crate-level
// cfg above, but preserved unchanged so the file set is byte-identical after the move).
// NOTE: the locally-defined macros `insn!` (i386_dis.rs), `cmd!` (tables.rs), `w!` and
// `RTL_NUMBER_OF!` (kdb_cli.rs) live INSIDE these files and move with them — do NOT
// redefine them here.
pub mod types;

pub mod kdb;
pub mod kdb_cli;
pub mod tables;
pub mod kdb_cmdhist;
pub mod kdb_expr;
pub mod kdb_print;
pub mod kdb_symbols;
pub mod kdbg;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod i386_dis;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod kdb_help;
