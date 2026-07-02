//! win32ss/gdi/gdi32/objects/brush.c → Rust. GDI brush/pen client wrappers:
//! ExtCreatePen / CreateDIBPatternBrush{,Pt} / CreateHatchBrush / CreatePatternBrush /
//! CreateSolidBrush / CreateBrushIndirect / GetBrushOrgEx / SetBrushOrgEx /
//! Get/Set/ClearBrushAttributes / UnrealizeObject.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/DWORD/UINT/INT/ULONG/ULONG_PTR/COLORREF/FLONG/PVOID/PULONG/HBRUSH/HPEN/
//   HBITMAP/HGLOBAL/HGDIOBJ/HDC/HANDLE/LPPOINT/PPOINTL/PDC_ATTR/PGDIBSSETBRHORG/
//   PBITMAPINFO/LPBITMAPINFO/BITMAPINFO/LOGBRUSH).
// NEEDS-IMPORT: `ConvertBitmapInfo` is a gdi32-private WINAPI helper (objects/utils.c,
//   gdi32p.h:308) — declared `extern "system"` below; resolves within gdi32.dll (still-C
//   object or a later cohab .rs). Flagged for central confirmation it stays exported.
// NEEDS-IMPORT: `GdiGetHandleUserData` is owned by misc.rs (plain cdecl) → `extern "C"`.
// NEEDS-IMPORT: `GlobalLock`/`GlobalUnlock` (kernel32 WINAPI) and `RtlFreeHeap` (ntdll
//   NTAPI) resolve via the respective import libs; `RtlGetProcessHeap()` is a FORCEINLINE
//   macro (`NtCurrentPeb()->ProcessHeap`) reproduced locally over the win32k_sys _TEB/_PEB.

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── Cross-file gdi32 / kernel32 / ntdll fns this file calls ────────────────────
//   objects/utils.c   LPBITMAPINFO WINAPI ConvertBitmapInfo(...)        → extern "system"
//   misc/misc.c       BOOL         (cdecl) GdiGetHandleUserData(...)    → extern "C"
//   kernel32          LPVOID WINAPI GlobalLock(HGLOBAL)                  → extern "system"
//   kernel32          BOOL   WINAPI GlobalUnlock(HGLOBAL)               → extern "system"
//   kernel32          VOID   WINAPI SetLastError(DWORD)                 → extern "system"
//   ntdll             BOOLEAN NTAPI  RtlFreeHeap(PVOID,ULONG,PVOID)     → extern "system"
extern "system" {
    fn ConvertBitmapInfo(
        BitmapInfo: PBITMAPINFO,
        ColorSpec: UINT,
        BitmapInfoSize: *mut UINT,
        FollowedByData: BOOL,
    ) -> LPBITMAPINFO;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
    fn RtlFreeHeap(HeapHandle: HANDLE, Flags: ULONG, P: PVOID) -> BOOLEAN;
}

extern "C" {
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
}

/// `RtlGetProcessHeap()` — FORCEINLINE macro `NtCurrentPeb()->ProcessHeap` (rtlfuncs.h),
/// reproduced over the win32k_sys _TEB/_PEB (NtCurrentTeb lives in crate::gdihelp).
#[inline]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtCreatePen(
    dwPenStyle: DWORD,
    dwWidth: DWORD,
    lplb: *const LOGBRUSH,
    dwStyleCount: DWORD,
    lpStyle: *const DWORD,
) -> HPEN {
    let mut lpPackedDIB: PVOID = core::ptr::null_mut();
    let hPen: HPEN;
    let mut pConvertedInfo: PBITMAPINFO = core::ptr::null_mut();
    let mut ConvertedInfoSize: UINT = 0;
    let mut Hit: BOOL = FALSE;

    if (dwPenStyle & PS_STYLE_MASK) == PS_USERSTYLE {
        if lpStyle.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            return core::ptr::null_mut();
        }
    }
    // This is an enhancement and prevents a call to kernel space.
    else if (dwPenStyle & PS_STYLE_MASK) == PS_INSIDEFRAME
        && (dwPenStyle & PS_TYPE_MASK) != PS_GEOMETRIC
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    } else if (dwPenStyle & PS_STYLE_MASK) == PS_ALTERNATE
        && (dwPenStyle & PS_TYPE_MASK) != PS_COSMETIC
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    } else if dwStyleCount != 0 || !lpStyle.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    let mut lbStyle: UINT = (*lplb).lbStyle;

    if (*lplb).lbStyle > BS_HATCHED {
        if (*lplb).lbStyle == BS_PATTERN {
            pConvertedInfo = (*lplb).lbHatch as PBITMAPINFO;
            if pConvertedInfo.is_null() {
                return core::ptr::null_mut();
            }
        } else if ((*lplb).lbStyle == BS_DIBPATTERN) || ((*lplb).lbStyle == BS_DIBPATTERNPT) {
            if (*lplb).lbStyle == BS_DIBPATTERN {
                lbStyle = BS_DIBPATTERNPT;
                lpPackedDIB = GlobalLock((*lplb).lbHatch as HGLOBAL);
                if lpPackedDIB.is_null() {
                    return core::ptr::null_mut();
                }
            }
            pConvertedInfo = ConvertBitmapInfo(
                lpPackedDIB as PBITMAPINFO,
                (*lplb).lbColor as UINT,
                &mut ConvertedInfoSize,
                TRUE,
            );
            Hit = TRUE; // We converted DIB.
        } else {
            pConvertedInfo = lpStyle as PBITMAPINFO;
        }
    } else {
        pConvertedInfo = (*lplb).lbHatch as PBITMAPINFO;
    }

    hPen = NtGdiExtCreatePen(
        dwPenStyle,
        dwWidth,
        lbStyle,
        (*lplb).lbColor,
        (*lplb).lbHatch,
        pConvertedInfo as ULONG_PTR,
        dwStyleCount,
        lpStyle as PULONG,
        ConvertedInfoSize,
        FALSE,
        core::ptr::null_mut(),
    );

    if (*lplb).lbStyle == BS_DIBPATTERN {
        GlobalUnlock((*lplb).lbHatch as HGLOBAL);
    }

    if Hit != FALSE {
        if (lpPackedDIB as PBITMAPINFO) != pConvertedInfo {
            RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, pConvertedInfo as PVOID);
        }
    }
    hPen
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDIBPatternBrush(
    hglbDIBPacked: HGLOBAL,
    fuColorSpec: UINT,
) -> HBRUSH {
    let lpPackedDIB: PVOID;
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let pConvertedInfo: PBITMAPINFO;
    let mut ConvertedInfoSize: UINT = 0;

    lpPackedDIB = GlobalLock(hglbDIBPacked);
    if lpPackedDIB.is_null() {
        return core::ptr::null_mut();
    }

    pConvertedInfo = ConvertBitmapInfo(
        lpPackedDIB as PBITMAPINFO,
        fuColorSpec,
        &mut ConvertedInfoSize,
        TRUE,
    );
    if !pConvertedInfo.is_null() {
        hBrush = NtGdiCreateDIBBrush(
            pConvertedInfo as PVOID,
            fuColorSpec as FLONG,
            ConvertedInfoSize,
            FALSE,
            FALSE,
            lpPackedDIB,
        );
        if (lpPackedDIB as PBITMAPINFO) != pConvertedInfo {
            RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, pConvertedInfo as PVOID);
        }
    }

    GlobalUnlock(hglbDIBPacked);

    hBrush
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateDIBPatternBrushPt(
    lpPackedDIB: *const core::ffi::c_void,
    fuColorSpec: UINT,
) -> HBRUSH {
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let pConvertedInfo: PBITMAPINFO;
    let mut ConvertedInfoSize: UINT = 0;

    if lpPackedDIB.is_null() {
        return core::ptr::null_mut();
    }

    pConvertedInfo = ConvertBitmapInfo(
        lpPackedDIB as PBITMAPINFO,
        fuColorSpec,
        &mut ConvertedInfoSize,
        TRUE,
    );
    if !pConvertedInfo.is_null() {
        hBrush = NtGdiCreateDIBBrush(
            pConvertedInfo as PVOID,
            fuColorSpec as FLONG,
            ConvertedInfoSize,
            FALSE,
            FALSE,
            lpPackedDIB as PVOID,
        );
        if (lpPackedDIB as PBITMAPINFO) != pConvertedInfo {
            RtlFreeHeap(RtlGetProcessHeap() as HANDLE, 0, pConvertedInfo as PVOID);
        }
    }

    hBrush
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateHatchBrush(fnStyle: INT, clrref: COLORREF) -> HBRUSH {
    NtGdiCreateHatchBrushInternal(fnStyle as ULONG, clrref, FALSE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePatternBrush(hbmp: HBITMAP) -> HBRUSH {
    NtGdiCreatePatternBrushInternal(hbmp, FALSE, FALSE)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateSolidBrush(crColor: COLORREF) -> HBRUSH {
    /* Call Server-Side API */
    NtGdiCreateSolidBrush(crColor, core::ptr::null_mut())
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateBrushIndirect(LogBrush: *const LOGBRUSH) -> HBRUSH {
    let hBrush: HBRUSH;

    match (*LogBrush).lbStyle {
        BS_DIBPATTERN => {
            hBrush = CreateDIBPatternBrush((*LogBrush).lbHatch as HGLOBAL, (*LogBrush).lbColor);
        }

        BS_DIBPATTERNPT => {
            hBrush = CreateDIBPatternBrushPt(
                (*LogBrush).lbHatch as *const core::ffi::c_void,
                (*LogBrush).lbColor,
            );
        }

        BS_PATTERN => {
            hBrush = NtGdiCreatePatternBrushInternal((*LogBrush).lbHatch as HBITMAP, FALSE, FALSE);
        }

        BS_PATTERN8X8 => {
            hBrush = NtGdiCreatePatternBrushInternal((*LogBrush).lbHatch as HBITMAP, FALSE, TRUE);
        }

        BS_SOLID => {
            /*        hBrush = hGetPEBHandle(hctBrushHandle, LogBrush->lbColor);
            if (!hBrush)*/
            hBrush = NtGdiCreateSolidBrush((*LogBrush).lbColor, core::ptr::null_mut());
        }

        BS_HATCHED => {
            hBrush = NtGdiCreateHatchBrushInternal(
                (*LogBrush).lbHatch as ULONG,
                (*LogBrush).lbColor,
                FALSE,
            );
        }

        BS_NULL => {
            hBrush = NtGdiGetStockObject(NULL_BRUSH as INT) as HBRUSH;
        }

        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
            hBrush = core::ptr::null_mut();
        }
    }

    hBrush
}

/*
 * @implemented
 *
 */
#[no_mangle]
pub unsafe extern "system" fn GetBrushOrgEx(hdc: HDC, pt: LPPOINT) -> BOOL {
    let mut Dc_Attr: PDC_ATTR = core::ptr::null_mut();

    if GdiGetHandleUserData(
        hdc as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut Dc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) == FALSE
    {
        return FALSE;
    }
    if !pt.is_null() {
        (*pt).x = (*Dc_Attr).ptlBrushOrigin.x;
        (*pt).y = (*Dc_Attr).ptlBrushOrigin.y;
    }
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBrushOrgEx(
    hdc: HDC,
    nXOrg: core::ffi::c_int,
    nYOrg: core::ffi::c_int,
    lppt: LPPOINT,
) -> BOOL {
    let mut Dc_Attr: PDC_ATTR = core::ptr::null_mut();
    // #if 0 block (LDC / metafile handling) — disabled in C source, omitted.

    if GdiGetHandleUserData(
        hdc as HGDIOBJ,
        GDI_OBJECT_TYPE_DC,
        &mut Dc_Attr as *mut PDC_ATTR as *mut PVOID,
    ) != FALSE
    {
        let pgSBO: PGDIBSSETBRHORG;

        /* Does the caller want the current brush origin to be returned? */
        if !lppt.is_null() {
            (*lppt).x = (*Dc_Attr).ptlBrushOrigin.x;
            (*lppt).y = (*Dc_Attr).ptlBrushOrigin.y;
        }

        /* Check if we have nothing to do */
        if (nXOrg == (*Dc_Attr).ptlBrushOrigin.x) && (nYOrg == (*Dc_Attr).ptlBrushOrigin.y) {
            return TRUE;
        }

        /* Allocate a batch command buffer */
        pgSBO = GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCSetBrushOrg) as PGDIBSSETBRHORG;
        if !pgSBO.is_null() {
            /* Set current brush origin in the DC attribute */
            (*Dc_Attr).ptlBrushOrigin.x = nXOrg;
            (*Dc_Attr).ptlBrushOrigin.y = nYOrg;

            /* Setup the GDI batch command */
            (*pgSBO).ptlBrushOrigin = (*Dc_Attr).ptlBrushOrigin;

            return TRUE;
        }
    }

    /* Fall back to the slower kernel path */
    NtGdiSetBrushOrg(hdc, nXOrg, nYOrg, lppt as PPOINTL)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetBrushAttributes(hbr: HBRUSH) -> DWORD {
    if GDI_HANDLE_IS_STOCKOBJ(hbr as HGDIOBJ) {
        return SC_BB_STOCKOBJ;
    }
    0
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetBrushAttributes(hbm: HBRUSH, dwFlags: DWORD) -> HBRUSH {
    if (dwFlags & !SC_BB_STOCKOBJ) != 0 {
        return core::ptr::null_mut();
    }
    NtGdiSetBrushAttributes(hbm, dwFlags)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ClearBrushAttributes(hbm: HBRUSH, dwFlags: DWORD) -> HBRUSH {
    if (dwFlags & !SC_BB_STOCKOBJ) != 0 {
        return core::ptr::null_mut();
    }
    NtGdiClearBrushAttributes(hbm, dwFlags)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn UnrealizeObject(hgdiobj: HGDIOBJ) -> BOOL {
    let mut retValue: BOOL = TRUE;
    /*
       Win 2k Graphics API, Black Book. by coriolis.com
       Page 62, Note that Steps 3, 5, and 6 are not required for Windows NT(tm)
       and Windows 2000(tm).

       Step 5. UnrealizeObject(hTrackBrush);
     */
    /*
        msdn.microsoft.com,
        "Windows 2000/XP: If hgdiobj is a brush, UnrealizeObject does nothing,
        and the function returns TRUE. Use SetBrushOrgEx to set the origin of
        a brush."
     */
    if GDI_HANDLE_GET_TYPE(hgdiobj) != GDI_OBJECT_TYPE_BRUSH as usize {
        retValue = NtGdiUnrealizeObject(hgdiobj as HANDLE);
    }

    retValue
}
