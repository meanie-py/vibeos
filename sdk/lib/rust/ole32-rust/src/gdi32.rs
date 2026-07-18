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

/// wingdi.h `tagBITMAPINFOHEADER`. WIRE FORMAT (a DIB header is serialized into the
/// presentation-cache stream), so field order/width are exact.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: WORD,
    pub biBitCount: WORD,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}

/// wingdi.h `tagRGBQUAD`
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGBQUAD {
    pub rgbBlue: BYTE,
    pub rgbGreen: BYTE,
    pub rgbRed: BYTE,
    pub rgbReserved: BYTE,
}

/// wingdi.h `tagBITMAPINFO` — header followed by a colour table. The `[RGBQUAD; 1]` is
/// the C's flexible-array idiom: real allocations are larger and are indexed past the end.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}

/// wingdi.h `tagBITMAPCOREHEADER` — the OS/2-style DIB header. WIRE FORMAT.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAPCOREHEADER {
    pub bcSize: DWORD,
    pub bcWidth: WORD,
    pub bcHeight: WORD,
    pub bcPlanes: WORD,
    pub bcBitCount: WORD,
}

/// wingdi.h `tagRGBTRIPLE` — the core-header colour table entry (3 bytes, NOT 4).
#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGBTRIPLE {
    pub rgbtBlue: BYTE,
    pub rgbtGreen: BYTE,
    pub rgbtRed: BYTE,
}

/// wingdi.h `tagBITMAPFILEHEADER`. WIRE FORMAT, and note it is `#pragma pack(2)` in the C
/// (a 2-byte WORD followed by a DWORD): sizeof is 14, NOT 16. `packed` is load-bearing —
/// a natural-aligned version would silently write a 16-byte header and corrupt every DIB.
#[repr(C, packed(2))]
#[derive(Copy, Clone)]
pub struct BITMAPFILEHEADER {
    pub bfType: WORD,
    pub bfSize: DWORD,
    pub bfReserved1: WORD,
    pub bfReserved2: WORD,
    pub bfOffBits: DWORD,
}

/// wingdi.h `tagENHMETAHEADER`. datacache.c only ever takes `sizeof(ENHMETAHEADER)` of
/// this, so the SIZE is what matters — every field must be present. ole32 builds with
/// _WIN32_WINNT=0x600, so both the `WINVER >= 0x0400` and `>= 0x0500` blocks are IN.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ENHMETAHEADER {
    pub iType: DWORD,
    pub nSize: DWORD,
    pub rclBounds: RECTL,
    pub rclFrame: RECTL,
    pub dSignature: DWORD,
    pub nVersion: DWORD,
    pub nBytes: DWORD,
    pub nRecords: DWORD,
    pub nHandles: WORD,
    pub sReserved: WORD,
    pub nDescription: DWORD,
    pub offDescription: DWORD,
    pub nPalEntries: DWORD,
    pub szlDevice: SIZEL,
    pub szlMillimeters: SIZEL,
    pub cbPixelFormat: DWORD,
    pub offPixelFormat: DWORD,
    pub bOpenGL: DWORD,
    pub szlMicrometers: SIZEL,
}

/// wine/wingdi16.h `METAFILEPICT16` — the 16-bit-era metafile picture header that still
/// prefixes an OlePres stream on disk. storage32.c reads `sizeof(METAFILEPICT16)` (= 10)
/// off the front of the data, so the 16-bit widths are WIRE FORMAT, not a portability bug.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct METAFILEPICT16 {
    pub mm: i16,
    pub xExt: i16,
    pub yExt: i16,
    /// HMETAFILE16 — a 16-bit handle.
    pub hMF: u16,
}

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

    // ---- Tier-D (datacache's presentation-cache rendering) additions ----
    pub fn CreateDIBitmap(
        hdc: HDC,
        pbmih: *const BITMAPINFOHEADER,
        flInit: DWORD,
        pjBits: *const c_void,
        pbmi: *const BITMAPINFO,
        iUsage: UINT,
    ) -> HBITMAP;
    pub fn GetDIBits(
        hdc: HDC,
        hbm: HBITMAP,
        start: UINT,
        cLines: UINT,
        lpvBits: *mut c_void,
        lpbmi: *mut BITMAPINFO,
        usage: UINT,
    ) -> c_int;
    pub fn StretchDIBits(
        hdc: HDC,
        xDest: c_int,
        yDest: c_int,
        DestWidth: c_int,
        DestHeight: c_int,
        xSrc: c_int,
        ySrc: c_int,
        SrcWidth: c_int,
        SrcHeight: c_int,
        lpBits: *const c_void,
        lpbmi: *const BITMAPINFO,
        iUsage: UINT,
        rop: DWORD,
    ) -> c_int;
    pub fn PlayMetaFile(hdc: HDC, hmf: HMETAFILE) -> BOOL;
    pub fn SetMetaFileBitsEx(cbBuffer: UINT, lpData: *const BYTE) -> HMETAFILE;
    pub fn GetWinMetaFileBits(
        hemf: HENHMETAFILE,
        cbData16: UINT,
        pData16: *mut BYTE,
        iMapMode: c_int,
        hdcRef: HDC,
    ) -> UINT;
    pub fn SetWinMetaFileBits(
        nSize: UINT,
        lpMeta16Data: *const BYTE,
        hdcRef: HDC,
        lpMFP: *const METAFILEPICT,
    ) -> HENHMETAFILE;
    pub fn SetViewportExtEx(hdc: HDC, x: c_int, y: c_int, lpsz: *mut SIZE) -> BOOL;
    pub fn SetViewportOrgEx(hdc: HDC, x: c_int, y: c_int, lppt: *mut POINT) -> BOOL;
}
