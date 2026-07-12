//! The C runtime functions the ring calls.
//!
//! These are the ONE class that genuinely IS `extern "C"` (cdecl) on every arch — the
//! CRT is not WINAPI. Contrast crate::win32 / crate::gdi32, where an `extern "C"` decl
//! would be an i386 stdcall-decoration bug. Keeping them here makes the distinction
//! explicit rather than accidental.
#![allow(clippy::all)]
use core::ffi::{c_char, c_int, c_void};

extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int;
    pub fn strlen(s: *const c_char) -> usize;
    pub fn snprintf(s: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}
