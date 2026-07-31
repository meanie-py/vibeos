//! Whole-file translations of `dll/win32/msvcrt/*.c`, one module per C file.
//!
//! Distinct from the hand-assembled families in `lib.rs`: those delegate to
//! `cnt_rust::core_rs` and were written family-by-family, while these are
//! rustconv translations of a WHOLE C file, kept 1:1 with their source so the
//! next re-translation lands as a readable diff.
//!
//! The cohab rule is unchanged — every `#[no_mangle]` here wins over the C copy
//! in `msvcrt_static` by lld first-match. A function this module does NOT define
//! simply stays C, which is why a partial file is a safe state.

// file.c is TRANSLATED (189/189 exports, tools/rustconv/.responses/…-file.c) but NOT
// yet wired in. The msvcrt-sys island has since been extended to the stream layer and
// crate-repair has run across all three arches, taking this from 1,363 cargo errors to
// ~484 — still a broken build for every lane. The remaining bulk is the C-variadic
// printf/scanf shells, which need the wave-14c C trampoline (pinned stable: no
// `#![feature(c_variadic)]`). file.rs carries the full ORPHAN-BY-DESIGN banner.
//
// This is the netcfgx precedent (translated, committed, deliberately not a member until
// it compiles), NOT the anti-pattern AGENTS.md forbids: nothing here is being hidden to
// dodge a regression, and no C is being kept alive in preference to working Rust. The C
// file.c is still the only implementation and still correct.
//
// It must go in as ONE flip, never partially: file.c's entry points share one fd table
// and one FILE ring, so a C `fopen` beside a Rust `fclose` is split state.
pub mod file;
pub mod mbcs;
