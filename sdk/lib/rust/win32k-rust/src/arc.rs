use win32k_sys::win32k::*;
use core::ffi::c_void;
use crate::brushbody::pbrushbody;
use crate::floatobj::*;

// KeSave/RestoreFloatingPointState: real KE functions on i386/amd64, but
// FORCEINLINE no-ops (return STATUS_SUCCESS) on arm64 (xdk/arm64/ke.h) — no
// symbol there. Provide inline no-ops on arm64 to match.
#[cfg(not(target_arch = "aarch64"))]
use ntkrnl_sys::ddk::{KeRestoreFloatingPointState, KeSaveFloatingPointState};
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeSaveFloatingPointState(_fs: *mut c_void) -> NTSTATUS { 0 }
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeRestoreFloatingPointState(_fs: *mut c_void) -> NTSTATUS { 0 }

// NT_SUCCESS(s) = s >= 0  (NTSTATUS is i32)
#[inline(always)]
fn NT_SUCCESS(status: NTSTATUS) -> bool { status >= 0 }
// ROP2_TO_MIX(Rop2) = ((Rop2) << 8) | (Rop2)  [inteng.h]; jROP2 is BYTE.
#[inline(always)]
fn ROP2_TO_MIX(rop2: u8) -> MIX { ((rop2 as MIX) << 8) | (rop2 as MIX) }

extern "C" {
    fn atan2(y: f64, x: f64) -> f64;
    fn abs(x: i32) -> i32;
    fn cos(x: f64) -> f64;
    fn sin(x: f64) -> f64;
}

// PEN_ShareLockPen is FASTCALL (pen.h).
#[cfg(target_arch = "x86")]
extern "fastcall" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }
#[cfg(not(target_arch = "x86"))]
extern "C" { fn PEN_ShareLockPen(hpen: HPEN) -> PBRUSH; }

// PEN_ShareUnlockPen / PATH_IsPathOpen are macros (pen.h / path.h) — inline.
#[inline(always)]
unsafe fn PEN_ShareUnlockPen(ppen: PBRUSH) { GDIOBJ_vDereferenceObject(ppen as POBJ); }
#[inline(always)]
unsafe fn PATH_IsPathOpen(dclevel: *mut c_void) -> i32 {
    let d = dclevel as *mut _DCLEVEL;
    ((!(*d).hPath.is_null()) && ((*d).flPath & DCPATH_ACTIVE) != 0) as i32
}

// IntLPtoDP is `static inline` (coord.h) — no symbol. Inline it and its
// callee chain (INTERNAL_LPTODP → INTERNAL_APPLY_MATRIX), using the shared
// per-arch FLOATOBJ shim.
#[inline(always)]
unsafe fn INTERNAL_APPLY_MATRIX(matrix: PMATRIX, points: LPPOINT, mut count: u32) -> u8 {
    while count != 0 {
        count -= 1;
        let p = points.offset(count as isize);
        let mut x: FLOATOBJ = core::mem::zeroed();
        let mut y: FLOATOBJ = core::mem::zeroed();
        let mut tmp: FLOATOBJ;
        // x = x * efM11 + y * efM21 + efDx
        FLOATOBJ_SetLong(core::ptr::addr_of_mut!(x), (*p).x);
        FLOATOBJ_Mul(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efM11));
        tmp = (*matrix).efM21;
        FLOATOBJ_MulLong(core::ptr::addr_of_mut!(tmp), (*p).y);
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!(tmp));
        FLOATOBJ_Add(core::ptr::addr_of_mut!(x), core::ptr::addr_of_mut!((*matrix).efDx));
        // y = x * efM12 + y * efM22 + efDy
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
unsafe fn IntLPtoDP(pdc: *mut DC, ppt: LPPOINT, count: i32) -> i32 {
    DC_vUpdateWorldToDevice(pdc);
    INTERNAL_APPLY_MATRIX(
        core::ptr::addr_of_mut!((*(*pdc).pdcattr).mxWorldToDevice) as PMATRIX,
        ppt,
        count as u32,
    ) as i32
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
// The provided C preamble contains only:
// - #define NDEBUG (preprocessor directive, not a constant)
// - Two function-like macros (PUTPIXEL, PUTLINE) which are skipped per rules

#[repr(C)]
union gxf_long {
    l: u32,
    f: f32,
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntArc(
    dc: *mut DC,
    mut Left: i32,
    mut Top: i32,
    mut Right: i32,
    mut Bottom: i32,
    XRadialStart: i32,
    YRadialStart: i32,
    XRadialEnd: i32,
    YRadialEnd: i32,
    arctype: ARCTYPE,
) -> BOOL {
    let pdcattr: PDC_ATTR;
    let mut RectBounds: RECTL = core::mem::zeroed();
    let mut RectSEpts: RECTL = core::mem::zeroed();
    let pbrPen: PBRUSH;
    let psurf: *mut SURFACE;
    let mut ret: BOOL = 1;
    let mut PenWidth: i32;
    let PenOrigWidth: i32;
    let mut AngleStart: f64;
    let mut AngleEnd: f64;
    let CenterX: i32;
    let CenterY: i32;

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

    if (Left == Right) || (Top == Bottom) {
        return 1;
    }

    if (arctype == GdiTypeChord) || (arctype == GdiTypePie) {
        if (Right - Left == 1) || (Bottom - Top == 1) {
            return 1;
        }
    }

    pdcattr = (*dc).pdcattr;

    pbrPen = PEN_ShareLockPen((*pdcattr).hpen as HPEN);
    if pbrPen.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    PenOrigWidth = (*pbrushbody(pbrPen)).lWidth;
    PenWidth = PenOrigWidth;
    if (*pbrushbody(pbrPen)).ulPenStyle == PS_NULL {
        PenWidth = 0;
    }

    if (*pbrushbody(pbrPen)).ulPenStyle == PS_INSIDEFRAME {
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
    (*pbrushbody(pbrPen)).lWidth = PenWidth;

    RectBounds.left = Left;
    RectBounds.right = Right;
    RectBounds.top = Top;
    RectBounds.bottom = Bottom;

    RectSEpts.left = XRadialStart;
    RectSEpts.top = YRadialStart;
    RectSEpts.right = XRadialEnd;
    RectSEpts.bottom = YRadialEnd;

    IntLPtoDP(dc, &mut RectBounds as *mut RECTL as LPPOINT, 2);
    IntLPtoDP(dc, &mut RectSEpts as *mut RECTL as LPPOINT, 2);

    RectBounds.left += (*dc).ptlDCOrig.x;
    RectBounds.right += (*dc).ptlDCOrig.x;
    RectBounds.top += (*dc).ptlDCOrig.y;
    RectBounds.bottom += (*dc).ptlDCOrig.y;

    RectSEpts.left += (*dc).ptlDCOrig.x;
    RectSEpts.top += (*dc).ptlDCOrig.y;
    RectSEpts.right += (*dc).ptlDCOrig.x;
    RectSEpts.bottom += (*dc).ptlDCOrig.y;

    CenterX = (RectBounds.right + RectBounds.left) / 2;
    CenterY = (RectBounds.bottom + RectBounds.top) / 2;
    AngleEnd = atan2((RectSEpts.bottom - CenterY) as f64, (RectSEpts.right - CenterX) as f64) * (360.0 / (M_PI * 2.0));
    AngleStart = atan2((RectSEpts.top - CenterY) as f64, (RectSEpts.left - CenterX) as f64) * (360.0 / (M_PI * 2.0));

    if AngleEnd == AngleStart {
        AngleStart = AngleEnd + 360.0;
    }

    if (arctype == GdiTypePie) || (arctype == GdiTypeChord) {
        ret = IntFillArc(
            dc,
            RectBounds.left,
            RectBounds.top,
            abs(RectBounds.right - RectBounds.left),
            abs(RectBounds.bottom - RectBounds.top),
            AngleStart,
            AngleEnd,
            arctype,
        );
    }

    if ret != 0 {
        ret = IntDrawArc(
            dc,
            RectBounds.left,
            RectBounds.top,
            abs(RectBounds.right - RectBounds.left),
            abs(RectBounds.bottom - RectBounds.top),
            AngleStart,
            AngleEnd,
            arctype,
            pbrPen,
        );
    }

    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        PEN_ShareUnlockPen(pbrPen);
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    if arctype == GdiTypePie {
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            CenterX,
            CenterY,
            RectSEpts.left,
            RectSEpts.top,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            RectSEpts.right,
            RectSEpts.bottom,
            CenterX,
            CenterY,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
    }
    if arctype == GdiTypeChord {
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            RectSEpts.right,
            RectSEpts.bottom,
            RectSEpts.left,
            RectSEpts.top,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
    }

    (*pbrushbody(pbrPen)).lWidth = PenOrigWidth;
    PEN_ShareUnlockPen(pbrPen);
    ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntArc(
    dc: *mut DC,
    mut Left: i32,
    mut Top: i32,
    mut Right: i32,
    mut Bottom: i32,
    XRadialStart: i32,
    YRadialStart: i32,
    XRadialEnd: i32,
    YRadialEnd: i32,
    arctype: ARCTYPE,
) -> BOOL {
    let pdcattr: PDC_ATTR;
    let mut RectBounds: RECTL = core::mem::zeroed();
    let mut RectSEpts: RECTL = core::mem::zeroed();
    let pbrPen: PBRUSH;
    let psurf: *mut SURFACE;
    let mut ret: BOOL = 1;
    let mut PenWidth: i32;
    let PenOrigWidth: i32;
    let mut AngleStart: f64;
    let mut AngleEnd: f64;
    let CenterX: i32;
    let CenterY: i32;

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

    if (Left == Right) || (Top == Bottom) {
        return 1;
    }

    if (arctype == GdiTypeChord) || (arctype == GdiTypePie) {
        if (Right - Left == 1) || (Bottom - Top == 1) {
            return 1;
        }
    }

    pdcattr = (*dc).pdcattr;

    pbrPen = PEN_ShareLockPen((*pdcattr).hpen as HPEN);
    if pbrPen.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    PenOrigWidth = (*pbrushbody(pbrPen)).lWidth;
    PenWidth = PenOrigWidth;
    if (*pbrushbody(pbrPen)).ulPenStyle == PS_NULL {
        PenWidth = 0;
    }

    if (*pbrushbody(pbrPen)).ulPenStyle == PS_INSIDEFRAME {
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
    (*pbrushbody(pbrPen)).lWidth = PenWidth;

    RectBounds.left = Left;
    RectBounds.right = Right;
    RectBounds.top = Top;
    RectBounds.bottom = Bottom;

    RectSEpts.left = XRadialStart;
    RectSEpts.top = YRadialStart;
    RectSEpts.right = XRadialEnd;
    RectSEpts.bottom = YRadialEnd;

    IntLPtoDP(dc, &mut RectBounds as *mut RECTL as LPPOINT, 2);
    IntLPtoDP(dc, &mut RectSEpts as *mut RECTL as LPPOINT, 2);

    RectBounds.left += (*dc).ptlDCOrig.x;
    RectBounds.right += (*dc).ptlDCOrig.x;
    RectBounds.top += (*dc).ptlDCOrig.y;
    RectBounds.bottom += (*dc).ptlDCOrig.y;

    RectSEpts.left += (*dc).ptlDCOrig.x;
    RectSEpts.top += (*dc).ptlDCOrig.y;
    RectSEpts.right += (*dc).ptlDCOrig.x;
    RectSEpts.bottom += (*dc).ptlDCOrig.y;

    CenterX = (RectBounds.right + RectBounds.left) / 2;
    CenterY = (RectBounds.bottom + RectBounds.top) / 2;
    AngleEnd = atan2((RectSEpts.bottom - CenterY) as f64, (RectSEpts.right - CenterX) as f64) * (360.0 / (M_PI * 2.0));
    AngleStart = atan2((RectSEpts.top - CenterY) as f64, (RectSEpts.left - CenterX) as f64) * (360.0 / (M_PI * 2.0));

    if AngleEnd == AngleStart {
        AngleStart = AngleEnd + 360.0;
    }

    if (arctype == GdiTypePie) || (arctype == GdiTypeChord) {
        ret = IntFillArc(
            dc,
            RectBounds.left,
            RectBounds.top,
            abs(RectBounds.right - RectBounds.left),
            abs(RectBounds.bottom - RectBounds.top),
            AngleStart,
            AngleEnd,
            arctype,
        );
    }

    if ret != 0 {
        ret = IntDrawArc(
            dc,
            RectBounds.left,
            RectBounds.top,
            abs(RectBounds.right - RectBounds.left),
            abs(RectBounds.bottom - RectBounds.top),
            AngleStart,
            AngleEnd,
            arctype,
            pbrPen,
        );
    }

    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        PEN_ShareUnlockPen(pbrPen);
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    if arctype == GdiTypePie {
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            CenterX,
            CenterY,
            RectSEpts.left,
            RectSEpts.top,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            RectSEpts.right,
            RectSEpts.bottom,
            CenterX,
            CenterY,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
    }
    if arctype == GdiTypeChord {
        ret = ret & IntEngLineTo(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::addr_of_mut!((*dc).eboLine.BrushObject),
            RectSEpts.right,
            RectSEpts.bottom,
            RectSEpts.left,
            RectSEpts.top,
            core::ptr::addr_of_mut!(RectBounds),
            ROP2_TO_MIX((*pdcattr).jROP2),
        );
    }

    (*pbrushbody(pbrPen)).lWidth = PenOrigWidth;
    PEN_ShareUnlockPen(pbrPen);
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiArcInternal(
    arctype: ARCTYPE,
    dc: *mut DC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
    XStartArc: i32,
    YStartArc: i32,
    XEndArc: i32,
    YEndArc: i32,
) -> i32 {
    let Ret: i32;

    if (LeftRect == RightRect) || (TopRect == BottomRect) {
        return 1;
    }

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_Arc(
            dc,
            LeftRect,
            TopRect,
            RightRect,
            BottomRect,
            XStartArc,
            YStartArc,
            XEndArc,
            YEndArc,
            0,
            arctype as i32,
        );
    }

    if arctype == GdiTypeArcTo {
        if ((*dc).dclevel.flPath & DCPATH_CLOCKWISE) != 0 {
            IntGdiLineTo(dc, XEndArc, YEndArc);
        } else {
            IntGdiLineTo(dc, XStartArc, YStartArc);
        }
    }

    Ret = IntArc(
        dc,
        LeftRect,
        TopRect,
        RightRect,
        BottomRect,
        XStartArc,
        YStartArc,
        XEndArc,
        YEndArc,
        arctype,
    );

    if arctype == GdiTypeArcTo {
        if ((*dc).dclevel.flPath & DCPATH_CLOCKWISE) != 0 {
            IntGdiMoveToEx(dc, XStartArc, YStartArc, core::ptr::null_mut());
        } else {
            IntGdiMoveToEx(dc, XEndArc, YEndArc, core::ptr::null_mut());
        }
    }

    Ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiArcInternal(
    arctype: ARCTYPE,
    dc: *mut DC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
    XStartArc: i32,
    YStartArc: i32,
    XEndArc: i32,
    YEndArc: i32,
) -> i32 {
    let Ret: i32;

    if (LeftRect == RightRect) || (TopRect == BottomRect) {
        return 1;
    }

    if PATH_IsPathOpen(core::ptr::addr_of_mut!((*dc).dclevel) as *mut c_void) != 0 {
        return PATH_Arc(
            dc,
            LeftRect,
            TopRect,
            RightRect,
            BottomRect,
            XStartArc,
            YStartArc,
            XEndArc,
            YEndArc,
            0,
            arctype as i32,
        );
    }

    if arctype == GdiTypeArcTo {
        if ((*dc).dclevel.flPath & DCPATH_CLOCKWISE) != 0 {
            IntGdiLineTo(dc, XEndArc, YEndArc);
        } else {
            IntGdiLineTo(dc, XStartArc, YStartArc);
        }
    }

    Ret = IntArc(
        dc,
        LeftRect,
        TopRect,
        RightRect,
        BottomRect,
        XStartArc,
        YStartArc,
        XEndArc,
        YEndArc,
        arctype,
    );

    if arctype == GdiTypeArcTo {
        if ((*dc).dclevel.flPath & DCPATH_CLOCKWISE) != 0 {
            IntGdiMoveToEx(dc, XStartArc, YStartArc, core::ptr::null_mut());
        } else {
            IntGdiMoveToEx(dc, XEndArc, YEndArc, core::ptr::null_mut());
        }
    }

    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiAngleArc(
    pDC: PDC,
    x: i32,
    y: i32,
    dwRadius: u32,
    eStartAngle: FLOAT,
    eSweepAngle: FLOAT,
) -> i32 {
    let x1: i32;
    let y1: i32;
    let x2: i32;
    let y2: i32;
    let arcdir: u32;
    let result: i32;

    x2 = x + (cos(((eStartAngle as f64 + eSweepAngle as f64) / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;
    y2 = y - (sin(((eStartAngle as f64 + eSweepAngle as f64) / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;

    x1 = x + (cos((eStartAngle as f64 / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;
    y1 = y - (sin((eStartAngle as f64 / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;

    arcdir = (*pDC).dclevel.flPath & DCPATH_CLOCKWISE;
    if eSweepAngle >= 0.0 {
        (*pDC).dclevel.flPath &= !DCPATH_CLOCKWISE;
    } else {
        (*pDC).dclevel.flPath |= DCPATH_CLOCKWISE;
    }

    result = IntGdiArcInternal(
        GdiTypeArcTo,
        pDC,
        x - dwRadius as i32,
        y - dwRadius as i32,
        x + dwRadius as i32,
        y + dwRadius as i32,
        x1,
        y1,
        x2,
        y2,
    );

    (*pDC).dclevel.flPath |= arcdir & DCPATH_CLOCKWISE;

    if result != 0 {
        IntGdiMoveToEx(pDC, x2, y2, core::ptr::null_mut());
    }
    result
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiAngleArc(
    pDC: PDC,
    x: i32,
    y: i32,
    dwRadius: u32,
    eStartAngle: FLOAT,
    eSweepAngle: FLOAT,
) -> i32 {
    let x1: i32;
    let y1: i32;
    let x2: i32;
    let y2: i32;
    let arcdir: u32;
    let result: i32;

    x2 = x + (cos(((eStartAngle as f64 + eSweepAngle as f64) / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;
    y2 = y - (sin(((eStartAngle as f64 + eSweepAngle as f64) / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;

    x1 = x + (cos((eStartAngle as f64 / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;
    y1 = y - (sin((eStartAngle as f64 / 360.0) * (M_PI as f64 * 2.0)) * dwRadius as f64) as i32;

    arcdir = (*pDC).dclevel.flPath & DCPATH_CLOCKWISE;
    if eSweepAngle >= 0.0 {
        (*pDC).dclevel.flPath &= !DCPATH_CLOCKWISE;
    } else {
        (*pDC).dclevel.flPath |= DCPATH_CLOCKWISE;
    }

    result = IntGdiArcInternal(
        GdiTypeArcTo,
        pDC,
        x - dwRadius as i32,
        y - dwRadius as i32,
        x + dwRadius as i32,
        y + dwRadius as i32,
        x1,
        y1,
        x2,
        y2,
    );

    (*pDC).dclevel.flPath |= arcdir & DCPATH_CLOCKWISE;

    if result != 0 {
        IntGdiMoveToEx(pDC, x2, y2, core::ptr::null_mut());
    }
    result
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiAngleArc(
    hDC: HDC,
    x: i32,
    y: i32,
    dwRadius: u32,
    dwStartAngle: u32,
    dwSweepAngle: u32,
) -> i32 {
    let pDC: *mut DC;
    let mut Ret: i32 = 0;
    let mut worker: gxf_long = gxf_long { l: 0 };
    let mut worker1: gxf_long = gxf_long { l: 0 };
    let mut FloatSave: KFLOATING_SAVE = core::mem::zeroed();
    let status: i32;

    pDC = DC_LockDc(hDC);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    status = KeSaveFloatingPointState(core::ptr::addr_of_mut!(FloatSave) as *mut _);
    if !NT_SUCCESS(status) {
        DC_UnlockDc(pDC);
        return 0;
    }

    worker.l = dwStartAngle;
    worker1.l = dwSweepAngle;
    DC_vPrepareDCsForBlit(pDC, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    if ((*(*pDC).pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY)) != 0 {
        DC_vUpdateFillBrush(pDC);
    }
    if ((*(*pDC).pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY)) != 0 {
        DC_vUpdateLineBrush(pDC);
    }
    {
        Ret = IntGdiAngleArc(pDC, x, y, dwRadius, worker.f, worker1.f);
    }
    DC_vFinishBlit(pDC, core::ptr::null_mut());
    DC_UnlockDc(pDC);

    KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave) as *mut _);

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiArcInternal(
    arctype: ARCTYPE,
    hDC: HDC,
    LeftRect: i32,
    TopRect: i32,
    RightRect: i32,
    BottomRect: i32,
    XStartArc: i32,
    YStartArc: i32,
    XEndArc: i32,
    YEndArc: i32,
) -> i32 {
    let mut dc: *mut DC;
    let Ret: i32;
    let mut FloatSave: KFLOATING_SAVE = core::mem::zeroed();
    let status: i32;

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    if arctype > GdiTypePie {
        DC_UnlockDc(dc);
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    DC_vPrepareDCsForBlit(dc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    if (*(*dc).pdcattr).ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY) != 0 {
        DC_vUpdateFillBrush(dc);
    }

    if (*(*dc).pdcattr).ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY) != 0 {
        DC_vUpdateLineBrush(dc);
    }

    status = KeSaveFloatingPointState(core::ptr::addr_of_mut!(FloatSave) as *mut _);
    if !NT_SUCCESS(status) {
        DC_UnlockDc(dc);
        return 0;
    }

    {
        Ret = IntGdiArcInternal(
            arctype,
            dc,
            LeftRect,
            TopRect,
            RightRect,
            BottomRect,
            XStartArc,
            YStartArc,
            XEndArc,
            YEndArc,
        );
    }

    KeRestoreFloatingPointState(core::ptr::addr_of_mut!(FloatSave) as *mut _);
    DC_vFinishBlit(dc, core::ptr::null_mut());
    DC_UnlockDc(dc);
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * A couple of macros to fill a single pixel or a line
//  */
// #define PUTPIXEL(x,y,BrushInst)        \
//   ret = ret && IntEngLineTo(&psurf->SurfObj, \
//        (CLIPOBJ *)&dc->co,                       \
//        &BrushInst.BrushObject,                   \
//        x, y, (x)+1, y,                           \
//        &RectBounds,                              \
//        ROP2_TO_MIX(pdcattr->jROP2));
//
// #define PUTLINE(x1,y1,x2,y2,BrushInst) \
//   ret = ret && IntEngLineTo(&psurf->SurfObj, \
//        (CLIPOBJ *)&dc->co,                       \
//        &BrushInst.BrushObject,                   \
//        x1, y1, x2, y2,                           \
//        &RectBounds,                              \
//        ROP2_TO_MIX(pdcattr->jROP2));
//
// static
// BOOL
// FASTCALL
// IntArc( DC *dc,
//         int  Left,
//         int  Top,
//         int  Right,
//         int  Bottom,
//         int  XRadialStart,
//         int  YRadialStart,
//         int  XRadialEnd,
//         int  YRadialEnd,
//         ARCTYPE arctype)
// {
//     PDC_ATTR pdcattr;
//     RECTL RectBounds, RectSEpts;
//     PBRUSH pbrPen;
//     SURFACE *psurf;
//     BOOL ret = TRUE;
//     LONG PenWidth, PenOrigWidth;
//     double AngleStart, AngleEnd;
//     LONG CenterX, CenterY;
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
//     /* Check if the target rect is empty */
//     if ((Left == Right) || (Top == Bottom)) return TRUE;
//
//     // FIXME: this needs to be verified
//     if ((arctype == GdiTypeChord ) || (arctype == GdiTypePie))
//     {
//         if ((Right - Left == 1) || (Bottom - Top == 1))
//            return TRUE;
//     }
//
//
//     pdcattr = dc->pdcattr;
//
//     pbrPen = PEN_ShareLockPen(pdcattr->hpen);
//     if (!pbrPen)
//     {
//         DPRINT1("Arc Fail 1\n");
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         return FALSE;
//     }
//
//     PenOrigWidth = PenWidth = pbrPen->lWidth;
//     if (pbrPen->ulPenStyle == PS_NULL) PenWidth = 0;
//
//     if (pbrPen->ulPenStyle == PS_INSIDEFRAME)
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
//     pbrPen->lWidth = PenWidth;
//
//     RectBounds.left   = Left;
//     RectBounds.right  = Right;
//     RectBounds.top    = Top;
//     RectBounds.bottom = Bottom;
//
//     RectSEpts.left   = XRadialStart;
//     RectSEpts.top    = YRadialStart;
//     RectSEpts.right  = XRadialEnd;
//     RectSEpts.bottom = YRadialEnd;
//
//     IntLPtoDP(dc, (LPPOINT)&RectBounds, 2);
//     IntLPtoDP(dc, (LPPOINT)&RectSEpts, 2);
//
//     RectBounds.left   += dc->ptlDCOrig.x;
//     RectBounds.right  += dc->ptlDCOrig.x;
//     RectBounds.top    += dc->ptlDCOrig.y;
//     RectBounds.bottom += dc->ptlDCOrig.y;
//
//     RectSEpts.left    += dc->ptlDCOrig.x;
//     RectSEpts.top     += dc->ptlDCOrig.y;
//     RectSEpts.right   += dc->ptlDCOrig.x;
//     RectSEpts.bottom  += dc->ptlDCOrig.y;
//
//     DPRINT("1: StartX: %d, StartY: %d, EndX: %d, EndY: %d\n",
//                RectSEpts.left,RectSEpts.top,RectSEpts.right,RectSEpts.bottom);
//
//     DPRINT("1: Left: %d, Top: %d, Right: %d, Bottom: %d\n",
//                RectBounds.left,RectBounds.top,RectBounds.right,RectBounds.bottom);
//
//     CenterX = (RectBounds.right + RectBounds.left) / 2;
//     CenterY = (RectBounds.bottom + RectBounds.top) / 2;
//     AngleEnd   = atan2((RectSEpts.bottom - CenterY), RectSEpts.right - CenterX)*(360.0/(M_PI*2));
//     AngleStart = atan2((RectSEpts.top - CenterY), RectSEpts.left - CenterX)*(360.0/(M_PI*2));
//
//     /* Edge Case: Check if the start segments overlaps(is equal) the end segment */
//     if (AngleEnd == AngleStart)
//     {
//         AngleStart = AngleEnd + 360.0; // Arc(), ArcTo(), Pie() and Chord() are counterclockwise APIs.
//     }
//
//     if ((arctype == GdiTypePie) || (arctype == GdiTypeChord))
//     {
//         ret = IntFillArc( dc,
//               RectBounds.left,
//               RectBounds.top,
//               abs(RectBounds.right-RectBounds.left), // Width
//               abs(RectBounds.bottom-RectBounds.top), // Height
//               AngleStart,
//               AngleEnd,
//               arctype);
//     }
//
//     if(ret)
//     {
//         ret = IntDrawArc( dc,
//                   RectBounds.left,
//                   RectBounds.top,
//                   abs(RectBounds.right-RectBounds.left), // Width
//                   abs(RectBounds.bottom-RectBounds.top), // Height
//                   AngleStart,
//                   AngleEnd,
//                   arctype,
//                   pbrPen);
//     }
//
//     psurf = dc->dclevel.pSurface;
//     if (NULL == psurf)
//     {
//         DPRINT1("Arc Fail 2\n");
//         PEN_ShareUnlockPen(pbrPen);
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         return FALSE;
//     }
//
//     if (arctype == GdiTypePie)
//     {
//         PUTLINE(CenterX, CenterY, RectSEpts.left, RectSEpts.top, dc->eboLine);
//         PUTLINE(RectSEpts.right, RectSEpts.bottom, CenterX, CenterY, dc->eboLine);
//     }
//     if (arctype == GdiTypeChord)
//         PUTLINE(RectSEpts.right, RectSEpts.bottom, RectSEpts.left, RectSEpts.top, dc->eboLine);
//
//     pbrPen->lWidth = PenOrigWidth;
//     PEN_ShareUnlockPen(pbrPen);
//     DPRINT("IntArc Exit.\n");
//     return ret;
// }
//
//
// BOOL FASTCALL
// IntGdiArcInternal(
//           ARCTYPE arctype,
//           DC  *dc,
//           int LeftRect,
//           int TopRect,
//           int RightRect,
//           int BottomRect,
//           int XStartArc,
//           int YStartArc,
//           int XEndArc,
//           int YEndArc)
// {
//   BOOL Ret;
//   //PDC_ATTR pdcattr;
//
//   DPRINT("StartX: %d, StartY: %d, EndX: %d, EndY: %d\n",
//            XStartArc,YStartArc,XEndArc,YEndArc);
//   DPRINT("Left: %d, Top: %d, Right: %d, Bottom: %d\n",
//            LeftRect,TopRect,RightRect,BottomRect);
//
//   if ((LeftRect == RightRect) || (TopRect == BottomRect)) return TRUE;
//
//   if (PATH_IsPathOpen(dc->dclevel))
//   {
//      return PATH_Arc( dc,
//                 LeftRect,
//                  TopRect,
//                RightRect,
//               BottomRect,
//                XStartArc,
//                YStartArc,
//                  XEndArc,
//                  YEndArc,
//                        0,
//                  arctype);
//   }
//
//   //pdcattr = dc->pdcattr;
//
//   if (arctype == GdiTypeArcTo)
//   {
//     if (dc->dclevel.flPath & DCPATH_CLOCKWISE)
//        IntGdiLineTo(dc, XEndArc, YEndArc);
//     else
//        IntGdiLineTo(dc, XStartArc, YStartArc);
//   }
//
//   Ret = IntArc( dc,
//           LeftRect,
//            TopRect,
//          RightRect,
//         BottomRect,
//          XStartArc,
//          YStartArc,
//            XEndArc,
//            YEndArc,
//            arctype);
//
//   if (arctype == GdiTypeArcTo)
//   {
//      if (dc->dclevel.flPath & DCPATH_CLOCKWISE)
//        IntGdiMoveToEx(dc, XStartArc, YStartArc, NULL);
//      else
//        IntGdiMoveToEx(dc, XEndArc, YEndArc, NULL);
//   }
//   return Ret;
// }
//
// BOOL
// FASTCALL
// IntGdiAngleArc( PDC pDC,
//                   INT x,
//                   INT y,
//          DWORD dwRadius,
//       FLOAT eStartAngle,
//       FLOAT eSweepAngle)
// {
//   INT  x1, y1, x2, y2, arcdir;
//   BOOL result;
//
//   /* Calculate the end point */
//   x2 = x + (INT)(cos(((eStartAngle+eSweepAngle)/360)*(M_PI*2)) * dwRadius);
//   y2 = y - (INT)(sin(((eStartAngle+eSweepAngle)/360)*(M_PI*2)) * dwRadius);
//
//   x1 = x + (INT)(cos((eStartAngle/360)*(M_PI*2)) * dwRadius);
//   y1 = y - (INT)(sin((eStartAngle/360)*(M_PI*2)) * dwRadius);
//
//   arcdir = pDC->dclevel.flPath & DCPATH_CLOCKWISE;
//   if (eSweepAngle >= 0)
//      pDC->dclevel.flPath &= ~DCPATH_CLOCKWISE;
//   else
//      pDC->dclevel.flPath |= DCPATH_CLOCKWISE;
//
//   result = IntGdiArcInternal( GdiTypeArcTo,
//                                        pDC,
//                                 x-dwRadius,
//                                 y-dwRadius,
//                                 x+dwRadius,
//                                 y+dwRadius,
//                                         x1,
//                                         y1,
//                                         x2,
//                                         y2 );
//
//   pDC->dclevel.flPath |= (arcdir & DCPATH_CLOCKWISE);
//
//   if (result)
//   {
//      IntGdiMoveToEx(pDC, x2, y2, NULL);
//   }
//   return result;
// }
//
// /* FUNCTIONS *****************************************************************/
//
// BOOL
// APIENTRY
// NtGdiAngleArc(
//     IN HDC hDC,
//     IN INT x,
//     IN INT y,
//     IN DWORD dwRadius,
//     IN DWORD dwStartAngle,
//     IN DWORD dwSweepAngle)
// {
//   DC *pDC;
//   BOOL Ret = FALSE;
//   gxf_long worker, worker1;
//   KFLOATING_SAVE FloatSave;
//   NTSTATUS status;
//
//   pDC = DC_LockDc (hDC);
//   if(!pDC)
//   {
//     EngSetLastError(ERROR_INVALID_HANDLE);
//     return FALSE;
//   }
//
//   status = KeSaveFloatingPointState(&FloatSave);
//   if (!NT_SUCCESS(status))
//   {
//       DC_UnlockDc( pDC );
//       return FALSE;
//   }
//
//   worker.l  = dwStartAngle;
//   worker1.l = dwSweepAngle;
//   DC_vPrepareDCsForBlit(pDC, NULL, NULL, NULL);
//   if (pDC->pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//     DC_vUpdateFillBrush(pDC);
//   if (pDC->pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//     DC_vUpdateLineBrush(pDC);
//   Ret = IntGdiAngleArc( pDC, x, y, dwRadius, worker.f, worker1.f);
//   DC_vFinishBlit(pDC, NULL);
//   DC_UnlockDc( pDC );
//
//   KeRestoreFloatingPointState(&FloatSave);
//
//   return Ret;
// }
//
// BOOL
// APIENTRY
// NtGdiArcInternal(
//         ARCTYPE arctype,
//         HDC  hDC,
//         int  LeftRect,
//         int  TopRect,
//         int  RightRect,
//         int  BottomRect,
//         int  XStartArc,
//         int  YStartArc,
//         int  XEndArc,
//         int  YEndArc)
// {
//   DC *dc;
//   BOOL Ret;
//   KFLOATING_SAVE FloatSave;
//   NTSTATUS status;
//
//   dc = DC_LockDc (hDC);
//   if(!dc)
//   {
//     EngSetLastError(ERROR_INVALID_HANDLE);
//     return FALSE;
//   }
//   if (arctype > GdiTypePie)
//   {
//     DC_UnlockDc(dc);
//     EngSetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
//   }
//
//   DC_vPrepareDCsForBlit(dc, NULL, NULL, NULL);
//
//   if (dc->pdcattr->ulDirty_ & (DIRTY_FILL | DC_BRUSH_DIRTY))
//     DC_vUpdateFillBrush(dc);
//
//   if (dc->pdcattr->ulDirty_ & (DIRTY_LINE | DC_PEN_DIRTY))
//     DC_vUpdateLineBrush(dc);
//
//   status = KeSaveFloatingPointState(&FloatSave);
//   if (!NT_SUCCESS(status))
//   {
//       DC_UnlockDc( dc );
//       return FALSE;
//   }
//
//   Ret = IntGdiArcInternal(
//                   arctype,
//                   dc,
//                   LeftRect,
//                   TopRect,
//                   RightRect,
//                   BottomRect,
//                   XStartArc,
//                   YStartArc,
//                   XEndArc,
//                   YEndArc);
//
//   KeRestoreFloatingPointState(&FloatSave);
//   DC_vFinishBlit(dc, NULL);
//   DC_UnlockDc( dc );
//   return Ret;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
