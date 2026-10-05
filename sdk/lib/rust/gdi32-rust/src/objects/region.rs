//! win32ss/gdi/gdi32/objects/region.c → Rust. GDI region client wrappers, with the
//! RGN_ATTR user-mode cache fast paths: DeleteRegion (FASTCALL, DEFINED here), CombineRgn,
//! Create{Rect,RectIndirect,EllipticIndirect,Polygon,PolyPolygon}Rgn, ExtCreateRegion,
//! Ext/SelectClipRgn, Exclude/Intersect/OffsetClipRgn, Get{ClipRgn,MetaRgn,RegionData,RgnBox},
//! Offset/Set/SetMeta/Mirror Rgn helpers, PtInRegion, RectInRegion.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/INT/UINT/DWORD/ULONG/LONG/HRGN/HDC/HWND/HGDIOBJ/HANDLE/LPVOID/COLORREF/SIZE_T/
//   RECT/RECTL/PRECT/LPRECT/LPCRECT/POINT/PPOINT/PPOINTL/PULONG/RGN_ATTR/PRGN_ATTR/
//   RGNDATA/LPRGNDATA/XFORM/LPXFORM/PLDC/PGDI_TABLE_ENTRY/PGDIBSOBJECT/PGDIBSEXTSELCLPRGN).
// NEEDS-IMPORT (cross-file, resolved within gdi32.dll / imports):
//   include/gdi32p.h  BOOL  WINAPI    GdiValidateHandle(HGDIOBJ)          → extern "system"
//   include/gdi32p.h  HGDIOBJ FASTCALL hGetPEBHandle(HANDLECACHETYPE,COLORREF) → extern "fastcall"
//   include/gdi32p.h  PLDC  FASTCALL  GdiGetLDC(HDC)                      → extern "fastcall"
//   include/gdi32p.h  BOOL  (cdecl)   EMFDC_SetMetaRgn(LDC*)             → extern "C"
//   include/gdi32p.h  BOOL  (cdecl)   METADC_/EMFDC_ {ExcludeClipRect,
//                       ExtSelectClipRgn,IntersectClipRect,OffsetClipRgn} → extern "C"
//   objects/gdiobj.c  BOOL  WINAPI    DeleteObject(HGDIOBJ)              → extern "system"
//   kernel32          HeapAlloc/HeapFree/GetProcessHeap/SetLastError      → extern "system"
//   user32            IntersectRect/SubtractRect/EqualRect/GetWindowRect  → extern "system"
//   misc/misc.c       GdiHandleTable: PGDI_TABLE_ENTRY (global)           → extern "C" static mut
//   GdiGetRgnAttr/GdiGetDcAttr/GdiAllocBatchCommand + handle macros from crate::gdihelp.

use win32k_sys::win32k::*;
use crate::gdihelp::*;
use crate::dcattr_f;

// win32k_sys FALSE/TRUE are bindgen u32; BOOL is i32 — BOOL-typed shadows.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// region.c: MIN/MAX universal-set 28-bit bound box (INT_MIN/16 .. INT_MAX/16).
const MIN_COORD: INT = i32::MIN / 16;
const MAX_COORD: INT = i32::MAX / 16;

// INRECT(r, x, y): r.right > x && r.left <= x && r.bottom > y && r.top <= y
#[inline]
unsafe fn INRECT(r: &RECTL, x: INT, y: INT) -> bool {
    (r.right > x) && (r.left <= x) && (r.bottom > y) && (r.top <= y)
}

extern "system" {
    fn GdiValidateHandle(hgdiobj: HGDIOBJ) -> BOOL;
    fn DeleteObject(hObject: HGDIOBJ) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
    // kernel32
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    // user32
    fn IntersectRect(lprcDst: LPRECT, lprcSrc1: LPCRECT, lprcSrc2: LPCRECT) -> BOOL;
    fn SubtractRect(lprcDst: LPRECT, lprcSrc1: LPCRECT, lprcSrc2: LPCRECT) -> BOOL;
    fn EqualRect(lprc1: LPCRECT, lprc2: LPCRECT) -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
}

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
    fn hGetPEBHandle(Type: HANDLECACHETYPE, cr: COLORREF) -> HGDIOBJ;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
    fn hGetPEBHandle(Type: HANDLECACHETYPE, cr: COLORREF) -> HGDIOBJ;
}

extern "C" {
    fn EMFDC_SetMetaRgn(dc_attr: PLDC) -> BOOL;
    fn METADC_ExcludeClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn EMFDC_ExcludeClipRect(dc_attr: PLDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn METADC_ExtSelectClipRgn(hdc: HDC, hrgn: HRGN, mode: INT) -> BOOL;
    fn EMFDC_ExtSelectClipRgn(dc_attr: PLDC, hrgn: HRGN, mode: INT) -> BOOL;
    fn METADC_IntersectClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn EMFDC_IntersectClipRect(dc_attr: PLDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn METADC_OffsetClipRgn(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn EMFDC_OffsetClipRgn(dc_attr: PLDC, x: INT, y: INT) -> BOOL;
}

extern "C" {
    static mut GdiHandleTable: PGDI_TABLE_ENTRY;
}

// ── static FASTCALL SortRects ──────────────────────────────────────────────────
unsafe fn SortRects(pRect: PRECT, nCount: INT) {
    let mut i: INT;
    let mut a: INT;
    let mut b: INT;
    let mut c: INT;
    let mut s: INT;
    let mut sRect: RECT;

    if nCount > 0 {
        i = 1; // set index point
        c = nCount; // set inverse count
        loop {
            s = i; // set sort count
            if i < nCount {
                a = i - 1; // [0]
                b = i; // [1]
                loop {
                    if (*pRect.offset(a as isize)).top != (*pRect.offset(b as isize)).top {
                        break;
                    }
                    if (*pRect.offset(a as isize)).left > (*pRect.offset(b as isize)).left {
                        sRect = *pRect.offset(a as isize);
                        *pRect.offset(a as isize) = *pRect.offset(b as isize);
                        *pRect.offset(b as isize) = sRect;
                    }
                    s += 1;
                    b += 1;
                    if !(s < nCount) {
                        break;
                    }
                }
            }
            i += 1;
            let old_c = c;
            c -= 1;
            if old_c == 1 {
                break;
            }
        }
    }
}

/*
 * I thought it was okay to have this in DeleteObject but~ Speed. (jt)
 */
fastcall_fn! {
#[no_mangle]
pub unsafe fn DeleteRegion(hrgn: HRGN) -> BOOL {
    let Rgn_Attr: PRGN_ATTR = GdiGetRgnAttr(hrgn);

    if !Rgn_Attr.is_null() {
        let pgO: PGDIBSOBJECT;

        pgO = GdiAllocBatchCommand(core::ptr::null_mut(), crate::gdihelp::GdiBCDelRgn)
            as PGDIBSOBJECT;
        if !pgO.is_null() {
            (*pgO).hgdiobj = hrgn as HGDIOBJ;
            return TRUE;
        }
    }
    NtGdiDeleteObjectApp(hrgn as HANDLE)
}
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn MirrorRgnByWidth(
    hrgn: HRGN,
    Width: INT,
    phrgn: *mut HRGN,
) -> INT {
    let cRgnDSize: INT;
    let mut Ret: INT = 0;
    let pRgnData: PRGNDATA;

    cRgnDSize = NtGdiGetRegionData(hrgn, 0, core::ptr::null_mut()) as INT;

    if cRgnDSize != 0 {
        pRgnData = HeapAlloc(
            GetProcessHeap(),
            0,
            (cRgnDSize as usize * core::mem::size_of::<LONG>()) as SIZE_T,
        ) as PRGNDATA;
        if !pRgnData.is_null() {
            if GetRegionData(hrgn, cRgnDSize as DWORD, pRgnData) != 0 {
                let hRgnex: HRGN;
                let mut i: UINT;
                let mut SaveL: INT = (*pRgnData).rdh.rcBound.left;
                (*pRgnData).rdh.rcBound.left = Width - (*pRgnData).rdh.rcBound.right;
                (*pRgnData).rdh.rcBound.right = Width - SaveL;
                if (*pRgnData).rdh.nCount > 0 {
                    let pRect: PRECT = core::ptr::addr_of_mut!((*pRgnData).Buffer) as PRECT;
                    i = 0;
                    while i < (*pRgnData).rdh.nCount {
                        SaveL = (*pRect.offset(i as isize)).left;
                        (*pRect.offset(i as isize)).left =
                            Width - (*pRect.offset(i as isize)).right;
                        (*pRect.offset(i as isize)).right = Width - SaveL;
                        i += 1;
                    }
                }
                SortRects(
                    core::ptr::addr_of_mut!((*pRgnData).Buffer) as PRECT,
                    (*pRgnData).rdh.nCount as INT,
                );
                hRgnex = ExtCreateRegion(core::ptr::null(), cRgnDSize as DWORD, pRgnData);
                if !hRgnex.is_null() {
                    if !phrgn.is_null() {
                        *phrgn = hRgnex;
                    } else {
                        CombineRgn(hrgn, hRgnex, core::ptr::null_mut(), RGN_COPY as INT);
                        DeleteObject(hRgnex as HGDIOBJ);
                    }
                    Ret = 1;
                }
            }
            HeapFree(GetProcessHeap(), 0, pRgnData as LPVOID);
        }
    }
    Ret
}
}

#[no_mangle]
pub unsafe extern "system" fn MirrorRgnDC(hdc: HDC, hrgn: HRGN, phrn: *mut HRGN) -> INT {
    if GdiValidateHandle(hdc as HGDIOBJ) == FALSE
        || (GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDI_OBJECT_TYPE_DC as usize)
    {
        return 0;
    }

    MirrorRgnByWidth(hrgn, NtGdiGetDeviceWidth(hdc) as INT, phrn)
}

/* FUNCTIONS *****************************************************************/

#[inline]
unsafe fn IntSetNullRgn(prgnattr: PRGN_ATTR) -> ULONG {
    (*prgnattr).iComplexity = NULLREGION;
    (*prgnattr).AttrFlags |= ATTR_RGN_DIRTY;
    (*prgnattr).Rect.left = 0;
    (*prgnattr).Rect.top = 0;
    (*prgnattr).Rect.right = 0;
    (*prgnattr).Rect.bottom = 0;
    NULLREGION
}

#[inline]
unsafe fn IntSetRectRgn(
    prgnattr: PRGN_ATTR,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
) -> ULONG {
    // ASSERT(xLeft <= xRight); ASSERT(yTop <= yBottom);

    if (xLeft == xRight) || (yTop == yBottom) {
        return IntSetNullRgn(prgnattr);
    }

    (*prgnattr).iComplexity = SIMPLEREGION;
    (*prgnattr).Rect.left = xLeft;
    (*prgnattr).Rect.top = yTop;
    (*prgnattr).Rect.right = xRight;
    (*prgnattr).Rect.bottom = yBottom;
    (*prgnattr).AttrFlags |= ATTR_RGN_DIRTY;
    SIMPLEREGION
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CombineRgn(
    hrgnDest: HRGN,
    hrgnSrc1: HRGN,
    hrgnSrc2: HRGN,
    iCombineMode: INT,
) -> INT {
    let prngattrDest: PRGN_ATTR;
    let prngattrSrc1: PRGN_ATTR;
    let prngattrSrc2: PRGN_ATTR;
    let mut rcTemp: RECT = core::mem::zeroed();

    /* Get the region attribute for dest and source 1 */
    prngattrDest = GdiGetRgnAttr(hrgnDest);
    prngattrSrc1 = GdiGetRgnAttr(hrgnSrc1);

    /* If that failed or if the source 1 region is complex, go to win32k */
    if prngattrDest.is_null()
        || prngattrSrc1.is_null()
        || ((*prngattrSrc1).iComplexity > SIMPLEREGION)
    {
        return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
    }

    /* Handle RGN_COPY first, it needs only hrgnSrc1 */
    if iCombineMode == RGN_COPY as INT {
        /* Check if the source region is a NULLREGION */
        if (*prngattrSrc1).iComplexity == NULLREGION {
            return IntSetNullRgn(prngattrDest) as INT;
        }

        return IntSetRectRgn(
            prngattrDest,
            (*prngattrSrc1).Rect.left,
            (*prngattrSrc1).Rect.top,
            (*prngattrSrc1).Rect.right,
            (*prngattrSrc1).Rect.bottom,
        ) as INT;
    }

    /* For all other operations we need hrgnSrc2 */
    prngattrSrc2 = GdiGetRgnAttr(hrgnSrc2);

    /* If we got no attribute or the region is complex, go to win32k */
    if prngattrSrc2.is_null() || ((*prngattrSrc2).iComplexity > SIMPLEREGION) {
        return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
    }

    /* Handle RGN_AND */
    if iCombineMode == RGN_AND as INT {
        if ((*prngattrSrc1).iComplexity == NULLREGION)
            || ((*prngattrSrc2).iComplexity == NULLREGION)
        {
            return IntSetNullRgn(prngattrDest) as INT;
        }

        if IntersectRect(
            &mut rcTemp as LPRECT,
            core::ptr::addr_of!((*prngattrSrc1).Rect) as LPCRECT,
            core::ptr::addr_of!((*prngattrSrc2).Rect) as LPCRECT,
        ) == 0
        {
            return IntSetNullRgn(prngattrDest) as INT;
        }

        return IntSetRectRgn(
            prngattrDest,
            rcTemp.left,
            rcTemp.top,
            rcTemp.right,
            rcTemp.bottom,
        ) as INT;
    }

    /* Handle RGN_DIFF */
    if iCombineMode == RGN_DIFF as INT {
        if (*prngattrSrc1).iComplexity == NULLREGION {
            return IntSetNullRgn(prngattrDest) as INT;
        }

        if ((*prngattrSrc2).iComplexity == NULLREGION)
            || IntersectRect(
                &mut rcTemp as LPRECT,
                core::ptr::addr_of!((*prngattrSrc1).Rect) as LPCRECT,
                core::ptr::addr_of!((*prngattrSrc2).Rect) as LPCRECT,
            ) == 0
        {
            return IntSetRectRgn(
                prngattrDest,
                (*prngattrSrc1).Rect.left,
                (*prngattrSrc1).Rect.top,
                (*prngattrSrc1).Rect.right,
                (*prngattrSrc1).Rect.bottom,
            ) as INT;
        }

        if SubtractRect(
            &mut rcTemp as LPRECT,
            core::ptr::addr_of!((*prngattrSrc1).Rect) as LPCRECT,
            &rcTemp as LPCRECT,
        ) == 0
        {
            return IntSetNullRgn(prngattrDest) as INT;
        }

        if EqualRect(
            &rcTemp as LPCRECT,
            core::ptr::addr_of!((*prngattrSrc1).Rect) as LPCRECT,
        ) == 0
        {
            return IntSetRectRgn(
                prngattrDest,
                rcTemp.left,
                rcTemp.top,
                rcTemp.right,
                rcTemp.bottom,
            ) as INT;
        }

        return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
    }

    /* Handle OR and XOR */
    if (iCombineMode == RGN_OR as INT) || (iCombineMode == RGN_XOR as INT) {
        /* Check if source 1 is a NULLREGION */
        if (*prngattrSrc1).iComplexity == NULLREGION {
            if (*prngattrSrc2).iComplexity == NULLREGION {
                return IntSetNullRgn(prngattrDest) as INT;
            }

            return IntSetRectRgn(
                prngattrDest,
                (*prngattrSrc2).Rect.left,
                (*prngattrSrc2).Rect.top,
                (*prngattrSrc2).Rect.right,
                (*prngattrSrc2).Rect.bottom,
            ) as INT;
        }

        /* Check if only source 2 is a NULLREGION */
        if (*prngattrSrc2).iComplexity == NULLREGION {
            return IntSetRectRgn(
                prngattrDest,
                (*prngattrSrc1).Rect.left,
                (*prngattrSrc1).Rect.top,
                (*prngattrSrc1).Rect.right,
                (*prngattrSrc1).Rect.bottom,
            ) as INT;
        }

        /* Do the rects have the same x extent */
        if ((*prngattrSrc1).Rect.left == (*prngattrSrc2).Rect.left)
            && ((*prngattrSrc1).Rect.right == (*prngattrSrc2).Rect.right)
        {
            /* Do the rects also have the same y extent */
            if ((*prngattrSrc1).Rect.top == (*prngattrSrc2).Rect.top)
                && ((*prngattrSrc1).Rect.bottom == (*prngattrSrc2).Rect.bottom)
            {
                if iCombineMode == RGN_OR as INT {
                    return IntSetRectRgn(
                        prngattrDest,
                        (*prngattrSrc1).Rect.left,
                        (*prngattrSrc1).Rect.top,
                        (*prngattrSrc1).Rect.right,
                        (*prngattrSrc1).Rect.bottom,
                    ) as INT;
                } else {
                    return IntSetNullRgn(prngattrDest) as INT;
                }
            }

            /* Check if the rects are disjoint */
            if ((*prngattrSrc2).Rect.bottom < (*prngattrSrc1).Rect.top)
                || ((*prngattrSrc2).Rect.top > (*prngattrSrc1).Rect.bottom)
            {
                return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
            }

            /* Check if this is OR */
            if iCombineMode == RGN_OR as INT {
                return IntSetRectRgn(
                    prngattrDest,
                    (*prngattrSrc1).Rect.left,
                    core::cmp::min((*prngattrSrc1).Rect.top, (*prngattrSrc2).Rect.top),
                    (*prngattrSrc1).Rect.right,
                    core::cmp::max((*prngattrSrc1).Rect.bottom, (*prngattrSrc2).Rect.bottom),
                ) as INT;
            }

            /* Check if the rects are adjacent */
            if (*prngattrSrc2).Rect.bottom == (*prngattrSrc1).Rect.top {
                return IntSetRectRgn(
                    prngattrDest,
                    (*prngattrSrc1).Rect.left,
                    (*prngattrSrc2).Rect.top,
                    (*prngattrSrc1).Rect.right,
                    (*prngattrSrc1).Rect.bottom,
                ) as INT;
            } else if (*prngattrSrc2).Rect.top == (*prngattrSrc1).Rect.bottom {
                return IntSetRectRgn(
                    prngattrDest,
                    (*prngattrSrc1).Rect.left,
                    (*prngattrSrc1).Rect.top,
                    (*prngattrSrc1).Rect.right,
                    (*prngattrSrc2).Rect.bottom,
                ) as INT;
            }

            /* When we are here, this is RGN_XOR and the rects overlap */
            return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
        }

        /* Do the rects have the same y extent */
        if ((*prngattrSrc1).Rect.top == (*prngattrSrc2).Rect.top)
            && ((*prngattrSrc1).Rect.bottom == (*prngattrSrc2).Rect.bottom)
        {
            /* Check if the rects are disjoint */
            if ((*prngattrSrc2).Rect.right < (*prngattrSrc1).Rect.left)
                || ((*prngattrSrc2).Rect.left > (*prngattrSrc1).Rect.right)
            {
                return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
            }

            /* Check if this is OR */
            if iCombineMode == RGN_OR as INT {
                return IntSetRectRgn(
                    prngattrDest,
                    core::cmp::min((*prngattrSrc1).Rect.left, (*prngattrSrc2).Rect.left),
                    (*prngattrSrc1).Rect.top,
                    core::cmp::max((*prngattrSrc1).Rect.right, (*prngattrSrc2).Rect.right),
                    (*prngattrSrc1).Rect.bottom,
                ) as INT;
            }

            /* Check if the rects are adjacent */
            if (*prngattrSrc2).Rect.right == (*prngattrSrc1).Rect.left {
                return IntSetRectRgn(
                    prngattrDest,
                    (*prngattrSrc2).Rect.left,
                    (*prngattrSrc1).Rect.top,
                    (*prngattrSrc1).Rect.right,
                    (*prngattrSrc1).Rect.bottom,
                ) as INT;
            } else if (*prngattrSrc2).Rect.left == (*prngattrSrc1).Rect.right {
                return IntSetRectRgn(
                    prngattrDest,
                    (*prngattrSrc1).Rect.left,
                    (*prngattrSrc1).Rect.top,
                    (*prngattrSrc2).Rect.right,
                    (*prngattrSrc1).Rect.bottom,
                ) as INT;
            }

            /* When we are here, this is RGN_XOR and the rects overlap */
            return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
        }

        /* Last case: RGN_OR and one rect is completely within the other */
        if iCombineMode == RGN_OR as INT {
            /* Check if rect 1 can contain rect 2 */
            if (*prngattrSrc1).Rect.left <= (*prngattrSrc2).Rect.left {
                if ((*prngattrSrc1).Rect.right >= (*prngattrSrc2).Rect.right)
                    && ((*prngattrSrc1).Rect.top <= (*prngattrSrc2).Rect.top)
                    && ((*prngattrSrc1).Rect.bottom >= (*prngattrSrc2).Rect.bottom)
                {
                    return IntSetRectRgn(
                        prngattrDest,
                        (*prngattrSrc1).Rect.left,
                        (*prngattrSrc1).Rect.top,
                        (*prngattrSrc1).Rect.right,
                        (*prngattrSrc1).Rect.bottom,
                    ) as INT;
                }
            } else {
                if ((*prngattrSrc2).Rect.right >= (*prngattrSrc1).Rect.right)
                    && ((*prngattrSrc2).Rect.top <= (*prngattrSrc1).Rect.top)
                    && ((*prngattrSrc2).Rect.bottom >= (*prngattrSrc1).Rect.bottom)
                {
                    return IntSetRectRgn(
                        prngattrDest,
                        (*prngattrSrc2).Rect.left,
                        (*prngattrSrc2).Rect.top,
                        (*prngattrSrc2).Rect.right,
                        (*prngattrSrc2).Rect.bottom,
                    ) as INT;
                }
            }
        }

        /* We couldn't handle the operation, go to win32k */
        return NtGdiCombineRgn(hrgnDest, hrgnSrc1, hrgnSrc2, iCombineMode);
    }

    // DPRINT1("Invalid iCombineMode %d\n", iCombineMode);
    SetLastError(ERROR_INVALID_PARAMETER);
    ERROR as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateEllipticRgnIndirect(prc: *const RECT) -> HRGN {
    /* Notes if prc is NULL it will crash on All Windows NT I checked 2000/XP/VISTA */
    NtGdiCreateEllipticRgn(
        (*prc).left,
        (*prc).top,
        (*prc).right,
        (*prc).bottom,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePolygonRgn(
    lppt: *const POINT,
    cPoints: core::ffi::c_int,
    fnPolyFillMode: core::ffi::c_int,
) -> HRGN {
    let mut cPoints = cPoints;
    NtGdiPolyPolyDraw(
        fnPolyFillMode as usize as HDC,
        lppt as PPOINTL,
        &mut cPoints as *mut core::ffi::c_int as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyPolyRgn as INT,
    ) as HRGN
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreatePolyPolygonRgn(
    lppt: *const POINT,
    lpPolyCounts: *const INT,
    nCount: core::ffi::c_int,
    fnPolyFillMode: core::ffi::c_int,
) -> HRGN {
    NtGdiPolyPolyDraw(
        fnPolyFillMode as usize as HDC,
        lppt as PPOINTL,
        lpPolyCounts as PULONG,
        nCount as ULONG,
        _POLYFUNCTYPE_GdiPolyPolyRgn as INT,
    ) as HRGN
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateRectRgn(
    mut x1: core::ffi::c_int,
    mut y1: core::ffi::c_int,
    mut x2: core::ffi::c_int,
    mut y2: core::ffi::c_int,
) -> HRGN {
    let pRgn_Attr: PRGN_ATTR;
    let mut hrgn: HRGN;
    let mut tmp: core::ffi::c_int;

    /* Normalize points, REGION_SetRectRgn does this too. */
    if x1 > x2 {
        tmp = x1;
        x1 = x2;
        x2 = tmp;
    }

    if y1 > y2 {
        tmp = y1;
        y1 = y2;
        y2 = tmp;
    }
    /* Check outside 28 bit limit for universal set bound box. */
    if x1 < MIN_COORD || y1 < MIN_COORD || x2 > MAX_COORD || y2 > MAX_COORD {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    hrgn = hGetPEBHandle(hctRegionHandle, 0 as COLORREF) as HRGN;
    if !hrgn.is_null() {
        // DPRINT1("PEB Handle Cache Test return hrgn %p, should be NULL!\n", hrgn);
        hrgn = core::ptr::null_mut();
    }

    if hrgn.is_null() {
        hrgn = NtGdiCreateRectRgn(0, 0, 1, 1);
    }

    if hrgn.is_null() {
        return hrgn;
    }

    pRgn_Attr = GdiGetRgnAttr(hrgn);
    if pRgn_Attr.is_null() {
        // DPRINT1("No Attr for Region handle!!!\n");
        DeleteRegion(hrgn);
        return core::ptr::null_mut();
    }

    (*pRgn_Attr).AttrFlags = ATTR_RGN_VALID;

    IntSetRectRgn(pRgn_Attr, x1, y1, x2, y2);

    hrgn
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn CreateRectRgnIndirect(prc: *const RECT) -> HRGN {
    /* Notes if prc is NULL it will crash on All Windows NT I checked 2000/XP/VISTA */
    CreateRectRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExcludeClipRect(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
) -> INT {
    // HANDLE_METADC(INT, ExcludeClipRect, ERROR, hdc, xLeft, yTop, xRight, yBottom)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_ExcludeClipRect(hdc, xLeft, yTop, xRight, yBottom) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = ERROR as INT;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ExcludeClipRect(pLDC, xLeft, yTop, xRight, yBottom) == 0
            {
                return _Ret;
            }
        }
    }

    NtGdiExcludeClipRect(hdc, xLeft, yTop, xRight, yBottom)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtCreateRegion(
    lpXform: *const XFORM,
    nCount: DWORD,
    lpRgnData: *const RGNDATA,
) -> HRGN {
    if !lpRgnData.is_null() {
        if lpXform.is_null() && ((*lpRgnData).rdh.nCount == 1) {
            let pRect: PRECT = core::ptr::addr_of!((*lpRgnData).Buffer[0]) as PRECT;
            return CreateRectRgn(
                (*pRect).left,
                (*pRect).top,
                (*pRect).right,
                (*pRect).bottom,
            );
        }
        return NtGdiExtCreateRegion(lpXform as LPXFORM, nCount, lpRgnData as LPRGNDATA);
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    core::ptr::null_mut()
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtSelectClipRgn(hdc: HDC, hrgn: HRGN, iMode: INT) -> INT {
    let mut Ret: INT;
    let NewRgn: HRGN = core::ptr::null_mut();

    // HANDLE_METADC(INT, ExtSelectClipRgn, 0, hdc, hrgn, iMode)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_ExtSelectClipRgn(hdc, hrgn, iMode) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = 0; // nolint: let-shadow (HANDLE_METADC's own _Ret, as in the C)
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ExtSelectClipRgn(pLDC, hrgn, iMode) == 0
            {
                return _Ret;
            }
        }
    }

    // #if 0 RTL-mirroring block omitted (disabled in C source).

    /* Batch handles RGN_COPY only! */
    if iMode == RGN_COPY as INT {
        let pdcattr: PDC_ATTR;
        let mut pRgn_Attr: PRGN_ATTR = core::ptr::null_mut();

        /* Get the DC attribute */
        pdcattr = GdiGetDcAttr(hdc);
        if !pdcattr.is_null() {
            /* Entry Flags = byte 3 of the Type dword (wow64-aware read). */
            let (EntryTy, _, _) =
                crate::misc::misc::GdiReadSharedEntry(GDI_HANDLE_GET_INDEX(hdc as HGDIOBJ));

            /* hrgn can be NULL unless the RGN_COPY mode is specified. */
            if !hrgn.is_null() {
                pRgn_Attr = GdiGetRgnAttr(hrgn);
            }

            if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_DIBSECTION) == 0
                && (((EntryTy >> 24) & 0xFF) as DWORD
                    & GDI_ENTRY_VALIDATE_VIS)
                    == 0
            {
                if hrgn.is_null()
                    || (!hrgn.is_null()
                        && !pRgn_Attr.is_null()
                        && (*pRgn_Attr).iComplexity <= SIMPLEREGION)
                {
                    let pgO: PGDIBSEXTSELCLPRGN =
                        GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCExtSelClipRgn)
                            as PGDIBSEXTSELCLPRGN;
                    if !pgO.is_null() {
                        (*pgO).fnMode = iMode;

                        if !hrgn.is_null() && !pRgn_Attr.is_null() {
                            Ret = (*pRgn_Attr).iComplexity as INT;
                            // The VisRectRegion field needs to be set to a valid state.
                            if (*dcattr_f!(pdcattr, VisRectRegion)).Rect.left >= (*pRgn_Attr).Rect.right
                                || (*dcattr_f!(pdcattr, VisRectRegion)).Rect.top >= (*pRgn_Attr).Rect.bottom
                                || (*dcattr_f!(pdcattr, VisRectRegion)).Rect.right <= (*pRgn_Attr).Rect.left
                                || (*dcattr_f!(pdcattr, VisRectRegion)).Rect.bottom <= (*pRgn_Attr).Rect.top
                            {
                                Ret = NULLREGION as INT;
                            }

                            // Pass the rect since this region will go away.
                            (*pgO).rcl = (*pRgn_Attr).Rect;

                            if !NewRgn.is_null() {
                                DeleteObject(NewRgn as HGDIOBJ);
                            }
                            return Ret;
                        } else {
                            Ret = (*dcattr_f!(pdcattr, VisRectRegion)).iComplexity as INT;
                            (*pgO).fnMode |= GDIBS_NORECT as INT; // Set no hrgn mode.

                            if !NewRgn.is_null() {
                                DeleteObject(NewRgn as HGDIOBJ);
                            }
                            return Ret;
                        }
                    }
                }
            }
        }
    }
    Ret = NtGdiExtSelectClipRgn(hdc, hrgn, iMode);

    if !NewRgn.is_null() {
        DeleteObject(NewRgn as HGDIOBJ);
    }

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetClipRgn(hdc: HDC, hrgn: HRGN) -> core::ffi::c_int {
    let Ret: INT;

    /* Check if DC handle is valid */
    if GdiGetDcAttr(hdc).is_null() {
        /* Last error code differs from what NtGdiGetRandomRgn returns */
        SetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    Ret = NtGdiGetRandomRgn(hdc, hrgn, CLIPRGN as INT);

    Ret
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetMetaRgn(hdc: HDC, hrgn: HRGN) -> core::ffi::c_int {
    NtGdiGetRandomRgn(hdc, hrgn, METARGN as INT)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetRegionData(
    hrgn: HRGN,
    mut nCount: DWORD,
    lpRgnData: LPRGNDATA,
) -> DWORD {
    if lpRgnData.is_null() {
        nCount = 0;
    }

    NtGdiGetRegionData(hrgn, nCount, lpRgnData)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetRgnBox(hrgn: HRGN, prcOut: LPRECT) -> INT {
    let Rgn_Attr: PRGN_ATTR;

    Rgn_Attr = GdiGetRgnAttr(hrgn);
    if Rgn_Attr.is_null() {
        return NtGdiGetRgnBox(hrgn, prcOut as LPRECTL);
    }

    if (*Rgn_Attr).iComplexity == NULLREGION {
        (*prcOut).left = 0;
        (*prcOut).top = 0;
        (*prcOut).right = 0;
        (*prcOut).bottom = 0;
    } else {
        if (*Rgn_Attr).iComplexity != SIMPLEREGION {
            return NtGdiGetRgnBox(hrgn, prcOut as LPRECTL);
        }
        /* WARNING! prcOut is never checked newbies! */
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*Rgn_Attr).Rect) as *const u8,
            prcOut as *mut u8,
            core::mem::size_of::<RECT>(),
        );
    }
    (*Rgn_Attr).iComplexity as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn IntersectClipRect(
    hdc: HDC,
    nLeft: INT,
    nTop: INT,
    nRight: INT,
    nBottom: INT,
) -> INT {
    // HANDLE_METADC(INT, IntersectClipRect, ERROR, hdc, nLeft, nTop, nRight, nBottom)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_IntersectClipRect(hdc, nLeft, nTop, nRight, nBottom) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = ERROR as INT;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_IntersectClipRect(pLDC, nLeft, nTop, nRight, nBottom) == 0
            {
                return _Ret;
            }
        }
    }
    NtGdiIntersectClipRect(hdc, nLeft, nTop, nRight, nBottom)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn MirrorRgn(hwnd: HWND, hrgn: HRGN) -> BOOL {
    let l: INT;
    let mut Rect: RECT = core::mem::zeroed();
    GetWindowRect(hwnd, &mut Rect as LPRECT);
    l = Rect.right - Rect.left;
    Rect.right -= Rect.left;
    (MirrorRgnByWidth(hrgn, l, core::ptr::null_mut()) != 0) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn OffsetClipRgn(hdc: HDC, nXOffset: INT, nYOffset: INT) -> INT {
    // HANDLE_METADC(INT, OffsetClipRgn, ERROR, hdc, nXOffset, nYOffset)
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == GDILoObjType_LO_METADC16_TYPE as usize {
            return METADC_OffsetClipRgn(hdc, nXOffset, nYOffset) as INT;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: INT = ERROR as INT;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_OffsetClipRgn(pLDC, nXOffset, nYOffset) == 0
            {
                return _Ret;
            }
        }
    }
    NtGdiOffsetClipRgn(hdc, nXOffset, nYOffset)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn OffsetRgn(
    hrgn: HRGN,
    nXOffset: core::ffi::c_int,
    nYOffset: core::ffi::c_int,
) -> INT {
    let pRgn_Attr: PRGN_ATTR;
    let mut rc: RECTL;

    pRgn_Attr = GdiGetRgnAttr(hrgn);
    if pRgn_Attr.is_null() {
        return NtGdiOffsetRgn(hrgn, nXOffset, nYOffset);
    }

    if (*pRgn_Attr).iComplexity == NULLREGION {
        return (*pRgn_Attr).iComplexity as INT;
    }

    if (*pRgn_Attr).iComplexity != SIMPLEREGION {
        return NtGdiOffsetRgn(hrgn, nXOffset, nYOffset);
    }

    rc = (*pRgn_Attr).Rect;

    if rc.left < rc.right {
        if rc.top < rc.bottom {
            rc.left += nXOffset;
            rc.top += nYOffset;
            rc.right += nXOffset;
            rc.bottom += nYOffset;

            /* Make sure the offset is within the legal range */
            if (rc.left & MIN_COORD != 0 && ((rc.left & MIN_COORD) != MIN_COORD))
                || (rc.top & MIN_COORD != 0 && ((rc.top & MIN_COORD) != MIN_COORD))
                || (rc.right & MIN_COORD != 0 && ((rc.right & MIN_COORD) != MIN_COORD))
                || (rc.bottom & MIN_COORD != 0 && ((rc.bottom & MIN_COORD) != MIN_COORD))
            {
                // DPRINT("OffsetRgn ERROR\n");
                return ERROR as INT;
            }

            (*pRgn_Attr).Rect = rc;
            (*pRgn_Attr).AttrFlags |= ATTR_RGN_DIRTY;
        }
    }
    (*pRgn_Attr).iComplexity as INT
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PtInRegion(
    hrgn: HRGN,
    x: core::ffi::c_int,
    y: core::ffi::c_int,
) -> BOOL {
    let pRgn_Attr: PRGN_ATTR;

    pRgn_Attr = GdiGetRgnAttr(hrgn);
    if pRgn_Attr.is_null() {
        return NtGdiPtInRegion(hrgn, x, y);
    }

    if (*pRgn_Attr).iComplexity == NULLREGION {
        return FALSE;
    }

    if (*pRgn_Attr).iComplexity != SIMPLEREGION {
        return NtGdiPtInRegion(hrgn, x, y);
    }

    INRECT(&(*pRgn_Attr).Rect, x, y) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RectInRegion(hrgn: HRGN, prcl: LPCRECT) -> BOOL {
    let pRgn_Attr: PRGN_ATTR;
    let mut rc: RECTL = core::mem::zeroed();

    pRgn_Attr = GdiGetRgnAttr(hrgn);
    if pRgn_Attr.is_null() {
        return NtGdiRectInRegion(hrgn, prcl as LPRECTL);
    }

    if (*pRgn_Attr).iComplexity == NULLREGION {
        return FALSE;
    }

    if (*pRgn_Attr).iComplexity != SIMPLEREGION {
        return NtGdiRectInRegion(hrgn, prcl as LPRECTL);
    }

    /* swap the coordinates to make right >= left and bottom >= top */
    if (*prcl).top > (*prcl).bottom {
        rc.top = (*prcl).bottom;
        rc.bottom = (*prcl).top;
    } else {
        rc.top = (*prcl).top;
        rc.bottom = (*prcl).bottom;
    }
    if (*prcl).right < (*prcl).left {
        rc.right = (*prcl).left;
        rc.left = (*prcl).right;
    } else {
        rc.right = (*prcl).right;
        rc.left = (*prcl).left;
    }

    if ((*pRgn_Attr).Rect.left >= rc.right)
        || ((*pRgn_Attr).Rect.right <= rc.left)
        || ((*pRgn_Attr).Rect.top >= rc.bottom)
        || ((*pRgn_Attr).Rect.bottom <= rc.top)
    {
        return FALSE;
    }

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SelectClipRgn(hdc: HDC, hrgn: HRGN) -> core::ffi::c_int {
    ExtSelectClipRgn(hdc, hrgn, RGN_COPY as INT)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetRectRgn(
    hrgn: HRGN,
    xLeft: INT,
    yTop: INT,
    xRight: INT,
    yBottom: INT,
) -> BOOL {
    let prngattr: PRGN_ATTR;

    /* Try to get the region attribute */
    prngattr = GdiGetRgnAttr(hrgn);
    if prngattr.is_null() {
        return NtGdiSetRectRgn(hrgn, xLeft, yTop, xRight, yBottom);
    }

    /* check for NULL region */
    if (xLeft == xRight) || (yTop == yBottom) {
        IntSetNullRgn(prngattr);
        return TRUE;
    }

    if xLeft > xRight {
        (*prngattr).Rect.left = xRight;
        (*prngattr).Rect.right = xLeft;
    } else {
        (*prngattr).Rect.left = xLeft;
        (*prngattr).Rect.right = xRight;
    }

    if yTop > yBottom {
        (*prngattr).Rect.top = yBottom;
        (*prngattr).Rect.bottom = yTop;
    } else {
        (*prngattr).Rect.top = yTop;
        (*prngattr).Rect.bottom = yBottom;
    }

    (*prngattr).AttrFlags |= ATTR_RGN_DIRTY;
    (*prngattr).iComplexity = SIMPLEREGION;

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetMetaRgn(hDC: HDC) -> core::ffi::c_int {
    if GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) != GDILoObjType_LO_DC_TYPE as usize {
        let pLDC: PLDC = GdiGetLDC(hDC);
        if !pLDC.is_null()
            && GDI_HANDLE_GET_TYPE(hDC as HGDIOBJ) != GDILoObjType_LO_METADC16_TYPE as usize
        {
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetMetaRgn(pLDC) == 0 {
                return ERROR as INT;
            }
        } else {
            SetLastError(ERROR_INVALID_HANDLE);
            return ERROR as INT;
        }
    }
    NtGdiSetMetaRgn(hDC)
}
