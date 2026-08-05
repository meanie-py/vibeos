//! desk-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/DPRINT1 — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier).
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! DPRINT1 { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, DPRINT1};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= constants bindgen cannot fold ==============================================
// commctrl.h:137 `#define NM_CUSTOMDRAW (NM_FIRST-12)` where NM_FIRST is `(0U- 0U)`. bindgen
// emits NM_FIRST but NOT this one — a macro whose body is an EXPRESSION does not always fold —
// so the name was absent from the island and `NM_CUSTOMDRAW =>` in background.rs was a fresh
// BINDING that matched every notification and made all seven arms below it dead code (§9.112,
// the single largest and least visible translation bug class). `profile_derive` had already
// flagged this exact risk as SWITCH_CONST_EXPRESSION: "bindgen may not fold it — verify it
// lands in the island". That finding is a TODO, not a note.
//
// Built from the header's own macro rather than a pasted 0xFFFFFFF4, so it stays checkable:
// the C arithmetic is UNSIGNED (`0U-0U`, then `-12`), which wraps.
pub const NM_CUSTOMDRAW: UINT = NM_FIRST.wrapping_sub(12);
