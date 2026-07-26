//! wldap32-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME). Kept as no-op MACROS, never
// out-of-line fns: an out-of-line no-op is an optimization barrier that uncloaks latent
// ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => {{}}; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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

// ============ wldap32.h's inline helpers =====================================
// In C these are `static inline` in wldap32.h, so every TU inlined its own copy and
// calling one needed no declaration. In Rust they are one module; re-exported here so
// the 20 translated TUs keep resolving them by bare name, exactly as the C did.
// NB: `helpers` itself does `use crate::prelude::*`, so this must not be a glob back
// into it — name them.
pub use crate::wldap32::helpers::{
    bvarraydup, bvarrayfree, bvarraylen, bvdup, controlAtoW, controlUtoW, controlWtoA,
    controlWtoU, controlarrayAtoW, controlarrayUtoW, controlarrayWtoA, controlarrayWtoU,
    controlarrayfreeA, controlarrayfreeU, controlarrayfreeW, controlarraylenA,
    controlarraylenU, controlarraylenW, controlfreeA, controlfreeU, controlfreeW, modAtoW,
    modWtoU, modarrayAtoW, modarrayWtoU, modarrayfreeU, modarrayfreeW, modarraylenA,
    modarraylenW, modfreeU, modfreeW, sortkeyAtoW, sortkeyWtoA, sortkeyWtoU,
    sortkeyarrayAtoW, sortkeyarrayWtoU, sortkeyarrayfreeU, sortkeyarrayfreeW,
    sortkeyarraylenA, sortkeyarraylenW, sortkeyfreeA, sortkeyfreeU, sortkeyfreeW, strAtoW,
    strUtoW, strWtoA, strWtoU, strarrayAtoW, strarrayUtoW, strarrayWtoA, strarrayWtoU,
    strarrayfreeA, strarrayfreeU, strarrayfreeW, strarraylenA, strarraylenU, strarraylenW,
    strdupU, strdupW, strfreeA, strfreeU, strfreeW,
};
