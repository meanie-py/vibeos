//! SHGetKnownFolderPath / SHGetKnownFolderIDList / SHGetStockIconInfo (Vista+ exports).
//!
//! Known folders are resolved by walking the CSIDL table in `shellpath` for the entry whose
//! FOLDERID equals the request, then delegating to the CSIDL machinery. Folders with no CSIDL
//! entry (e.g. FOLDERID_Downloads) report ERROR_FILE_NOT_FOUND, as the spec says for unknowns.
//!
//! Stock icons: PARTIAL by design. SIIDs are mapped onto the IDI_SHELL_* icon resources that
//! this shell32 actually contains (drives, folders, network, printers, recycle bin, ...);
//! every SIID without a distinct resource falls back to the generic document icon, and
//! `iIcon` is reported as the negative resource id (the ExtractIcon convention).

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use crate::types::*;
use crate::wine::shellpath::{CSIDL_DATA, SHGetFolderPathW};
use core::ffi::c_void;
use core::ptr::{null_mut, read_unaligned};

const KF_FLAG_CREATE: DWORD = 0x0000_8000;
const KF_FLAG_DONT_VERIFY: DWORD = 0x0000_4000;
const CSIDL_FLAG_CREATE: INT = 0x8000;
const CSIDL_FLAG_DONT_VERIFY: INT = 0x4000;
const SHGFP_TYPE_CURRENT: DWORD = 0;
const ERROR_FILE_NOT_FOUND: LONG = 2;
const ERROR_NOT_ENOUGH_MEMORY: LONG = 8;

extern "system" {
    fn CoTaskMemAlloc(cb: usize) -> *mut c_void;
    fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
    fn LoadImageW(hInst: HINSTANCE, name: LPCWSTR, typ: UINT, cx: INT, cy: INT, fuLoad: UINT) -> *mut c_void;
    fn GetSystemMetrics(nIndex: INT) -> INT;
}
extern "C" {
    fn SIC_GetIconIndex(sSourceFile: LPCWSTR, dwSourceIndex: INT, dwFlags: DWORD) -> INT;
}

#[inline]
unsafe fn guid_eq(a: *const u8, b: *const u8) -> bool {
    read_unaligned(a as *const u64) == read_unaligned(b as *const u64)
        && read_unaligned(a.add(8) as *const u64) == read_unaligned(b.add(8) as *const u64)
}

/// FOLDERID -> CSIDL (first table hit; all-zero GUID_NULL placeholder rows never match).
unsafe fn csidl_from_folderid(rfid: *const c_void) -> Option<INT> {
    let zero = [0u8; 16];
    for (i, e) in CSIDL_DATA.iter().enumerate() {
        let id = e.id as *const u8;
        if guid_eq(id, zero.as_ptr()) {
            continue;
        }
        if guid_eq(id, rfid as *const u8) {
            return Some(i as INT);
        }
    }
    None
}

#[inline]
fn kf_to_csidl_flags(dwFlags: DWORD) -> INT {
    let mut f = 0;
    if dwFlags & KF_FLAG_CREATE != 0 {
        f |= CSIDL_FLAG_CREATE;
    }
    if dwFlags & KF_FLAG_DONT_VERIFY != 0 {
        f |= CSIDL_FLAG_DONT_VERIFY;
    }
    f
}

#[no_mangle]
pub unsafe extern "system" fn SHGetKnownFolderPath(
    rfid: *const c_void,
    dwFlags: DWORD,
    hToken: HANDLE,
    ppszPath: *mut LPWSTR,
) -> HRESULT {
    if ppszPath.is_null() {
        return E_INVALIDARG;
    }
    *ppszPath = null_mut();
    if rfid.is_null() {
        return E_INVALIDARG;
    }
    let csidl = match csidl_from_folderid(rfid) {
        Some(c) => c,
        None => return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND),
    };
    let mut buf = [0u16; MAX_PATH];
    let hr = SHGetFolderPathW(null_mut(), csidl | kf_to_csidl_flags(dwFlags), hToken, SHGFP_TYPE_CURRENT, buf.as_mut_ptr());
    if hr < 0 {
        return hr;
    }
    let mut len = 0usize;
    while len < MAX_PATH && buf[len] != 0 {
        len += 1;
    }
    let out = CoTaskMemAlloc((len + 1) * 2) as LPWSTR;
    if out.is_null() {
        return HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
    }
    core::ptr::copy_nonoverlapping(buf.as_ptr(), out, len);
    *out.add(len) = 0;
    *ppszPath = out;
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn SHGetKnownFolderIDList(
    rfid: *const c_void,
    dwFlags: DWORD,
    hToken: HANDLE,
    ppidl: *mut *mut c_void,
) -> HRESULT {
    if ppidl.is_null() {
        return E_INVALIDARG;
    }
    *ppidl = null_mut();
    if rfid.is_null() {
        return E_INVALIDARG;
    }
    match csidl_from_folderid(rfid) {
        Some(c) => crate::win32::SHGetFolderLocation(null_mut(), c | kf_to_csidl_flags(dwFlags), hToken, 0, ppidl as *mut _),
        None => E_INVALIDARG,
    }
}

#[repr(C)]
pub struct SHSTOCKICONINFO {
    pub cbSize: DWORD,
    pub hIcon: *mut c_void,
    pub iSysImageIndex: INT,
    pub iIcon: INT,
    pub szPath: [WCHAR; MAX_PATH],
}

const SHGSI_ICON: UINT = 0x100;
const SHGSI_SMALLICON: UINT = 0x1;
const SHGSI_SHELLICONSIZE: UINT = 0x4;
const SHGSI_SYSICONINDEX: UINT = 0x4000;
// The island value (our shellapi.h, 175); the Windows 10 SDK adds six more ids (181).
const SIID_MAX_ICONS: INT = crate::island::SHSTOCKICONID_SIID_MAX_ICONS as INT;
const IDI_SHELL_DOCUMENT: INT = 1;

/// SIID -> shell32 icon resource id (IDI_SHELL_*). Unlisted SIIDs use the document icon.
fn stock_icon_resource(siid: INT) -> INT {
    match siid {
        0 | 1 => 1,
        2 => 3,
        3 | 34 | 52 | 53 => 4,
        4 => 5,
        5 => 6,
        6 => 7,
        7 => 8,
        8 | 35 => 9,
        9 => 10,
        10 => 11,
        11 | 24 | 33 | 36..=47 | 62..=70 => 12,
        12 => 13,
        13 => 14,
        14 | 28 => 15,
        15 | 29 => 17,
        16 | 71 | 72 => 18,
        17 => 23,
        18 => 24,
        19 => 29,
        20 => 30,
        22 => 32,
        23 => 33,
        25 => 48,
        27 | 30 => 140,
        31 => 141,
        83 => 148,
        _ => IDI_SHELL_DOCUMENT,
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHGetStockIconInfo(siid: INT, uFlags: UINT, psii: *mut SHSTOCKICONINFO) -> HRESULT {
    if psii.is_null() || (*psii).cbSize as usize != core::mem::size_of::<SHSTOCKICONINFO>() {
        return E_INVALIDARG;
    }
    if siid < 0 || siid >= SIID_MAX_ICONS {
        return E_INVALIDARG;
    }
    let inst = crate::win32::shell32_hInstance;
    (*psii).hIcon = null_mut();
    (*psii).iSysImageIndex = -1;
    (*psii).iIcon = 0;
    core::ptr::write_bytes((*psii).szPath.as_mut_ptr(), 0, MAX_PATH);

    let res = stock_icon_resource(siid);
    GetModuleFileNameW(inst, (*psii).szPath.as_mut_ptr(), MAX_PATH as DWORD);
    (*psii).iIcon = -res;

    if uFlags & SHGSI_ICON != 0 {
        // Small unless the caller asked for large; SHELLICONSIZE is treated as large.
        let small = uFlags & SHGSI_SMALLICON != 0 && uFlags & SHGSI_SHELLICONSIZE == 0;
        let (cx, cy) = if small {
            (GetSystemMetrics(49), GetSystemMetrics(50))
        } else {
            (GetSystemMetrics(11), GetSystemMetrics(12))
        };
        // IMAGE_ICON = 1
        (*psii).hIcon = LoadImageW(inst, res as usize as LPCWSTR, 1, cx, cy, 0);
        if (*psii).hIcon.is_null() {
            return E_FAIL;
        }
    }
    if uFlags & SHGSI_SYSICONINDEX != 0 {
        (*psii).iSysImageIndex = SIC_GetIconIndex((*psii).szPath.as_ptr(), -res, 0);
    }
    S_OK
}
