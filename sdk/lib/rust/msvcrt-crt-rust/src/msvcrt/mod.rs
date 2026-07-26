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
// yet wired in: it needs an msvcrt-sys island carrying FILE/ioinfo/_O_*/_IO*, which the
// current island — scoped to thread_data_t + threadlocinfo — does not have. Enabling it
// today is 1,363 cargo errors, i.e. a broken build for every lane.
//
// This is the netcfgx precedent (translated, committed, deliberately not a member until
// it compiles), NOT the anti-pattern AGENTS.md forbids: nothing here is being hidden to
// dodge a regression, and no C is being kept alive in preference to working Rust. The C
// file.c is still the only implementation and still correct.
//
// It must go in as ONE flip, never partially: file.c's entry points share one fd table
// and one FILE ring, so a C `fopen` beside a Rust `fclose` is split state.
// pub mod file;
pub mod mbcs;
