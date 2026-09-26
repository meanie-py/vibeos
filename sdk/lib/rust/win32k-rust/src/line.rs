use win32k_sys::win32k::*;
use core::ffi::c_void;
use crate::brushbody::pbrushbody;
use crate::floatobj::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};

// Not in ntkrnl_sys::base; declared per-file, as the crate does elsewhere.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

#[inline(always)] unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) { core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n); }

// CRT (cdecl); GDI_Bezier is a real cdecl fn (bezier.c → bezier.rs).
extern "C" {
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn GDI_Bezier(pt: *const POINT, count: i32, n_out: *mut i32) -> *mut POINT;
}

// SetLastNtError: VOID FASTCALL (misc.h). RECTL_bUnionRect/bIntersectRect:
// BOOL FASTCALL (rect.c → rect.rs). Dual-cfg: fastcall on x86, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn RECTL_bUnionRect(dst: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> BOOL;
    fn RECTL_bIntersectRect(dst: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn RECTL_bUnionRect(dst: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> BOOL;
    fn RECTL_bIntersectRect(dst: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> BOOL;
}
// SafeProbeForWrite: NTSTATUS NTAPI (safe_user_io.h, validate-only under
// DUMMY_PSEH — does not raise). NTAPI = stdcall on i386 → extern "system".
extern "system" {
    fn SafeProbeForWrite(addr: *mut c_void, len: SIZE_T, align: u32) -> NTSTATUS;
}

// NT_SUCCESS(s) = s >= 0
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool { status >= 0 }
// ROP2_TO_MIX(Rop2) = ((Rop2) << 8) | (Rop2)  [inteng.h]; jROP2 is BYTE.
#[inline(always)]
fn ROP2_TO_MIX(rop2: u8) -> MIX { ((rop2 as MIX) << 8) | (rop2 as MIX) }
// min/max macros.
#[inline(always)] fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } }
#[inline(always)] fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }
// ASSERT_DC_PREPARED — debug assert; no-op.
#[inline(always)] unsafe fn ASSERT_DC_PREPARED(_dc: PDC) {}
// PATH_IsPathOpen (path.h macro) — inline over _DCLEVEL.
#[inline(always)]
unsafe fn PATH_IsPathOpen(dclevel: *mut c_void) -> BOOL {
    let d = dclevel as *mut _DCLEVEL;
    (((!(*d).hPath.is_null()) && ((*d).flPath & DCPATH_ACTIVE) != 0)) as BOOL
}
// PATH_UnlockPath (path.h macro) — GDIOBJ_vDereferenceObject.
#[inline(always)]
unsafe fn PATH_UnlockPath(ppath: POBJ) { GDIOBJ_vDereferenceObject(ppath); }
// IntIsEffectiveWidePen (pen.h macro) — inline via BRUSHBODY view.
#[inline(always)]
unsafe fn IntIsEffectiveWidePen(pbr: PBRUSH) -> BOOL {
    let b = pbrushbody(pbr);
    (((*b).lWidth > 1)
        && (((*b).flAttrs & BR_IS_OLDSTYLEPEN) != 0
            || ((*b).ulPenStyle & PS_TYPE_MASK) == PS_GEOMETRIC)) as BOOL
}

// coord.h transform chain — INTERNAL_APPLY_MATRIX + Int{LP,DP}toDP / CoordLPtoDP
// are all static inline / macros (no symbols). Inline via the FLOATOBJ shim.
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
// CoordLPtoDP(pdc, ppt) = INTERNAL_LPTODP(pdc, ppt, 1)  [no DC update]
#[inline(always)]
unsafe fn CoordLPtoDP(pdc: *mut DC, ppt: LPPOINT) -> BOOL {
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX, ppt, 1) as BOOL
}

// ProbeArrayForRead — FORCEINLINE (reactos probe.h). Validate-only under
// DUMMY_PSEH (the _SEH2_EXCEPT handler is dead; a raised probe would crash
// uncaught). Matches safe_user_io.h's validate-only contract.
#[inline(always)]
unsafe fn ProbeArrayForRead(_array_ptr: *const c_void, _item_size: usize, _item_count: u32, _alignment: usize) {}
// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// The C preamble contains only preprocessor directives and function attributes,
// which are handled per-function during translation.

const TAG_SHAPE: u32 = 0x70616853;

#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn AddPenLinesBounds(dc: PDC, count: i32, points: *mut POINT) {
    let mut join: u32;
    let mut endcap: u32;
    let mut bounds: RECTL = core::mem::zeroed();
    let mut rect: RECTL = core::mem::zeroed();
    let mut lWidth: i32;
    let pbrLine: PBRUSH;

    pbrLine = (*dc).dclevel.pbrLine;

    lWidth = 0;

    bounds.left = INT_MAX as i32;
    bounds.top = INT_MAX as i32;
    bounds.right = INT_MIN;
    bounds.bottom = INT_MIN;

    if (((*pbrushbody(pbrLine)).ulPenStyle & PS_TYPE_MASK) & PS_GEOMETRIC) != 0 || (*pbrushbody(pbrLine)).lWidth > 1 {
        lWidth = (*pbrushbody(pbrLine)).lWidth + 2;
        endcap = PS_ENDCAP_MASK & (*pbrushbody(pbrLine)).ulPenStyle;
        join = PS_JOIN_MASK & (*pbrushbody(pbrLine)).ulPenStyle;
        if join == PS_JOIN_MITER {
            lWidth *= 5;
            if endcap == PS_ENDCAP_SQUARE {
                lWidth = (lWidth * 3 + 1) / 2;
            }
        } else {
            if endcap == PS_ENDCAP_SQUARE {
                lWidth -= lWidth / 4;
            } else {
                lWidth = (lWidth + 1) / 2;
            }
        }
    }

    let mut count_var = count;
    let mut points_var = points;
    while count_var > 0 {
        rect.left = (*points_var).x - lWidth;
        rect.top = (*points_var).y - lWidth;
        rect.right = (*points_var).x + lWidth + 1;
        rect.bottom = (*points_var).y + lWidth + 1;
        RECTL_bUnionRect(&mut bounds, &bounds, &rect);
        points_var = points_var.offset(1);
        count_var -= 1;
    }

    {
        let mut rcRgn: RECTL = (*dc).erclClip;

        if RECTL_bIntersectRect(&mut rcRgn, &rcRgn, &bounds) != 0 {
            IntUpdateBoundsRect(dc, &mut rcRgn);
        } else {
            IntUpdateBoundsRect(dc, &mut bounds);
        }
    }
}

#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn AddPenLinesBounds(dc: PDC, count: i32, points: *mut POINT) {
    let mut join: u32;
    let mut endcap: u32;
    let mut bounds: RECTL = core::mem::zeroed();
    let mut rect: RECTL = core::mem::zeroed();
    let mut lWidth: i32;
    let pbrLine: PBRUSH;

    pbrLine = (*dc).dclevel.pbrLine;

    lWidth = 0;

    bounds.left = INT_MAX as i32;
    bounds.top = INT_MAX as i32;
    bounds.right = INT_MIN;
    bounds.bottom = INT_MIN;

    if (((*pbrushbody(pbrLine)).ulPenStyle & PS_TYPE_MASK) & PS_GEOMETRIC) != 0 || (*pbrushbody(pbrLine)).lWidth > 1 {
        lWidth = (*pbrushbody(pbrLine)).lWidth + 2;
        endcap = PS_ENDCAP_MASK & (*pbrushbody(pbrLine)).ulPenStyle;
        join = PS_JOIN_MASK & (*pbrushbody(pbrLine)).ulPenStyle;
        if join == PS_JOIN_MITER {
            lWidth *= 5;
            if endcap == PS_ENDCAP_SQUARE {
                lWidth = (lWidth * 3 + 1) / 2;
            }
        } else {
            if endcap == PS_ENDCAP_SQUARE {
                lWidth -= lWidth / 4;
            } else {
                lWidth = (lWidth + 1) / 2;
            }
        }
    }

    let mut count_var = count;
    let mut points_var = points;
    while count_var > 0 {
        rect.left = (*points_var).x - lWidth;
        rect.top = (*points_var).y - lWidth;
        rect.right = (*points_var).x + lWidth + 1;
        rect.bottom = (*points_var).y + lWidth + 1;
        RECTL_bUnionRect(&mut bounds, &bounds, &rect);
        points_var = points_var.offset(1);
        count_var -= 1;
    }

    {
        let mut rcRgn: RECTL = (*dc).erclClip;

        if RECTL_bIntersectRect(&mut rcRgn, &rcRgn, &bounds) != 0 {
            IntUpdateBoundsRect(dc, &mut rcRgn);
        } else {
            IntUpdateBoundsRect(dc, &mut bounds);
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiMoveToEx(
    dc: *mut DC,
    X: i32,
    Y: i32,
    Point: LPPOINT,
) -> BOOL {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    if !Point.is_null() {
        if (*pdcattr).ulDirty_ & DIRTY_PTLCURRENT != 0 {
            (*Point).x = (*pdcattr).ptfxCurrent.x;
            (*Point).y = (*pdcattr).ptfxCurrent.y;
            IntDPtoLP(dc, Point, 1);
        } else {
            (*Point).x = (*pdcattr).ptlCurrent.x;
            (*Point).y = (*pdcattr).ptlCurrent.y;
        }
    }
    (*pdcattr).ptlCurrent.x = X;
    (*pdcattr).ptlCurrent.y = Y;
    (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
    CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
    (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiMoveToEx(
    dc: *mut DC,
    X: i32,
    Y: i32,
    Point: LPPOINT,
) -> BOOL {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    if !Point.is_null() {
        if (*pdcattr).ulDirty_ & DIRTY_PTLCURRENT != 0 {
            (*Point).x = (*pdcattr).ptfxCurrent.x;
            (*Point).y = (*pdcattr).ptfxCurrent.y;
            IntDPtoLP(dc, Point, 1);
        } else {
            (*Point).x = (*pdcattr).ptlCurrent.x;
            (*Point).y = (*pdcattr).ptlCurrent.y;
        }
    }
    (*pdcattr).ptlCurrent.x = X;
    (*pdcattr).ptlCurrent.y = Y;
    (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
    CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
    (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GreMoveTo(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> BOOL {
    let Ret: BOOL;
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    Ret = IntGdiMoveToEx(dc, x, y, pptOut);
    DC_UnlockDc(dc);
    return Ret;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GreMoveTo(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> BOOL {
    let Ret: BOOL;
    let dc: PDC;
    dc = DC_LockDc(hdc);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    Ret = IntGdiMoveToEx(dc, x, y, pptOut);
    DC_UnlockDc(dc);
    return Ret;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCurrentPositionEx(dc: PDC, pt: LPPOINT) {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !pt.is_null() {
        if ((*pdcattr).ulDirty_ & DIRTY_PTFXCURRENT) != 0 {
            (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
            CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
            (*pdcattr).ulDirty_ &= !(DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
        }
        (*pt).x = (*pdcattr).ptlCurrent.x;
        (*pt).y = (*pdcattr).ptlCurrent.y;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCurrentPositionEx(dc: PDC, pt: LPPOINT) {
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if !pt.is_null() {
        if ((*pdcattr).ulDirty_ & DIRTY_PTFXCURRENT) != 0 {
            (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
            CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
            (*pdcattr).ulDirty_ &= !(DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
        }
        (*pt).x = (*pdcattr).ptlCurrent.x;
        (*pt).y = (*pdcattr).ptlCurrent.y;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiLineTo(dc: PDC, XEnd: i32, YEnd: i32) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut Ret: BOOL = 1;
    let mut pbrLine: PBRUSH;
    let mut Bounds: RECTL = core::mem::zeroed();
    let mut Points: [POINT; 2] = core::mem::zeroed();
    let pdcattr: PDC_ATTR;
    let mut pPath: PPATH;

    ASSERT_DC_PREPARED(dc);

    pdcattr = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        Ret = PATH_LineTo(dc, XEnd, YEnd);
    } else {
        psurf = (*dc).dclevel.pSurface;
        if psurf.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        Points[0].x = (*pdcattr).ptlCurrent.x;
        Points[0].y = (*pdcattr).ptlCurrent.y;
        Points[1].x = XEnd;
        Points[1].y = YEnd;

        IntLPtoDP(dc, Points.as_mut_ptr(), 2);

        Points[0].x += (*dc).ptlDCOrig.x;
        Points[0].y += (*dc).ptlDCOrig.y;
        Points[1].x += (*dc).ptlDCOrig.x;
        Points[1].y += (*dc).ptlDCOrig.y;

        Bounds.left = min(Points[0].x, Points[1].x);
        Bounds.top = min(Points[0].y, Points[1].y);
        Bounds.right = max(Points[0].x, Points[1].x);
        Bounds.bottom = max(Points[0].y, Points[1].y);

        pbrLine = (*dc).dclevel.pbrLine;

        if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
            AddPenLinesBounds(dc, 2, Points.as_mut_ptr());
        }

        if ((*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL) == 0 {
            if IntIsEffectiveWidePen(pbrLine) != 0 {
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();

                pPath = PATH_CreatePath(2);
                (*dc).dclevel.flPath |= DCPATH_ACTIVE;
                (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
                IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT);
                IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT, 1);

                PATH_MoveTo(dc, pPath);
                PATH_LineTo(dc, XEnd, YEnd);

                (*pPath).state = PATH_Closed;
                (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

                Ret = PATH_StrokePath(dc, pPath);

                PATH_UnlockPath(pPath as POBJ);
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();
            } else {
                Ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    Points[0].x,
                    Points[0].y,
                    Points[1].x,
                    Points[1].y,
                    core::ptr::addr_of_mut!(Bounds),
                    ROP2_TO_MIX((*pdcattr).jROP2),
                );
            }
        }
    }

    if Ret != 0 {
        (*pdcattr).ptlCurrent.x = XEnd;
        (*pdcattr).ptlCurrent.y = YEnd;
        (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
        CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
        (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiLineTo(dc: PDC, XEnd: i32, YEnd: i32) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut Ret: BOOL = 1;
    let mut pbrLine: PBRUSH;
    let mut Bounds: RECTL = core::mem::zeroed();
    let mut Points: [POINT; 2] = core::mem::zeroed();
    let pdcattr: PDC_ATTR;
    let mut pPath: PPATH;

    ASSERT_DC_PREPARED(dc);

    pdcattr = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        Ret = PATH_LineTo(dc, XEnd, YEnd);
    } else {
        psurf = (*dc).dclevel.pSurface;
        if psurf.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }

        Points[0].x = (*pdcattr).ptlCurrent.x;
        Points[0].y = (*pdcattr).ptlCurrent.y;
        Points[1].x = XEnd;
        Points[1].y = YEnd;

        IntLPtoDP(dc, Points.as_mut_ptr(), 2);

        Points[0].x += (*dc).ptlDCOrig.x;
        Points[0].y += (*dc).ptlDCOrig.y;
        Points[1].x += (*dc).ptlDCOrig.x;
        Points[1].y += (*dc).ptlDCOrig.y;

        Bounds.left = min(Points[0].x, Points[1].x);
        Bounds.top = min(Points[0].y, Points[1].y);
        Bounds.right = max(Points[0].x, Points[1].x);
        Bounds.bottom = max(Points[0].y, Points[1].y);

        pbrLine = (*dc).dclevel.pbrLine;

        if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
            AddPenLinesBounds(dc, 2, Points.as_mut_ptr());
        }

        if ((*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL) == 0 {
            if IntIsEffectiveWidePen(pbrLine) != 0 {
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();

                pPath = PATH_CreatePath(2);
                (*dc).dclevel.flPath |= DCPATH_ACTIVE;
                (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
                IntGetCurrentPositionEx(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT);
                IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT, 1);

                PATH_MoveTo(dc, pPath);
                PATH_LineTo(dc, XEnd, YEnd);

                (*pPath).state = PATH_Closed;
                (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

                Ret = PATH_StrokePath(dc, pPath);

                PATH_UnlockPath(pPath as POBJ);
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();
            } else {
                Ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    Points[0].x,
                    Points[0].y,
                    Points[1].x,
                    Points[1].y,
                    core::ptr::addr_of_mut!(Bounds),
                    ROP2_TO_MIX((*pdcattr).jROP2),
                );
            }
        }
    }

    if Ret != 0 {
        (*pdcattr).ptlCurrent.x = XEnd;
        (*pdcattr).ptlCurrent.y = YEnd;
        (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
        CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
        (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
    }

    Ret
}

unsafe fn IntGdiPolyBezier_impl(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    let mut ret: BOOL = 0;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_PolyBezier(dc, pt as *const POINTL, Count);
    }

    {
        let mut Pts: *mut POINT;
        let mut nOut: i32 = 0;

        Pts = GDI_Bezier(pt as *const POINT, Count as i32, &mut nOut);
        if !Pts.is_null() {
            ret = IntGdiPolyline(dc, Pts, nOut);
            ExFreePoolWithTag(Pts as *mut core::ffi::c_void, 0x72657a42);
        }
    }

    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolyBezier(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    IntGdiPolyBezier_impl(dc, pt, Count)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolyBezier(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    IntGdiPolyBezier_impl(dc, pt, Count)
}

unsafe fn IntGdiPolyBezierTo_impl(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    let mut ret: BOOL = 0;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        ret = PATH_PolyBezierTo(dc, pt as *const POINTL, Count);
    } else {
        let npt = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<POINT>() * (Count as usize + 1)) as SIZE_T,
            0x42455A49u32
        ) as *mut POINT;
        if !npt.is_null() {
            (*npt.offset(0)).x = (*pdcattr).ptlCurrent.x;
            (*npt.offset(0)).y = (*pdcattr).ptlCurrent.y;
            memcpy(
                npt.offset(1) as *mut core::ffi::c_void,
                pt as *const core::ffi::c_void,
                core::mem::size_of::<POINT>() * Count as usize
            );
            ret = IntGdiPolyBezier(dc, npt, Count + 1);
            ExFreePoolWithTag(npt as *mut core::ffi::c_void, 0x42455A49u32);
        }
    }
    if ret != 0 {
        (*pdcattr).ptlCurrent.x = (*pt.offset(Count as isize - 1)).x;
        (*pdcattr).ptlCurrent.y = (*pt.offset(Count as isize - 1)).y;
        (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
        CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
        (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
    }

    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolyBezierTo(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    IntGdiPolyBezierTo_impl(dc, pt, Count)
}
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolyBezierTo(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    IntGdiPolyBezierTo_impl(dc, pt, Count)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolyline(dc: PDC, pt: LPPOINT, Count: i32) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut pbrLine: *mut BRUSH;
    let mut Points: LPPOINT;
    let mut Ret: BOOL = 1;
    let mut i: i32;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    let mut pPath: PPATH;

    if (*dc).dclevel.pSurface.is_null() {
        return 1;
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    psurf = (*dc).dclevel.pSurface;

    pbrLine = (*dc).dclevel.pbrLine;

    if ((*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL) == 0 {
        Points = EngAllocMem(0, (Count * core::mem::size_of::<POINT>() as i32) as u32, 0x706D6554) as LPPOINT;
        if !Points.is_null() {
            RtlCopyMemory(Points as *mut core::ffi::c_void, pt as *const core::ffi::c_void, (Count * core::mem::size_of::<POINT>() as i32) as usize);
            IntLPtoDP(dc, Points, Count);

            i = 0;
            while i < Count {
                (*Points.offset(i as isize)).x += (*dc).ptlDCOrig.x;
                (*Points.offset(i as isize)).y += (*dc).ptlDCOrig.y;
                i += 1;
            }

            if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
                AddPenLinesBounds(dc, Count, Points);
            }

            if IntIsEffectiveWidePen(pbrLine) != 0 {
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();

                pPath = PATH_CreatePath(Count);
                (*dc).dclevel.flPath |= DCPATH_ACTIVE;
                (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
                (*pPath).pos = core::mem::transmute(*pt.offset(0));
                IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT, 1);

                PATH_MoveTo(dc, pPath);
                i = 1;
                while i < Count {
                    PATH_LineTo(dc, (*pt.offset(i as isize)).x, (*pt.offset(i as isize)).y);
                    i += 1;
                }

                (*pPath).state = PATH_Closed;
                (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

                Ret = PATH_StrokePath(dc, pPath);

                PATH_UnlockPath(pPath as POBJ);
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();
            } else {
                Ret = IntEngPolyline(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    Points as *mut POINTL,
                    Count,
                    ROP2_TO_MIX((*pdcattr).jROP2)
                );
            }
            EngFreeMem(Points as *mut core::ffi::c_void);
        } else {
            Ret = 0;
        }
    }

    DC_vFinishBlit(dc, core::ptr::null_mut());

    return Ret;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolyline(dc: PDC, pt: LPPOINT, Count: i32) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut pbrLine: *mut BRUSH;
    let mut Points: LPPOINT;
    let mut Ret: BOOL = 1;
    let mut i: i32;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    let mut pPath: PPATH;

    if (*dc).dclevel.pSurface.is_null() {
        return 1;
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    psurf = (*dc).dclevel.pSurface;

    pbrLine = (*dc).dclevel.pbrLine;

    if ((*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL) == 0 {
        Points = EngAllocMem(0, (Count * core::mem::size_of::<POINT>() as i32) as u32, 0x706D6554) as LPPOINT;
        if !Points.is_null() {
            RtlCopyMemory(Points as *mut core::ffi::c_void, pt as *const core::ffi::c_void, (Count * core::mem::size_of::<POINT>() as i32) as usize);
            IntLPtoDP(dc, Points, Count);

            i = 0;
            while i < Count {
                (*Points.offset(i as isize)).x += (*dc).ptlDCOrig.x;
                (*Points.offset(i as isize)).y += (*dc).ptlDCOrig.y;
                i += 1;
            }

            if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
                AddPenLinesBounds(dc, Count, Points);
            }

            if IntIsEffectiveWidePen(pbrLine) != 0 {
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();

                pPath = PATH_CreatePath(Count);
                (*dc).dclevel.flPath |= DCPATH_ACTIVE;
                (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
                (*pPath).pos = core::mem::transmute(*pt.offset(0));
                IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as LPPOINT, 1);

                PATH_MoveTo(dc, pPath);
                i = 1;
                while i < Count {
                    PATH_LineTo(dc, (*pt.offset(i as isize)).x, (*pt.offset(i as isize)).y);
                    i += 1;
                }

                (*pPath).state = PATH_Closed;
                (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

                Ret = PATH_StrokePath(dc, pPath);

                PATH_UnlockPath(pPath as POBJ);
                PATH_Delete((*dc).dclevel.hPath);
                (*dc).dclevel.hPath = core::ptr::null_mut();
            } else {
                Ret = IntEngPolyline(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    Points as *mut POINTL,
                    Count,
                    ROP2_TO_MIX((*pdcattr).jROP2)
                );
            }
            EngFreeMem(Points as *mut core::ffi::c_void);
        } else {
            Ret = 0;
        }
    }

    DC_vFinishBlit(dc, core::ptr::null_mut());

    return Ret;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolylineTo(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    let mut ret: BOOL = 0;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        ret = PATH_PolylineTo(dc, pt as *const POINTL, Count);
    } else {
        let pts = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<POINT>() * (Count as usize + 1)) as SIZE_T,
            TAG_SHAPE
        ) as *mut POINT;
        if !pts.is_null() {
            (*pts.offset(0)).x = (*pdcattr).ptlCurrent.x;
            (*pts.offset(0)).y = (*pdcattr).ptlCurrent.y;
            memcpy(
                pts.offset(1) as *mut core::ffi::c_void,
                pt as *const core::ffi::c_void,
                core::mem::size_of::<POINT>() * Count as usize
            );
            ret = IntGdiPolyline(dc, pts, (Count + 1) as i32);
            ExFreePoolWithTag(pts as *mut core::ffi::c_void, TAG_SHAPE);
        }
    }
    if ret != 0 {
        (*pdcattr).ptlCurrent.x = (*pt.offset(Count as isize - 1)).x;
        (*pdcattr).ptlCurrent.y = (*pt.offset(Count as isize - 1)).y;
        (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
        CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
        (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
    }

    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolylineTo(dc: *mut DC, pt: LPPOINT, Count: u32) -> BOOL {
    let mut ret: BOOL = 0;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        ret = PATH_PolylineTo(dc, pt as *const POINTL, Count);
    } else {
        let pts = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<POINT>() * (Count as usize + 1)) as SIZE_T,
            TAG_SHAPE
        ) as *mut POINT;
        if !pts.is_null() {
            (*pts.offset(0)).x = (*pdcattr).ptlCurrent.x;
            (*pts.offset(0)).y = (*pdcattr).ptlCurrent.y;
            memcpy(
                pts.offset(1) as *mut core::ffi::c_void,
                pt as *const core::ffi::c_void,
                core::mem::size_of::<POINT>() * Count as usize
            );
            ret = IntGdiPolyline(dc, pts, (Count + 1) as i32);
            ExFreePoolWithTag(pts as *mut core::ffi::c_void, TAG_SHAPE);
        }
    }
    if ret != 0 {
        (*pdcattr).ptlCurrent.x = (*pt.offset(Count as isize - 1)).x;
        (*pdcattr).ptlCurrent.y = (*pt.offset(Count as isize - 1)).y;
        (*pdcattr).ptfxCurrent = (*pdcattr).ptlCurrent;
        CoordLPtoDP(dc, core::ptr::addr_of_mut!((*pdcattr).ptfxCurrent) as LPPOINT);
        (*pdcattr).ulDirty_ &= !(DIRTY_PTLCURRENT | DIRTY_PTFXCURRENT | DIRTY_STYLESTATE);
    }

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiPolyPolyline(
    dc: *mut DC,
    pt: LPPOINT,
    PolyPoints: *mut ULONG,
    Count: DWORD,
) -> BOOL {
    let mut i: ULONG;
    let mut pts: LPPOINT;
    let mut pc: *mut ULONG;
    let mut ret: BOOL = 0;
    pts = pt;
    pc = PolyPoints;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_PolyPolyline(dc, pt as *const POINTL, PolyPoints, Count);
    }
    i = 0;
    while i < Count {
        ret = IntGdiPolyline(dc, pts, *pc as i32);
        if ret == 0 {
            return ret;
        }
        pts = pts.offset(*pc as isize);
        pc = pc.offset(1);
        i += 1;
    }

    return ret;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiPolyPolyline(
    dc: *mut DC,
    pt: LPPOINT,
    PolyPoints: *mut ULONG,
    Count: DWORD,
) -> BOOL {
    let mut i: ULONG;
    let mut pts: LPPOINT;
    let mut pc: *mut ULONG;
    let mut ret: BOOL = 0;
    pts = pt;
    pc = PolyPoints;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_PolyPolyline(dc, pt as *const POINTL, PolyPoints, Count);
    }
    i = 0;
    while i < Count {
        ret = IntGdiPolyline(dc, pts, *pc as i32);
        if ret == 0 {
            return ret;
        }
        pts = pts.offset(*pc as isize);
        pc = pc.offset(1);
        i += 1;
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiLineTo(
    hDC: HDC,
    XEnd: i32,
    YEnd: i32,
) -> i32 {
    let dc: *mut DC;
    let mut Ret: i32;
    let mut rcLockRect: RECT = core::mem::zeroed();

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    rcLockRect.left = (*(*dc).pdcattr).ptlCurrent.x;
    rcLockRect.top = (*(*dc).pdcattr).ptlCurrent.y;
    rcLockRect.right = XEnd;
    rcLockRect.bottom = YEnd;

    IntLPtoDP(dc, &mut rcLockRect as *mut RECT as PPOINT, 2);

    rcLockRect.left += (*dc).ptlDCOrig.x;
    rcLockRect.top += (*dc).ptlDCOrig.y;
    rcLockRect.right += (*dc).ptlDCOrig.x;
    rcLockRect.bottom += (*dc).ptlDCOrig.y;

    DC_vPrepareDCsForBlit(dc, core::ptr::addr_of_mut!(rcLockRect) as *const RECTL, core::ptr::null_mut(), core::ptr::null_mut());

    Ret = IntGdiLineTo(dc, XEnd, YEnd);

    DC_vFinishBlit(dc, core::ptr::null_mut());

    DC_UnlockDc(dc);
    Ret
}

unsafe fn GdiPolyDraw(
    hdc: HDC,
    lppt: LPPOINT,
    lpbTypes: LPBYTE,
    cCount: u32,
) -> BOOL {
    let dc: PDC;
    let pdcattr: PDC_ATTR;
    let mut bzr: [POINT; 4] = core::mem::zeroed();
    let mut line_pts: PPOINT;
    let mut line_pts_old: PPOINT;
    let mut bzr_pts: PPOINT;
    let mut num_pts: i32;
    let mut num_bzr_pts: i32 = 0;
    let mut space: i32;
    let mut space_old: i32;
    let mut size: i32;
    let mut i: u32;
    let mut result: BOOL = 0;

    dc = DC_LockDc(hdc);
    if dc.is_null() {
        return 0;
    }
    pdcattr = (*dc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    if cCount == 0 {
        DC_UnlockDc(dc);
        return 1;
    }

    line_pts = core::ptr::null_mut();
    line_pts_old = core::ptr::null_mut();
    bzr_pts = core::ptr::null_mut();

    'cleanup: {
        if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
            result = PATH_PolyDraw(dc, lppt as *const POINTL, lpbTypes as *const u8, cCount);
            break 'cleanup;
        }

        i = 0;
        while i < cCount {
            match *lpbTypes.offset(i as isize) as u32 {
                PT_MOVETO | PT_LINETO => {}
                x if x == (PT_LINETO | PT_CLOSEFIGURE) => {}
                PT_BEZIERTO => {
                    if (i + 2 < cCount)
                        && (*lpbTypes.offset((i + 1) as isize) as u32 == PT_BEZIERTO)
                        && ((*lpbTypes.offset((i + 2) as isize) as u32 & !PT_CLOSEFIGURE) == PT_BEZIERTO)
                    {
                        i += 2;
                    } else {
                        break 'cleanup;
                    }
                }
                _ => {
                    break 'cleanup;
                }
            }
            i += 1;
        }

        space = cCount as i32 + 300;
        line_pts = ExAllocatePoolWithTag(
            PagedPool,
            (space as usize * core::mem::size_of::<POINT>()) as SIZE_T,
            0x45504148u32,
        ) as PPOINT;
        if line_pts.is_null() {
            result = 0;
            break 'cleanup;
        }

        num_pts = 1;

        (*line_pts.offset(0)).x = (*pdcattr).ptlCurrent.x;
        (*line_pts.offset(0)).y = (*pdcattr).ptlCurrent.y;

        i = 0;
        while i < cCount {
            match *lpbTypes.offset(i as isize) as u32 {
                PT_MOVETO => {
                    if num_pts >= 2 {
                        IntGdiPolyline(dc, line_pts, num_pts);
                    }
                    num_pts = 0;
                    *line_pts.offset(num_pts as isize) = *lppt.offset(i as isize);
                    num_pts += 1;
                }
                x if x == PT_LINETO || x == (PT_LINETO | PT_CLOSEFIGURE) => {
                    *line_pts.offset(num_pts as isize) = *lppt.offset(i as isize);
                    num_pts += 1;
                }
                PT_BEZIERTO => {
                    bzr[0].x = (*line_pts.offset((num_pts - 1) as isize)).x;
                    bzr[0].y = (*line_pts.offset((num_pts - 1) as isize)).y;
                    core::ptr::copy_nonoverlapping(
                        lppt.offset(i as isize),
                        bzr.as_mut_ptr().offset(1),
                        3,
                    );

                    bzr_pts = GDI_Bezier(bzr.as_mut_ptr(), 4, &mut num_bzr_pts as *mut i32);
                    if !bzr_pts.is_null() {
                        size = num_pts + (cCount as i32 - i as i32) + num_bzr_pts;
                        if space < size {
                            space_old = space;
                            space = size * 2;
                            line_pts_old = line_pts;
                            line_pts = ExAllocatePoolWithTag(
                                PagedPool,
                                (space as usize * core::mem::size_of::<POINT>()) as SIZE_T,
                                0x45504148u32,
                            ) as PPOINT;
                            if line_pts.is_null() {
                                break 'cleanup;
                            }
                            core::ptr::copy_nonoverlapping(
                                line_pts_old,
                                line_pts,
                                space_old as usize,
                            );
                            ExFreePoolWithTag(line_pts_old as *mut core::ffi::c_void, 0x45504148u32);
                            line_pts_old = core::ptr::null_mut();
                        }
                        core::ptr::copy_nonoverlapping(
                            bzr_pts.offset(1),
                            line_pts.offset(num_pts as isize),
                            (num_bzr_pts - 1) as usize,
                        );
                        num_pts += num_bzr_pts - 1;
                        ExFreePoolWithTag(bzr_pts as *mut core::ffi::c_void, 0x525A4542u32);
                        bzr_pts = core::ptr::null_mut();
                    }
                    i += 2;
                }
                _ => {}
            }
            if (*lpbTypes.offset(i as isize) as u32 & PT_CLOSEFIGURE) != 0 {
                *line_pts.offset(num_pts as isize) = *line_pts.offset(0);
                num_pts += 1;
            }
            i += 1;
        }

        if num_pts >= 2 {
            IntGdiPolyline(dc, line_pts, num_pts);
        }
        IntGdiMoveToEx(
            dc,
            (*line_pts.offset((num_pts - 1) as isize)).x,
            (*line_pts.offset((num_pts - 1) as isize)).y,
            core::ptr::null_mut(),
        );
        result = 1;
    }

    if !line_pts.is_null() {
        ExFreePoolWithTag(line_pts as *mut core::ffi::c_void, 0x45504148u32);
    }

    if !line_pts_old.is_null() && line_pts_old != line_pts {
        ExFreePoolWithTag(line_pts_old as *mut core::ffi::c_void, 0x45504148u32);
    }

    if !bzr_pts.is_null() {
        ExFreePoolWithTag(bzr_pts as *mut core::ffi::c_void, 0x525A4542u32);
    }

    DC_UnlockDc(dc);

    result
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPolyDraw(
    hdc: HDC,
    ppt: LPPOINT,
    pjAttr: LPBYTE,
    cpt: u32,
) -> i32 {
    let pjBuffer: PBYTE;
    let pptSafe: PPOINT;
    let pjSafe: PBYTE;
    let cjSizePt: usize;
    let mut bResult: i32;

    if cpt == 0 {
        ERR!("cpt is 0\n");
        return 0;
    }

    if cpt > ((MAXULONG - cpt) / core::mem::size_of::<POINT>() as u32) {
        ERR!("cpt is too large\n", cpt);
        return 0;
    }

    cjSizePt = (cpt as usize) * core::mem::size_of::<POINT>();

    pjBuffer = ExAllocatePoolWithTag(PagedPool, (cjSizePt + cpt as usize) as SIZE_T, TAG_SHAPE) as PBYTE;
    if pjBuffer.is_null() {
        ERR!("Failed to allocate buffer\n");
        return 0;
    }

    pptSafe = pjBuffer as PPOINT;
    pjSafe = pjBuffer.add(cjSizePt);

    ProbeArrayForRead(ppt as *const core::ffi::c_void, core::mem::size_of::<POINT>(), cpt, core::mem::size_of::<u32>());
    ProbeArrayForRead(pjAttr as *const core::ffi::c_void, core::mem::size_of::<u8>(), cpt, core::mem::size_of::<u8>());

    core::ptr::copy_nonoverlapping(ppt as *const u8, pptSafe as *mut u8, cjSizePt);
    core::ptr::copy_nonoverlapping(pjAttr as *const u8, pjSafe as *mut u8, cpt as usize);

    bResult = GdiPolyDraw(hdc, pptSafe, pjSafe, cpt);

    ExFreePoolWithTag(pjBuffer as *mut core::ffi::c_void, TAG_SHAPE);

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiMoveTo(
    hdc: HDC,
    x: i32,
    y: i32,
    pptOut: LPPOINT,
) -> BOOL {
    let pdc: PDC = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    let mut Point: POINT = core::mem::zeroed();
    let mut Ret: BOOL = IntGdiMoveToEx(pdc, x, y, &mut Point);

    if Ret != 0 && !pptOut.is_null() {
        let Status: NTSTATUS = SafeProbeForWrite(pptOut as *mut core::ffi::c_void, core::mem::size_of::<POINT>() as SIZE_T, 1);
        // Validate-only probe; the publish is the access. Two branches so the error
        // reported is the one that happened.
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            Ret = 0;
        } else if !crate::copy::publish_to_user(
            pptOut as *mut core::ffi::c_void,
            &Point as *const POINT as *const core::ffi::c_void,
            core::mem::size_of::<POINT>(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            Ret = 0;
        }
    }

    DC_UnlockDc(pdc);

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32k Subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/line.c
//  * PURPOSE:         Line functions
//  * PROGRAMMERS:     ...
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// DBG_DEFAULT_CHANNEL(GdiLine);
//
// // Some code from the WINE project source (www.winehq.com)
//
// VOID FASTCALL
// AddPenLinesBounds(PDC dc, int count, POINT *points)
// {
//     DWORD join, endcap;
//     RECTL bounds, rect;
//     LONG lWidth;
//     PBRUSH pbrLine;
//
//     /* Get BRUSH from current pen. */
//     pbrLine = dc->dclevel.pbrLine;
//     ASSERT(pbrLine);
//
//     lWidth = 0;
//
//     // Setup bounds
//     bounds.left = bounds.top = INT_MAX as i32;
//     bounds.right = bounds.bottom = INT_MIN;
//
//     if (((pbrLine->ulPenStyle & PS_TYPE_MASK) & PS_GEOMETRIC) || pbrLine->lWidth > 1)
//     {
//         /* Windows uses some heuristics to estimate the distance from the point that will be painted */
//         lWidth = pbrLine->lWidth + 2;
//         endcap = (PS_ENDCAP_MASK & pbrLine->ulPenStyle);
//         join   = (PS_JOIN_MASK   & pbrLine->ulPenStyle);
//         if (join == PS_JOIN_MITER)
//         {
//            lWidth *= 5;
//            if (endcap == PS_ENDCAP_SQUARE) lWidth = (lWidth * 3 + 1) / 2;
//         }
//         else
//         {
//            if (endcap == PS_ENDCAP_SQUARE) lWidth -= lWidth / 4;
//            else lWidth = (lWidth + 1) / 2;
//         }
//     }
//
//     while (count-- > 0)
//     {
//         rect.left   = points->x - lWidth;
//         rect.top    = points->y - lWidth;
//         rect.right  = points->x + lWidth + 1;
//         rect.bottom = points->y + lWidth + 1;
//         RECTL_bUnionRect(&bounds, &bounds, &rect);
//         points++;
//     }
//
//     DPRINT("APLB dc %p l %d t %d\n",dc,rect.left,rect.top);
//     DPRINT("                 r %d b %d\n",rect.right,rect.bottom);
//
//     {
//        RECTL rcRgn = dc->erclClip; // Use the clip box for now.
//
//        if (RECTL_bIntersectRect( &rcRgn, &rcRgn, &bounds ))
//            IntUpdateBoundsRect(dc, &rcRgn);
//        else
//            IntUpdateBoundsRect(dc, &bounds);
//     }
// }
//
// // Should use Fx in Point
// //
// BOOL FASTCALL
// IntGdiMoveToEx(DC      *dc,
//                int     X,
//                int     Y,
//                LPPOINT Point)
// {
//     PDC_ATTR pdcattr = dc->pdcattr;
//     if ( Point )
//     {
//         if ( pdcattr->ulDirty_ & DIRTY_PTLCURRENT ) // Double hit!
//         {
//             Point->x = pdcattr->ptfxCurrent.x; // ret prev before change.
//             Point->y = pdcattr->ptfxCurrent.y;
//             IntDPtoLP ( dc, Point, 1);         // Reconvert back.
//         }
//         else
//         {
//             Point->x = pdcattr->ptlCurrent.x;
//             Point->y = pdcattr->ptlCurrent.y;
//         }
//     }
//     pdcattr->ptlCurrent.x = X;
//     pdcattr->ptlCurrent.y = Y;
//     pdcattr->ptfxCurrent = pdcattr->ptlCurrent;
//     CoordLPtoDP(dc, &pdcattr->ptfxCurrent); // Update fx
//     pdcattr->ulDirty_ &= ~(DIRTY_PTLCURRENT|DIRTY_PTFXCURRENT|DIRTY_STYLESTATE);
//
//     return TRUE;
// }
//
// BOOL FASTCALL
// GreMoveTo( HDC hdc,
//            INT x,
//            INT y,
//            LPPOINT pptOut)
// {
//    BOOL Ret;
//    PDC dc;
//    if (!(dc = DC_LockDc(hdc)))
//    {
//       EngSetLastError(ERROR_INVALID_HANDLE);
//       return FALSE;
//    }
//    Ret = IntGdiMoveToEx(dc, x, y, pptOut);
//    DC_UnlockDc(dc);
//    return Ret;
// }
//
// // Should use Fx in pt
// //
// VOID FASTCALL
// IntGetCurrentPositionEx(PDC dc, LPPOINT pt)
// {
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     if ( pt )
//     {
//         if (pdcattr->ulDirty_ & DIRTY_PTFXCURRENT)
//         {
//             pdcattr->ptfxCurrent = pdcattr->ptlCurrent;
//             CoordLPtoDP(dc, &pdcattr->ptfxCurrent); // Update fx
//             pdcattr->ulDirty_ &= ~(DIRTY_PTFXCURRENT|DIRTY_STYLESTATE);
//         }
//         pt->x = pdcattr->ptlCurrent.x;
//         pt->y = pdcattr->ptlCurrent.y;
//     }
// }
//
// BOOL FASTCALL
// IntGdiLineTo(DC  *dc,
//              int XEnd,
//              int YEnd)
// {
//     SURFACE *psurf;
//     BOOL      Ret = TRUE;
//     PBRUSH pbrLine;
//     RECTL     Bounds;
//     POINT     Points[2];
//     PDC_ATTR  pdcattr;
//     PPATH     pPath;
//
//     ASSERT_DC_PREPARED(dc);
//
//     pdcattr = dc->pdcattr;
//
//     if (PATH_IsPathOpen(dc->dclevel))
//     {
//         Ret = PATH_LineTo(dc, XEnd, YEnd);
//     }
//     else
//     {
//         psurf = dc->dclevel.pSurface;
//         if (NULL == psurf)
//         {
//             EngSetLastError(ERROR_INVALID_HANDLE);
//             return FALSE;
//         }
//
//         Points[0].x = pdcattr->ptlCurrent.x;
//         Points[0].y = pdcattr->ptlCurrent.y;
//         Points[1].x = XEnd;
//         Points[1].y = YEnd;
//
//         IntLPtoDP(dc, Points, 2);
//
//         /* The DCOrg is in device coordinates */
//         Points[0].x += dc->ptlDCOrig.x;
//         Points[0].y += dc->ptlDCOrig.y;
//         Points[1].x += dc->ptlDCOrig.x;
//         Points[1].y += dc->ptlDCOrig.y;
//
//         Bounds.left = min(Points[0].x, Points[1].x);
//         Bounds.top = min(Points[0].y, Points[1].y);
//         Bounds.right = max(Points[0].x, Points[1].x);
//         Bounds.bottom = max(Points[0].y, Points[1].y);
//
//         /* Get BRUSH from current pen. */
//         pbrLine = dc->dclevel.pbrLine;
//         ASSERT(pbrLine);
//
//         if (dc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//         {
//            DPRINT("Bounds dc %p l %d t %d\n",dc,Bounds.left,Bounds.top);
//            DPRINT("                   r %d b %d\n",Bounds.right,Bounds.bottom);
//            AddPenLinesBounds(dc, 2, Points);
//         }
//
//         if (!(pbrLine->flAttrs & BR_IS_NULL))
//         {
//             if (IntIsEffectiveWidePen(pbrLine))
//             {
//                 /* Clear the path */
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//
//                 /* Begin a path */
//                 pPath = PATH_CreatePath(2);
//                 dc->dclevel.flPath |= DCPATH_ACTIVE;
//                 dc->dclevel.hPath = pPath->BaseObject.hHmgr;
//                 IntGetCurrentPositionEx(dc, &pPath->pos);
//                 IntLPtoDP(dc, &pPath->pos, 1);
//
//                 PATH_MoveTo(dc, pPath);
//                 PATH_LineTo(dc, XEnd, YEnd);
//
//                 /* Close the path */
//                 pPath->state = PATH_Closed;
//                 dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//                 /* Actually stroke a path */
//                 Ret = PATH_StrokePath(dc, pPath);
//
//                 /* Clear the path */
//                 PATH_UnlockPath(pPath as POBJ);
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//             }
//             else
//             {
//                 Ret = IntEngLineTo(&psurf->SurfObj,
//                                    (CLIPOBJ *)&dc->co,
//                                    &dc->eboLine.BrushObject,
//                                    Points[0].x, Points[0].y,
//                                    Points[1].x, Points[1].y,
//                                    &Bounds,
//                                    ROP2_TO_MIX(pdcattr->jROP2));
//             }
//         }
//     }
//
//     if (Ret)
//     {
//         pdcattr->ptlCurrent.x = XEnd;
//         pdcattr->ptlCurrent.y = YEnd;
//         pdcattr->ptfxCurrent = pdcattr->ptlCurrent;
//         CoordLPtoDP(dc, &pdcattr->ptfxCurrent); // Update fx
//         pdcattr->ulDirty_ &= ~(DIRTY_PTLCURRENT|DIRTY_PTFXCURRENT|DIRTY_STYLESTATE);
//     }
//
//     return Ret;
// }
//
// BOOL FASTCALL
// IntGdiPolyBezier(DC      *dc,
//                  LPPOINT pt,
//                  DWORD   Count)
// {
//     BOOL ret = FALSE; // Default to FAILURE
//
//     if ( PATH_IsPathOpen(dc->dclevel) )
//     {
//         return PATH_PolyBezier ( dc, pt, Count );
//     }
//
//     /* We'll convert it into line segments and draw them using Polyline */
//     {
//         POINT *Pts;
//         INT nOut;
//
//         Pts = GDI_Bezier ( pt, Count, &nOut );
//         if ( Pts )
//         {
//             ret = IntGdiPolyline(dc, Pts, nOut);
//             ExFreePoolWithTag(Pts, TAG_BEZIER);
//         }
//     }
//
//     return ret;
// }
//
// BOOL FASTCALL
// IntGdiPolyBezierTo(DC      *dc,
//                    LPPOINT pt,
//                    DWORD  Count)
// {
//     BOOL ret = FALSE; // Default to failure
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     if ( PATH_IsPathOpen(dc->dclevel) )
//         ret = PATH_PolyBezierTo ( dc, pt, Count );
//     else /* We'll do it using PolyBezier */
//     {
//         POINT *npt;
//         npt = ExAllocatePoolWithTag(PagedPool,
//                                     sizeof(POINT) * (Count + 1),
//                                     TAG_BEZIER);
//         if ( npt )
//         {
//             npt[0].x = pdcattr->ptlCurrent.x;
//             npt[0].y = pdcattr->ptlCurrent.y;
//             memcpy(npt + 1, pt, sizeof(POINT) * Count);
//             ret = IntGdiPolyBezier(dc, npt, Count+1);
//             ExFreePoolWithTag(npt, TAG_BEZIER);
//         }
//     }
//     if ( ret )
//     {
//         pdcattr->ptlCurrent.x = pt[Count-1].x;
//         pdcattr->ptlCurrent.y = pt[Count-1].y;
//         pdcattr->ptfxCurrent = pdcattr->ptlCurrent;
//         CoordLPtoDP(dc, &pdcattr->ptfxCurrent); // Update fx
//         pdcattr->ulDirty_ &= ~(DIRTY_PTLCURRENT|DIRTY_PTFXCURRENT|DIRTY_STYLESTATE);
//     }
//
//     return ret;
// }
//
// BOOL FASTCALL
// IntGdiPolyline(DC      *dc,
//                LPPOINT pt,
//                int     Count)
// {
//     SURFACE *psurf;
//     BRUSH *pbrLine;
//     LPPOINT Points;
//     BOOL Ret = TRUE;
//     LONG i;
//     PDC_ATTR pdcattr = dc->pdcattr;
//     PPATH pPath;
//
//     if (!dc->dclevel.pSurface)
//     {
//         return TRUE;
//     }
//
//     DC_vPrepareDCsForBlit(dc, NULL, NULL, NULL);
//     psurf = dc->dclevel.pSurface;
//
//     /* Get BRUSHOBJ from current pen. */
//     pbrLine = dc->dclevel.pbrLine;
//     ASSERT(pbrLine);
//
//     if (!(pbrLine->flAttrs & BR_IS_NULL))
//     {
//         Points = EngAllocMem(0, Count * sizeof(POINT), GDITAG_TEMP);
//         if (Points != NULL)
//         {
//             RtlCopyMemory(Points, pt, Count * sizeof(POINT));
//             IntLPtoDP(dc, Points, Count);
//
//             /* Offset the array of points by the DC origin */
//             for (i = 0; i < Count; i++)
//             {
//                 Points[i].x += dc->ptlDCOrig.x;
//                 Points[i].y += dc->ptlDCOrig.y;
//             }
//
//             if (dc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//             {
//                AddPenLinesBounds(dc, Count, Points);
//             }
//
//             if (IntIsEffectiveWidePen(pbrLine))
//             {
//                 /* Clear the path */
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//
//                 /* Begin a path */
//                 pPath = PATH_CreatePath(Count);
//                 dc->dclevel.flPath |= DCPATH_ACTIVE;
//                 dc->dclevel.hPath = pPath->BaseObject.hHmgr;
//                 pPath->pos = pt[0];
//                 IntLPtoDP(dc, &pPath->pos, 1);
//
//                 PATH_MoveTo(dc, pPath);
//                 for (i = 1; i < Count; ++i)
//                 {
//                     PATH_LineTo(dc, pt[i].x, pt[i].y);
//                 }
//
//                 /* Close the path */
//                 pPath->state = PATH_Closed;
//                 dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//                 /* Actually stroke a path */
//                 Ret = PATH_StrokePath(dc, pPath);
//
//                 /* Clear the path */
//                 PATH_UnlockPath(pPath as POBJ);
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//             }
//             else
//             {
//                 Ret = IntEngPolyline(&psurf->SurfObj,
//                                      (CLIPOBJ *)&dc->co,
//                                      &dc->eboLine.BrushObject,
//                                      Points,
//                                      Count,
//                                      ROP2_TO_MIX(pdcattr->jROP2));
//             }
//             EngFreeMem(Points);
//         }
//         else
//         {
//             Ret = FALSE;
//         }
//     }
//
//     DC_vFinishBlit(dc, NULL);
//
//     return Ret;
// }
//
// BOOL FASTCALL
// IntGdiPolylineTo(DC      *dc,
//                  LPPOINT pt,
//                  DWORD   Count)
// {
//     BOOL ret = FALSE; // Default to failure
//     PDC_ATTR pdcattr = dc->pdcattr;
//
//     if (PATH_IsPathOpen(dc->dclevel))
//     {
//         ret = PATH_PolylineTo(dc, pt as *const POINTL, Count);
//     }
//     else /* Do it using Polyline */
//     {
//         POINT *pts = ExAllocatePoolWithTag(PagedPool,
//                                            sizeof(POINT) * (Count + 1),
//                                            TAG_SHAPE);
//         if ( pts )
//         {
//             pts[0].x = pdcattr->ptlCurrent.x;
//             pts[0].y = pdcattr->ptlCurrent.y;
//             memcpy( pts + 1, pt, sizeof(POINT) * Count);
//             ret = IntGdiPolyline(dc, pts, Count + 1);
//             ExFreePoolWithTag(pts, TAG_SHAPE);
//         }
//     }
//     if ( ret )
//     {
//         pdcattr->ptlCurrent.x = pt[Count-1].x;
//         pdcattr->ptlCurrent.y = pt[Count-1].y;
//         pdcattr->ptfxCurrent = pdcattr->ptlCurrent;
//         CoordLPtoDP(dc, &pdcattr->ptfxCurrent); // Update fx
//         pdcattr->ulDirty_ &= ~(DIRTY_PTLCURRENT|DIRTY_PTFXCURRENT|DIRTY_STYLESTATE);
//     }
//
//     return ret;
// }
//
//
// BOOL FASTCALL
// IntGdiPolyPolyline(DC      *dc,
//                    LPPOINT pt,
//                    PULONG  PolyPoints,
//                    DWORD   Count)
// {
//     ULONG i;
//     LPPOINT pts;
//     PULONG pc;
//     BOOL ret = FALSE; // Default to failure
//     pts = pt;
//     pc = PolyPoints;
//
//     if (PATH_IsPathOpen(dc->dclevel))
//     {
//         return PATH_PolyPolyline( dc, pt, PolyPoints, Count );
//     }
//     for (i = 0; i < Count; i++)
//     {
//         ret = IntGdiPolyline ( dc, pts, *pc );
//         if (ret == FALSE)
//         {
//             return ret;
//         }
//         pts+=*pc++;
//     }
//
//     return ret;
// }
//
// /******************************************************************************/
//
// BOOL
// APIENTRY
// NtGdiLineTo(HDC  hDC,
//             int  XEnd,
//             int  YEnd)
// {
//     DC *dc;
//     BOOL Ret;
//     RECT rcLockRect ;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     rcLockRect.left = dc->pdcattr->ptlCurrent.x;
//     rcLockRect.top = dc->pdcattr->ptlCurrent.y;
//     rcLockRect.right = XEnd;
//     rcLockRect.bottom = YEnd;
//
//     IntLPtoDP(dc, (PPOINT)&rcLockRect, 2);
//
//     /* The DCOrg is in device coordinates */
//     rcLockRect.left += dc->ptlDCOrig.x;
//     rcLockRect.top += dc->ptlDCOrig.y;
//     rcLockRect.right += dc->ptlDCOrig.x;
//     rcLockRect.bottom += dc->ptlDCOrig.y;
//
//     DC_vPrepareDCsForBlit(dc, &rcLockRect, NULL, NULL);
//
//     Ret = IntGdiLineTo(dc, XEnd, YEnd);
//
//     DC_vFinishBlit(dc, NULL);
//
//     DC_UnlockDc(dc);
//     return Ret;
// }
//
// // FIXME: This function is completely broken
// static
// BOOL
// GdiPolyDraw(
//     IN HDC hdc,
//     IN LPPOINT lppt,
//     IN LPBYTE lpbTypes,
//     IN ULONG cCount)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//     POINT bzr[4];
//     volatile PPOINT line_pts, line_pts_old, bzr_pts;
//     INT num_pts, num_bzr_pts, space, space_old, size;
//     ULONG i;
//     BOOL result = FALSE;
//
//     dc = DC_LockDc(hdc);
//     if (!dc) return FALSE;
//     pdcattr = dc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//        DC_vUpdateFillBrush(dc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//        DC_vUpdateLineBrush(dc);
//
//     if (!cCount)
//     {
//        DC_UnlockDc(dc);
//        return TRUE;
//     }
//
//     line_pts = NULL;
//     line_pts_old = NULL;
//     bzr_pts = NULL;
//
//     {
//         if (PATH_IsPathOpen(dc->dclevel))
//         {
//            result = PATH_PolyDraw(dc, (const POINT *)lppt, (const BYTE *)lpbTypes, cCount);
//            goto Cleanup;
//         }
//
//         /* Check for valid point types */
//         for (i = 0; i < cCount; i++)
//         {
//            switch (lpbTypes[i])
//            {
//            case PT_MOVETO:
//            case PT_LINETO | PT_CLOSEFIGURE:
//            case PT_LINETO:
//                break;
//            case PT_BEZIERTO:
//                if((i + 2 < cCount) && (lpbTypes[i + 1] == PT_BEZIERTO) &&
//                   ((lpbTypes[i + 2] & ~PT_CLOSEFIGURE) == PT_BEZIERTO))
//                {
//                    i += 2;
//                    break;
//                }
//            default:
//                goto Cleanup;
//            }
//         }
//
//         space = cCount + 300;
//         line_pts = ExAllocatePoolWithTag(PagedPool, space * sizeof(POINT), TAG_SHAPE);
//         if (line_pts == NULL)
//         {
//             result = FALSE;
//             goto Cleanup;
//         }
//
//         num_pts = 1;
//
//         line_pts[0].x = pdcattr->ptlCurrent.x;
//         line_pts[0].y = pdcattr->ptlCurrent.y;
//
//         for ( i = 0; i < cCount; i++ )
//         {
//            switch (lpbTypes[i])
//            {
//            case PT_MOVETO:
//                if (num_pts >= 2) IntGdiPolyline( dc, line_pts, num_pts );
//                num_pts = 0;
//                line_pts[num_pts++] = lppt[i];
//                break;
//            case PT_LINETO:
//            case (PT_LINETO | PT_CLOSEFIGURE):
//                line_pts[num_pts++] = lppt[i];
//                break;
//            case PT_BEZIERTO:
//                bzr[0].x = line_pts[num_pts - 1].x;
//                bzr[0].y = line_pts[num_pts - 1].y;
//                RtlCopyMemory( &bzr[1], &lppt[i], 3 * sizeof(POINT) );
//
//                if ((bzr_pts = GDI_Bezier( bzr, 4, &num_bzr_pts )))
//                {
//                    size = num_pts + (cCount - i) + num_bzr_pts;
//                    if (space < size)
//                    {
//                       space_old = space;
//                       space = size * 2;
//                       line_pts_old = line_pts;
//                       line_pts = ExAllocatePoolWithTag(PagedPool, space * sizeof(POINT), TAG_SHAPE);
//                       if (!line_pts) goto Cleanup;
//                       RtlCopyMemory(line_pts, line_pts_old, space_old * sizeof(POINT));
//                       ExFreePoolWithTag(line_pts_old, TAG_SHAPE);
//                       line_pts_old = NULL;
//                    }
//                    RtlCopyMemory( &line_pts[num_pts], &bzr_pts[1], (num_bzr_pts - 1) * sizeof(POINT) );
//                    num_pts += num_bzr_pts - 1;
//                    ExFreePoolWithTag(bzr_pts, TAG_BEZIER);
//                    bzr_pts = NULL;
//                }
//                i += 2;
//                break;
//            }
//            if (lpbTypes[i] & PT_CLOSEFIGURE) line_pts[num_pts++] = line_pts[0];
//         }
//
//         if (num_pts >= 2) IntGdiPolyline( dc, line_pts, num_pts );
//         IntGdiMoveToEx( dc, line_pts[num_pts - 1].x, line_pts[num_pts - 1].y, NULL );
//         result = TRUE;
//     }
//
// Cleanup:
//
//     if (line_pts != NULL)
//     {
//         ExFreePoolWithTag(line_pts, TAG_SHAPE);
//     }
//
//     if ((line_pts_old != NULL) && (line_pts_old != line_pts))
//     {
//         ExFreePoolWithTag(line_pts_old, TAG_SHAPE);
//     }
//
//     if (bzr_pts != NULL)
//     {
//         ExFreePoolWithTag(bzr_pts, TAG_BEZIER);
//     }
//
//     DC_UnlockDc(dc);
//
//     return result;
// }
//
// __kernel_entry
// W32KAPI
// BOOL
// APIENTRY
// NtGdiPolyDraw(
//     _In_ HDC hdc,
//     _In_reads_(cpt) LPPOINT ppt,
//     _In_reads_(cpt) LPBYTE pjAttr,
//     _In_ ULONG cpt)
// {
//     PBYTE pjBuffer;
//     PPOINT pptSafe;
//     PBYTE pjSafe;
//     SIZE_T cjSizePt;
//     BOOL bResult;
//
//     if (cpt == 0)
//     {
//         ERR!("cpt is 0\n");
//         return FALSE;
//     }
//
//     /* Validate that cpt isn't too large */
//     if (cpt > ((MAXULONG - cpt) / sizeof(*ppt)))
//     {
//         ERR!("cpt is too large\n", cpt);
//         return FALSE;
//     }
//
//     /* Calculate size for a buffer */
//     cjSizePt = cpt * sizeof(*ppt);
//     ASSERT(cjSizePt + cpt > cjSizePt);
//
//     /* Allocate a buffer for all data */
//     pjBuffer = ExAllocatePoolWithTag(PagedPool, cjSizePt + cpt, TAG_SHAPE);
//     if (pjBuffer == NULL)
//     {
//         ERR!("Failed to allocate buffer\n");
//         return FALSE;
//     }
//
//     pptSafe = (PPOINT)pjBuffer;
//     pjSafe = pjBuffer + cjSizePt;
//
//     _SEH2_TRY
//     {
//         ProbeArrayForRead(ppt, sizeof(*ppt), cpt, sizeof(ULONG));
//         ProbeArrayForRead(pjAttr, sizeof(*pjAttr), cpt, sizeof(BYTE));
//
//         /* Copy the arrays */
//         RtlCopyMemory(pptSafe, ppt, cjSizePt);
//         RtlCopyMemory(pjSafe, pjAttr, cpt);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         SetLastNtError(_SEH2_GetExceptionCode());
//         bResult = FALSE;
//         goto Cleanup;
//     }
//     _SEH2_END;
//
//     /* Call the internal function */
//     bResult = GdiPolyDraw(hdc, pptSafe, pjSafe, cpt);
//
// Cleanup:
//
//     /* Free the buffer */
//     ExFreePoolWithTag(pjBuffer, TAG_SHAPE);
//
//     return bResult;
// }
//
// /*
//  * @implemented
//  */
// _Success_(return != FALSE)
// BOOL
// APIENTRY
// NtGdiMoveTo(
//     IN HDC hdc,
//     IN INT x,
//     IN INT y,
//     OUT OPTIONAL LPPOINT pptOut)
// {
//     PDC pdc;
//     BOOL Ret;
//     POINT Point;
//     NTSTATUS Status;
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc) return FALSE;
//
//     Ret = IntGdiMoveToEx(pdc, x, y, &Point);
//
//     if (Ret && pptOut)
//     {
//         Status = SafeProbeForWrite(pptOut, sizeof(POINT), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             Ret = FALSE; // CHECKME: is this correct?
//         }
//         else
//         {
//             RtlCopyMemory(pptOut, &Point, sizeof(POINT));
//         }
//     }
//
//     DC_UnlockDc(pdc);
//
//     return Ret;
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
