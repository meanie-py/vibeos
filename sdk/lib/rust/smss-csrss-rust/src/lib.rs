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

// csrss.exe `_main` — clean rustconv translation after the P1a
// pipeline fixes (user-mode FFI routing + prompt addendum + ntdll-sys
// NtCurrent* inlines). Compiles ×3; cohabitation cmake flip (comment
// csrss.c, link this rlib, build csrss.exe, smoke-gate) is the next
// step and needs the main checkout's build tree.
pub mod csrss;
