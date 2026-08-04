//! aclui-rust prelude — the single import surface the translated modules use.
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

// ================= enum member aliases ========================================================
// bindgen prefixes a C enum's members with the enum's tag, so the bare spelling the C uses is
// absent from the island. Alias, never re-declare: a literal here could drift from the island's
// value silently. These are the uxtheme part/state ids checklist.c draws with (SUPPORT_UXTHEME).
// None is a `case` label here, so this is convenience rather than the §9.112 correctness trap —
// but it is the same missing-bare-name cause, so it is checked the same way.
pub const BP_CHECKBOX: BUTTONPARTS = BUTTONPARTS_BP_CHECKBOX;
pub const CBS_UNCHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDNORMAL;
pub const CBS_UNCHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDHOT;
pub const CBS_UNCHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDPRESSED;
pub const CBS_UNCHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_UNCHECKEDDISABLED;
pub const CBS_CHECKEDNORMAL: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDNORMAL;
pub const CBS_CHECKEDHOT: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDHOT;
pub const CBS_CHECKEDPRESSED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDPRESSED;
pub const CBS_CHECKEDDISABLED: CHECKBOXSTATES = CHECKBOXSTATES_CBS_CHECKEDDISABLED;
