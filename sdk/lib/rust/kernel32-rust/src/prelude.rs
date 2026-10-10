//! Crate-wide prelude of the *universal* symbols the per-file DeepSeek translations
//! expect to be in scope (CRT functions, Wine wide-string helpers, common Win32/NT
//! constants, and no-op Wine debug shims). Per-file modules add `use crate::prelude::*;`.
//!
//! This is a substrate convenience layer — subsystem-specific types/consts (console
//! CSR messages, SERIAL_*, IOCTL_TAPE_*, toolhelp entries) do NOT belong here; they
//! live in their owning module.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use ntdll_sys::nt::*;

// ---- re-exports of symbols already defined elsewhere in the crate ----
pub use crate::k32_externs::{
    INVALID_HANDLE_VALUE, SetLastError, GetLastError, BaseSetLastNTError, RtlGetProcessHeap,
    HANDLE_DETACHED_PROCESS, HANDLE_CREATE_NEW_CONSOLE, HANDLE_CREATE_NO_WINDOW,
    STARTF_SHELLPRIVATE,
};
pub use crate::string::nls_subst::{NT_SUCCESS, CP_ACP};

// ---- universal type aliases the c2rust output uses ----
pub type VOID = core::ffi::c_void;
pub type size_t = usize;
pub use core::ffi::{c_int, c_long};

// ---- universal NT / Win32 constants ----
pub const UNICODE_NULL: u16 = 0;
pub const ANSI_NULL: CHAR = 0;
pub const NULL: PVOID = core::ptr::null_mut();
pub const CP_UTF8: UINT = 65001;
pub const MAXULONG_PTR: ULONG_PTR = ULONG_PTR::MAX;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFF_FFFF;
pub const HFILE_ERROR: INT = -1;
pub const STD_INPUT_HANDLE: DWORD = -10i32 as DWORD;
pub const STD_OUTPUT_HANDLE: DWORD = -11i32 as DWORD;
pub const STD_ERROR_HANDLE: DWORD = -12i32 as DWORD;

// SharedUserData — KUSER_SHARED_DATA at the fixed user VA 0x7FFE0000.
pub const SharedUserData: *mut KUSER_SHARED_DATA = 0x7FFE_0000 as *mut KUSER_SHARED_DATA;

// ---- min() helper (c2rust emits bare `min(a, b)`) ----
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

// ---- CRT functions (cdecl → extern "C"); resolve against ntdll/CRT at link ----
extern "C" {
    pub fn strlen(s: *const i8) -> usize;
    pub fn strchr(s: *const i8, c: c_int) -> *mut i8;
    pub fn strrchr(s: *const i8, c: c_int) -> *mut i8;
    pub fn strcmp(a: *const i8, b: *const i8) -> c_int;
    pub fn strcpy(d: *mut i8, s: *const i8) -> *mut i8;
    pub fn strcat(d: *mut i8, s: *const i8) -> *mut i8;
    pub fn strncpy(d: *mut i8, s: *const i8, n: usize) -> *mut i8;
    pub fn strpbrk(s: *const i8, a: *const i8) -> *mut i8;
    pub fn memcpy(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memmove(d: *mut VOID, s: *const VOID, n: usize) -> *mut VOID;
    pub fn memset(d: *mut VOID, c: c_int, n: usize) -> *mut VOID;
    pub fn memcmp(a: *const VOID, b: *const VOID, n: usize) -> c_int;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcscpy(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcscat(d: *mut u16, s: *const u16) -> *mut u16;
    pub fn wcsncpy(d: *mut u16, s: *const u16, n: usize) -> *mut u16;
    pub fn wcsstr(h: *const u16, n: *const u16) -> *mut u16;
    pub fn wcscspn(s: *const u16, set: *const u16) -> usize;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn _strcmpi(a: *const i8, b: *const i8) -> c_int;
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> ULONG;
    pub fn tolower(c: c_int) -> c_int;
    pub fn toupper(c: c_int) -> c_int;
    pub fn isalpha(c: c_int) -> c_int;
    pub fn islower(c: c_int) -> c_int;
}

// ---- Wine wide-string helpers (wine/unicode.h) → map to the CRT wide fns ----
#[inline] pub unsafe fn strlenW(s: *const u16) -> usize { wcslen(s) }
#[inline] pub unsafe fn strcmpW(a: *const u16, b: *const u16) -> c_int { wcscmp(a, b) }
#[inline] pub unsafe fn strcmpiW(a: *const u16, b: *const u16) -> c_int { _wcsicmp(a, b) }
#[inline] pub unsafe fn strncmpiW(a: *const u16, b: *const u16, n: usize) -> c_int { _wcsnicmp(a, b, n) }
#[inline] pub unsafe fn strcpyW(d: *mut u16, s: *const u16) -> *mut u16 { wcscpy(d, s) }
#[inline] pub unsafe fn strcatW(d: *mut u16, s: *const u16) -> *mut u16 { wcscat(d, s) }
#[inline] pub unsafe fn strchrW(s: *const u16, c: u16) -> *mut u16 { wcschr(s, c) }
#[inline] pub unsafe fn strrchrW(s: *const u16, c: u16) -> *mut u16 { wcsrchr(s, c) }
#[inline] pub unsafe fn strstrW(h: *const u16, n: *const u16) -> *mut u16 { wcsstr(h, n) }
#[inline] pub unsafe fn strtoulW(s: *const u16, e: *mut *mut u16, b: c_int) -> ULONG { wcstoul(s, e, b) }

// Wine ctype-W helpers.
#[inline] pub fn toupperW(c: u16) -> u16 {
    if (b'a' as u16..=b'z' as u16).contains(&c) { c - 0x20 } else { c }
}
#[inline] pub fn isspaceW(c: u16) -> bool {
    matches!(c, 0x20 | 0x09 | 0x0A | 0x0B | 0x0C | 0x0D)
}
#[inline] pub fn isxdigitW(c: u16) -> bool {
    matches!(c, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66)
}
#[inline] pub unsafe fn memchrW(s: *const u16, c: u16, n: usize) -> *mut u16 {
    let mut i = 0;
    while i < n { if *s.add(i) == c { return s.add(i) as *mut u16; } i += 1; }
    core::ptr::null_mut()
}
#[inline] pub unsafe fn memrchrW(s: *const u16, c: u16, n: usize) -> *mut u16 {
    let mut i = n;
    while i > 0 { i -= 1; if *s.add(i) == c { return s.add(i) as *mut u16; } }
    core::ptr::null_mut()
}

// lstrlenW lives in nls_subst (extern import) — re-export so all files see it.
pub use crate::string::nls_subst::lstrlenW;


// DUMMY_PSEH: SEH filters/handlers are dead code; this only appears in such handlers.
#[inline] pub fn _SEH2_GetExceptionCode() -> LONG { 0 }

// ---- no-op Wine debug shims (NDEBUG build: TRACE/WARN/ERR/FIXME compile out) ----
#[inline] pub fn debugstr_w(_s: *const u16) -> *const i8 { core::ptr::null() }
#[inline] pub fn debugstr_a(_s: *const i8) -> *const i8 { core::ptr::null() }
#[inline] pub fn debugstr_wn(_s: *const u16, _n: c_int) -> *const i8 { core::ptr::null() }
