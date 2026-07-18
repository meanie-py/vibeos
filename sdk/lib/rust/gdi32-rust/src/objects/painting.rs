//! win32ss/gdi/gdi32/objects/painting.c → Rust. GDI drawing-primitive client
//! wrappers: MoveToEx/LineTo, Ellipse/Rectangle/RoundRect, Get/SetPixel(V),
//! Fill/Frame/Invert/PaintRgn, the Poly* family (Bezier/Draw/gon/line/Polyline),
//! Ext/FloodFill, Bit/Pat/PolyPat/Stretch/Mask/Plg-Blt, GdiAlphaBlend,
//! GdiTransparentBlt and GdiGradientFill. Each fast-paths the meta/EMF DC via the
//! HANDLE_METADC / HANDLE_EMETAFDC macro logic, then falls through to NtGdi*.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/INT/UINT/DWORD/ULONG/USHORT/SHORT/ULONG_PTR/COLORREF/FLONG/PVOID/PULONG/
//    HDC/HGDIOBJ/HRGN/HBRUSH/HBITMAP/HANDLE/POINT/PPOINT/LPPOINT/POINTL/PPOINTL/
//    TRIVERTEX/PTRIVERTEX/BLENDFUNCTION/PDC_ATTR/PLDC/PTEB/PPOLYPATBLT/PATRECT/
//    PGDIBSPATBLT/PGDIBSPPATBLT/PGDIBATCHHDR).
// NEEDS-IMPORT: cross-file gdi32 WINAPI client fns (coord.c DPtoLP; dc/gdiobj
//   GetCurrentObject/GetStockObject/SelectObject; gdi32p GdiConvertAndCheckDC /
//   GdiValidateHandle) → `extern "system"`. GdiGetLDC is FASTCALL → `extern "fastcall"`.
//   kernel32 SetLastError → `extern "system"`. All METADC_*/EMFDC_* are plain cdecl
//   (gdi32p.h, no WINAPI) → `extern "C"`. Flagged for central confirmation they stay
//   exported within gdi32.dll (still-C objects emfdc.c / mfdc.c, or later cohab .rs).

use win32k_sys::win32k::*;
use crate::gdihelp::*;
use crate::dcattr_f;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── Cross-file gdi32 / kernel32 client fns (WINAPI → extern "system") ───────────
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn DPtoLP(hdc: HDC, lpPoints: LPPOINT, nCount: INT) -> BOOL;
    fn GdiConvertAndCheckDC(hdc: HDC) -> HDC;
    fn GdiValidateHandle(hGdiObj: HGDIOBJ) -> BOOL;
    fn GetCurrentObject(hdc: HDC, uObjectType: UINT) -> HGDIOBJ;
    fn GetStockObject(i: INT) -> HGDIOBJ;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
}

// ── GdiGetLDC (FASTCALL → extern "fastcall", i386 `@GdiGetLDC@4`) ───────────────
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

// ── Meta-DC / enhanced-metafile recorders (plain cdecl → extern "C") ────────────
extern "C" {
    fn METADC_LineTo(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_MoveTo(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_Ellipse(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn METADC_Rectangle(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn METADC_RoundRect(
        hdc: HDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        ell_width: INT,
        ell_height: INT,
    ) -> BOOL;
    fn METADC_SetPixel(hdc: HDC, x: INT, y: INT, color: COLORREF) -> BOOL;
    fn METADC_FillRgn(hdc: HDC, hrgn: HRGN, hbrush: HBRUSH) -> BOOL;
    fn METADC_FrameRgn(hdc: HDC, hrgn: HRGN, hbrush: HBRUSH, x: INT, y: INT) -> BOOL;
    fn METADC_InvertRgn(hdc: HDC, hrgn: HRGN) -> BOOL;
    fn METADC_Polygon(hdc: HDC, points: *const POINT, count: INT) -> BOOL;
    fn METADC_Polyline(hdc: HDC, points: *const POINT, count: INT) -> BOOL;
    fn METADC_PolyPolygon(
        hdc: HDC,
        points: *const POINT,
        counts: *const INT,
        polygons: UINT,
    ) -> BOOL;
    fn METADC_ExtFloodFill(hdc: HDC, x: INT, y: INT, color: COLORREF, fill_type: UINT) -> BOOL;
    fn METADC_BitBlt(
        hdc_dst: HDC,
        x_dst: INT,
        y_dst: INT,
        width: INT,
        height: INT,
        hdc_src: HDC,
        x_src: INT,
        y_src: INT,
        rop: DWORD,
    ) -> BOOL;
    fn METADC_PatBlt(hdc: HDC, left: INT, top: INT, width: INT, height: INT, rop: DWORD) -> BOOL;
    fn METADC_StretchBlt(
        hdc_dst: HDC,
        x_dst: INT,
        y_dst: INT,
        width_dst: INT,
        height_dst: INT,
        hdc_src: HDC,
        x_src: INT,
        y_src: INT,
        width_src: INT,
        height_src: INT,
        rop: DWORD,
    ) -> BOOL;

    fn EMFDC_PolyBezier(dc_attr: PLDC, points: *const POINT, count: DWORD) -> BOOL;
    fn EMFDC_PolyBezierTo(dc_attr: PLDC, points: *const POINT, count: DWORD) -> BOOL;
    fn EMFDC_PolyDraw(dc_attr: PLDC, points: *const POINT, types: *const BYTE, count: DWORD)
        -> BOOL;
    fn EMFDC_PolylineTo(dc_attr: PLDC, points: *const POINT, count: INT) -> BOOL;
    fn EMFDC_PolyPolyline(
        dc_attr: PLDC,
        points: *const POINT,
        counts: *const DWORD,
        polys: DWORD,
    ) -> BOOL;
    fn EMFDC_PatBlt(dc_attr: PLDC, left: INT, top: INT, width: INT, height: INT, rop: DWORD)
        -> BOOL;
    fn EMFDC_MaskBlt(
        dc_attr: PLDC,
        xDest: INT,
        yDest: INT,
        cx: INT,
        cy: INT,
        hdcSrc: HDC,
        xSrc: INT,
        ySrc: INT,
        hbmMask: HBITMAP,
        xMask: INT,
        yMask: INT,
        dwRop: DWORD,
    ) -> BOOL;
    fn EMFDC_PlgBlt(
        dc_attr: PLDC,
        ppt: *const POINT,
        hdcSrc: HDC,
        xSrc: INT,
        ySrc: INT,
        cx: INT,
        cy: INT,
        hbmMask: HBITMAP,
        xMask: INT,
        yMask: INT,
    ) -> BOOL;
    fn EMFDC_AlphaBlend(
        dc_attr: PLDC,
        x_dst: INT,
        y_dst: INT,
        width_dst: INT,
        height_dst: INT,
        hdc_src: HDC,
        x_src: INT,
        y_src: INT,
        width_src: INT,
        height_src: INT,
        blend_function: BLENDFUNCTION,
    ) -> BOOL;
    fn EMFDC_TransparentBlt(
        dc_attr: PLDC,
        xDst: INT,
        yDst: INT,
        cxDst: INT,
        cyDst: INT,
        hdcSrc: HDC,
        xSrc: INT,
        ySrc: INT,
        cxSrc: INT,
        cySrc: INT,
        crTransparent: UINT,
    ) -> BOOL;
    fn EMFDC_GradientFill(
        dc_attr: PLDC,
        vert_array: PTRIVERTEX,
        nvert: ULONG,
        grad_array: PVOID,
        ngrad: ULONG,
        mode: ULONG,
    ) -> BOOL;
}

// ── handle-type constants (ntgdihdl.h / GDILoObjType), as `usize` for compare ──
const LO_DC_TYPE: usize = GDILoObjType_GDILoObjType_LO_DC_TYPE as usize;
const LO_METADC16_TYPE: usize = GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize;
const LO_ALTDC_TYPE: usize = GDILoObjType_GDILoObjType_LO_ALTDC_TYPE as usize;

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn LineTo(hdc: HDC, x: INT, y: INT) -> BOOL {
    // HANDLE_METADC(BOOL, LineTo, FALSE, hdc, x, y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_LineTo(hdc, x, y);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_LineTo(pLDC, x, y) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiLineTo(hdc, x, y)
}

extern "C" {
    fn EMFDC_LineTo(dc_attr: PLDC, x: INT, y: INT) -> BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MoveToEx(hdc: HDC, x: INT, y: INT, ppt: LPPOINT) -> BOOL {
    let pdcattr: PDC_ATTR;

    // HANDLE_METADC(BOOL, MoveTo, FALSE, hdc, x, y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_MoveTo(hdc, x, y);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_MoveTo(pLDC, x, y) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if !ppt.is_null() {
        if ((*dcattr_f!(pdcattr, ulDirty_)) & DIRTY_PTLCURRENT) != 0 {
            // Double hit!
            (*ppt).x = (*dcattr_f!(pdcattr, ptfxCurrent)).x; // ret prev before change.
            (*ppt).y = (*dcattr_f!(pdcattr, ptfxCurrent)).y;
            DPtoLP(hdc, ppt, 1); // reconvert back.
        } else {
            (*ppt).x = (*dcattr_f!(pdcattr, ptlCurrent)).x;
            (*ppt).y = (*dcattr_f!(pdcattr, ptlCurrent)).y;
        }
    }

    (*dcattr_f!(pdcattr, ptlCurrent)).x = x;
    (*dcattr_f!(pdcattr, ptlCurrent)).y = y;

    (*dcattr_f!(pdcattr, ulDirty_)) &= !DIRTY_PTLCURRENT;
    (*dcattr_f!(pdcattr, ulDirty_)) |= DIRTY_PTFXCURRENT | DIRTY_STYLESTATE; // Set dirty
    TRUE
}

extern "C" {
    fn EMFDC_MoveTo(dc_attr: PLDC, x: INT, y: INT) -> BOOL;
}

// Faithfulness note: in MoveToEx's `if (ppt)` branch the C source tests
// `pdcattr->ulDirty_ & DIRTY_PTLCURRENT` but then reads `ptfxCurrent` — a known
// ReactOS quirk, reproduced verbatim above (mask DIRTY_PTLCURRENT, read ptfxCurrent).

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn Ellipse(
    hdc: HDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, Ellipse, FALSE, hdc, left, top, right, bottom);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_Ellipse(hdc, left, top, right, bottom);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_Ellipse(pLDC, left, top, right, bottom) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiEllipse(hdc, left, top, right, bottom)
}

extern "C" {
    fn EMFDC_Ellipse(dc_attr: PLDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn Rectangle(
    hdc: HDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, Rectangle, FALSE, hdc, left, top, right, bottom);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_Rectangle(hdc, left, top, right, bottom);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_Rectangle(pLDC, left, top, right, bottom) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiRectangle(hdc, left, top, right, bottom)
}

extern "C" {
    fn EMFDC_Rectangle(dc_attr: PLDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn RoundRect(
    hdc: HDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    width: INT,
    height: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, RoundRect, FALSE, hdc, left, top, right, bottom, width, height);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_RoundRect(hdc, left, top, right, bottom, width, height);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_RoundRect(pLDC, left, top, right, bottom, width, height) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiRoundRect(hdc, left, top, right, bottom, width, height)
}

extern "C" {
    fn EMFDC_RoundRect(
        dc_attr: PLDC,
        left: INT,
        top: INT,
        right: INT,
        bottom: INT,
        ell_width: INT,
        ell_height: INT,
    ) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetPixel(hdc: HDC, x: INT, y: INT) -> COLORREF {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDI_OBJECT_TYPE_DC as usize {
        return CLR_INVALID;
    }
    if GdiValidateHandle(hdc as HGDIOBJ) == FALSE {
        return CLR_INVALID;
    }
    NtGdiGetPixel(hdc, x, y) as COLORREF
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetPixel(hdc: HDC, x: INT, y: INT, crColor: COLORREF) -> COLORREF {
    // HANDLE_METADC(COLORREF, SetPixel, CLR_INVALID, hdc, x, y, crColor);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_SetPixel(hdc, x, y, crColor) as COLORREF;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: COLORREF = CLR_INVALID;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetPixel(pLDC, x, y, crColor) == 0 {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE as COLORREF;
    }

    NtGdiSetPixel(hdc, x, y, crColor)
}

extern "C" {
    fn EMFDC_SetPixel(dc_attr: PLDC, x: INT, y: INT, color: COLORREF) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetPixelV(hdc: HDC, x: INT, y: INT, crColor: COLORREF) -> BOOL {
    (SetPixel(hdc, x, y, crColor) != CLR_INVALID) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FillRgn(hdc: HDC, hrgn: HRGN, hbr: HBRUSH) -> BOOL {
    if hrgn.is_null() || hbr.is_null() {
        return FALSE;
    }

    // HANDLE_METADC(BOOL, FillRgn, FALSE, hdc, hrgn, hbr);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_FillRgn(hdc, hrgn, hbr);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_FillRgn(pLDC, hrgn, hbr) == 0 {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiFillRgn(hdc, hrgn, hbr)
}

extern "C" {
    fn EMFDC_FillRgn(dc_attr: PLDC, hrgn: HRGN, hbrush: HBRUSH) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FrameRgn(
    hdc: HDC,
    hrgn: HRGN,
    hbr: HBRUSH,
    nWidth: INT,
    nHeight: INT,
) -> BOOL {
    if hrgn.is_null() || hbr.is_null() {
        return FALSE;
    }

    // HANDLE_METADC(BOOL, FrameRgn, FALSE, hdc, hrgn, hbr, nWidth, nHeight);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_FrameRgn(hdc, hrgn, hbr, nWidth, nHeight);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_FrameRgn(pLDC, hrgn, hbr, nWidth, nHeight) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiFrameRgn(hdc, hrgn, hbr, nWidth, nHeight)
}

extern "C" {
    fn EMFDC_FrameRgn(dc_attr: PLDC, hrgn: HRGN, hbrush: HBRUSH, x: INT, y: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn InvertRgn(hdc: HDC, hrgn: HRGN) -> BOOL {
    if hrgn.is_null() {
        return FALSE;
    }

    // HANDLE_METADC(BOOL, InvertRgn, FALSE, hdc, hrgn);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_InvertRgn(hdc, hrgn);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_InvertRgn(pLDC, hrgn) == 0 {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiInvertRgn(hdc, hrgn)
}

extern "C" {
    fn EMFDC_InvertRgn(dc_attr: PLDC, hrgn: HRGN) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PaintRgn(hdc: HDC, hrgn: HRGN) -> BOOL {
    FillRgn(hdc, hrgn, GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyBezier(hdc: HDC, apt: *const POINT, cpt: DWORD) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, PolyBezier, FALSE, hdc, apt, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_PolyBezier(pLDC, apt, cpt) == 0 {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    let mut cpt = cpt;
    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        &mut cpt as *mut DWORD as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyBezier as INT,
    ) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyBezierTo(hdc: HDC, apt: *const POINT, cpt: DWORD) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, PolyBezierTo, FALSE, hdc, apt, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_PolyBezierTo(pLDC, apt, cpt) == 0 {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    let mut cpt = cpt;
    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        &mut cpt as *mut DWORD as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyBezierTo as INT,
    ) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyDraw(
    hdc: HDC,
    apt: *const POINT,
    aj: *const BYTE,
    cpt: INT,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, PolyDraw, FALSE, hdc, apt, aj, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_PolyDraw(pLDC, apt, aj, cpt as DWORD) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiPolyDraw(hdc, apt as PPOINTL, aj as PBYTE as LPBYTE, cpt as ULONG)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn Polygon(hdc: HDC, apt: *const POINT, cpt: INT) -> BOOL {
    // HANDLE_METADC(BOOL, Polygon, FALSE, hdc, apt, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_Polygon(hdc, apt, cpt);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_Polygon(pLDC, apt, cpt) == 0 {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    let mut cpt = cpt;
    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        &mut cpt as *mut INT as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyPolygon as INT,
    ) as BOOL
}

extern "C" {
    fn EMFDC_Polygon(dc_attr: PLDC, points: *const POINT, count: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn Polyline(hdc: HDC, apt: *const POINT, cpt: INT) -> BOOL {
    // HANDLE_METADC(BOOL, Polyline, FALSE, hdc, apt, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_Polyline(hdc, apt, cpt);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_Polyline(pLDC, apt, cpt) == 0 {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    let mut cpt = cpt;
    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        &mut cpt as *mut INT as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyPolyLine as INT,
    ) as BOOL
}

extern "C" {
    fn EMFDC_Polyline(dc_attr: PLDC, points: *const POINT, count: INT) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolylineTo(hdc: HDC, apt: *const POINT, cpt: DWORD) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, PolylineTo, FALSE, hdc, apt, cpt);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_PolylineTo(pLDC, apt, cpt as INT) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    let mut cpt = cpt;
    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        &mut cpt as *mut DWORD as PULONG,
        1,
        _POLYFUNCTYPE_GdiPolyLineTo as INT,
    ) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyPolygon(
    hdc: HDC,
    apt: *const POINT,
    asz: *const INT,
    csz: INT,
) -> BOOL {
    // HANDLE_METADC(BOOL, PolyPolygon, FALSE, hdc, apt, asz, csz);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_PolyPolygon(hdc, apt, asz, csz as UINT);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_PolyPolygon(pLDC, apt, asz, csz as UINT) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        asz as PULONG,
        csz as ULONG,
        _POLYFUNCTYPE_GdiPolyPolygon as INT,
    ) as BOOL
}

extern "C" {
    fn EMFDC_PolyPolygon(dc_attr: PLDC, points: *const POINT, counts: *const INT, polys: UINT)
        -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PolyPolyline(
    hdc: HDC,
    apt: *const POINT,
    asz: *const DWORD,
    csz: DWORD,
) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
        return FALSE;
    }

    // HANDLE_EMETAFDC(BOOL, PolyPolyline, FALSE, hdc, apt, asz, csz);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_PolyPolyline(pLDC, apt, asz, csz) == 0 {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiPolyPolyDraw(
        hdc,
        apt as PPOINTL,
        asz as PULONG,
        csz as ULONG,
        _POLYFUNCTYPE_GdiPolyPolyLine as INT,
    ) as BOOL
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ExtFloodFill(
    hdc: HDC,
    xStart: INT,
    yStart: INT,
    crFill: COLORREF,
    fuFillType: UINT,
) -> BOOL {
    // HANDLE_METADC(BOOL, ExtFloodFill, FALSE, hdc, xStart, yStart, crFill, fuFillType);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_ExtFloodFill(hdc, xStart, yStart, crFill, fuFillType);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ExtFloodFill(pLDC, xStart, yStart, crFill, fuFillType) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiExtFloodFill(hdc, xStart, yStart, crFill, fuFillType)
}

extern "C" {
    fn EMFDC_ExtFloodFill(
        dc_attr: PLDC,
        x: INT,
        y: INT,
        color: COLORREF,
        fill_type: UINT,
    ) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn FloodFill(
    hdc: HDC,
    xStart: INT,
    yStart: INT,
    crFill: COLORREF,
) -> BOOL {
    ExtFloodFill(hdc, xStart, yStart, crFill, FLOODFILLBORDER as UINT)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn BitBlt(
    hdcDest: HDC,
    xDest: INT,
    yDest: INT,
    cx: INT,
    cy: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    dwRop: DWORD,
) -> BOOL {
    /* Use PatBlt for no source blt, like windows does */
    if !ROP_USES_SOURCE(dwRop) {
        return PatBlt(hdcDest, xDest, yDest, cx, cy, dwRop);
    }

    /* For meta DCs we use StretchBlt via emfdc.c */
    // HANDLE_METADC(BOOL, BitBlt, FALSE, hdcDest, xDest, yDest, cx, cy, hdcSrc, xSrc, ySrc, dwRop);
    if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_BitBlt(hdcDest, xDest, yDest, cx, cy, hdcSrc, xSrc, ySrc, dwRop);
        } else {
            let pLDC: PLDC = GdiGetLDC(hdcDest);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_BitBlt(pLDC, xDest, yDest, cx, cy, hdcSrc, xSrc, ySrc, dwRop) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdcDest).is_null() {
        return FALSE;
    }

    NtGdiBitBlt(
        hdcDest,
        xDest,
        yDest,
        cx,
        cy,
        hdcSrc,
        xSrc,
        ySrc,
        dwRop,
        CLR_INVALID,
        0,
    )
}

extern "C" {
    fn EMFDC_BitBlt(
        dc_attr: PLDC,
        x_dst: INT,
        y_dst: INT,
        width: INT,
        height: INT,
        hdc_src: HDC,
        x_src: INT,
        y_src: INT,
        rop: DWORD,
    ) -> BOOL;
}

/// `ROP_USES_SOURCE(Rop) = (((Rop) << 2 ^ Rop) & 0xCC0000)` (gdi32p.h) — true if the
/// ternary raster op references the source.
#[inline]
fn ROP_USES_SOURCE(rop: DWORD) -> bool {
    (((rop << 2) ^ rop) & 0xCC0000) != 0
}

#[no_mangle]
pub unsafe extern "system" fn PatBlt(
    hdc: HDC,
    nXLeft: INT,
    nYLeft: INT,
    nWidth: INT,
    nHeight: INT,
    dwRop: DWORD,
) -> BOOL {
    let pdcattr: PDC_ATTR;

    // HANDLE_EMETAFDC(BOOL, PatBlt, FALSE, hdc, nXLeft, nYLeft, nWidth, nHeight, dwRop);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_PatBlt(pLDC, nXLeft, nYLeft, nWidth, nHeight, dwRop) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if !pdcattr.is_null() && ((*dcattr_f!(pdcattr, ulDirty_)) & DC_DIBSECTION) == 0 {
        let pgO: PGDIBSPATBLT;

        pgO = GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCPatBlt) as PGDIBSPATBLT;
        if !pgO.is_null() {
            (*dcattr_f!(pdcattr, ulDirty_)) |= DC_MODE_DIRTY;
            (*pgO).nXLeft = nXLeft;
            (*pgO).nYLeft = nYLeft;
            (*pgO).nWidth = nWidth;
            (*pgO).nHeight = nHeight;
            (*pgO).dwRop = dwRop;
            /* Snapshot attributes */
            (*pgO).hbrush = (*dcattr_f!(pdcattr, hbrush));
            (*pgO).crForegroundClr = (*dcattr_f!(pdcattr, crForegroundClr));
            (*pgO).crBackgroundClr = (*dcattr_f!(pdcattr, crBackgroundClr));
            (*pgO).crBrushClr = (*dcattr_f!(pdcattr, crBrushClr));
            (*pgO).ulForegroundClr = (*dcattr_f!(pdcattr, ulForegroundClr));
            (*pgO).ulBackgroundClr = (*dcattr_f!(pdcattr, ulBackgroundClr));
            (*pgO).ulBrushClr = (*dcattr_f!(pdcattr, ulBrushClr));
            return TRUE;
        }
    }
    NtGdiPatBlt(hdc, nXLeft, nYLeft, nWidth, nHeight, dwRop)
}

#[no_mangle]
pub unsafe extern "system" fn PolyPatBlt(
    hdc: HDC,
    dwRop: DWORD,
    pPoly: PPOLYPATBLT,
    nCount: DWORD,
    dwMode: DWORD,
) -> BOOL {
    let mut bResult: BOOL;
    let hbrOld: HBRUSH;
    let pdcattr: PDC_ATTR;

    /* Handle meta DCs */
    if (GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE)
        || (GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_ALTDC_TYPE)
    {
        if GdiValidateHandle(hdc as HGDIOBJ) == FALSE {
            return FALSE;
        }

        /* Save the current DC brush */
        hbrOld = SelectObject(hdc, GetStockObject(DC_BRUSH as INT)) as HBRUSH;

        /* Assume success */
        bResult = TRUE;

        /* Loop all rect */
        let mut i: UINT = 0;
        while i < nCount {
            /* Select the brush for this rect */
            SelectObject(hdc, (*pPoly.add(i as usize)).hBrush as HGDIOBJ);

            /* Do the PatBlt operation for this rect */
            bResult &= PatBlt(
                hdc,
                (*pPoly.add(i as usize)).nXLeft,
                (*pPoly.add(i as usize)).nYLeft,
                (*pPoly.add(i as usize)).nWidth,
                (*pPoly.add(i as usize)).nHeight,
                dwRop,
            );
            i += 1;
        }

        /* Restore the old brush */
        SelectObject(hdc, hbrOld as HGDIOBJ);

        return bResult;
    }

    /* Get the DC attribute */
    pdcattr = GdiGetDcAttr(hdc);
    if nCount != 0 && !pdcattr.is_null() && ((*dcattr_f!(pdcattr, ulDirty_)) & DC_DIBSECTION) == 0 {
        let pgO: PGDIBSPPATBLT;
        let pTeb: PTEB = NtCurrentTeb();

        pgO = GdiAllocBatchCommand(hdc, crate::gdihelp::GdiBCPolyPatBlt) as PGDIBSPPATBLT;
        if !pgO.is_null() {
            let mut cjSize: USHORT = 0;
            if nCount > 1 {
                cjSize = ((nCount - 1) as usize * core::mem::size_of::<PATRECT>()) as USHORT;
            }

            if ((*pTeb).GdiTebBatch.Offset + cjSize as ULONG) <= GDIBATCHBUFSIZE {
                (*dcattr_f!(pdcattr, ulDirty_)) |= DC_MODE_DIRTY;
                (*pgO).Count = nCount;
                (*pgO).Mode = dwMode;
                (*pgO).rop4 = dwRop;
                /* Snapshot attributes */
                (*pgO).crForegroundClr = (*dcattr_f!(pdcattr, crForegroundClr));
                (*pgO).crBackgroundClr = (*dcattr_f!(pdcattr, crBackgroundClr));
                (*pgO).crBrushClr = (*dcattr_f!(pdcattr, crBrushClr));
                (*pgO).ulForegroundClr = (*dcattr_f!(pdcattr, ulForegroundClr));
                (*pgO).ulBackgroundClr = (*dcattr_f!(pdcattr, ulBackgroundClr));
                (*pgO).ulBrushClr = (*dcattr_f!(pdcattr, ulBrushClr));
                core::ptr::copy_nonoverlapping(
                    pPoly as *const u8,
                    (*pgO).pRect.as_mut_ptr() as *mut u8,
                    nCount as usize * core::mem::size_of::<PATRECT>(),
                );
                // Recompute offset and return size, remember one is already accounted for in the structure.
                (*pTeb).GdiTebBatch.Offset += cjSize as ULONG;
                (*(pgO as PGDIBATCHHDR)).Size += cjSize as SHORT;
                return TRUE;
            }
            // Reset offset and count then fall through
            (*pTeb).GdiTebBatch.Offset -= core::mem::size_of::<GDIBSPPATBLT>() as ULONG;
            (*pTeb).GdiBatchCount -= 1;
        }
    }
    NtGdiPolyPatBlt(hdc, dwRop, pPoly, nCount, dwMode)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn StretchBlt(
    hdcDest: HDC,
    xDest: INT,
    yDest: INT,
    cxDest: INT,
    cyDest: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    dwRop: DWORD,
) -> BOOL {
    // HANDLE_METADC(BOOL, StretchBlt, FALSE, hdcDest, xDest, yDest, cxDest, cyDest,
    //               hdcSrc, xSrc, ySrc, cxSrc, cySrc, dwRop);
    if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) != LO_DC_TYPE {
        if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) == LO_METADC16_TYPE {
            return METADC_StretchBlt(
                hdcDest, xDest, yDest, cxDest, cyDest, hdcSrc, xSrc, ySrc, cxSrc, cySrc, dwRop,
            );
        } else {
            let pLDC: PLDC = GdiGetLDC(hdcDest);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_StretchBlt(
                    pLDC, xDest, yDest, cxDest, cyDest, hdcSrc, xSrc, ySrc, cxSrc, cySrc, dwRop,
                ) == 0
            {
                return _Ret;
            }
        }
    }

    if GdiConvertAndCheckDC(hdcDest).is_null() {
        return FALSE;
    }

    NtGdiStretchBlt(
        hdcDest,
        xDest,
        yDest,
        cxDest,
        cyDest,
        hdcSrc,
        xSrc,
        ySrc,
        cxSrc,
        cySrc,
        dwRop,
        CLR_INVALID,
    )
}

extern "C" {
    fn EMFDC_StretchBlt(
        dc_attr: PLDC,
        x_dst: INT,
        y_dst: INT,
        width_dst: INT,
        height_dst: INT,
        hdc_src: HDC,
        x_src: INT,
        y_src: INT,
        width_src: INT,
        height_src: INT,
        rop: DWORD,
    ) -> BOOL;
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn MaskBlt(
    hdcDest: HDC,
    xDest: INT,
    yDest: INT,
    cx: INT,
    cy: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    hbmMask: HBITMAP,
    xMask: INT,
    yMask: INT,
    dwRop: DWORD,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, MaskBlt, FALSE, hdcDest, xDest, yDest, cx, cy, hdcSrc,
    //                 xSrc, ySrc, hbmMask, xMask, yMask, dwRop);
    if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdcDest);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_MaskBlt(
                pLDC, xDest, yDest, cx, cy, hdcSrc, xSrc, ySrc, hbmMask, xMask, yMask, dwRop,
            ) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdcDest).is_null() {
        return FALSE;
    }

    NtGdiMaskBlt(
        hdcDest,
        xDest,
        yDest,
        cx,
        cy,
        hdcSrc,
        xSrc,
        ySrc,
        hbmMask,
        xMask,
        yMask,
        dwRop,
        CLR_INVALID,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn PlgBlt(
    hdcDest: HDC,
    ppt: *const POINT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cx: INT,
    cy: INT,
    hbmMask: HBITMAP,
    xMask: INT,
    yMask: INT,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, PlgBlt, FALSE, hdcDest, ppt, hdcSrc, xSrc, ySrc, cx, cy,
    //                 hbmMask, xMask, yMask);
    if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdcDest as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdcDest);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_PlgBlt(pLDC, ppt, hdcSrc, xSrc, ySrc, cx, cy, hbmMask, xMask, yMask) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdcDest).is_null() {
        return FALSE;
    }

    NtGdiPlgBlt(
        hdcDest,
        ppt as LPPOINT as PPOINTL,
        hdcSrc,
        xSrc,
        ySrc,
        cx,
        cy,
        hbmMask,
        xMask,
        yMask,
        CLR_INVALID,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GdiAlphaBlend(
    hdcDst: HDC,
    xDst: INT,
    yDst: INT,
    cxDst: INT,
    cyDst: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    blendfn: BLENDFUNCTION,
) -> BOOL {
    if hdcSrc.is_null() {
        return FALSE;
    }

    if GDI_HANDLE_GET_TYPE(hdcSrc as HGDIOBJ) == GDI_OBJECT_TYPE_METADC as usize {
        return FALSE;
    }

    // HANDLE_EMETAFDC(BOOL, AlphaBlend, FALSE, hdcDst, xDst, yDst, cxDst, cyDst,
    //                 hdcSrc, xSrc, ySrc, cxSrc, cySrc, blendfn);
    if GDI_HANDLE_GET_TYPE(hdcDst as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdcDst as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdcDst);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_AlphaBlend(
                pLDC, xDst, yDst, cxDst, cyDst, hdcSrc, xSrc, ySrc, cxSrc, cySrc, blendfn,
            ) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdcDst).is_null() {
        return FALSE;
    }

    NtGdiAlphaBlend(
        hdcDst,
        xDst,
        yDst,
        cxDst,
        cyDst,
        hdcSrc,
        xSrc,
        ySrc,
        cxSrc,
        cySrc,
        blendfn,
        core::ptr::null_mut(),
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiTransparentBlt(
    hdcDst: HDC,
    xDst: INT,
    yDst: INT,
    cxDst: INT,
    cyDst: INT,
    hdcSrc: HDC,
    xSrc: INT,
    ySrc: INT,
    cxSrc: INT,
    cySrc: INT,
    crTransparent: UINT,
) -> BOOL {
    // HANDLE_EMETAFDC(BOOL, TransparentBlt, FALSE, hdcDst, xDst, yDst, cxDst, cyDst,
    //                 hdcSrc, xSrc, ySrc, cxSrc, cySrc, crTransparent);
    if GDI_HANDLE_GET_TYPE(hdcDst as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdcDst as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdcDst);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_TransparentBlt(
                pLDC, xDst, yDst, cxDst, cyDst, hdcSrc, xSrc, ySrc, cxSrc, cySrc, crTransparent,
            ) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdcDst).is_null() {
        return FALSE;
    }

    NtGdiTransparentBlt(
        hdcDst,
        xDst,
        yDst,
        cxDst,
        cyDst,
        hdcSrc,
        xSrc,
        ySrc,
        cxSrc,
        cySrc,
        crTransparent as COLORREF,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GdiGradientFill(
    hdc: HDC,
    pVertex: PTRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nCount: ULONG,
    ulMode: ULONG,
) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
        return TRUE;
    }

    // HANDLE_EMETAFDC(BOOL, GradientFill, FALSE, hdc, pVertex, nVertex, pMesh, nCount, ulMode);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != LO_DC_TYPE {
        let mut pLDC: PLDC = core::ptr::null_mut();
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) == LO_METADC16_TYPE {
            return _Ret;
        }
        pLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_GradientFill(pLDC, pVertex, nVertex, pMesh, nCount, ulMode) == 0
        {
            return _Ret;
        }
    }

    if GdiConvertAndCheckDC(hdc).is_null() {
        return FALSE;
    }

    NtGdiGradientFill(hdc, pVertex, nVertex, pMesh, nCount, ulMode)
}
