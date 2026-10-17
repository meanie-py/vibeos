//! The C runtime functions the ring calls.
//!
//! These are the ONE class that genuinely IS `extern "C"` (cdecl) on every arch — the
//! CRT is not WINAPI. Contrast crate::win32 / crate::gdi32, where an `extern "C"` decl
//! would be an i386 stdcall-decoration bug. Keeping them here makes the distinction
//! explicit rather than accidental.
#![allow(clippy::all)]
use core::ffi::{c_char, c_int, c_long, c_ulong, c_void};

extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    pub fn sprintf(s: *mut c_char, fmt: *const c_char, ...) -> c_int;
    // The C's count-form `swprintf(buf, n, fmt, ...)` is a static inline in <stdio.h> over
    // `_vsnwprintf`; msvcrt's EXPORTED `swprintf` is the legacy `(buf, fmt, ...)`, which
    // read `n` as the format pointer (cscript crashed reporting a script error).
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcstol(s: *const u16, endp: *mut *mut u16, base: c_int) -> c_long;
    pub fn wcstoul(s: *const u16, endp: *mut *mut u16, base: c_int) -> c_ulong;
    pub fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> c_int;
    pub fn towupper(c: u16) -> u16; // wint_t: the C returns an unsigned short, not an int

    /// user32's wsprintfW. It lives with the CRT rather than in user32.rs because it is
    /// VARIADIC: a variadic Win32 function is __cdecl even on i386 (callee-cleans-stack
    /// cannot express a variable arg count), so `extern "C"` is the correct ABI here.
    pub fn wsprintfW(buf: *mut u16, fmt: *const u16, ...) -> c_int;

    /// stdlib.h `int abs(int)` — cdecl.
    pub fn abs(n: c_int) -> c_int;
    /// string.h `char *strcpy(char *, const char *)` — cdecl.
    pub fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}
