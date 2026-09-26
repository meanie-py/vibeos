use win32k_sys::win32k::*;

use core::ffi::c_void;
use crate::floatobj::*;
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// IntLPtoDP/IntDPtoLP (coord.h `static inline`) + INTERNAL_APPLY_MATRIX (FORCEINLINE):
// no exported symbols — reimplement inline (mirrors dcutil.rs). DC_vUpdate* come via glob.
#[inline(always)]
unsafe fn INTERNAL_APPLY_MATRIX(matrix: PMATRIX, points: LPPOINT, mut count: u32) -> u8 {
    while count != 0 {
        count -= 1;
        let p = points.offset(count as isize);
        let mut x: FLOATOBJ = core::mem::zeroed();
        let mut y: FLOATOBJ = core::mem::zeroed();
        let mut tmp: FLOATOBJ;
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(x), (*p).x);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efM11));
        tmp = (*matrix).efM21;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).y);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efDx));
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(y), (*p).y);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efM22));
        tmp = (*matrix).efM12;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).x);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(y), core::ptr::addr_of_mut!((*matrix).efDy));
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(x), core::ptr::addr_of_mut!((*p).x)) == 0 { return 0; }
        if FLOATOBJ_bConvertToLong(core::ptr::addr_of!(y), core::ptr::addr_of_mut!((*p).y)) == 0 { return 0; }
    }
    1
}
#[inline(always)]
unsafe fn IntLPtoDP(pdc: *mut DC, ppt: LPPOINT, count: i32) -> BOOL {
    DC_vUpdateWorldToDevice(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX, ppt, count as u32) as BOOL
}
#[inline(always)]
unsafe fn IntDPtoLP(pdc: *mut DC, ppt: LPPOINT, count: i32) -> BOOL {
    DC_vUpdateDeviceToWorld(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxDeviceToWorld) as PMATRIX, ppt, count as u32) as BOOL
}
// IntSysCreateRectpRgnIndirect(prc) macro → IntSysCreateRectpRgn(l,t,r,b) (region.h)
#[inline(always)]
unsafe fn IntSysCreateRectpRgnIndirect(prc: *const RECTL) -> PREGION {
    IntSysCreateRectpRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}
// GDI_HANDLE_GET_INDEX(h) (ntgdihdl.h)
#[inline(always)]
fn GDI_HANDLE_GET_INDEX(h: HANDLE) -> u32 { (h as usize as u32) & GDI_HANDLE_INDEX_MASK }
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n);
}
// ERR(...) debug channel. NOT a no-op: win32k's ERR is unconditional upstream,
// and both call sites report a failed region allocation that the code then
// carries on past ("Expect drawing issues!").
extern "C" { fn DbgPrint(fmt: *const u8, ...) -> u32; }
#[inline(always)] unsafe fn ERR(msg: *const u8) { DbgPrint(msg); }
// RECTL_vSetEmptyRect (rect.h FORCEINLINE) — zero all four edges.
#[inline(always)] unsafe fn RECTL_vSetEmptyRect(prcl: *mut RECTL) {
    (*prcl).left = 0; (*prcl).top = 0; (*prcl).right = 0; (*prcl).bottom = 0;
}

// SafeProbeForRead/Write (safe_user_io) — NTAPI, SIZE_T length (mirror dcutil.rs).
extern "system" {
    fn SafeProbeForRead(a: *const core::ffi::c_void, l: usize, al: u32) -> i32;
    fn SafeProbeForWrite(a: *mut core::ffi::c_void, l: usize, al: u32) -> i32;
}
// RECTL_vMakeWellOrdered, SetLastNtError — FASTCALL (@Name@N on i386).
#[cfg(target_arch = "x86")] extern "fastcall" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))] extern "C" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn SetLastNtError(Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntGdiReleaseRaoRgn(pDC: PDC) {
    IntGdiReleaseRaoRgnImpl(pDC)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntGdiReleaseRaoRgn(pDC: PDC) {
    IntGdiReleaseRaoRgnImpl(pDC)
}

#[inline(always)]
unsafe fn IntGdiReleaseRaoRgnImpl(pDC: PDC) {
    let Index = GDI_HANDLE_GET_INDEX((*pDC).BaseObject.hHmgr);
    let Entry: PGDI_TABLE_ENTRY = core::ptr::addr_of_mut!((*GdiHandleTable).Entries[Index as usize]);
    (*pDC).fs |= DC_DIRTY_RAO as i32;
    (*Entry).__bindgen_anon_1.__bindgen_anon_1.Flags |= GDI_ENTRY_VALIDATE_VIS as u8;
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntGdiReleaseVisRgn(pDC: PDC) {
    IntGdiReleaseVisRgnImpl(pDC)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntGdiReleaseVisRgn(pDC: PDC) {
    IntGdiReleaseVisRgnImpl(pDC)
}

#[inline(always)]
unsafe fn IntGdiReleaseVisRgnImpl(pDC: PDC) {
    IntGdiReleaseRaoRgn(pDC);
    REGION_Delete((*pDC).prgnVis);
    (*pDC).prgnVis = prgnDefault;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn UpdateVisRgn(pdc: PDC) {
    let Index = GDI_HANDLE_GET_INDEX((*pdc).BaseObject.hHmgr);
    let pEntry: PGDI_TABLE_ENTRY = core::ptr::addr_of_mut!((*GdiHandleTable).Entries[Index as usize]);

    (*pEntry).__bindgen_anon_1.__bindgen_anon_1.Flags |= GDI_ENTRY_VALIDATE_VIS as u8;
    (*(*pdc).pdcattr).VisRectRegion.iComplexity = REGION_GetRgnBox((*pdc).prgnVis, core::ptr::addr_of_mut!((*(*pdc).pdcattr).VisRectRegion.Rect)) as _;
    (*(*pdc).pdcattr).VisRectRegion.AttrFlags = ATTR_RGN_VALID as _;
    (*pEntry).__bindgen_anon_1.__bindgen_anon_1.Flags &= !(GDI_ENTRY_VALIDATE_VIS as u8);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn UpdateVisRgn(pdc: PDC) {
    let Index = GDI_HANDLE_GET_INDEX((*pdc).BaseObject.hHmgr);
    let pEntry: PGDI_TABLE_ENTRY = core::ptr::addr_of_mut!((*GdiHandleTable).Entries[Index as usize]);

    (*pEntry).__bindgen_anon_1.__bindgen_anon_1.Flags |= GDI_ENTRY_VALIDATE_VIS as u8;
    (*(*pdc).pdcattr).VisRectRegion.iComplexity = REGION_GetRgnBox((*pdc).prgnVis, core::ptr::addr_of_mut!((*(*pdc).pdcattr).VisRectRegion.Rect)) as _;
    (*(*pdc).pdcattr).VisRectRegion.AttrFlags = ATTR_RGN_VALID as _;
    (*pEntry).__bindgen_anon_1.__bindgen_anon_1.Flags &= !(GDI_ENTRY_VALIDATE_VIS as u8);
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn GdiSelectVisRgn(hdc: HDC, prgn: PREGION) {
    GdiSelectVisRgnImpl(hdc, prgn)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn GdiSelectVisRgn(hdc: HDC, prgn: PREGION) {
    GdiSelectVisRgnImpl(hdc, prgn)
}

#[inline(always)]
unsafe fn GdiSelectVisRgnImpl(hdc: HDC, prgn: PREGION) {
    let dc: *mut DC = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return;
    }

    if prgn.is_null() {
        IntGdiReleaseVisRgn(dc);
        IntSetDefaultRegion(dc);
        DC_UnlockDc(dc);
        return;
    }

    (*dc).fs |= DC_DIRTY_RAO as i32;

    REGION_bCopy((*dc).prgnVis, prgn);
    REGION_bOffsetRgn((*dc).prgnVis, -(*dc).ptlDCOrig.x, -(*dc).ptlDCOrig.y);

    DC_UnlockDc(dc);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSelectClipRgn(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    IntSelectClipRgnImpl(dc, prgn, fnMode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSelectClipRgn(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    IntSelectClipRgnImpl(dc, prgn, fnMode)
}

#[inline(always)]
unsafe fn IntSelectClipRgnImpl(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    let mut Ret: i32 = ERROR as i32;
    let prgnOrigClip: PREGION = (*dc).dclevel.prgnClip;
    let prgnNClip: PREGION;

    if fnMode != RGN_COPY as i32
        && (fnMode != RGN_AND as i32 || prgn.is_null() || !prgnOrigClip.is_null())
    {
        prgnNClip = IntSysCreateRectpRgn(0, 0, 0, 0);

        if !prgnOrigClip.is_null() {
            Ret = IntGdiCombineRgn(prgnNClip, prgnOrigClip, prgn, fnMode);

            if Ret != 0 {
                REGION_Delete(prgnOrigClip);
                (*dc).dclevel.prgnClip = prgnNClip;
                IntGdiReleaseRaoRgn(dc);
            } else {
                REGION_Delete(prgnNClip);
            }
        } else {
            let prgnClip: PREGION;
            let mut rcl: RECTL = core::mem::zeroed();

            REGION_GetRgnBox((*dc).prgnVis, core::ptr::addr_of_mut!(rcl));

            prgnClip = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!(rcl));

            Ret = IntGdiCombineRgn(prgnNClip, prgnClip, prgn, fnMode);

            if Ret != 0 {
                (*dc).dclevel.prgnClip = prgnNClip;
                IntGdiReleaseRaoRgn(dc);
            } else {
                REGION_Delete(prgnNClip);
            }

            REGION_Delete(prgnClip);
        }
        return Ret;
    }

    if prgn.is_null() {
        if !prgnOrigClip.is_null() {
            REGION_Delete((*dc).dclevel.prgnClip);
            (*dc).dclevel.prgnClip = core::ptr::null_mut();
            IntGdiReleaseRaoRgn(dc);
        }
        return SIMPLEREGION as i32;
    }

    if !prgnOrigClip.is_null() && {
        Ret = IntGdiCombineRgn(prgnOrigClip, prgn, core::ptr::null_mut(), RGN_COPY as i32);
        Ret != 0
    } {
        IntGdiReleaseRaoRgn(dc);
    } else {
        let prgnNClip2 = IntSysCreateRectpRgn(0, 0, 0, 0);
        REGION_bCopy(prgnNClip2, prgn);
        Ret = REGION_Complexity(prgnNClip2);
        (*dc).dclevel.prgnClip = prgnNClip2;
        IntGdiReleaseRaoRgn(dc);
    }
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiExtSelectClipRect(dc: PDC, prcl: PRECTL, fnMode: i32) -> i32 {
    IntGdiExtSelectClipRect_impl(dc, prcl, fnMode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiExtSelectClipRect(dc: PDC, prcl: PRECTL, fnMode: i32) -> i32 {
    IntGdiExtSelectClipRect_impl(dc, prcl, fnMode)
}

unsafe fn IntGdiExtSelectClipRect_impl(dc: PDC, prcl: PRECTL, mut fnMode: i32) -> i32 {

    let mut Ret: i32 = ERROR as i32;
    let prgn: PREGION;
    let mut rect: RECTL = core::mem::zeroed();
    let NoRegion: i32 = fnMode & (GDIBS_NORECT as i32);

    fnMode &= !(GDIBS_NORECT as i32);

    if NoRegion != 0 {
        if fnMode == RGN_COPY as i32 {
            Ret = IntSelectClipRgn(dc, core::ptr::null_mut(), RGN_COPY as i32);

            if ((*dc).fs as u32 & DC_DIRTY_RAO) != 0 {
                CLIPPING_UpdateGCRegion(dc);
            }

            if Ret != 0 {
                Ret = REGION_Complexity((*dc).prgnVis);
            }
        }
    } else {
        if !(*dc).dclevel.prgnClip.is_null()
            && REGION_Complexity((*dc).dclevel.prgnClip) == SIMPLEREGION as i32
        {
            REGION_GetRgnBox((*dc).dclevel.prgnClip, &mut rect);

            if (*prcl).left == rect.left
                && (*prcl).top == rect.top
                && (*prcl).right == rect.right
                && (*prcl).bottom == rect.bottom
            {
                let prgn_use = if !(*dc).prgnRao.is_null() {
                    (*dc).prgnRao
                } else {
                    (*dc).prgnVis
                };
                return REGION_Complexity(prgn_use);
            }
        }

        prgn = IntSysCreateRectpRgnIndirect(prcl);

        Ret = IntSelectClipRgn(dc, prgn, fnMode);

        if ((*dc).fs as u32 & DC_DIRTY_RAO) != 0 {
            CLIPPING_UpdateGCRegion(dc);
        }

        if Ret != 0 {
            let prgn_use = if !(*dc).prgnRao.is_null() {
                (*dc).prgnRao
            } else {
                (*dc).prgnVis
            };
            Ret = REGION_Complexity(prgn_use);
        }

        REGION_Delete(prgn);
    }
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiExtSelectClipRgn(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    IntGdiExtSelectClipRgn_impl(dc, prgn, fnMode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiExtSelectClipRgn(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    IntGdiExtSelectClipRgn_impl(dc, prgn, fnMode)
}

unsafe fn IntGdiExtSelectClipRgn_impl(dc: PDC, prgn: PREGION, fnMode: i32) -> i32 {
    let mut Ret: i32 = ERROR as i32;

    if prgn.is_null() {
        if fnMode == RGN_COPY as i32 {
            Ret = IntSelectClipRgn(dc, core::ptr::null_mut(), RGN_COPY as i32);
            if Ret != 0 {
                Ret = REGION_Complexity((*dc).prgnVis);
            }
        }
    } else {
        Ret = IntSelectClipRgn(dc, prgn, fnMode);
        if Ret != 0 {
            if ((*dc).fs as u32 & DC_DIRTY_RAO) != 0 {
                CLIPPING_UpdateGCRegion(dc);
            }

            let rgn = if !(*dc).prgnRao.is_null() {
                (*dc).prgnRao
            } else {
                (*dc).prgnVis
            };
            Ret = REGION_Complexity(rgn);
        }
    }
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtSelectClipRgn(hDC: HDC, hrgn: HRGN, fnMode: i32) -> i32 {
    let retval: i32;
    let dc: *mut DC;
    let prgn: PREGION;

    if fnMode < RGN_AND as i32 || fnMode > RGN_COPY as i32 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return ERROR as i32;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return ERROR as i32;
    }

    prgn = REGION_LockRgn(hrgn);

    if prgn.is_null() && fnMode != RGN_COPY as i32 {
        retval = ERROR as i32;
    } else {
        retval = IntGdiExtSelectClipRgn(dc, prgn, fnMode);
    }

    if !prgn.is_null() {
        REGION_UnlockRgn(prgn);
    }

    DC_UnlockDc(dc);
    retval
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GdiGetClipBox(hdc: HDC, prc: LPRECT) -> i32 {
    GdiGetClipBox_impl(hdc, prc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GdiGetClipBox(hdc: HDC, prc: LPRECT) -> i32 {
    GdiGetClipBox_impl(hdc, prc)
}

unsafe fn GdiGetClipBox_impl(hdc: HDC, prc: LPRECT) -> i32 {

    let pdc: PDC;
    let iComplexity: i32;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return ERROR as i32;
    }

    if ((*pdc).fs as u32 & DC_DIRTY_RAO) != 0 {
        CLIPPING_UpdateGCRegion(pdc);
    }

    if !(*pdc).prgnRao.is_null() {
        iComplexity = REGION_GetRgnBox((*pdc).prgnRao, prc as *mut RECTL);
    } else {
        iComplexity = REGION_GetRgnBox((*pdc).prgnVis, prc as *mut RECTL);
    }

    DC_UnlockDc(pdc);

    IntDPtoLP(pdc, prc as LPPOINT, 2);

    iComplexity
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetAppClipBox(hdc: HDC, prc: LPRECT) -> i32 {

    let mut rect: RECT = core::mem::zeroed();
    let mut iComplexity: i32;
    let Status: i32;

    iComplexity = GdiGetClipBox(hdc, &mut rect);

    if iComplexity != ERROR as i32 {
        Status = SafeProbeForWrite(prc as *mut core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
        if !NT_SUCCESS(Status) {
            iComplexity = ERROR as i32;
        } else if !crate::copy::publish_to_user(
            prc as *mut core::ffi::c_void,
            core::ptr::addr_of!(rect) as *const core::ffi::c_void,
            core::mem::size_of::<RECT>(),
        ) {
            // Probe is validate-only; the C had probe AND store in one _SEH2_TRY.
            iComplexity = ERROR as i32;
        }
    }

    iComplexity
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExcludeClipRect(
    hdc: HDC,
    xLeft: i32,
    yTop: i32,
    xRight: i32,
    yBottom: i32,
) -> i32 {

    let mut iComplexity: i32 = ERROR as i32;
    let mut rect: RECTL = core::mem::zeroed();
    let pdc: PDC;
    let prgn: PREGION;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return ERROR as i32;
    }

    rect.left = xLeft;
    rect.top = yTop;
    rect.right = xRight;
    rect.bottom = yBottom;
    RECTL_vMakeWellOrdered(&mut rect);
    IntLPtoDP(pdc, &mut rect as *mut RECTL as LPPOINT, 2);

    prgn = IntSysCreateRectpRgnIndirect(&mut rect);
    if !prgn.is_null() {
        iComplexity = IntSelectClipRgn(pdc, prgn, RGN_DIFF as i32);
        REGION_Delete(prgn);
    }

    if iComplexity == SIMPLEREGION as i32 {
        iComplexity = COMPLEXREGION as i32;
    }

    DC_UnlockDc(pdc);

    iComplexity
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiIntersectClipRect(
    hdc: HDC,
    xLeft: i32,
    yTop: i32,
    xRight: i32,
    yBottom: i32,
) -> i32 {

    let mut iComplexity: i32 = ERROR as i32;
    let mut rect: RECTL = core::mem::zeroed();

    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return ERROR as i32;
    }

    rect.left = xLeft;
    rect.top = yTop;
    rect.right = xRight;
    rect.bottom = yBottom;
    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(rect));
    IntLPtoDP(pdc, core::ptr::addr_of_mut!(rect) as LPPOINT, 2);

    let prgn: PREGION = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!(rect));
    if !prgn.is_null() {
        iComplexity = IntSelectClipRgn(pdc, prgn, RGN_AND as i32);
        REGION_Delete(prgn);
    }

    if iComplexity == SIMPLEREGION as i32 {
        iComplexity = COMPLEXREGION as i32;
    }

    DC_UnlockDc(pdc);

    iComplexity
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiOffsetClipRgn(hdc: HDC, xOffset: i32, yOffset: i32) -> i32 {
    let mut iComplexity: i32 = 0;
    let pdc: PDC;
    let mut apt: [POINTL; 2] = core::mem::zeroed();

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        if hdc.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
        }
        return ERROR as i32;
    }

    if !(*pdc).dclevel.prgnClip.is_null() {
        apt[0].x = 0;
        apt[0].y = 0;
        apt[1].x = xOffset;
        apt[1].y = yOffset;
        IntLPtoDP(pdc, apt.as_mut_ptr() as LPPOINT, 2);

        if REGION_bOffsetRgn(
            (*pdc).dclevel.prgnClip,
            apt[1].x - apt[0].x,
            apt[1].y - apt[0].y,
        ) == 0
        {
            iComplexity = ERROR as i32;
        } else {
            IntGdiReleaseRaoRgn(pdc);
            UpdateVisRgn(pdc);
            iComplexity = REGION_Complexity((*pdc).dclevel.prgnClip) as i32;
        }

        (*pdc).fs |= DC_DIRTY_RAO as i32;
    } else {
        iComplexity = SIMPLEREGION as i32;
    }

    DC_UnlockDc(pdc);
    iComplexity
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPtVisible(hDC: HDC, X: i32, Y: i32) -> BOOL {
    let mut ret: BOOL = 0;
    let dc: PDC = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    let prgn: PREGION = if !(*dc).prgnRao.is_null() {
        (*dc).prgnRao
    } else {
        (*dc).prgnVis
    };

    if !prgn.is_null() {
        let mut pt: POINT = POINT { x: X, y: Y };
        IntLPtoDP(dc, core::ptr::addr_of_mut!(pt), 1);
        ret = REGION_PtInRegion(prgn, pt.x, pt.y) as i32;
    }

    DC_UnlockDc(dc);

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRectVisible(hDC: HDC, UnsafeRect: LPRECT) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let dc: PDC = DC_LockDc(hDC);
    let mut Result: BOOL = 0;
    let mut Rect: RECTL = core::mem::zeroed();
    let prgn: PREGION;

    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    Status = SafeProbeForRead(UnsafeRect as *const core::ffi::c_void, core::mem::size_of::<RECT>(), 1);
    if NT_SUCCESS(Status)
        // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
        && !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Rect) as *mut core::ffi::c_void,
            UnsafeRect as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        DC_UnlockDc(dc);
        SetLastNtError(Status);
        return 0;
    }

    if ((*dc).fs as u32 & DC_DIRTY_RAO) != 0 {
        CLIPPING_UpdateGCRegion(dc);
    }

    prgn = if !(*dc).prgnRao.is_null() { (*dc).prgnRao } else { (*dc).prgnVis };
    if !prgn.is_null() {
        IntLPtoDP(dc, core::ptr::addr_of_mut!(Rect) as LPPOINT, 2);
        Result = REGION_RectInRegion(prgn, core::ptr::addr_of_mut!(Rect));
    }
    DC_UnlockDc(dc);

    Result
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntGdiSetMetaRgn(pDC: PDC) -> INT {
    IntGdiSetMetaRgnImpl(pDC)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntGdiSetMetaRgn(pDC: PDC) -> INT {
    IntGdiSetMetaRgnImpl(pDC)
}

#[inline(always)]
unsafe fn IntGdiSetMetaRgnImpl(pDC: PDC) -> INT {
    let mut Ret: INT = ERROR as INT;

    if !(*pDC).dclevel.prgnMeta.is_null() {
        if !(*pDC).dclevel.prgnClip.is_null() {
            let prgn: PREGION = IntSysCreateRectpRgn(0, 0, 0, 0);
            if !prgn.is_null() {
                if REGION_bIntersectRegion(prgn, (*pDC).dclevel.prgnMeta, (*pDC).dclevel.prgnClip) != 0 {
                    REGION_Delete((*pDC).dclevel.prgnMeta);
                    (*pDC).dclevel.prgnMeta = prgn;

                    REGION_Delete((*pDC).dclevel.prgnClip);
                    (*pDC).dclevel.prgnClip = core::ptr::null_mut();
                    IntGdiReleaseRaoRgn(pDC);

                    Ret = REGION_Complexity((*pDC).dclevel.prgnMeta);
                } else {
                    REGION_Delete(prgn);
                }
            }
        } else {
            Ret = REGION_Complexity((*pDC).dclevel.prgnMeta);
        }
    } else {
        if !(*pDC).dclevel.prgnClip.is_null() {
            Ret = REGION_Complexity((*pDC).dclevel.prgnClip);
            (*pDC).dclevel.prgnMeta = (*pDC).dclevel.prgnClip;
            (*pDC).dclevel.prgnClip = core::ptr::null_mut();
        } else {
            Ret = SIMPLEREGION as INT;
        }
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetMetaRgn(hDC: HDC) -> i32 {
    let pDC: PDC = DC_LockDc(hDC);

    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return ERROR as i32;
    }

    let Ret = IntGdiSetMetaRgn(pDC);

    DC_UnlockDc(pDC);
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CLIPPING_UpdateGCRegion(pDC: PDC) {
    CLIPPING_UpdateGCRegion_impl(pDC);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CLIPPING_UpdateGCRegion(pDC: PDC) {
    CLIPPING_UpdateGCRegion_impl(pDC);
}

unsafe fn CLIPPING_UpdateGCRegion_impl(pDC: PDC) {
    RECTL_vSetEmptyRect(core::ptr::addr_of_mut!((*pDC).erclClip));

    if !(*pDC).prgnAPI.is_null() {
        REGION_Delete((*pDC).prgnAPI as _);
        (*pDC).prgnAPI = core::ptr::null_mut();
    }

    if !(*pDC).dclevel.prgnMeta.is_null() || !(*pDC).dclevel.prgnClip.is_null() {
        (*pDC).prgnAPI = IntSysCreateRectpRgn(0, 0, 0, 0);
        if (*pDC).prgnAPI.is_null() {
            ERR(b"Failed to allocate prgnAPI! Expect drawing issues!\n\0".as_ptr());
            return;
        }

        if (*pDC).dclevel.prgnMeta.is_null() {
            REGION_bCopy((*pDC).prgnAPI, (*pDC).dclevel.prgnClip);
        } else if (*pDC).dclevel.prgnClip.is_null() {
            REGION_bCopy((*pDC).prgnAPI, (*pDC).dclevel.prgnMeta);
        } else {
            REGION_bIntersectRegion((*pDC).prgnAPI, (*pDC).dclevel.prgnClip, (*pDC).dclevel.prgnMeta);
        }
    }

    if !(*pDC).prgnRao.is_null() {
        REGION_Delete((*pDC).prgnRao as _);
    }

    (*pDC).prgnRao = IntSysCreateRectpRgn(0, 0, 0, 0);
    if (*pDC).prgnRao.is_null() {
        ERR(b"Failed to allocate prgnRao! Expect drawing issues!\n\0".as_ptr());
        return;
    }

    if !(*pDC).prgnAPI.is_null() {
        REGION_bIntersectRegion((*pDC).prgnRao, (*pDC).prgnVis, (*pDC).prgnAPI);
    } else {
        REGION_bCopy((*pDC).prgnRao, (*pDC).prgnVis);
    }

    REGION_bOffsetRgn((*pDC).prgnRao, (*pDC).ptlDCOrig.x, (*pDC).ptlDCOrig.y);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*pDC).erclClip) as _,
        core::ptr::addr_of!((*(*pDC).prgnRao).rdh.rcBound) as _,
        core::mem::size_of::<RECTL>() as _,
    );

    (*pDC).fs &= !(DC_DIRTY_RAO as i32);
    UpdateVisRgn(pDC);

    IntEngUpdateClipRegion(
        core::ptr::addr_of_mut!((*pDC).co),
        (*(*pDC).prgnRao).rdh.nCount,
        // C: `pDC->prgnRao->Buffer` — the POINTER VALUE (PRECTL rect array). A prior
        // translation passed addr_of!(Buffer) (the address of the pointer field
        // itself), so IntEngUpdateClipRegion memcpy'd nCount rects starting INSIDE
        // the REGION struct — overrunning its paged-pool allocation (intermittent
        // post-boot BUGCHECK 0x50 @ region+page-end when the next page was
        // uncommitted; silent garbage clip rects when it wasn't).
        (*(*pDC).prgnRao).Buffer as _,
        core::ptr::addr_of_mut!((*pDC).erclClip),
    );

    REGION_bOffsetRgn((*pDC).prgnRao, -(*pDC).ptlDCOrig.x, -(*pDC).ptlDCOrig.y);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        GNU GPL, See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Clip region functions
//  * FILE:             win32ss/gdi/ntgdi/cliprgn.c
//  * PROGRAMER:        Unknown
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// DBG_DEFAULT_CHANNEL(GdiClipRgn);
//
// VOID
// FASTCALL
// IntGdiReleaseRaoRgn(PDC pDC)
// {
//     INT Index = GDI_HANDLE_GET_INDEX(pDC->BaseObject.hHmgr);
//     PGDI_TABLE_ENTRY Entry = &GdiHandleTable->Entries[Index];
//     pDC->fs |= DC_DIRTY_RAO as i32;
//     Entry->Flags |= GDI_ENTRY_VALIDATE_VIS; // Need to validate Vis.
// }
//
// VOID
// FASTCALL
// IntGdiReleaseVisRgn(PDC pDC)
// {
//     IntGdiReleaseRaoRgn(pDC);
//     REGION_Delete(pDC->prgnVis);
//     pDC->prgnVis = prgnDefault; // Vis can not be NULL!!!
// }
//
// //
// // Updating Vis Region Attribute for DC Attributes.
// // BTW: This system region has an user attribute for it.
// //
// VOID
// FASTCALL
// UpdateVisRgn(
//     PDC pdc)
// {
//     INT Index = GDI_HANDLE_GET_INDEX(pdc->BaseObject.hHmgr);
//     PGDI_TABLE_ENTRY pEntry = &GdiHandleTable->Entries[Index];
//
//     /* Setup Vis Region Attribute information to User side */
//     pEntry->Flags |= GDI_ENTRY_VALIDATE_VIS;
//     pdc->pdcattr->VisRectRegion.iComplexity = REGION_GetRgnBox(pdc->prgnVis, &pdc->pdcattr->VisRectRegion.Rect);
//     pdc->pdcattr->VisRectRegion.AttrFlags = ATTR_RGN_VALID;
//     pEntry->Flags &= ~GDI_ENTRY_VALIDATE_VIS;
// }
//
// //
// //  Selecting Vis Region.
// //
// VOID
// FASTCALL
// GdiSelectVisRgn(
//     HDC hdc,
//     PREGION prgn)
// {
//     DC *dc;
//
//     if (!(dc = DC_LockDc(hdc)))
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return;
//     }
//
//     if (!prgn)
//     {
//        DPRINT1("SVR: Setting NULL Region\n");
//        IntGdiReleaseVisRgn(dc);
//        IntSetDefaultRegion(dc);
//        DC_UnlockDc(dc);
//        return;
//     }
//
//     dc->fs |= DC_DIRTY_RAO as i32;
//
//     ASSERT(dc->prgnVis != NULL);
//     ASSERT(prgn != NULL);
//
//     REGION_bCopy(dc->prgnVis, prgn);
//     REGION_bOffsetRgn(dc->prgnVis, -dc->ptlDCOrig.x, -dc->ptlDCOrig.y);
//
//     DC_UnlockDc(dc);
// }
//
// _Success_(return!=ERROR)
// int
// FASTCALL
// IntSelectClipRgn(
//     _In_ PDC dc,
//     _In_ PREGION prgn,
//     _In_ int fnMode)
// {
//     int Ret = ERROR;
//     PREGION prgnNClip, prgnOrigClip = dc->dclevel.prgnClip;
//
//     //
//     // No Coping Regions and no intersecting Regions or an User calling w NULL Region or have the Original Clip Region.
//     //
//     if (fnMode != RGN_COPY && (fnMode != RGN_AND || !prgn || prgnOrigClip))
//     {
//         prgnNClip = IntSysCreateRectpRgn(0, 0, 0, 0);
//
//         // Have Original Clip Region.
//         if (prgnOrigClip)
//         {
//            // This will fail on NULL prgn.
//            Ret = IntGdiCombineRgn(prgnNClip, prgnOrigClip, prgn, fnMode);
//
//            if (Ret)
//            {
//               REGION_Delete(prgnOrigClip);
//               dc->dclevel.prgnClip = prgnNClip;
//               IntGdiReleaseRaoRgn(dc);
//            }
//            else
//               REGION_Delete(prgnNClip);
//         }
//         else // NULL Original Clip Region, setup a new one and process mode.
//         {
//             PREGION prgnClip;
//             RECTL rcl;
// #if 0
//             PSURFACE pSurface;
//
//             // See IntSetDefaultRegion.
//
//             rcl.left   = 0;
//             rcl.top    = 0;
//             rcl.right  = dc->dclevel.sizl.cx;
//             rcl.bottom = dc->dclevel.sizl.cy;
//
//             //EngAcquireSemaphoreShared(pdc->ppdev->hsemDevLock);
//             if (dc->ppdev->flFlags & PDEV_META_DEVICE)
//             {
//                 pSurface = dc->dclevel.pSurface;
//                 if (pSurface && pSurface->flags & PDEV_SURFACE)
//                 {
//                    rcl.left   += dc->ppdev->ptlOrigion.x;
//                    rcl.top    += dc->ppdev->ptlOrigion.y;
//                    rcl.right  += dc->ppdev->ptlOrigion.x;
//                    rcl.bottom += dc->ppdev->ptlOrigion.y;
//                 }
//             }
//             //EngReleaseSemaphore(pdc->ppdev->hsemDevLock);
// //#if 0
//             rcl.left   += dc->ptlDCOrig.x;
//             rcl.top    += dc->ptlDCOrig.y;
//             rcl.right  += dc->ptlDCOrig.x;
//             rcl.bottom += dc->ptlDCOrig.y;
// #endif
//             REGION_GetRgnBox(dc->prgnVis, &rcl);
//
//             prgnClip = IntSysCreateRectpRgnIndirect(&rcl);
//
//             Ret = IntGdiCombineRgn(prgnNClip, prgnClip, prgn, fnMode);
//
//             if (Ret)
//             {
//                 dc->dclevel.prgnClip = prgnNClip;
//                 IntGdiReleaseRaoRgn(dc);
//             }
//             else
//                 REGION_Delete(prgnNClip);
//
//             REGION_Delete(prgnClip);
//         }
//         return Ret;
//     }
//
//     // Fall through to normal RectOS mode.
//
//     //
//     // Handle NULL Region and Original Clip Region.
//     //
//     if (!prgn)
//     {
//         if (prgnOrigClip)
//         {
//             REGION_Delete(dc->dclevel.prgnClip);
//             dc->dclevel.prgnClip = NULL;
//             IntGdiReleaseRaoRgn(dc);
//         }
//         return SIMPLEREGION;
//     }
//
//     //
//     // Combine the new Clip region with original Clip and caller Region.
//     //
//     if ( prgnOrigClip &&
//         (Ret = IntGdiCombineRgn(prgnOrigClip, prgn, NULL, RGN_COPY)) ) // Clip could fail.
//     {
//         IntGdiReleaseRaoRgn(dc);
//     }
//     else // NULL original Clip, just copy caller region to new.
//     {
//        prgnNClip = IntSysCreateRectpRgn(0, 0, 0, 0);
//        REGION_bCopy(prgnNClip, prgn);
//        Ret = REGION_Complexity(prgnNClip);
//        dc->dclevel.prgnClip = prgnNClip;
//        IntGdiReleaseRaoRgn(dc);
//     }
//     return Ret;
// }
//
// //
// // Call from Gdi Batch Subsystem.
// //
// // Was setup to just handle RGN_COPY only and return VOID, since this was called from Gdi32.
// // Tested in place of the other, complexity aside.
// //
//
// _Success_(return!=ERROR)
// int
// FASTCALL
// IntGdiExtSelectClipRect(
//     _In_ PDC dc,
//     _In_ PRECTL prcl,
//     _In_ int fnMode)
// {
//     int Ret = ERROR;
//     PREGION prgn;
//     RECTL rect;
//     BOOL NoRegion = fnMode & GDIBS_NORECT;
//
//     fnMode &= ~GDIBS_NORECT;
//
//     if (NoRegion) // NULL Region.
//     {
//         if (fnMode == RGN_COPY)
//         {
//            Ret = IntSelectClipRgn( dc, NULL, RGN_COPY);
//
//            if (dc->fs & DC_DIRTY_RAO)
//                CLIPPING_UpdateGCRegion(dc);
//
//            if (Ret) // Copy? Return Vis complexity.
//                Ret = REGION_Complexity(dc->prgnVis);
//         }
//     }
//     else // Have a box to build a region with.
//     {                             //       See CORE-16246 : Needs to be a one box Clip Region.
//         if ( dc->dclevel.prgnClip && (REGION_Complexity(dc->dclevel.prgnClip) == SIMPLEREGION) )
//         {
//             REGION_GetRgnBox(dc->dclevel.prgnClip, &rect);
//
//             if (prcl->left   == rect.left  &&
//                 prcl->top    == rect.top   &&
//                 prcl->right  == rect.right &&
//                 prcl->bottom == rect.bottom)
//             {
//                 return REGION_Complexity( dc->prgnRao ? dc->prgnRao : dc->prgnVis );
//             }
//         }
//
//         prgn = IntSysCreateRectpRgnIndirect(prcl);
//
//         Ret = IntSelectClipRgn( dc, prgn, fnMode);
//
//         if (dc->fs & DC_DIRTY_RAO)
//             CLIPPING_UpdateGCRegion(dc);
//
//         if (Ret) // In this case NtGdiExtSelectClipRgn tests pass.
//             Ret = REGION_Complexity( dc->prgnRao ? dc->prgnRao : dc->prgnVis );
//
//         REGION_Delete(prgn);
//     }
//     return Ret;
// }
//
// _Success_(return!=ERROR)
// int
// FASTCALL
// IntGdiExtSelectClipRgn(
//     _In_ PDC dc,
//     _In_ PREGION prgn,
//     _In_ int fnMode)
// {
//     int Ret = ERROR;
//
//     if (!prgn)
//     {
//         if (fnMode == RGN_COPY)
//         {
//            if ((Ret = IntSelectClipRgn( dc, NULL, RGN_COPY)))
//                Ret = REGION_Complexity(dc->prgnVis);
//         }
//     }
//     else
//     {
//         if ((Ret = IntSelectClipRgn( dc, prgn, fnMode)))
//         {
//             DPRINT("IntGdiExtSelectClipRgn A %d\n",Ret);
//             // Update the Rao, it must be this way for now.
//             if (dc->fs & DC_DIRTY_RAO)
//                 CLIPPING_UpdateGCRegion(dc);
//
//             Ret = REGION_Complexity( dc->prgnRao ? dc->prgnRao : dc->prgnVis );
//             DPRINT("IntGdiExtSelectClipRgn B %d\n",Ret);
//         }
//     }
//     return Ret;
// }
//
// int
// APIENTRY
// NtGdiExtSelectClipRgn(
//     HDC  hDC,
//     HRGN  hrgn,
//     int  fnMode)
// {
//     int retval;
//     DC *dc;
//     PREGION prgn;
//
//     if ( fnMode < RGN_AND || fnMode > RGN_COPY )
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return ERROR;
//     }
//
//     if (!(dc = DC_LockDc(hDC)))
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return ERROR;
//     }
//
//     prgn = REGION_LockRgn(hrgn);
//
//     if ((prgn == NULL) && (fnMode != RGN_COPY))
//     {
//         //EngSetLastError(ERROR_INVALID_HANDLE); doesn't set this.
//         retval = ERROR;
//     }
//     else
//     {
// #if 0   // Testing GDI Batch.
//         {
//             RECTL rcl;
//             if (prgn)
//                 REGION_GetRgnBox(prgn, &rcl);
//             else
//                 fnMode |= GDIBS_NORECT;
//             retval = IntGdiExtSelectClipRect(dc, &rcl, fnMode);
//         }
// #else
//         retval = IntGdiExtSelectClipRgn(dc, prgn, fnMode);
// #endif
//     }
//
//     if (prgn)
//         REGION_UnlockRgn(prgn);
//
//     DC_UnlockDc(dc);
//     return retval;
// }
//
// _Success_(return!=ERROR)
// INT
// FASTCALL
// GdiGetClipBox(
//     _In_ HDC hdc,
//     _Out_ LPRECT prc)
// {
//     PDC pdc;
//     INT iComplexity;
//
//     /* Lock the DC */
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         return ERROR;
//     }
//
//     /* Update RAO region if necessary */
//     if (pdc->fs & DC_DIRTY_RAO)
//         CLIPPING_UpdateGCRegion(pdc);
//
//     /* Check if we have a RAO region (intersection of API and VIS region) */
//     if (pdc->prgnRao)
//     {
//         /* We have a RAO region, use it */
//         iComplexity = REGION_GetRgnBox(pdc->prgnRao, prc);
//     }
//     else
//     {
//         /* No RAO region means no API region, so use the VIS region */
//         ASSERT(pdc->prgnVis);
//         iComplexity = REGION_GetRgnBox(pdc->prgnVis, prc);
//     }
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     /* Convert the rect to logical coordinates */
//     IntDPtoLP(pdc, (LPPOINT)prc, 2);
//
//     /* Return the complexity */
//     return iComplexity;
// }
//
// _Success_(return!=ERROR)
// INT
// APIENTRY
// NtGdiGetAppClipBox(
//     _In_ HDC hdc,
//     _Out_ LPRECT prc)
// {
//     RECT rect;
//     INT iComplexity;
//     NTSTATUS Status;
//
//     /* Call the internal function */
//     iComplexity = GdiGetClipBox(hdc, &rect);
//
//     if (iComplexity != ERROR)
//     {
//         Status = SafeProbeForWrite(prc, sizeof(RECT), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             iComplexity = ERROR;
//         }
//         else
//         {
//             *prc = rect;
//         }
//     }
//
//     /* Return the complexity */
//     return iComplexity;
// }
//
//
// INT
// APIENTRY
// NtGdiExcludeClipRect(
//     _In_ HDC hdc,
//     _In_ INT xLeft,
//     _In_ INT yTop,
//     _In_ INT xRight,
//     _In_ INT yBottom)
// {
//     INT iComplexity = ERROR;
//     RECTL rect;
//     PDC pdc;
//     PREGION prgn;
//
//     /* Lock the DC */
//     pdc = DC_LockDc(hdc);
//     if (pdc == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return ERROR;
//     }
//
//     /* Convert coordinates to device space */
//     rect.left   = xLeft;
//     rect.top    = yTop;
//     rect.right  = xRight;
//     rect.bottom = yBottom;
//     RECTL_vMakeWellOrdered(&rect);
//     IntLPtoDP(pdc, (LPPOINT)&rect, 2);
//
//     prgn = IntSysCreateRectpRgnIndirect(&rect);
//     if ( prgn )
//     {
//         iComplexity = IntSelectClipRgn( pdc, prgn, RGN_DIFF );
//
//         REGION_Delete(prgn);
//     }
//
//     /* Emulate Windows behavior */
//     if (iComplexity == SIMPLEREGION)
//         iComplexity = COMPLEXREGION;
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     return iComplexity;
// }
//
// INT
// APIENTRY
// NtGdiIntersectClipRect(
//     _In_ HDC hdc,
//     _In_ INT xLeft,
//     _In_ INT yTop,
//     _In_ INT xRight,
//     _In_ INT yBottom)
// {
//     INT iComplexity = ERROR;
//     RECTL rect;
//     PDC pdc;
//     PREGION prgn;
//
//     DPRINT("NtGdiIntersectClipRect(%p, %d,%d-%d,%d)\n",
//             hdc, xLeft, yTop, xRight, yBottom);
//
//     /* Lock the DC */
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return ERROR;
//     }
//
//     /* Convert coordinates to device space */
//     rect.left   = xLeft;
//     rect.top    = yTop;
//     rect.right  = xRight;
//     rect.bottom = yBottom;
//     RECTL_vMakeWellOrdered(&rect);
//     IntLPtoDP(pdc, (LPPOINT)&rect, 2);
//
//     prgn = IntSysCreateRectpRgnIndirect(&rect);
//     if ( prgn )
//     {
//         iComplexity = IntSelectClipRgn( pdc, prgn, RGN_AND );
//
//         REGION_Delete(prgn);
//     }
//
//     /* Emulate Windows behavior */
//     if ( iComplexity == SIMPLEREGION )
//         iComplexity = COMPLEXREGION;
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     return iComplexity;
// }
//
// INT
// APIENTRY
// NtGdiOffsetClipRgn(
//     _In_ HDC hdc,
//     _In_ INT xOffset,
//     _In_ INT yOffset)
// {
//     INT iComplexity;
//     PDC pdc;
//     POINTL apt[2];
//
//     /* Lock the DC */
//     pdc = DC_LockDc(hdc);
//     if (pdc == NULL)
//     {
//         if (!hdc) EngSetLastError(ERROR_INVALID_HANDLE);
//         return ERROR;
//     }
//
//     /* Check if we have a clip region */
//     if (pdc->dclevel.prgnClip != NULL)
//     {
//         /* Convert coordinates into device space. Note that we need to convert
//            2 coordinates to account for rotation / shear / offset */
//         apt[0].x = 0;
//         apt[0].y = 0;
//         apt[1].x = xOffset;
//         apt[1].y = yOffset;
//         IntLPtoDP(pdc, apt, 2);
//
//         /* Offset the clip region */
//         if (!REGION_bOffsetRgn(pdc->dclevel.prgnClip,
//                                apt[1].x - apt[0].x,
//                                apt[1].y - apt[0].y))
//         {
//             iComplexity = ERROR;
//         }
//         else
//         {
//             IntGdiReleaseRaoRgn(pdc);
//             UpdateVisRgn(pdc);
//             iComplexity = REGION_Complexity(pdc->dclevel.prgnClip);
//         }
//
//         /* Mark the RAO region as dirty */
//         pdc->fs |= DC_DIRTY_RAO as i32;
//     }
//     else
//     {
//         /* NULL means no clipping, i.e. the "whole" region */
//         iComplexity = SIMPLEREGION;
//     }
//
//     /* Unlock the DC and return the complexity */
//     DC_UnlockDc(pdc);
//     return iComplexity;
// }
//
// BOOL APIENTRY NtGdiPtVisible(HDC  hDC,
//                     int  X,
//                     int  Y)
// {
//     BOOL ret = FALSE;
//     PDC dc;
//     PREGION prgn;
//
//     if(!(dc = DC_LockDc(hDC)))
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     prgn = dc->prgnRao ? dc->prgnRao : dc->prgnVis;
//
//     if (prgn)
//     {
//         POINT pt = {X, Y};
//         IntLPtoDP(dc, &pt, 1);
//         ret = REGION_PtInRegion(prgn, pt.x, pt.y) as i32;
//     }
//
//     DC_UnlockDc(dc);
//
//     return ret;
// }
//
// BOOL
// APIENTRY
// NtGdiRectVisible(
//     HDC hDC,
//     LPRECT UnsafeRect)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PDC dc = DC_LockDc(hDC);
//     BOOL Result = FALSE;
//     RECTL Rect;
//     PREGION prgn;
//
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     Status = SafeProbeForRead(UnsafeRect, sizeof(RECT), 1);
//     if (NT_SUCCESS(Status))
//     {
//         Rect = *UnsafeRect;
//     }
//
//     if(!NT_SUCCESS(Status))
//     {
//         DC_UnlockDc(dc);
//         SetLastNtError(Status);
//         return FALSE;
//     }
//
//     if (dc->fs & DC_DIRTY_RAO)
//         CLIPPING_UpdateGCRegion(dc);
//
//     prgn = dc->prgnRao ? dc->prgnRao : dc->prgnVis;
//     if (prgn)
//     {
//          IntLPtoDP(dc, (LPPOINT)&Rect, 2);
//          Result = REGION_RectInRegion(prgn, &Rect);
//     }
//     DC_UnlockDc(dc);
//
//     return Result;
// }
//
// int
// FASTCALL
// IntGdiSetMetaRgn(PDC pDC)
// {
//     INT Ret = ERROR;
//
//     if ( pDC->dclevel.prgnMeta )
//     {
//         if ( pDC->dclevel.prgnClip )
//         {
//             PREGION prgn = IntSysCreateRectpRgn(0,0,0,0);
//             if ( prgn )
//             {
//                 if (REGION_bIntersectRegion(prgn, pDC->dclevel.prgnMeta, pDC->dclevel.prgnClip))
//                 {
//                     // See Restore/SaveDC
//                     REGION_Delete(pDC->dclevel.prgnMeta);
//                     pDC->dclevel.prgnMeta = prgn;
//
//                     REGION_Delete(pDC->dclevel.prgnClip);
//                     pDC->dclevel.prgnClip = NULL;
//                     IntGdiReleaseRaoRgn(pDC);
//
//                     Ret = REGION_Complexity(pDC->dclevel.prgnMeta);
//                 }
//                 else
//                     REGION_Delete(prgn);
//             }
//         }
//         else
//             Ret = REGION_Complexity(pDC->dclevel.prgnMeta);
//     }
//     else
//     {
//         if ( pDC->dclevel.prgnClip )
//         {
//             Ret = REGION_Complexity(pDC->dclevel.prgnClip);
//             pDC->dclevel.prgnMeta = pDC->dclevel.prgnClip;
//             pDC->dclevel.prgnClip = NULL;
//         }
//         else
//             Ret = SIMPLEREGION;
//     }
//
//     return Ret;
// }
//
//
// int APIENTRY NtGdiSetMetaRgn(HDC  hDC)
// {
//   INT Ret;
//   PDC pDC = DC_LockDc(hDC);
//
//   if (!pDC)
//   {
//      EngSetLastError(ERROR_INVALID_PARAMETER);
//      return ERROR;
//   }
//   Ret = IntGdiSetMetaRgn(pDC);
//
//   DC_UnlockDc(pDC);
//   return Ret;
// }
//
// VOID
// FASTCALL
// CLIPPING_UpdateGCRegion(PDC pDC)
// {
//     // Moved from Release Rao. Though it still gets over written.
//     RECTL_vSetEmptyRect(&pDC->erclClip);
//
//     /* Must have VisRgn set to a valid state! */
//     ASSERT (pDC->prgnVis);
// #if 0 // (w2k3) This works with limitations. (w7u) ReactOS relies on Rao.
//     if ( !pDC->dclevel.prgnClip &&
//          !pDC->dclevel.prgnMeta &&
//          !pDC->prgnAPI)
//     {
//         if (pDC->prgnRao)
//             REGION_Delete(pDC->prgnRao);
//         pDC->prgnRao = NULL;
//
//         REGION_bOffsetRgn(pDC->prgnVis, pDC->ptlDCOrig.x, pDC->ptlDCOrig.y);
//
//         RtlCopyMemory(&pDC->erclClip,
//                       &pDC->prgnVis->rdh.rcBound,
//                        sizeof(RECTL));
//
//         IntEngUpdateClipRegion(&pDC->co,
//                                 pDC->prgnVis->rdh.nCount,
//                                 pDC->prgnVis->Buffer,
//                                &pDC->erclClip);
//
//         REGION_bOffsetRgn(pDC->prgnVis, -pDC->ptlDCOrig.x, -pDC->ptlDCOrig.y);
//
//         pDC->fs &= ~DC_DIRTY_RAO;
//         UpdateVisRgn(pDC);
//         return;
//     }
// #endif
//     if (pDC->prgnAPI)
//     {
//         REGION_Delete(pDC->prgnAPI);
//         pDC->prgnAPI = NULL;
//     }
//
//     if (pDC->dclevel.prgnMeta || pDC->dclevel.prgnClip)
//     {
//         pDC->prgnAPI = IntSysCreateRectpRgn(0,0,0,0);
//         if (!pDC->prgnAPI)
//         {
//             /* Best we can do here. Better than crashing. */
//             ERR("Failed to allocate prgnAPI! Expect drawing issues!\n");
//             return;
//         }
//
//         if (!pDC->dclevel.prgnMeta)
//         {
//             REGION_bCopy(pDC->prgnAPI,
//                          pDC->dclevel.prgnClip);
//         }
//         else if (!pDC->dclevel.prgnClip)
//         {
//             REGION_bCopy(pDC->prgnAPI,
//                          pDC->dclevel.prgnMeta);
//         }
//         else
//         {
//             REGION_bIntersectRegion(pDC->prgnAPI,
//                                     pDC->dclevel.prgnClip,
//                                     pDC->dclevel.prgnMeta);
//         }
//     }
//
//     if (pDC->prgnRao)
//         REGION_Delete(pDC->prgnRao);
//
//     pDC->prgnRao = IntSysCreateRectpRgn(0,0,0,0);
//     if (!pDC->prgnRao)
//     {
//         /* Best we can do here. Better than crashing. */
//         ERR("Failed to allocate prgnRao! Expect drawing issues!\n");
//         return;
//     }
//
//     if (pDC->prgnAPI)
//     {
//         REGION_bIntersectRegion(pDC->prgnRao,
//                                 pDC->prgnVis,
//                                 pDC->prgnAPI);
//     }
//     else
//     {
//         REGION_bCopy(pDC->prgnRao,
//                      pDC->prgnVis);
//     }
//
//
//     REGION_bOffsetRgn(pDC->prgnRao, pDC->ptlDCOrig.x, pDC->ptlDCOrig.y);
//
//     RtlCopyMemory(&pDC->erclClip,
//                   &pDC->prgnRao->rdh.rcBound,
//                   sizeof(RECTL));
//
//     pDC->fs &= ~DC_DIRTY_RAO;
//     UpdateVisRgn(pDC);
//
//     // pDC->co should be used. Example, CLIPOBJ_cEnumStart uses XCLIPOBJ to build
//     // the rects from region objects rects in pClipRgn->Buffer.
//     // With pDC->co.pClipRgn->Buffer,
//     // pDC->co.pClipRgn = pDC->prgnRao ? pDC->prgnRao : pDC->prgnVis;
//
//     IntEngUpdateClipRegion(&pDC->co,
//                            pDC->prgnRao->rdh.nCount,
//                            pDC->prgnRao->Buffer,
//                            &pDC->erclClip);
//
//     REGION_bOffsetRgn(pDC->prgnRao, -pDC->ptlDCOrig.x, -pDC->ptlDCOrig.y);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
