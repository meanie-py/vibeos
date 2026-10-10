use win32k_sys::win32k::*;

// Engine-side GDI DDI exports (ENGAPI = APIENTRY, stdcall on i386 / cdecl elsewhere)
// and FASTCALL helpers from win32ss/gdi/{eng,ntgdi}. Not bindgen'd — declared here
// to match the C signatures from sdk/include/psdk/winddi.h + win32ss/gdi/ntgdi/rect.h.
extern "system" {
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: *mut ULONG) -> i32;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}

// Local shadow of `DibFunctionsForBitmapFormat` — the bindgen view
// (`[DIB_FUNCTIONS; 0]` with `Option<extern fn>` fields) isn't directly
// indexable/callable. Same pattern as lineto.rs / paint.rs.
#[repr(C)]
struct DIB_FN_TABLE {
    DIB_PutPixel: unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, ULONG),
    DIB_GetPixel: unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG) -> ULONG,
    DIB_HLine:    unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, LONG, ULONG),
    DIB_VLine:    unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, LONG, ULONG),
    DIB_BitBlt:   unsafe extern "C" fn(*mut core::ffi::c_void) -> BOOLEAN,
    DIB_BitBltSrcCopy: unsafe extern "C" fn(*mut core::ffi::c_void) -> BOOLEAN,
    DIB_StretchBlt: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut SURFOBJ, *mut SURFOBJ,
        *mut RECTL, *mut RECTL, *mut POINTL, *mut BRUSHOBJ,
        *mut POINTL, *mut XLATEOBJ, u32,
    ) -> BOOLEAN,
    DIB_TransparentBlt: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut RECTL, *mut RECTL, *mut XLATEOBJ, ULONG,
    ) -> BOOLEAN,
    DIB_ColorFill: unsafe extern "C" fn(*mut SURFOBJ, *mut RECTL, ULONG) -> BOOLEAN,
    DIB_AlphaBlend: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut RECTL, *mut RECTL,
        *mut CLIPOBJ, *mut XLATEOBJ, *mut BLENDOBJ,
    ) -> BOOLEAN,
}
extern "C" {
    // The real symbol is a sized array indexed by `psoOutput->iBitmapFormat`;
    // bindgen exposes it as `[DIB_FUNCTIONS; 0]` so we re-declare with a
    // generous fixed size here purely so the indexing arithmetic resolves.
    static DibFunctionsForBitmapFormat: [DIB_FN_TABLE; 11];
}

#[inline]
unsafe fn rgb_pack(r: i32, g: i32, b: i32) -> ULONG {
    // `RGB(r,g,b)` from wingdi.h:3386:
    //   ((COLORREF)(((BYTE)(r) | ((WORD)((BYTE)(g))<<8)) | (((DWORD)(BYTE)(b))<<16)))
    ((r as u8 as ULONG)
        | ((g as u8 as ULONG) << 8)
        | ((b as u8 as ULONG) << 16)) as ULONG
}

// Bindgen exposes `DRIVER_FUNCTIONS.GradientFill` as
// `Option<unsafe extern "<cc>" fn(...) -> BOOL>` — `stdcall` on i386,
// `C` on amd64/arm64 (matches the driver DDI ABI). The C macro just
// dereferences the slot.
type FnDrvGradientFill = unsafe extern "system" fn(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: ULONG,
    pMesh: PVOID,
    nMesh: ULONG,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    ulMode: ULONG,
) -> BOOL;

#[inline]
unsafe fn gdidev_gradientfill(pso: *mut SURFOBJ) -> FnDrvGradientFill {
    // Inline expansion of `GDIDEVFUNCS(SurfObj).GradientFill` from
    // win32ss/gdi/eng/surface.h:106 — matches alphablend.rs::gdidev_alphablend.
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    (*pdev).__bindgen_anon_1.DriverFunctions.GradientFill
        .expect("GradientFill driver slot null with HOOK_GRADIENTFILL set")
}

// ---- C-macro translations (inline pre-substituted at use sites via macro_rules!) ----
//
// The C source uses preprocessor macros that reach out to caller-scope
// locals (`c`, `dc`, `ec`, `ic`, `x`, `dx`, `dy`, `ex`, `incx`, `destx`,
// `sx`, `gc`, `gd`, `ge`, `gi`, `gx`, `gxi`, `g`, `g_end`, `sy`, `y`,
// `InY`, `FillRect`, `psoOutput`, `pxlo`, `Color`, `pptlDitherOrg`).
// We mirror that with macro_rules! that capture the same identifiers
// hygienically by passing expressions in.
//
// Source: win32ss/gdi/eng/gradient.c lines 14-311.
macro_rules! SMALLER {
    ($a:expr, $b:expr) => {
        ((*$a).y < (*$b).y) || ((*$a).y == (*$b).y && (*$a).x < (*$b).x)
    };
}
macro_rules! SWAP {
    ($a:ident, $b:ident, $c:ident) => {
        $c = $a;
        $a = $b;
        $b = $c;
    };
}
macro_rules! VCMPCLR {
    ($a:expr, $b:expr, $c:expr, $color:ident) => {
        (*$a).$color == (*$b).$color && (*$a).$color == (*$c).$color
    };
}
macro_rules! VCMPCLRS {
    ($a:expr, $b:expr, $c:expr) => {
        !(VCMPCLR!($a, $b, $c, Red) && VCMPCLR!($a, $b, $c, Green) && VCMPCLR!($a, $b, $c, Blue))
    };
}

// The IntEngGradientFillTriangle line-walk macros (INITLINE, INITCOL, DOLINE,
// STEPCOL, IFINY, FILLLINE, FINITCOL, FDOCOL, S_INITLINE, S_DOSTEP, S_FILLLINE)
// reach out to ~20 outer-scope locals. macro_rules! at module level is
// hygienic w.r.t. local variables (those don't resolve), so we define them
// INSIDE each IntEngGradientFillTriangle copy below. See the C source
// `win32ss/gdi/eng/gradient.c` lines 14-311 for the original `#define`s.

// ---- file-scope preamble (translated) ----
pub const LINC: [i32; 2] = [-1, 1];

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntEngGradientFillRect(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: u32,
    gRect: PGRADIENT_RECT,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    Horizontal: BOOL,
) -> BOOL {
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let v1: *mut TRIVERTEX = pVertex.offset((*gRect).UpperLeft as isize);
    let v2: *mut TRIVERTEX = pVertex.offset((*gRect).LowerRight as isize);
    let mut rcGradient: RECTL = core::mem::zeroed();
    let mut rcSG: RECTL;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: BOOL;
    let mut i: u32;
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeave: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut y: i32;
    let dy: i32;
    let mut c: [i32; 3] = [0; 3];
    let mut dc: [i32; 3] = [0; 3];
    let mut ec: [i32; 3] = [0; 3];
    let mut ic: [i32; 3] = [0; 3];

    rcGradient.left = core::cmp::min((*v1).x, (*v2).x);
    rcGradient.right = core::cmp::max((*v1).x, (*v2).x);
    rcGradient.top = core::cmp::min((*v1).y, (*v2).y);
    rcGradient.bottom = core::cmp::max((*v1).y, (*v2).y);
    rcSG = rcGradient;

    if Horizontal != 0 {
        dy = (rcGradient.right - rcGradient.left).abs();
    } else {
        dy = (rcGradient.bottom - rcGradient.top).abs();
    }

    if IntEngEnter(&mut EnterLeave, psoDest, &mut rcSG, 0 as BOOL, &mut Translate, &mut psoOutput) == 0 {
        return 0;
    }

    if ((*v1).Red != (*v2).Red || (*v1).Green != (*v2).Green || (*v1).Blue != (*v2).Blue) && dy > 1 {
        CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        loop {
            let mut FillRect: RECTL = core::mem::zeroed();
            let mut Color: u32;

            if Horizontal != 0 {
                EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
                i = 0;
                while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
                    if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                        // HVINITCOL(Red, 0)/(Green, 1)/(Blue, 2) — TRIVERTEX
                        // color fields are COLOR16 (u16); promote to i32 before
                        // signed subtraction to match the C `LONG` arithmetic.
                        c[0] = ((*v1).Red >> 8) as i32;
                        dc[0] = (((*v2).Red >> 8) as i32 - c[0]).abs();
                        ec[0] = -(dy >> 1);
                        ic[0] = LINC[(((*v2).Red >> 8) as i32 > c[0]) as usize];
                        c[1] = ((*v1).Green >> 8) as i32;
                        dc[1] = (((*v2).Green >> 8) as i32 - c[1]).abs();
                        ec[1] = -(dy >> 1);
                        ic[1] = LINC[(((*v2).Green >> 8) as i32 > c[1]) as usize];
                        c[2] = ((*v1).Blue >> 8) as i32;
                        dc[2] = (((*v2).Blue >> 8) as i32 - c[2]).abs();
                        ec[2] = -(dy >> 1);
                        ic[2] = LINC[(((*v2).Blue >> 8) as i32 > c[2]) as usize];

                        y = rcSG.left;
                        while y < FillRect.right {
                            if y >= FillRect.left {
                                Color = XLATEOBJ_iXlate(pxlo, rgb_pack(c[0], c[1], c[2]));
                                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_VLine)(
                                    psoOutput, y + Translate.x, FillRect.top + Translate.y, FillRect.bottom + Translate.y, Color);
                            }
                            ec[0] += dc[0];
                            while ec[0] > 0 {
                                c[0] += ic[0];
                                ec[0] -= dy;
                            }
                            ec[1] += dc[1];
                            while ec[1] > 0 {
                                c[1] += ic[1];
                                ec[1] -= dy;
                            }
                            ec[2] += dc[2];
                            while ec[2] > 0 {
                                c[2] += ic[2];
                                ec[2] -= dy;
                            }
                            y += 1;
                        }
                    }
                    i += 1;
                }

                if EnumMore == 0 {
                    break;
                }
                continue;
            }

            EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
            i = 0;
            while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
                if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                    // HVINITCOL(Red, 0)/(Green, 1)/(Blue, 2) — vertical
                    c[0] = ((*v1).Red >> 8) as i32;
                    dc[0] = (((*v2).Red >> 8) as i32 - c[0]).abs();
                    ec[0] = -(dy >> 1);
                    ic[0] = LINC[(((*v2).Red >> 8) as i32 > c[0]) as usize];
                    c[1] = ((*v1).Green >> 8) as i32;
                    dc[1] = (((*v2).Green >> 8) as i32 - c[1]).abs();
                    ec[1] = -(dy >> 1);
                    ic[1] = LINC[(((*v2).Green >> 8) as i32 > c[1]) as usize];
                    c[2] = ((*v1).Blue >> 8) as i32;
                    dc[2] = (((*v2).Blue >> 8) as i32 - c[2]).abs();
                    ec[2] = -(dy >> 1);
                    ic[2] = LINC[(((*v2).Blue >> 8) as i32 > c[2]) as usize];

                    y = rcSG.top;
                    while y < FillRect.bottom {
                        if y >= FillRect.top {
                            Color = XLATEOBJ_iXlate(pxlo, rgb_pack(c[0], c[1], c[2]));
                            (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(psoOutput,
                                                                                        FillRect.left + Translate.x,
                                                                                        FillRect.right + Translate.x,
                                                                                        y + Translate.y,
                                                                                        Color);
                        }
                        ec[0] += dc[0];
                        while ec[0] > 0 {
                            c[0] += ic[0];
                            ec[0] -= dy;
                        }
                        ec[1] += dc[1];
                        while ec[1] > 0 {
                            c[1] += ic[1];
                            ec[1] -= dy;
                        }
                        ec[2] += dc[2];
                        while ec[2] > 0 {
                            c[2] += ic[2];
                            ec[2] -= dy;
                        }
                        y += 1;
                    }
                }
                i += 1;
            }

            if EnumMore == 0 {
                break;
            }
        }

        return IntEngLeave(&mut EnterLeave);
    }

    CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
    loop {
        let mut FillRect: RECTL = core::mem::zeroed();
        let Color: ULONG = XLATEOBJ_iXlate(pxlo, rgb_pack(((*v1).Red >> 8) as i32, ((*v1).Green >> 8) as i32, ((*v1).Blue >> 8) as i32));

        EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
        i = 0;
        while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
            if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                while FillRect.top < FillRect.bottom {
                    (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(psoOutput,
                                                                                    FillRect.left + Translate.x,
                                                                                    FillRect.right + Translate.x,
                                                                                    FillRect.top + Translate.y,
                                                                                    Color);
                    FillRect.top += 1;
                }
            }
            i += 1;
        }

        if EnumMore == 0 {
            break;
        }
    }

    IntEngLeave(&mut EnterLeave)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntEngGradientFillRect(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: u32,
    gRect: PGRADIENT_RECT,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    Horizontal: BOOL,
) -> BOOL {
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let v1: *mut TRIVERTEX = pVertex.offset((*gRect).UpperLeft as isize);
    let v2: *mut TRIVERTEX = pVertex.offset((*gRect).LowerRight as isize);
    let mut rcGradient: RECTL = core::mem::zeroed();
    let mut rcSG: RECTL;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: BOOL;
    let mut i: u32;
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeave: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut y: i32;
    let dy: i32;
    let mut c: [i32; 3] = [0; 3];
    let mut dc: [i32; 3] = [0; 3];
    let mut ec: [i32; 3] = [0; 3];
    let mut ic: [i32; 3] = [0; 3];

    rcGradient.left = core::cmp::min((*v1).x, (*v2).x);
    rcGradient.right = core::cmp::max((*v1).x, (*v2).x);
    rcGradient.top = core::cmp::min((*v1).y, (*v2).y);
    rcGradient.bottom = core::cmp::max((*v1).y, (*v2).y);
    rcSG = rcGradient;

    if Horizontal != 0 {
        dy = (rcGradient.right - rcGradient.left).abs();
    } else {
        dy = (rcGradient.bottom - rcGradient.top).abs();
    }

    if IntEngEnter(&mut EnterLeave, psoDest, &mut rcSG, 0 as BOOL, &mut Translate, &mut psoOutput) == 0 {
        return 0;
    }

    if ((*v1).Red != (*v2).Red || (*v1).Green != (*v2).Green || (*v1).Blue != (*v2).Blue) && dy > 1 {
        CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        loop {
            let mut FillRect: RECTL = core::mem::zeroed();
            let mut Color: u32;

            if Horizontal != 0 {
                EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
                i = 0;
                while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
                    if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                        // HVINITCOL(Red, 0)/(Green, 1)/(Blue, 2) — TRIVERTEX
                        // color fields are COLOR16 (u16); promote to i32 before
                        // signed subtraction to match the C `LONG` arithmetic.
                        c[0] = ((*v1).Red >> 8) as i32;
                        dc[0] = (((*v2).Red >> 8) as i32 - c[0]).abs();
                        ec[0] = -(dy >> 1);
                        ic[0] = LINC[(((*v2).Red >> 8) as i32 > c[0]) as usize];
                        c[1] = ((*v1).Green >> 8) as i32;
                        dc[1] = (((*v2).Green >> 8) as i32 - c[1]).abs();
                        ec[1] = -(dy >> 1);
                        ic[1] = LINC[(((*v2).Green >> 8) as i32 > c[1]) as usize];
                        c[2] = ((*v1).Blue >> 8) as i32;
                        dc[2] = (((*v2).Blue >> 8) as i32 - c[2]).abs();
                        ec[2] = -(dy >> 1);
                        ic[2] = LINC[(((*v2).Blue >> 8) as i32 > c[2]) as usize];

                        y = rcSG.left;
                        while y < FillRect.right {
                            if y >= FillRect.left {
                                Color = XLATEOBJ_iXlate(pxlo, rgb_pack(c[0], c[1], c[2]));
                                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_VLine)(
                                    psoOutput, y + Translate.x, FillRect.top + Translate.y, FillRect.bottom + Translate.y, Color);
                            }
                            ec[0] += dc[0];
                            while ec[0] > 0 {
                                c[0] += ic[0];
                                ec[0] -= dy;
                            }
                            ec[1] += dc[1];
                            while ec[1] > 0 {
                                c[1] += ic[1];
                                ec[1] -= dy;
                            }
                            ec[2] += dc[2];
                            while ec[2] > 0 {
                                c[2] += ic[2];
                                ec[2] -= dy;
                            }
                            y += 1;
                        }
                    }
                    i += 1;
                }

                if EnumMore == 0 {
                    break;
                }
                continue;
            }

            EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
            i = 0;
            while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
                if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                    // HVINITCOL(Red, 0)/(Green, 1)/(Blue, 2) — vertical
                    c[0] = ((*v1).Red >> 8) as i32;
                    dc[0] = (((*v2).Red >> 8) as i32 - c[0]).abs();
                    ec[0] = -(dy >> 1);
                    ic[0] = LINC[(((*v2).Red >> 8) as i32 > c[0]) as usize];
                    c[1] = ((*v1).Green >> 8) as i32;
                    dc[1] = (((*v2).Green >> 8) as i32 - c[1]).abs();
                    ec[1] = -(dy >> 1);
                    ic[1] = LINC[(((*v2).Green >> 8) as i32 > c[1]) as usize];
                    c[2] = ((*v1).Blue >> 8) as i32;
                    dc[2] = (((*v2).Blue >> 8) as i32 - c[2]).abs();
                    ec[2] = -(dy >> 1);
                    ic[2] = LINC[(((*v2).Blue >> 8) as i32 > c[2]) as usize];

                    y = rcSG.top;
                    while y < FillRect.bottom {
                        if y >= FillRect.top {
                            Color = XLATEOBJ_iXlate(pxlo, rgb_pack(c[0], c[1], c[2]));
                            (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(psoOutput,
                                                                                        FillRect.left + Translate.x,
                                                                                        FillRect.right + Translate.x,
                                                                                        y + Translate.y,
                                                                                        Color);
                        }
                        ec[0] += dc[0];
                        while ec[0] > 0 {
                            c[0] += ic[0];
                            ec[0] -= dy;
                        }
                        ec[1] += dc[1];
                        while ec[1] > 0 {
                            c[1] += ic[1];
                            ec[1] -= dy;
                        }
                        ec[2] += dc[2];
                        while ec[2] > 0 {
                            c[2] += ic[2];
                            ec[2] -= dy;
                        }
                        y += 1;
                    }
                }
                i += 1;
            }

            if EnumMore == 0 {
                break;
            }
        }

        return IntEngLeave(&mut EnterLeave);
    }

    CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
    loop {
        let mut FillRect: RECTL = core::mem::zeroed();
        let Color: ULONG = XLATEOBJ_iXlate(pxlo, rgb_pack(((*v1).Red >> 8) as i32, ((*v1).Green >> 8) as i32, ((*v1).Blue >> 8) as i32));

        EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, &mut RectEnum as *mut _ as *mut ULONG) as i32;
        i = 0;
        while i < RectEnum.c && RectEnum.arcl[i as usize].top <= rcSG.bottom {
            if RECTL_bIntersectRect(&mut FillRect, &RectEnum.arcl[i as usize], &rcSG) != 0 {
                while FillRect.top < FillRect.bottom {
                    (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(psoOutput,
                                                                                    FillRect.left + Translate.x,
                                                                                    FillRect.right + Translate.x,
                                                                                    FillRect.top + Translate.y,
                                                                                    Color);
                    FillRect.top += 1;
                }
            }
            i += 1;
        }

        if EnumMore == 0 {
            break;
        }
    }

    IntEngLeave(&mut EnterLeave)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntEngGradientFillTriangle(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: PTRIVERTEX,
    nVertex: u32,
    gTriangle: PGRADIENT_TRIANGLE,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
) -> BOOL {
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let mut v1: PTRIVERTEX;
    let mut v2: PTRIVERTEX;
    let mut v3: PTRIVERTEX;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: BOOL;
    let mut i: u32;
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeave: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut FillRect: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Color: u32;

    let mut sx: [i32; 3] = [0; 3];
    let mut x: [i32; 3] = [0; 3];
    let mut dx: [i32; 3] = [0; 3];
    let mut dy: [i32; 3] = [0; 3];
    let mut incx: [i32; 3] = [0; 3];
    let mut ex: [i32; 3] = [0; 3];
    let mut destx: [i32; 3] = [0; 3];
    let mut c: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut dc: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut ec: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut ic: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut g: i32;
    let mut gx: i32;
    let mut gxi: i32;
    let mut gc: [i32; 3] = [0; 3];
    let mut gd: [i32; 3] = [0; 3];
    let mut ge: [i32; 3] = [0; 3];
    let mut gi: [i32; 3] = [0; 3];
    let mut sy: i32 = 0;
    let mut y: i32 = 0;
    let mut bt: i32;
    let mut g_end: i32 = 0;
    let mut InY: bool = false;

    // --- IntEngGradientFillTriangle gradient/solid line-walk macros ---
    // These rely on outer-scope locals (`c`, `dc`, `ec`, `ic`, `x`, `dx`,
    // `dy`, `ex`, `incx`, `destx`, `sx`, `gc`, `gd`, `ge`, `gi`, `gx`,
    // `gxi`, `g`, `g_end`, `sy`, `InY`, `FillRect`, `pxlo`, `psoOutput`,
    // `Color`, `pptlDitherOrg`, `y`) so they're defined here, inside the
    // function. Names mirror the C `#define`s in gradient.c lines 14-311.
    //
    // The C `GOLINE(a,b,line)`/`ENDLINE(a,b,line)` pair opens/closes an
    // `if (y >= a->y && y <= b->y) {` block; Rust macros can't open a
    // brace without matching it in the same expansion, so the pair is
    // collapsed into `IFINY!(a, b; <body>)`.
    macro_rules! INITLINE {
        ($a:expr, $b:expr, $line:expr) => {
            x[$line] = (*$a).x;
            sx[$line] = (*$a).x + (*pptlDitherOrg).x - 1;
            dx[$line] = ((*$b).x - (*$a).x).abs();
            dy[$line] = ((*$b).y - (*$a).y).abs();
            incx[$line] = LINC[((*$b).x > (*$a).x) as usize];
            ex[$line] = -(dy[$line] >> 1);
            destx[$line] = (*$b).x;
        };
    }
    macro_rules! S_INITLINE {
        ($a:expr, $b:expr, $line:expr) => {
            x[$line] = (*$a).x;
            sx[$line] = (*$a).x + (*pptlDitherOrg).x;
            dx[$line] = ((*$b).x - (*$a).x).abs();
            dy[$line] = ((*$b).y - (*$a).y).abs();
            incx[$line] = LINC[((*$b).x > (*$a).x) as usize];
            ex[$line] = -(dy[$line] >> 1);
            destx[$line] = (*$b).x;
        };
    }
    macro_rules! INITCOL {
        ($a:expr, $b:expr, $line:expr, $col:ident, $id:expr) => {
            c[$line][$id] = ((*$a).$col >> 8) as i32;
            dc[$line][$id] = (((*$b).$col >> 8) as i32 - c[$line][$id]).abs();
            ec[$line][$id] = -(dy[$line] >> 1);
            ic[$line][$id] = LINC[(((*$b).$col >> 8) as i32 > c[$line][$id]) as usize];
        };
    }
    macro_rules! DOINIT {
        ($a:expr, $b:expr, $line:expr) => {
            INITLINE!($a, $b, $line);
            INITCOL!($a, $b, $line, Red, 0);
            INITCOL!($a, $b, $line, Green, 1);
            INITCOL!($a, $b, $line, Blue, 2);
        };
    }
    macro_rules! STEPCOL {
        ($line:expr, $id:expr) => {
            ec[$line][$id] += dc[$line][$id];
            if dy[$line] != 0 {
                while ec[$line][$id] > 0 {
                    c[$line][$id] += ic[$line][$id];
                    ec[$line][$id] -= dy[$line];
                }
            }
        };
    }
    macro_rules! S_DOSTEP {
        ($line:expr) => {
            ex[$line] += dx[$line];
            while ex[$line] > 0 && x[$line] != destx[$line] {
                x[$line] += incx[$line];
                sx[$line] += incx[$line];
                ex[$line] -= dy[$line];
            }
        };
    }
    macro_rules! DOLINE {
        ($line:expr) => {
            STEPCOL!($line, 0);
            STEPCOL!($line, 1);
            STEPCOL!($line, 2);
            S_DOSTEP!($line);
        };
    }
    macro_rules! IFINY {
        ($a:expr, $b:expr; $($body:tt)*) => {
            if y >= (*$a).y && y <= (*$b).y {
                $($body)*
            }
        };
    }
    macro_rules! FINITCOL {
        ($linefrom:expr, $lineto:expr, $colid:expr) => {
            gc[$colid] = c[$linefrom][$colid];
            gd[$colid] = (c[$lineto][$colid] - gc[$colid]).abs();
            ge[$colid] = -(gx >> 1);
            gi[$colid] = LINC[(c[$lineto][$colid] > gc[$colid]) as usize];
        };
    }
    macro_rules! FDOCOL {
        ($colid:expr) => {
            ge[$colid] += gd[$colid];
            if gx != 0 {
                while ge[$colid] > 0 {
                    gc[$colid] += gi[$colid];
                    ge[$colid] -= gx;
                }
            }
        };
    }
    macro_rules! FILLLINE {
        ($linefrom:expr, $lineto:expr) => {
            gx = (sx[$lineto] - sx[$linefrom]).abs();
            gxi = LINC[(sx[$linefrom] < sx[$lineto]) as usize];
            FINITCOL!($linefrom, $lineto, 0);
            FINITCOL!($linefrom, $lineto, 1);
            FINITCOL!($linefrom, $lineto, 2);
            g_end = sx[$lineto] + gxi;
            g = sx[$linefrom];
            while g != g_end {
                if InY && g >= FillRect.left && g < FillRect.right {
                    Color = XLATEOBJ_iXlate(pxlo, rgb_pack(gc[0], gc[1], gc[2]));
                    (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_PutPixel)(
                        psoOutput, g, sy, Color,
                    );
                }
                FDOCOL!(0);
                FDOCOL!(1);
                FDOCOL!(2);
                g += gxi;
            }
        };
    }
    macro_rules! S_FILLLINE {
        ($linefrom:expr, $lineto:expr) => {
            if sx[$lineto] < sx[$linefrom] {
                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(
                    psoOutput,
                    core::cmp::max(sx[$lineto], FillRect.left),
                    core::cmp::min(sx[$linefrom], FillRect.right),
                    sy,
                    Color,
                );
            } else {
                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(
                    psoOutput,
                    core::cmp::max(sx[$linefrom], FillRect.left),
                    core::cmp::min(sx[$lineto], FillRect.right),
                    sy,
                    Color,
                );
            }
        };
    }

    v1 = pVertex.offset((*gTriangle).Vertex1 as isize);
    v2 = pVertex.offset((*gTriangle).Vertex2 as isize);
    v3 = pVertex.offset((*gTriangle).Vertex3 as isize);

    if SMALLER!(v2, v1) {
        let mut t: PTRIVERTEX;
        SWAP!(v1, v2, t);
    }

    if SMALLER!(v3, v2) {
        let mut t: PTRIVERTEX;
        SWAP!(v2, v3, t);
        if SMALLER!(v2, v1) {
            SWAP!(v1, v2, t);
        }
    }

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeave),
        psoDest,
        core::ptr::addr_of_mut!(FillRect),
        0 as BOOL,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(psoOutput),
    ) == 0 {
        return 0;
    }

    if VCMPCLRS!(v1, v2, v3) {
        CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        loop {
            EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(RectEnum) as *mut ULONG) as i32;
            i = 0;
            while i < RectEnum.c && (*RectEnum.arcl.as_ptr().offset(i as isize)).top <= (*prclExtents).bottom {
                if RECTL_bIntersectRect(core::ptr::addr_of_mut!(FillRect), RectEnum.arcl.as_ptr().offset(i as isize) as *mut RECTL, prclExtents) != 0 {
                    DOINIT!(v1, v3, 0);
                    DOINIT!(v1, v2, 1);
                    DOINIT!(v2, v3, 2);

                    y = (*v1).y;
                    sy = (*v1).y + (*pptlDitherOrg).y;
                    bt = core::cmp::min((*v3).y + (*pptlDitherOrg).y, FillRect.bottom);

                    while sy < bt {
                        InY = !(sy < FillRect.top || sy >= FillRect.bottom);
                        IFINY!(v1, v3; DOLINE!(0););

                        IFINY!(v1, v2;
                            DOLINE!(1);
                            FILLLINE!(0, 1);
                        );

                        // NOTE: the C source has `ENDLINE(23, v3, 2)` here — `23`
                        // is a stray identifier that ENDLINE discards; it doesn't
                        // change the closing `}` semantics. We preserve the same
                        // outer `if (y >= v2->y && y <= v3->y)` bracket using IFINY!(v2, v3; ...).
                        IFINY!(v2, v3;
                            FILLLINE!(0, 2);
                            DOLINE!(2);
                            FILLLINE!(0, 2);
                        );

                        y += 1;
                        sy += 1;
                    }
                }
                i += 1;
            }
            if EnumMore == 0 {
                break;
            }
        }

        return IntEngLeave(core::ptr::addr_of_mut!(EnterLeave));
    }

    Color = XLATEOBJ_iXlate(pxlo, rgb_pack(((*v1).Red >> 8) as i32, ((*v1).Green >> 8) as i32, ((*v1).Blue >> 8) as i32));
    CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
    loop {
        EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(RectEnum) as *mut ULONG) as i32;
        i = 0;
        while i < RectEnum.c && (*RectEnum.arcl.as_ptr().offset(i as isize)).top <= (*prclExtents).bottom {
            if RECTL_bIntersectRect(core::ptr::addr_of_mut!(FillRect), RectEnum.arcl.as_ptr().offset(i as isize) as *mut RECTL, prclExtents) != 0 {
                S_INITLINE!(v1, v3, 0);
                S_INITLINE!(v1, v2, 1);
                S_INITLINE!(v2, v3, 2);

                y = (*v1).y;
                sy = (*v1).y + (*pptlDitherOrg).y;
                bt = core::cmp::min((*v3).y + (*pptlDitherOrg).y, FillRect.bottom);

                while sy < bt {
                    IFINY!(v1, v3; S_DOSTEP!(0););

                    IFINY!(v1, v2;
                        S_DOSTEP!(1);
                        S_FILLLINE!(0, 1);
                    );

                    // ENDLINE(23, v3, 2) — `23` discarded (see gradient.c:405/450).
                    IFINY!(v2, v3;
                        S_DOSTEP!(2);
                        S_FILLLINE!(0, 2);
                    );

                    y += 1;
                    sy += 1;
                }
            }
            i += 1;
        }
        if EnumMore == 0 {
            break;
        }
    }

    IntEngLeave(core::ptr::addr_of_mut!(EnterLeave))
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntEngGradientFillTriangle(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: PTRIVERTEX,
    nVertex: u32,
    gTriangle: PGRADIENT_TRIANGLE,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
) -> BOOL {
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let mut v1: PTRIVERTEX;
    let mut v2: PTRIVERTEX;
    let mut v3: PTRIVERTEX;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: BOOL;
    let mut i: u32;
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeave: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut FillRect: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Color: u32;

    let mut sx: [i32; 3] = [0; 3];
    let mut x: [i32; 3] = [0; 3];
    let mut dx: [i32; 3] = [0; 3];
    let mut dy: [i32; 3] = [0; 3];
    let mut incx: [i32; 3] = [0; 3];
    let mut ex: [i32; 3] = [0; 3];
    let mut destx: [i32; 3] = [0; 3];
    let mut c: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut dc: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut ec: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut ic: [[i32; 3]; 3] = [[0; 3]; 3];
    let mut g: i32;
    let mut gx: i32;
    let mut gxi: i32;
    let mut gc: [i32; 3] = [0; 3];
    let mut gd: [i32; 3] = [0; 3];
    let mut ge: [i32; 3] = [0; 3];
    let mut gi: [i32; 3] = [0; 3];
    let mut sy: i32 = 0;
    let mut y: i32 = 0;
    let mut bt: i32;
    let mut g_end: i32 = 0;
    let mut InY: bool = false;

    // --- IntEngGradientFillTriangle gradient/solid line-walk macros ---
    // These rely on outer-scope locals (`c`, `dc`, `ec`, `ic`, `x`, `dx`,
    // `dy`, `ex`, `incx`, `destx`, `sx`, `gc`, `gd`, `ge`, `gi`, `gx`,
    // `gxi`, `g`, `g_end`, `sy`, `InY`, `FillRect`, `pxlo`, `psoOutput`,
    // `Color`, `pptlDitherOrg`, `y`) so they're defined here, inside the
    // function. Names mirror the C `#define`s in gradient.c lines 14-311.
    //
    // The C `GOLINE(a,b,line)`/`ENDLINE(a,b,line)` pair opens/closes an
    // `if (y >= a->y && y <= b->y) {` block; Rust macros can't open a
    // brace without matching it in the same expansion, so the pair is
    // collapsed into `IFINY!(a, b; <body>)`.
    macro_rules! INITLINE {
        ($a:expr, $b:expr, $line:expr) => {
            x[$line] = (*$a).x;
            sx[$line] = (*$a).x + (*pptlDitherOrg).x - 1;
            dx[$line] = ((*$b).x - (*$a).x).abs();
            dy[$line] = ((*$b).y - (*$a).y).abs();
            incx[$line] = LINC[((*$b).x > (*$a).x) as usize];
            ex[$line] = -(dy[$line] >> 1);
            destx[$line] = (*$b).x;
        };
    }
    macro_rules! S_INITLINE {
        ($a:expr, $b:expr, $line:expr) => {
            x[$line] = (*$a).x;
            sx[$line] = (*$a).x + (*pptlDitherOrg).x;
            dx[$line] = ((*$b).x - (*$a).x).abs();
            dy[$line] = ((*$b).y - (*$a).y).abs();
            incx[$line] = LINC[((*$b).x > (*$a).x) as usize];
            ex[$line] = -(dy[$line] >> 1);
            destx[$line] = (*$b).x;
        };
    }
    macro_rules! INITCOL {
        ($a:expr, $b:expr, $line:expr, $col:ident, $id:expr) => {
            c[$line][$id] = ((*$a).$col >> 8) as i32;
            dc[$line][$id] = (((*$b).$col >> 8) as i32 - c[$line][$id]).abs();
            ec[$line][$id] = -(dy[$line] >> 1);
            ic[$line][$id] = LINC[(((*$b).$col >> 8) as i32 > c[$line][$id]) as usize];
        };
    }
    macro_rules! DOINIT {
        ($a:expr, $b:expr, $line:expr) => {
            INITLINE!($a, $b, $line);
            INITCOL!($a, $b, $line, Red, 0);
            INITCOL!($a, $b, $line, Green, 1);
            INITCOL!($a, $b, $line, Blue, 2);
        };
    }
    macro_rules! STEPCOL {
        ($line:expr, $id:expr) => {
            ec[$line][$id] += dc[$line][$id];
            if dy[$line] != 0 {
                while ec[$line][$id] > 0 {
                    c[$line][$id] += ic[$line][$id];
                    ec[$line][$id] -= dy[$line];
                }
            }
        };
    }
    macro_rules! S_DOSTEP {
        ($line:expr) => {
            ex[$line] += dx[$line];
            while ex[$line] > 0 && x[$line] != destx[$line] {
                x[$line] += incx[$line];
                sx[$line] += incx[$line];
                ex[$line] -= dy[$line];
            }
        };
    }
    macro_rules! DOLINE {
        ($line:expr) => {
            STEPCOL!($line, 0);
            STEPCOL!($line, 1);
            STEPCOL!($line, 2);
            S_DOSTEP!($line);
        };
    }
    macro_rules! IFINY {
        ($a:expr, $b:expr; $($body:tt)*) => {
            if y >= (*$a).y && y <= (*$b).y {
                $($body)*
            }
        };
    }
    macro_rules! FINITCOL {
        ($linefrom:expr, $lineto:expr, $colid:expr) => {
            gc[$colid] = c[$linefrom][$colid];
            gd[$colid] = (c[$lineto][$colid] - gc[$colid]).abs();
            ge[$colid] = -(gx >> 1);
            gi[$colid] = LINC[(c[$lineto][$colid] > gc[$colid]) as usize];
        };
    }
    macro_rules! FDOCOL {
        ($colid:expr) => {
            ge[$colid] += gd[$colid];
            if gx != 0 {
                while ge[$colid] > 0 {
                    gc[$colid] += gi[$colid];
                    ge[$colid] -= gx;
                }
            }
        };
    }
    macro_rules! FILLLINE {
        ($linefrom:expr, $lineto:expr) => {
            gx = (sx[$lineto] - sx[$linefrom]).abs();
            gxi = LINC[(sx[$linefrom] < sx[$lineto]) as usize];
            FINITCOL!($linefrom, $lineto, 0);
            FINITCOL!($linefrom, $lineto, 1);
            FINITCOL!($linefrom, $lineto, 2);
            g_end = sx[$lineto] + gxi;
            g = sx[$linefrom];
            while g != g_end {
                if InY && g >= FillRect.left && g < FillRect.right {
                    Color = XLATEOBJ_iXlate(pxlo, rgb_pack(gc[0], gc[1], gc[2]));
                    (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_PutPixel)(
                        psoOutput, g, sy, Color,
                    );
                }
                FDOCOL!(0);
                FDOCOL!(1);
                FDOCOL!(2);
                g += gxi;
            }
        };
    }
    macro_rules! S_FILLLINE {
        ($linefrom:expr, $lineto:expr) => {
            if sx[$lineto] < sx[$linefrom] {
                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(
                    psoOutput,
                    core::cmp::max(sx[$lineto], FillRect.left),
                    core::cmp::min(sx[$linefrom], FillRect.right),
                    sy,
                    Color,
                );
            } else {
                (DibFunctionsForBitmapFormat[(*psoOutput).iBitmapFormat as usize].DIB_HLine)(
                    psoOutput,
                    core::cmp::max(sx[$linefrom], FillRect.left),
                    core::cmp::min(sx[$lineto], FillRect.right),
                    sy,
                    Color,
                );
            }
        };
    }

    v1 = pVertex.offset((*gTriangle).Vertex1 as isize);
    v2 = pVertex.offset((*gTriangle).Vertex2 as isize);
    v3 = pVertex.offset((*gTriangle).Vertex3 as isize);

    if SMALLER!(v2, v1) {
        let mut t: PTRIVERTEX;
        SWAP!(v1, v2, t);
    }

    if SMALLER!(v3, v2) {
        let mut t: PTRIVERTEX;
        SWAP!(v2, v3, t);
        if SMALLER!(v2, v1) {
            SWAP!(v1, v2, t);
        }
    }

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeave),
        psoDest,
        core::ptr::addr_of_mut!(FillRect),
        0 as BOOL,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(psoOutput),
    ) == 0 {
        return 0;
    }

    if VCMPCLRS!(v1, v2, v3) {
        CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        loop {
            EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(RectEnum) as *mut ULONG) as i32;
            i = 0;
            while i < RectEnum.c && (*RectEnum.arcl.as_ptr().offset(i as isize)).top <= (*prclExtents).bottom {
                if RECTL_bIntersectRect(core::ptr::addr_of_mut!(FillRect), RectEnum.arcl.as_ptr().offset(i as isize) as *mut RECTL, prclExtents) != 0 {
                    DOINIT!(v1, v3, 0);
                    DOINIT!(v1, v2, 1);
                    DOINIT!(v2, v3, 2);

                    y = (*v1).y;
                    sy = (*v1).y + (*pptlDitherOrg).y;
                    bt = core::cmp::min((*v3).y + (*pptlDitherOrg).y, FillRect.bottom);

                    while sy < bt {
                        InY = !(sy < FillRect.top || sy >= FillRect.bottom);
                        IFINY!(v1, v3; DOLINE!(0););

                        IFINY!(v1, v2;
                            DOLINE!(1);
                            FILLLINE!(0, 1);
                        );

                        // NOTE: the C source has `ENDLINE(23, v3, 2)` here — `23`
                        // is a stray identifier that ENDLINE discards; it doesn't
                        // change the closing `}` semantics. We preserve the same
                        // outer `if (y >= v2->y && y <= v3->y)` bracket using IFINY!(v2, v3; ...).
                        IFINY!(v2, v3;
                            FILLLINE!(0, 2);
                            DOLINE!(2);
                            FILLLINE!(0, 2);
                        );

                        y += 1;
                        sy += 1;
                    }
                }
                i += 1;
            }
            if EnumMore == 0 {
                break;
            }
        }

        return IntEngLeave(core::ptr::addr_of_mut!(EnterLeave));
    }

    Color = XLATEOBJ_iXlate(pxlo, rgb_pack(((*v1).Red >> 8) as i32, ((*v1).Green >> 8) as i32, ((*v1).Blue >> 8) as i32));
    CLIPOBJ_cEnumStart(pco, 0 as BOOL, CT_RECTANGLES, CD_RIGHTDOWN, 0);
    loop {
        EnumMore = CLIPOBJ_bEnum(pco, core::mem::size_of::<RECT_ENUM>() as ULONG, core::ptr::addr_of_mut!(RectEnum) as *mut ULONG) as i32;
        i = 0;
        while i < RectEnum.c && (*RectEnum.arcl.as_ptr().offset(i as isize)).top <= (*prclExtents).bottom {
            if RECTL_bIntersectRect(core::ptr::addr_of_mut!(FillRect), RectEnum.arcl.as_ptr().offset(i as isize) as *mut RECTL, prclExtents) != 0 {
                S_INITLINE!(v1, v3, 0);
                S_INITLINE!(v1, v2, 1);
                S_INITLINE!(v2, v3, 2);

                y = (*v1).y;
                sy = (*v1).y + (*pptlDitherOrg).y;
                bt = core::cmp::min((*v3).y + (*pptlDitherOrg).y, FillRect.bottom);

                while sy < bt {
                    IFINY!(v1, v3; S_DOSTEP!(0););

                    IFINY!(v1, v2;
                        S_DOSTEP!(1);
                        S_FILLLINE!(0, 1);
                    );

                    // ENDLINE(23, v3, 2) — `23` discarded (see gradient.c:405/450).
                    IFINY!(v2, v3;
                        S_DOSTEP!(2);
                        S_FILLLINE!(0, 2);
                    );

                    y += 1;
                    sy += 1;
                }
            }
            i += 1;
        }
        if EnumMore == 0 {
            break;
        }
    }

    IntEngLeave(core::ptr::addr_of_mut!(EnterLeave))
}

unsafe fn IntEngIsNULLTriangle(pVertex: *mut TRIVERTEX, gt: *mut GRADIENT_TRIANGLE) -> BOOL {
    let v1 = pVertex.offset((*gt).Vertex1 as isize);
    let v2 = pVertex.offset((*gt).Vertex2 as isize);
    let v3 = pVertex.offset((*gt).Vertex3 as isize);

    if (*v1).x == (*v2).x && (*v1).y == (*v2).y {
        return 1;
    }
    if (*v1).x == (*v3).x && (*v1).y == (*v3).y {
        return 1;
    }
    if (*v2).x == (*v3).x && (*v2).y == (*v3).y {
        return 1;
    }
    0
}
#[no_mangle]
pub unsafe extern "system" fn EngGradientFill(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: u32,
    pMesh: *mut core::ffi::c_void,
    nMesh: u32,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    ulMode: u32,
) -> BOOL {
    let mut i: u32;
    let mut ret: BOOL = 0;
    let mut pco_local = pco;

    if pco.is_null() {
        pco_local = core::ptr::addr_of_mut!(gxcoTrivial) as *mut CLIPOBJ;
    }

    match ulMode {
        GRADIENT_FILL_RECT_H | GRADIENT_FILL_RECT_V => {
            let mut gr = pMesh as PGRADIENT_RECT;
            i = 0;
            while i < nMesh {
                #[cfg(target_arch = "x86")]
                let result = IntEngGradientFillRect(
                    psoDest,
                    pco_local,
                    pxlo,
                    pVertex,
                    nVertex,
                    gr,
                    prclExtents,
                    pptlDitherOrg,
                    if ulMode == GRADIENT_FILL_RECT_H { 1 } else { 0 },
                );
                #[cfg(not(target_arch = "x86"))]
                let result = IntEngGradientFillRect(
                    psoDest,
                    pco_local,
                    pxlo,
                    pVertex,
                    nVertex,
                    gr,
                    prclExtents,
                    pptlDitherOrg,
                    if ulMode == GRADIENT_FILL_RECT_H { 1 } else { 0 },
                );
                if result == 0 {
                    break;
                }
                i += 1;
                gr = gr.offset(1);
            }
            ret = 1;
        }
        GRADIENT_FILL_TRIANGLE => {
            let mut gt = pMesh as PGRADIENT_TRIANGLE;
            i = 0;
            while i < nMesh {
                if IntEngIsNULLTriangle(pVertex, gt) != 0 {
                    i += 1;
                    gt = gt.offset(1);
                    continue;
                }
                let result = IntEngGradientFillTriangle(
                    psoDest,
                    pco_local,
                    pxlo,
                    pVertex,
                    nVertex,
                    gt,
                    prclExtents,
                    pptlDitherOrg,
                );
                if result == 0 {
                    break;
                }
                i += 1;
                gt = gt.offset(1);
            }
            ret = 1;
        }
        _ => {}
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn IntEngGradientFill(
    psoDest: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    pVertex: *mut TRIVERTEX,
    nVertex: u32,
    pMesh: *mut core::ffi::c_void,
    nMesh: u32,
    prclExtents: *mut RECTL,
    pptlDitherOrg: *mut POINTL,
    ulMode: u32,
) -> BOOL {
    let Ret: BOOL;
    let psurf: *mut SURFACE;

    psurf = (psoDest as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if ((*psurf).flags & HOOK_GRADIENTFILL) != 0 {
        // GDIDEVFUNCS(psoDest).GradientFill — see gdidev_gradientfill above.
        // Same pattern as alphablend.rs::gdidev_alphablend.
        Ret = (gdidev_gradientfill(psoDest))(
            psoDest,
            pco,
            pxlo,
            pVertex,
            nVertex,
            pMesh,
            nMesh,
            prclExtents,
            pptlDitherOrg,
            ulMode,
        );
    } else {
        Ret = EngGradientFill(
            psoDest,
            pco,
            pxlo,
            pVertex,
            nVertex,
            pMesh,
            nMesh,
            prclExtents,
            pptlDitherOrg,
            ulMode,
        );
    }

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Gradient Functions
//  * FILE:              win32ss/gdi/eng/gradient.c
//  * PROGRAMER:         Thomas Weidenmueller
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* MACROS *********************************************************************/
//
// const LONG LINC[2] = {-1, 1};
//
// #define VERTEX(n) (pVertex + gt->n)
// #define COMPAREVERTEX(a, b) ((a)->x == (b)->x && (a)->y == (b)->y)
//
// /* Check if all three vectors have the same color, either R, G, or B */
// #define VCMPCLR(a, b, c, color) (a->color == b->color && a->color == c->color)
// /* Check if all three vectors have the same colors for R, G, and B, then
//  * NOT the result because we want to check for not using solid color logic */
// #define VCMPCLRS(a, b, c) \
//   !(VCMPCLR(a, b, c, Red) && VCMPCLR(a, b, c, Green) && VCMPCLR(a, b, c, Blue))
//
// /* Horizontal/Vertical gradients */
// #define HVINITCOL(Col, id) \
//   c[id] = v1->Col >> 8; \
//   dc[id] = abs((v2->Col >> 8) - c[id]); \
//   ec[id] = -(dy >> 1); \
//   ic[id] = LINC[(v2->Col >> 8) > c[id]]
// #define HVSTEPCOL(id) \
//   ec[id] += dc[id]; \
//   while(ec[id] > 0) \
//   { \
//     c[id] += ic[id]; \
//     ec[id] -= dy; \
//   }
//
// /* FUNCTIONS ******************************************************************/
//
// BOOL
// FASTCALL
// IntEngGradientFillRect(
//     IN SURFOBJ  *psoDest,
//     IN CLIPOBJ  *pco,
//     IN XLATEOBJ  *pxlo,
//     IN TRIVERTEX  *pVertex,
//     IN ULONG  nVertex,
//     IN PGRADIENT_RECT gRect,
//     IN RECTL  *prclExtents,
//     IN POINTL  *pptlDitherOrg,
//     IN BOOL Horizontal)
// {
//     SURFOBJ *psoOutput;
//     TRIVERTEX *v1, *v2;
//     RECTL rcGradient, rcSG;
//     RECT_ENUM RectEnum;
//     BOOL EnumMore;
//     ULONG i;
//     POINTL Translate;
//     INTENG_ENTER_LEAVE EnterLeave;
//     LONG y, dy, c[3], dc[3], ec[3], ic[3];
//
//     v1 = (pVertex + gRect->UpperLeft);
//     v2 = (pVertex + gRect->LowerRight);
//
//     rcGradient.left = min(v1->x, v2->x);
//     rcGradient.right = max(v1->x, v2->x);
//     rcGradient.top = min(v1->y, v2->y);
//     rcGradient.bottom = max(v1->y, v2->y);
//     rcSG = rcGradient;
//
//     if(Horizontal)
//     {
//         dy = abs(rcGradient.right - rcGradient.left);
//     }
//     else
//     {
//         dy = abs(rcGradient.bottom - rcGradient.top);
//     }
//
//     if(!IntEngEnter(&EnterLeave, psoDest, &rcSG, FALSE, &Translate, &psoOutput))
//     {
//         return FALSE;
//     }
//
//     if((v1->Red != v2->Red || v1->Green != v2->Green || v1->Blue != v2->Blue) && dy > 1)
//     {
//         CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//         do
//         {
//             RECTL FillRect;
//             ULONG Color;
//
//             if (Horizontal)
//             {
//                 EnumMore = CLIPOBJ_bEnum(pco, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//                 for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top <= rcSG.bottom; i++)
//                 {
//                     if (RECTL_bIntersectRect(&FillRect, &RectEnum.arcl[i], &rcSG))
//                     {
//                         HVINITCOL(Red, 0);
//                         HVINITCOL(Green, 1);
//                         HVINITCOL(Blue, 2);
//
//                         for (y = rcSG.left; y < FillRect.right; y++)
//                         {
//                             if (y >= FillRect.left)
//                             {
//                                 Color = XLATEOBJ_iXlate(pxlo, RGB(c[0], c[1], c[2]));
//                                 DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_VLine(
//                                     psoOutput, y + Translate.x, FillRect.top + Translate.y, FillRect.bottom + Translate.y, Color);
//                             }
//                             HVSTEPCOL(0);
//                             HVSTEPCOL(1);
//                             HVSTEPCOL(2);
//                         }
//                     }
//                 }
//
//                 continue;
//             }
//
//             /* vertical */
//             EnumMore = CLIPOBJ_bEnum(pco, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//             for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top <= rcSG.bottom; i++)
//             {
//                 if (RECTL_bIntersectRect(&FillRect, &RectEnum.arcl[i], &rcSG))
//                 {
//                     HVINITCOL(Red, 0);
//                     HVINITCOL(Green, 1);
//                     HVINITCOL(Blue, 2);
//
//                     for (y = rcSG.top; y < FillRect.bottom; y++)
//                     {
//                         if (y >= FillRect.top)
//                         {
//                             Color = XLATEOBJ_iXlate(pxlo, RGB(c[0], c[1], c[2]));
//                             DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_HLine(psoOutput,
//                                                                                             FillRect.left + Translate.x,
//                                                                                             FillRect.right + Translate.x,
//                                                                                             y + Translate.y,
//                                                                                             Color);
//                         }
//                         HVSTEPCOL(0);
//                         HVSTEPCOL(1);
//                         HVSTEPCOL(2);
//                     }
//                 }
//             }
//
//         }
//         while (EnumMore);
//
//         return IntEngLeave(&EnterLeave);
//     }
//
//     /* rectangle has only one color, no calculation required */
//     CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//     do
//     {
//         RECTL FillRect;
//         ULONG Color = XLATEOBJ_iXlate(pxlo, RGB(v1->Red >> 8, v1->Green >> 8, v1->Blue >> 8));
//
//         EnumMore = CLIPOBJ_bEnum(pco, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//         for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top <= rcSG.bottom; i++)
//         {
//             if (RECTL_bIntersectRect(&FillRect, &RectEnum.arcl[i], &rcSG))
//             {
//                 for (; FillRect.top < FillRect.bottom; FillRect.top++)
//                 {
//                     DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_HLine(psoOutput,
//                                                                                     FillRect.left + Translate.x,
//                                                                                     FillRect.right + Translate.x,
//                                                                                     FillRect.top + Translate.y,
//                                                                                     Color);
//                 }
//             }
//         }
//     }
//     while (EnumMore);
//
//     return IntEngLeave(&EnterLeave);
// }
//
// /* Fill triangle with solid color */
// #define S_FILLLINE(linefrom,lineto) \
//   if(sx[lineto] < sx[linefrom]) \
//     DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_HLine(psoOutput, max(sx[lineto], FillRect.left), min(sx[linefrom], FillRect.right), sy, Color); \
//   else \
//     DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_HLine(psoOutput, max(sx[linefrom], FillRect.left), min(sx[lineto], FillRect.right), sy, Color);
//
// #define S_DOLINE(a,b,line) \
//   ex[line] += dx[line]; \
//   while(ex[line] > 0 && x[line] != destx[line]) \
//   { \
//     x[line] += incx[line]; \
//     sx[line] += incx[line]; \
//     ex[line] -= dy[line]; \
//   }
//
// #define S_GOLINE(a,b,line) \
//   if(y >= a->y && y <= b->y) \
//   {
//
// #define S_ENDLINE(a,b,line) \
//   }
//
// #define S_INITLINE(a,b,line) \
//   x[line] = a->x; \
//   sx[line] =  a->x + pptlDitherOrg->x; \
//   dx[line] = abs(b->x - a->x); \
//   dy[line] = abs(b->y - a->y); \
//   incx[line] = LINC[b->x > a->x]; \
//   ex[line] = -(dy[line]>>1); \
//   destx[line] = b->x
//
// /* Fill triangle with gradient */
// #define INITCOL(a,b,line,col,id) \
//   c[line][id] = a->col >> 8; \
//   dc[line][id] = abs((b->col >> 8) - c[line][id]); \
//   ec[line][id] = -(dy[line]>>1); \
//   ic[line][id] = LINC[(b->col >> 8) > c[line][id]]
//
// #define STEPCOL(a,b,line,col,id) \
//   ec[line][id] += dc[line][id]; \
//   if(dy[line] != 0) \
//   while(ec[line][id] > 0) \
//   { \
//     c[line][id] += ic[line][id]; \
//     ec[line][id] -= dy[line]; \
//   }
//
// #define FINITCOL(linefrom,lineto,colid) \
//   gc[colid] = c[linefrom][colid]; \
//   gd[colid] = abs(c[lineto][colid] - gc[colid]); \
//   ge[colid] = -(gx >> 1); \
//   gi[colid] = LINC[c[lineto][colid] > gc[colid]]
//
// #define FDOCOL(linefrom,lineto,colid) \
//   ge[colid] += gd[colid]; \
//   if (gx != 0) \
//   while(ge[colid] > 0) \
//   { \
//     gc[colid] += gi[colid]; \
//     ge[colid] -= gx; \
//   }
//
// #define FILLLINE(linefrom,lineto) \
//   gx = abs(sx[lineto] - sx[linefrom]); \
//   gxi = LINC[sx[linefrom] < sx[lineto]]; \
//   FINITCOL(linefrom, lineto, 0); \
//   FINITCOL(linefrom, lineto, 1); \
//   FINITCOL(linefrom, lineto, 2); \
//   g_end = sx[lineto] + gxi; \
//   for(g = sx[linefrom]; g != g_end; g += gxi) \
//   { \
//     if(InY && g >= FillRect.left && g < FillRect.right) \
//     { \
//       Color = XLATEOBJ_iXlate(pxlo, RGB(gc[0], gc[1], gc[2])); \
//       DibFunctionsForBitmapFormat[psoOutput->iBitmapFormat].DIB_PutPixel(psoOutput, g, sy, Color); \
//     } \
//     FDOCOL(linefrom, lineto, 0); \
//     FDOCOL(linefrom, lineto, 1); \
//     FDOCOL(linefrom, lineto, 2); \
//   }
//
// #define DOLINE(a,b,line) \
//   STEPCOL(a, b, line, Red, 0); \
//   STEPCOL(a, b, line, Green, 1); \
//   STEPCOL(a, b, line, Blue, 2); \
//   ex[line] += dx[line]; \
//   while(ex[line] > 0 && x[line] != destx[line]) \
//   { \
//     x[line] += incx[line]; \
//     sx[line] += incx[line]; \
//     ex[line] -= dy[line]; \
//   }
//
// #define GOLINE(a,b,line) \
//   if(y >= a->y && y <= b->y) \
//   {
//
// #define ENDLINE(a,b,line) \
//   }
//
// #define INITLINE(a,b,line) \
//   x[line] = a->x; \
//   sx[line] = a->x + pptlDitherOrg->x - 1; \
//   dx[line] = abs(b->x - a->x); \
//   dy[line] = abs(b->y - a->y); \
//   incx[line] = LINC[b->x > a->x]; \
//   ex[line] = -(dy[line]>>1); \
//   destx[line] = b->x
//
// #define DOINIT(a, b, line) \
//   INITLINE(a, b, line); \
//   INITCOL(a, b, line, Red, 0); \
//   INITCOL(a, b, line, Green, 1); \
//   INITCOL(a, b, line, Blue, 2);
//
// #define SMALLER(a,b)     (a->y < b->y) || (a->y == b->y && a->x < b->x)
//
// #define SWAP(a,b,c)  c = a;\
//                      a = b;\
//                      b = c
//
// #define NLINES 3
//
// BOOL
// FASTCALL
// IntEngGradientFillTriangle(
//     IN SURFOBJ  *psoDest,
//     IN CLIPOBJ  *pco,
//     IN XLATEOBJ  *pxlo,
//     IN TRIVERTEX  *pVertex,
//     IN ULONG  nVertex,
//     IN PGRADIENT_TRIANGLE gTriangle,
//     IN RECTL  *prclExtents,
//     IN POINTL  *pptlDitherOrg)
// {
//     SURFOBJ *psoOutput;
//     PTRIVERTEX v1, v2, v3;
//     RECT_ENUM RectEnum;
//     BOOL EnumMore;
//     ULONG i;
//     POINTL Translate;
//     INTENG_ENTER_LEAVE EnterLeave;
//     RECTL FillRect = { 0, 0, 0, 0 };
//     ULONG Color;
//
//     BOOL sx[NLINES];
//     LONG x[NLINES], dx[NLINES], dy[NLINES], incx[NLINES], ex[NLINES], destx[NLINES];
//     LONG c[NLINES][3], dc[NLINES][3], ec[NLINES][3], ic[NLINES][3]; /* colors on lines */
//     LONG g, gx, gxi, gc[3], gd[3], ge[3], gi[3]; /* colors in triangle */
//     LONG sy, y, bt, g_end;
//
//     v1 = (pVertex + gTriangle->Vertex1);
//     v2 = (pVertex + gTriangle->Vertex2);
//     v3 = (pVertex + gTriangle->Vertex3);
//
//     /* bubble sort */
//     if (SMALLER(v2, v1))
//     {
//         TRIVERTEX *t;
//         SWAP(v1, v2, t);
//     }
//
//     if (SMALLER(v3, v2))
//     {
//         TRIVERTEX *t;
//         SWAP(v2, v3, t);
//         if (SMALLER(v2, v1))
//         {
//             SWAP(v1, v2, t);
//         }
//     }
//
//     DPRINT("Triangle: (%i,%i) (%i,%i) (%i,%i)\n", v1->x, v1->y, v2->x, v2->y, v3->x, v3->y);
//
//     if (!IntEngEnter(&EnterLeave, psoDest, &FillRect, FALSE, &Translate, &psoOutput))
//     {
//         return FALSE;
//     }
//
//     if (VCMPCLRS(v1, v2, v3))
//     {
//       CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//       do
//       {
//         EnumMore = CLIPOBJ_bEnum(pco, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//         for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top <= prclExtents->bottom; i++)
//         {
//           if (RECTL_bIntersectRect(&FillRect, &RectEnum.arcl[i], prclExtents))
//           {
//             BOOL InY;
//
//             DOINIT(v1, v3, 0);
//             DOINIT(v1, v2, 1);
//             DOINIT(v2, v3, 2);
//
//             y = v1->y;
//             sy = v1->y + pptlDitherOrg->y;
//             bt = min(v3->y + pptlDitherOrg->y, FillRect.bottom);
//
//             while (sy < bt)
//             {
//               InY = !(sy < FillRect.top || sy >= FillRect.bottom);
//               GOLINE(v1, v3, 0);
//               DOLINE(v1, v3, 0);
//               ENDLINE(v1, v3, 0);
//
//               GOLINE(v1, v2, 1);
//               DOLINE(v1, v2, 1);
//               FILLLINE(0, 1);
//               ENDLINE(v1, v2, 1);
//
//               GOLINE(v2, v3, 2);
//               FILLLINE(0, 2);
//               DOLINE(v2, v3, 2);
//               FILLLINE(0, 2);
//               ENDLINE(23, v3, 2);
//
//               y++;
//               sy++;
//             }
//           }
//         }
//       } while (EnumMore);
//
//       return IntEngLeave(&EnterLeave);
//     }
//
//     /* fill triangle with one solid color */
//
//     Color = XLATEOBJ_iXlate(pxlo, RGB(v1->Red >> 8, v1->Green >> 8, v1->Blue >> 8));
//     CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//     do
//     {
//       EnumMore = CLIPOBJ_bEnum(pco, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//       for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top <= prclExtents->bottom; i++)
//       {
//         if (RECTL_bIntersectRect(&FillRect, &RectEnum.arcl[i], prclExtents))
//         {
//           S_INITLINE(v1, v3, 0);
//           S_INITLINE(v1, v2, 1);
//           S_INITLINE(v2, v3, 2);
//
//           y = v1->y;
//           sy = v1->y + pptlDitherOrg->y;
//           bt = min(v3->y + pptlDitherOrg->y, FillRect.bottom);
//
//           while (sy < bt)
//           {
//             S_GOLINE(v1, v3, 0);
//             S_DOLINE(v1, v3, 0);
//             S_ENDLINE(v1, v3, 0);
//
//             S_GOLINE(v1, v2, 1);
//             S_DOLINE(v1, v2, 1);
//             S_FILLLINE(0, 1);
//             S_ENDLINE(v1, v2, 1);
//
//             S_GOLINE(v2, v3, 2);
//             S_DOLINE(v2, v3, 2);
//             S_FILLLINE(0, 2);
//             S_ENDLINE(23, v3, 2);
//
//             y++;
//             sy++;
//           }
//         }
//       }
//     } while (EnumMore);
//
//     return IntEngLeave(&EnterLeave);
// }
//
//
// static
// BOOL
// IntEngIsNULLTriangle(TRIVERTEX  *pVertex, GRADIENT_TRIANGLE *gt)
// {
//     if(COMPAREVERTEX(VERTEX(Vertex1), VERTEX(Vertex2)))
//         return TRUE;
//     if(COMPAREVERTEX(VERTEX(Vertex1), VERTEX(Vertex3)))
//         return TRUE;
//     if(COMPAREVERTEX(VERTEX(Vertex2), VERTEX(Vertex3)))
//         return TRUE;
//     return FALSE;
// }
//
//
// BOOL
// APIENTRY
// EngGradientFill(
//     _Inout_ SURFOBJ *psoDest,
//     _In_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ TRIVERTEX *pVertex,
//     _In_ ULONG nVertex,
//     _In_ PVOID pMesh,
//     _In_ ULONG nMesh,
//     _In_ RECTL *prclExtents,
//     _In_ POINTL *pptlDitherOrg,
//     _In_ ULONG ulMode)
// {
//     ULONG i;
//     BOOL ret = FALSE;
//
//     /* Check for NULL clip object */
//     if (pco == NULL)
//     {
//         /* Use the trivial one instead */
//         pco = (CLIPOBJ *)&gxcoTrivial;//.coClient;
//     }
//
//     switch(ulMode)
//     {
//         case GRADIENT_FILL_RECT_H:
//         case GRADIENT_FILL_RECT_V:
//         {
//             PGRADIENT_RECT gr = (PGRADIENT_RECT)pMesh;
//             for (i = 0; i < nMesh; i++, gr++)
//             {
//                 if (!IntEngGradientFillRect(psoDest,
//                                             pco,
//                                             pxlo,
//                                             pVertex,
//                                             nVertex,
//                                             gr,
//                                             prclExtents,
//                                             pptlDitherOrg,
//                                             (ulMode == GRADIENT_FILL_RECT_H)))
//                 {
//                     break;
//                 }
//             }
//             ret = TRUE;
//             break;
//         }
//         case GRADIENT_FILL_TRIANGLE:
//         {
//             PGRADIENT_TRIANGLE gt = (PGRADIENT_TRIANGLE)pMesh;
//             for (i = 0; i < nMesh; i++, gt++)
//             {
//                 if (IntEngIsNULLTriangle(pVertex, gt))
//                 {
//                     /* skip empty triangles */
//                     continue;
//                 }
//                 if (!IntEngGradientFillTriangle(psoDest,
//                                                 pco,
//                                                 pxlo,
//                                                 pVertex,
//                                                 nVertex,
//                                                 gt,
//                                                 prclExtents,
//                                                 pptlDitherOrg))
//                 {
//                     break;
//                 }
//             }
//             ret = TRUE;
//             break;
//         }
//     }
//
//     return ret;
// }
//
// BOOL
// APIENTRY
// IntEngGradientFill(
//     IN SURFOBJ *psoDest,
//     IN CLIPOBJ *pco,
//     IN XLATEOBJ *pxlo,
//     IN TRIVERTEX *pVertex,
//     IN ULONG nVertex,
//     IN PVOID pMesh,
//     IN ULONG nMesh,
//     IN RECTL *prclExtents,
//     IN POINTL *pptlDitherOrg,
//     IN ULONG ulMode)
// {
//     BOOL Ret;
//     SURFACE *psurf;
//     ASSERT(psoDest);
//
//     psurf = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//     ASSERT(psurf);
//
//     if (psurf->flags & HOOK_GRADIENTFILL)
//     {
//         Ret = GDIDEVFUNCS(psoDest).GradientFill(psoDest,
//                                                 pco,
//                                                 pxlo,
//                                                 pVertex,
//                                                 nVertex,
//                                                 pMesh,
//                                                 nMesh,
//                                                 prclExtents,
//                                                 pptlDitherOrg,
//                                                 ulMode);
//     }
//     else
//     {
//         Ret = EngGradientFill(psoDest,
//                               pco,
//                               pxlo,
//                               pVertex,
//                               nVertex,
//                               pMesh,
//                               nMesh,
//                               prclExtents,
//                               pptlDitherOrg,
//                               ulMode);
//     }
//
//     return Ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
