//! Preamble: Win32 predefined constants and the extern `WLSession`
//! global that the per-file translation can't resolve. winlogon-sys
//! covers the API surface + WLSESSION layout, but the predefined HKEY
//! roots are winreg.h macros (not bindgen consts) and `WLSession` is a
//! global owned by the still-C winlogon core. Shared by the leaf modules.

#![allow(non_snake_case, non_upper_case_globals)]

use winlogon_sys::nt::*;

// Predefined registry roots (winreg.h #defines — not bindgen-visible).
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY;
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000usize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003usize as HKEY;

// The session state, owned by the still-C winlogon core (winlogon.c:
// `PWLSESSION WLSession;`). The flipped leaves read it; the layout comes
// from winlogon-sys (_WLSESSION, size-asserted ×3). Use raw pointer reads
// — the C core writes it during init before any leaf runs.
extern "C" {
    pub static mut WLSession: PWLSESSION;
}

// CRT (cdecl) wide-string helpers the leaves call — not Win32 APIs, so
// not in winlogon-sys's bindgen. winlogon.exe links msvcrt.
extern "C" {
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsncpy(dest: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
}
