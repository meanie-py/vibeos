use win32k_sys::win32k::*;
use core::ffi::c_void;
use crate::brushbody::pbrushbody;
use crate::floatobj::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::STATUS_SUCCESS;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;


// CRT (cdecl); DIB_XXBPP_FloodFillSolid is plain cdecl (dib*.h, BOOLEAN).
extern "C" {
    fn abs(x: i32) -> i32;
    fn DIB_XXBPP_FloodFillSolid(surf: *mut c_void, brush: *mut c_void, rect: *mut RECTL, pt: *mut POINTL, color: u32, filltype: u32) -> u8;
}
// EXLATEOBJ_* / SafeProbeForRead: NTAPI (extern "system"). gpalRGB: real global.
extern "system" {
    fn EXLATEOBJ_vInitialize(exlo: *mut EXLATEOBJ, pal1: *mut c_void, pal2: *mut c_void, clr1: u32, clr2: u32, clr3: u32);
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
    fn SafeProbeForRead(address: *const c_void, length: usize, alignment: u32) -> i32;
    static gpalRGB: c_void;
}
// SetLastNtError: VOID FASTCALL (misc.h). PEN_ShareLockPen: FASTCALL (pen.h).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH;
}

// NT_SUCCESS(s) = s >= 0
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool { status >= 0 }
// ROP2_TO_MIX(Rop2) = ((Rop2)<<8)|(Rop2)  [inteng.h]
#[inline(always)]
fn ROP2_TO_MIX(rop2: u32) -> u32 { (rop2 << 8) | rop2 }
// ROP4_FROM_INDEX(i) = (i) | ((i) << 8)  [inteng.h]
#[inline(always)]
fn ROP4_FROM_INDEX(index: u32) -> u32 { index | (index << 8) }
// BRUSH_ShareLockBrush / BRUSH_ShareUnlockBrush (brush.h macros).
#[inline(always)]
unsafe fn BRUSH_ShareLockBrush(hbrush: HBRUSH) -> PBRUSH {
    GDIOBJ_ShareLockObj(hbrush as HGDIOBJ, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH
}
#[inline(always)]
unsafe fn BRUSH_ShareUnlockBrush(pbrush: PBRUSH) { GDIOBJ_vDereferenceObject(pbrush as POBJ); }
// PEN_ShareUnlockPen / PATH_UnlockPath (pen.h / path.h macros).
#[inline(always)]
unsafe fn PEN_ShareUnlockPen(ppen: PBRUSH) { GDIOBJ_vDereferenceObject(ppen as POBJ); }
#[inline(always)]
unsafe fn PATH_UnlockPath(ppath: POBJ) { GDIOBJ_vDereferenceObject(ppath); }
// PATH_IsPathOpen (path.h macro) — inline over _DCLEVEL.
#[inline(always)]
unsafe fn PATH_IsPathOpen(dclevel: *mut c_void) -> i32 {
    let d = dclevel as *mut _DCLEVEL;
    (((!(*d).hPath.is_null()) && ((*d).flPath & DCPATH_ACTIVE) != 0)) as i32
}
// IntIsEffectiveWidePen (pen.h macro) — inline via BRUSHBODY view.
#[inline(always)]
unsafe fn IntIsEffectiveWidePen(pbr: PBRUSH) -> BOOL {
    let b = pbrushbody(pbr);
    (((*b).lWidth > 1)
        && (((*b).flAttrs & BR_IS_OLDSTYLEPEN) != 0
            || ((*b).ulPenStyle & PS_TYPE_MASK) == PS_GEOMETRIC)) as BOOL
}
// RECTL_vSetRect / RECTL_bIsEmptyRect (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_vSetRect(rc: *mut RECTL, left: i32, top: i32, right: i32, bottom: i32) {
    (*rc).left = left; (*rc).top = top; (*rc).right = right; (*rc).bottom = bottom;
}
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(rc: *const RECTL) -> BOOL {
    (((*rc).left >= (*rc).right) || ((*rc).top >= (*rc).bottom)) as BOOL
}
// RtlCopyMemory intrinsic.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}

// coord.h transform chain (static inline / macros — no symbols).
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
unsafe fn IntLPtoDP(pdc: PDC, points: PPOINT, count: i32) -> BOOL {
    DC_vUpdateWorldToDevice(pdc);
    INTERNAL_APPLY_MATRIX(core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX, points as LPPOINT, count as u32) as BOOL
}

// ---- file-scope preamble (translated) ----
pub const TAG_SHAPE: u32 = 0x70616853;

pub const M_PI: f64 = 3.14159265358979323846;

#[inline]
fn min(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}

#[inline]
fn max(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiPolygon(
    dc: PDC,
    Points: PPOINT,
    Count: i32,
) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut pbrLine: PBRUSH;
    let mut pbrFill: PBRUSH;
    let mut ret: BOOL = 0;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut i: i32;
    let mut CurrentPoint: i32;
    let pdcattr: PDC_ATTR;
    let mut BrushOrigin: POINTL;
    let mut pPath: PPATH;

    if Points.is_null() || Count < 2 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdcattr = (*dc).pdcattr;

    IntLPtoDP(dc, Points, Count);
    CurrentPoint = 0;
    while CurrentPoint < Count {
        (*Points.offset(CurrentPoint as isize)).x += (*dc).ptlDCOrig.x;
        (*Points.offset(CurrentPoint as isize)).y += (*dc).ptlDCOrig.y;
        CurrentPoint += 1;
    }

    DestRect.left = (*Points.offset(0)).x;
    DestRect.right = (*Points.offset(0)).x;
    DestRect.top = (*Points.offset(0)).y;
    DestRect.bottom = (*Points.offset(0)).y;

    CurrentPoint = 1;
    while CurrentPoint < Count {
        DestRect.left = min(DestRect.left, (*Points.offset(CurrentPoint as isize)).x);
        DestRect.right = max(DestRect.right, (*Points.offset(CurrentPoint as isize)).x);
        DestRect.top = min(DestRect.top, (*Points.offset(CurrentPoint as isize)).y);
        DestRect.bottom = max(DestRect.bottom, (*Points.offset(CurrentPoint as isize)).y);
        CurrentPoint += 1;
    }

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    pbrFill = (*dc).dclevel.pbrFill;
    pbrLine = (*dc).dclevel.pbrLine;
    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        return 1;
    }

    if (*pbrushbody(pbrFill)).flAttrs & BR_IS_NULL == 0 {
        BrushOrigin = *(core::ptr::addr_of!((*pbrushbody(pbrFill)).ptOrigin) as PPOINTL);
        BrushOrigin.x += (*dc).ptlDCOrig.x;
        BrushOrigin.y += (*dc).ptlDCOrig.y;
        ret = IntFillPolygon(
            dc,
            psurf,
            core::ptr::addr_of_mut!((*dc).eboFill.BrushObject),
            Points as *mut POINTL,
            Count,
            DestRect,
            core::ptr::addr_of_mut!(BrushOrigin),
        );
    }

    if (*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL == 0 {
        if IntIsEffectiveWidePen(pbrLine) != 0 {
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();

            pPath = PATH_CreatePath(Count + 1);
            (*dc).dclevel.flPath |= DCPATH_ACTIVE;
            (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
            (*pPath).pos = core::mem::transmute(*Points.offset(0));
            IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as PPOINT, 1);

            PATH_MoveTo(dc, pPath);
            i = 1;
            while i < Count {
                PATH_LineTo(dc, (*Points.offset(i as isize)).x, (*Points.offset(i as isize)).y);
                i += 1;
            }
            PATH_LineTo(dc, (*Points.offset(0)).x, (*Points.offset(0)).y);

            (*pPath).state = PATH_Closed;
            (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

            ret = PATH_StrokePath(dc, pPath);

            PATH_UnlockPath(pPath as POBJ);
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();
        } else {
            i = 0;
            while i < Count - 1 {
                ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    (*Points.offset(i as isize)).x,
                    (*Points.offset(i as isize)).y,
                    (*Points.offset((i + 1) as isize)).x,
                    (*Points.offset((i + 1) as isize)).y,
                    core::ptr::addr_of_mut!(DestRect),
                    ROP2_TO_MIX((*pdcattr).jROP2 as u32),
                );
                if ret == 0 {
                    break;
                }
                i += 1;
            }
            if ret != 0 {
                ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    (*Points.offset((Count - 1) as isize)).x,
                    (*Points.offset((Count - 1) as isize)).y,
                    (*Points.offset(0)).x,
                    (*Points.offset(0)).y,
                    core::ptr::addr_of_mut!(DestRect),
                    ROP2_TO_MIX((*pdcattr).jROP2 as u32),
                );
            }
        }
    }

    ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiPolygon(
    dc: PDC,
    Points: PPOINT,
    Count: i32,
) -> BOOL {
    let mut psurf: *mut SURFACE;
    let mut pbrLine: PBRUSH;
    let mut pbrFill: PBRUSH;
    let mut ret: BOOL = 0;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut i: i32;
    let mut CurrentPoint: i32;
    let pdcattr: PDC_ATTR;
    let mut BrushOrigin: POINTL;
    let mut pPath: PPATH;

    if Points.is_null() || Count < 2 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    pdcattr = (*dc).pdcattr;

    IntLPtoDP(dc, Points, Count);
    CurrentPoint = 0;
    while CurrentPoint < Count {
        (*Points.offset(CurrentPoint as isize)).x += (*dc).ptlDCOrig.x;
        (*Points.offset(CurrentPoint as isize)).y += (*dc).ptlDCOrig.y;
        CurrentPoint += 1;
    }

    DestRect.left = (*Points.offset(0)).x;
    DestRect.right = (*Points.offset(0)).x;
    DestRect.top = (*Points.offset(0)).y;
    DestRect.bottom = (*Points.offset(0)).y;

    CurrentPoint = 1;
    while CurrentPoint < Count {
        DestRect.left = min(DestRect.left, (*Points.offset(CurrentPoint as isize)).x);
        DestRect.right = max(DestRect.right, (*Points.offset(CurrentPoint as isize)).x);
        DestRect.top = min(DestRect.top, (*Points.offset(CurrentPoint as isize)).y);
        DestRect.bottom = max(DestRect.bottom, (*Points.offset(CurrentPoint as isize)).y);
        CurrentPoint += 1;
    }

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    pbrFill = (*dc).dclevel.pbrFill;
    pbrLine = (*dc).dclevel.pbrLine;
    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        return 1;
    }

    if (*pbrushbody(pbrFill)).flAttrs & BR_IS_NULL == 0 {
        BrushOrigin = *(core::ptr::addr_of!((*pbrushbody(pbrFill)).ptOrigin) as PPOINTL);
        BrushOrigin.x += (*dc).ptlDCOrig.x;
        BrushOrigin.y += (*dc).ptlDCOrig.y;
        ret = IntFillPolygon(
            dc,
            psurf,
            core::ptr::addr_of_mut!((*dc).eboFill.BrushObject),
            Points as *mut POINTL,
            Count,
            DestRect,
            core::ptr::addr_of_mut!(BrushOrigin),
        );
    }

    if (*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL == 0 {
        if IntIsEffectiveWidePen(pbrLine) != 0 {
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();

            pPath = PATH_CreatePath(Count + 1);
            (*dc).dclevel.flPath |= DCPATH_ACTIVE;
            (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
            (*pPath).pos = core::mem::transmute(*Points.offset(0));
            IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as PPOINT, 1);

            PATH_MoveTo(dc, pPath);
            i = 1;
            while i < Count {
                PATH_LineTo(dc, (*Points.offset(i as isize)).x, (*Points.offset(i as isize)).y);
                i += 1;
            }
            PATH_LineTo(dc, (*Points.offset(0)).x, (*Points.offset(0)).y);

            (*pPath).state = PATH_Closed;
            (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

            ret = PATH_StrokePath(dc, pPath);

            PATH_UnlockPath(pPath as POBJ);
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();
        } else {
            i = 0;
            while i < Count - 1 {
                ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    (*Points.offset(i as isize)).x,
                    (*Points.offset(i as isize)).y,
                    (*Points.offset((i + 1) as isize)).x,
                    (*Points.offset((i + 1) as isize)).y,
                    core::ptr::addr_of_mut!(DestRect),
                    ROP2_TO_MIX((*pdcattr).jROP2 as u32),
                );
                if ret == 0 {
                    break;
                }
                i += 1;
            }
            if ret != 0 {
                ret = IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
                    (*Points.offset((Count - 1) as isize)).x,
                    (*Points.offset((Count - 1) as isize)).y,
                    (*Points.offset(0)).x,
                    (*Points.offset(0)).y,
                    core::ptr::addr_of_mut!(DestRect),
                    ROP2_TO_MIX((*pdcattr).jROP2 as u32),
                );
            }
        }
    }

    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiPolyPolygon(
    dc: *mut DC,
    Points: LPPOINT,
    PolyCounts: *mut u32,
    Count: i32,
) -> i32 {
    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_PolyPolygon(dc, Points as *const POINTL, PolyCounts as PINT, Count as u32);
    }

    let mut count = Count;
    let mut points = Points;
    let mut poly_counts = PolyCounts;

    loop {
        count -= 1;
        if count < 0 {
            break;
        }

        if IntGdiPolygon(dc, points, *poly_counts as i32) == 0 {
            return 0;
        }

        points = points.offset(*poly_counts as isize);
        poly_counts = poly_counts.offset(1);
    }

    1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiPolyPolygon(
    dc: *mut DC,
    Points: LPPOINT,
    PolyCounts: *mut u32,
    Count: i32,
) -> i32 {
    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_PolyPolygon(dc, Points as *const POINTL, PolyCounts as PINT, Count as u32);
    }

    let mut count = Count;
    let mut points = Points;
    let mut poly_counts = PolyCounts;

    loop {
        count -= 1;
        if count < 0 {
            break;
        }

        if IntGdiPolygon(dc, points, *poly_counts as i32) == 0 {
            return 0;
        }

        points = points.offset(*poly_counts as isize);
        poly_counts = poly_counts.offset(1);
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntPolygon(hdc: HDC, Point: *mut POINT, Count: i32) -> i32 {
    let mut bResult: i32;
    let pdc: PDC;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    bResult = IntGdiPolygon(pdc, Point, Count);

    DC_UnlockDc(pdc);
    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntPolygon(hdc: HDC, Point: *mut POINT, Count: i32) -> i32 {
    let mut bResult: i32;
    let pdc: PDC;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    bResult = IntGdiPolygon(pdc, Point, Count);

    DC_UnlockDc(pdc);
    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEllipse(
    hDC: HDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
) -> i32 {
    let mut dc: PDC;
    let mut pdcattr: PDC_ATTR;
    let mut RectBounds: RECTL = core::mem::zeroed();
    let mut pbrush: PBRUSH;
    let mut ret: i32 = 1;
    let mut PenWidth: i32;
    let mut PenOrigWidth: i32;
    let mut RadiusX: i32;
    let mut RadiusY: i32;
    let mut CenterX: i32;
    let mut CenterY: i32;
    let mut pFillBrushObj: PBRUSH;
    let mut tmpFillBrushObj: BRUSH = core::mem::zeroed();
    let mut Left = Left;
    let mut Top = Top;
    let mut Right = Right;
    let mut Bottom = Bottom;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        ret = PATH_Ellipse(dc, Left, Top, Right, Bottom);
        DC_UnlockDc(dc);
        return ret;
    }

    if (Left == Right) || (Top == Bottom) {
        DC_UnlockDc(dc);
        return 1;
    }

    if Right < Left {
        let tmp = Right;
        Right = Left;
        Left = tmp;
    }
    if Bottom < Top {
        let tmp = Bottom;
        Bottom = Top;
        Top = tmp;
    }

    pdcattr = (*dc).pdcattr;

    if (*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    pbrush = PEN_ShareLockPen((*pdcattr).hpen as HPEN);
    if pbrush.is_null() {
        DC_UnlockDc(dc);
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    PenOrigWidth = (*pbrushbody(pbrush)).lWidth;
    PenWidth = PenOrigWidth;
    if (*pbrushbody(pbrush)).ulPenStyle == PS_NULL {
        PenWidth = 0;
    }

    if (*pbrushbody(pbrush)).ulPenStyle == PS_INSIDEFRAME {
        if 2 * PenWidth > (Right - Left) {
            PenWidth = (Right - Left + 1) / 2;
        }
        if 2 * PenWidth > (Bottom - Top) {
            PenWidth = (Bottom - Top + 1) / 2;
        }
        Left += PenWidth / 2;
        Right -= (PenWidth - 1) / 2;
        Top += PenWidth / 2;
        Bottom -= (PenWidth - 1) / 2;
    }

    if PenWidth == 0 {
        PenWidth = 1;
    }
    (*pbrushbody(pbrush)).lWidth = PenWidth;

    RectBounds.left = Left;
    RectBounds.right = Right;
    RectBounds.top = Top;
    RectBounds.bottom = Bottom;

    IntLPtoDP(dc, core::ptr::addr_of_mut!(RectBounds) as LPPOINT, 2);

    RectBounds.left += (*dc).ptlDCOrig.x;
    RectBounds.right += (*dc).ptlDCOrig.x;
    RectBounds.top += (*dc).ptlDCOrig.y;
    RectBounds.bottom += (*dc).ptlDCOrig.y;

    RadiusX = max((RectBounds.right - RectBounds.left) / 2, 2);
    RadiusY = max((RectBounds.bottom - RectBounds.top) / 2, 2);
    CenterX = (RectBounds.right + RectBounds.left) / 2;
    CenterY = (RectBounds.bottom + RectBounds.top) / 2;

    pFillBrushObj = BRUSH_ShareLockBrush((*pdcattr).hbrush as HBRUSH);
    if pFillBrushObj.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        ret = 0;
    } else {
        RtlCopyMemory(
            core::ptr::addr_of_mut!(tmpFillBrushObj) as *mut core::ffi::c_void,
            pFillBrushObj as *mut core::ffi::c_void,
            core::mem::size_of::<BRUSH>(),
        );
        (*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.x += (*dc).ptlDCOrig.x;
        (*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.y += (*dc).ptlDCOrig.y;

        DC_vPrepareDCsForBlit(dc, core::ptr::addr_of_mut!(RectBounds), core::ptr::null_mut(), core::ptr::null_mut());

        ret = IntFillEllipse(
            dc,
            CenterX - RadiusX,
            CenterY - RadiusY,
            RadiusX * 2,
            RadiusY * 2,
            core::ptr::addr_of_mut!(tmpFillBrushObj),
        );
        BRUSH_ShareUnlockBrush(pFillBrushObj);

        if ret != 0 {
            ret = IntDrawEllipse(
                dc,
                CenterX - RadiusX,
                CenterY - RadiusY,
                RadiusX * 2,
                RadiusY * 2,
                pbrush,
            );
        }

        DC_vFinishBlit(dc, core::ptr::null_mut());
    }

    (*pbrushbody(pbrush)).lWidth = PenOrigWidth;
    PEN_ShareUnlockPen(pbrush);
    DC_UnlockDc(dc);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiPolyPolyDraw(
    hDC: HDC,
    UnsafePoints: PPOINT,
    UnsafeCounts: *mut u32,
    Count: u32,
    iFunc: i32,
) -> usize {
    let mut dc: *mut DC;
    let mut pTemp: *mut core::ffi::c_void;
    let mut SafePoints: LPPOINT;
    let mut SafeCounts: *mut u32;
    let mut Status: i32 = STATUS_SUCCESS;
    let mut Ret: i32 = 1;
    let mut nPoints: u32 = 0;
    let mut nMaxPoints: u32 = 0;
    let mut nInvalid: u32 = 0;
    let mut i: u32;

    if UnsafePoints.is_null() || UnsafeCounts.is_null() ||
        Count == 0 || iFunc == 0 || iFunc > GdiPolyPolyRgn as i32
    {
        return 0;
    }

    {
        if Count > (isize::MAX as u32) / core::mem::size_of::<POINT>() as u32 {
            Status = -1073741670;
        } else if Count > (isize::MAX as u32) / core::mem::size_of::<u32>() as u32 {
            Status = -1073741670;
        } else {
            // The C probes both arrays here — ProbeForRead(UnsafePoints, Count *
            // sizeof(POINT), 1) and ProbeForRead(UnsafeCounts, Count * sizeof(ULONG), 1)
            // — inside a _SEH2_TRY that stashes the exception code in Status. Both were
            // dropped, so the loop below read UnsafeCounts[] straight off the caller's
            // pointer. Note what survived and what did not: the overflow guards above
            // were kept, the probes were not, and an overflow guard bounds the SIZE of
            // an access, never its ADDRESS.
            Status = SafeProbeForRead(
                UnsafePoints as *const c_void,
                Count as usize * core::mem::size_of::<POINT>(),
                1,
            );
            if NT_SUCCESS(Status) {
                Status = SafeProbeForRead(
                    UnsafeCounts as *const c_void,
                    Count as usize * core::mem::size_of::<u32>(),
                    1,
                );
            }
            i = 0;
            while NT_SUCCESS(Status) && i < Count {
                // Probe is validate-only; the C had probe AND these loads in one
                // _SEH2_TRY. Count is caller-controlled, so capture per element
                // rather than allocating a kernel mirror of the whole array.
                let mut count_val: u32 = 0;
                if !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(count_val) as *mut core::ffi::c_void,
                    UnsafeCounts.add(i as usize) as *const core::ffi::c_void,
                    core::mem::size_of::<u32>(),
                ) {
                    Status = STATUS_ACCESS_VIOLATION;
                    break;
                }
                if count_val < 2 {
                    nInvalid += 1;
                }
                nPoints = nPoints.wrapping_add(count_val);
                if count_val > nMaxPoints {
                    nMaxPoints = count_val;
                }
                i += 1;
            }
        }
    }

    if !NT_SUCCESS(Status) {
        return 0;
    }

    if nPoints == 0 || nPoints < nMaxPoints {
        return 0;
    }

    if nInvalid != 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    // nPoints is the SUM of the caller's count array, so unlike Count (guarded above)
    // nothing has bounded it — it can reach 0xFFFFFFFF. Both the allocation below and
    // the copy that fills it size themselves on `nPoints * sizeof(POINT)`, which on a
    // 32-bit usize wraps SILENTLY in release: a wrapped size allocates a small buffer,
    // the range check passes on the same small length, and the copy then writes the
    // real (huge) count into it. The C has the identical shape in size_t and is wrong
    // the same way on i386. Settle the arithmetic once, here, where both users of it
    // are still downstream.
    let Some(points_bytes) = (nPoints as usize).checked_mul(core::mem::size_of::<POINT>())
    else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    };
    let Some(temp_bytes) =
        (Count as usize).checked_mul(core::mem::size_of::<u32>()).and_then(|c| c.checked_add(points_bytes))
    else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    };

    pTemp = ExAllocatePoolWithTag(
        PagedPool,
        temp_bytes as SIZE_T,
        0x70616853u32,
    );
    if pTemp.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    SafeCounts = pTemp as *mut u32;
    SafePoints = SafeCounts.add(Count as usize) as *mut POINT;

    // The C's SECOND _SEH2_TRY covers these two copies, and it too was dropped, which
    // left the `if (!NT_SUCCESS(Status))` below it checking a status nothing could set.
    // This copy is the dangerous one: its length is nPoints, the SUM of the caller's
    // count array, so it is not bounded by the Count * sizeof(POINT) probed above and
    // can be made arbitrarily large. Probe the length actually about to be copied.
    //
    // What this does NOT restore, exactly as in NtSetValueKey (#79): the C's handler
    // also covered the window between the check and the copy. A range check proves
    // where the buffer is, never that it stays mapped.
    Status = SafeProbeForRead(UnsafePoints as *const c_void, points_bytes, 1);

    // The probes VALIDATE only -- which is exactly what the comment above says a range
    // check cannot do about staying mapped. These two copies ARE the access, so they go
    // through the protected path and a fault lands in Status, checked just below.
    // `points_bytes` and the counts length are the same byte counts the probes used.
    if NT_SUCCESS(Status)
        && (!crate::copy::capture_from_user(
            SafeCounts as *mut c_void,
            UnsafeCounts as *const c_void,
            (Count as usize) * core::mem::size_of::<u32>(),
        ) || !crate::copy::capture_from_user(
            SafePoints as *mut c_void,
            UnsafePoints as *const c_void,
            points_bytes,
        ))
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(pTemp, 0x70616853u32);
        return 0;
    }

    if iFunc == GdiPolyPolyRgn as i32 {
        let iMode: i32 = hDC as usize as i32;
        let hrgn: usize;

        hrgn = GreCreatePolyPolygonRgn(SafePoints as *const POINTL, SafeCounts, Count, iMode) as usize;

        ExFreePoolWithTag(pTemp, 0x70616853u32);
        return hrgn;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(0x6);
        ExFreePoolWithTag(pTemp, 0x70616853u32);
        return 0;
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    if (*(*dc).pdcattr).ulDirty_ & (0x00000002 | 0x00010000) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*(*dc).pdcattr).ulDirty_ & (0x00000004 | 0x00020000) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    match iFunc {
        x if x == 1 => {
            Ret = IntGdiPolyPolygon(dc, SafePoints, SafeCounts, Count as i32);
        }
        x if x == 2 => {
            Ret = IntGdiPolyPolyline(dc, SafePoints as *mut POINTL, SafeCounts, Count);
        }
        x if x == 3 => {
            Ret = IntGdiPolyBezier(dc, SafePoints as *mut POINTL, *SafeCounts);
        }
        x if x == 4 => {
            Ret = IntGdiPolylineTo(dc, SafePoints as *mut POINTL, *SafeCounts);
        }
        x if x == 5 => {
            Ret = IntGdiPolyBezierTo(dc, SafePoints as *mut POINTL, *SafeCounts);
        }
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
        }
    }

    DC_vFinishBlit(dc, core::ptr::null_mut());
    DC_UnlockDc(dc);
    ExFreePoolWithTag(pTemp, 0x70616853u32);
    return Ret as usize;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntRectangle(
    dc: PDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
) -> i32 {
    IntRectangle_impl(dc, LeftRect, TopRect, RightRect, BottomRect)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntRectangle(
    dc: PDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
) -> i32 {
    IntRectangle_impl(dc, LeftRect, TopRect, RightRect, BottomRect)
}

unsafe fn IntRectangle_impl(
    dc: PDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
) -> i32 {
    let mut psurf: *mut SURFACE = core::ptr::null_mut();
    let mut pbrLine: PBRUSH;
    let mut pbrFill: PBRUSH;
    let mut ret: i32 = 0;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut Mix: MIX;
    let pdcattr: PDC_ATTR;
    let mut BrushOrigin: POINTL = core::mem::zeroed();
    let mut pPath: PPATH;

    pdcattr = (*dc).pdcattr;

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_Rectangle(dc, LeftRect, TopRect, RightRect, BottomRect);
    }

    DestRect.left = min(LeftRect, RightRect);
    DestRect.right = max(LeftRect, RightRect);
    DestRect.top = min(TopRect, BottomRect);
    DestRect.bottom = max(TopRect, BottomRect);

    IntLPtoDP(dc, &mut DestRect as *mut RECTL as LPPOINT, 2);

    DestRect.left += (*dc).ptlDCOrig.x;
    DestRect.right += (*dc).ptlDCOrig.x;
    DestRect.top += (*dc).ptlDCOrig.y;
    DestRect.bottom += (*dc).ptlDCOrig.y;

    if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
        IntUpdateBoundsRect(dc, &mut DestRect);
    }

    if (*pdcattr).iGraphicsMode == GM_COMPATIBLE as i32 {
        DestRect.right -= 1;
        DestRect.bottom -= 1;
    }

    DC_vPrepareDCsForBlit(dc, &mut DestRect, core::ptr::null_mut(), core::ptr::null_mut());

    if ((*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY)) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if ((*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY)) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    pbrFill = (*dc).dclevel.pbrFill;
    pbrLine = (*dc).dclevel.pbrLine;
    if pbrLine.is_null() {
        ret = 0;
        DC_vFinishBlit(dc, core::ptr::null_mut());
        return ret;
    }

    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        ret = 1;
        DC_vFinishBlit(dc, core::ptr::null_mut());
        return ret;
    }

    if !pbrFill.is_null() {
        if ((*pbrushbody(pbrFill)).flAttrs & BR_IS_NULL) == 0 {
            BrushOrigin = *(core::ptr::addr_of!((*pbrushbody(pbrFill)).ptOrigin) as PPOINTL);
            BrushOrigin.x += (*dc).ptlDCOrig.x;
            BrushOrigin.y += (*dc).ptlDCOrig.y;
            ret = IntEngBitBlt(
                &mut (*psurf).SurfObj,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &(*dc).co as *const _ as *mut CLIPOBJ,
                core::ptr::null_mut(),
                &mut DestRect,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut (*dc).eboFill.BrushObject,
                &mut BrushOrigin,
                ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY),
            );
        }
    }

    ret = 1;

    if ((*pbrushbody(pbrLine)).flAttrs & BR_IS_NULL) == 0 {
        if IntIsEffectiveWidePen(pbrLine) != 0 {
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();

            pPath = PATH_CreatePath(5);
            (*dc).dclevel.flPath |= DCPATH_ACTIVE;
            (*dc).dclevel.hPath = (*pPath).BaseObject.hHmgr;
            (*pPath).pos.x = LeftRect;
            (*pPath).pos.y = TopRect;
            IntLPtoDP(dc, core::ptr::addr_of_mut!((*pPath).pos) as PPOINT, 1);

            PATH_MoveTo(dc, pPath);
            PATH_LineTo(dc, RightRect, TopRect);
            PATH_LineTo(dc, RightRect, BottomRect);
            PATH_LineTo(dc, LeftRect, BottomRect);
            PATH_LineTo(dc, LeftRect, TopRect);

            (*pPath).state = PATH_Closed;
            (*dc).dclevel.flPath &= !DCPATH_ACTIVE;

            ret = PATH_StrokePath(dc, pPath);

            PATH_UnlockPath(pPath as POBJ);
            PATH_Delete((*dc).dclevel.hPath);
            (*dc).dclevel.hPath = core::ptr::null_mut();
        } else {
            Mix = ROP2_TO_MIX((*pdcattr).jROP2 as u32);
            ret = ret & IntEngLineTo(
                &mut (*psurf).SurfObj,
                &(*dc).co as *const _ as *mut CLIPOBJ,
                &mut (*dc).eboLine.BrushObject,
                DestRect.left,
                DestRect.top,
                DestRect.right,
                DestRect.top,
                &mut DestRect,
                Mix,
            );

            ret = ret & IntEngLineTo(
                &mut (*psurf).SurfObj,
                &(*dc).co as *const _ as *mut CLIPOBJ,
                &mut (*dc).eboLine.BrushObject,
                DestRect.right,
                DestRect.top,
                DestRect.right,
                DestRect.bottom,
                &mut DestRect,
                Mix,
            );

            ret = ret & IntEngLineTo(
                &mut (*psurf).SurfObj,
                &(*dc).co as *const _ as *mut CLIPOBJ,
                &mut (*dc).eboLine.BrushObject,
                DestRect.right,
                DestRect.bottom,
                DestRect.left,
                DestRect.bottom,
                &mut DestRect,
                Mix,
            );

            ret = ret & IntEngLineTo(
                &mut (*psurf).SurfObj,
                &(*dc).co as *const _ as *mut CLIPOBJ,
                &mut (*dc).eboLine.BrushObject,
                DestRect.left,
                DestRect.bottom,
                DestRect.left,
                DestRect.top,
                &mut DestRect,
                Mix,
            );
        }
    }

    DC_vFinishBlit(dc, core::ptr::null_mut());

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRectangle(
    hDC: HDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
) -> i32 {
    let dc: *mut DC;
    let ret: i32;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if ((*(*dc).pdcattr).mxWorldToDevice.flAccel & XFORM_SCALE) == 0 {
        let mut DestCoords: [POINTL; 4] = core::mem::zeroed();
        let mut PolyCounts: u32 = 4;

        DestCoords[0].x = LeftRect;
        DestCoords[3].x = LeftRect;
        DestCoords[0].y = TopRect;
        DestCoords[1].y = TopRect;
        DestCoords[1].x = RightRect;
        DestCoords[2].x = RightRect;
        DestCoords[2].y = BottomRect;
        DestCoords[3].y = BottomRect;

        #[cfg(target_arch = "x86")]
        {
            ret = IntGdiPolyPolygon(dc, DestCoords.as_mut_ptr() as *mut POINT, core::ptr::addr_of_mut!(PolyCounts), 1);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            ret = IntGdiPolyPolygon(dc, DestCoords.as_mut_ptr() as *mut POINT, core::ptr::addr_of_mut!(PolyCounts), 1);
        }
    } else {
        #[cfg(target_arch = "x86")]
        {
            ret = IntRectangle(dc, LeftRect, TopRect, RightRect, BottomRect);
        }
        #[cfg(not(target_arch = "x86"))]
        {
            ret = IntRectangle(dc, LeftRect, TopRect, RightRect, BottomRect);
        }
    }

    DC_UnlockDc(dc);

    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    xCurveDiameter: i32,
    yCurveDiameter: i32,
) -> u8 {
    IntRoundRect_impl(dc, Left, Top, Right, Bottom, xCurveDiameter, yCurveDiameter)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    xCurveDiameter: i32,
    yCurveDiameter: i32,
) -> u8 {
    IntRoundRect_impl(dc, Left, Top, Right, Bottom, xCurveDiameter, yCurveDiameter)
}

unsafe fn IntRoundRect_impl(
    dc: PDC,
    mut Left: i32,
    mut Top: i32,
    mut Right: i32,
    mut Bottom: i32,
    xCurveDiameter: i32,
    yCurveDiameter: i32,
) -> u8 {

    let pdcattr: PDC_ATTR;
    let pbrLine: PBRUSH;
    let pbrFill: PBRUSH;
    let mut RectBounds: RECTL = core::mem::zeroed();
    let mut PenWidth: i32;
    let PenOrigWidth: i32;
    let mut ret: u8 = 1u8;
    let mut brushTemp: BRUSH = core::mem::zeroed();

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_RoundRect(dc, Left, Top, Right, Bottom, xCurveDiameter, yCurveDiameter) as u8;
    }

    if (Left == Right) || (Top == Bottom) {
        return 1u8;
    }

    let xCurveDiameter = max(abs(xCurveDiameter), 1);
    let yCurveDiameter = max(abs(yCurveDiameter), 1);

    if Right < Left {
        let tmp = Right;
        Right = Left;
        Left = tmp;
    }
    if Bottom < Top {
        let tmp = Bottom;
        Bottom = Top;
        Top = tmp;
    }

    pdcattr = (*dc).pdcattr;

    if ((*pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY)) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if ((*pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY)) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    pbrLine = PEN_ShareLockPen((*pdcattr).hpen as HPEN);
    if pbrLine.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0u8;
    }

    PenOrigWidth = (*pbrushbody(pbrLine)).lWidth;
    PenWidth = PenOrigWidth;
    if (*pbrushbody(pbrLine)).ulPenStyle == PS_NULL {
        PenWidth = 0;
    }

    if (*pbrushbody(pbrLine)).ulPenStyle == PS_INSIDEFRAME {
        if 2 * PenWidth > (Right - Left) {
            PenWidth = (Right - Left + 1) / 2;
        }
        if 2 * PenWidth > (Bottom - Top) {
            PenWidth = (Bottom - Top + 1) / 2;
        }
        Left += PenWidth / 2;
        Right -= (PenWidth - 1) / 2;
        Top += PenWidth / 2;
        Bottom -= (PenWidth - 1) / 2;
    }

    if PenWidth == 0 {
        PenWidth = 1;
    }
    (*pbrushbody(pbrLine)).lWidth = PenWidth;

    RectBounds.left = Left;
    RectBounds.top = Top;
    RectBounds.right = Right;
    RectBounds.bottom = Bottom;

    IntLPtoDP(dc, core::ptr::addr_of_mut!(RectBounds) as LPPOINT, 2);

    RectBounds.left += (*dc).ptlDCOrig.x;
    RectBounds.top += (*dc).ptlDCOrig.y;
    RectBounds.right += (*dc).ptlDCOrig.x;
    RectBounds.bottom += (*dc).ptlDCOrig.y;

    pbrFill = BRUSH_ShareLockBrush((*pdcattr).hbrush as HBRUSH);
    if pbrFill.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        ret = 0u8;
    } else {
        DC_vPrepareDCsForBlit(dc, core::ptr::addr_of_mut!(RectBounds), core::ptr::null_mut(), core::ptr::null_mut());

        RtlCopyMemory(
            core::ptr::addr_of_mut!(brushTemp) as *mut core::ffi::c_void,
            pbrFill as *const core::ffi::c_void,
            core::mem::size_of::<BRUSH>(),
        );
        (*pbrushbody(core::ptr::addr_of_mut!(brushTemp))).ptOrigin.x += RectBounds.left - Left;
        (*pbrushbody(core::ptr::addr_of_mut!(brushTemp))).ptOrigin.y += RectBounds.top - Top;
        ret = IntFillRoundRect(
            dc,
            RectBounds.left,
            RectBounds.top,
            RectBounds.right,
            RectBounds.bottom,
            xCurveDiameter,
            yCurveDiameter,
            core::ptr::addr_of_mut!(brushTemp),
        ) as u8;
        BRUSH_ShareUnlockBrush(pbrFill);

        if ret != 0 {
            ret = IntDrawRoundRect(
                dc,
                RectBounds.left,
                RectBounds.top,
                RectBounds.right,
                RectBounds.bottom,
                xCurveDiameter,
                yCurveDiameter,
                pbrLine,
            ) as u8;
        }

        DC_vFinishBlit(dc, core::ptr::null_mut());
    }

    (*pbrushbody(pbrLine)).lWidth = PenOrigWidth;
    PEN_ShareUnlockPen(pbrLine);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiRoundRect(
    hDC: HDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
    Width: i32,
    Height: i32,
) -> u8 {
    let dc: *mut DC = DC_LockDc(hDC);
    let mut ret: u8 = 0u8;

    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
    } else {
        ret = IntRoundRect(dc, LeftRect, TopRect, RightRect, BottomRect, Width, Height);
        DC_UnlockDc(dc);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn GreGradientFill(
    hdc: HDC,
    pVertex: PTRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nMesh: ULONG,
    ulMode: ULONG,
) -> BOOL {
    let mut pdc: PDC;
    let mut psurf: *mut SURFACE;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut rclExtent: RECTL = core::mem::zeroed();
    let mut ptlDitherOrg: POINTL = core::mem::zeroed();
    let mut i: ULONG;
    let bRet: BOOL;

    if ulMode == GRADIENT_FILL_TRIANGLE {
        let mut pTriangle = pMesh as PGRADIENT_TRIANGLE;

        i = 0;
        while i < nMesh {
            if (*pTriangle).Vertex1 >= nVertex ||
                (*pTriangle).Vertex2 >= nVertex ||
                (*pTriangle).Vertex3 >= nVertex
            {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            pTriangle = pTriangle.add(1);
            i += 1;
        }
    } else {
        let mut pRect = pMesh as PGRADIENT_RECT;
        i = 0;
        while i < nMesh {
            if (*pRect).UpperLeft >= nVertex || (*pRect).LowerRight >= nVertex {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                return 0;
            }
            pRect = pRect.add(1);
            i += 1;
        }
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if (*pdc).dclevel.pSurface.is_null() {
        DC_UnlockDc(pdc);
        return 1;
    }

    rclExtent.left = (*pVertex).x;
    rclExtent.right = (*pVertex).x;
    rclExtent.top = (*pVertex).y;
    rclExtent.bottom = (*pVertex).y;

    i = 0;
    while i < nVertex {
        rclExtent.left = min(rclExtent.left, (*pVertex.add(i as usize)).x);
        rclExtent.right = max(rclExtent.right, (*pVertex.add(i as usize)).x);
        rclExtent.top = min(rclExtent.top, (*pVertex.add(i as usize)).y);
        rclExtent.bottom = max(rclExtent.bottom, (*pVertex.add(i as usize)).y);
        i += 1;
    }

    IntLPtoDP(pdc, core::ptr::addr_of_mut!(rclExtent) as LPPOINT, 2);

    rclExtent.left += (*pdc).ptlDCOrig.x;
    rclExtent.right += (*pdc).ptlDCOrig.x;
    rclExtent.top += (*pdc).ptlDCOrig.y;
    rclExtent.bottom += (*pdc).ptlDCOrig.y;

    if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(rclExtent)) != 0 {
        DC_UnlockDc(pdc);
        return 1;
    }

    if ulMode == GRADIENT_FILL_RECT_H || ulMode == GRADIENT_FILL_RECT_V {
        i = 0;
        while i < nVertex {
            IntLPtoDP(pdc, core::ptr::addr_of_mut!((*pVertex.add(i as usize))) as LPPOINT, 1);
            (*pVertex.add(i as usize)).x += (*pdc).ptlDCOrig.x;
            (*pVertex.add(i as usize)).y += (*pdc).ptlDCOrig.y;
            i += 1;
        }
    }

    ptlDitherOrg.x = 0;
    ptlDitherOrg.y = 0;
    IntLPtoDP(pdc, core::ptr::addr_of_mut!(ptlDitherOrg) as LPPOINT, 1);

    ptlDitherOrg.x += (*pdc).ptlDCOrig.x;
    ptlDitherOrg.y += (*pdc).ptlDCOrig.y;

    if ((*pdc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
        IntUpdateBoundsRect(pdc, core::ptr::addr_of_mut!(rclExtent));
    }

    DC_vPrepareDCsForBlit(pdc, core::ptr::addr_of_mut!(rclExtent), core::ptr::null_mut(), core::ptr::null_mut());

    psurf = (*pdc).dclevel.pSurface;

    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exlo),
        core::ptr::addr_of!(gpalRGB) as *mut c_void,
        (*psurf).ppal as *mut c_void,
        0,
        0,
        0,
    );

    bRet = IntEngGradientFill(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        core::ptr::addr_of!((*pdc).co) as *mut CLIPOBJ,
        core::ptr::addr_of_mut!(exlo.xlo),
        pVertex,
        nVertex,
        pMesh,
        nMesh,
        core::ptr::addr_of_mut!(rclExtent),
        core::ptr::addr_of_mut!(ptlDitherOrg),
        ulMode,
    );

    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));
    DC_vFinishBlit(pdc, core::ptr::null_mut());
    DC_UnlockDc(pdc);

    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGradientFill(
    hdc: HDC,
    pVertex: PTRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nMesh: ULONG,
    ulMode: ULONG,
) -> BOOL {
    let mut bRet: BOOL;
    let mut SafeVertex: PTRIVERTEX;
    let mut SafeMesh: PVOID;
    let mut cbVertex: ULONG;
    let mut cbMesh: ULONG;
    let mut Status: NTSTATUS;

    if pVertex.is_null() || nVertex == 0 || pMesh.is_null() || nMesh == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    match ulMode {
        GRADIENT_FILL_RECT_H | GRADIENT_FILL_RECT_V => {
            cbMesh = nMesh.wrapping_mul(core::mem::size_of::<GRADIENT_RECT>() as ULONG);
        }
        GRADIENT_FILL_TRIANGLE => {
            cbMesh = nMesh.wrapping_mul(core::mem::size_of::<GRADIENT_TRIANGLE>() as ULONG);
        }
        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    cbVertex = nVertex.wrapping_mul(core::mem::size_of::<TRIVERTEX>() as ULONG);
    if cbVertex.wrapping_add(cbMesh) <= cbVertex {
        return 0;
    }

    SafeVertex = ExAllocatePoolWithTag(PagedPool, cbVertex.wrapping_add(cbMesh) as SIZE_T, TAG_SHAPE) as PTRIVERTEX;
    if SafeVertex.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    SafeMesh = (SafeVertex as ULONG_PTR).wrapping_add(cbVertex as ULONG_PTR) as PVOID;

    Status = SafeProbeForRead(pVertex as *const c_void, cbVertex as usize, 1);
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(pMesh as *const c_void, cbMesh as usize, 1);
    }
    // The probes VALIDATE only; both copies are the access and take the same exit.
    if !NT_SUCCESS(Status)
        || !crate::copy::capture_from_user(
            SafeVertex as *mut c_void,
            pVertex as *const c_void,
            cbVertex as usize,
        )
        || !crate::copy::capture_from_user(
            SafeMesh as *mut c_void,
            pMesh as *const c_void,
            cbMesh as usize,
        )
    {
        ExFreePoolWithTag(SafeVertex as PVOID, TAG_SHAPE);
        SetLastNtError(if NT_SUCCESS(Status) { STATUS_ACCESS_VIOLATION } else { Status });
        return 0;
    }

    bRet = GreGradientFill(hdc, SafeVertex, nVertex, SafeMesh, nMesh, ulMode);

    ExFreePoolWithTag(SafeVertex as PVOID, TAG_SHAPE);
    return bRet;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtFloodFill(
    hDC: HDC,
    XStart: i32,
    YStart: i32,
    Color: COLORREF,
    FillType: u32,
) -> u8 {
    let mut dc: PDC;
    let mut psurf: *mut SURFACE;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let mut Ret: u8 = 0u8;
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut Pt: POINTL = core::mem::zeroed();
    let mut ConvColor: u32;
    let mut prgn: PREGION;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0u8;
    }

    if (*dc).dclevel.pSurface.is_null() {
        Ret = 1u8;
        DC_UnlockDc(dc);
        return Ret;
    }

    Pt.x = XStart;
    Pt.y = YStart;
    IntLPtoDP(dc, &mut Pt as *mut POINTL as LPPOINT, 1);

    DC_vPrepareDCsForBlit(dc, &mut DestRect, core::ptr::null_mut(), core::ptr::null_mut());

    psurf = (*dc).dclevel.pSurface;

    prgn = if !(*dc).prgnRao.is_null() {
        (*dc).prgnRao
    } else {
        (*dc).prgnVis
    };

    if !prgn.is_null() {
        Ret = REGION_PtInRegion(prgn, Pt.x, Pt.y) as u8;
        if Ret != 0 {
            REGION_GetRgnBox(prgn, core::ptr::addr_of_mut!(DestRect) as *mut RECTL);
        } else {
            DC_vFinishBlit(dc, core::ptr::null_mut());
            DC_UnlockDc(dc);
            return Ret;
        }
    } else {
        RECTL_vSetRect(
            &mut DestRect,
            0,
            0,
            (*psurf).SurfObj.sizlBitmap.cx,
            (*psurf).SurfObj.sizlBitmap.cy,
        );
    }

    if ((*dc).fs as u32 & (DC_ACCUM_APP | DC_ACCUM_WMGR)) != 0 {
        IntUpdateBoundsRect(dc, &mut DestRect);
    }

    EXLATEOBJ_vInitialize(
        &mut exlo,
        &gpalRGB as *const _ as *mut core::ffi::c_void,
        (*psurf).ppal as *mut core::ffi::c_void,
        0,
        0xffffff,
        0,
    );

    ConvColor = XLATEOBJ_iXlate(&mut exlo.xlo, Color);
    Ret = DIB_XXBPP_FloodFillSolid(
        &mut (*psurf).SurfObj as *mut _ as *mut core::ffi::c_void,
        &mut (*dc).eboFill.BrushObject as *mut _ as *mut core::ffi::c_void,
        &mut DestRect,
        &mut Pt,
        ConvColor,
        FillType,
    );

    DC_vFinishBlit(dc, core::ptr::null_mut());

    EXLATEOBJ_vCleanup(&mut exlo);

    DC_UnlockDc(dc);
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/fillshap.c
//  * PURPOSE:         fillshap
//  * PROGRAMMER:
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define Rsin(d) ((d) == 0.0 ? 0.0 : ((d) == 90.0 ? 1.0 : sin(d*M_PI/180.0)))
// #define Rcos(d) ((d) == 0.0 ? 1.0 : ((d) == 90.0 ? 0.0 : cos(d*M_PI/180.0)))
//
// BOOL FASTCALL
// IntGdiPolygon(PDC    dc,
//               PPOINT Points,
//               int    Count)
// {
//     SURFACE *psurf;
//     PBRUSH pbrLine, pbrFill;
//     BOOL ret = FALSE; // Default to failure
//     RECTL DestRect;
//     INT i, CurrentPoint;
//     PDC_ATTR pdcattr;
//     POINTL BrushOrigin;
//     PPATH pPath;
// //    int Left;
// //    int Top;
//
//     ASSERT(dc); // Caller's responsibility to pass a valid dc
//
//     if (!Points || Count < 2 )
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
// /*
//     // Find start x, y
//     Left = Points[0].x;
//     Top  = Points[0].y;
//     for (CurrentPoint = 1; CurrentPoint < Count; ++CurrentPoint) {
//       Left = min(Left, Points[CurrentPoint].x);
//       Top  = min(Top, Points[CurrentPoint].y);
//     }
// */
//
//     pdcattr = dc->pdcattr;
//
//     /* Convert to screen coordinates */
//     IntLPtoDP(dc, Points, Count);
//     for (CurrentPoint = 0; CurrentPoint < Count; CurrentPoint++)
//     {
//         Points[CurrentPoint].x += dc->ptlDCOrig.x;
//         Points[CurrentPoint].y += dc->ptlDCOrig.y;
//     }
//     // No need to have path here.
//     {
//         DestRect.left   = Points[0].x;
//         DestRect.right  = Points[0].x;
//         DestRect.top    = Points[0].y;
//         DestRect.bottom = Points[0].y;
//
//         for (CurrentPoint = 1; CurrentPoint < Count; ++CurrentPoint)
//         {
//             DestRect.left     = min(DestRect.left, Points[CurrentPoint].x);
//             DestRect.right    = max(DestRect.right, Points[CurrentPoint].x);
//             DestRect.top      = min(DestRect.top, Points[CurrentPoint].y);
//             DestRect.bottom   = max(DestRect.bottom, Points[CurrentPoint].y);
//         }
//
//         if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//             DC_vUpdateFillBrush(dc);
//
//         if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//             DC_vUpdateLineBrush(dc);
//
//         /* Special locking order to avoid lock-ups */
//         pbrFill = dc->dclevel.pbrFill;
//         pbrLine = dc->dclevel.pbrLine;
//         psurf = dc->dclevel.pSurface;
//         if (psurf == NULL)
//         {
//             /* Memory DC without a bitmap selected, nothing to do. */
//             return TRUE;
//         }
//
//         /* Now fill the polygon with the current fill brush. */
//         if (!(pbrFill->flAttrs & BR_IS_NULL))
//         {
//             BrushOrigin = *((PPOINTL)&pbrFill->ptOrigin);
//             BrushOrigin.x += dc->ptlDCOrig.x;
//             BrushOrigin.y += dc->ptlDCOrig.y;
//             ret = IntFillPolygon (dc,
//                                   psurf,
//                                   &dc->eboFill.BrushObject,
//                                   Points,
//                                   Count,
//                                   DestRect,
//                                   &BrushOrigin);
//         }
//
//         // Draw the Polygon Edges with the current pen ( if not a NULL pen )
//         if (!(pbrLine->flAttrs & BR_IS_NULL))
//         {
//             if (IntIsEffectiveWidePen(pbrLine))
//             {
//                 /* Clear the path */
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//
//                 /* Begin a path */
//                 pPath = PATH_CreatePath(Count + 1);
//                 dc->dclevel.flPath |= DCPATH_ACTIVE;
//                 dc->dclevel.hPath = pPath->BaseObject.hHmgr;
//                 pPath->pos = Points[0];
//                 IntLPtoDP(dc, &pPath->pos, 1);
//
//                 PATH_MoveTo(dc, pPath);
//                 for (i = 1; i < Count; ++i)
//                 {
//                     PATH_LineTo(dc, Points[i].x, Points[i].y);
//                 }
//                 PATH_LineTo(dc, Points[0].x, Points[0].y);
//
//                 /* Close the path */
//                 pPath->state = PATH_Closed;
//                 dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//                 /* Actually stroke a path */
//                 ret = PATH_StrokePath(dc, pPath);
//
//                 /* Clear the path */
//                 PATH_UnlockPath(pPath as POBJ);
//                 PATH_Delete(dc->dclevel.hPath);
//                 dc->dclevel.hPath = NULL;
//             }
//             else
//             {
//                 for (i = 0; i < Count-1; i++)
//                 {
// // DPRINT1("Polygon Making line from (%d,%d) to (%d,%d)\n",
// //                                 Points[0].x, Points[0].y,
// //                                 Points[1].x, Points[1].y );
//
//                     ret = IntEngLineTo(&psurf->SurfObj,
//                                        (CLIPOBJ *)&dc->co,
//                                        &dc->eboLine.BrushObject,
//                                        Points[i].x,          /* From */
//                                        Points[i].y,
//                                        Points[i+1].x,        /* To */
//                                        Points[i+1].y,
//                                        &DestRect,
//                                        ROP2_TO_MIX(pdcattr->jROP2)); /* MIX */
//                     if (!ret) break;
//                 }
//                 /* Close the polygon */
//                 if (ret)
//                 {
//                     ret = IntEngLineTo(&psurf->SurfObj,
//                                        (CLIPOBJ *)&dc->co,
//                                        &dc->eboLine.BrushObject,
//                                        Points[Count-1].x, /* From */
//                                        Points[Count-1].y,
//                                        Points[0].x,       /* To */
//                                        Points[0].y,
//                                        &DestRect,
//                                        ROP2_TO_MIX(pdcattr->jROP2)); /* MIX */
//                 }
//             }
//         }
//     }
//
//     return ret;
// }
//
// BOOL FASTCALL
// IntGdiPolyPolygon(DC      *dc,
//                   LPPOINT Points,
//                   PULONG  PolyCounts,
//                   int     Count)
// {
//     if (PATH_IsPathOpen(dc->dclevel))
//         return PATH_PolyPolygon ( dc, Points, (PINT)PolyCounts, Count);
//
//     while (--Count >=0)
//     {
//         if (!IntGdiPolygon ( dc, Points, *PolyCounts ))
//             return FALSE;
//         Points+=*PolyCounts++;
//     }
//     return TRUE;
// }
//
// BOOL FASTCALL
// IntPolygon(HDC hdc, POINT *Point, int Count)
// {
//     BOOL bResult;
//     PDC pdc;
//
//     pdc = DC_LockDc(hdc);
//     if (pdc == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     bResult = IntGdiPolygon(pdc, Point, Count);
//
//     DC_UnlockDc(pdc);
//     return bResult;
// }
//
//
// /******************************************************************************/
//
// /*
//  * NtGdiEllipse
//  *
//  * Author
//  *    Filip Navara
//  *
//  * Remarks
//  *    This function uses optimized Bresenham's ellipse algorithm. It draws
//  *    four lines of the ellipse in one pass.
//  *
//  */
//
// BOOL APIENTRY
// NtGdiEllipse(
//     HDC hDC,
//     int Left,
//     int Top,
//     int Right,
//     int Bottom)
// {
//     PDC dc;
//     PDC_ATTR pdcattr;
//     RECTL RectBounds;
//     PBRUSH pbrush;
//     BOOL ret = TRUE;
//     LONG PenWidth, PenOrigWidth;
//     LONG RadiusX, RadiusY, CenterX, CenterY;
//     PBRUSH pFillBrushObj;
//     BRUSH tmpFillBrushObj;
//
//     dc = DC_LockDc(hDC);
//     if (dc == NULL)
//     {
//        EngSetLastError(ERROR_INVALID_HANDLE);
//        return FALSE;
//     }
//
//     if (PATH_IsPathOpen(dc->dclevel))
//     {
//         ret = PATH_Ellipse(dc, Left, Top, Right, Bottom);
//         DC_UnlockDc(dc);
//         return ret;
//     }
//
//     ////
//     //// Could this use PATH_CheckCorners ?
//     ////
//     if ((Left == Right) || (Top == Bottom))
//     {
//        DC_UnlockDc(dc);
//        return TRUE;
//     }
//
//     if (Right < Left)
//     {
//        INT tmp = Right; Right = Left; Left = tmp;
//     }
//     if (Bottom < Top)
//     {
//        INT tmp = Bottom; Bottom = Top; Top = tmp;
//     }
//     ////
//
//     pdcattr = dc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(dc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(dc);
//
//     pbrush = PEN_ShareLockPen(pdcattr->hpen);
//     if (!pbrush)
//     {
//         DPRINT1("Ellipse Fail 1\n");
//         DC_UnlockDc(dc);
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         return FALSE;
//     }
//
//     PenOrigWidth = PenWidth = pbrush->lWidth;
//     if (pbrush->ulPenStyle == PS_NULL) PenWidth = 0;
//
//     if (pbrush->ulPenStyle == PS_INSIDEFRAME)
//     {
//        if (2*PenWidth > (Right - Left)) PenWidth = (Right -Left + 1)/2;
//        if (2*PenWidth > (Bottom - Top)) PenWidth = (Bottom -Top + 1)/2;
//        Left   += PenWidth / 2;
//        Right  -= (PenWidth - 1) / 2;
//        Top    += PenWidth / 2;
//        Bottom -= (PenWidth - 1) / 2;
//     }
//
//     if (!PenWidth) PenWidth = 1;
//     pbrush->lWidth = PenWidth;
//
//     RectBounds.left   = Left;
//     RectBounds.right  = Right;
//     RectBounds.top    = Top;
//     RectBounds.bottom = Bottom;
//
//     IntLPtoDP(dc, (LPPOINT)&RectBounds, 2);
//
//     RectBounds.left += dc->ptlDCOrig.x;
//     RectBounds.right += dc->ptlDCOrig.x;
//     RectBounds.top += dc->ptlDCOrig.y;
//     RectBounds.bottom += dc->ptlDCOrig.y;
//
//     // Setup for dynamic width and height.
//     RadiusX = max((RectBounds.right - RectBounds.left) / 2, 2); // Needs room
//     RadiusY = max((RectBounds.bottom - RectBounds.top) / 2, 2);
//     CenterX = (RectBounds.right + RectBounds.left) / 2;
//     CenterY = (RectBounds.bottom + RectBounds.top) / 2;
//
//     DPRINT("Ellipse 1: Left: %d, Top: %d, Right: %d, Bottom: %d\n",
//                RectBounds.left,RectBounds.top,RectBounds.right,RectBounds.bottom);
//
//     DPRINT("Ellipse 2: XLeft: %d, YLeft: %d, Width: %d, Height: %d\n",
//                CenterX - RadiusX, CenterY + RadiusY, RadiusX*2, RadiusY*2);
//
//     pFillBrushObj = BRUSH_ShareLockBrush(pdcattr->hbrush);
//     if (NULL == pFillBrushObj)
//     {
//         DPRINT1("FillEllipse Fail\n");
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         ret = FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(&tmpFillBrushObj, pFillBrushObj, sizeof(tmpFillBrushObj));
//         //(*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.x += RectBounds.left - Left;
//         //(*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.y += RectBounds.top - Top;
//         (*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.x += dc->ptlDCOrig.x;
//         (*pbrushbody(core::ptr::addr_of_mut!(tmpFillBrushObj))).ptOrigin.y += dc->ptlDCOrig.y;
//
//         DC_vPrepareDCsForBlit(dc, &RectBounds, NULL, NULL);
//
//         ret = IntFillEllipse( dc,
//                               CenterX - RadiusX,
//                               CenterY - RadiusY,
//                               RadiusX*2, // Width
//                               RadiusY*2, // Height
//                               &tmpFillBrushObj);
//         BRUSH_ShareUnlockBrush(pFillBrushObj);
//
//         if (ret)
//         {
//            ret = IntDrawEllipse( dc,
//                                  CenterX - RadiusX,
//                                  CenterY - RadiusY,
//                                  RadiusX*2, // Width
//                                  RadiusY*2, // Height
//                                  pbrush);
//         }
//
//         DC_vFinishBlit(dc, NULL);
//     }
//
//     pbrush->lWidth = PenOrigWidth;
//     PEN_ShareUnlockPen(pbrush);
//     DC_UnlockDc(dc);
//     DPRINT("Ellipse Exit.\n");
//     return ret;
// }
//
// #if 0
//
// // When the fill mode is ALTERNATE, GDI fills the area between odd-numbered and
// // even-numbered polygon sides on each scan line. That is, GDI fills the area between the
// // first and second side, between the third and fourth side, and so on.
//
// // WINDING Selects winding mode (fills any region with a nonzero winding value).
// // When the fill mode is WINDING, GDI fills any region that has a nonzero winding value.
// // This value is defined as the number of times a pen used to draw the polygon would go around the region.
// // The direction of each edge of the polygon is important.
//
// extern BOOL FillPolygon(PDC dc,
//                             SURFOBJ *SurfObj,
//                             PBRUSHOBJ BrushObj,
//                             MIX RopMode,
//                             CONST PPOINT Points,
//                             int Count,
//                             RECTL BoundRect);
//
// #endif
//
//
// ULONG_PTR
// APIENTRY
// NtGdiPolyPolyDraw( IN HDC hDC,
//                    IN PPOINT UnsafePoints,
//                    IN PULONG UnsafeCounts,
//                    IN ULONG Count,
//                    IN INT iFunc )
// {
//     DC *dc;
//     PVOID pTemp;
//     LPPOINT SafePoints;
//     PULONG SafeCounts;
//     NTSTATUS Status = STATUS_SUCCESS;
//     BOOL Ret = TRUE;
//     ULONG nPoints = 0, nMaxPoints = 0, nInvalid = 0, i;
//
//     if (!UnsafePoints || !UnsafeCounts ||
//         Count == 0 || iFunc == 0 || iFunc > GdiPolyPolyRgn)
//     {
//         /* Windows doesn't set last error */
//         return FALSE;
//     }
//
//     _SEH2_TRY
//     {
//         ProbeForRead(UnsafePoints, Count * sizeof(POINT), 1);
//         ProbeForRead(UnsafeCounts, Count * sizeof(ULONG), 1);
//
//         /* Count points and validate poligons */
//         for (i = 0; i < Count; i++)
//         {
//             if (UnsafeCounts[i] < 2)
//             {
//                 nInvalid++;
//             }
//             nPoints += UnsafeCounts[i];
//             nMaxPoints = max(nMaxPoints, UnsafeCounts[i]);
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Windows doesn't set last error */
//         return FALSE;
//     }
//
//     if (nPoints == 0 || nPoints < nMaxPoints)
//     {
//         /* If all polygon counts are zero, or we have overflow,
//            return without setting a last error code. */
//         return FALSE;
//     }
//
//     if (nInvalid != 0)
//     {
//         /* If at least one poly count is 0 or 1, fail */
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Allocate one buffer for both counts and points */
//     pTemp = ExAllocatePoolWithTag(PagedPool,
//                                   Count * sizeof(ULONG) + nPoints * sizeof(POINT),
//                                   TAG_SHAPE);
//     if (!pTemp)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     SafeCounts = pTemp;
//     SafePoints = (PVOID)(SafeCounts + Count);
//
//     _SEH2_TRY
//     {
//         /* Pointers already probed! */
//         RtlCopyMemory(SafeCounts, UnsafeCounts, Count * sizeof(ULONG));
//         RtlCopyMemory(SafePoints, UnsafePoints, nPoints * sizeof(POINT));
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(pTemp, TAG_SHAPE);
//         return FALSE;
//     }
//
//     /* Special handling for GdiPolyPolyRgn */
//     if (iFunc == GdiPolyPolyRgn)
//     {
//         INT iMode = (INT)(UINT_PTR)hDC;
//         HRGN hrgn;
//
//         hrgn = GreCreatePolyPolygonRgn(SafePoints as *const POINTL, SafeCounts, Count, iMode) as usize;
//
//         ExFreePoolWithTag(pTemp, TAG_SHAPE);
//         return (ULONG_PTR)hrgn;
//     }
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         ExFreePoolWithTag(pTemp, TAG_SHAPE);
//         return FALSE;
//     }
//
//     DC_vPrepareDCsForBlit(dc, NULL, NULL, NULL);
//
//     if (dc->pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(dc);
//
//     if (dc->pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(dc);
//
//     /* Perform the actual work */
//     switch (iFunc)
//     {
//         case GdiPolyPolygon:
//             Ret = IntGdiPolyPolygon(dc, SafePoints, SafeCounts, Count as i32);
//             break;
//         case GdiPolyPolyLine:
//             Ret = IntGdiPolyPolyline(dc, SafePoints as *mut POINTL, SafeCounts, Count);
//             break;
//         case GdiPolyBezier:
//             Ret = IntGdiPolyBezier(dc, SafePoints as *mut POINTL, *SafeCounts);
//             break;
//         case GdiPolyLineTo:
//             Ret = IntGdiPolylineTo(dc, SafePoints as *mut POINTL, *SafeCounts);
//             break;
//         case GdiPolyBezierTo:
//             Ret = IntGdiPolyBezierTo(dc, SafePoints as *mut POINTL, *SafeCounts);
//             break;
//         default:
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             Ret = FALSE;
//     }
//
//     /* Cleanup and return */
//     DC_vFinishBlit(dc, NULL);
//     DC_UnlockDc(dc);
//     ExFreePoolWithTag(pTemp, TAG_SHAPE);
//
//     return (ULONG_PTR)Ret;
// }
//
//
// BOOL
// FASTCALL
// IntRectangle(PDC dc,
//              int LeftRect,
//              int TopRect,
//              int RightRect,
//              int BottomRect)
// {
//     SURFACE *psurf = NULL;
//     PBRUSH pbrLine, pbrFill;
//     BOOL       ret = FALSE; // Default to failure
//     RECTL      DestRect;
//     MIX        Mix;
//     PDC_ATTR pdcattr;
//     POINTL BrushOrigin;
//     PPATH pPath;
//
//     ASSERT ( dc ); // Caller's responsibility to set this up
//
//     pdcattr = dc->pdcattr;
//
//     // Rectangle Path only.
//     if ( PATH_IsPathOpen(dc->dclevel) )
//     {
//         return PATH_Rectangle ( dc, LeftRect, TopRect, RightRect, BottomRect );
//     }
//
// 	/* Make sure rectangle is not inverted */
//     DestRect.left   = min(LeftRect, RightRect);
//     DestRect.right  = max(LeftRect, RightRect);
//     DestRect.top    = min(TopRect,  BottomRect);
//     DestRect.bottom = max(TopRect,  BottomRect);
//
//     IntLPtoDP(dc, (LPPOINT)&DestRect, 2);
//
//     DestRect.left   += dc->ptlDCOrig.x;
//     DestRect.right  += dc->ptlDCOrig.x;
//     DestRect.top    += dc->ptlDCOrig.y;
//     DestRect.bottom += dc->ptlDCOrig.y;
//
//     if (dc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//     {
//        IntUpdateBoundsRect(dc, &DestRect);
//     }
//
//     /* In GM_COMPATIBLE, don't include bottom and right edges */
//     if (pdcattr->iGraphicsMode == GM_COMPATIBLE)
//     {
//         DestRect.right--;
//         DestRect.bottom--;
//     }
//
//     DC_vPrepareDCsForBlit(dc, &DestRect, NULL, NULL);
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(dc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(dc);
//
//     pbrFill = dc->dclevel.pbrFill;
//     pbrLine = dc->dclevel.pbrLine;
//     if (!pbrLine)
//     {
//         ret = FALSE;
//         goto cleanup;
//     }
//
//     psurf = dc->dclevel.pSurface;
//     if (!psurf)
//     {
//         ret = TRUE;
//         goto cleanup;
//     }
//
//     if (pbrFill)
//     {
//         if (!(pbrFill->flAttrs & BR_IS_NULL))
//         {
//             BrushOrigin = *((PPOINTL)&pbrFill->ptOrigin);
//             BrushOrigin.x += dc->ptlDCOrig.x;
//             BrushOrigin.y += dc->ptlDCOrig.y;
//             ret = IntEngBitBlt(&psurf->SurfObj,
//                                NULL,
//                                NULL,
//                                (CLIPOBJ *)&dc->co,
//                                NULL,
//                                &DestRect,
//                                NULL,
//                                NULL,
//                                &dc->eboFill.BrushObject,
//                                &BrushOrigin,
//                                ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY));
//         }
//     }
//
//     // Draw the rectangle with the current pen
//
//     ret = TRUE; // Change default to success
//
//     if (!(pbrLine->flAttrs & BR_IS_NULL))
//     {
//         if (IntIsEffectiveWidePen(pbrLine))
//         {
//             /* Clear the path */
//             PATH_Delete(dc->dclevel.hPath);
//             dc->dclevel.hPath = NULL;
//
//             /* Begin a path */
//             pPath = PATH_CreatePath(5);
//             dc->dclevel.flPath |= DCPATH_ACTIVE;
//             dc->dclevel.hPath = pPath->BaseObject.hHmgr;
//             pPath->pos.x = LeftRect;
//             pPath->pos.y = TopRect;
//             IntLPtoDP(dc, &pPath->pos, 1);
//
//             PATH_MoveTo(dc, pPath);
//             PATH_LineTo(dc, RightRect, TopRect);
//             PATH_LineTo(dc, RightRect, BottomRect);
//             PATH_LineTo(dc, LeftRect, BottomRect);
//             PATH_LineTo(dc, LeftRect, TopRect);
//
//             /* Close the path */
//             pPath->state = PATH_Closed;
//             dc->dclevel.flPath &= ~DCPATH_ACTIVE;
//
//             /* Actually stroke a path */
//             ret = PATH_StrokePath(dc, pPath);
//
//             /* Clear the path */
//             PATH_UnlockPath(pPath as POBJ);
//             PATH_Delete(dc->dclevel.hPath);
//             dc->dclevel.hPath = NULL;
//         }
//         else
//         {
//             Mix = ROP2_TO_MIX(pdcattr->jROP2);
//             ret = ret && IntEngLineTo(&psurf->SurfObj,
//                                       (CLIPOBJ *)&dc->co,
//                                       &dc->eboLine.BrushObject,
//                                       DestRect.left, DestRect.top, DestRect.right, DestRect.top,
//                                       &DestRect, // Bounding rectangle
//                                       Mix);
//
//             ret = ret && IntEngLineTo(&psurf->SurfObj,
//                                       (CLIPOBJ *)&dc->co,
//                                       &dc->eboLine.BrushObject,
//                                       DestRect.right, DestRect.top, DestRect.right, DestRect.bottom,
//                                       &DestRect, // Bounding rectangle
//                                       Mix);
//
//             ret = ret && IntEngLineTo(&psurf->SurfObj,
//                                       (CLIPOBJ *)&dc->co,
//                                       &dc->eboLine.BrushObject,
//                                       DestRect.right, DestRect.bottom, DestRect.left, DestRect.bottom,
//                                       &DestRect, // Bounding rectangle
//                                       Mix);
//
//             ret = ret && IntEngLineTo(&psurf->SurfObj,
//                                       (CLIPOBJ *)&dc->co,
//                                       &dc->eboLine.BrushObject,
//                                       DestRect.left, DestRect.bottom, DestRect.left, DestRect.top,
//                                       &DestRect, // Bounding rectangle
//                                       Mix);
//         }
//     }
//
// cleanup:
//     DC_vFinishBlit(dc, NULL);
//
//     /* Move current position in DC?
//        MSDN: The current position is neither used nor updated by Rectangle. */
//
//     return ret;
// }
//
// BOOL
// APIENTRY
// NtGdiRectangle(HDC  hDC,
//                int  LeftRect,
//                int  TopRect,
//                int  RightRect,
//                int  BottomRect)
// {
//     DC   *dc;
//     BOOL ret; // Default to failure
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Do we rotate or shear? */
//     if (!(dc->pdcattr->mxWorldToDevice.flAccel & XFORM_SCALE))
//     {
//         POINTL DestCoords[4];
//         ULONG PolyCounts = 4;
//
//         DestCoords[0].x = DestCoords[3].x = LeftRect;
//         DestCoords[0].y = DestCoords[1].y = TopRect;
//         DestCoords[1].x = DestCoords[2].x = RightRect;
//         DestCoords[2].y = DestCoords[3].y = BottomRect;
//         // Use IntGdiPolyPolygon so to support PATH.
//         ret = IntGdiPolyPolygon(dc, DestCoords, &PolyCounts, 1);
//     }
//     else
//     {
//         ret = IntRectangle(dc, LeftRect, TopRect, RightRect, BottomRect );
//     }
//
//     DC_UnlockDc(dc);
//
//     return ret;
// }
//
//
// BOOL
// FASTCALL
// IntRoundRect(
//     PDC  dc,
//     int  Left,
//     int  Top,
//     int  Right,
//     int  Bottom,
//     int  xCurveDiameter,
//     int  yCurveDiameter)
// {
//     PDC_ATTR pdcattr;
//     PBRUSH   pbrLine, pbrFill;
//     RECTL RectBounds;
//     LONG PenWidth, PenOrigWidth;
//     BOOL ret = TRUE; // Default to success
//     BRUSH brushTemp;
//
//     ASSERT ( dc ); // Caller's responsibility to set this up
//
//     if ( PATH_IsPathOpen(dc->dclevel) )
//         return PATH_RoundRect ( dc, Left, Top, Right, Bottom,
//                                 xCurveDiameter, yCurveDiameter );
//
//     if ((Left == Right) || (Top == Bottom)) return TRUE;
//
//     xCurveDiameter = max(abs( xCurveDiameter ), 1);
//     yCurveDiameter = max(abs( yCurveDiameter ), 1);
//
//     if (Right < Left)
//     {
//        INT tmp = Right; Right = Left; Left = tmp;
//     }
//     if (Bottom < Top)
//     {
//        INT tmp = Bottom; Bottom = Top; Top = tmp;
//     }
//
//     pdcattr = dc->pdcattr;
//
//     if (pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//         DC_vUpdateFillBrush(dc);
//
//     if (pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//         DC_vUpdateLineBrush(dc);
//
//     pbrLine = PEN_ShareLockPen(pdcattr->hpen);
//     if (!pbrLine)
//     {
//         /* Nothing to do, as we don't have a bitmap */
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         return FALSE;
//     }
//
//     PenOrigWidth = PenWidth = pbrLine->lWidth;
//     if (pbrLine->ulPenStyle == PS_NULL) PenWidth = 0;
//
//     if (pbrLine->ulPenStyle == PS_INSIDEFRAME)
//     {
//        if (2*PenWidth > (Right - Left)) PenWidth = (Right -Left + 1)/2;
//        if (2*PenWidth > (Bottom - Top)) PenWidth = (Bottom -Top + 1)/2;
//        Left   += PenWidth / 2;
//        Right  -= (PenWidth - 1) / 2;
//        Top    += PenWidth / 2;
//        Bottom -= (PenWidth - 1) / 2;
//     }
//
//     if (!PenWidth) PenWidth = 1;
//     pbrLine->lWidth = PenWidth;
//
//     RectBounds.left = Left;
//     RectBounds.top = Top;
//     RectBounds.right = Right;
//     RectBounds.bottom = Bottom;
//
//     IntLPtoDP(dc, (LPPOINT)&RectBounds, 2);
//
//     RectBounds.left   += dc->ptlDCOrig.x;
//     RectBounds.top    += dc->ptlDCOrig.y;
//     RectBounds.right  += dc->ptlDCOrig.x;
//     RectBounds.bottom += dc->ptlDCOrig.y;
//
//     pbrFill = BRUSH_ShareLockBrush(pdcattr->hbrush);
//     if (!pbrFill)
//     {
//         DPRINT1("FillRound Fail\n");
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         ret = FALSE;
//     }
//     else
//     {
//
//         DC_vPrepareDCsForBlit(dc, &RectBounds, NULL, NULL);
//
//         RtlCopyMemory(&brushTemp, pbrFill, sizeof(brushTemp));
//         (*pbrushbody(core::ptr::addr_of_mut!(brushTemp))).ptOrigin.x += RectBounds.left - Left;
//         (*pbrushbody(core::ptr::addr_of_mut!(brushTemp))).ptOrigin.y += RectBounds.top - Top;
//         ret = IntFillRoundRect( dc,
//                                 RectBounds.left,
//                                 RectBounds.top,
//                                 RectBounds.right,
//                                 RectBounds.bottom,
//                                 xCurveDiameter,
//                                 yCurveDiameter,
//                                 &brushTemp);
//         BRUSH_ShareUnlockBrush(pbrFill);
//
//         if (ret)
//         {
//            ret = IntDrawRoundRect( dc,
//                       RectBounds.left,
//                        RectBounds.top,
//                      RectBounds.right,
//                     RectBounds.bottom,
//                        xCurveDiameter,
//                        yCurveDiameter,
//                        pbrLine);
//         }
//
//         DC_vFinishBlit(dc, NULL);
//     }
//
//
//     pbrLine->lWidth = PenOrigWidth;
//     PEN_ShareUnlockPen(pbrLine);
//     return ret;
// }
//
// BOOL
// APIENTRY
// NtGdiRoundRect(
//     HDC  hDC,
//     int  LeftRect,
//     int  TopRect,
//     int  RightRect,
//     int  BottomRect,
//     int  Width,
//     int  Height)
// {
//     DC   *dc = DC_LockDc(hDC);
//     BOOL  ret = FALSE; /* Default to failure */
//
//     DPRINT("NtGdiRoundRect(0x%p,%i,%i,%i,%i,%i,%i)\n",hDC,LeftRect,TopRect,RightRect,BottomRect,Width,Height);
//     if ( !dc )
//     {
//         DPRINT1("NtGdiRoundRect() - hDC is invalid\n");
//         EngSetLastError(ERROR_INVALID_HANDLE);
//     }
//     else
//     {
//         ret = IntRoundRect ( dc, LeftRect, TopRect, RightRect, BottomRect, Width, Height );
//         DC_UnlockDc ( dc );
//     }
//
//     return ret;
// }
//
// BOOL
// NTAPI
// GreGradientFill(
//     HDC hdc,
//     PTRIVERTEX pVertex,
//     ULONG nVertex,
//     PVOID pMesh,
//     ULONG nMesh,
//     ULONG ulMode)
// {
//     PDC pdc;
//     SURFACE *psurf;
//     EXLATEOBJ exlo;
//     RECTL rclExtent;
//     POINTL ptlDitherOrg;
//     ULONG i;
//     BOOL bRet;
//
//     /* Check parameters */
//     if (ulMode == GRADIENT_FILL_TRIANGLE)
//     {
//         PGRADIENT_TRIANGLE pTriangle = (PGRADIENT_TRIANGLE)pMesh;
//
//         for (i = 0; i < nMesh; i++, pTriangle++)
//         {
//             if (pTriangle->Vertex1 >= nVertex ||
//                 pTriangle->Vertex2 >= nVertex ||
//                 pTriangle->Vertex3 >= nVertex)
//             {
//                 EngSetLastError(ERROR_INVALID_PARAMETER);
//                 return FALSE;
//             }
//         }
//     }
//     else
//     {
//         PGRADIENT_RECT pRect = (PGRADIENT_RECT)pMesh;
//         for (i = 0; i < nMesh; i++, pRect++)
//         {
//             if (pRect->UpperLeft >= nVertex || pRect->LowerRight >= nVertex)
//             {
//                 EngSetLastError(ERROR_INVALID_PARAMETER);
//                 return FALSE;
//             }
//         }
//     }
//
//     /* Lock the output DC */
//     pdc = DC_LockDc(hdc);
//     if(!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (!pdc->dclevel.pSurface)
//     {
//         /* Memory DC with no surface selected */
//         DC_UnlockDc(pdc);
//         return TRUE; // CHECKME
//     }
//
//     /* Calculate extent */
//     rclExtent.left = rclExtent.right = pVertex->x;
//     rclExtent.top = rclExtent.bottom = pVertex->y;
//     for (i = 0; i < nVertex; i++)
//     {
//         rclExtent.left = min(rclExtent.left, (pVertex + i)->x);
//         rclExtent.right = max(rclExtent.right, (pVertex + i)->x);
//         rclExtent.top = min(rclExtent.top, (pVertex + i)->y);
//         rclExtent.bottom = max(rclExtent.bottom, (pVertex + i)->y);
//     }
//     IntLPtoDP(pdc, (LPPOINT)&rclExtent, 2);
//
//     rclExtent.left   += pdc->ptlDCOrig.x;
//     rclExtent.right  += pdc->ptlDCOrig.x;
//     rclExtent.top    += pdc->ptlDCOrig.y;
//     rclExtent.bottom += pdc->ptlDCOrig.y;
//
//     if (RECTL_bIsEmptyRect(&rclExtent))
//     {
//         DC_UnlockDc(pdc);
//         return TRUE;
//     }
//
//     /* Offset vertex for rectangles */
//     if (ulMode == GRADIENT_FILL_RECT_H ||
//         ulMode == GRADIENT_FILL_RECT_V)
//     {
//         for (i = 0; i < nVertex; i++)
//         {
//             IntLPtoDP(pdc, (LPPOINT)&pVertex[i], 1);
//             pVertex[i].x += pdc->ptlDCOrig.x;
//             pVertex[i].y += pdc->ptlDCOrig.y;
//         }
//     }
//
//     ptlDitherOrg.x = ptlDitherOrg.y = 0;
//     IntLPtoDP(pdc, (LPPOINT)&ptlDitherOrg, 1);
//
//     ptlDitherOrg.x += pdc->ptlDCOrig.x;
//     ptlDitherOrg.y += pdc->ptlDCOrig.y;
//
//    if (pdc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//    {
//       IntUpdateBoundsRect(pdc, &rclExtent);
//    }
//
//     DC_vPrepareDCsForBlit(pdc, &rclExtent, NULL, NULL);
//
//     psurf = pdc->dclevel.pSurface;
//
//     EXLATEOBJ_vInitialize(&exlo, &gpalRGB, psurf->ppal, 0, 0, 0);
//
//     bRet = IntEngGradientFill(&psurf->SurfObj,
//                              (CLIPOBJ *)&pdc->co,
//                              &exlo.xlo,
//                              pVertex,
//                              nVertex,
//                              pMesh,
//                              nMesh,
//                              &rclExtent,
//                              &ptlDitherOrg,
//                              ulMode);
//
//     EXLATEOBJ_vCleanup(&exlo);
//     DC_vFinishBlit(pdc, NULL);
//     DC_UnlockDc(pdc);
//
//     return bRet;
// }
//
// BOOL
// APIENTRY
// NtGdiGradientFill(
//     HDC hdc,
//     PTRIVERTEX pVertex,
//     ULONG nVertex,
//     PVOID pMesh,
//     ULONG nMesh,
//     ULONG ulMode)
// {
//     BOOL bRet;
//     PTRIVERTEX SafeVertex;
//     PVOID SafeMesh;
//     ULONG cbVertex, cbMesh;
//     NTSTATUS Status;
//
//     /* Validate parameters */
//     if (!pVertex || !nVertex || !pMesh || !nMesh)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     switch (ulMode)
//     {
//         case GRADIENT_FILL_RECT_H:
//         case GRADIENT_FILL_RECT_V:
//             cbMesh = nMesh * sizeof(GRADIENT_RECT);
//             break;
//         case GRADIENT_FILL_TRIANGLE:
//             cbMesh = nMesh * sizeof(GRADIENT_TRIANGLE);
//             break;
//         default:
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             return FALSE;
//     }
//
//     cbVertex = nVertex * sizeof(TRIVERTEX) ;
//     if(cbVertex + cbMesh <= cbVertex)
//     {
//         /* Overflow */
//         return FALSE ;
//     }
//
//     /* Allocate a kernel mode buffer */
//     SafeVertex = ExAllocatePoolWithTag(PagedPool, cbVertex + cbMesh, TAG_SHAPE);
//     if(!SafeVertex)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         return FALSE;
//     }
//
//     SafeMesh = (PVOID)((ULONG_PTR)SafeVertex + cbVertex);
//
//     /* Copy the parameters to kernel mode */
//     Status = SafeProbeForRead(pVertex, cbVertex, 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pMesh as *const c_void, cbMesh as usize, 1);
//     if (!NT_SUCCESS(Status))
//     {
//         ExFreePoolWithTag(SafeVertex, TAG_SHAPE);
//         SetLastNtError(Status);
//         return FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(SafeVertex, pVertex, cbVertex);
//         RtlCopyMemory(SafeMesh, pMesh, cbMesh);
//     }
//
//     /* Call the internal function */
//     bRet = GreGradientFill(hdc, SafeVertex, nVertex, SafeMesh, nMesh, ulMode);
//
//     /* Cleanup and return result */
//     ExFreePoolWithTag(SafeVertex, TAG_SHAPE);
//     return bRet;
// }
//
//
// BOOL APIENTRY
// NtGdiExtFloodFill(
//     HDC  hDC,
//     INT  XStart,
//     INT  YStart,
//     COLORREF  Color,
//     UINT  FillType)
// {
//     PDC dc;
// #if 0
//     PDC_ATTR   pdcattr;
// #endif
//     SURFACE    *psurf;
//     EXLATEOBJ  exlo;
//     BOOL       Ret = FALSE;
//     RECTL      DestRect;
//     POINTL     Pt;
//     ULONG      ConvColor;
//     PREGION    prgn;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (!dc->dclevel.pSurface)
//     {
//         Ret = TRUE;
//         goto cleanup;
//     }
//
// #if 0
//     pdcattr = dc->pdcattr;
// #endif
//
//     Pt.x = XStart;
//     Pt.y = YStart;
//     IntLPtoDP(dc, (LPPOINT)&Pt, 1);
//
//     DC_vPrepareDCsForBlit(dc, &DestRect, NULL, NULL);
//
//     psurf = dc->dclevel.pSurface;
//
//     prgn = dc->prgnRao ? dc->prgnRao : dc->prgnVis;
//     if (prgn)
//     {
//         Ret = REGION_PtInRegion(prgn, Pt.x, Pt.y) as u8;
//         if (Ret)
//             REGION_GetRgnBox(prgn, (LPRECT)&DestRect);
//         else
//         {
//             DC_vFinishBlit(dc, NULL);
//             goto cleanup;
//         }
//     }
//     else
//     {
//         RECTL_vSetRect(&DestRect, 0, 0, psurf->SurfObj.sizlBitmap.cx, psurf->SurfObj.sizlBitmap.cy);
//     }
//
//     if (dc->fs & (DC_ACCUM_APP|DC_ACCUM_WMGR))
//     {
//        IntUpdateBoundsRect(dc, &DestRect);
//     }
//
//     EXLATEOBJ_vInitialize(&exlo, &gpalRGB, psurf->ppal, 0, 0xffffff, 0);
//
//     /* Only solid fills supported for now
//      * How to support pattern brushes and non standard surfaces (not offering dib functions):
//      * Version a (most likely slow): call DrvPatBlt for every pixel
//      * Version b: create a flood mask and let MaskBlt blit a masked brush */
//     ConvColor = XLATEOBJ_iXlate(&exlo.xlo, Color);
//     Ret = DIB_XXBPP_FloodFillSolid(&psurf->SurfObj, &dc->eboFill.BrushObject, &DestRect, &Pt, ConvColor, FillType);
//
//     DC_vFinishBlit(dc, NULL);
//
//     EXLATEOBJ_vCleanup(&exlo);
//
// cleanup:
//     DC_UnlockDc(dc);
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH;
}
