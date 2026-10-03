//! Preamble: Win32 macros, resource IDs, registry paths, and wide-string
//! literals that the per-file translation dropped (rustconv only captures each
//! .c's top-of-file preamble, so header #defines vanish) or mis-emitted (the
//! stitcher hoisted `L"..."` literals to `L_xxx()` fns, and emitted Win32
//! macros like RGB/MAKEINTRESOURCEW as extern fns). Shared by userinit.rs +
//! livecd.rs via `use crate::preamble::*`.

#![allow(non_snake_case)]

use winsrv_sys::winsrv::*;
use winsrv_sys::w;

// ---- resource.h ----
pub const RC_STRING_MAX_SIZE: u32 = 2048;
pub const IDB_ROSLOGO: INT = 100;
pub const IDB_ROSMASK: INT = 101;
pub const IDD_LOCALEPAGE: INT = 200;
// IDC_* are matched against LOWORD(wParam) alongside winsrv's IDOK/IDCANCEL
// (u32) in the dialog procs → u32. GetDlgItem call sites cast back to INT.
pub const IDC_LOCALELOGO: u32 = 201;
pub const IDC_LANGUAGELIST: u32 = 202;
pub const IDC_LAYOUTLIST: u32 = 204;
pub const IDD_STARTPAGE: INT = 300;
pub const IDC_STARTLOGO: u32 = 301;
pub const IDC_RUN: u32 = 302;
pub const IDC_INSTALL: u32 = 303;
// IDS_* are passed to LoadStringW (UINT uID) → u32.
pub const IDS_SHELL_FAIL: u32 = 600;
pub const IDS_INSTALLER_FAIL: u32 = 601;
pub const IDS_CANCEL_CONFIRM: u32 = 602;
pub const IDS_SPAIN: u32 = 3900;

// ---- predefined registry roots (winreg.h) ----
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002usize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001usize as HKEY;

pub const UNICODE_NULL: u16 = 0;

// ---- registry paths (regstr.h) + privilege name (winnt.h) ----
pub fn REGSTR_PATH_CURRENT_CONTROL_SET() -> *const u16 {
    w!("System\\CurrentControlSet\\Control").as_ptr()
}
pub fn REGSTR_PATH_DESKTOP() -> *const u16 {
    w!("Control Panel\\Desktop").as_ptr()
}
pub fn REGSTR_PATH_COLORS() -> *const u16 {
    w!("Control Panel\\Colors").as_ptr()
}
pub fn SE_SHUTDOWN_NAME() -> *const u16 {
    w!("SeShutdownPrivilege").as_ptr()
}

// ---- hoisted wide-string literals (stitcher named them L_xxx, calls as fns) ----
pub fn L_STR_FMT() -> *const u16 {
    w!("SYSTEM\\CurrentControlSet\\Control\\Keyboard Layouts\\%s").as_ptr()
}
pub fn L_DISPNAME() -> *const u16 { w!("Layout Display Name").as_ptr() }
pub fn L_LAYOUTTEXT() -> *const u16 { w!("Layout Text").as_ptr() }
pub fn L_00000409() -> *const u16 { w!("00000409").as_ptr() }
pub fn L_KeyboardLayouts() -> *const u16 {
    w!("System\\CurrentControlSet\\Control\\Keyboard Layouts").as_ptr()
}
pub fn L_ReactOS_LiveCD() -> *const u16 { w!("ReactOS LiveCD").as_ptr() }

// ---- Win32 macros (emitted as extern fns / used as fns by the LLM) ----
pub const fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}
pub const fn GetRValue(rgb: COLORREF) -> u8 { (rgb & 0xff) as u8 }
pub const fn GetGValue(rgb: COLORREF) -> u8 { ((rgb >> 8) & 0xff) as u8 }
pub const fn GetBValue(rgb: COLORREF) -> u8 { ((rgb >> 16) & 0xff) as u8 }
pub const fn LOWORD(l: u32) -> u16 { (l & 0xffff) as u16 }
pub const fn HIWORD(l: u32) -> u16 { ((l >> 16) & 0xffff) as u16 }
pub const fn MAKEWPARAM(lo: u16, hi: u16) -> u32 { (lo as u32) | ((hi as u32) << 16) }
pub const fn MAKELANGID(p: u32, s: u32) -> u32 { ((s & 0x3ff) << 10) | (p & 0x3ff) }
pub const fn MAKELCID(lgid: u32, srtid: u32) -> u32 { ((srtid & 0xffff) << 16) | (lgid & 0xffff) }
pub const fn PRIMARYLANGID(lgid: u32) -> u32 { lgid & 0x3ff }
pub const fn LANGIDFROMLCID(lcid: u32) -> u32 { lcid & 0xffff }
pub fn MAKEINTRESOURCEW(i: INT) -> LPWSTR { (i as u16 as usize) as LPWSTR }
pub fn ARRAYSIZE<T, const N: usize>(_: &[T; N]) -> usize { N }

// StringCchCatW (strsafe.h) is a C FORCEINLINE — no exported symbol to link
// against, so reimplement it: append pszSrc to pszDest with the combined
// length (incl. NUL) bounded by cchDest. Returns S_OK / STRSAFE_E_INSUFFICIENT_BUFFER.
pub unsafe fn StringCchCatW(pszDest: *mut u16, cchDest: usize, pszSrc: *const u16) -> i32 {
    const S_OK: i32 = 0;
    const STRSAFE_E_INSUFFICIENT_BUFFER: i32 = 0x8007_007Au32 as i32;
    if cchDest == 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    let mut d = 0usize;
    while d < cchDest && *pszDest.add(d) != 0 {
        d += 1;
    }
    let mut s = 0usize;
    while d + 1 < cchDest && *pszSrc.add(s) != 0 {
        *pszDest.add(d) = *pszSrc.add(s);
        d += 1;
        s += 1;
    }
    if d < cchDest {
        *pszDest.add(d) = 0;
    }
    if *pszSrc.add(s) == 0 { S_OK } else { STRSAFE_E_INSUFFICIENT_BUFFER }
}

// GetWindowLongPtrW/SetWindowLongPtrW: on 32-bit Win32 these are #defines
// for the non-Ptr GetWindowLongW/SetWindowLongW (winsrv-sys only exports the
// non-Ptr forms on i386). On 64-bit the real Ptr APIs exist.
#[cfg(target_pointer_width = "32")]
pub unsafe fn GetWindowLongPtrW(hwnd: HWND, index: INT) -> LONG_PTR {
    GetWindowLongW(hwnd, index) as LONG_PTR
}
#[cfg(target_pointer_width = "32")]
pub unsafe fn SetWindowLongPtrW(hwnd: HWND, index: INT, new: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hwnd, index, new as LONG) as LONG_PTR
}
#[cfg(target_pointer_width = "64")]
pub use winsrv_sys::winsrv::{GetWindowLongPtrW, SetWindowLongPtrW};

// ---- CRT (cdecl) — single source of truth, shared by both modules ----
extern "C" {
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn _wtoi(s: *const u16) -> i32;
    pub fn wcscat(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    pub fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcstoul(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> core::ffi::c_ulong;
    pub fn wcstol(nptr: *const u16, endptr: *mut *mut u16, base: i32) -> core::ffi::c_long;
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}

// ---- gap fns not in winsrv-sys bindgen ----
// Declared with "system" callback fn-ptr params (not winsrv's bindgen DLGPROC /
// LOCALE_ENUMPROCW, which are "stdcall" on i386 — a distinct Rust fn type even
// though "system" == stdcall there). Lets the translated "system" callbacks
// pass without per-arch ABI cfg. Shadows winsrv's EnumSystemLocalesW via the
// explicit-decl-beats-glob rule.
extern "system" {
    pub fn ExpandEnvironmentStringsW(lpSrc: *const u16, lpDst: *mut u16, nSize: DWORD) -> DWORD;
    // userenv.dll: the LiveCD shell's environment block (upstream eec8c977083).
    pub fn CreateEnvironmentBlock(lpEnvironment: *mut LPVOID, hToken: HANDLE, bInherit: BOOL) -> BOOL;
    pub fn DestroyEnvironmentBlock(lpEnvironment: LPVOID) -> BOOL;
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR>,
        dwInitParam: LPARAM,
    ) -> INT_PTR;
    pub fn EnumSystemLocalesW(
        lpLocaleEnumProc: Option<unsafe extern "system" fn(LPWSTR) -> BOOL>,
        dwFlags: DWORD,
    ) -> BOOL;
    pub fn ExitWindowsEx(uFlags: UINT, dwReason: DWORD) -> BOOL;
    pub fn GdiAlphaBlend(
        hdcDest: HDC,
        xoriginDest: INT,
        yoriginDest: INT,
        wDest: INT,
        hDest: INT,
        hdcSrc: HDC,
        xoriginSrc: INT,
        yoriginSrc: INT,
        wSrc: INT,
        hSrc: INT,
        ftn: BLENDFUNCTION,
    ) -> BOOL;
}
