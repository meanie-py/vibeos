//! smss-csrss-rust — Rust translation of the SMSS→CSRSS user-mode
//! spine (smss.exe, csrss.exe, csrsrv.dll, basesrv.dll, …).
//!
//! Per-binary cohabitation: each translated `.c` is commented out in
//! its CMakeLists and the `#[no_mangle]` symbol here takes over. FFI
//! surface is `ntdll-sys` (the user-mode Native API + SM/CSR/BASE
//! protocol). See docs/engineering/smss-csrss-translation-audit.md.
//!
//! NOT a standalone binary — an `rlib` linked into the cmake-built
//! native PEs (smss.exe / csrss.exe / csrsrv.dll …).

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code)]

// csrss.exe spike. ⚠ The raw spike output does NOT compile yet — it
// flushed 3 systematic Phase-1 pipeline gaps (wrong FFI crate
// `ntkrnl_sys` vs `ntdll_sys::nt`; FORCEINLINE NtCurrent* needing a
// hand-written ntdll-sys inline module; i386 `_main` decoration). See
// csrss.rs header. Module is left declared (failure visible, per
// CLAUDE.md) but this crate is intentionally NOT a workspace member
// yet — it's a pre-integration scaffold; adding it before the pipeline
// fixes would break the whole-workspace `cargo build` for unrelated
// work. Wire it up when the pipeline fixes land + cohabitation begins.
pub mod csrss;
