//! win32ss/gdi/gdi32/objects/coord.c → Rust. GDI coordinate-system client wrappers:
//! viewport/window origin + extents (Get/Set/Offset/Scale), DPtoLP/LPtoDP, GetCurrentPositionEx,
//! Get/SetMapMode, World/Modify/Combine transforms, GetGraphicsMode glue, layout, DC org.
//
// NEEDS-TYPE: none — all scalar aliases + GDI types come from `win32k_sys::win32k`
//   (BOOL/INT/DWORD/ULONG/FLONG/LONG/UINT/USHORT/HDC/HGDIOBJ/HANDLE/FLOAT/FLOATL/FLOATOBJ/
//   EFLOAT_S/XFORM/XFORML/LPXFORM/MATRIX/POINT/POINTL/SIZE/SIZEL/LPPOINT/LPSIZE/PPOINTL/
//   PDC_ATTR/PLDC).
// NEEDS-IMPORT: cross-file gdi32 helpers — all flagged inline at the extern blocks below:
//   - GetAndSetDCDWord  (objects/dc.c, WINAPI)                    → extern "system"
//   - EFtoF             (i386-only, FASTCALL)                     → extern "fastcall"
//   - METADC_* / EMFDC_* (gdi32p.h metafile glue, plain cdecl)   → extern "C"
//   - SetLastError      (kernel32 WINAPI)                         → extern "system"

use win32k_sys::win32k::*;
use crate::gdihelp::*;
use crate::dcattr_f;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

#[allow(non_upper_case_globals)]
const MAX_OFFSET: f64 = 4294967041.0;

// ── Cross-file gdi32 / kernel32 fns this file calls ────────────────────────────
//   objects/dc.c   DWORD WINAPI GetAndSetDCDWord(HDC,UINT,DWORD,ULONG,USHORT,DWORD) → extern "system"
//   kernel32       VOID  WINAPI SetLastError(DWORD)                                 → extern "system"
extern "system" {
    fn GetAndSetDCDWord(
        hdc: HDC,
        u: UINT,
        dwIn: DWORD,
        ulMFId: ULONG,
        usMF16Id: USHORT,
        dwError: DWORD,
    ) -> DWORD;
    fn SetLastError(dwErrCode: DWORD);
}

// gdi32 metafile glue (gdi32p.h) — all PLAIN cdecl (no WINAPI) → `extern "C"`
// (`_METADC_*` / `_EMFDC_*`, not `@N`), per the Wave contract.
extern "C" {
    fn METADC_SetMapMode(hdc: HDC, mode: INT) -> BOOL;
    fn METADC_SetViewportExtEx(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_SetViewportOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_SetWindowExtEx(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_SetWindowOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_ScaleViewportExtEx(hdc: HDC, x_num: INT, x_denom: INT, y_num: INT, y_denom: INT) -> BOOL;
    fn METADC_ScaleWindowExtEx(hdc: HDC, x_num: INT, x_denom: INT, y_num: INT, y_denom: INT) -> BOOL;
    fn METADC_SetLayout(hdc: HDC, layout: DWORD) -> BOOL;
    fn METADC_OffsetViewportOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL;
    fn METADC_OffsetWindowOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL;

    fn EMFDC_ModifyWorldTransform(dc: PLDC, xform: *const XFORM, mode: DWORD) -> BOOL;
    fn EMFDC_SetWorldTransform(dc: PLDC, xform: *const XFORM) -> BOOL;
    fn EMFDC_SetViewportExtEx(dc: PLDC, x: INT, y: INT) -> BOOL;
    fn EMFDC_SetViewportOrgEx(dc: PLDC, x: INT, y: INT) -> BOOL;
    fn EMFDC_SetWindowExtEx(dc: PLDC, x: INT, y: INT) -> BOOL;
    fn EMFDC_SetWindowOrgEx(dc: PLDC, x: INT, y: INT) -> BOOL;
    fn EMFDC_ScaleViewportExtEx(dc: PLDC, x_num: INT, x_denom: INT, y_num: INT, y_denom: INT) -> BOOL;
    fn EMFDC_ScaleWindowExtEx(dc: PLDC, x_num: INT, x_denom: INT, y_num: INT, y_denom: INT) -> BOOL;
    fn EMFDC_SetLayout(dc: PLDC, layout: DWORD) -> BOOL;
}

// gdi32p.h: `GdiGetLDC` is defined FASTCALL in misc.rs (i386 `@GdiGetLDC@4`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn GdiGetLDC(hdc: HDC) -> PLDC;
}

// i386-only: `FLOATL FASTCALL EFtoF(EFLOAT_S *)` (the EFLOAT software-float helper).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn EFtoF(efp: *mut EFLOAT_S) -> FLOATL;
}

/// `FOtoF(pfo)` (gdi32p.h): on i386 `EFtoF((EFLOAT_S*)pfo)`, else just `*(pfo)`.
#[inline(always)]
unsafe fn FOtoF(pfo: *const FLOATOBJ) -> FLOATL {
    #[cfg(target_arch = "x86")]
    {
        EFtoF(pfo as *mut EFLOAT_S)
    }
    #[cfg(not(target_arch = "x86"))]
    {
        *pfo
    }
}

/// `_lrintf` (gdi32p.h FORCEINLINE) — portable rounding. A macro (not a fn) so no f32 crosses
/// a function boundary: the i386 user-mode no-SSE rule forbids f32 in the Rust ABI, but f32 math
/// *inside* a body uses x87 fine.
macro_rules! _lrintf {
    ($f:expr) => {{
        let f: FLOAT = $f;
        if f >= 0.0 {
            (f + 0.5) as INT
        } else {
            (f - 0.5) as INT
        }
    }};
}

/* Currently we use a MATRIX inside the DC_ATTR containing the
   coordinate transformations, while we deal with XFORM structures
   internally. If we move all coordinate transformation to gdi32,
   we might as well have an XFORM structure in the DC_ATTR. */
#[no_mangle]
pub unsafe extern "C" fn MatrixToXForm(pxform: *mut XFORM, pmx: *const MATRIX) {
    let pxforml: *mut XFORML = pxform as *mut XFORML;
    (*pxforml).eM11 = FOtoF(&(*pmx).efM11);
    (*pxforml).eM12 = FOtoF(&(*pmx).efM12);
    (*pxforml).eM21 = FOtoF(&(*pmx).efM21);
    (*pxforml).eM22 = FOtoF(&(*pmx).efM22);
    (*pxforml).eDx = FOtoF(&(*pmx).efDx);
    (*pxforml).eDy = FOtoF(&(*pmx).efDy);
}

#[no_mangle]
pub unsafe extern "C" fn GdiTransformPoints2(
    pxform: *mut XFORM,
    pptOut: PPOINT,
    pptIn: PPOINT,
    nCount: ULONG,
) {
    let mut i: ULONG = 0;
    while i < nCount {
        let idx = i as isize;
        let x: FLOAT = (*pptIn.offset(idx)).x as FLOAT * (*pxform).eM11
            + (*pptIn.offset(idx)).y as FLOAT * (*pxform).eM12
            + (*pxform).eDx;
        (*pptOut.offset(idx)).x = _lrintf!(x);
        let y: FLOAT = (*pptIn.offset(idx)).x as FLOAT * (*pxform).eM21
            + (*pptIn.offset(idx)).y as FLOAT * (*pxform).eM22
            + (*pxform).eDy;
        (*pptOut.offset(idx)).y = _lrintf!(y);
        i += 1;
    }
}

/// FORCEINLINE `GdiTransformPoints` — MatrixToXForm then GdiTransformPoints2.
#[inline]
unsafe fn GdiTransformPoints(pmx: *mut MATRIX, pptOut: PPOINT, pptIn: PPOINT, nCount: ULONG) {
    let mut xform: XFORM = core::mem::zeroed();
    MatrixToXForm(&mut xform, pmx);
    GdiTransformPoints2(&mut xform, pptOut, pptIn, nCount);
}

// Keep `GdiTransformPoints` referenced (it is FORCEINLINE/static-equivalent and otherwise
// has no active call site in this translation unit, matching the C source).
#[allow(dead_code)]
const _GDI_TRANSFORM_POINTS_REF: unsafe fn(*mut MATRIX, PPOINT, PPOINT, ULONG) = GdiTransformPoints;

/// `_fmul!(x,y)` (coord.c macro `((x)==0)?0:(x)*(y)`) — kept a macro (it IS a C macro) so no f32
/// crosses a function boundary (i386 no-SSE ABI rule; x87 body math is fine).
macro_rules! _fmul {
    ($x:expr, $y:expr) => {{
        let x: FLOAT = $x;
        if x == 0.0 {
            0.0 as FLOAT
        } else {
            x * ($y)
        }
    }};
}

#[no_mangle]
pub unsafe extern "system" fn CombineTransform(
    pxfResult: LPXFORM,
    pxf1: *const XFORM,
    pxf2: *const XFORM,
) -> BOOL {
    let mut xformTmp: XFORM = core::mem::zeroed();

    /* Check paramters */
    if pxfResult.is_null() || pxf1.is_null() || pxf2.is_null() {
        return FALSE;
    }

    /* Do matrix multiplication, start with scaling elements */
    xformTmp.eM11 = ((*pxf1).eM11 * (*pxf2).eM11) + ((*pxf1).eM12 * (*pxf2).eM21);
    xformTmp.eM22 = ((*pxf1).eM21 * (*pxf2).eM12) + ((*pxf1).eM22 * (*pxf2).eM22);

    /* Calculate shear/rotate elements only of they are present */
    if ((*pxf1).eM12 != 0.) || ((*pxf1).eM21 != 0.) || ((*pxf2).eM12 != 0.) || ((*pxf2).eM21 != 0.)
    {
        xformTmp.eM12 = ((*pxf1).eM11 * (*pxf2).eM12) + ((*pxf1).eM12 * (*pxf2).eM22);
        xformTmp.eM21 = ((*pxf1).eM21 * (*pxf2).eM11) + ((*pxf1).eM22 * (*pxf2).eM21);
    } else {
        xformTmp.eM12 = 0.;
        xformTmp.eM21 = 0.;
    }

    /* Calculate the offset */
    xformTmp.eDx =
        _fmul!((*pxf1).eDx, (*pxf2).eM11) + _fmul!((*pxf1).eDy, (*pxf2).eM21) + (*pxf2).eDx;
    xformTmp.eDy =
        _fmul!((*pxf1).eDx, (*pxf2).eM12) + _fmul!((*pxf1).eDy, (*pxf2).eM22) + (*pxf2).eDy;

    /* Check for invalid offset ranges */
    if (xformTmp.eDx as f64 > MAX_OFFSET)
        || ((xformTmp.eDx as f64) < -MAX_OFFSET)
        || (xformTmp.eDy as f64 > MAX_OFFSET)
        || ((xformTmp.eDy as f64) < -MAX_OFFSET)
    {
        return FALSE;
    }

    /* All is ok, return the calculated values */
    *pxfResult = xformTmp;
    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetMapMode(hdc: HDC) -> INT {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Return the map mode */
    (*dcattr_f!(pdcattr, iMapMode))
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetMapMode(hdc: HDC, iMode: INT) -> INT {
    /* Handle METADC16 here, since we don't have a DCATTR. */
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
        == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
    {
        return METADC_SetMapMode(hdc, iMode) as INT;
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    /* Force change if Isotropic is set for recompute. */
    if (iMode != (*dcattr_f!(pdcattr, iMapMode))) || (iMode == MM_ISOTROPIC as INT) {
        (*dcattr_f!(pdcattr, ulDirty_)) &= !(SLOW_WIDTHS as ULONG);
        return GetAndSetDCDWord(
            hdc,
            _GETSETDCDWORD_GdiGetSetMapMode as UINT,
            iMode as DWORD,
            EMR_SETMAPMODE as ULONG,
            0,
            0,
        ) as INT;
    }

    (*dcattr_f!(pdcattr, iMapMode))
}

#[no_mangle]
pub unsafe extern "system" fn DPtoLP(hdc: HDC, lpPoints: LPPOINT, nCount: INT) -> BOOL {
    let mut sizlView: SIZEL = core::mem::zeroed();

    if nCount <= 0 {
        return TRUE;
    }

    if hdc.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if lpPoints.is_null() {
        return TRUE;
    }

    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        return FALSE;
    }

    if (*dcattr_f!(pdcattr, iMapMode)) == MM_ISOTROPIC as INT {
        if NtGdiGetDCPoint(
            hdc,
            _GETDCPOINT_GdiGetViewPortExt as UINT,
            &mut sizlView as *mut SIZEL as PPOINTL,
        ) != 0
        {
            if sizlView.cx == 0 || sizlView.cy == 0 {
                return FALSE;
            }
        }
    }

    NtGdiTransformPoints(
        hdc,
        lpPoints as PPOINTL,
        lpPoints as PPOINTL,
        nCount,
        _TRANSFORMTYPE_GdiDpToLp as INT,
    )
}

#[no_mangle]
pub unsafe extern "system" fn LPtoDP(hdc: HDC, lpPoints: LPPOINT, nCount: INT) -> BOOL {
    if nCount <= 0 {
        return TRUE;
    }

    if hdc.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }
    if lpPoints.is_null() {
        return TRUE;
    }

    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        return FALSE;
    }

    NtGdiTransformPoints(
        hdc,
        lpPoints as PPOINTL,
        lpPoints as PPOINTL,
        nCount,
        _TRANSFORMTYPE_GdiLpToDp as INT,
    )
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetCurrentPositionEx(hdc: HDC, lpPoint: LPPOINT) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() || lpPoint.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if ((*dcattr_f!(pdcattr, ulDirty_)) & DIRTY_PTLCURRENT as ULONG) != 0 {
        // have a hit!
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptfxCurrent)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptfxCurrent)).y;
        DPtoLP(hdc, lpPoint, 1); // reconvert back.
        (*dcattr_f!(pdcattr, ptlCurrent)).x = (*lpPoint).x; // save it
        (*dcattr_f!(pdcattr, ptlCurrent)).y = (*lpPoint).y;
        (*dcattr_f!(pdcattr, ulDirty_)) &= !(DIRTY_PTLCURRENT as ULONG); // clear bit
    } else {
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptlCurrent)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptlCurrent)).y;
    }

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetWorldTransform(hdc: HDC, pxform: LPXFORM) -> BOOL {
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }
    // #if 0 block (GdiFixupTransforms + MatrixToXForm of mxWorldToDevice) — disabled in C.
    NtGdiGetTransform(hdc, GdiWorldSpaceToPageSpace as DWORD, pxform)
}

#[no_mangle]
pub unsafe extern "system" fn SetWorldTransform(hdc: HDC, pxform: *const XFORM) -> BOOL {
    ModifyWorldTransform(hdc, pxform, MWT_SET as DWORD)
}

#[no_mangle]
pub unsafe extern "system" fn ModifyWorldTransform(
    hdc: HDC,
    pxform: *const XFORM,
    dwMode: DWORD,
) -> BOOL {
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
        == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
    {
        return FALSE;
    }

    if dwMode == MWT_SET as DWORD {
        // HANDLE_EMETAFDC(BOOL, SetWorldTransform, FALSE, hdc, pxform);
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
            let _Ret: BOOL = FALSE;
            if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
                == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
            {
                return _Ret;
            }
            let pLDC: PLDC = GdiGetLDC(hdc);
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetWorldTransform(pLDC, pxform) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    } else {
        // HANDLE_EMETAFDC(BOOL, ModifyWorldTransform, FALSE, hdc, pxform, dwMode);
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
            let _Ret: BOOL = FALSE;
            if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
                == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
            {
                return _Ret;
            }
            let pLDC: PLDC = GdiGetLDC(hdc);
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ModifyWorldTransform(pLDC, pxform, dwMode) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Check that graphics mode is GM_ADVANCED */
    if (*dcattr_f!(pdcattr, iGraphicsMode)) != GM_ADVANCED as INT {
        return FALSE;
    }

    /* Call win32k to do the work */
    NtGdiModifyWorldTransform(hdc, pxform as LPXFORM, dwMode)
}

#[no_mangle]
pub unsafe extern "system" fn GetViewportExtEx(hdc: HDC, lpSize: LPSIZE) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    /* Check if we need to update values */
    if ((*dcattr_f!(pdcattr, flXform)) & PAGE_EXTENTS_CHANGED as FLONG) != 0
        && ((*dcattr_f!(pdcattr, iMapMode)) == MM_ISOTROPIC as INT)
    {
        /* Call win32k to do the work */
        return NtGdiGetDCPoint(
            hdc,
            _GETDCPOINT_GdiGetViewPortExt as UINT,
            lpSize as PPOINTL,
        );
    }

    /* Nothing to calculate, return the current extension */
    (*lpSize).cx = (*dcattr_f!(pdcattr, szlViewportExt)).cx;
    (*lpSize).cy = (*dcattr_f!(pdcattr, szlViewportExt)).cy;

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetViewportOrgEx(hdc: HDC, lpPoint: LPPOINT) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    /* Get the current viewport org */
    (*lpPoint).x = (*dcattr_f!(pdcattr, ptlViewportOrg)).x;
    (*lpPoint).y = (*dcattr_f!(pdcattr, ptlViewportOrg)).y;

    /* Handle right-to-left layout */
    if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
        (*lpPoint).x = -(*lpPoint).x;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowExtEx(hdc: HDC, lpSize: LPSIZE) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    /* Get the current window extension */
    (*lpSize).cx = (*dcattr_f!(pdcattr, szlWindowExt)).cx;
    (*lpSize).cy = (*dcattr_f!(pdcattr, szlWindowExt)).cy;

    /* Handle right-to-left layout */
    if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
        (*lpSize).cx = -(*lpSize).cx;
    }

    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowOrgEx(hdc: HDC, lpPoint: LPPOINT) -> BOOL {
    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    /* Get the current window origin */
    (*lpPoint).x = (*dcattr_f!(pdcattr, ptlWindowOrg)).x;
    (*lpPoint).y = (*dcattr_f!(pdcattr, ptlWindowOrg)).y;

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetViewportExtEx(
    hdc: HDC,
    nXExtent: core::ffi::c_int,
    nYExtent: core::ffi::c_int,
    lpSize: LPSIZE,
) -> BOOL {
    // HANDLE_METADC(BOOL, SetViewportExtEx, FALSE, hdc, nXExtent, nYExtent);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetViewportExtEx(hdc, nXExtent, nYExtent) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetViewportExtEx(pLDC, nXExtent, nYExtent) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Check if the caller wants the old extension */
    if !lpSize.is_null() {
        /* Return the current viewport extension */
        (*lpSize).cx = (*dcattr_f!(pdcattr, szlViewportExt)).cx;
        (*lpSize).cy = (*dcattr_f!(pdcattr, szlViewportExt)).cy;
    }

    /* Check for trivial case */
    if ((*dcattr_f!(pdcattr, szlViewportExt)).cx == nXExtent) && ((*dcattr_f!(pdcattr, szlViewportExt)).cy == nYExtent) {
        return TRUE;
    }

    if nXExtent == 0 || nYExtent == 0 {
        return TRUE;
    }

    /* Only change viewport extension if we are in iso or aniso mode */
    if ((*dcattr_f!(pdcattr, iMapMode)) == MM_ISOTROPIC as INT)
        || ((*dcattr_f!(pdcattr, iMapMode)) == MM_ANISOTROPIC as INT)
    {
        if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
            if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
                NtGdiFlush(); // Sync up pdcattr from Kernel space.
                (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
            }
        }

        /* Set the new viewport extension */
        (*dcattr_f!(pdcattr, szlViewportExt)).cx = nXExtent;
        (*dcattr_f!(pdcattr, szlViewportExt)).cy = nYExtent;

        /* Handle right-to-left layout */
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            NtGdiMirrorWindowOrg(hdc);
        }

        /* Update xform flags */
        (*dcattr_f!(pdcattr, flXform)) |=
            (PAGE_EXTENTS_CHANGED | INVALIDATE_ATTRIBUTES | DEVICE_TO_WORLD_INVALID) as FLONG;
    }

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetWindowOrgEx(
    hdc: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    lpPoint: LPPOINT,
) -> BOOL {
    // HANDLE_METADC(BOOL, SetWindowOrgEx, FALSE, hdc, X, Y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetWindowOrgEx(hdc, X, Y) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetWindowOrgEx(pLDC, X, Y) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    if !lpPoint.is_null() {
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptlWindowOrg)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptlWindowOrg)).y;
    }

    if ((*dcattr_f!(pdcattr, ptlWindowOrg)).x == X) && ((*dcattr_f!(pdcattr, ptlWindowOrg)).y == Y) {
        return TRUE;
    }

    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
            NtGdiFlush(); // Sync up pdcattr from Kernel space.
            (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
        }
    }

    (*dcattr_f!(pdcattr, ptlWindowOrg)).x = X;
    (*dcattr_f!(pdcattr, ptlWindowOrg)).y = Y;

    (*dcattr_f!(pdcattr, lWindowOrgx)) = X;
    if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
        NtGdiMirrorWindowOrg(hdc);
    }
    (*dcattr_f!(pdcattr, flXform)) |=
        (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as FLONG;
    TRUE

    //    return NtGdiSetWindowOrgEx(hdc, X, Y, lpPoint);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetWindowExtEx(
    hdc: HDC,
    nXExtent: INT,
    nYExtent: INT,
    lpSize: LPSIZE,
) -> BOOL {
    // HANDLE_METADC(BOOL, SetWindowExtEx, FALSE, hdc, nXExtent, nYExtent);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetWindowExtEx(hdc, nXExtent, nYExtent) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_SetWindowExtEx(pLDC, nXExtent, nYExtent) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attr */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Set the error value and return failure */
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    /* Check if the caller wants the old extension */
    if !lpSize.is_null() {
        /* Return the current window extension */
        (*lpSize).cx = (*dcattr_f!(pdcattr, szlWindowExt)).cx;
        (*lpSize).cy = (*dcattr_f!(pdcattr, szlWindowExt)).cy;

        /* Handle right-to-left layout */
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            (*lpSize).cx = -(*lpSize).cx;
        }
    }

    if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
        NtGdiMirrorWindowOrg(hdc);
        (*dcattr_f!(pdcattr, flXform)) |=
            (PAGE_EXTENTS_CHANGED | INVALIDATE_ATTRIBUTES | DEVICE_TO_WORLD_INVALID) as FLONG;
    } else if ((*dcattr_f!(pdcattr, iMapMode)) == MM_ISOTROPIC as INT)
        || ((*dcattr_f!(pdcattr, iMapMode)) == MM_ANISOTROPIC as INT)
    {
        if ((*dcattr_f!(pdcattr, szlWindowExt)).cx == nXExtent) && ((*dcattr_f!(pdcattr, szlWindowExt)).cy == nYExtent) {
            return TRUE;
        }

        if (nXExtent == 0) || (nYExtent == 0) {
            return FALSE;
        }

        if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
            if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
                NtGdiFlush(); // Sync up Dc_Attr from Kernel space.
                (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
            }
        }

        (*dcattr_f!(pdcattr, szlWindowExt)).cx = nXExtent;
        (*dcattr_f!(pdcattr, szlWindowExt)).cy = nYExtent;
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            NtGdiMirrorWindowOrg(hdc);
        }

        (*dcattr_f!(pdcattr, flXform)) |=
            (PAGE_EXTENTS_CHANGED | INVALIDATE_ATTRIBUTES | DEVICE_TO_WORLD_INVALID) as FLONG;
    }

    TRUE
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetViewportOrgEx(
    hdc: HDC,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    lpPoint: LPPOINT,
) -> BOOL {
    // HANDLE_METADC(BOOL, SetViewportOrgEx, FALSE, hdc, X, Y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetViewportOrgEx(hdc, X, Y) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetViewportOrgEx(pLDC, X, Y) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }
    //// HACK : XP+ doesn't do this. See CORE-16656 & CORE-16644.
    if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
        if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
            NtGdiFlush();
            (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
        }
    }
    ////
    if !lpPoint.is_null() {
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptlViewportOrg)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptlViewportOrg)).y;
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            (*lpPoint).x = -(*lpPoint).x;
        }
    }
    (*dcattr_f!(pdcattr, flXform)) |=
        (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as FLONG;
    let mut X = X;
    if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
        X = -X;
    }
    (*dcattr_f!(pdcattr, ptlViewportOrg)).x = X;
    (*dcattr_f!(pdcattr, ptlViewportOrg)).y = Y;
    TRUE

    //    return NtGdiSetViewportOrgEx(hdc,X,Y,lpPoint);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ScaleViewportExtEx(
    hdc: HDC,
    xNum: INT,
    xDenom: INT,
    yNum: INT,
    yDenom: INT,
    lpSize: LPSIZE,
) -> BOOL {
    // HANDLE_METADC(BOOL, ScaleViewportExtEx, FALSE, hdc, xNum, xDenom, yNum, yDenom);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_ScaleViewportExtEx(hdc, xNum, xDenom, yNum, yDenom) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ScaleViewportExtEx(pLDC, xNum, xDenom, yNum, yDenom) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiGetDcAttr(hdc).is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    NtGdiScaleViewportExtEx(hdc, xNum, xDenom, yNum, yDenom, lpSize)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn ScaleWindowExtEx(
    hdc: HDC,
    xNum: INT,
    xDenom: INT,
    yNum: INT,
    yDenom: INT,
    lpSize: LPSIZE,
) -> BOOL {
    // HANDLE_METADC(BOOL, ScaleWindowExtEx, FALSE, hdc, xNum, xDenom, yNum, yDenom);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_ScaleWindowExtEx(hdc, xNum, xDenom, yNum, yDenom) as BOOL;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: BOOL = FALSE;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT
                && EMFDC_ScaleWindowExtEx(pLDC, xNum, xDenom, yNum, yDenom) == 0
            {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiGetDcAttr(hdc).is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    NtGdiScaleWindowExtEx(hdc, xNum, xDenom, yNum, yDenom, lpSize)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetLayout(hdc: HDC) -> DWORD {
    /* METADC16 is not supported in this API */
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
        == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
    {
        return GDI_ERROR;
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Set the error value and return failure */
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    /* Return the layout */
    (*dcattr_f!(pdcattr, dwLayout))
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetLayout(hdc: HDC, dwLayout: DWORD) -> DWORD {
    // HANDLE_METADC(DWORD, SetLayout, GDI_ERROR, hdc, dwLayout);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_SetLayout(hdc, dwLayout) as DWORD;
        } else {
            let pLDC: PLDC = GdiGetLDC(hdc);
            let _Ret: DWORD = GDI_ERROR;
            if pLDC.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
                return _Ret;
            }
            if (*pLDC).iType == LDC_EMFLDC as INT && EMFDC_SetLayout(pLDC, dwLayout) == 0 {
                return _Ret;
            }
            /* Fall through to support information DC's. */
        }
    }

    if GdiGetDcAttr(hdc).is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    NtGdiSetLayout(hdc, -1, dwLayout)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn SetLayoutWidth(hdc: HDC, wox: LONG, dwLayout: DWORD) -> DWORD {
    /* Only normal DCs are handled here */
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        return GDI_ERROR;
    }

    if GdiGetDcAttr(hdc).is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return GDI_ERROR;
    }

    NtGdiSetLayout(hdc, wox, dwLayout)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDCOrgEx(hdc: HDC, lpPoint: LPPOINT) -> BOOL {
    NtGdiGetDCPoint(hdc, _GETDCPOINT_GdiGetDCOrg as UINT, lpPoint as PPOINTL)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetDCOrg(hdc: HDC) -> LONG {
    let mut pt: POINT = core::mem::zeroed();

    /* Call the new API */
    if GetDCOrgEx(hdc, &mut pt) == 0 {
        return 0;
    }

    /* Return the point in the old way */
    // MAKELONG(pt.x, pt.y)
    ((pt.x as u16 as u32) | ((pt.y as u16 as u32) << 16)) as LONG
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn OffsetViewportOrgEx(
    hdc: HDC,
    nXOffset: core::ffi::c_int,
    nYOffset: core::ffi::c_int,
    lpPoint: LPPOINT,
) -> BOOL {
    // HANDLE_METADC16(BOOL, OffsetViewportOrgEx, FALSE, hdc, nXOffset, nYOffset);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_OffsetViewportOrgEx(hdc, nXOffset, nYOffset);
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    if !lpPoint.is_null() {
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptlViewportOrg)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptlViewportOrg)).y;
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            (*lpPoint).x = -(*lpPoint).x;
        }
    }

    let mut nXOffset = nXOffset;
    if nXOffset != 0 || nYOffset != nXOffset {
        if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
            if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
                NtGdiFlush();
                (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
            }
        }

        (*dcattr_f!(pdcattr, flXform)) |=
            (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as FLONG;
        if ((*dcattr_f!(pdcattr, dwLayout)) & LAYOUT_RTL as DWORD) != 0 {
            nXOffset = -nXOffset;
        }
        (*dcattr_f!(pdcattr, ptlViewportOrg)).x += nXOffset;
        (*dcattr_f!(pdcattr, ptlViewportOrg)).y += nYOffset;
    }

    // HANDLE_EMETAFDC(BOOL, SetViewportOrgEx, FALSE, hdc, ptlViewportOrg.x, ptlViewportOrg.y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return _Ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_SetViewportOrgEx(pLDC, (*dcattr_f!(pdcattr, ptlViewportOrg)).x, (*dcattr_f!(pdcattr, ptlViewportOrg)).y)
                == 0
        {
            return _Ret;
        }
        /* Fall through to support information DC's. */
    }

    TRUE

    //    return  NtGdiOffsetViewportOrgEx(hdc, nXOffset, nYOffset, lpPoint);
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn OffsetWindowOrgEx(
    hdc: HDC,
    nXOffset: core::ffi::c_int,
    nYOffset: core::ffi::c_int,
    lpPoint: LPPOINT,
) -> BOOL {
    // HANDLE_METADC16(BOOL, OffsetWindowOrgEx, FALSE, hdc, nXOffset, nYOffset);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return METADC_OffsetWindowOrgEx(hdc, nXOffset, nYOffset);
        }
    }

    /* Get the DC attribute */
    let pdcattr: PDC_ATTR = GdiGetDcAttr(hdc);
    if pdcattr.is_null() {
        /* Do not set LastError here! */
        return FALSE;
    }

    if !lpPoint.is_null() {
        (*lpPoint).x = (*dcattr_f!(pdcattr, ptlWindowOrg)).x;
        (*lpPoint).y = (*dcattr_f!(pdcattr, ptlWindowOrg)).y;
        //lpPoint->x = pdcattr->lWindowOrgx;
    }

    if nXOffset != 0 || nYOffset != nXOffset {
        if (*NtCurrentTeb()).GdiTebBatch.HDC == hdc as HANDLE {
            if ((*dcattr_f!(pdcattr, ulDirty_)) & DC_MODE_DIRTY as ULONG) != 0 {
                NtGdiFlush();
                (*dcattr_f!(pdcattr, ulDirty_)) &= !(DC_MODE_DIRTY as ULONG);
            }
        }

        (*dcattr_f!(pdcattr, flXform)) |=
            (PAGE_XLATE_CHANGED | WORLD_XFORM_CHANGED | DEVICE_TO_WORLD_INVALID) as FLONG;
        (*dcattr_f!(pdcattr, ptlWindowOrg)).x += nXOffset;
        (*dcattr_f!(pdcattr, ptlWindowOrg)).y += nYOffset;
        (*dcattr_f!(pdcattr, lWindowOrgx)) += nXOffset;
    }

    // HANDLE_EMETAFDC(BOOL, SetWindowOrgEx, FALSE, hdc, ptlWindowOrg.x, ptlWindowOrg.y);
    if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ) != GDILoObjType_GDILoObjType_LO_DC_TYPE as usize {
        let _Ret: BOOL = FALSE;
        if GDI_HANDLE_GET_TYPE(hdc as HGDIOBJ)
            == GDILoObjType_GDILoObjType_LO_METADC16_TYPE as usize
        {
            return _Ret;
        }
        let pLDC: PLDC = GdiGetLDC(hdc);
        if pLDC.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            return _Ret;
        }
        if (*pLDC).iType == LDC_EMFLDC as INT
            && EMFDC_SetWindowOrgEx(pLDC, (*dcattr_f!(pdcattr, ptlWindowOrg)).x, (*dcattr_f!(pdcattr, ptlWindowOrg)).y) == 0
        {
            return _Ret;
        }
        /* Fall through to support information DC's. */
    }

    TRUE

    //    return NtGdiOffsetWindowOrgEx(hdc, nXOffset, nYOffset, lpPoint);
}
