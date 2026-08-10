//! Hand-written residue for the bindgen island — the things bindgen has nothing to bind.
//!
//! `island.rs` re-exports this through the same glob (see the `<stem>_extra.rs` hook in
//! tools/rustconv/gen_driver_types.sh), so regenerating the island never eats it.
//!
//! Everything here is macro TEXT rather than a C declaration, which is why no header-derived
//! generator can produce it:
//!
//!   * pointer-valued macros — `#define NULL ((void*)0)`, INVALID_HANDLE_VALUE,
//!     HBMMENU_CALLBACK, HKEY_CLASSES_ROOT. bindgen emits integer and string macros; a macro
//!     whose expansion is a pointer cast is not a constant expression it can type.
//!   * cast-expression constants — `#define E_FAIL ((HRESULT)0x80004005L)`, same reason.
//!   * function-like macros — MAKE_HRESULT, HRESULT_FROM_WIN32, kept as `#[inline] fn` because
//!     they are used in value position and are not debug no-ops (the §9.65 rule against
//!     out-of-line no-op fns is about DEBUG macros, where the barrier uncloaks ABI mismatches).
//!   * CRT/Win32 string entry points, while the island runs BLOCKLIST_FUNCS=1.
//!
//! Before adding anything here, check whether a header already declares it. An item in this
//! file is invisible to `verify_ffi`, and shell32 having no island at all is exactly how it
//! accumulated four real ABI disagreements while 40 other crates were being converged.

// Base types come straight from ntdll-sys, the same crate the island's own glob resolves its
// field types through — NOT from `crate::island`, which would be a resolution cycle.
use ntdll_sys::nt::{HANDLE, HBITMAP, HKEY, HRESULT, INT, LONG, LPCWSTR, LPVOID, LPWSTR, WCHAR};

pub const NULL: LPVOID = core::ptr::null_mut();
pub const UNICODE_NULL: WCHAR = 0;
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;
pub const HBMMENU_CALLBACK: HBITMAP = -1isize as HBITMAP;
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110u32 as HRESULT;
pub const MK_E_NOOBJECT: HRESULT = 0x800401E5u32 as HRESULT;

/// `MAKE_HRESULT` (winerror.h).
#[inline]
pub const fn MAKE_HRESULT(severity: u32, facility: u32, code: u32) -> HRESULT {
    ((severity << 31) | (facility << 16) | code) as HRESULT
}

/// `HRESULT_FROM_WIN32` (winerror.h): positive Win32 error → 0x8007xxxx; pass-through
/// otherwise. Faithful port of `__HRESULT_FROM_WIN32`.
#[inline]
pub fn HRESULT_FROM_WIN32(x: LONG) -> HRESULT {
    const FACILITY_WIN32: u32 = 7;
    if x > 0 {
        (((x as u32) & 0x0000FFFF) | (FACILITY_WIN32 << 16) | 0x80000000) as HRESULT
    } else {
        x as HRESULT
    }
}

// `lstrcpynW` USED to be declared here. It is not residue any more: all three arch islands now
// bind it from the real header (`unsafe extern "stdcall"`, identical to the `extern "system"`
// this block had), and two glob-visible declarations of one name make every unqualified use an
// E0659 ambiguity — even `crate::island::lstrcpynW`, because both globs land in `island`.
// Anything hand-written here that bindgen later learns to emit must be deleted, not kept "just
// in case": the duplicate does not shadow, it collides.

extern "C" {
    pub fn wcscpy(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
    pub fn wcschr(text: LPCWSTR, character: WCHAR) -> LPWSTR;
    pub fn wcscmp(left: LPCWSTR, right: LPCWSTR) -> INT;
    pub fn wcslen(text: LPCWSTR) -> usize;
    pub fn wcsstr(haystack: LPCWSTR, needle: LPCWSTR) -> LPWSTR;
    pub fn wcsrchr(text: LPCWSTR, character: WCHAR) -> LPWSTR;
    pub fn iswspace(c: INT) -> INT;
    #[allow(non_snake_case)]
    pub fn _wcsnicmp(s1: LPCWSTR, s2: LPCWSTR, n: usize) -> INT;
}
