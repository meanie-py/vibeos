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
// wine/debug.h's `debugstr_a`/`debugstr_w` format a string for a TRACE argument. Every
// call in this module was an argument to a TRACE that the translation dropped, leaving
// the formatter as an orphaned statement. No-op MACROS, not fns, for the §9.65 reason
// above -- and macro form keeps them visibly dead rather than silently deleted.
#[macro_export] macro_rules! debugstr_a { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! debugstr_w { ($($arg:tt)*) => {{}}; }
pub use crate::{debugstr_a, debugstr_w};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// str*/sprintf: wldap32.h's marshalling helpers call these directly. `strlenW`/`strcpyW`
// are wine/unicode.h MACROS over wcslen/wcscpy (unicode.h:34-35), not separate symbols.
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    // init.c's split_hostnames walks a hostname list with it. Real msvcrt `isspace` takes an
    // `int`, and the argument must be zero-extended from the `char` — a negative value other
    // than EOF is undefined behaviour there. Callers pass `*p as u8 as c_int`.
    pub fn isspace(c: c_int) -> c_int;
}
#[inline(always)]
pub unsafe fn strlenW(s: *const u16) -> usize {
    wcslen(s)
}
#[inline(always)]
pub unsafe fn strcpyW(dst: *mut u16, src: *const u16) -> *mut u16 {
    wcscpy(dst, src)
}

// ================= Win32 ======================================================================
// `extern "system"`, not `extern "C"`: every one of these is WINAPI, which on i386 is stdcall,
// so the import lib carries `_Name@N`. They were declared cdecl and nothing caught it because
// wldap32-rust is translated but not yet flipped into the build — `cargo check` does not link,
// so this would have surfaced as a wall of undefined symbols on the day of the flip.
extern "system" {
    pub fn GetProcessHeap() -> HANDLE;
    // `dwBytes` is SIZE_T == ULONG_PTR == pointer-width unsigned, which is exactly Rust
    // `usize` on every target. Spelling it `usize` rather than the bindgen `SIZE_T` alias
    // is ABI-identical and keeps `strlen(..) * size_of::<T>()` type-checking on all three
    // arches -- with SIZE_T, crate-repair had to bake `as u32` on i386 and `as u64` on
    // amd64/arm64, and refused (rightly) rather than pick one.
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
    pub fn LoadStringA(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPSTR, cchBufferMax: c_int)
        -> c_int;
    pub fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: c_int)
        -> c_int;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: c_int,
    ) -> c_int;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: c_int,
        lpMultiByteStr: LPSTR,
        cbMultiByte: c_int,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: LPBOOL,
    ) -> c_int;
}

// wine/heap.h's `static inline` allocators. Every TU inlined its own copy in C; here they
// are one definition each. Real bodies, so §9.65 (never lower a no-op debug MACRO to an
// out-of-line fn) does not apply -- that rule is about no-op barriers, not real code.
pub const HEAP_ZERO_MEMORY: DWORD = 0x0000_0008;
#[inline(always)]
pub unsafe fn heap_alloc(len: usize) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), 0, len)
}
#[inline(always)]
pub unsafe fn heap_alloc_zero(len: usize) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[inline(always)]
pub unsafe fn heap_free(mem: *mut c_void) {
    HeapFree(GetProcessHeap(), 0, mem);
}

// winnt.h's `ARRAY_SIZE(a)`. A macro, not a fn: it must take the array by name and read
// its compile-time length at each call site.
#[macro_export]
macro_rules! ARRAY_SIZE {
    ($a:expr) => {
        ($a).len() as u32
    };
}
pub use crate::ARRAY_SIZE;

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
    strAtoW, strUtoW, strWtoA, strWtoU, strarrayAtoW, strarrayUtoW, strarrayWtoA,
    strarrayWtoU, strarrayfreeA, strarrayfreeU, strarrayfreeW, strarraylenA, strarraylenU,
    strarraylenW, strdupU, strdupW, strfreeA, strfreeU, strfreeW,
};
// The other 43 helpers marshal LDAPMod/LDAPControl/LDAPSortKey/berval, whose types only
// exist when a real libldap is present. wldap32.h:244 puts them inside `#ifdef HAVE_LDAP`,
// so the re-export has to carry the same guard or it names items that are not compiled.
#[cfg(feature = "have_ldap")]
pub use crate::wldap32::helpers::{
    bvarraydup, bvarrayfree, bvarraylen, bvdup, controlAtoW, controlUtoW, controlWtoA,
    controlWtoU, controlarrayAtoW, controlarrayUtoW, controlarrayWtoA, controlarrayWtoU,
    controlarrayfreeA, controlarrayfreeU, controlarrayfreeW, controlarraylenA,
    controlarraylenU, controlarraylenW, controlfreeA, controlfreeU, controlfreeW, modAtoW,
    modWtoU, modarrayAtoW, modarrayWtoU, modarrayfreeU, modarrayfreeW, modarraylenA,
    modarraylenW, modfreeU, modfreeW, sortkeyAtoW, sortkeyWtoA, sortkeyWtoU,
    sortkeyarrayAtoW, sortkeyarrayWtoU, sortkeyarrayfreeU, sortkeyarrayfreeW,
    sortkeyarraylenA, sortkeyarraylenW, sortkeyfreeA, sortkeyfreeU, sortkeyfreeW,
};

// error.c's `map_error` maps a libldap return code to a WLDAP32_LDAP_* one. Nine other
// TUs call it by bare name; in C each got it from wldap32.h's declaration.
pub use crate::wldap32::error::map_error;

// ============ enum bare-name aliases =========================================
// winldap_private.h declares these as an anonymous-typedef'd `enum LDAP_RETCODE`, so
// bindgen namespaces every member under the enum's own name. The C refers to them by
// the bare name; sealing restored those bare names into the translation, so without
// these aliases 160 occurrences resolve to nothing. Values come from the island -- this
// is a rename of an existing constant, never a second opinion about its value.
pub const WLDAP32_LDAP_AUTH_UNKNOWN: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_AUTH_UNKNOWN;
pub const WLDAP32_LDAP_CLIENT_LOOP: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_CLIENT_LOOP;
pub const WLDAP32_LDAP_CONNECT_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_CONNECT_ERROR;
pub const WLDAP32_LDAP_CONTROL_NOT_FOUND: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_CONTROL_NOT_FOUND;
pub const WLDAP32_LDAP_DECODING_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_DECODING_ERROR;
pub const WLDAP32_LDAP_ENCODING_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_ENCODING_ERROR;
pub const WLDAP32_LDAP_FILTER_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_FILTER_ERROR;
pub const WLDAP32_LDAP_LOCAL_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_LOCAL_ERROR;
pub const WLDAP32_LDAP_MORE_RESULTS_TO_RETURN: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_MORE_RESULTS_TO_RETURN;
pub const WLDAP32_LDAP_NOT_SUPPORTED: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_NOT_SUPPORTED;
pub const WLDAP32_LDAP_NO_MEMORY: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_NO_MEMORY;
pub const WLDAP32_LDAP_NO_RESULTS_RETURNED: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_NO_RESULTS_RETURNED;
pub const WLDAP32_LDAP_PARAM_ERROR: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_PARAM_ERROR;
pub const WLDAP32_LDAP_REFERRAL_LIMIT_EXCEEDED: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_REFERRAL_LIMIT_EXCEEDED;
pub const WLDAP32_LDAP_SERVER_DOWN: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_SERVER_DOWN;
pub const WLDAP32_LDAP_SUCCESS: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_SUCCESS;
pub const WLDAP32_LDAP_TIMEOUT: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_TIMEOUT;
pub const WLDAP32_LDAP_UNWILLING_TO_PERFORM: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_UNWILLING_TO_PERFORM;
pub const WLDAP32_LDAP_USER_CANCELLED: LDAP_RETCODE = LDAP_RETCODE_WLDAP32_LDAP_USER_CANCELLED;
