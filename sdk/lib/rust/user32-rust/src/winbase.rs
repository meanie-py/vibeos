//! winbase — the NT/gdi32/kernel32/CRT base surface that the Wave-3 leaf modules
//! (rect/accel/text/font/display) call but that `win32k_sys::win32k` does not expose.
//! These are forward declarations resolved at link against the real DLLs (gdi32,
//! kernel32, ntdll, the CRT), plus the handful of consts/macros those files use.
//! Centralised here so font.rs and display.rs don't each redeclare SelectObject etc.
#![allow(non_snake_case, dead_code)]

use core::ffi::{c_int, c_void};
pub use core::ffi::c_char;

use win32k_sys::win32k::*;

// ---- gdi32 (the DC/text/draw surface display.rs + font.rs marshal into) ----
extern "system" {
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn CreatePen(iStyle: c_int, cWidth: c_int, color: COLORREF) -> HPEN;
    pub fn MoveToEx(hdc: HDC, x: c_int, y: c_int, lppt: LPPOINT) -> BOOL;
    pub fn LineTo(hdc: HDC, x: c_int, y: c_int) -> BOOL;
    pub fn GetBkMode(hdc: HDC) -> c_int;
    pub fn GetTextColor(hdc: HDC) -> COLORREF;
    pub fn TextOutW(hdc: HDC, x: c_int, y: c_int, lpString: LPCWSTR, c: c_int) -> BOOL;
    pub fn ExtTextOutW(
        hdc: HDC,
        x: c_int,
        y: c_int,
        options: UINT,
        lprect: *const RECT,
        lpString: LPCWSTR,
        c: UINT,
        lpDx: *const INT,
    ) -> BOOL;
    pub fn GetTextExtentPointW(hdc: HDC, lpString: LPCWSTR, c: c_int, lpsz: LPSIZE) -> BOOL;
    pub fn GetTextMetricsW(hdc: HDC, lptm: LPTEXTMETRICW) -> BOOL;
}

// ---- kernel32 (codepage conversion used by the A/W string thunks) ----
extern "system" {
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
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
    pub fn GetProcessHeap() -> PVOID;
    pub fn GetStringTypeW(dwInfoType: DWORD, lpSrcStr: LPCWSTR, cchSrc: c_int, lpCharType: LPWORD) -> BOOL;
    pub fn IsDBCSLeadByte(TestChar: BYTE) -> BOOL;
    pub fn IsDBCSLeadByteEx(CodePage: UINT, TestChar: BYTE) -> BOOL;
}

// RtlGetProcessHeap lives in crate::internal (the NtCurrentPeb()->ProcessHeap inline);
// the modules import it via `use crate::internal::*`, so winbase must NOT redefine it.

// ---- CRT (the few libc helpers the text/accel paths use) ----
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn abs(i: c_int) -> c_int;
    pub fn iswspace(c: c_int) -> c_int;
    pub fn towlower(c: wint_t) -> wint_t;
    pub fn towupper(c: wint_t) -> wint_t;
}

// ---- codepage + heap constants ----
pub const CP_ACP: UINT = 0;
pub const CP_THREAD_ACP: UINT = 3;
pub const LMEM_FIXED: UINT = 0x0000;
pub const CP_OEMCP: UINT = 1;
// GetStringTypeW dwInfoType + CT_CTYPE1 character-type bits (winnls.h)
pub const CT_CTYPE1: DWORD = 0x0001;
pub const C1_UPPER: WORD = 0x0001;
pub const C1_LOWER: WORD = 0x0002;
pub const C1_DIGIT: WORD = 0x0004;
pub const C1_SPACE: WORD = 0x0008;
pub const C1_ALPHA: WORD = 0x0100;

// ---- CRITICAL_SECTION (RTL_CRITICAL_SECTION layout; the modules only pass &mut) ----
#[repr(C)]
pub struct CRITICAL_SECTION {
    pub DebugInfo: PVOID,
    pub LockCount: LONG,
    pub RecursionCount: LONG,
    pub OwningThread: HANDLE,
    pub LockSemaphore: HANDLE,
    pub SpinCount: ULONG_PTR,
}
pub type LPCRITICAL_SECTION = *mut CRITICAL_SECTION;

// ---- NT/Rtl helpers the translation calls function-style ----
#[inline]
pub fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

#[inline]
pub unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

#[inline]
pub unsafe fn RtlZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

/// `FIELD_OFFSET(T, field)` → byte offset of `field` within `T`.
#[macro_export]
macro_rules! FIELD_OFFSET {
    ($ty:ty, $field:ident) => {{
        let uninit = core::mem::MaybeUninit::<$ty>::uninit();
        let base = uninit.as_ptr();
        // byte offset of the field — a usize (matches the dmSize/OldSize usize uses)
        unsafe { (core::ptr::addr_of!((*base).$field) as usize) - (base as usize) }
    }};
}
