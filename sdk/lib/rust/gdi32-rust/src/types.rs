//! Minimal Win32 type aliases for the gdi32 translation. Kept self-contained (no heavy
//! `*-sys` dependency) until the FFI surface grows enough to warrant the bindgen win32k-sys
//! GDI surface at Wave 1a. Layout matches the Win32 SDK on all three targets (LLP64 on
//! amd64/arm64, ILP32 on i386).

use core::ffi::c_void;

pub type BOOL = i32;
pub type BOOLEAN = u8;
pub type DWORD = u32;
pub type UINT = u32;
pub type INT = i32;
pub type LONG = i32;
pub type WORD = u16;
pub type BYTE = u8;

/// Pointer-width unsigned/signed (UINT_PTR/INT_PTR/DWORD_PTR/LPARAM/WPARAM/LRESULT etc.).
pub type UINT_PTR = usize;
pub type INT_PTR = isize;
pub type DWORD_PTR = usize;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;
pub type LPARAM = LONG_PTR;
pub type WPARAM = UINT_PTR;
pub type LRESULT = LONG_PTR;

pub type HANDLE = *mut c_void;
pub type HWND = HANDLE;
pub type HDC = HANDLE;
pub type HGDIOBJ = HANDLE;
pub type HBRUSH = HANDLE;
pub type HPEN = HANDLE;
pub type HFONT = HANDLE;
pub type HBITMAP = HANDLE;
pub type HRGN = HANDLE;
pub type HPALETTE = HANDLE;
pub type HINSTANCE = HANDLE;
pub type LPVOID = *mut c_void;
pub type LPDWORD = *mut DWORD;
pub type LPWSTR = *mut u16;
pub type LPCWSTR = *const u16;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

/// COLORREF — packed 0x00BBGGRR.
pub type COLORREF = DWORD;
