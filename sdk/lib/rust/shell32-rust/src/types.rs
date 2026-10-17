//! Minimal Win32 type aliases for the shell32 translation. Self-contained (no heavy
//! `*-sys` dependency) until the FFI surface grows enough (the PIDL/shell-struct waves)
//! to warrant a bindgen shell32-sys. Layout matches the Win32 SDK on all three targets
//! (LLP64 on amd64/arm64, ILP32 on i386).

use core::ffi::c_void;

// Canonical per-architecture SDK layouts. Keep ABI-heavy COM/Win32 records in
// com-sys rather than growing a second hand-maintained shell32 copy.
pub use com_sys::{
    BIND_OPTS, FORMATETC, HDC, HICON, HOLEMENU, INT8, LPOLEMENUGROUPWIDTHS, LRESULT,
    PBYTE, STATDATA, STATSTG, STGMEDIUM, SYSTEMTIME, ULARGE_INTEGER, ULONGLONG,
    WPARAM,
};
// Same rule as POINT/SIZE below: the island owns these layouts, so take them from there
// rather than from com-sys -- otherwise a vtable slot and the body behind it disagree.
pub use crate::island::{MSG, POINTL, RECT};

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

// The pointer-sized scalars and the nominal handles are the island's (ntdll-sys's), not
// `usize`/`*mut c_void`: two spellings of the same bytes are two Rust types, and every
// island function declaration rejects the other one at the call.
pub use ntdll_sys::nt::{
    DWORD_PTR, HBITMAP, HINSTANCE, HKEY, HMENU, HWND, INT_PTR, LONG_PTR, UINT_PTR, ULONG_PTR,
};

pub type HANDLE = *mut c_void;
pub type PHKEY = *mut HKEY;
pub type HGLOBAL = HANDLE;
pub type HPROPSHEETPAGE = HANDLE;
pub type COLORREF = DWORD;

pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type PVOID = *mut c_void;
pub type LPDWORD = *mut DWORD;
pub type LPBYTE = *mut BYTE;
pub type LPSTR = *mut CHAR;
pub type LPCSTR = *const CHAR;
pub type LPWSTR = *mut WCHAR;
pub type LPCWSTR = *const WCHAR;
pub type PCWSTR = *const WCHAR;
pub type LPCOLESTR = *const WCHAR;

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;
pub const NULL: LPVOID = core::ptr::null_mut();

/// COM result code.
pub type HRESULT = LONG;
/// Registry security-access mask.
pub type REGSAM = DWORD;

pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT;
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
pub const CLASS_E_NOAGGREGATION: HRESULT = 0x80040110u32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT;
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;
pub const MK_E_NOOBJECT: HRESULT = 0x800401E5u32 as HRESULT;
pub const ERROR_SUCCESS: LONG = 0;
pub const HEAP_ZERO_MEMORY: DWORD = 8;
pub const CP_ACP: UINT = 0;
pub const UNICODE_NULL: WCHAR = 0;
pub const MAX_PATH: usize = 260;
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const INVALID_HANDLE_VALUE: HANDLE = usize::MAX as HANDLE;
pub const RRF_RT_REG_SZ: DWORD = 0x0000_0002;
pub const RRF_RT_ANY: DWORD = 0x0000_FFFF;
pub const HBMMENU_CALLBACK: HBITMAP = -1isize as HBITMAP;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FOLDERSETTINGS {
    pub ViewMode: UINT,
    pub fFlags: UINT,
}
pub type LPFOLDERSETTINGS = *mut FOLDERSETTINGS;
pub type LPCFOLDERSETTINGS = *const FOLDERSETTINGS;

pub use crate::island::DRAWITEMSTRUCT;

pub const CF_UNICODETEXT: UINT = 13;
pub const GMEM_MOVEABLE: UINT = 2;
pub const GMEM_SHARE: UINT = 8192;
pub const ERROR_BUFFER_OVERFLOW: DWORD = 111;
pub const CMF_NOVERBS: UINT = 0x0000_0008;
pub const CMF_EXTENDEDVERBS: UINT = 0x0000_0100;
pub const MIIM_ID: UINT = 0x0000_0002;
pub const MIIM_TYPE: UINT = 0x0000_0010;
pub const MFT_STRING: UINT = 0;
pub const SEVERITY_SUCCESS: DWORD = 0;

#[inline]
pub const fn MAKE_HRESULT(severity: DWORD, facility: DWORD, code: DWORD) -> HRESULT {
    ((severity << 31) | (facility << 16) | code) as HRESULT
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CMINVOKECOMMANDINFO {
    pub cbSize: DWORD,
    pub fMask: DWORD,
    pub hwnd: HWND,
    pub lpVerb: LPCSTR,
    pub lpParameters: LPCSTR,
    pub lpDirectory: LPCSTR,
    pub nShow: INT,
    pub dwHotKey: DWORD,
    pub hIcon: HANDLE,
}
pub type LPCMINVOKECOMMANDINFO = *mut CMINVOKECOMMANDINFO;

pub use crate::island::MENUITEMINFOW;

/// SIZE and POINT from wtypes.h. RE-EXPORTED from the bindgen island, not declared here:
/// the island owns DATA LAYOUTS (see shellcom.rs's header). A second local copy is a
/// distinct nominal type, so a vtable slot typed from here and a body typed from the
/// island stop agreeing -- and `as *mut _` cannot bridge two structs, it only splits one
/// error into two.
pub use crate::island::{POINT, SIZE};

/// winuser.h notification header. `idFrom` is pointer-sized on Win64.
pub use crate::island::NMHDR;

pub type LPNMHDR = *mut NMHDR;

pub use crate::island::PSHNOTIFY;

/// Canonical shellapi.h file-information records shared by C and C++ ports.
pub use crate::island::SHFILEINFOA;

pub use crate::island::SHFILEINFOW;

extern "system" {
    pub fn lstrcpynW(dst: LPWSTR, src: LPCWSTR, capacity: INT) -> LPWSTR;
}

extern "C" {
    pub fn wcscpy(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
    pub fn wcschr(text: LPCWSTR, character: WCHAR) -> LPWSTR;
    pub fn wcscmp(left: LPCWSTR, right: LPCWSTR) -> INT;
    pub fn wcslen(text: LPCWSTR) -> usize;
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

/// LUID — { LowPart: DWORD, HighPart: LONG }.
pub use crate::island::LUID;

// LARGE_INTEGER: the island owns it (same rule as POINT/SIZE/RECT above). The local copy
// kept only the quad arm.
pub use crate::island::LARGE_INTEGER;

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
