//! win32ss/gdi/gdi32/misc/misc.c → Rust. Miscellaneous gdi32 client functions + the GDI
//! handle-table substrate ROOT: this file OWNS (defines) the cross-TU globals
//! `GdiHandleTable` / `GdiSharedHandleTable` / `CurrentProcessId` / `GDI_BatchLimit`, plus the
//! handle-validation primitives (`GdiValidateHandle` / `GdiFixUpHandle` / `GdiQueryTable` /
//! `GdiGetHandleUserData` / `GdiGetLDC` / `GdiSetLDC`) the rest of gdi32-rust links against.
//! Cohab staticlib → gdi32.dll with misc.c commented out. See gdi32p.h/ntgdihdl.h.
//!
//! NEEDS-TYPE: `GDI_DRAW_STREAM` (undocgdi.h) is not in win32k_sys; defined locally below.
//! NEEDS-IMPORT: the cross-TU gdi32/kernel32 exports this file calls (AbortDoc, StartDocA,
//!   BitBlt, HeapAlloc, MultiByteToWideChar, …) are declared `extern "system"` below; they
//!   resolve against the surviving C objects of gdi32.dll / the kernel32 importlib.
//! NEEDS-IMPORT (central): this file globs `crate::types::*` + `win32k_sys::win32k::*` per the
//!   contract (matching sibling dllmain.rs/gdiobj.rs). Those two surfaces both alias BOOL/DWORD/
//!   etc.; central integration reconciles `types.rs` so the globs are unambiguous.

use win32k_sys::win32k::*;
use crate::gdihelp::*;
use crate::dcattr_f;

// ── Local constants not present in the win32k_sys bindgen surface ──────────────
// (Locally-defined items shadow any same-named win32k_sys glob import — no ambiguity.)
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

// wingdi.h Escape() function codes
const ABORTDOC: INT = 2;
const SETCOLORTABLE: INT = 4;
const GETCOLORTABLE: INT = 5;
const FLUSHOUTPUT: INT = 6;
const DRAFTMODE: INT = 7;
const SETABORTPROC: INT = 9;
const STARTDOC: INT = 10;
const ENDDOC: INT = 11;
const GETSCALINGFACTOR: INT = 14;
const GETEXTENDEDTEXTMETRICS: INT = 256;

// wingdi.h Escape() return codes
const SP_ERROR: INT = -1;
const SP_OUTOFDISK: INT = -4;
const SP_OUTOFMEMORY: INT = -5;
const SP_USERABORT: INT = -3;

// winerror.h (DWORD-typed: passed to *SetLastError(DWORD))
const ERROR_INVALID_HANDLE: DWORD = 6;
const ERROR_CALL_NOT_IMPLEMENTED: DWORD = 120;
// winerror.h (INT-typed: matched against an INT GetLastError() value)
const ERROR_NOT_ENOUGH_MEMORY: INT = 8;
const ERROR_PRINT_CANCELLED: INT = 63;
const ERROR_DISK_FULL: INT = 112;

// misc.c local #defines / enums
const ALPHABLEND_NONE: INT = 0;
const ALPHABLEND_BINARY: INT = 1;
const ALPHABLEND_FULL: INT = 2;
const ST_TRUESIZE: INT = 0;
const ST_STRETCH: INT = 1;
const ST_TILE: INT = 2;

// undocgdi.h DS_ flags
const DS_TILE: DWORD = 0x2;
const DS_TRANSPARENTALPHA: DWORD = 0x4;
const DS_TRANSPARENTCLR: DWORD = 0x8;
const DS_TRUESIZE: DWORD = 0x20;

// wingdi.h / winnls.h / winerror.h misc
const SRCCOPY: DWORD = 0x00CC0020;
const HALFTONE: INT = 4;
const CCHDEVICENAME: INT = 32;
const CCHFORMNAME: INT = 32;
const CP_ACP: UINT = 0;
const DM_FORMNAME: DWORD = 0x00010000;
const S_OK: HRESULT = 0;
const FACILITY_WIN32: DWORD = 7;

// gdi32p.h / ntgdihdl.h
const SAPCALLBACKDELAY: DWORD = 244;
const GDI_BATCH_LIMIT: DWORD = 20;
const LDC_META_PRINT: ULONG = 0x00020000;

// ── GDI_DRAW_STREAM (undocgdi.h) — not in win32k_sys; reproduced locally ───────
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GDI_DRAW_STREAM {
    pub signature: DWORD, // must be 0x44727753 ("Swrd")
    pub reserved: DWORD,  // must be 0
    pub hDC: HDC,
    pub rcDest: RECT,
    pub unknown1: DWORD, // must be 1
    pub hImage: HBITMAP,
    pub unknown2: DWORD, // must be 9
    pub rcClip: RECT,
    pub rcSrc: RECT,
    pub drawOption: DWORD, // DS_ flags
    pub leftSizingMargin: DWORD,
    pub rightSizingMargin: DWORD,
    pub topSizingMargin: DWORD,
    pub bottomSizingMargin: DWORD,
    pub crTransparent: DWORD,
}
pub type PGDI_DRAW_STREAM = *mut GDI_DRAW_STREAM;

// misc.c local _MARGINS
#[repr(C)]
#[derive(Copy, Clone)]
struct MARGINS {
    cxLeftWidth: INT,
    cxRightWidth: INT,
    cyTopHeight: INT,
    cyBottomHeight: INT,
}

// ── Substrate globals OWNED by this file (misc.c lines 33-36) ──────────────────
#[no_mangle]
pub static mut GdiHandleTable: PGDI_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut GdiSharedHandleTable: PGDI_SHARED_HANDLE_TABLE = core::ptr::null_mut();
#[no_mangle]
pub static mut CurrentProcessId: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut GDI_BatchLimit: DWORD = 1;

// extern PGDIHANDLECACHE GdiHandleCache; (defined in dllmain.rs)
extern "C" {
    static mut GdiHandleCache: PGDIHANDLECACHE;
}

// ── WoW64 shared-table entry width (i386 gdi32 under a 64-bit kernel) ─────────
// The 64-bit kernel writes 24-byte entries into the shared GDI handle table:
// same field ORDER as GDI_TABLE_ENTRY, wide KernelData/UserData. Detected once
// in GdiProcessSetup (ProcessWow64Information != 0). UserData (DC_ATTR etc.) is
// mapped below 4GB for WoW64 processes by win32k, so the truncation is
// lossless; KernelData is unusable from user mode either way.
#[cfg(target_arch = "x86")]
#[repr(C)]
struct GDI_TABLE_ENTRY64 {
    KernelData: u64,
    ProcessId: u32,
    Type: i32,
    UserData: u64,
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub static mut g_GdiWow64: BOOL = 0;

/// Normalized (Type, ProcessId, UserData) read of shared-table entry `Index`,
/// honoring the WoW64 wide layout when `g_GdiWow64` is set.
#[inline]
pub(crate) unsafe fn GdiReadSharedEntry(Index: usize) -> (LONG, usize, PVOID) {
    #[cfg(target_arch = "x86")]
    if g_GdiWow64 != 0 {
        let e = (GdiHandleTable as *const GDI_TABLE_ENTRY64).add(Index);
        return ((*e).Type, (*e).ProcessId as usize, (*e).UserData as usize as PVOID);
    }
    let e = GdiHandleTable.add(Index);
    ((*e).__bindgen_anon_1.Type, (*e).ProcessId as usize, (*e).UserData)
}

// ── Cross-TU gdi32 / kernel32 exports (WINAPI = stdcall on i386) ───────────────
extern "system" {
    // gdi32.dll (surviving C objects)
    fn AbortDoc(hdc: HDC) -> INT;
    fn EndDoc(hdc: HDC) -> INT;
    fn StartDocA(hdc: HDC, lpdi: *const DOCINFOA) -> INT;
    fn GetSystemPaletteEntries(hdc: HDC, iStart: UINT, cEntries: UINT, pPalEntries: LPPALETTEENTRY) -> UINT;
    fn SetAbortProc(hdc: HDC, proc_: ABORTPROC) -> INT;
    fn CancelDC(hdc: HDC) -> BOOL;
    fn SetStretchBltMode(hdc: HDC, mode: INT) -> INT;
    fn SetBrushOrgEx(hdc: HDC, x: INT, y: INT, lppt: LPPOINT) -> BOOL;
    fn StretchBlt(hdcDest: HDC, xDest: INT, yDest: INT, wDest: INT, hDest: INT,
                  hdcSrc: HDC, xSrc: INT, ySrc: INT, wSrc: INT, hSrc: INT, rop: DWORD) -> BOOL;
    fn GdiTransparentBlt(hdcDst: HDC, xDst: INT, yDst: INT, wDst: INT, hDst: INT,
                         hdcSrc: HDC, xSrc: INT, ySrc: INT, wSrc: INT, hSrc: INT, crTransparent: UINT) -> BOOL;
    fn GdiAlphaBlend(hdcDst: HDC, xDst: INT, yDst: INT, wDst: INT, hDst: INT,
                     hdcSrc: HDC, xSrc: INT, ySrc: INT, wSrc: INT, hSrc: INT, blend: BLENDFUNCTION) -> BOOL;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn CreateCompatibleBitmap(hdc: HDC, cx: INT, cy: INT) -> HBITMAP;
    fn CreateBitmap(nWidth: INT, nHeight: INT, nPlanes: UINT, nBitCount: UINT, lpBits: *const core::ffi::c_void) -> HBITMAP;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn BitBlt(hdc: HDC, x: INT, y: INT, cx: INT, cy: INT, hdcSrc: HDC, x1: INT, y1: INT, rop: DWORD) -> BOOL;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn OffsetViewportOrgEx(hdc: HDC, x: INT, y: INT, lppt: LPPOINT) -> BOOL;
    fn SetViewportOrgEx(hdc: HDC, x: INT, y: INT, lppt: LPPOINT) -> BOOL;
    fn GetETM(hdc: HDC, petm: *mut EXTTEXTMETRIC) -> BOOL;
    // kernel32.dll
    fn SetLastError(dwErrCode: DWORD);
    fn GetLastError() -> DWORD;
    fn GetTickCount() -> DWORD;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn GetProcessHeap() -> HANDLE;
    fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT,
                           lpWideCharStr: LPWSTR, cchWideChar: INT) -> INT;
}

// CRT (cdecl) — resolves via the C runtime / ntdll
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
    // gdi32 metafile glue (gdi32p.h: plain cdecl, no WINAPI → `_METADC_ExtEscape`, not @24)
    fn METADC_ExtEscape(hdc: HDC, escape: INT, input_size: INT, input: LPCSTR, output_size: INT, output: LPVOID) -> INT;
}

// ── Interlocked* pointer ops (i386 = kernel32 import, amd64/arm64 = intrinsic with
//    no link symbol → portable core-atomic impl, per the cohab contract) ─────────
#[inline]
unsafe fn InterlockedCompareExchangePointer(dest: *mut PVOID, exchange: PVOID, comparand: PVOID) -> PVOID {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    match a.compare_exchange(
        comparand,
        exchange,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}

#[inline]
unsafe fn InterlockedExchangePointer(dest: *mut PVOID, value: PVOID) -> PVOID {
    let a = &*(dest as *const core::sync::atomic::AtomicPtr<core::ffi::c_void>);
    a.swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ── small helpers ─────────────────────────────────────────────────────────────
/// `HRESULT_FROM_WIN32` (winerror.h __HRESULT_FROM_WIN32 inline form — single eval).
#[inline]
fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if (x as HRESULT) <= 0 {
        x as HRESULT
    } else {
        ((x & 0x0000FFFF) | (FACILITY_WIN32 << 16) | 0x80000000) as HRESULT
    }
}

/// `abs()` for INT — C `abs(INT_MIN)` is UB; we wrap (never panics in debug).
#[inline]
fn abs(x: INT) -> INT {
    x.wrapping_abs()
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiFlush() -> BOOL {
    NtGdiFlush();
    TRUE
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn Escape(
    hdc: HDC,
    nEscape: INT,
    cbInput: INT,
    lpvInData: LPCSTR,
    lpvOutData: LPVOID,
) -> INT {
    let mut retValue: INT = SP_ERROR;
    let ulObjType: ULONG;

    ulObjType = GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) as ULONG;

    if ulObjType == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as ULONG {
        return METADC_ExtEscape(hdc, nEscape, cbInput, lpvInData, 0, lpvOutData);
    }

    match nEscape {
        ABORTDOC => {
            /* Note: Windows checks if the handle has any user data for the ABORTDOC command
             * ReactOS copies this behavior to be compatible with windows 2003 */
            if GdiGetDcAttr(hdc).is_null() {
                GdiSetLastError(ERROR_INVALID_HANDLE);
                retValue = FALSE;
            } else {
                retValue = AbortDoc(hdc);
            }
        }

        DRAFTMODE | FLUSHOUTPUT | SETCOLORTABLE => {
            /* Note 1: DRAFTMODE, FLUSHOUTPUT, SETCOLORTABLE are outdated */
            /* Note 2: Windows checks if the handle has any user data ... (win2003 compat) */
            if GdiGetDcAttr(hdc).is_null() {
                GdiSetLastError(ERROR_INVALID_HANDLE);
            }
            retValue = FALSE;
        }

        SETABORTPROC => {
            /* Note: Windows checks if the handle has any user data ... (win2003 compat) */
            if GdiGetDcAttr(hdc).is_null() {
                GdiSetLastError(ERROR_INVALID_HANDLE);
                retValue = FALSE;
            }
            retValue = SetAbortProc(hdc, core::mem::transmute::<LPCSTR, ABORTPROC>(lpvInData));
        }

        GETCOLORTABLE => {
            retValue = GetSystemPaletteEntries(hdc, (*lpvInData) as UINT, 1, lpvOutData as LPPALETTEENTRY) as INT;
            if retValue == 0 {
                retValue = SP_ERROR;
            }
        }

        ENDDOC => {
            /* Note: Windows checks if the handle has any user data ... (win2003 compat) */
            if GdiGetDcAttr(hdc).is_null() {
                GdiSetLastError(ERROR_INVALID_HANDLE);
                retValue = FALSE;
            }
            retValue = EndDoc(hdc);
        }

        GETSCALINGFACTOR => {
            /* Note GETSCALINGFACTOR is outdated, replaced by GetDeviceCaps */
            if ulObjType == GDI_OBJECT_TYPE_DC {
                if !lpvOutData.is_null() {
                    let ptr = lpvOutData as PPOINT;
                    (*ptr).x = 0;
                    (*ptr).y = 0;
                }
            }
            retValue = FALSE;
        }

        GETEXTENDEDTEXTMETRICS => {
            retValue = (GetETM(hdc, lpvOutData as *mut EXTTEXTMETRIC) != 0) as INT;
        }

        STARTDOC => {
            let mut di: DOCINFOA = core::mem::zeroed();

            /* Note: Windows checks if the handle has any user data ... (win2003 compat) */
            if GdiGetDcAttr(hdc).is_null() {
                GdiSetLastError(ERROR_INVALID_HANDLE);
                retValue = FALSE;
            }

            di.cbSize = core::mem::size_of::<DOCINFOA>() as INT;
            di.lpszOutput = core::ptr::null();
            di.lpszDatatype = core::ptr::null();
            di.fwType = 0;
            di.lpszDocName = lpvInData;

            /* NOTE: doc for StartDocA/W at msdn nf-wingdi-startdoca */
            retValue = StartDocA(hdc, &di);

            /* Check if StartDocA failed */
            if retValue < 0 {
                retValue = GetLastError() as INT;

                /* Translate StartDocA error code to STARTDOC error code (nf-wingdi-escape) */
                retValue = match retValue {
                    ERROR_NOT_ENOUGH_MEMORY => SP_OUTOFMEMORY,
                    ERROR_PRINT_CANCELLED => SP_USERABORT,
                    ERROR_DISK_FULL => SP_OUTOFDISK,
                    _ => SP_ERROR,
                };
            }
        }

        _ => {
            // UNIMPLEMENTED;
            SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        }
    }

    retValue
}

#[no_mangle]
pub unsafe extern "system" fn ExtEscape(
    hDC: HDC,
    nEscape: INT,
    cbInput: INT,
    lpszInData: LPCSTR,
    cbOutput: INT,
    lpszOutData: LPSTR,
) -> INT {
    NtGdiExtEscape(hDC, core::ptr::null_mut(), 0, nEscape, cbInput, lpszInData as LPSTR, cbOutput, lpszOutData)
}

#[no_mangle]
pub unsafe extern "system" fn NamedEscape(
    hdc: HDC,
    pDriver: PWCHAR,
    iEsc: INT,
    cjIn: INT,
    pjIn: LPSTR,
    cjOut: INT,
    pjOut: LPSTR,
) -> INT {
    /* FIXME metadc done mostly in user mode, unsupported.
     * Windows 2000/XP/Vista ignore the passed hdc and always set hdc to NULL when calling
     * NtGdiExtEscape from NamedEscape */
    let _ = hdc;
    NtGdiExtEscape(core::ptr::null_mut(), pDriver, wcslen(pDriver as *const WCHAR) as INT, iEsc, cjIn, pjIn, cjOut, pjOut)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DrawEscape(
    hDC: HDC,
    nEscape: INT,
    cbInput: INT,
    lpszInData: LPCSTR,
) -> INT {
    if GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) == GDI_OBJECT_TYPE_DC as usize {
        return NtGdiDrawEscape(hDC, nEscape, cbInput, lpszInData as LPSTR);
    }

    if GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) != GDI_OBJECT_TYPE_METADC as usize {
        let pLDC: PLDC = GdiGetLDC(hDC);
        if !pLDC.is_null() {
            if (*pLDC).Flags & LDC_META_PRINT != 0 {
                //  if (nEscape != QUERYESCSUPPORT)
                //     return EMFDRV_WriteEscape(hDC, nEscape, cbInput, lpszInData, EMR_DRAWESCAPE);
                return NtGdiDrawEscape(hDC, nEscape, cbInput, lpszInData as LPSTR);
            }
        }
        SetLastError(ERROR_INVALID_HANDLE);
    }
    0
}

/***********************************************************************
 *      UXTHEME_StretchBlt — Pseudo TransparentBlt/StretchBlt
 */
unsafe fn UXTHEME_StretchBlt(
    hdcDst: HDC,
    nXOriginDst: INT,
    nYOriginDst: INT,
    nWidthDst: INT,
    nHeightDst: INT,
    hdcSrc: HDC,
    nXOriginSrc: INT,
    nYOriginSrc: INT,
    nWidthSrc: INT,
    nHeightSrc: INT,
    transparent: INT,
    transcolor: COLORREF,
) -> BOOL {
    let blendFunc = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER as BYTE,   /* BlendOp */
        BlendFlags: 0,                  /* BlendFlag */
        SourceConstantAlpha: 255,       /* SourceConstantAlpha */
        AlphaFormat: AC_SRC_ALPHA as BYTE, /* AlphaFormat */
    };

    let mut ret: BOOL = TRUE;
    let old_stretch_mode: INT;
    let mut old_brush_org: POINT = core::mem::zeroed();

    old_stretch_mode = SetStretchBltMode(hdcDst, HALFTONE);
    SetBrushOrgEx(hdcDst, nXOriginDst, nYOriginDst, &mut old_brush_org);

    if transparent == ALPHABLEND_BINARY {
        /* Ensure we don't pass any negative values to TransparentBlt */
        ret = GdiTransparentBlt(
            hdcDst, nXOriginDst, nYOriginDst, abs(nWidthDst), abs(nHeightDst),
            hdcSrc, nXOriginSrc, nYOriginSrc, abs(nWidthSrc), abs(nHeightSrc),
            transcolor,
        );
    } else if (transparent == ALPHABLEND_NONE)
        || GdiAlphaBlend(
            hdcDst, nXOriginDst, nYOriginDst, nWidthDst, nHeightDst,
            hdcSrc, nXOriginSrc, nYOriginSrc, nWidthSrc, nHeightSrc,
            blendFunc,
        ) == 0
    {
        ret = StretchBlt(
            hdcDst, nXOriginDst, nYOriginDst, nWidthDst, nHeightDst,
            hdcSrc, nXOriginSrc, nYOriginSrc, nWidthSrc, nHeightSrc,
            SRCCOPY,
        );
    }

    SetBrushOrgEx(hdcDst, old_brush_org.x, old_brush_org.y, core::ptr::null_mut());
    SetStretchBltMode(hdcDst, old_stretch_mode);

    ret
}

/***********************************************************************
 *      UXTHEME_Blt — same width/height for both source and dest
 */
unsafe fn UXTHEME_Blt(
    hdcDest: HDC,
    nXOriginDest: INT,
    nYOriginDest: INT,
    nWidthDest: INT,
    nHeightDest: INT,
    hdcSrc: HDC,
    nXOriginSrc: INT,
    nYOriginSrc: INT,
    transparent: INT,
    transcolor: COLORREF,
) -> BOOL {
    UXTHEME_StretchBlt(
        hdcDest, nXOriginDest, nYOriginDest, nWidthDest, nHeightDest,
        hdcSrc, nXOriginSrc, nYOriginSrc, nWidthDest, nHeightDest,
        transparent, transcolor,
    )
}

/***********************************************************************
 *      UXTHEME_SizedBlt — Stretches or tiles, depending on sizingtype.
 */
unsafe fn UXTHEME_SizedBlt(
    hdcDst: HDC,
    nXOriginDst: INT,
    nYOriginDst: INT,
    nWidthDst: INT,
    nHeightDst: INT,
    hdcSrc: HDC,
    nXOriginSrc: INT,
    nYOriginSrc: INT,
    nWidthSrc: INT,
    nHeightSrc: INT,
    sizingtype: INT,
    transparent: INT,
    transcolor: COLORREF,
) -> BOOL {
    if sizingtype == ST_TILE {
        let hdcTemp: HDC;
        let mut result: BOOL = FALSE;

        if nWidthSrc == 0 || nHeightSrc == 0 {
            return TRUE;
        }

        /* For destination width/height <= source, skip the memory bitmap optimization */
        if nWidthSrc >= nWidthDst && nHeightSrc >= nHeightDst {
            let bltWidth = core::cmp::min(nWidthDst, nWidthSrc);
            let bltHeight = core::cmp::min(nHeightDst, nHeightSrc);

            return UXTHEME_Blt(
                hdcDst, nXOriginDst, nYOriginDst, bltWidth, bltHeight,
                hdcSrc, nXOriginSrc, nYOriginSrc,
                transparent, transcolor,
            );
        }

        /* Create a DC with a bitmap tiling of the source bitmap (faster than iterating Blt). */
        hdcTemp = CreateCompatibleDC(hdcSrc);
        if !hdcTemp.is_null() {
            let nWidthTemp: INT;
            let nHeightTemp: INT;
            let mut xOfs: INT;
            let mut xRemaining: INT;
            let mut yOfs: INT;
            let mut yRemaining: INT;
            let mut growSize: INT;

            /* Calculate temp dimensions of integer multiples of source dimensions */
            nWidthTemp = ((nWidthDst + nWidthSrc - 1) / nWidthSrc) * nWidthSrc;
            nHeightTemp = ((nHeightDst + nHeightSrc - 1) / nHeightSrc) * nHeightSrc;
            let bitmapTemp: HBITMAP = CreateCompatibleBitmap(hdcSrc, nWidthTemp, nHeightTemp);
            let bitmapOrig: HBITMAP = SelectObject(hdcTemp, bitmapTemp as HGDIOBJ) as HBITMAP;

            /* Initial copy of bitmap */
            BitBlt(hdcTemp, 0, 0, nWidthSrc, nHeightSrc, hdcSrc, nXOriginSrc, nYOriginSrc, SRCCOPY);

            /* Extend bitmap in the X direction. Growth of width is exponential */
            xOfs = nWidthSrc;
            xRemaining = nWidthTemp - nWidthSrc;
            growSize = nWidthSrc;
            while xRemaining > 0 {
                growSize = core::cmp::min(growSize, xRemaining);
                BitBlt(hdcTemp, xOfs, 0, growSize, nHeightSrc, hdcTemp, 0, 0, SRCCOPY);
                xOfs += growSize;
                xRemaining -= growSize;
                growSize *= 2;
            }

            /* Extend bitmap in the Y direction. Growth of height is exponential */
            yOfs = nHeightSrc;
            yRemaining = nHeightTemp - nHeightSrc;
            growSize = nHeightSrc;
            while yRemaining > 0 {
                growSize = core::cmp::min(growSize, yRemaining);
                BitBlt(hdcTemp, 0, yOfs, nWidthTemp, growSize, hdcTemp, 0, 0, SRCCOPY);
                yOfs += growSize;
                yRemaining -= growSize;
                growSize *= 2;
            }

            /* Use temporary hdc for source */
            result = UXTHEME_Blt(
                hdcDst, nXOriginDst, nYOriginDst, nWidthDst, nHeightDst,
                hdcTemp, 0, 0,
                transparent, transcolor,
            );

            SelectObject(hdcTemp, bitmapOrig as HGDIOBJ);
            DeleteObject(bitmapTemp as HGDIOBJ);
        }
        DeleteDC(hdcTemp);
        result
    } else {
        UXTHEME_StretchBlt(
            hdcDst, nXOriginDst, nYOriginDst, nWidthDst, nHeightDst,
            hdcSrc, nXOriginSrc, nYOriginSrc, nWidthSrc, nHeightSrc,
            transparent, transcolor,
        )
    }
}

/***********************************************************************
 *      UXTHEME_DrawImageBackground — Draw an imagefile background
 */
unsafe fn UXTHEME_DrawImageBackground(
    hdc: HDC,
    bmpSrc: HBITMAP,
    prcSrc: *mut RECT,
    transparent: INT,
    transparentcolor: COLORREF,
    borderonly: BOOL,
    sizingtype: INT,
    psm: *mut MARGINS,
    pRect: *mut RECT,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    /* The Wine resize path (#ifndef __REACTOS__) is reverted here, so these stay NULL. */
    let bmpSrcResized: HBITMAP = core::ptr::null_mut();
    let oldSrc: HGDIOBJ;
    let hdcSrc: HDC;
    let hdcOrigSrc: HDC = core::ptr::null_mut();
    let rcDst: RECT = *pRect;
    let rcSrc: RECT = *prcSrc;
    let sm: MARGINS = *psm;
    let mut dstSize: POINT = core::mem::zeroed();
    let srcSize: POINT;

    hdcSrc = CreateCompatibleDC(hdc);
    if hdcSrc.is_null() {
        hr = HRESULT_FROM_WIN32(GetLastError());
        return hr;
    }
    oldSrc = SelectObject(hdcSrc, bmpSrc as HGDIOBJ);

    dstSize.x = rcDst.right - rcDst.left;
    dstSize.y = rcDst.bottom - rcDst.top;
    srcSize = POINT {
        x: rcSrc.right - rcSrc.left,
        y: rcSrc.bottom - rcSrc.top,
    };

    if sizingtype == ST_TRUESIZE {
        if UXTHEME_StretchBlt(
            hdc, rcDst.left, rcDst.top, dstSize.x, dstSize.y,
            hdcSrc, rcSrc.left, rcSrc.top, srcSize.x, srcSize.y,
            transparent, transparentcolor,
        ) == 0
        {
            hr = HRESULT_FROM_WIN32(GetLastError());
        }
    } else {
        let hdcDst: HDC;
        let mut org: POINT = core::mem::zeroed();

        dstSize.x = abs(dstSize.x);
        dstSize.y = abs(dstSize.y);

        /* Resize source image if destination smaller than margins:
         * #ifndef __REACTOS__ block omitted (reverts Wine 2b650fa; CORE-9636). */

        hdcDst = hdc;
        OffsetViewportOrgEx(hdcDst, rcDst.left, rcDst.top, &mut org);

        'draw_error: {
            /* Upper left corner */
            if UXTHEME_Blt(
                hdcDst, 0, 0, sm.cxLeftWidth, sm.cyTopHeight,
                hdcSrc, rcSrc.left, rcSrc.top,
                transparent, transparentcolor,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'draw_error;
            }
            /* Upper right corner */
            if UXTHEME_Blt(
                hdcDst, dstSize.x - sm.cxRightWidth, 0, sm.cxRightWidth, sm.cyTopHeight,
                hdcSrc, rcSrc.right - sm.cxRightWidth, rcSrc.top,
                transparent, transparentcolor,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'draw_error;
            }
            /* Lower left corner */
            if UXTHEME_Blt(
                hdcDst, 0, dstSize.y - sm.cyBottomHeight, sm.cxLeftWidth, sm.cyBottomHeight,
                hdcSrc, rcSrc.left, rcSrc.bottom - sm.cyBottomHeight,
                transparent, transparentcolor,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'draw_error;
            }
            /* Lower right corner */
            if UXTHEME_Blt(
                hdcDst, dstSize.x - sm.cxRightWidth, dstSize.y - sm.cyBottomHeight,
                sm.cxRightWidth, sm.cyBottomHeight,
                hdcSrc, rcSrc.right - sm.cxRightWidth, rcSrc.bottom - sm.cyBottomHeight,
                transparent, transparentcolor,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32(GetLastError());
                break 'draw_error;
            }

            if (sizingtype == ST_STRETCH) || (sizingtype == ST_TILE) {
                let destCenterWidth = dstSize.x - (sm.cxLeftWidth + sm.cxRightWidth);
                let srcCenterWidth = srcSize.x - (sm.cxLeftWidth + sm.cxRightWidth);
                let destCenterHeight = dstSize.y - (sm.cyTopHeight + sm.cyBottomHeight);
                let srcCenterHeight = srcSize.y - (sm.cyTopHeight + sm.cyBottomHeight);

                if destCenterWidth > 0 {
                    /* Center top */
                    if UXTHEME_SizedBlt(
                        hdcDst, sm.cxLeftWidth, 0, destCenterWidth, sm.cyTopHeight,
                        hdcSrc, rcSrc.left + sm.cxLeftWidth, rcSrc.top, srcCenterWidth, sm.cyTopHeight,
                        sizingtype, transparent, transparentcolor,
                    ) == 0
                    {
                        hr = HRESULT_FROM_WIN32(GetLastError());
                        break 'draw_error;
                    }
                    /* Center bottom */
                    if UXTHEME_SizedBlt(
                        hdcDst, sm.cxLeftWidth, dstSize.y - sm.cyBottomHeight, destCenterWidth, sm.cyBottomHeight,
                        hdcSrc, rcSrc.left + sm.cxLeftWidth, rcSrc.bottom - sm.cyBottomHeight, srcCenterWidth, sm.cyBottomHeight,
                        sizingtype, transparent, transparentcolor,
                    ) == 0
                    {
                        hr = HRESULT_FROM_WIN32(GetLastError());
                        break 'draw_error;
                    }
                }
                if destCenterHeight > 0 {
                    /* Left center */
                    if UXTHEME_SizedBlt(
                        hdcDst, 0, sm.cyTopHeight, sm.cxLeftWidth, destCenterHeight,
                        hdcSrc, rcSrc.left, rcSrc.top + sm.cyTopHeight, sm.cxLeftWidth, srcCenterHeight,
                        sizingtype, transparent, transparentcolor,
                    ) == 0
                    {
                        hr = HRESULT_FROM_WIN32(GetLastError());
                        break 'draw_error;
                    }
                    /* Right center */
                    if UXTHEME_SizedBlt(
                        hdcDst, dstSize.x - sm.cxRightWidth, sm.cyTopHeight, sm.cxRightWidth, destCenterHeight,
                        hdcSrc, rcSrc.right - sm.cxRightWidth, rcSrc.top + sm.cyTopHeight, sm.cxRightWidth, srcCenterHeight,
                        sizingtype, transparent, transparentcolor,
                    ) == 0
                    {
                        hr = HRESULT_FROM_WIN32(GetLastError());
                        break 'draw_error;
                    }
                }
                if destCenterHeight > 0 && destCenterWidth > 0 {
                    if borderonly == 0 {
                        /* Center */
                        if UXTHEME_SizedBlt(
                            hdcDst, sm.cxLeftWidth, sm.cyTopHeight, destCenterWidth, destCenterHeight,
                            hdcSrc, rcSrc.left + sm.cxLeftWidth, rcSrc.top + sm.cyTopHeight, srcCenterWidth, srcCenterHeight,
                            sizingtype, transparent, transparentcolor,
                        ) == 0
                        {
                            hr = HRESULT_FROM_WIN32(GetLastError());
                            break 'draw_error;
                        }
                    }
                }
            }
        } // draw_error:

        SetViewportOrgEx(hdcDst, org.x, org.y, core::ptr::null_mut());
    }
    SelectObject(hdcSrc, oldSrc);
    DeleteDC(hdcSrc);
    if !bmpSrcResized.is_null() {
        DeleteObject(bmpSrcResized as HGDIOBJ);
    }
    if !hdcOrigSrc.is_null() {
        DeleteDC(hdcOrigSrc);
    }
    *pRect = rcDst;
    hr
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiDrawStream(dc: HDC, l: ULONG, pDS: PGDI_DRAW_STREAM) -> BOOL {
    let _ = dc;
    if pDS.is_null() || l != core::mem::size_of::<GDI_DRAW_STREAM>() as ULONG {
        // DPRINT1("GdiDrawStream: Invalid params\n");
        return 0;
    }

    if (*pDS).signature != 0x44727753
        || (*pDS).reserved != 0
        || (*pDS).unknown1 != 1
        || (*pDS).unknown2 != 9
    {
        // DPRINT1("GdiDrawStream: Got unknown pDS data\n");
        return 0;
    }

    {
        let mut sm = MARGINS {
            cxLeftWidth: (*pDS).leftSizingMargin as INT,
            cxRightWidth: (*pDS).rightSizingMargin as INT,
            cyTopHeight: (*pDS).topSizingMargin as INT,
            cyBottomHeight: (*pDS).bottomSizingMargin as INT,
        };
        let transparent: INT;
        let sizingtype: INT;

        if (*pDS).drawOption & DS_TRANSPARENTALPHA != 0 {
            transparent = ALPHABLEND_FULL;
        } else if (*pDS).drawOption & DS_TRANSPARENTCLR != 0 {
            transparent = ALPHABLEND_BINARY;
        } else {
            transparent = ALPHABLEND_NONE;
        }

        if (*pDS).drawOption & DS_TILE != 0 {
            sizingtype = ST_TILE;
        } else if (*pDS).drawOption & DS_TRUESIZE != 0 {
            sizingtype = ST_TRUESIZE;
        } else {
            sizingtype = ST_STRETCH;
        }

        if (*pDS).rcDest.right < (*pDS).rcDest.left || (*pDS).rcDest.bottom < (*pDS).rcDest.top {
            return 0;
        }

        if sm.cxLeftWidth + sm.cxRightWidth > (*pDS).rcDest.right - (*pDS).rcDest.left {
            sm.cxLeftWidth = 0;
            sm.cxRightWidth = 0;
        }

        if sm.cyTopHeight + sm.cyBottomHeight > (*pDS).rcDest.bottom - (*pDS).rcDest.top {
            sm.cyTopHeight = 0;
            sm.cyBottomHeight = 0;
        }

        UXTHEME_DrawImageBackground(
            (*pDS).hDC,
            (*pDS).hImage,
            core::ptr::addr_of_mut!((*pDS).rcSrc),
            transparent,
            (*pDS).crTransparent,
            FALSE,
            sizingtype,
            &mut sm,
            core::ptr::addr_of_mut!((*pDS).rcDest),
        );
    }
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiValidateHandle(hobj: HGDIOBJ) -> BOOL {
    let (ty, EntryPid, _) = GdiReadSharedEntry(GDI_HANDLE_GET_INDEX(hobj));
    if (ty & GDI_ENTRY_BASETYPE_MASK as LONG) != 0
        && ((ty.wrapping_shl(GDI_ENTRY_UPPER_SHIFT) & GDI_HANDLE_TYPE_MASK as LONG) as usize)
            == GDI_HANDLE_GET_TYPE(hobj)
    {
        let pid: HANDLE = (EntryPid & !0x1usize) as HANDLE;
        if pid.is_null() || pid == CurrentProcessId {
            return TRUE;
        }
    }
    FALSE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiFixUpHandle(hGdiObj: HGDIOBJ) -> HGDIOBJ {
    if GDI_HANDLE_GET_UPPER(hGdiObj) != 0 {
        return hGdiObj;
    }

    /* FIXME is this right ?? */
    let (ty, _, _) = GdiReadSharedEntry(GDI_HANDLE_GET_INDEX(hGdiObj));

    /* Rebuild handle for Object */
    ((hGdiObj as usize) | (ty.wrapping_shl(GDI_ENTRY_UPPER_SHIFT) as isize as usize)) as HGDIOBJ
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiQueryTable() -> PVOID {
    GdiHandleTable as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn GdiGetHandleUserData(
    hGdiObj: HGDIOBJ,
    ObjectType: DWORD,
    UserData: *mut PVOID,
) -> BOOL {
    /* GdiHandleTable is set by GdiProcessSetup (called from user32 init).
     * If user32 isn't loaded yet, the table is NULL. */
    if GdiHandleTable.is_null() {
        return FALSE;
    }

    let (ty, EntryPid, EntryUser) = GdiReadSharedEntry(GDI_HANDLE_GET_INDEX(hGdiObj));

    /* Check if we have the correct type */
    if GDI_HANDLE_GET_TYPE(hGdiObj) != ObjectType as usize
        || ((ty.wrapping_shl(GDI_ENTRY_UPPER_SHIFT) & GDI_HANDLE_TYPE_MASK as LONG) as usize) != ObjectType as usize
        || (ty & GDI_ENTRY_BASETYPE_MASK as LONG) != (ObjectType & GDI_ENTRY_BASETYPE_MASK) as LONG
    {
        return FALSE;
    }

    /* Check if we are the owner */
    if (EntryPid & !0x1usize) as HANDLE != CurrentProcessId {
        return FALSE;
    }

    *UserData = EntryUser;
    TRUE
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn GdiGetLDC(hdc: HDC) -> PLDC {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        return core::ptr::null_mut();
    }

    /* Return the LDC pointer */
    (*dcattr_f!(pdcattr, pvLDC)) as PLDC
}
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn GdiSetLDC(hdc: HDC, pvLDC: PVOID) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        return FALSE;
    }

    /* Set the LDC pointer */
    (*dcattr_f!(pdcattr, pvLDC)) = pvLDC;
    TRUE
}
}

#[no_mangle]
pub unsafe extern "C" fn GdiSAPCallback(pldc: PLDC) {
    let NewTime: DWORD = GetTickCount();
    let Time: DWORD = NewTime.wrapping_sub((*pldc).CallBackTick);

    if Time < SAPCALLBACKDELAY {
        return;
    }

    (*pldc).CallBackTick = NewTime;

    if ((*pldc).pAbortProc.unwrap())((*pldc).hDC, 0) == 0 {
        CancelDC((*pldc).hDC);
        AbortDoc((*pldc).hDC);
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiSetBatchLimit(Limit: DWORD) -> DWORD {
    let OldLimit: DWORD = GDI_BatchLimit;

    if (Limit == 0) || (Limit >= GDI_BATCH_LIMIT) {
        return Limit;
    }

    GdiFlush();
    GDI_BatchLimit = Limit;
    OldLimit
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGetBatchLimit() -> DWORD {
    GDI_BatchLimit
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiSetLastError(dwErrCode: DWORD) {
    (*NtCurrentTeb()).LastErrorValue = dwErrCode as ULONG;
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn hGetPEBHandle(Type: HANDLECACHETYPE, cr: COLORREF) -> HGDIOBJ {
    let Number: INT;
    let Offset: INT;
    let MaxNum: INT;
    let GdiType: INT;
    let mut Handle: HGDIOBJ = core::ptr::null_mut();

    let Lock: HANDLE = InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!((*GdiHandleCache).ulLock) as *mut PVOID,
        NtCurrentTeb() as PVOID,
        core::ptr::null_mut(),
    );

    if !Lock.is_null() {
        return Handle;
    }

    Number = (*GdiHandleCache).ulNumHandles[Type as usize] as INT;

    if Type == _HANDLECACHETYPE_hctBrushHandle {
        Offset = 0;
        MaxNum = CACHE_BRUSH_ENTRIES as INT;
        GdiType = GDILoObjType_GDILoObjType_LO_BRUSH_TYPE as INT;
    } else if Type == _HANDLECACHETYPE_hctPenHandle {
        Offset = CACHE_BRUSH_ENTRIES as INT;
        MaxNum = CACHE_PEN_ENTRIES as INT;
        GdiType = GDILoObjType_GDILoObjType_LO_PEN_TYPE as INT;
    } else if Type == _HANDLECACHETYPE_hctRegionHandle {
        Offset = (CACHE_BRUSH_ENTRIES + CACHE_PEN_ENTRIES) as INT;
        MaxNum = CACHE_REGION_ENTRIES as INT;
        GdiType = GDILoObjType_GDILoObjType_LO_REGION_TYPE as INT;
    } else {
        // Font is not supported here.
        return Handle;
    }

    if Number != 0 && Number <= MaxNum {
        let mut pBrush_Attr: PBRUSH_ATTR = core::ptr::null_mut();
        let hPtr: *mut HGDIOBJ =
            (core::ptr::addr_of_mut!((*GdiHandleCache).Handle) as *mut HGDIOBJ).add(Offset as usize);
        Handle = *hPtr.add((Number - 1) as usize);

        if GdiGetHandleUserData(Handle, GdiType as DWORD, &mut pBrush_Attr as *mut PBRUSH_ATTR as *mut PVOID) != 0 {
            if (*pBrush_Attr).AttrFlags & ATTR_CACHED != 0 {
                // DPRINT("Get Handle! ...");
                (*pBrush_Attr).AttrFlags &= !ATTR_CACHED;
                *hPtr.add((Number - 1) as usize) = core::ptr::null_mut();
                (*GdiHandleCache).ulNumHandles[Type as usize] -= 1;
                if Type == _HANDLECACHETYPE_hctBrushHandle {
                    // Handle only brush.
                    if (*pBrush_Attr).lbColor != cr {
                        (*pBrush_Attr).lbColor = cr;
                        (*pBrush_Attr).AttrFlags |= ATTR_NEW_COLOR;
                    }
                }
            }
        } else {
            Handle = core::ptr::null_mut();
        }
    }
    InterlockedExchangePointer(core::ptr::addr_of_mut!((*GdiHandleCache).ulLock) as *mut PVOID, Lock);
    Handle
}
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn bMakePathNameW(
    lpBuffer: LPWSTR,
    lpFileName: LPCWSTR,
    lpFilePart: *mut LPWSTR,
    unknown: DWORD,
) -> BOOL {
    let _ = (lpBuffer, lpFileName, lpFilePart, unknown);
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}

/*
 * @implemented
 * Synchronized with WINE dlls/gdi32/driver.c
 */
#[no_mangle]
pub unsafe extern "system" fn GdiConvertToDevmodeW(dmA: *const DEVMODEA) -> *mut DEVMODEW {
    let mut dmA_size: WORD;

    dmA_size = (*dmA).dmSize;

    /* this is the minimal dmSize that XP accepts */
    if (dmA_size as usize) < core::mem::offset_of!(DEVMODEA, dmFields) {
        return core::ptr::null_mut();
    }

    if dmA_size as usize > core::mem::size_of::<DEVMODEA>() {
        dmA_size = core::mem::size_of::<DEVMODEA>() as WORD;
    }

    let mut dmW_size: WORD = dmA_size.wrapping_add(CCHDEVICENAME as WORD);
    if dmA_size as usize >= core::mem::offset_of!(DEVMODEA, dmFormName) + CCHFORMNAME as usize {
        dmW_size = dmW_size.wrapping_add(CCHFORMNAME as WORD);
    }

    let dmW: *mut DEVMODEW = HeapAlloc(
        GetProcessHeap(),
        0,
        (dmW_size as u32 + (*dmA).dmDriverExtra as u32) as SIZE_T,
    ) as *mut DEVMODEW;
    if dmW.is_null() {
        return core::ptr::null_mut();
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        (*dmA).dmDeviceName.as_ptr() as LPCSTR,
        -1,
        (*dmW).dmDeviceName.as_mut_ptr(),
        CCHDEVICENAME,
    );
    /* copy slightly more, to avoid long computations */
    core::ptr::copy_nonoverlapping(
        core::ptr::addr_of!((*dmA).dmSpecVersion) as *const u8,
        core::ptr::addr_of_mut!((*dmW).dmSpecVersion) as *mut u8,
        (dmA_size as usize) - CCHDEVICENAME as usize,
    );

    if dmA_size as usize >= core::mem::offset_of!(DEVMODEA, dmFormName) + CCHFORMNAME as usize {
        if (*dmA).dmFields & DM_FORMNAME != 0 {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*dmA).dmFormName.as_ptr() as LPCSTR,
                -1,
                (*dmW).dmFormName.as_mut_ptr(),
                CCHFORMNAME,
            );
        } else {
            (*dmW).dmFormName[0] = 0;
        }

        if dmA_size as usize > core::mem::offset_of!(DEVMODEA, dmLogPixels) {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*dmA).dmLogPixels) as *const u8,
                core::ptr::addr_of_mut!((*dmW).dmLogPixels) as *mut u8,
                (dmA_size as usize) - core::mem::offset_of!(DEVMODEA, dmLogPixels),
            );
        }
    }

    if (*dmA).dmDriverExtra != 0 {
        core::ptr::copy_nonoverlapping(
            (dmA as *const u8).add(dmA_size as usize),
            (dmW as *mut u8).add(dmW_size as usize),
            (*dmA).dmDriverExtra as usize,
        );
    }

    (*dmW).dmSize = dmW_size;

    dmW
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiRealizationInfo(hdc: HDC, pri: PREALIZATION_INFO) -> BOOL {
    // ATM we do not support local font data and Language Pack.
    NtGdiGetRealizationInfo(hdc, pri, core::ptr::null_mut())
}

/*
 * @halfplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiInitializeLanguagePack(InitParam: DWORD) {
    let _ = InitParam;
    /* gdi32's CMakeLists defines LANGPACK, so the C body is live: load lpk.dll
     * (whose DllMain applies RTL mirroring for the process) and, if it is present,
     * record it in gbLpk, which GetFontLanguageInfo consults. LPK_INIT is gdi32p.h's 1. */
    const LPK_INIT: INT = 1;
    if crate::objects::utils::LoadLPK(LPK_INIT) == 0 {
        // no lpk found!
        return;
    }

    /* Call InitializeLpkHooks with 4 procedure addresses loaded from lpk.dll —
     * compiled out (#if 0) in the C as well. */

    crate::main::dllmain::gbLpk = TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GdiAddGlsBounds(hdc: HDC, prc: LPRECT) -> BOOL {
    if NtGdiSetBoundsRect(hdc, prc as LPRECTL, DCB_WINDOWMGR | DCB_ACCUMULATE) != 0 {
        TRUE
    } else {
        FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetBoundsRectAlt(hdc: HDC, prc: LPRECT, flags: UINT) -> BOOL {
    NtGdiGetBoundsRect(hdc, prc as LPRECTL, flags) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn SetBoundsRectAlt(hdc: HDC, prc: LPRECT, flags: UINT) -> BOOL {
    NtGdiSetBoundsRect(hdc, prc as LPRECTL, flags) as BOOL
}

/*
 * @unimplemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiAddGlsRecord(
    hdc: HDC,
    unknown1: DWORD,
    unknown2: LPCSTR,
    unknown3: LPRECT,
) -> BOOL {
    let _ = (hdc, unknown1, unknown2, unknown3);
    // UNIMPLEMENTED;
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    0
}
