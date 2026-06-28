//! Minimal Win32 type aliases for the shell32 translation. Self-contained (no heavy
//! `*-sys` dependency) until the FFI surface grows enough (the PIDL/shell-struct waves)
//! to warrant a bindgen shell32-sys. Layout matches the Win32 SDK on all three targets
//! (LLP64 on amd64/arm64, ILP32 on i386).

use core::ffi::c_void;

pub type BOOL = i32;
pub type BOOLEAN = u8;
pub type DWORD = u32;
pub type UINT = u32;
pub type INT = i32;
pub type LONG = i32;
pub type WORD = u16;
pub type BYTE = u8;
pub type CHAR = i8;
pub type WCHAR = u16;

pub type UINT_PTR = usize;
pub type INT_PTR = isize;
pub type DWORD_PTR = usize;
pub type ULONG_PTR = usize;
pub type LONG_PTR = isize;

pub type HANDLE = *mut c_void;
pub type HKEY = HANDLE;
pub type PHKEY = *mut HKEY;
pub type HINSTANCE = HANDLE;
pub type HWND = HANDLE;

pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type PVOID = *mut c_void;
pub type LPDWORD = *mut DWORD;
pub type LPBYTE = *mut BYTE;
pub type LPSTR = *mut CHAR;
pub type LPCSTR = *const CHAR;
pub type LPWSTR = *mut WCHAR;
pub type LPCWSTR = *const WCHAR;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const NULL: LPVOID = core::ptr::null_mut();

/// COM result code.
pub type HRESULT = LONG;
/// Registry security-access mask.
pub type REGSAM = DWORD;

pub const S_OK: HRESULT = 0;
pub const ERROR_SUCCESS: LONG = 0;

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

/// LUID — { LowPart: DWORD, HighPart: LONG }.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LUID {
    pub LowPart: DWORD,
    pub HighPart: LONG,
}

/// LARGE_INTEGER (only the quad is needed here).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LARGE_INTEGER {
    pub QuadPart: i64,
}

/// TOKEN_STATISTICS (setypes.h) — faithful layout so `size_of` matches what
/// GetTokenInformation expects. Only AuthenticationId (offset 8) is read here.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TOKEN_STATISTICS {
    pub TokenId: LUID,
    pub AuthenticationId: LUID,
    pub ExpirationTime: LARGE_INTEGER,
    pub TokenType: DWORD,          // TOKEN_TYPE enum
    pub ImpersonationLevel: DWORD, // SECURITY_IMPERSONATION_LEVEL enum
    pub DynamicCharged: DWORD,
    pub DynamicAvailable: DWORD,
    pub GroupCount: DWORD,
    pub PrivilegeCount: DWORD,
    pub ModifiedId: LUID,
}
