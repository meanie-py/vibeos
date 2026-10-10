use win32k_sys::win32k::*;

extern "system" {
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: *mut core::ffi::c_void) -> i32;
}

// Local shadow of `DibFunctionsForBitmapFormat` — the bindgen view
// (`[DIB_FUNCTIONS; 0]` with `Option<extern fn>` fields) isn't directly
// indexable/callable. Same pattern as gradient.rs.
#[repr(C)]
struct DIB_FN_TABLE {
    DIB_PutPixel:      unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, ULONG),
    DIB_GetPixel:      unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG) -> ULONG,
    DIB_HLine:         unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, LONG, ULONG),
    DIB_VLine:         unsafe extern "C" fn(*mut SURFOBJ, LONG, LONG, LONG, ULONG),
    DIB_BitBlt:        unsafe extern "C" fn(*mut core::ffi::c_void) -> BOOLEAN,
    DIB_BitBltSrcCopy: unsafe extern "C" fn(*mut core::ffi::c_void) -> BOOLEAN,
    DIB_StretchBlt: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut SURFOBJ, *mut SURFOBJ,
        *mut RECTL, *mut RECTL, *mut POINTL, *mut BRUSHOBJ,
        *mut POINTL, *mut XLATEOBJ, u32,
    ) -> BOOLEAN,
    DIB_TransparentBlt: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut RECTL, *mut RECTL, *mut XLATEOBJ, ULONG,
    ) -> BOOLEAN,
    DIB_ColorFill:  unsafe extern "C" fn(*mut SURFOBJ, *mut RECTL, ULONG) -> BOOLEAN,
    DIB_AlphaBlend: unsafe extern "C" fn(
        *mut SURFOBJ, *mut SURFOBJ, *mut RECTL, *mut RECTL,
        *mut CLIPOBJ, *mut XLATEOBJ, *mut BLENDOBJ,
    ) -> BOOLEAN,
}
extern "C" {
    // The real symbol is a sized array indexed by `iBitmapFormat`; bindgen
    // exposes it as `[DIB_FUNCTIONS; 0]`, so we redeclare with a generous
    // fixed size for the indexing arithmetic.
    static DibFunctionsForBitmapFormat: [DIB_FN_TABLE; 11];
}

// Inline-expand of `GDIDEVFUNCS(SurfObj)` from `win32ss/gdi/eng/surface.h:106`:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
// Mirrors bitblt.rs / copybits.rs / gradient.rs.
#[inline(always)]
unsafe fn GDIDEVFUNCS<'a>(pso: *mut SURFOBJ) -> &'a DRIVER_FUNCTIONS {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    &(*pdev).__bindgen_anon_1.DriverFunctions
}

#[inline(always)]
fn long_max(a: LONG, b: LONG) -> LONG { core::cmp::max(a, b) }
#[inline(always)]
fn long_min(a: LONG, b: LONG) -> LONG { core::cmp::min(a, b) }

// BRUSHBODY view — the C `_BRUSH` struct embeds `BRUSHBODY` anonymously
// (win32ss/gdi/ntgdi/brush.h:58); bindgen lost the field names and emits
// `__bindgen_padding_0`. We reconstruct a typed view over the bytes after
// `BaseObject` so we can access `flAttrs`, `pStyle`, `dwStyleCount`,
// `ulStyleSize`. Layout mirrors `_BRUSHBODY` exactly (bindgen_amd64.rs:116201).
#[repr(C)]
struct BRUSHBODY_View {
    iHatch:        ULONG,
    hbmPattern:    HBITMAP,
    hbmClient:     HBITMAP,
    flAttrs:       ULONG,
    ulBrushUnique: ULONG,
    pBrushAttr:    *mut BRUSH_ATTR,
    BrushAttr:     BRUSH_ATTR,
    ptOrigin:      POINTL,
    bCacheGrabbed: ULONG,
    crBack:        COLORREF,
    crFore:        COLORREF,
    ulPalTime:     ULONG,
    ulSurfTime:    ULONG,
    pvRBrush:      PVOID,
    hdev:          HDEV,
    lWidth:        LONG,
    eWidth:        FLOATOBJ,
    ulPenStyle:    ULONG,
    pStyle:        *mut DWORD,
    dwStyleCount:  ULONG,
    jJoin:         BYTE,
    jEndCap:       BYTE,
    iBrushStyle:   INT,
    ulStyleSize:   ULONG,
    // ListHead omitted — lineto.c doesn't touch it.
}

#[inline(always)]
unsafe fn brushbody(pbrush: PBRUSH) -> *mut BRUSHBODY_View {
    // BRUSHBODY immediately follows BaseObject in `_BRUSH`.
    (pbrush as *mut u8).add(core::mem::size_of::<BASEOBJECT>()) as *mut BRUSHBODY_View
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the provided preamble.
// The #define NDEBUG is a preprocessor directive that does not generate runtime declarations.

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn TranslateRects(RectEnum: *mut RECT_ENUM, Translate: *mut POINTL) {
    if (*Translate).x != 0 || (*Translate).y != 0 {
        let mut CurrentRect = (*RectEnum).arcl.as_mut_ptr();
        let end = CurrentRect.add((*RectEnum).c as usize);
        while CurrentRect < end {
            (*CurrentRect).left   += (*Translate).x;
            (*CurrentRect).right  += (*Translate).x;
            (*CurrentRect).top    += (*Translate).y;
            (*CurrentRect).bottom += (*Translate).y;
            CurrentRect = CurrentRect.add(1);
        }
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe fn TranslateRects(RectEnum: *mut RECT_ENUM, Translate: *mut POINTL) {
    if (*Translate).x != 0 || (*Translate).y != 0 {
        let mut CurrentRect = (*RectEnum).arcl.as_mut_ptr();
        let end = CurrentRect.add((*RectEnum).c as usize);
        while CurrentRect < end {
            (*CurrentRect).left   += (*Translate).x;
            (*CurrentRect).right  += (*Translate).x;
            (*CurrentRect).top    += (*Translate).y;
            (*CurrentRect).bottom += (*Translate).y;
            CurrentRect = CurrentRect.add(1);
        }
    }
}

unsafe fn HandleStyles(
    pbo: *mut BRUSHOBJ,
    Translate: *mut POINTL,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    dx: LONG,
    dy: LONG,
    piStyle: *mut ULONG,
) -> LONG {
    let pebo = pbo as PEBRUSHOBJ;
    let pulStyles = (*brushbody((*pebo).pbrush)).pStyle;
    let cStyles = (*brushbody((*pebo).pbrush)).dwStyleCount;
    let mut iStyle: ULONG;
    let diStyle: LONG;
    let mut offStyle: LONG;
    let mut lStyleMax: LONG;

    if cStyles > 0 {
        if deltax > deltay {
            offStyle = (-(*Translate).x) % (*brushbody((*pebo).pbrush)).ulStyleSize as LONG;
            diStyle = dx;
            lStyleMax = x;
        } else {
            offStyle = (-(*Translate).y) % (*brushbody((*pebo).pbrush)).ulStyleSize as LONG;
            diStyle = dy;
            lStyleMax = y;
        }

        iStyle = 0;
        while offStyle >= *pulStyles.offset(iStyle as isize) as LONG {
            offStyle -= *pulStyles.offset(iStyle as isize) as LONG;
            iStyle += 1;
        }

        if diStyle > 0 {
            lStyleMax += *pulStyles.offset(iStyle as isize) as LONG - offStyle;
        } else {
            lStyleMax -= offStyle + 1;
        }
    } else {
        iStyle = 0;
        lStyleMax = MAXLONG as LONG;
    }

    *piStyle = iStyle;
    lStyleMax
}

// ---------------------------------------------------------------------------
// NWtoSE / SWtoNE / NEtoSW / SEtoNW
// FASTCALL on i386, plain on amd64/arm64. Bodies via shared `nwtose_body` etc.
// to keep the per-arch shim tiny.
// ---------------------------------------------------------------------------

unsafe fn nwtose_body(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    mut x: LONG,
    mut y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let Pixel: ULONG = (*pbo).iSolidColor;
    let pebo: PEBRUSHOBJ = pbo as PEBRUSHOBJ;
    let pulStyles: *mut ULONG = (*brushbody((*pebo).pbrush)).pStyle;
    let cStyles: ULONG = (*brushbody((*pebo).pbrush)).dwStyleCount;
    let mut iStyle: ULONG = 0;
    let mut lStyleMax: LONG;

    lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, 1, 1, &mut iStyle);

    CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_RIGHTDOWN, 0);
    let mut EnumMore: BOOL = CLIPOBJ_bEnum(
        Clip,
        core::mem::size_of::<RECT_ENUM>() as ULONG,
        &mut RectEnum as *mut _ as *mut core::ffi::c_void,
    ) as i32;
    TranslateRects(&mut RectEnum, Translate);
    let mut ClipRect: *mut RECTL = RectEnum.arcl.as_mut_ptr();
    let delta: LONG = long_max(deltax, deltay);
    let mut i: LONG = 0;
    let mut error: LONG = delta >> 1;

    while i < delta
        && (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) || EnumMore != 0)
    {
        while (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize)
            && ((*ClipRect).bottom <= y
                || ((*ClipRect).top <= y && (*ClipRect).right <= x)))
            || EnumMore != 0
        {
            if RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) <= ClipRect {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    &mut RectEnum as *mut _ as *mut core::ffi::c_void,
                ) as i32;
                TranslateRects(&mut RectEnum, Translate);
                ClipRect = RectEnum.arcl.as_mut_ptr();
            } else {
                ClipRect = ClipRect.add(1);
            }
        }
        if ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) {
            if ((*ClipRect).left <= x && (*ClipRect).top <= y) && ((iStyle & 1) == 0) {
                (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_PutPixel)(
                    OutputObj, x, y, Pixel,
                );
            }
            if deltax < deltay {
                y += 1;
                if y == lStyleMax {
                    iStyle = (iStyle + 1) % cStyles;
                    lStyleMax = y + *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltax;
                if deltay <= error {
                    x += 1;
                    error -= deltay;
                }
            } else {
                x += 1;
                if x == lStyleMax {
                    iStyle = (iStyle + 1) % cStyles;
                    lStyleMax = x + *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltay;
                if deltax <= error {
                    y += 1;
                    error -= deltax;
                }
            }
            i += 1;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn NWtoSE(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    nwtose_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

#[cfg(not(target_arch = "x86"))]
unsafe fn NWtoSE(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    nwtose_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

unsafe fn swtone_body(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    mut x: LONG,
    mut y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let Pixel: ULONG = (*pbo).iSolidColor;
    let pebo: PEBRUSHOBJ = pbo as PEBRUSHOBJ;
    let pulStyles: *mut ULONG = (*brushbody((*pebo).pbrush)).pStyle;
    let cStyles: ULONG = (*brushbody((*pebo).pbrush)).dwStyleCount;
    let mut iStyle: ULONG = 0;
    let mut lStyleMax: LONG;

    lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, 1, -1, &mut iStyle);

    CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_RIGHTUP, 0);
    let mut EnumMore: BOOL = CLIPOBJ_bEnum(
        Clip,
        core::mem::size_of::<RECT_ENUM>() as ULONG,
        &mut RectEnum as *mut _ as *mut core::ffi::c_void,
    ) as i32;
    TranslateRects(&mut RectEnum, Translate);
    let mut ClipRect: *mut RECTL = RectEnum.arcl.as_mut_ptr();
    let delta: LONG = long_max(deltax, deltay);
    let mut i: LONG = 0;
    let mut error: LONG = delta >> 1;

    while i < delta
        && (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) || EnumMore != 0)
    {
        while (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize)
            && (y < (*ClipRect).top
                || (y < (*ClipRect).bottom && (*ClipRect).right <= x)))
            || EnumMore != 0
        {
            if RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) <= ClipRect {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    &mut RectEnum as *mut _ as *mut core::ffi::c_void,
                ) as i32;
                TranslateRects(&mut RectEnum, Translate);
                ClipRect = RectEnum.arcl.as_mut_ptr();
            } else {
                ClipRect = ClipRect.add(1);
            }
        }
        if ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) {
            if ((*ClipRect).left <= x && y < (*ClipRect).bottom) && ((iStyle & 1) == 0) {
                (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_PutPixel)(
                    OutputObj, x, y, Pixel,
                );
            }
            if deltax < deltay {
                y -= 1;
                if y == lStyleMax {
                    iStyle = iStyle.wrapping_sub(1) % cStyles;
                    lStyleMax = y - *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltax;
                if deltay <= error {
                    x += 1;
                    error -= deltay;
                }
            } else {
                x += 1;
                if x == lStyleMax {
                    iStyle = (iStyle + 1) % cStyles;
                    lStyleMax = x + *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltay;
                if deltax <= error {
                    y -= 1;
                    error -= deltax;
                }
            }
            i += 1;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn SWtoNE(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    swtone_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

#[cfg(not(target_arch = "x86"))]
unsafe fn SWtoNE(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    swtone_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

unsafe fn netosw_body(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    mut x: LONG,
    mut y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let Pixel: ULONG = (*pbo).iSolidColor;
    let pebo: PEBRUSHOBJ = pbo as PEBRUSHOBJ;
    let pulStyles: *mut ULONG = (*brushbody((*pebo).pbrush)).pStyle;
    let cStyles: ULONG = (*brushbody((*pebo).pbrush)).dwStyleCount;
    let mut iStyle: ULONG = 0;
    let mut lStyleMax: LONG;

    lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, -1, 1, &mut iStyle);

    CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_LEFTDOWN, 0);
    let mut EnumMore: BOOL = CLIPOBJ_bEnum(
        Clip,
        core::mem::size_of::<RECT_ENUM>() as ULONG,
        &mut RectEnum as *mut _ as *mut core::ffi::c_void,
    ) as i32;
    TranslateRects(&mut RectEnum, Translate);
    let mut ClipRect: *mut RECTL = RectEnum.arcl.as_mut_ptr();
    let delta: LONG = long_max(deltax, deltay);
    let mut i: LONG = 0;
    let mut error: LONG = delta >> 1;

    while i < delta
        && (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) || EnumMore != 0)
    {
        while (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize)
            && ((*ClipRect).bottom <= y
                || ((*ClipRect).top <= y && x < (*ClipRect).left)))
            || EnumMore != 0
        {
            if RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) <= ClipRect {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    &mut RectEnum as *mut _ as *mut core::ffi::c_void,
                ) as i32;
                TranslateRects(&mut RectEnum, Translate);
                ClipRect = RectEnum.arcl.as_mut_ptr();
            } else {
                ClipRect = ClipRect.add(1);
            }
        }
        if ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) {
            if (x < (*ClipRect).right && (*ClipRect).top <= y) && ((iStyle & 1) == 0) {
                (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_PutPixel)(
                    OutputObj, x, y, Pixel,
                );
            }
            if deltax < deltay {
                y += 1;
                if y == lStyleMax {
                    iStyle = (iStyle + 1) % cStyles;
                    lStyleMax = y + *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltax;
                if deltay <= error {
                    x -= 1;
                    error -= deltay;
                }
            } else {
                x -= 1;
                if x == lStyleMax {
                    iStyle = iStyle.wrapping_sub(1) % cStyles;
                    lStyleMax = x - *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltay;
                if deltax <= error {
                    y += 1;
                    error -= deltax;
                }
            }
            i += 1;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn NEtoSW(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    netosw_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

#[cfg(not(target_arch = "x86"))]
unsafe fn NEtoSW(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    netosw_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

unsafe fn setonw_body(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    mut x: LONG,
    mut y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let Pixel: ULONG = (*pbo).iSolidColor;
    let pebo: PEBRUSHOBJ = pbo as PEBRUSHOBJ;
    let pulStyles: *mut ULONG = (*brushbody((*pebo).pbrush)).pStyle;
    let cStyles: ULONG = (*brushbody((*pebo).pbrush)).dwStyleCount;
    let mut iStyle: ULONG = 0;
    let mut lStyleMax: LONG;

    lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, -1, -1, &mut iStyle);

    CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_LEFTUP, 0);
    let mut EnumMore: BOOL = CLIPOBJ_bEnum(
        Clip,
        core::mem::size_of::<RECT_ENUM>() as ULONG,
        &mut RectEnum as *mut _ as *mut core::ffi::c_void,
    ) as i32;
    TranslateRects(&mut RectEnum, Translate);
    let mut ClipRect: *mut RECTL = RectEnum.arcl.as_mut_ptr();
    let delta: LONG = long_max(deltax, deltay);
    let mut i: LONG = 0;
    let mut error: LONG = delta >> 1;

    while i < delta
        && (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) || EnumMore != 0)
    {
        while (ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize)
            && (y < (*ClipRect).top
                || (y < (*ClipRect).bottom && x < (*ClipRect).left)))
            || EnumMore != 0
        {
            if RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) <= ClipRect {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    &mut RectEnum as *mut _ as *mut core::ffi::c_void,
                ) as i32;
                TranslateRects(&mut RectEnum, Translate);
                ClipRect = RectEnum.arcl.as_mut_ptr();
            } else {
                ClipRect = ClipRect.add(1);
            }
        }
        if ClipRect < RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize) {
            if (x < (*ClipRect).right && y < (*ClipRect).bottom) && ((iStyle & 1) == 0) {
                (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_PutPixel)(
                    OutputObj, x, y, Pixel,
                );
            }
            if deltax < deltay {
                y -= 1;
                if y == lStyleMax {
                    iStyle = iStyle.wrapping_sub(1) % cStyles;
                    lStyleMax = y - *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltax;
                if deltay <= error {
                    x -= 1;
                    error -= deltay;
                }
            } else {
                x -= 1;
                if x == lStyleMax {
                    iStyle = iStyle.wrapping_sub(1) % cStyles;
                    lStyleMax = x - *pulStyles.add(iStyle as usize) as LONG;
                }
                error += deltay;
                if deltax <= error {
                    y -= 1;
                    error -= deltax;
                }
            }
            i += 1;
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn SEtoNW(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    setonw_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

#[cfg(not(target_arch = "x86"))]
unsafe fn SEtoNW(
    OutputObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x: LONG,
    y: LONG,
    deltax: LONG,
    deltay: LONG,
    Translate: *mut POINTL,
) {
    setonw_body(OutputObj, Clip, pbo, x, y, deltax, deltay, Translate)
}

// ---------------------------------------------------------------------------
// EngLineTo — APIENTRY (= NTAPI = stdcall on i386, cdecl on amd64/arm64)
// spec: `@ stdcall EngLineTo(ptr ptr ptr long long long long ptr long)`
// ---------------------------------------------------------------------------

unsafe fn eng_lineto_body(
    DestObj: *mut SURFOBJ,
    mut Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    mut x1: LONG,
    mut y1: LONG,
    mut x2: LONG,
    mut y2: LONG,
    RectBounds: *mut RECTL,
    _mix: MIX,
) -> BOOL {
    let Pixel: ULONG = (*pbo).iSolidColor;
    let mut OutputObj: *mut SURFOBJ = core::ptr::null_mut();
    let mut DestRect: RECTL = core::mem::zeroed();
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeave: INTENG_ENTER_LEAVE_TAG = core::mem::zeroed();
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut pcoPriv: *mut CLIPOBJ = core::ptr::null_mut();
    let pebo: PEBRUSHOBJ = pbo as PEBRUSHOBJ;
    let cStyles: ULONG = (*brushbody((*pebo).pbrush)).dwStyleCount;

    if x1 < x2 {
        DestRect.left = x1;
        DestRect.right = x2;
    } else {
        DestRect.left = x2;
        DestRect.right = x1 + 1;
    }
    if y1 < y2 {
        DestRect.top = y1;
        DestRect.bottom = y2;
    } else {
        DestRect.top = y2;
        DestRect.bottom = y1 + 1;
    }

    if IntEngEnter(&mut EnterLeave, DestObj, &mut DestRect, 0, &mut Translate, &mut OutputObj) == 0 {
        return 0;
    }

    if Clip.is_null() {
        Clip = EngCreateClip();
        pcoPriv = Clip;
        if Clip.is_null() {
            return 0;
        }
        IntEngUpdateClipRegion(Clip as *mut XCLIPOBJ, 0, core::ptr::null(), RectBounds);
    }

    x1 += Translate.x;
    x2 += Translate.x;
    y1 += Translate.y;
    y2 += Translate.y;

    let x: LONG = x1;
    let y: LONG = y1;
    let mut deltax: LONG = x2 - x1;
    let mut deltay: LONG = y2 - y1;

    if deltax == 0 && deltay == 0 {
        return 1;
    }

    let xchange: LONG;
    let hx: LONG;
    if deltax < 0 {
        xchange = -1;
        deltax = -deltax;
        hx = x2 + 1;
    } else {
        xchange = 1;
        hx = x1;
    }

    let ychange: LONG;
    let vy: LONG;
    if deltay < 0 {
        ychange = -1;
        deltay = -deltay;
        vy = y2 + 1;
    } else {
        ychange = 1;
        vy = y1;
    }

    if (y1 == y2) && (cStyles == 0) {
        CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        let mut EnumMore: BOOL;
        loop {
            EnumMore = CLIPOBJ_bEnum(
                Clip,
                core::mem::size_of::<RECT_ENUM>() as ULONG,
                &mut RectEnum as *mut _ as *mut core::ffi::c_void,
            ) as i32;
            let mut i: ULONG = 0;
            while i < RectEnum.c && RectEnum.arcl[i as usize].top + Translate.y <= y1 {
                if y1 < RectEnum.arcl[i as usize].bottom + Translate.y
                    && RectEnum.arcl[i as usize].left + Translate.x <= hx + deltax
                    && hx < RectEnum.arcl[i as usize].right + Translate.x
                    && long_max(hx, RectEnum.arcl[i as usize].left + Translate.x)
                        < long_min(hx + deltax, RectEnum.arcl[i as usize].right + Translate.x)
                {
                    (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_HLine)(
                        OutputObj,
                        long_max(hx, RectEnum.arcl[i as usize].left + Translate.x),
                        long_min(hx + deltax, RectEnum.arcl[i as usize].right + Translate.x),
                        y1,
                        Pixel,
                    );
                }
                i += 1;
            }
            if EnumMore == 0 {
                break;
            }
        }
    } else if (x1 == x2) && (cStyles == 0) {
        CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_RIGHTDOWN, 0);
        let mut EnumMore: BOOL;
        loop {
            EnumMore = CLIPOBJ_bEnum(
                Clip,
                core::mem::size_of::<RECT_ENUM>() as ULONG,
                &mut RectEnum as *mut _ as *mut core::ffi::c_void,
            ) as i32;
            let mut i: ULONG = 0;
            while i < RectEnum.c {
                if RectEnum.arcl[i as usize].left + Translate.x <= x1
                    && x1 < RectEnum.arcl[i as usize].right + Translate.x
                    && RectEnum.arcl[i as usize].top + Translate.y <= vy + deltay
                    && vy < RectEnum.arcl[i as usize].bottom + Translate.y
                {
                    (DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize].DIB_VLine)(
                        OutputObj,
                        x1,
                        long_max(vy, RectEnum.arcl[i as usize].top + Translate.y),
                        long_min(vy + deltay, RectEnum.arcl[i as usize].bottom + Translate.y),
                        Pixel,
                    );
                }
                i += 1;
            }
            if EnumMore == 0 {
                break;
            }
        }
    } else {
        if 0 < xchange {
            if 0 < ychange {
                NWtoSE(OutputObj, Clip, pbo, x, y, deltax, deltay, &mut Translate);
            } else {
                SWtoNE(OutputObj, Clip, pbo, x, y, deltax, deltay, &mut Translate);
            }
        } else {
            if 0 < ychange {
                NEtoSW(OutputObj, Clip, pbo, x, y, deltax, deltay, &mut Translate);
            } else {
                SEtoNW(OutputObj, Clip, pbo, x, y, deltax, deltay, &mut Translate);
            }
        }
    }

    if !pcoPriv.is_null() {
        EngDeleteClip(pcoPriv);
    }

    let _ = RectEnum; // keep storage alive across the conditional code paths.
    IntEngLeave(&mut EnterLeave)
}

#[no_mangle]
pub unsafe extern "system" fn EngLineTo(
    DestObj: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x1: LONG,
    y1: LONG,
    x2: LONG,
    y2: LONG,
    RectBounds: *mut RECTL,
    mix: MIX,
) -> BOOL {
    eng_lineto_body(DestObj, Clip, pbo, x1, y1, x2, y2, RectBounds, mix)
}

// ---------------------------------------------------------------------------
// IntEngLineTo — internal APIENTRY (NTAPI).
// ---------------------------------------------------------------------------

unsafe fn int_eng_lineto_body(
    psoDest: *mut SURFOBJ,
    ClipObj: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x1: LONG,
    y1: LONG,
    x2: LONG,
    y2: LONG,
    RectBounds: *mut RECTL,
    Mix: MIX,
) -> BOOL {
    let psurfDest = (psoDest as *mut u8)
        .offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    let GdiBrush = (pbo as *mut u8)
        .offset(-(core::mem::offset_of!(EBRUSHOBJ, BrushObject) as isize)) as *mut EBRUSHOBJ;

    if ((*brushbody((*GdiBrush).pbrush)).flAttrs & BR_IS_NULL as FLONG) != 0 {
        return 1;
    }

    let mut ret: BOOL = 0;

    let b_outer: RECTL;
    if ClipObj.is_null() || (*ClipObj).iDComplexity as u32 == DC_TRIVIAL {
        b_outer = RECTL {
            left: 0,
            right: (*psoDest).sizlBitmap.cx,
            top: 0,
            bottom: (*psoDest).sizlBitmap.cy,
        };
    } else {
        b_outer = (*ClipObj).rclBounds;
    }

    if (x1 < b_outer.left && x2 < b_outer.left)
        || (b_outer.right <= x1 && b_outer.right <= x2)
        || (y1 < b_outer.top && y2 < b_outer.top)
        || (b_outer.bottom <= y1 && b_outer.bottom <= y2)
    {
        return 1;
    }

    let mut b = RECTL {
        left:   long_min(x1, x2),
        right:  long_max(x1, x2),
        top:    long_min(y1, y2),
        bottom: long_max(y1, y2),
    };
    if b.left == b.right {
        b.right += 1;
    }
    if b.top == b.bottom {
        b.bottom += 1;
    }

    if ((*psurfDest).flags & HOOK_LINETO as FLONG) != 0 {
        // GDIDEVFUNCS(psoDest).LineTo(...) — pull from the driver dispatch table.
        let pfn_opt = GDIDEVFUNCS(psoDest).LineTo;
        if let Some(pfn) = pfn_opt {
            ret = pfn(psoDest, ClipObj, pbo, x1, y1, x2, y2, &mut b, Mix);
        }
    }

    if ret == 0 {
        ret = EngLineTo(psoDest, ClipObj, pbo, x1, y1, x2, y2, RectBounds, Mix);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IntEngLineTo(
    psoDest: *mut SURFOBJ,
    ClipObj: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    x1: LONG,
    y1: LONG,
    x2: LONG,
    y2: LONG,
    RectBounds: *mut RECTL,
    Mix: MIX,
) -> BOOL {
    int_eng_lineto_body(psoDest, ClipObj, pbo, x1, y1, x2, y2, RectBounds, Mix)
}

// ---------------------------------------------------------------------------
// IntEngPolyline — internal APIENTRY (NTAPI).
// ---------------------------------------------------------------------------

unsafe fn int_eng_polyline_body(
    psoDest: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pt: LPPOINT,
    dCount: LONG,
    Mix: MIX,
) -> BOOL {
    let mut rect: RECTL = core::mem::zeroed();
    let mut ret: BOOL = 0;
    let mut i: LONG = 1;
    while i < dCount {
        let p0 = pt.offset((i - 1) as isize);
        let p1 = pt.offset(i as isize);
        rect.left   = long_min((*p0).x, (*p1).x);
        rect.top    = long_min((*p0).y, (*p1).y);
        rect.right  = long_max((*p0).x, (*p1).x);
        rect.bottom = long_max((*p0).y, (*p1).y);
        ret = IntEngLineTo(
            psoDest,
            Clip,
            pbo,
            (*p0).x,
            (*p0).y,
            (*p1).x,
            (*p1).y,
            &mut rect,
            Mix,
        );
        if ret == 0 {
            break;
        }
        i += 1;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn IntEngPolyline(
    psoDest: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pt: LPPOINT,
    dCount: LONG,
    Mix: MIX,
) -> BOOL {
    int_eng_polyline_body(psoDest, Clip, pbo, pt, dCount, Mix)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Line functions
//  * FILE:             win32ss/gdi/eng/lineto.c
//  * PROGRAMER:        ReactOS Team
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// static void FASTCALL
// TranslateRects(RECT_ENUM *RectEnum, POINTL* Translate)
// {
//     RECTL* CurrentRect;
//
//     if (0 != Translate->x || 0 != Translate->y)
//     {
//         for (CurrentRect = RectEnum->arcl; CurrentRect < RectEnum->arcl + RectEnum->c; CurrentRect++)
//         {
//             CurrentRect->left += Translate->x;
//             CurrentRect->right += Translate->x;
//             CurrentRect->top += Translate->y;
//             CurrentRect->bottom += Translate->y;
//         }
//     }
// }
//
// LONG
// HandleStyles(
//     BRUSHOBJ *pbo,
//     POINTL* Translate,
//     LONG x,
//     LONG y,
//     LONG deltax,
//     LONG deltay,
//     LONG dx,
//     LONG dy,
//     PULONG piStyle)
// {
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     PULONG pulStyles = pebo->pbrush->pStyle;
//     ULONG iStyle, cStyles = pebo->pbrush->dwStyleCount;
//     LONG diStyle, offStyle, lStyleMax;
//
//     if (cStyles > 0)
//     {
//         if (deltax > deltay)
//         {
//             offStyle = (- Translate->x) % pebo->pbrush->ulStyleSize;
//             diStyle = dx;
//             lStyleMax = x;
//         }
//         else
//         {
//             offStyle = (- Translate->y) % pebo->pbrush->ulStyleSize;
//             diStyle = dy;
//             lStyleMax = y;
//         }
//
//         /* Now loop until we have found the style index */
//         for (iStyle = 0; offStyle >= pulStyles[iStyle]; iStyle++)
//         {
//             offStyle -= pulStyles[iStyle];
//         }
//
//         if (diStyle > 0)
//         {
//             lStyleMax += pulStyles[iStyle] - offStyle;
//         }
//         else
//         {
//             lStyleMax -= offStyle + 1;
//         }
//     }
//     else
//     {
//         iStyle = 0;
//         lStyleMax = MAXLONG;
//     }
//
//     *piStyle = iStyle;
//     return lStyleMax;
// }
//
// /*
//  * Draw a line from top-left to bottom-right
//  */
// void FASTCALL
// NWtoSE(SURFOBJ* OutputObj, CLIPOBJ* Clip,
//        BRUSHOBJ* pbo, LONG x, LONG y, LONG deltax, LONG deltay,
//        POINTL* Translate)
// {
//     int i;
//     int error;
//     BOOLEAN EnumMore;
//     RECTL* ClipRect;
//     RECT_ENUM RectEnum;
//     ULONG Pixel = pbo->iSolidColor;
//     LONG delta;
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     PULONG pulStyles = pebo->pbrush->pStyle;
//     ULONG iStyle, cStyles = pebo->pbrush->dwStyleCount;
//     LONG lStyleMax;
//
//     lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, 1, 1, &iStyle);
//
//     CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//     EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//     TranslateRects(&RectEnum, Translate);
//     ClipRect = RectEnum.arcl;
//     delta = max(deltax, deltay);
//     i = 0;
//     error = delta >> 1;
//     while (i < delta && (ClipRect < RectEnum.arcl + RectEnum.c || EnumMore))
//     {
//         while ((ClipRect < RectEnum.arcl + RectEnum.c /* there's still a current clip rect */
//                 && (ClipRect->bottom <= y             /* but it's above us */
//                     || (ClipRect->top <= y && ClipRect->right <= x))) /* or to the left of us */
//                 || EnumMore)                           /* no current clip rect, but rects left */
//         {
//             /* Skip to the next clip rect */
//             if (RectEnum.arcl + RectEnum.c <= ClipRect)
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//                 TranslateRects(&RectEnum, Translate);
//                 ClipRect = RectEnum.arcl;
//             }
//             else
//             {
//                 ClipRect++;
//             }
//         }
//         if (ClipRect < RectEnum.arcl + RectEnum.c) /* If there's no current clip rect we're done */
//         {
//             if ((ClipRect->left <= x && ClipRect->top <= y) && ((iStyle & 1) == 0))
//             {
//                 DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_PutPixel(
//                     OutputObj, x, y, Pixel);
//             }
//             if (deltax < deltay)
//             {
//                 y++;
//                 if (y == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle + 1) % cStyles;
//                     lStyleMax = y + pulStyles[iStyle];
//                 }
//                 error = error + deltax;
//                 if (deltay <= error)
//                 {
//                     x++;
//                     error = error - deltay;
//                 }
//             }
//             else
//             {
//                 x++;
//                 if (x == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle + 1) % cStyles;
//                     lStyleMax = x + pulStyles[iStyle];
//                 }
//                 error = error + deltay;
//                 if (deltax <= error)
//                 {
//                     y++;
//                     error = error - deltax;
//                 }
//             }
//             i++;
//         }
//     }
// }
//
// void FASTCALL
// SWtoNE(SURFOBJ* OutputObj, CLIPOBJ* Clip,
//        BRUSHOBJ* pbo, LONG x, LONG y, LONG deltax, LONG deltay,
//        POINTL* Translate)
// {
//     int i;
//     int error;
//     BOOLEAN EnumMore;
//     RECTL* ClipRect;
//     RECT_ENUM RectEnum;
//     ULONG Pixel = pbo->iSolidColor;
//     LONG delta;
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     PULONG pulStyles = pebo->pbrush->pStyle;
//     ULONG iStyle, cStyles = pebo->pbrush->dwStyleCount;
//     LONG lStyleMax;
//
//     lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, 1, -1, &iStyle);
//
//     CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_RIGHTUP, 0);
//     EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//     TranslateRects(&RectEnum, Translate);
//     ClipRect = RectEnum.arcl;
//     delta = max(deltax, deltay);
//     i = 0;
//     error = delta >> 1;
//     while (i < delta && (ClipRect < RectEnum.arcl + RectEnum.c || EnumMore))
//     {
//         while ((ClipRect < RectEnum.arcl + RectEnum.c
//                 && (y < ClipRect->top
//                     || (y < ClipRect->bottom && ClipRect->right <= x)))
//                 || EnumMore)
//         {
//             if (RectEnum.arcl + RectEnum.c <= ClipRect)
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//                 TranslateRects(&RectEnum, Translate);
//                 ClipRect = RectEnum.arcl;
//             }
//             else
//             {
//                 ClipRect++;
//             }
//         }
//         if (ClipRect < RectEnum.arcl + RectEnum.c)
//         {
//             if ((ClipRect->left <= x && y < ClipRect->bottom) && ((iStyle & 1) == 0))
//             {
//                 DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_PutPixel(
//                     OutputObj, x, y, Pixel);
//             }
//             if (deltax < deltay)
//             {
//                 y--;
//                 if (y == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle - 1) % cStyles;
//                     lStyleMax = y - pulStyles[iStyle];
//                 }
//                 error = error + deltax;
//                 if (deltay <= error)
//                 {
//                     x++;
//                     error = error - deltay;
//                 }
//             }
//             else
//             {
//                 x++;
//                 if (x == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle + 1) % cStyles;
//                     lStyleMax = x + pulStyles[iStyle];
//                 }
//                 error = error + deltay;
//                 if (deltax <= error)
//                 {
//                     y--;
//                     error = error - deltax;
//                 }
//             }
//             i++;
//         }
//     }
// }
//
// void FASTCALL
// NEtoSW(SURFOBJ* OutputObj, CLIPOBJ* Clip,
//        BRUSHOBJ* pbo, LONG x, LONG y, LONG deltax, LONG deltay,
//        POINTL* Translate)
// {
//     int i;
//     int error;
//     BOOLEAN EnumMore;
//     RECTL* ClipRect;
//     RECT_ENUM RectEnum;
//     ULONG Pixel = pbo->iSolidColor;
//     LONG delta;
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     PULONG pulStyles = pebo->pbrush->pStyle;
//     ULONG iStyle, cStyles = pebo->pbrush->dwStyleCount;
//     LONG lStyleMax;
//
//     lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, -1, 1, &iStyle);
//
//     CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_LEFTDOWN, 0);
//     EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//     TranslateRects(&RectEnum, Translate);
//     ClipRect = RectEnum.arcl;
//     delta = max(deltax, deltay);
//     i = 0;
//     error = delta >> 1;
//     while (i < delta && (ClipRect < RectEnum.arcl + RectEnum.c || EnumMore))
//     {
//         while ((ClipRect < RectEnum.arcl + RectEnum.c
//                 && (ClipRect->bottom <= y
//                     || (ClipRect->top <= y && x < ClipRect->left)))
//                 || EnumMore)
//         {
//             if (RectEnum.arcl + RectEnum.c <= ClipRect)
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//                 TranslateRects(&RectEnum, Translate);
//                 ClipRect = RectEnum.arcl;
//             }
//             else
//             {
//                 ClipRect++;
//             }
//         }
//         if (ClipRect < RectEnum.arcl + RectEnum.c)
//         {
//             if ((x < ClipRect->right && ClipRect->top <= y) && ((iStyle & 1) == 0))
//             {
//                 DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_PutPixel(
//                     OutputObj, x, y, Pixel);
//             }
//             if (deltax < deltay)
//             {
//                 y++;
//                 if (y == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle + 1) % cStyles;
//                     lStyleMax = y + pulStyles[iStyle];
//                 }
//                 error = error + deltax;
//                 if (deltay <= error)
//                 {
//                     x--;
//                     error = error - deltay;
//                 }
//             }
//             else
//             {
//                 x--;
//                 if (x == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle - 1) % cStyles;
//                     lStyleMax = x - pulStyles[iStyle];
//                 }
//                 error = error + deltay;
//                 if (deltax <= error)
//                 {
//                     y++;
//                     error = error - deltax;
//                 }
//             }
//             i++;
//         }
//     }
// }
//
// void FASTCALL
// SEtoNW(SURFOBJ* OutputObj, CLIPOBJ* Clip,
//        BRUSHOBJ* pbo, LONG x, LONG y, LONG deltax, LONG deltay,
//        POINTL* Translate)
// {
//     int i;
//     int error;
//     BOOLEAN EnumMore;
//     RECTL* ClipRect;
//     RECT_ENUM RectEnum;
//     ULONG Pixel = pbo->iSolidColor;
//     LONG delta;
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     PULONG pulStyles = pebo->pbrush->pStyle;
//     ULONG iStyle, cStyles = pebo->pbrush->dwStyleCount;
//     LONG lStyleMax;
//
//     lStyleMax = HandleStyles(pbo, Translate, x, y, deltax, deltay, -1, -1, &iStyle);
//
//     CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_LEFTUP, 0);
//     EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//     TranslateRects(&RectEnum, Translate);
//     ClipRect = RectEnum.arcl;
//     delta = max(deltax, deltay);
//     i = 0;
//     error = delta >> 1;
//     while (i < delta && (ClipRect < RectEnum.arcl + RectEnum.c || EnumMore))
//     {
//         while ((ClipRect < RectEnum.arcl + RectEnum.c
//                 && (y < ClipRect->top
//                     || (y < ClipRect->bottom && x < ClipRect->left)))
//                 || EnumMore)
//         {
//             if (RectEnum.arcl + RectEnum.c <= ClipRect)
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//                 TranslateRects(&RectEnum, Translate);
//                 ClipRect = RectEnum.arcl;
//             }
//             else
//             {
//                 ClipRect++;
//             }
//         }
//         if (ClipRect < RectEnum.arcl + RectEnum.c)
//         {
//             if ((x < ClipRect->right && y < ClipRect->bottom) && ((iStyle & 1) == 0))
//             {
//                 DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_PutPixel(
//                     OutputObj, x, y, Pixel);
//             }
//             if (deltax < deltay)
//             {
//                 y--;
//                 if (y == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle - 1) % cStyles;
//                     lStyleMax = y - pulStyles[iStyle];
//                 }
//                 error = error + deltax;
//                 if (deltay <= error)
//                 {
//                     x--;
//                     error = error - deltay;
//                 }
//             }
//             else
//             {
//                 x--;
//                 if (x == lStyleMax)
//                 {
//                     ASSERT(cStyles);
//                     iStyle = (iStyle - 1) % cStyles;
//                     lStyleMax = x - pulStyles[iStyle];
//                 }
//                 error = error + deltay;
//                 if (deltax <= error)
//                 {
//                     y--;
//                     error = error - deltax;
//                 }
//             }
//             i++;
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// BOOL APIENTRY
// EngLineTo(
//     _Inout_ SURFOBJ *DestObj,
//     _In_ CLIPOBJ *Clip,
//     _In_ BRUSHOBJ *pbo,
//     _In_ LONG x1,
//     _In_ LONG y1,
//     _In_ LONG x2,
//     _In_ LONG y2,
//     _In_opt_ RECTL *RectBounds,
//     _In_ MIX mix)
// {
//     LONG x, y, deltax, deltay, xchange, ychange, hx, vy;
//     ULONG i;
//     ULONG Pixel = pbo->iSolidColor;
//     SURFOBJ *OutputObj;
//     RECTL DestRect;
//     POINTL Translate;
//     INTENG_ENTER_LEAVE EnterLeave;
//     RECT_ENUM RectEnum;
//     BOOL EnumMore;
//     CLIPOBJ *pcoPriv = NULL;
//     PEBRUSHOBJ pebo = (PEBRUSHOBJ)pbo;
//     ULONG cStyles = pebo->pbrush->dwStyleCount;
//
//     if (x1 < x2)
//     {
//         DestRect.left = x1;
//         DestRect.right = x2;
//     }
//     else
//     {
//         DestRect.left = x2;
//         DestRect.right = x1 + 1;
//     }
//     if (y1 < y2)
//     {
//         DestRect.top = y1;
//         DestRect.bottom = y2;
//     }
//     else
//     {
//         DestRect.top = y2;
//         DestRect.bottom = y1 + 1;
//     }
//
//     if (! IntEngEnter(&EnterLeave, DestObj, &DestRect, FALSE, &Translate, &OutputObj))
//     {
//         return FALSE;
//     }
//
//     if (!Clip)
//     {
//         Clip = pcoPriv = EngCreateClip();
//         if (!Clip)
//         {
//             return FALSE;
//         }
//         IntEngUpdateClipRegion((XCLIPOBJ*)Clip, 0, 0, RectBounds);
//     }
//
//     x1 += Translate.x;
//     x2 += Translate.x;
//     y1 += Translate.y;
//     y2 += Translate.y;
//
//     x = x1;
//     y = y1;
//     deltax = x2 - x1;
//     deltay = y2 - y1;
//
//     if (0 == deltax && 0 == deltay)
//     {
//         return TRUE;
//     }
//
//     if (deltax < 0)
//     {
//         xchange = -1;
//         deltax = - deltax;
//         hx = x2 + 1;
//     }
//     else
//     {
//         xchange = 1;
//         hx = x1;
//     }
//
//     if (deltay < 0)
//     {
//         ychange = -1;
//         deltay = - deltay;
//         vy = y2 + 1;
//     }
//     else
//     {
//         ychange = 1;
//         vy = y1;
//     }
//
//     if ((y1 == y2) && (cStyles == 0))
//     {
//         CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//         do
//         {
//             EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//             for (i = 0; i < RectEnum.c && RectEnum.arcl[i].top + Translate.y <= y1; i++)
//             {
//                 if (y1 < RectEnum.arcl[i].bottom + Translate.y &&
//                         RectEnum.arcl[i].left + Translate.x <= hx + deltax &&
//                         hx < RectEnum.arcl[i].right + Translate.x &&
//                         max(hx, RectEnum.arcl[i].left + Translate.x) <
//                         min(hx + deltax, RectEnum.arcl[i].right + Translate.x))
//                 {
//                     DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_HLine(
//                         OutputObj,
//                         max(hx, RectEnum.arcl[i].left + Translate.x),
//                         min(hx + deltax, RectEnum.arcl[i].right + Translate.x),
//                         y1, Pixel);
//                 }
//             }
//         }
//         while (EnumMore);
//     }
//     else if ((x1 == x2) && (cStyles == 0))
//     {
//         CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_RIGHTDOWN, 0);
//         do
//         {
//             EnumMore = CLIPOBJ_bEnum(Clip, (ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//             for (i = 0; i < RectEnum.c; i++)
//             {
//                 if (RectEnum.arcl[i].left + Translate.x <= x1 &&
//                         x1 < RectEnum.arcl[i].right + Translate.x &&
//                         RectEnum.arcl[i].top + Translate.y <= vy + deltay &&
//                         vy < RectEnum.arcl[i].bottom + Translate.y)
//                 {
//                     DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_VLine(
//                         OutputObj, x1,
//                         max(vy, RectEnum.arcl[i].top + Translate.y),
//                         min(vy + deltay, RectEnum.arcl[i].bottom + Translate.y),
//                         Pixel);
//                 }
//             }
//         }
//         while (EnumMore);
//     }
//     else
//     {
//         if (0 < xchange)
//         {
//             if (0 < ychange)
//             {
//                 NWtoSE(OutputObj, Clip, pbo, x, y, deltax, deltay, &Translate);
//             }
//             else
//             {
//                 SWtoNE(OutputObj, Clip, pbo, x, y, deltax, deltay, &Translate);
//             }
//         }
//         else
//         {
//             if (0 < ychange)
//             {
//                 NEtoSW(OutputObj, Clip, pbo, x, y, deltax, deltay, &Translate);
//             }
//             else
//             {
//                 SEtoNW(OutputObj, Clip, pbo, x, y, deltax, deltay, &Translate);
//             }
//         }
//     }
//
//     if (pcoPriv)
//     {
//         EngDeleteClip(pcoPriv);
//     }
//
//     return IntEngLeave(&EnterLeave);
// }
//
// BOOL APIENTRY
// IntEngLineTo(SURFOBJ *psoDest,
//              CLIPOBJ *ClipObj,
//              BRUSHOBJ *pbo,
//              LONG x1,
//              LONG y1,
//              LONG x2,
//              LONG y2,
//              RECTL *RectBounds,
//              MIX Mix)
// {
//     BOOLEAN ret;
//     SURFACE *psurfDest;
//     PEBRUSHOBJ GdiBrush;
//     RECTL b;
//
//     ASSERT(psoDest);
//     psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//     ASSERT(psurfDest);
//
//     GdiBrush = CONTAINING_RECORD(
//                    pbo,
//                    EBRUSHOBJ,
//                    BrushObject);
//     ASSERT(GdiBrush);
//     ASSERT(GdiBrush->pbrush);
//
//     if (GdiBrush->pbrush->flAttrs & BR_IS_NULL)
//         return TRUE;
//
//     /* No success yet */
//     ret = FALSE;
//
//     /* Clip lines totally outside the clip region. This is not done as an
//      * optimization (there are very few lines drawn outside the region) but
//      * as a workaround for what seems to be a problem in the CL54XX driver */
//     if (NULL == ClipObj || DC_TRIVIAL == ClipObj->iDComplexity)
//     {
//         b.left = 0;
//         b.right = psoDest->sizlBitmap.cx;
//         b.top = 0;
//         b.bottom = psoDest->sizlBitmap.cy;
//     }
//     else
//     {
//         b = ClipObj->rclBounds;
//     }
//     if ((x1 < b.left && x2 < b.left) || (b.right <= x1 && b.right <= x2) ||
//             (y1 < b.top && y2 < b.top) || (b.bottom <= y1 && b.bottom <= y2))
//     {
//         return TRUE;
//     }
//
//     b.left = min(x1, x2);
//     b.right = max(x1, x2);
//     b.top = min(y1, y2);
//     b.bottom = max(y1, y2);
//     if (b.left == b.right) b.right++;
//     if (b.top == b.bottom) b.bottom++;
//
//     if (psurfDest->flags & HOOK_LINETO)
//     {
//         /* Call the driver's DrvLineTo */
//         ret = GDIDEVFUNCS(psoDest).LineTo(
//                   psoDest, ClipObj, pbo, x1, y1, x2, y2, &b, Mix);
//     }
//
// #if 0
//     if (! ret && (psurfDest->flags & HOOK_STROKEPATH))
//     {
//         /* FIXME: Emulate LineTo using drivers DrvStrokePath and set ret on success */
//     }
// #endif
//
//     if (! ret)
//     {
//         ret = EngLineTo(psoDest, ClipObj, pbo, x1, y1, x2, y2, RectBounds, Mix);
//     }
//
//     return ret;
// }
//
// BOOL APIENTRY
// IntEngPolyline(SURFOBJ *psoDest,
//                CLIPOBJ *Clip,
//                BRUSHOBJ *pbo,
//                CONST LPPOINT  pt,
//                LONG dCount,
//                MIX Mix)
// {
//     LONG i;
//     RECTL rect;
//     BOOL ret = FALSE;
//
//     // Draw the Polyline with a call to IntEngLineTo for each segment.
//     for (i = 1; i < dCount; i++)
//     {
//         rect.left = min(pt[i-1].x, pt[i].x);
//         rect.top = min(pt[i-1].y, pt[i].y);
//         rect.right = max(pt[i-1].x, pt[i].x);
//         rect.bottom = max(pt[i-1].y, pt[i].y);
//         ret = IntEngLineTo(psoDest,
//                            Clip,
//                            pbo,
//                            pt[i-1].x,
//                            pt[i-1].y,
//                            pt[i].x,
//                            pt[i].y,
//                            &rect,
//                            Mix);
//         if (!ret)
//         {
//             break;
//         }
//     }
//
//     return ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
