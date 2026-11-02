//! Host stand-in for normaliz-rust's `crate::prelude` (the shipping one re-exports a
//! Windows bindgen island, whose `c_ulong` DWORD is 8 bytes on an LP64 host). The bought
//! modules reach everything through `use crate::prelude::*`, so this is the one surface they
//! see. Constants carry the SDK's values; a wrong one shows up as a last-error mismatch,
//! since normaliz.json probes the error cell. Externs resolve to normaliz_env.c, which the
//! genuine C links too.
#![allow(dead_code, non_camel_case_types, non_snake_case)]

pub use core::ffi::c_void;

pub type BOOL = i32;
pub type INT = i32;
pub type UINT = u32;
pub type DWORD = u32;
pub type WORD = u16;
pub type WCHAR = u16;
pub type SIZE_T = usize;
pub type HANDLE = *mut c_void;
pub type LPCWSTR = *const WCHAR;
pub type LPWSTR = *mut WCHAR;
pub type NORM_FORM = i32;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// winerror.h
pub const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
pub const ERROR_INVALID_PARAMETER: DWORD = 87;
pub const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;
pub const ERROR_INVALID_NAME: DWORD = 123;
pub const ERROR_INVALID_FLAGS: DWORD = 1004;
pub const ERROR_NO_UNICODE_TRANSLATION: DWORD = 1113;
// winnls.h
pub const IDN_ALLOW_UNASSIGNED: DWORD = 0x01;
pub const IDN_USE_STD3_ASCII_RULES: DWORD = 0x02;
pub const MAP_FOLDCZONE: DWORD = 0x10;

extern "C" {
    pub fn SetLastError(code: DWORD);
    pub fn GetLastError() -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(heap: HANDLE, flags: DWORD, size: SIZE_T) -> *mut c_void;
    pub fn HeapFree(heap: HANDLE, flags: DWORD, mem: *mut c_void) -> BOOL;
    pub fn FoldStringW(flags: DWORD, src: LPCWSTR, srclen: INT, dst: LPWSTR, dstlen: INT) -> INT;
    pub fn memcpy(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    pub fn memmove(d: *mut c_void, s: *const c_void, n: usize) -> *mut c_void;
    fn wcslen(s: *const WCHAR) -> usize;
    fn towlower(c: u16) -> u16;
}

// win32macros-rust's strlenW/tolowerW, over the environment's 16-bit wcslen/towlower.
pub unsafe fn strlenW(s: *const WCHAR) -> usize {
    unsafe { wcslen(s) }
}
pub unsafe fn tolowerW(c: WCHAR) -> WCHAR {
    unsafe { towlower(c) }
}

#[macro_export]
macro_rules! DPRINT {
    ($($arg:tt)*) => {{}};
}
#[macro_export]
macro_rules! DPRINT1 {
    ($($arg:tt)*) => {{}};
}
