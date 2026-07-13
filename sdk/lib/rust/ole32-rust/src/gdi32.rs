//! gdi32 / user32 import surface for the ring's metafile path
//! (`OleMetafilePictFromIconAndLabel` in ole32_main.c draws an icon + its label into
//! a metafile, so it reaches for a slice of GDI that nothing else in ole32 needs).
//!
//! Same ABI rule as win32.rs: these are all WINAPI, i.e. __stdcall on i386 (`_Foo@N`).
//! Declaring them `extern "C"` links on amd64/arm64 and breaks i386 — so they are
//! declared `extern "system"` here, ONCE, and the ring's local extern blocks are
//! stripped at integration.
//!
//! Constants are cited to their defining header and re-checked by verify_constants.py.
#![allow(clippy::all)]
use crate::prelude::*;
use core::ffi::c_void;

pub type HMETAFILE = *mut c_void; // wingdi.h: DECLARE_HANDLE(HMETAFILE)
pub type HGDIOBJ = *mut c_void;

/// wingdi.h `tagMETAFILEPICT` — { LONG mm; LONG xExt; LONG yExt; HMETAFILE hMF; }
#[repr(C)]
#[derive(Copy, Clone)]
pub struct METAFILEPICT {
    pub mm: LONG,
    pub xExt: LONG,
    pub yExt: LONG,
    pub hMF: HMETAFILE,
}
pub type LPMETAFILEPICT = *mut METAFILEPICT;

// wingdi.h — GetDeviceCaps indices
pub const LOGPIXELSX: c_int = 88;
pub const LOGPIXELSY: c_int = 90;

// winuser.h — GetSystemMetrics indices
pub const SM_CXICON: c_int = 11;
pub const SM_CYICON: c_int = 12;

// winuser.h — SystemParametersInfo action
pub const SPI_GETICONTITLELOGFONT: UINT = 0x001F;

extern "system" {
    // --- device contexts ---
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    pub fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;

    // --- metafiles ---
    pub fn CreateMetaFileW(pszFile: *const u16) -> HDC;
    pub fn CloseMetaFile(hdc: HDC) -> HMETAFILE;
    pub fn DeleteMetaFile(hmf: HMETAFILE) -> BOOL;

    // --- drawing ---
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    pub fn TextOutW(hdc: HDC, x: c_int, y: c_int, lpString: *const u16, c: c_int) -> BOOL;
    pub fn GetTextExtentPoint32W(hdc: HDC, lpString: *const u16, c: c_int, psizl: *mut SIZE) -> BOOL;
    pub fn DrawIcon(hDC: HDC, X: c_int, Y: c_int, hIcon: HICON) -> BOOL;
    pub fn ExtEscape(
        hdc: HDC,
        iEscape: c_int,
        cjInput: c_int,
        lpInData: *const i8,
        cjOutput: c_int,
        lpOutData: *mut i8,
    ) -> c_int;

    // --- mapping mode / extents ---
    pub fn SetMapMode(hdc: HDC, iMode: c_int) -> c_int;
    pub fn SetWindowExtEx(hdc: HDC, x: c_int, y: c_int, lpsz: *mut SIZE) -> BOOL;
    pub fn SetWindowOrgEx(hdc: HDC, x: c_int, y: c_int, lppt: *mut POINT) -> BOOL;

    // --- misc ---
    pub fn GetSystemMetrics(nIndex: c_int) -> c_int;
    pub fn SystemParametersInfoW(
        uiAction: UINT,
        uiParam: UINT,
        pvParam: *mut c_void,
        fWinIni: UINT,
    ) -> BOOL;
    pub fn MulDiv(nNumber: c_int, nNumerator: c_int, nDenominator: c_int) -> c_int;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: *const u16,
        cchWideChar: c_int,
        lpMultiByteStr: *mut i8,
        cbMultiByte: c_int,
        lpDefaultChar: *const i8,
        lpUsedDefaultChar: *mut BOOL,
    ) -> c_int;
}
