use win32k_sys::win32k::*;

extern "system" {
    fn SafeProbeForRead(address: *const core::ffi::c_void, length: usize, alignment: u32) -> i32;
    fn EBRUSHOBJ_psoMask(pebo: *mut EBRUSHOBJ) -> *mut SURFOBJ;
    fn EBRUSHOBJ_psoPattern(pebo: *mut EBRUSHOBJ) -> *mut SURFOBJ;
    fn CLIPOBJ_bEnum(ClipRegion: *mut CLIPOBJ, size: u32, RectEnum: *mut core::ffi::c_void) -> i32;
    fn CLIPOBJ_cEnumStart(ClipRegion: *mut CLIPOBJ, bAll: u32, iType: u32, iDirection: u32, cLimit: u32) -> u32;
    fn XLATEOBJ_iXlate(pxlo: *mut XLATEOBJ, iColor: u32) -> u32;
}

// C macros from win32ss/gdi/ntgdi/brush.h:
//   #define BRUSHOBJ_psoMask(pbo)    EBRUSHOBJ_psoMask(CONTAINING_RECORD(pbo, EBRUSHOBJ, BrushObject))
//   #define BRUSHOBJ_psoPattern(pbo) EBRUSHOBJ_psoPattern(CONTAINING_RECORD(pbo, EBRUSHOBJ, BrushObject))
// `BrushObject` is the first field of `EBRUSHOBJ`, so CONTAINING_RECORD is a no-op cast.
#[inline(always)]
unsafe fn BRUSHOBJ_psoMask(pbo: *mut BRUSHOBJ) -> *mut SURFOBJ {
    EBRUSHOBJ_psoMask(pbo as *mut EBRUSHOBJ)
}
#[inline(always)]
unsafe fn BRUSHOBJ_psoPattern(pbo: *mut BRUSHOBJ) -> *mut SURFOBJ {
    EBRUSHOBJ_psoPattern(pbo as *mut EBRUSHOBJ)
}

// RECTL_v/bIntersectRect are FASTCALL in C (rect.c: @Name@N decoration on i386).
// RECTL_bClipRectBySize is FORCEINLINE — inline-expand below.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(dest: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> u32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(dest: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> u32;
}

// Inline-expansion of FORCEINLINE BOOL RECTL_bClipRectBySize (win32ss/gdi/ntgdi/rect.h:71).
#[inline(always)]
unsafe fn RECTL_bClipRectBySize(prcl_dst: *mut RECTL, prcl_src: *mut RECTL, pszl: *const tagSIZE) -> u32 {
    (*prcl_dst).left = core::cmp::max((*prcl_src).left, 0);
    (*prcl_dst).top = core::cmp::max((*prcl_src).top, 0);
    (*prcl_dst).right = core::cmp::min((*prcl_src).right, (*pszl).cx);
    (*prcl_dst).bottom = core::cmp::min((*prcl_src).bottom, (*pszl).cy);
    // !RECTL_bIsEmptyRect(prcl_dst)
    if (*prcl_dst).left >= (*prcl_dst).right || (*prcl_dst).top >= (*prcl_dst).bottom {
        0
    } else {
        1
    }
}

extern "C" {
    fn DIB_DoRop(rop: u32, dst: u32, src: u32, pat: u32) -> u32;
    // Shadow win32k-sys's `[T; 0]` flexible-array decl so [] indexing
    // works. See lineto.rs for the same pattern. The underlying linker
    // symbol is `DibFunctionsForBitmapFormat` (a real array in C).
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11];
}

// C macros from wingdi.h — inline-expand instead of calling a non-existent fn.
#[inline(always)]
unsafe fn GetRValue(rgb: u32) -> u32 { rgb & 0xFF }
#[inline(always)]
unsafe fn GetGValue(rgb: u32) -> u32 { (rgb >> 8) & 0xFF }
#[inline(always)]
unsafe fn GetBValue(rgb: u32) -> u32 { (rgb >> 16) & 0xFF }
#[inline(always)]
unsafe fn RGB(r: u32, g: u32, b: u32) -> u32 { (r & 0xFF) | ((g & 0xFF) << 8) | ((b & 0xFF) << 16) }
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}
// C macro from win32ss/gdi/dib/dib.h:136:
//   #define DIB_GetSource(SourceSurf,sx,sy,ColorTranslation) \
//     XLATEOBJ_iXlate(ColorTranslation, \
//       DibFunctionsForBitmapFormat[SourceSurf->iBitmapFormat].DIB_GetPixel(SourceSurf, sx, sy))
#[inline(always)]
unsafe fn DIB_GetSource(pso: *mut SURFOBJ, x: i32, y: i32, pxlo: *mut XLATEOBJ) -> u32 {
    let fmt = (*pso).iBitmapFormat as usize;
    let getpx = DibFunctionsForBitmapFormat[fmt].DIB_GetPixel;
    let pixel = match getpx {
        Some(f) => f(pso, x, y),
        None => 0,
    };
    XLATEOBJ_iXlate(pxlo, pixel)
}

// ROP4 helpers (macros in inteng.h)
#[inline(always)]
unsafe fn ROP4_USES_DEST(rop4: u32) -> u32 {
    if ((rop4 & 0xAAAA) >> 1) != (rop4 & 0x5555) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_USES_SOURCE(rop4: u32) -> u32 {
    if ((rop4 & 0xCCCC) >> 2) != (rop4 & 0x3333) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_USES_PATTERN(rop4: u32) -> u32 {
    if ((rop4 & 0xF0F0) >> 4) != (rop4 & 0x0F0F) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_USES_MASK(rop4: u32) -> u32 {
    if ((rop4 & 0xFF00) >> 8) != (rop4 & 0x00FF) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_FGND(rop4: u32) -> u32 { rop4 & 0x00FF }
#[inline(always)]
unsafe fn ROP4_BKGND(rop4: u32) -> u32 { (rop4 & 0xFF00) >> 8 }
#[inline(always)]
unsafe fn NT_SUCCESS(s: i32) -> bool { s >= 0 }
#[inline(always)]
unsafe fn _rotr8(v: u8, n: u32) -> u8 { v.rotate_right(n) }

// Inline-expand of `GDIDEVFUNCS(SurfObj)` from `win32ss/gdi/eng/surface.h:106`:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
// Mirrors copybits.rs::gdidev_copybits. Returns a reference into the
// DriverFunctions union so callers can pick the per-fn entry directly.
#[inline(always)]
unsafe fn GDIDEVFUNCS<'a>(pso: *mut SURFOBJ) -> &'a DRIVER_FUNCTIONS {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    &(*pdev).__bindgen_anon_1.DriverFunctions
}

// ROP4 constants from inteng.h
const ROP4_NOOP: u32 = 0xAA | (0xAA << 8);          // 0xAAAA
const ROP4_MASK: u32 = 0xCC | (0xAA << 8);          // 0xAACC
const ROP4_MASKPAINT: u32 = 0xF0 | (0xAA << 8);     // 0xAAF0
const R3_OPINDEX_SRCCOPY: u32 = 0xCC;
const R3_OPINDEX_PATCOPY: u32 = 0xF0;

// ---- file-scope preamble (translated) ----
//
// `gxcoTrivial` is defined in C as an XCLIPOBJ — a `_CLIPOBJ` followed by
// `_RWNDOBJ` followed by 13 trailing words. bindgen's `_XCLIPOBJ` hides
// the first 48 bytes as `__bindgen_padding_0: [u32; 12]` and exposes the
// remaining fields, so we can't use field-named init for the CLIPOBJ
// prefix. Define a layout-compatible local struct with the same size +
// initialize the CLIPOBJ prefix explicitly. Other crates extern this as
// `gxcoTrivial: CLIPOBJ` and only read the CLIPOBJ prefix bytes.
#[repr(C)]
struct GxcoTrivialLayout {
    // CLIPOBJ (24 bytes)
    iUniq: u32,
    rclBounds: RECTL,
    iDComplexity: u8,
    iFComplexity: u8,
    iMode: u8,
    fjOptions: u8,
    // RWNDOBJ (24 bytes)
    pvConsumer: *mut core::ffi::c_void,
    rclClient: RECTL,
    psoOwner: *mut SURFOBJ,
    // XCLIPOBJ trailer (matches bindgen _XCLIPOBJ fields after padding)
    pClipRgn: *mut core::ffi::c_void,
    rclClipRgn: RECTL,
    Rects: *mut RECTL,
    cScan: u32,
    reserved: u32,
    EnumPos: u32,
    lscnSize: i32,
    EnumMax: u32,
    iDirection: u32,
    iType: u32,
    reserved1: u32,
    lUpDown: i32,
    reserved2: u32,
    bAll: i32,
    RectCount: u32,
    pDDA: *mut core::ffi::c_void,
}
unsafe impl Sync for GxcoTrivialLayout {}

#[no_mangle]
pub static mut gxcoTrivial: GxcoTrivialLayout = GxcoTrivialLayout {
    iUniq: 0,
    rclBounds: RECTL {
        left: LONG_MIN as i32,
        top: LONG_MIN as i32,
        right: LONG_MAX as i32,
        bottom: LONG_MAX as i32,
    },
    iDComplexity: DC_TRIVIAL as u8,
    iFComplexity: FC_RECT as u8,
    iMode: TC_RECTANGLES as u8,
    fjOptions: 0,
    pvConsumer: core::ptr::null_mut(),
    rclClient: RECTL { left: 0, top: 0, right: 0, bottom: 0 },
    psoOwner: core::ptr::null_mut(),
    pClipRgn: core::ptr::null_mut(),
    rclClipRgn: RECTL { left: 0, top: 0, right: 0, bottom: 0 },
    Rects: core::ptr::null_mut(),
    cScan: 0,
    reserved: 0,
    EnumPos: 0,
    lscnSize: 0,
    EnumMax: 0,
    iDirection: 0,
    iType: 0,
    reserved1: 0,
    lUpDown: 0,
    reserved2: 0,
    bAll: 0,
    RectCount: 0,
    pDDA: core::ptr::null_mut(),
};

pub type PBLTRECTFUNC = Option<
    unsafe extern "system" fn(
        OutputObj: *mut SURFOBJ,
        InputObj: *mut SURFOBJ,
        Mask: *mut SURFOBJ,
        ColorTranslation: *mut XLATEOBJ,
        OutputRect: *mut RECTL,
        InputPoint: *mut POINTL,
        MaskOrigin: *mut POINTL,
        pbo: *mut BRUSHOBJ,
        BrushOrigin: *mut POINTL,
        Rop4: ROP4,
    ) -> BOOLEAN,
>;

unsafe extern "system" fn BltMask(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    ColorTranslation: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    pptlSource: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    Rop4: ROP4,
) -> u8 {
    let mut x: i32;
    let mut y: i32;
    let cx: i32;
    let cy: i32;
    let mut pjMskLine: PBYTE;
    let mut pjMskCurrent: PBYTE;
    let fjMaskBit0: u8;
    let mut fjMaskBit: u8;
    let mut psoPattern: *mut SURFOBJ;
    let mut PatternWidth: u32 = 0;
    let mut PatternHeight: u32 = 0;
    let mut PatternX0: i32 = 0;
    let mut PatternX: i32 = 0;
    let mut PatternY: i32 = 0;
    let mut SrcX: i32 = 0;
    let mut SrcY: i32 = 0;
    let mut fnDest_PutPixel: PFN_DIB_PutPixel = None;
    let mut fnPattern_GetPixel: PFN_DIB_GetPixel = None;
    let mut fnSrc_GetPixel: PFN_DIB_GetPixel = None;
    let fnDest_GetPixel: PFN_DIB_GetPixel;
    let mut Pattern: u32 = 0;
    let mut Source: u32 = 0;
    let mut Dest: u32 = 0;
    let fgndRop: u32;
    let bkgndRop: u32;
    let mut Rop4_local: ROP4;

    fgndRop = ROP4_FGND(Rop4);
    bkgndRop = ROP4_BKGND(Rop4);

    if !pbo.is_null() && (*pbo).iSolidColor == 0xFFFFFFFF {
        psoPattern = BRUSHOBJ_psoPattern(pbo);
        if !psoPattern.is_null() {
            PatternWidth = (*psoPattern).sizlBitmap.cx as u32;
            PatternHeight = (*psoPattern).sizlBitmap.cy as u32;
            fnPattern_GetPixel = DibFunctionsForBitmapFormat[(*psoPattern).iBitmapFormat as usize].DIB_GetPixel;
        }
    } else {
        psoPattern = core::ptr::null_mut();
    }

    cx = (*prclDest).right - (*prclDest).left;
    cy = (*prclDest).bottom - (*prclDest).top;
    if ((*pptlMask).x + cx > (*psoMask).sizlBitmap.cx) ||
       ((*pptlMask).y + cy > (*psoMask).sizlBitmap.cy) {
        return 0u8;
    }

    pjMskLine = ((*psoMask).pvScan0 as PBYTE)
        .offset(((*pptlMask).y * (*psoMask).lDelta) as isize)
        .offset(((*pptlMask).x >> 3) as isize);
    fjMaskBit0 = 0x80 >> ((*pptlMask).x & 0x07);

    fnDest_PutPixel = DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize].DIB_PutPixel;
    fnDest_GetPixel = DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize].DIB_GetPixel;

    if !psoSource.is_null() {
        fnSrc_GetPixel = DibFunctionsForBitmapFormat[(*psoSource).iBitmapFormat as usize].DIB_GetPixel;
        SrcY = (*pptlSource).y;
        SrcX = (*pptlSource).x;
    }

    if !psoPattern.is_null() {
        PatternY = ((*prclDest).top - (*pptlBrush).y) % (PatternHeight as i32);
        if PatternY < 0 {
            PatternY += PatternHeight as i32;
        }
        PatternX0 = ((*prclDest).left - (*pptlBrush).x) % (PatternWidth as i32);
        if PatternX0 < 0 {
            PatternX0 += PatternWidth as i32;
        }
        PatternX = PatternX0;
    } else {
        Pattern = if !pbo.is_null() { (*pbo).iSolidColor } else { 0 };
    }

    y = (*prclDest).top;
    while y < (*prclDest).bottom {
        pjMskCurrent = pjMskLine;
        fjMaskBit = fjMaskBit0;

        x = (*prclDest).left;
        while x < (*prclDest).right {
            Rop4_local = if (*pjMskCurrent & fjMaskBit) != 0 { fgndRop } else { bkgndRop };

            if !psoPattern.is_null() {
                if ROP4_USES_PATTERN(Rop4_local) != 0 {
                    Pattern = fnPattern_GetPixel.unwrap()(psoPattern, PatternX, PatternY);
                }
                PatternX += 1;
                PatternX %= PatternWidth as i32;
            }

            if !psoSource.is_null() {
                if ROP4_USES_SOURCE(Rop4_local) != 0 {
                    Source = XLATEOBJ_iXlate(
                        ColorTranslation,
                        fnSrc_GetPixel.unwrap()(psoSource, SrcX, SrcY)
                    );
                }
                SrcX += 1;
            }

            if ROP4_USES_DEST(Rop4_local) != 0 {
                Dest = fnDest_GetPixel.unwrap()(psoDest, x, y);
            }

            fnDest_PutPixel.unwrap()(
                psoDest,
                x,
                y,
                DIB_DoRop(Rop4_local, Dest, Source, Pattern)
            );

            fjMaskBit = _rotr8(fjMaskBit, 1);
            pjMskCurrent = pjMskCurrent.offset((fjMaskBit >> 7) as isize);

            x += 1;
        }

        pjMskLine = pjMskLine.offset((*psoMask).lDelta as isize);
        if !psoPattern.is_null() {
            PatternY += 1;
            PatternY %= PatternHeight as i32;
            PatternX = PatternX0;
        }
        if !psoSource.is_null() {
            SrcY += 1;
            SrcX = (*pptlSource).x;
        }

        y += 1;
    }

    1u8
}

unsafe extern "system" fn BltPatCopy(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: *mut RECTL,
    SourcePoint: *mut POINTL,
    MaskPoint: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushPoint: *mut POINTL,
    Rop4: DWORD,
) -> BOOLEAN {
    let color: ULONG = if !pbo.is_null() {
        (*pbo).iSolidColor
    } else {
        0
    };

    DibFunctionsForBitmapFormat[(*Dest).iBitmapFormat as usize].DIB_ColorFill.unwrap()(Dest, DestRect, color);

    1u8
}

unsafe extern "system" fn CallDibBitBlt(
    OutputObj: *mut SURFOBJ,
    InputObj: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ColorTranslation: *mut XLATEOBJ,
    OutputRect: *mut RECTL,
    InputPoint: *mut POINTL,
    MaskOrigin: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
    Rop4: ROP4,
) -> u8 {
    let mut BltInfo: BLTINFO = core::mem::zeroed();
    let mut psoPattern: *mut SURFOBJ;
    let Result: u8;

    BltInfo.DestSurface = OutputObj;
    BltInfo.SourceSurface = InputObj;
    BltInfo.PatternSurface = core::ptr::null_mut();
    BltInfo.XlateSourceToDest = ColorTranslation;
    BltInfo.DestRect = *OutputRect;
    BltInfo.SourcePoint = *InputPoint;

    if (Rop4 & 0xFF) == R3_OPINDEX_SRCCOPY {
        return DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize]
            .DIB_BitBltSrcCopy.unwrap()(core::ptr::addr_of_mut!(BltInfo));
    }

    BltInfo.Brush = pbo;
    BltInfo.BrushOrigin = *BrushOrigin;
    BltInfo.Rop4 = Rop4;

    if ROP4_USES_PATTERN(Rop4) != 0 && !pbo.is_null() && (*pbo).iSolidColor == 0xFFFFFFFF {
        psoPattern = BRUSHOBJ_psoPattern(pbo);
        if !psoPattern.is_null() {
            BltInfo.PatternSurface = psoPattern;
        } else {
        }
    } else {
        psoPattern = core::ptr::null_mut();
    }

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(BltInfo.DestRect));

    Result = DibFunctionsForBitmapFormat[(*OutputObj).iBitmapFormat as usize]
        .DIB_BitBlt.unwrap()(core::ptr::addr_of_mut!(BltInfo));

    return Result;
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    Rop4: ROP4,
) -> i32 {
    if crate::safe_user_io::umpd_call_refused() {
        return 0;
    }
    let mut rclTrg: RECTL = core::mem::zeroed();
    let mut ptlSrc: POINTL = core::mem::zeroed();
    let mut ptlMask: POINTL = core::mem::zeroed();
    let mut ptlBrush: POINTL = core::mem::zeroed();
    let mut Status: i32;

    Status = SafeProbeForRead(prclTrg as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(pptlSrc as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(pptlMask as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if NT_SUCCESS(Status) {
        Status = SafeProbeForRead(pptlBrush as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
    }
    if !NT_SUCCESS(Status) {
        return 0;
    }
    // The probes above VALIDATE only; each capture is the access, so every one of
    // them takes the same exit a failed probe takes.
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(rclTrg) as *mut core::ffi::c_void,
        prclTrg as *const core::ffi::c_void,
        core::mem::size_of::<RECTL>(),
    ) {
        return 0;
    }
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(ptlSrc) as *mut core::ffi::c_void,
        pptlSrc as *const core::ffi::c_void,
        core::mem::size_of::<POINTL>(),
    ) {
        return 0;
    }
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(ptlMask) as *mut core::ffi::c_void,
        pptlMask as *const core::ffi::c_void,
        core::mem::size_of::<POINTL>(),
    ) {
        return 0;
    }
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(ptlBrush) as *mut core::ffi::c_void,
        pptlBrush as *const core::ffi::c_void,
        core::mem::size_of::<POINTL>(),
    ) {
        return 0;
    }

    return EngBitBlt(
        psoTrg,
        psoSrc,
        psoMask,
        pco,
        pxlo,
        core::ptr::addr_of_mut!(rclTrg),
        core::ptr::addr_of_mut!(ptlSrc),
        core::ptr::addr_of_mut!(ptlMask),
        pbo,
        core::ptr::addr_of_mut!(ptlBrush),
        Rop4,
    );
}
#[no_mangle]
pub unsafe extern "system" fn EngBitBlt(
    psoTrg: *mut SURFOBJ,
    psoSrc: *mut SURFOBJ,
    mut psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    mut rop4: ROP4,
) -> i32 {
    let mut clippingType: u8;
    let mut CombinedRect: RECTL = core::mem::zeroed();
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: u32;
    let mut InputPoint: POINTL = core::mem::zeroed();
    let mut InputRect: RECTL = core::mem::zeroed();
    let mut OutputRect: RECTL = core::mem::zeroed();
    let mut InputObj: *mut SURFOBJ = core::ptr::null_mut();
    let OutputObj: *mut SURFOBJ;
    let mut BltRectFunc: PBLTRECTFUNC;
    let mut Ret: u8 = 1u8;
    let mut ClipRect: RECTL = core::mem::zeroed();
    let mut i: u32;
    let mut Pt: POINTL = core::mem::zeroed();
    let mut Direction: u32;
    let UsesSource: u32;
    let mut UsesMask: u32;
    let mut AdjustedBrushOrigin: POINTL = core::mem::zeroed();
    let mut lTmp: i32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;

    UsesSource = ROP4_USES_SOURCE(rop4);
    UsesMask = ROP4_USES_MASK(rop4);

    if (*prclTrg).left > (*prclTrg).right {
        bLeftToRight = 1u8;
    } else {
        bLeftToRight = 0u8;
    }

    if (*prclTrg).top > (*prclTrg).bottom {
        bTopToBottom = 1u8;
    } else {
        bTopToBottom = 0u8;
    }

    if rop4 == ROP4_NOOP {
        return 1;
    }

    OutputRect = *prclTrg;
    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(OutputRect));

    if UsesSource != 0 {
        if psoSrc.is_null() || pptlSrc.is_null() {
            return 0;
        }

        InputPoint = *pptlSrc;
        if InputPoint.x < 0 {
            OutputRect.left -= InputPoint.x;
            InputPoint.x = 0;
        }
        if InputPoint.y < 0 {
            OutputRect.top -= InputPoint.y;
            InputPoint.y = 0;
        }
        if (*psoSrc).sizlBitmap.cx < InputPoint.x + OutputRect.right - OutputRect.left {
            OutputRect.right = OutputRect.left + (*psoSrc).sizlBitmap.cx - InputPoint.x;
        }
        if (*psoSrc).sizlBitmap.cy < InputPoint.y + OutputRect.bottom - OutputRect.top {
            OutputRect.bottom = OutputRect.top + (*psoSrc).sizlBitmap.cy - InputPoint.y;
        }

        InputRect.left = InputPoint.x;
        InputRect.right = InputPoint.x + (OutputRect.right - OutputRect.left);
        InputRect.top = InputPoint.y;
        InputRect.bottom = InputPoint.y + (OutputRect.bottom - OutputRect.top);

        InputObj = psoSrc;
    } else {
        InputPoint.x = 0;
        InputPoint.y = 0;
        InputRect.left = 0;
        InputRect.right = (*prclTrg).right - (*prclTrg).left;
        InputRect.top = 0;
        InputRect.bottom = (*prclTrg).bottom - (*prclTrg).top;
    }

    if !pco.is_null() {
        if OutputRect.left < (*pco).rclBounds.left {
            InputRect.left += (*pco).rclBounds.left - OutputRect.left;
            InputPoint.x += (*pco).rclBounds.left - OutputRect.left;
            OutputRect.left = (*pco).rclBounds.left;
        }
        if (*pco).rclBounds.right < OutputRect.right {
            InputRect.right -= OutputRect.right - (*pco).rclBounds.right;
            OutputRect.right = (*pco).rclBounds.right;
        }
        if OutputRect.top < (*pco).rclBounds.top {
            InputRect.top += (*pco).rclBounds.top - OutputRect.top;
            InputPoint.y += (*pco).rclBounds.top - OutputRect.top;
            OutputRect.top = (*pco).rclBounds.top;
        }
        if (*pco).rclBounds.bottom < OutputRect.bottom {
            InputRect.bottom -= OutputRect.bottom - (*pco).rclBounds.bottom;
            OutputRect.bottom = (*pco).rclBounds.bottom;
        }
    }

    if OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top {
        return 1;
    }

    OutputObj = psoTrg;

    if !pptlBrush.is_null() {
        AdjustedBrushOrigin.x = (*pptlBrush).x;
        AdjustedBrushOrigin.y = (*pptlBrush).y;
    } else {
        AdjustedBrushOrigin.x = 0;
        AdjustedBrushOrigin.y = 0;
    }

    if pco.is_null() {
        clippingType = DC_TRIVIAL as u8;
    } else {
        clippingType = (*pco).iDComplexity;
    }

    if UsesMask != 0 && psoMask.is_null() {
        // C-exact: REASSIGN the params (C `psoMask = BRUSHOBJ_psoMask(pbo)` /
        // `rop4 = ...`). The translation shadowed both with block-local `let`s,
        // so a brush-provided mask was discarded (mask blits ran with
        // psoMask=NULL while UsesMask stayed set) and the no-mask rop4
        // degrade never applied.
        psoMask = BRUSHOBJ_psoMask(pbo);
        if psoMask.is_null() {
            rop4 = (rop4 & 0xFF) | ((rop4 & 0xFF) << 8);
            UsesMask = 0;
        }
    }

    if UsesMask != 0 {
        BltRectFunc = Some(BltMask);
    } else if (rop4 & 0xFF) == R3_OPINDEX_PATCOPY {
        if !pbo.is_null() && (*pbo).iSolidColor == 0xFFFFFFFF {
            BltRectFunc = Some(CallDibBitBlt);
        } else {
            BltRectFunc = Some(BltPatCopy);
        }
    } else {
        BltRectFunc = Some(CallDibBitBlt);
    }

    match clippingType as u32 {
        DC_TRIVIAL => {
            if bLeftToRight != 0 {
                lTmp = OutputRect.left;
                OutputRect.left = OutputRect.right;
                OutputRect.right = lTmp;
            }

            if bTopToBottom != 0 {
                lTmp = OutputRect.top;
                OutputRect.top = OutputRect.bottom;
                OutputRect.bottom = lTmp;
            }

            Ret = BltRectFunc.unwrap()(
                OutputObj,
                InputObj,
                psoMask,
                pxlo,
                core::ptr::addr_of_mut!(OutputRect),
                core::ptr::addr_of_mut!(InputPoint),
                pptlMask,
                pbo,
                core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                rop4,
            );
        }
        DC_RECT => {
            ClipRect.left = (*pco).rclBounds.left;
            ClipRect.right = (*pco).rclBounds.right;
            ClipRect.top = (*pco).rclBounds.top;
            ClipRect.bottom = (*pco).rclBounds.bottom;
            if RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(CombinedRect),
                core::ptr::addr_of_mut!(OutputRect),
                core::ptr::addr_of_mut!(ClipRect),
            ) != 0
            {
                Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
                Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
                Ret = BltRectFunc.unwrap()(
                    OutputObj,
                    InputObj,
                    psoMask,
                    pxlo,
                    core::ptr::addr_of_mut!(CombinedRect),
                    core::ptr::addr_of_mut!(Pt),
                    pptlMask,
                    pbo,
                    core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                    rop4,
                );
            }
        }
        DC_COMPLEX => {
            Ret = 1u8;
            if OutputObj == InputObj {
                if OutputRect.top < InputPoint.y {
                    Direction = if OutputRect.left < InputPoint.x {
                        CD_RIGHTDOWN
                    } else {
                        CD_LEFTDOWN
                    };
                } else {
                    Direction = if OutputRect.left < InputPoint.x {
                        CD_RIGHTUP
                    } else {
                        CD_LEFTUP
                    };
                }
            } else {
                Direction = CD_ANY;
            }
            CLIPOBJ_cEnumStart(pco, 0, CT_RECTANGLES, Direction, 0);
            loop {
                EnumMore = CLIPOBJ_bEnum(
                    pco,
                    core::mem::size_of::<RECT_ENUM>() as u32,
                    core::ptr::addr_of_mut!(RectEnum) as *mut core::ffi::c_void,
                ) as u32;

                i = 0;
                while i < RectEnum.c {
                    ClipRect.left = RectEnum.arcl[i as usize].left;
                    ClipRect.right = RectEnum.arcl[i as usize].right;
                    ClipRect.top = RectEnum.arcl[i as usize].top;
                    ClipRect.bottom = RectEnum.arcl[i as usize].bottom;
                    if RECTL_bIntersectRect(
                        core::ptr::addr_of_mut!(CombinedRect),
                        core::ptr::addr_of_mut!(OutputRect),
                        core::ptr::addr_of_mut!(ClipRect),
                    ) != 0
                    {
                        Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
                        Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
                        Ret = (BltRectFunc.unwrap()(
                            OutputObj,
                            InputObj,
                            psoMask,
                            pxlo,
                            core::ptr::addr_of_mut!(CombinedRect),
                            core::ptr::addr_of_mut!(Pt),
                            pptlMask,
                            pbo,
                            core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                            rop4,
                        ) != 0
                            && Ret != 0) as u8;
                    }
                    i += 1;
                }

                if EnumMore == 0 {
                    break;
                }
            }
        }
        _ => {}
    }

    Ret as i32
}
#[no_mangle]
pub unsafe extern "system" fn IntEngBitBlt(
    psoTrg: *mut SURFOBJ,
    mut psoSrc: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    mut pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclTrg: *mut RECTL,
    mut pptlSrc: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
    Rop4: ROP4,
) -> i32 {
    let mut psurfTrg: *mut SURFACE;
    let mut psurfSrc: *mut SURFACE = core::ptr::null_mut();
    let mut bResult: i32;
    let mut rclClipped: RECTL = core::mem::zeroed();
    let mut rclSrc: RECTL = core::mem::zeroed();
    let mut rclSrcClipped: RECTL = core::mem::zeroed();
    let mut ptlBrush: POINTL = core::mem::zeroed();
    let mut pfnBitBlt: PFN_DrvBitBlt;
    let mut lTmp: i32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;

    psurfTrg = (psoTrg as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    bLeftToRight = if (*prclTrg).left > (*prclTrg).right { 1u8 } else { 0u8 };
    bTopToBottom = if (*prclTrg).top > (*prclTrg).bottom { 1u8 } else { 0u8 };

    rclClipped = *prclTrg;
    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(rclClipped));

    if RECTL_bClipRectBySize(core::ptr::addr_of_mut!(rclClipped), core::ptr::addr_of_mut!(rclClipped), core::ptr::addr_of_mut!((*psoTrg).sizlBitmap)) == 0 {
        return 1;
    }

    if !pco.is_null() {
        if RECTL_bIntersectRect(core::ptr::addr_of_mut!(rclClipped), core::ptr::addr_of_mut!(rclClipped), core::ptr::addr_of_mut!((*pco).rclBounds)) == 0 {
            return 1;
        }

        if (*pco).iDComplexity as u32 == DC_RECT {
            pco = core::ptr::null_mut();
        }
    } else {
        pco = core::ptr::addr_of_mut!(gxcoTrivial) as *mut CLIPOBJ;
    }

    if ROP4_USES_SOURCE(Rop4) != 0 {
        psurfSrc = (psoSrc as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

        rclSrc.left = (*pptlSrc).x + rclClipped.left - (*prclTrg).left;
        rclSrc.top = (*pptlSrc).y + rclClipped.top - (*prclTrg).top;
        rclSrc.right = rclSrc.left + rclClipped.right - rclClipped.left;
        rclSrc.bottom = rclSrc.top + rclClipped.bottom - rclClipped.top;

        if RECTL_bClipRectBySize(core::ptr::addr_of_mut!(rclSrcClipped), core::ptr::addr_of_mut!(rclSrc), core::ptr::addr_of_mut!((*psoSrc).sizlBitmap)) == 0 {
            return 1;
        }

        rclClipped.left += rclSrcClipped.left - rclSrc.left;
        rclClipped.top += rclSrcClipped.top - rclSrc.top;
        rclClipped.right -= rclSrc.right - rclSrcClipped.right;
        rclClipped.bottom -= rclSrc.bottom - rclSrcClipped.bottom;

        pptlSrc = core::ptr::addr_of_mut!(rclSrcClipped) as PPOINTL;
    } else {
        psoSrc = core::ptr::null_mut();
        psurfSrc = core::ptr::null_mut();
    }

    if !pptlBrush.is_null() {
        ptlBrush = *pptlBrush;
    }

    if (*psurfTrg).flags & HOOK_BITBLT != 0 {
        if !psoSrc.is_null() && (*psoSrc).hdev != (*psoTrg).hdev && (*psurfSrc).flags & HOOK_BITBLT != 0 {
        }

        pfnBitBlt = GDIDEVFUNCS(psoTrg).BitBlt;
    } else if !psoSrc.is_null() && (*psurfSrc).flags & HOOK_BITBLT != 0 {
        pfnBitBlt = GDIDEVFUNCS(psoSrc).BitBlt;
    } else {
        pfnBitBlt = Some(EngBitBlt);
    }

    if bLeftToRight != 0 {
        lTmp = rclClipped.left;
        rclClipped.left = rclClipped.right;
        rclClipped.right = lTmp;
    }

    if bTopToBottom != 0 {
        lTmp = rclClipped.top;
        rclClipped.top = rclClipped.bottom;
        rclClipped.bottom = lTmp;
    }

    bResult = pfnBitBlt.unwrap()(
        psoTrg,
        psoSrc,
        psoMask,
        pco,
        pxlo,
        core::ptr::addr_of_mut!(rclClipped),
        pptlSrc,
        pptlMask,
        pbo,
        if !pptlBrush.is_null() { core::ptr::addr_of_mut!(ptlBrush) } else { core::ptr::null_mut() },
        Rop4,
    );

    bResult
}

unsafe extern "system" fn AlphaBltMask(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pxloRGB2Dest: *mut XLATEOBJ,
    pxloBrush: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    pptlSource: *mut POINTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrush: *mut POINTL,
) -> u8 {
    let dx = (*prclDest).right - (*prclDest).left;
    let dy = (*prclDest).bottom - (*prclDest).top;

    if !psoMask.is_null() {
        let BrushColor = XLATEOBJ_iXlate(
            pxloBrush,
            if !pbo.is_null() { (*pbo).iSolidColor } else { 0 }
        );
        let r = GetRValue(BrushColor) as i32;
        let g = GetGValue(BrushColor) as i32;
        let b = GetBValue(BrushColor) as i32;

        let mut tMask = ((*psoMask).pvScan0 as *mut u8)
            .offset(((*pptlMask).y * (*psoMask).lDelta) as isize)
            .offset((*pptlMask).x as isize);

        for j in 0..dy {
            let mut lMask = tMask;
            for i in 0..dx {
                if *lMask > 0 {
                    if *lMask == 0xff {
                        DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize].DIB_PutPixel.unwrap()(
                            psoDest,
                            (*prclDest).left + i,
                            (*prclDest).top + j,
                            if !pbo.is_null() { (*pbo).iSolidColor } else { 0 }
                        );
                    } else {
                        let Background = DIB_GetSource(
                            psoDest,
                            (*prclDest).left + i,
                            (*prclDest).top + j,
                            pxloBrush
                        );

                        let NewColor = RGB(
                            (((*lMask as i32 * (r - GetRValue(Background) as i32) >> 8) + GetRValue(Background) as i32) as u8) as u32,
                            (((*lMask as i32 * (g - GetGValue(Background) as i32) >> 8) + GetGValue(Background) as i32) as u8) as u32,
                            (((*lMask as i32 * (b - GetBValue(Background) as i32) >> 8) + GetBValue(Background) as i32) as u8) as u32
                        );

                        let Background = XLATEOBJ_iXlate(pxloRGB2Dest, NewColor);
                        DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize].DIB_PutPixel.unwrap()(
                            psoDest,
                            (*prclDest).left + i,
                            (*prclDest).top + j,
                            Background
                        );
                    }
                }
                lMask = lMask.offset(1);
            }
            tMask = tMask.offset((*psoMask).lDelta as isize);
        }
        1u8
    } else {
        0u8
    }
}

unsafe fn EngMaskBitBlt(
    psoDest: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    DestColorTranslation: *mut XLATEOBJ,
    SourceColorTranslation: *mut XLATEOBJ,
    DestRect: *mut RECTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
) -> u8 {
    let mut clippingType: u8;
    let mut CombinedRect: RECTL = core::mem::zeroed();
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: u32;
    let mut InputPoint: POINTL = core::mem::zeroed();
    let mut InputRect: RECTL = core::mem::zeroed();
    let mut OutputRect: RECTL = core::mem::zeroed();
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeaveSource: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut EnterLeaveDest: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut psoInput: *mut SURFOBJ = core::ptr::null_mut();
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let mut Ret: u8 = 1u8;
    let mut ClipRect: RECTL = core::mem::zeroed();
    let mut i: u32;
    let mut Pt: POINTL = core::mem::zeroed();
    let mut Direction: u32;
    let mut AdjustedBrushOrigin: POINTL = core::mem::zeroed();

    if !pptlMask.is_null() {
        InputRect.left = (*pptlMask).x;
        InputRect.right = (*pptlMask).x + ((*DestRect).right - (*DestRect).left);
        InputRect.top = (*pptlMask).y;
        InputRect.bottom = (*pptlMask).y + ((*DestRect).bottom - (*DestRect).top);
    } else {
        InputRect.left = 0;
        InputRect.right = (*DestRect).right - (*DestRect).left;
        InputRect.top = 0;
        InputRect.bottom = (*DestRect).bottom - (*DestRect).top;
    }

    OutputRect = *DestRect;
    if !ClipRegion.is_null() {
        if OutputRect.left < (*ClipRegion).rclBounds.left {
            InputRect.left += (*ClipRegion).rclBounds.left - OutputRect.left;
            OutputRect.left = (*ClipRegion).rclBounds.left;
        }
        if (*ClipRegion).rclBounds.right < OutputRect.right {
            InputRect.right -= OutputRect.right - (*ClipRegion).rclBounds.right;
            OutputRect.right = (*ClipRegion).rclBounds.right;
        }
        if OutputRect.top < (*ClipRegion).rclBounds.top {
            InputRect.top += (*ClipRegion).rclBounds.top - OutputRect.top;
            OutputRect.top = (*ClipRegion).rclBounds.top;
        }
        if (*ClipRegion).rclBounds.bottom < OutputRect.bottom {
            InputRect.bottom -= OutputRect.bottom - (*ClipRegion).rclBounds.bottom;
            OutputRect.bottom = (*ClipRegion).rclBounds.bottom;
        }
    }

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeaveSource),
        psoMask,
        core::ptr::addr_of_mut!(InputRect),
        1,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(psoInput),
    ) == 0
    {
        return 0u8;
    }

    InputPoint.x = InputRect.left + Translate.x;
    InputPoint.y = InputRect.top + Translate.y;

    if OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top {
        IntEngLeave(core::ptr::addr_of_mut!(EnterLeaveSource));
        return 1u8;
    }

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeaveDest),
        psoDest,
        core::ptr::addr_of_mut!(OutputRect),
        0,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(psoOutput),
    ) == 0
    {
        IntEngLeave(core::ptr::addr_of_mut!(EnterLeaveSource));
        return 0u8;
    }

    OutputRect.left = (*DestRect).left + Translate.x;
    OutputRect.right = (*DestRect).right + Translate.x;
    OutputRect.top = (*DestRect).top + Translate.y;
    OutputRect.bottom = (*DestRect).bottom + Translate.y;

    if !BrushOrigin.is_null() {
        AdjustedBrushOrigin.x = (*BrushOrigin).x + Translate.x;
        AdjustedBrushOrigin.y = (*BrushOrigin).y + Translate.y;
    } else {
        AdjustedBrushOrigin = Translate;
    }

    if ClipRegion.is_null() {
        clippingType = DC_TRIVIAL as u8;
    } else {
        clippingType = (*ClipRegion).iDComplexity;
    }

    match clippingType as u32 {
        DC_TRIVIAL => {
            if (*psoMask).iBitmapFormat == BMF_8BPP {
                Ret = AlphaBltMask(
                    psoOutput,
                    core::ptr::null_mut(),
                    psoInput,
                    DestColorTranslation,
                    SourceColorTranslation,
                    core::ptr::addr_of_mut!(OutputRect),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(InputPoint),
                    pbo,
                    core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                );
            } else {
                Ret = BltMask(
                    psoOutput,
                    core::ptr::null_mut(),
                    psoInput,
                    DestColorTranslation,
                    core::ptr::addr_of_mut!(OutputRect),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(InputPoint),
                    pbo,
                    core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                    ROP4_MASK,
                );
            }
        }
        DC_RECT => {
            ClipRect.left = (*ClipRegion).rclBounds.left + Translate.x;
            ClipRect.right = (*ClipRegion).rclBounds.right + Translate.x;
            ClipRect.top = (*ClipRegion).rclBounds.top + Translate.y;
            ClipRect.bottom = (*ClipRegion).rclBounds.bottom + Translate.y;
            if RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(CombinedRect),
                core::ptr::addr_of_mut!(OutputRect),
                core::ptr::addr_of_mut!(ClipRect),
            ) != 0
            {
                Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
                Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
                if (*psoMask).iBitmapFormat == BMF_8BPP {
                    Ret = AlphaBltMask(
                        psoOutput,
                        core::ptr::null_mut(),
                        psoInput,
                        DestColorTranslation,
                        SourceColorTranslation,
                        core::ptr::addr_of_mut!(CombinedRect),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(Pt),
                        pbo,
                        core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                    );
                } else {
                    Ret = BltMask(
                        psoOutput,
                        core::ptr::null_mut(),
                        psoInput,
                        DestColorTranslation,
                        core::ptr::addr_of_mut!(CombinedRect),
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(Pt),
                        pbo,
                        core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                        ROP4_MASK,
                    );
                }
            }
        }
        DC_COMPLEX => {
            Ret = 1u8;
            if psoOutput == psoInput {
                if OutputRect.top < InputPoint.y {
                    Direction = if OutputRect.left < InputPoint.x {
                        CD_RIGHTDOWN
                    } else {
                        CD_LEFTDOWN
                    };
                } else {
                    Direction = if OutputRect.left < InputPoint.x {
                        CD_RIGHTUP
                    } else {
                        CD_LEFTUP
                    };
                }
            } else {
                Direction = CD_ANY;
            }
            CLIPOBJ_cEnumStart(ClipRegion, (0u8) as _, CT_RECTANGLES, Direction, 0);
            loop {
                EnumMore = CLIPOBJ_bEnum(
                    ClipRegion,
                    core::mem::size_of::<RECT_ENUM>() as u32,
                    core::ptr::addr_of_mut!(RectEnum) as *mut core::ffi::c_void,
                ) as u32;

                i = 0;
                while i < RectEnum.c {
                    ClipRect.left = RectEnum.arcl[i as usize].left + Translate.x;
                    ClipRect.right = RectEnum.arcl[i as usize].right + Translate.x;
                    ClipRect.top = RectEnum.arcl[i as usize].top + Translate.y;
                    ClipRect.bottom = RectEnum.arcl[i as usize].bottom + Translate.y;
                    if RECTL_bIntersectRect(
                        core::ptr::addr_of_mut!(CombinedRect),
                        core::ptr::addr_of_mut!(OutputRect),
                        core::ptr::addr_of_mut!(ClipRect),
                    ) != 0
                    {
                        Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
                        Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
                        if (*psoMask).iBitmapFormat == BMF_8BPP {
                            Ret = (AlphaBltMask(
                                psoOutput,
                                core::ptr::null_mut(),
                                psoInput,
                                DestColorTranslation,
                                SourceColorTranslation,
                                core::ptr::addr_of_mut!(CombinedRect),
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(Pt),
                                pbo,
                                core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                            ) != 0 && Ret != 0) as u8;
                        } else {
                            Ret = (BltMask(
                                psoOutput,
                                core::ptr::null_mut(),
                                psoInput,
                                DestColorTranslation,
                                core::ptr::addr_of_mut!(CombinedRect),
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(Pt),
                                pbo,
                                core::ptr::addr_of_mut!(AdjustedBrushOrigin),
                                ROP4_MASK,
                            ) != 0 && Ret != 0) as u8;
                        }
                    }
                    i += 1;
                }

                if EnumMore == 0 {
                    break;
                }
            }
        }
        _ => {}
    }

    IntEngLeave(core::ptr::addr_of_mut!(EnterLeaveDest));
    IntEngLeave(core::ptr::addr_of_mut!(EnterLeaveSource));

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn IntEngMaskBlt(
    psoDest: *mut SURFOBJ,
    psoMask: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxloDest: *mut XLATEOBJ,
    pxloSource: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    pptlMask: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
) -> i32 {
    let mut ret: u8;
    let mut rcDest: RECTL = core::mem::zeroed();
    let mut ptMask: POINTL = POINTL { x: 0, y: 0 };
    let mut psurfTemp: PSURFACE;
    let mut rcTemp: RECTL = core::mem::zeroed();

    if (*psoMask).iBitmapFormat == BMF_1BPP {
        return IntEngBitBlt(
            psoDest,
            core::ptr::null_mut(),
            psoMask,
            pco,
            pxloDest,
            prclDest,
            core::ptr::null_mut(),
            pptlMask,
            pbo,
            pptlBrushOrg,
            ROP4_MASKPAINT,
        );
    }

    if !pptlMask.is_null() {
        ptMask = *pptlMask;
    }

    if !pco.is_null() {
        if RECTL_bIntersectRect(
            core::ptr::addr_of_mut!(rcDest),
            prclDest,
            core::ptr::addr_of!((*pco).rclBounds),
        ) == 0
        {
            return 1;
        }

        ptMask.x += rcDest.left - (*prclDest).left;
        ptMask.y += rcDest.top - (*prclDest).top;
    } else {
        rcDest = *prclDest;
    }

    if (*psoDest).iType as u32 != STYPE_BITMAP {
        rcTemp.left = 0;
        rcTemp.top = 0;
        rcTemp.right = rcDest.right - rcDest.left;
        rcTemp.bottom = rcDest.bottom - rcDest.top;

        psurfTemp = SURFACE_AllocSurface(
            STYPE_BITMAP as u16,
            rcTemp.right as u32,
            rcTemp.bottom as u32,
            (*psoDest).iBitmapFormat,
            0,
            0,
            0,
            core::ptr::null_mut(),
        );
        if psurfTemp.is_null() {
            return 0;
        }

        ret = EngCopyBits(
            core::ptr::addr_of_mut!((*psurfTemp).SurfObj),
            psoDest,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(rcTemp),
            core::ptr::addr_of!(rcDest) as PPOINTL,
        ) as u8;

        if ret != 0 {
            ret = EngMaskBitBlt(
                core::ptr::addr_of_mut!((*psurfTemp).SurfObj),
                psoMask,
                core::ptr::null_mut(),
                pxloDest,
                pxloSource,
                core::ptr::addr_of_mut!(rcTemp),
                core::ptr::addr_of_mut!(ptMask),
                pbo,
                pptlBrushOrg,
            );
        }

        if ret != 0 {
            ret = EngCopyBits(
                psoDest,
                core::ptr::addr_of_mut!((*psurfTemp).SurfObj),
                pco,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(rcDest),
                core::ptr::addr_of!(rcTemp) as PPOINTL,
            ) as u8;
        }

        GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*psurfTemp).BaseObject) as POBJ);
    } else {
        ret = EngMaskBitBlt(
            psoDest,
            psoMask,
            pco,
            pxloDest,
            pxloSource,
            core::ptr::addr_of_mut!(rcDest),
            core::ptr::addr_of_mut!(ptMask),
            pbo,
            pptlBrushOrg,
        );
    }

    ret as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          GDI BitBlt Functions
//  * FILE:             win32ss/gdi/eng/bitblt.c
//  * PROGRAMERS:       Jason Filby
//  *                   Timo Kreuzer
//  *                   Doug Lyons
//  *
//  * WARNING: Modify this file with extreme caution. It is very sensitive to timing changes.
//  *   Adding code can cause the system to show a Fatal Exception Error and fail to boot!.
//  *   This is especially true in CallDibBitBlt, IntEngBitBlt and EngBitBlt (even DPRINT's).
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// XCLIPOBJ gxcoTrivial =
// {
//     /* CLIPOBJ */
//     {
//         0, /* iUniq */
//         {LONG_MIN, LONG_MIN, LONG_MAX, LONG_MAX}, /* rclBounds */
//         DC_TRIVIAL,    /* idCOmplexity */
//         FC_RECT,       /* iFComplexity */
//         TC_RECTANGLES, /* iMode */
//         0              /* fjOptions */
//     },
//     { 0, {0,0,0,0}, 0},
//     0, {0,0,0,0}, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
// };
//
// typedef BOOLEAN (APIENTRY *PBLTRECTFUNC)(SURFOBJ* OutputObj,
//                                         SURFOBJ* InputObj,
//                                         SURFOBJ* Mask,
//                                         XLATEOBJ* ColorTranslation,
//                                         RECTL* OutputRect,
//                                         POINTL* InputPoint,
//                                         POINTL* MaskOrigin,
//                                         BRUSHOBJ* pbo,
//                                         POINTL* BrushOrigin,
//                                         ROP4 Rop4);
//
// static BOOLEAN APIENTRY
// BltMask(SURFOBJ* psoDest,
//         SURFOBJ* psoSource,
//         SURFOBJ* psoMask,
//         XLATEOBJ* ColorTranslation,
//         RECTL* prclDest,
//         POINTL* pptlSource,
//         POINTL* pptlMask,
//         BRUSHOBJ* pbo,
//         POINTL* pptlBrush,
//         ROP4 Rop4)
// {
//     LONG x, y, cx, cy;
//     BYTE *pjMskLine, *pjMskCurrent;
//     BYTE fjMaskBit0, fjMaskBit;
//     /* Pattern brushes */
//     SURFOBJ *psoPattern;
//     ULONG PatternWidth = 0, PatternHeight = 0;
//     LONG PatternX0 = 0, PatternX = 0, PatternY = 0;
//     LONG SrcX = 0, SrcY = 0;
//     PFN_DIB_PutPixel fnDest_PutPixel = NULL;
//     PFN_DIB_GetPixel fnPattern_GetPixel = NULL, fnSrc_GetPixel = NULL, fnDest_GetPixel;
//     ULONG Pattern = 0, Source = 0, Dest = 0;
//     DWORD fgndRop, bkgndRop;
//
//     ASSERT(IS_VALID_ROP4(Rop4));
//     ASSERT(psoMask->iBitmapFormat == BMF_1BPP);
//
//     fgndRop = ROP4_FGND(Rop4);
//     bkgndRop = ROP4_BKGND(Rop4);
//
//     //DPRINT1("Rop4 : 0x%08x\n", Rop4);
//
//     /* Determine pattern */
//     if (pbo && pbo->iSolidColor == 0xFFFFFFFF)
//     {
//         psoPattern = BRUSHOBJ_psoPattern(pbo);
//         if (psoPattern)
//         {
//             PatternWidth = psoPattern->sizlBitmap.cx;
//             PatternHeight = psoPattern->sizlBitmap.cy;
//             fnPattern_GetPixel = DibFunctionsForBitmapFormat[psoPattern->iBitmapFormat].DIB_GetPixel;
//         }
//     }
//     else
//         psoPattern = NULL;
//
//     cx = prclDest->right - prclDest->left;
//     cy = prclDest->bottom - prclDest->top;
//     if ((pptlMask->x + cx > psoMask->sizlBitmap.cx) ||
//         (pptlMask->y + cy > psoMask->sizlBitmap.cy))
//     {
//         return FALSE;
//     }
//
//     pjMskLine = (PBYTE)psoMask->pvScan0 + pptlMask->y * psoMask->lDelta + (pptlMask->x >> 3);
//     fjMaskBit0 = 0x80 >> (pptlMask->x & 0x07);
//
//     fnDest_PutPixel = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_PutPixel;
//     fnDest_GetPixel = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_GetPixel;
//
//     /* Do we have a source */
//     if(psoSource)
//     {
//         /* Sanity check */
//         ASSERT(ROP4_USES_SOURCE(Rop4));
//         fnSrc_GetPixel = DibFunctionsForBitmapFormat[psoSource->iBitmapFormat].DIB_GetPixel;
//         SrcY = pptlSource->y;
//         SrcX = pptlSource->x;
//     }
//
//     if (psoPattern)
//     {
//         PatternY = (prclDest->top - pptlBrush->y) % PatternHeight;
//         if (PatternY < 0)
//         {
//             PatternY += PatternHeight;
//         }
//         PatternX0 = (prclDest->left - pptlBrush->x) % PatternWidth;
//         if (PatternX0 < 0)
//         {
//             PatternX0 += PatternWidth;
//         }
//         PatternX = PatternX0;
//     }
//     else
//     {
//         Pattern = pbo ? pbo->iSolidColor : 0;
//     }
//
//     for (y = prclDest->top; y < prclDest->bottom; y++)
//     {
//         pjMskCurrent = pjMskLine;
//         fjMaskBit = fjMaskBit0;
//
//         for (x = prclDest->left; x < prclDest->right; x++)
//         {
//             Rop4 = (*pjMskCurrent & fjMaskBit) ? fgndRop : bkgndRop;
//
//             if(psoPattern)
//             {
//                 if(ROP4_USES_PATTERN(Rop4))
//                     Pattern = fnPattern_GetPixel(psoPattern, PatternX, PatternY);
//                 PatternX++;
//                 PatternX %= PatternWidth;
//             }
//
//             if(psoSource)
//             {
//                 if(ROP4_USES_SOURCE(Rop4))
//                 {
//                     Source = XLATEOBJ_iXlate(ColorTranslation,
//                                              fnSrc_GetPixel(psoSource, SrcX, SrcY));
//                 }
//                 SrcX++;
//             }
//
//             if(ROP4_USES_DEST(Rop4))
//                 Dest = fnDest_GetPixel(psoDest, x, y);
//
//             fnDest_PutPixel(psoDest,
//                             x,
//                             y,
//                             DIB_DoRop(Rop4,
//                                       Dest,
//                                       Source,
//                                       Pattern));
//             fjMaskBit = _rotr8(fjMaskBit, 1);
//             pjMskCurrent += (fjMaskBit >> 7);
//         }
//         pjMskLine += psoMask->lDelta;
//         if(psoPattern)
//         {
//             PatternY++;
//             PatternY %= PatternHeight;
//             PatternX = PatternX0;
//         }
//         if(psoSource)
//         {
//             SrcY++;
//             SrcX = pptlSource->x;
//         }
//     }
//
//     return TRUE;
// }
//
// #ifndef _USE_DIBLIB_
//
// static BOOLEAN APIENTRY
// BltPatCopy(SURFOBJ* Dest,
//            SURFOBJ* Source,
//            SURFOBJ* Mask,
//            XLATEOBJ* ColorTranslation,
//            RECTL* DestRect,
//            POINTL* SourcePoint,
//            POINTL* MaskPoint,
//            BRUSHOBJ* pbo,
//            POINTL* BrushPoint,
//            DWORD Rop4)
// {
//     // These functions are assigned if we're working with a DIB
//     // The assigned functions depend on the bitsPerPixel of the DIB
//     ULONG color = pbo ? pbo->iSolidColor : 0;
//
//     DibFunctionsForBitmapFormat[Dest->iBitmapFormat].DIB_ColorFill(Dest, DestRect, color);
//
//     return TRUE;
// }
//
// static BOOLEAN APIENTRY
// CallDibBitBlt(SURFOBJ* OutputObj,
//               SURFOBJ* InputObj,
//               SURFOBJ* Mask,
//               XLATEOBJ* ColorTranslation,
//               RECTL* OutputRect,
//               POINTL* InputPoint,
//               POINTL* MaskOrigin,
//               BRUSHOBJ* pbo,
//               POINTL* BrushOrigin,
//               ROP4 Rop4)
// {
//     BLTINFO BltInfo;
//     SURFOBJ *psoPattern;
//     BOOLEAN Result;
//
//     BltInfo.DestSurface = OutputObj;
//     BltInfo.SourceSurface = InputObj;
//     BltInfo.PatternSurface = NULL;
//     BltInfo.XlateSourceToDest = ColorTranslation;
//     BltInfo.DestRect = *OutputRect;
//     BltInfo.SourcePoint = *InputPoint;
//
//     if ((Rop4 & 0xFF) == R3_OPINDEX_SRCCOPY)
//         return DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_BitBltSrcCopy(&BltInfo);
//
//     BltInfo.Brush = pbo;
//     BltInfo.BrushOrigin = *BrushOrigin;
//     BltInfo.Rop4 = Rop4;
//
//     /* Pattern brush */
//     if (ROP4_USES_PATTERN(Rop4) && pbo && pbo->iSolidColor == 0xFFFFFFFF)
//     {
//         psoPattern = BRUSHOBJ_psoPattern(pbo);
//         if (psoPattern)
//         {
//             BltInfo.PatternSurface = psoPattern;
//         }
//         else
//         {
//             /* FIXME: What to do here? */
//         }
//     }
//     else
//     {
//         psoPattern = NULL;
//     }
//
//     /* Make WellOrdered with top < bottom and left < right */
//     RECTL_vMakeWellOrdered(&BltInfo.DestRect);
//
//     DPRINT("CallDibBitBlt: BltInfo.DestRect: (%d,%d)-(%d,%d)\n",
//            BltInfo.DestRect.left, BltInfo.DestRect.top, BltInfo.DestRect.right, BltInfo.DestRect.bottom);
//
//     Result = DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_BitBlt(&BltInfo);
//
//     return Result;
// }
//
// INT __cdecl abs(INT nm);
//
//
// /*
//  * @implemented
//  */
// BOOL APIENTRY
// NtGdiEngBitBlt(
//                 IN SURFOBJ  *psoTrg,
//                 IN SURFOBJ  *psoSrc,
//                 IN SURFOBJ  *psoMask,
//                 IN CLIPOBJ  *pco,
//                 IN XLATEOBJ  *pxlo,
//                 IN RECTL  *prclTrg,
//                 IN POINTL  *pptlSrc,
//                 IN POINTL  *pptlMask,
//                 IN BRUSHOBJ  *pbo,
//                 IN POINTL  *pptlBrush,
//                 IN ROP4 Rop4)
// {
//     RECTL  rclTrg;
//     POINTL ptlSrc;
//     POINTL ptlMask;
//     POINTL ptlBrush;
//     NTSTATUS Status;
//
//     Status = SafeProbeForRead(prclTrg, sizeof(RECTL), 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pptlSrc, sizeof(POINTL), 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pptlMask, sizeof(POINTL), 1);
//     if (NT_SUCCESS(Status)) Status = SafeProbeForRead(pptlBrush, sizeof(POINTL), 1);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(&rclTrg,prclTrg, sizeof(RECTL));
//         RtlCopyMemory(&ptlSrc, pptlSrc, sizeof(POINTL));
//         RtlCopyMemory(&ptlMask, pptlMask, sizeof(POINTL));
//         RtlCopyMemory(&ptlBrush, pptlBrush, sizeof(POINTL));
//     }
//
//     return  EngBitBlt(psoTrg, psoSrc, psoMask, pco, pxlo, &rclTrg, &ptlSrc, &ptlMask, pbo, &ptlBrush, Rop4);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngBitBlt(
//     _Inout_ SURFOBJ *psoTrg,
//     _In_opt_ SURFOBJ *psoSrc,
//     _In_opt_ SURFOBJ *psoMask,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclTrg,
//     _In_opt_ POINTL *pptlSrc,
//     _In_opt_ POINTL *pptlMask,
//     _In_opt_ BRUSHOBJ *pbo,
//     _In_opt_ POINTL *pptlBrush,
//     _In_ ROP4 rop4)
// {
//     BYTE               clippingType;
//     RECTL              CombinedRect;
//     RECT_ENUM          RectEnum;
//     BOOL               EnumMore;
//     POINTL             InputPoint;
//     RECTL              InputRect;
//     RECTL              OutputRect;
//     SURFOBJ*           InputObj = 0;
//     SURFOBJ*           OutputObj;
//     PBLTRECTFUNC       BltRectFunc;
//     BOOLEAN            Ret = TRUE;
//     RECTL              ClipRect;
//     ULONG              i;
//     POINTL             Pt;
//     ULONG              Direction;
//     BOOL               UsesSource, UsesMask;
//     POINTL             AdjustedBrushOrigin;
//     LONG               lTmp;
//     BOOLEAN            bTopToBottom, bLeftToRight;
//
//     UsesSource = ROP4_USES_SOURCE(rop4);
//     UsesMask = ROP4_USES_MASK(rop4);
//
//     if (prclTrg->left > prclTrg->right)
//     {
//       bLeftToRight = TRUE;
//     }
//     else
//     {
//       bLeftToRight = FALSE;
//     }
//
//     if (prclTrg->top > prclTrg->bottom)
//     {
//       bTopToBottom = TRUE;
//     }
//     else
//     {
//       bTopToBottom = FALSE;
//     }
//
//     if (rop4 == ROP4_NOOP)
//     {
//         /* Copy destination onto itself: nop */
//         return TRUE;
//     }
//
//     //DPRINT1("rop4 : 0x%08x\n", rop4);
//
//     OutputRect = *prclTrg;
//     RECTL_vMakeWellOrdered(&OutputRect);
//
//     DPRINT("EngBitBlt: prclTrg: (%d,%d)-(%d,%d)\n",
//            prclTrg->left, prclTrg->top, prclTrg->right, prclTrg->bottom);
//
//     DPRINT("EngBitBlt: OutputRect: (%d,%d)-(%d,%d)\n",
//            OutputRect.left, OutputRect.top, OutputRect.right, OutputRect.bottom);
//
//     if (UsesSource)
//     {
//         if (!psoSrc || !pptlSrc)
//         {
//             return FALSE;
//         }
//
//         /* Make sure we don't try to copy anything outside the valid source
//            region */
//         InputPoint = *pptlSrc;
//         if (InputPoint.x < 0)
//         {
//             OutputRect.left -= InputPoint.x;
//             InputPoint.x = 0;
//         }
//         if (InputPoint.y < 0)
//         {
//             OutputRect.top -= InputPoint.y;
//             InputPoint.y = 0;
//         }
//         if (psoSrc->sizlBitmap.cx < InputPoint.x +
//                 OutputRect.right - OutputRect.left)
//         {
//             OutputRect.right = OutputRect.left +
//                                psoSrc->sizlBitmap.cx - InputPoint.x;
//         }
//         if (psoSrc->sizlBitmap.cy < InputPoint.y +
//                 OutputRect.bottom - OutputRect.top)
//         {
//             OutputRect.bottom = OutputRect.top +
//                                 psoSrc->sizlBitmap.cy - InputPoint.y;
//         }
//
//         InputRect.left = InputPoint.x;
//         InputRect.right = InputPoint.x + (OutputRect.right - OutputRect.left);
//         InputRect.top = InputPoint.y;
//         InputRect.bottom = InputPoint.y + (OutputRect.bottom - OutputRect.top);
//
//         InputObj = psoSrc;
//     }
//     else
//     {
//         InputPoint.x = InputPoint.y = 0;
//         InputRect.left = 0;
//         InputRect.right = prclTrg->right - prclTrg->left;
//         InputRect.top = 0;
//         InputRect.bottom = prclTrg->bottom - prclTrg->top;
//     }
//
//     if (NULL != pco)
//     {
//         if (OutputRect.left < pco->rclBounds.left)
//         {
//             InputRect.left += pco->rclBounds.left - OutputRect.left;
//             InputPoint.x += pco->rclBounds.left - OutputRect.left;
//             OutputRect.left = pco->rclBounds.left;
//         }
//         if (pco->rclBounds.right < OutputRect.right)
//         {
//             InputRect.right -=  OutputRect.right - pco->rclBounds.right;
//             OutputRect.right = pco->rclBounds.right;
//         }
//         if (OutputRect.top < pco->rclBounds.top)
//         {
//             InputRect.top += pco->rclBounds.top - OutputRect.top;
//             InputPoint.y += pco->rclBounds.top - OutputRect.top;
//             OutputRect.top = pco->rclBounds.top;
//         }
//         if (pco->rclBounds.bottom < OutputRect.bottom)
//         {
//             InputRect.bottom -=  OutputRect.bottom - pco->rclBounds.bottom;
//             OutputRect.bottom = pco->rclBounds.bottom;
//         }
//     }
//
//     /* Check for degenerate case: if height or width of OutputRect is 0 pixels
//        there's nothing to do */
//     if (OutputRect.right <= OutputRect.left ||
//             OutputRect.bottom <= OutputRect.top)
//     {
//         return TRUE;
//     }
//
//     OutputObj = psoTrg;
//
//     if (pptlBrush)
//     {
//         AdjustedBrushOrigin.x = pptlBrush->x;
//         AdjustedBrushOrigin.y = pptlBrush->y;
//     }
//     else
//     {
//         AdjustedBrushOrigin.x = 0;
//         AdjustedBrushOrigin.y = 0;
//     }
//
//     /* Determine clipping type */
//     if (pco == (CLIPOBJ *) NULL)
//     {
//         clippingType = DC_TRIVIAL;
//     }
//     else
//     {
//         clippingType = pco->iDComplexity;
//     }
//
//     /* Check if we need a mask but have no mask surface */
//     if (UsesMask && (psoMask == NULL))
//     {
//         /* Check if the BRUSHOBJ can provide the mask */
//         psoMask = BRUSHOBJ_psoMask(pbo);
//         if (psoMask == NULL)
//         {
//             /* We have no mask, assume the mask is all foreground */
//             rop4 = (rop4 & 0xFF) | ((rop4 & 0xFF) << 8);
//             UsesMask = FALSE;
//         }
//     }
//
//     if (UsesMask)
//     {
//         BltRectFunc = Some(BltMask);
//     }
//     else if ((rop4 & 0xFF) == R3_OPINDEX_PATCOPY)
//     {
//         if (pbo && pbo->iSolidColor == 0xFFFFFFFF)
//             BltRectFunc = CallDibBitBlt;
//         else
//             BltRectFunc = BltPatCopy;
//     }
//     else
//     {
//         BltRectFunc = CallDibBitBlt;
//     }
//
//
//     switch (clippingType)
//     {
//         case DC_TRIVIAL:
//             /* Fix up OutputRect here */
//             if (bLeftToRight)
//             {
//                 lTmp = OutputRect.left;
//                 OutputRect.left = OutputRect.right;
//                 OutputRect.right = lTmp;
//             }
//
//             if (bTopToBottom)
//             {
//                 lTmp = OutputRect.top;
//                 OutputRect.top = OutputRect.bottom;
//                 OutputRect.bottom = lTmp;
//             }
//
//             Ret = (*BltRectFunc)(OutputObj,
//                                  InputObj,
//                                  psoMask,
//                                  pxlo,
//                                  &OutputRect,
//                                  &InputPoint,
//                                  pptlMask,
//                                  pbo,
//                                  &AdjustedBrushOrigin,
//                                  rop4);
//             break;
//         case DC_RECT:
//             /* Clip the blt to the clip rectangle */
//             ClipRect.left = pco->rclBounds.left;
//             ClipRect.right = pco->rclBounds.right;
//             ClipRect.top = pco->rclBounds.top;
//             ClipRect.bottom = pco->rclBounds.bottom;
//             if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//             {
// #ifdef _USE_DIBLIB_
//                 if (BrushOrigin)
//                 {
//                     AdjustedBrushOrigin.x = BrushOrigin->x + CombinedRect.left - OutputRect.left;
//                     AdjustedBrushOrigin.y = BrushOrigin->y + CombinedRect.top - OutputRect.top;
//                 }
// #endif
//                 Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
//                 Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
//                 Ret = (*BltRectFunc)(OutputObj,
//                                      InputObj,
//                                      psoMask,
//                                      pxlo,
//                                      &CombinedRect,
//                                      &Pt,
//                                      pptlMask,
//                                      pbo,
//                                      &AdjustedBrushOrigin,
//                                      rop4);
//             }
//             break;
//         case DC_COMPLEX:
//             Ret = TRUE;
//             if (OutputObj == InputObj)
//             {
//                 if (OutputRect.top < InputPoint.y)
//                 {
//                     Direction = OutputRect.left < InputPoint.x ?
//                                 CD_RIGHTDOWN : CD_LEFTDOWN;
//                 }
//                 else
//                 {
//                     Direction = OutputRect.left < InputPoint.x ?
//                                 CD_RIGHTUP : CD_LEFTUP;
//                 }
//             }
//             else
//             {
//                 Direction = CD_ANY;
//             }
//             CLIPOBJ_cEnumStart(pco, FALSE, CT_RECTANGLES, Direction, 0);
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(pco, sizeof(RectEnum),
//                                          (PVOID) &RectEnum) as u32;
//
//                 for (i = 0; i < RectEnum.c; i++)
//                 {
//                     ClipRect.left = RectEnum.arcl[i].left;
//                     ClipRect.right = RectEnum.arcl[i].right;
//                     ClipRect.top = RectEnum.arcl[i].top;
//                     ClipRect.bottom = RectEnum.arcl[i].bottom;
//                     if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//                     {
// #ifdef _USE_DIBLIB_
//                         if (BrushOrigin)
//                         {
//                             AdjustedBrushOrigin.x = BrushOrigin->x + CombinedRect.left - OutputRect.left;
//                             AdjustedBrushOrigin.y = BrushOrigin->y + CombinedRect.top - OutputRect.top;
//                         }
// #endif
//                         Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
//                         Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
//                         Ret = (*BltRectFunc)(OutputObj,
//                                              InputObj,
//                                              psoMask,
//                                              pxlo,
//                                              &CombinedRect,
//                                              &Pt,
//                                              pptlMask,
//                                              pbo,
//                                              &AdjustedBrushOrigin,
//                                              rop4) && Ret;
//                     }
//                 }
//             }
//             while (EnumMore);
//             break;
//     }
//
//     return Ret;
// }
//
// BOOL APIENTRY
// IntEngBitBlt(
//     SURFOBJ *psoTrg,
//     SURFOBJ *psoSrc,
//     SURFOBJ *psoMask,
//     CLIPOBJ *pco,
//     XLATEOBJ *pxlo,
//     RECTL *prclTrg,
//     POINTL *pptlSrc,
//     POINTL *pptlMask,
//     BRUSHOBJ *pbo,
//     POINTL *pptlBrush,
//     ROP4 Rop4)
// {
//     SURFACE *psurfTrg;
//     SURFACE *psurfSrc = NULL;
//     BOOL bResult;
//     RECTL rclClipped;
//     RECTL rclSrc;
//     RECTL rclSrcClipped;
//     POINTL ptlBrush;
//     PFN_DrvBitBlt pfnBitBlt;
//     LONG lTmp;
//     BOOLEAN bTopToBottom, bLeftToRight;
//
//     /* Sanity checks */
//     ASSERT(IS_VALID_ROP4(Rop4));
//     ASSERT(psoTrg);
//
//     psurfTrg = CONTAINING_RECORD(psoTrg, SURFACE, SurfObj);
//
//     bLeftToRight = prclTrg->left > prclTrg->right;
//     bTopToBottom = prclTrg->top > prclTrg->bottom;
//
//     /* Get the target rect and make it well ordered */
//     rclClipped = *prclTrg;
//     RECTL_vMakeWellOrdered(&rclClipped);
//
//     /* Clip the target rect against the bounds of the target surface */
//     if (!RECTL_bClipRectBySize(&rclClipped, &rclClipped, &psoTrg->sizlBitmap))
//     {
//         /* Nothing left */
//         return TRUE;
//     }
//
//     if (pco)
//     {
//         /* Clip target rect against the bounds of the clipping region */
//         if (!RECTL_bIntersectRect(&rclClipped, &rclClipped, &pco->rclBounds))
//         {
//             /* Nothing left */
//             return TRUE;
//         }
//
//         /* Don't pass a clipobj with only a single rect */
//         if (pco->iDComplexity == DC_RECT)
//             pco = NULL;
//     }
//     else
//         pco = (CLIPOBJ *)&gxcoTrivial;
//
//     if (ROP4_USES_SOURCE(Rop4))
//     {
//         ASSERT(psoSrc);
//         psurfSrc = CONTAINING_RECORD(psoSrc, SURFACE, SurfObj);
//
//         /* Calculate source rect */
//         rclSrc.left = pptlSrc->x + rclClipped.left - prclTrg->left;
//         rclSrc.top = pptlSrc->y + rclClipped.top - prclTrg->top;
//         rclSrc.right = rclSrc.left + rclClipped.right - rclClipped.left;
//         rclSrc.bottom = rclSrc.top + rclClipped.bottom - rclClipped.top;
//
//         /* Clip the source rect against the size of the source surface */
//         if (!RECTL_bClipRectBySize(&rclSrcClipped, &rclSrc, &psoSrc->sizlBitmap))
//         {
//             /* Nothing left */
//             return TRUE;
//         }
//
//         /* Fix up target rect */
//         rclClipped.left += (rclSrcClipped.left - rclSrc.left);
//         rclClipped.top += (rclSrcClipped.top - rclSrc.top);
//         rclClipped.right -= (rclSrc.right - rclSrcClipped.right);
//         rclClipped.bottom -= (rclSrc.bottom - rclSrcClipped.bottom);
//
//         pptlSrc = (PPOINTL)&rclSrcClipped;
//     }
//     else
//     {
//         psoSrc = NULL;
//         psurfSrc = NULL;
//     }
//
//     if (pptlBrush)
//     {
// #ifdef _USE_DIBLIB_
//         ptlBrush.x = pptlBrush->x + rclClipped.left - prclTrg->left;
//         ptlBrush.y = pptlBrush->y + rclClipped.top - prclTrg->top;
// #else
//         ptlBrush = *pptlBrush;
// #endif
//     }
//
//     /* Is the target surface device managed? */
//     if (psurfTrg->flags & HOOK_BITBLT)
//     {
//         /* Is the source a different device managed surface? */
//         if (psoSrc && psoSrc->hdev != psoTrg->hdev && psurfSrc->flags & HOOK_BITBLT)
//         {
//             DPRINT1("Need to copy to standard bitmap format!\n");
//             ASSERT(FALSE);
//         }
//
//         pfnBitBlt = GDIDEVFUNCS(psoTrg).BitBlt;
//     }
//
//     /* Is the source surface device managed? */
//     else if (psoSrc && psurfSrc->flags & HOOK_BITBLT)
//     {
//         pfnBitBlt = GDIDEVFUNCS(psoSrc).BitBlt;
//     }
//     else
//     {
//         pfnBitBlt = EngBitBlt;
//     }
//
//     /* rclClipped needs to be modified in accordance with flips here */
//     if (bLeftToRight)
//     {
//         lTmp = rclClipped.left;
//         rclClipped.left = rclClipped.right;
//         rclClipped.right = lTmp;
//     }
//
//     if (bTopToBottom)
//     {
//         lTmp = rclClipped.top;
//         rclClipped.top = rclClipped.bottom;
//         rclClipped.bottom = lTmp;
//     }
//
//     DPRINT("About to call EngBitBlt: rclClipped: (%d,%d)-(%d,%d)\n",
//            rclClipped.left, rclClipped.top, rclClipped.right, rclClipped.bottom);
//
//     bResult = pfnBitBlt(psoTrg,
//                         psoSrc,
//                         psoMask,
//                         pco,
//                         pxlo,
//                         &rclClipped,
//                         pptlSrc,
//                         pptlMask,
//                         pbo,
//                         pptlBrush ? &ptlBrush : NULL,
//                         Rop4);
//
//     // FIXME: cleanup temp surface!
//
//     return bResult;
// }
//
// #endif // !_USE_DIBLIB_
//
// /**** REACTOS FONT RENDERING CODE *********************************************/
//
// /* renders the alpha mask bitmap */
// static BOOLEAN APIENTRY
// AlphaBltMask(SURFOBJ* psoDest,
//              SURFOBJ* psoSource, // unused
//              SURFOBJ* psoMask,
//              XLATEOBJ* pxloRGB2Dest,
//              XLATEOBJ* pxloBrush,
//              RECTL* prclDest,
//              POINTL* pptlSource, // unused
//              POINTL* pptlMask,
//              BRUSHOBJ* pbo,
//              POINTL* pptlBrush)
// {
//     LONG i, j, dx, dy;
//     int r, g, b;
//     ULONG Background, BrushColor, NewColor;
//     BYTE *tMask, *lMask;
//
//     ASSERT(psoSource == NULL);
//     ASSERT(pptlSource == NULL);
//
//     dx = prclDest->right  - prclDest->left;
//     dy = prclDest->bottom - prclDest->top;
//
//     if (psoMask != NULL)
//     {
//         BrushColor = XLATEOBJ_iXlate(pxloBrush, pbo ? pbo->iSolidColor : 0);
//         r = (int)GetRValue(BrushColor);
//         g = (int)GetGValue(BrushColor);
//         b = (int)GetBValue(BrushColor);
//
//         tMask = (PBYTE)psoMask->pvScan0 + (pptlMask->y * psoMask->lDelta) + pptlMask->x;
//         for (j = 0; j < dy; j++)
//         {
//             lMask = tMask;
//             for (i = 0; i < dx; i++)
//             {
//                 if (*lMask > 0)
//                 {
//                     if (*lMask == 0xff)
//                     {
//                         DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_PutPixel(
//                             psoDest, prclDest->left + i, prclDest->top + j, pbo ? pbo->iSolidColor : 0);
//                     }
//                     else
//                     {
//                         Background = DIB_GetSource(psoDest, prclDest->left + i, prclDest->top + j,
//                                                    pxloBrush);
//
//                         NewColor =
//                             RGB((*lMask * (r - GetRValue(Background)) >> 8) + GetRValue(Background),
//                                 (*lMask * (g - GetGValue(Background)) >> 8) + GetGValue(Background),
//                                 (*lMask * (b - GetBValue(Background)) >> 8) + GetBValue(Background));
//
//                         Background = XLATEOBJ_iXlate(pxloRGB2Dest, NewColor);
//                         DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_PutPixel(
//                             psoDest, prclDest->left + i, prclDest->top + j, Background);
//                     }
//                 }
//                 lMask++;
//             }
//             tMask += psoMask->lDelta;
//         }
//         return TRUE;
//     }
//     else
//     {
//         return FALSE;
//     }
// }
//
// static
// BOOL APIENTRY
// EngMaskBitBlt(SURFOBJ *psoDest,
//               SURFOBJ *psoMask,
//               CLIPOBJ *ClipRegion,
//               XLATEOBJ *DestColorTranslation,
//               XLATEOBJ *SourceColorTranslation,
//               RECTL *DestRect,
//               POINTL *pptlMask,
//               BRUSHOBJ *pbo,
//               POINTL *BrushOrigin)
// {
//     BYTE               clippingType;
//     RECTL              CombinedRect;
//     RECT_ENUM          RectEnum;
//     BOOL               EnumMore;
//     POINTL             InputPoint;
//     RECTL              InputRect;
//     RECTL              OutputRect;
//     POINTL             Translate;
//     INTENG_ENTER_LEAVE EnterLeaveSource;
//     INTENG_ENTER_LEAVE EnterLeaveDest;
//     SURFOBJ*           psoInput;
//     SURFOBJ*           psoOutput;
//     BOOLEAN            Ret = TRUE;
//     RECTL              ClipRect;
//     unsigned           i;
//     POINTL             Pt;
//     ULONG              Direction;
//     POINTL             AdjustedBrushOrigin;
//
//     ASSERT(psoMask);
//
//     if (pptlMask)
//     {
//         InputRect.left = pptlMask->x;
//         InputRect.right = pptlMask->x + (DestRect->right - DestRect->left);
//         InputRect.top = pptlMask->y;
//         InputRect.bottom = pptlMask->y + (DestRect->bottom - DestRect->top);
//     }
//     else
//     {
//         InputRect.left = 0;
//         InputRect.right = DestRect->right - DestRect->left;
//         InputRect.top = 0;
//         InputRect.bottom = DestRect->bottom - DestRect->top;
//     }
//
//     OutputRect = *DestRect;
//     if (NULL != ClipRegion)
//     {
//         if (OutputRect.left < ClipRegion->rclBounds.left)
//         {
//             InputRect.left += ClipRegion->rclBounds.left - OutputRect.left;
//             OutputRect.left = ClipRegion->rclBounds.left;
//         }
//         if (ClipRegion->rclBounds.right < OutputRect.right)
//         {
//             InputRect.right -=  OutputRect.right - ClipRegion->rclBounds.right;
//             OutputRect.right = ClipRegion->rclBounds.right;
//         }
//         if (OutputRect.top < ClipRegion->rclBounds.top)
//         {
//             InputRect.top += ClipRegion->rclBounds.top - OutputRect.top;
//             OutputRect.top = ClipRegion->rclBounds.top;
//         }
//         if (ClipRegion->rclBounds.bottom < OutputRect.bottom)
//         {
//             InputRect.bottom -=  OutputRect.bottom - ClipRegion->rclBounds.bottom;
//             OutputRect.bottom = ClipRegion->rclBounds.bottom;
//         }
//     }
//
//     if (! IntEngEnter(&EnterLeaveSource, psoMask, &InputRect, TRUE, &Translate, &psoInput))
//     {
//         return FALSE;
//     }
//
//     InputPoint.x = InputRect.left + Translate.x;
//     InputPoint.y = InputRect.top + Translate.y;
//
//     /* Check for degenerate case: if height or width of OutputRect is 0 pixels there's
//        nothing to do */
//     if (OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top)
//     {
//         IntEngLeave(&EnterLeaveSource);
//         return TRUE;
//     }
//
//     if (! IntEngEnter(&EnterLeaveDest, psoDest, &OutputRect, FALSE, &Translate, &psoOutput))
//     {
//         IntEngLeave(&EnterLeaveSource);
//         return FALSE;
//     }
//
//     OutputRect.left = DestRect->left + Translate.x;
//     OutputRect.right = DestRect->right + Translate.x;
//     OutputRect.top = DestRect->top + Translate.y;
//     OutputRect.bottom = DestRect->bottom + Translate.y;
//
//     if (BrushOrigin)
//     {
//         AdjustedBrushOrigin.x = BrushOrigin->x + Translate.x;
//         AdjustedBrushOrigin.y = BrushOrigin->y + Translate.y;
//     }
//     else
//         AdjustedBrushOrigin = Translate;
//
//     // Determine clipping type
//     if (ClipRegion == (CLIPOBJ *) NULL)
//     {
//         clippingType = DC_TRIVIAL;
//     } else {
//         clippingType = ClipRegion->iDComplexity;
//     }
//
//     switch (clippingType)
//     {
//         case DC_TRIVIAL:
//             if (psoMask->iBitmapFormat == BMF_8BPP)
//                 Ret = AlphaBltMask(psoOutput, NULL , psoInput, DestColorTranslation, SourceColorTranslation,
//                                    &OutputRect, NULL, &InputPoint, pbo, &AdjustedBrushOrigin);
//             else
//                 Ret = BltMask(psoOutput, NULL, psoInput, DestColorTranslation,
//                               &OutputRect, NULL, &InputPoint, pbo, &AdjustedBrushOrigin,
//                               ROP4_MASK);
//             break;
//         case DC_RECT:
//             // Clip the blt to the clip rectangle
//             ClipRect.left = ClipRegion->rclBounds.left + Translate.x;
//             ClipRect.right = ClipRegion->rclBounds.right + Translate.x;
//             ClipRect.top = ClipRegion->rclBounds.top + Translate.y;
//             ClipRect.bottom = ClipRegion->rclBounds.bottom + Translate.y;
//             if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//             {
//                 Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
//                 Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
//                 if (psoMask->iBitmapFormat == BMF_8BPP)
//                 {
//                     Ret = AlphaBltMask(psoOutput, NULL, psoInput, DestColorTranslation, SourceColorTranslation,
//                                        &CombinedRect, NULL, &Pt, pbo, &AdjustedBrushOrigin);
//                 }
//                 else
//                 {
//                     Ret = BltMask(psoOutput, NULL, psoInput, DestColorTranslation,
//                                   &CombinedRect, NULL, &Pt, pbo, &AdjustedBrushOrigin, ROP4_MASK);
//                 }
//             }
//             break;
//         case DC_COMPLEX:
//             Ret = TRUE;
//             if (psoOutput == psoInput)
//             {
//                 if (OutputRect.top < InputPoint.y)
//                 {
//                     Direction = OutputRect.left < InputPoint.x ? CD_RIGHTDOWN : CD_LEFTDOWN;
//                 }
//                 else
//                 {
//                     Direction = OutputRect.left < InputPoint.x ? CD_RIGHTUP : CD_LEFTUP;
//                 }
//             }
//             else
//             {
//                 Direction = CD_ANY;
//             }
//             CLIPOBJ_cEnumStart(ClipRegion, FALSE, CT_RECTANGLES, Direction, 0);
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(ClipRegion,(ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as u32;
//
//                 for (i = 0; i < RectEnum.c; i++)
//                 {
//                     ClipRect.left = RectEnum.arcl[i].left + Translate.x;
//                     ClipRect.right = RectEnum.arcl[i].right + Translate.x;
//                     ClipRect.top = RectEnum.arcl[i].top + Translate.y;
//                     ClipRect.bottom = RectEnum.arcl[i].bottom + Translate.y;
//                     if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//                     {
//                         Pt.x = InputPoint.x + CombinedRect.left - OutputRect.left;
//                         Pt.y = InputPoint.y + CombinedRect.top - OutputRect.top;
//                         if (psoMask->iBitmapFormat == BMF_8BPP)
//                         {
//                             Ret = AlphaBltMask(psoOutput, NULL, psoInput,
//                                                DestColorTranslation,
//                                                SourceColorTranslation,
//                                                &CombinedRect, NULL, &Pt, pbo,
//                                                &AdjustedBrushOrigin) && Ret;
//                         }
//                         else
//                         {
//                             Ret = BltMask(psoOutput, NULL, psoInput,
//                                           DestColorTranslation, &CombinedRect, NULL,
//                                           &Pt, pbo, &AdjustedBrushOrigin,
//                                           ROP4_MASK) && Ret;
//                         }
//                     }
//                 }
//             }
//             while (EnumMore);
//             break;
//     }
//
//
//     IntEngLeave(&EnterLeaveDest);
//     IntEngLeave(&EnterLeaveSource);
//
//     return Ret;
// }
//
// BOOL
// APIENTRY
// IntEngMaskBlt(
//     _Inout_ SURFOBJ *psoDest,
//     _In_ SURFOBJ *psoMask,
//     _In_ CLIPOBJ *pco,
//     _In_ XLATEOBJ *pxloDest,
//     _In_ XLATEOBJ *pxloSource,
//     _In_ RECTL *prclDest,
//     _In_ POINTL *pptlMask,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg)
// {
//     BOOLEAN ret;
//     RECTL rcDest;
//     POINTL ptMask = {0,0};
//     PSURFACE psurfTemp;
//     RECTL rcTemp;
//
//     ASSERT(psoDest);
//     ASSERT(psoMask);
//
//     /* Is this a 1 BPP mask? */
//     if (psoMask->iBitmapFormat == BMF_1BPP)
//     {
//         /* Use IntEngBitBlt with an appropriate ROP4 */
//         return IntEngBitBlt(psoDest,
//                             NULL,
//                             psoMask,
//                             pco,
//                             pxloDest,
//                             prclDest,
//                             NULL,
//                             pptlMask,
//                             pbo,
//                             pptlBrushOrg,
//                             ROP4_MASKPAINT);
//     }
//
//     ASSERT(psoMask->iBitmapFormat == BMF_8BPP);
//
//     if (pptlMask)
//     {
//         ptMask = *pptlMask;
//     }
//
//     /* Clip against the bounds of the clipping region so we won't try to write
//      * outside the surface */
//     if (pco != NULL)
//     {
//         /* Intersect with the clip bounds and check if everything was clipped */
//         if (!RECTL_bIntersectRect(&rcDest, prclDest, &pco->rclBounds))
//         {
//             return TRUE;
//         }
//
//         /* Adjust the mask point */
//         ptMask.x += rcDest.left - prclDest->left;
//         ptMask.y += rcDest.top - prclDest->top;
//     }
//     else
//     {
//         rcDest = *prclDest;
//     }
//
//     /* Check if the target surface is device managed */
//     if (psoDest->iType != STYPE_BITMAP)
//     {
//         rcTemp.left = 0;
//         rcTemp.top = 0;
//         rcTemp.right = rcDest.right - rcDest.left;
//         rcTemp.bottom = rcDest.bottom - rcDest.top;
//
//         /* Allocate a temporary surface */
//         psurfTemp = SURFACE_AllocSurface(STYPE_BITMAP,
//                                          rcTemp.right,
//                                          rcTemp.bottom,
//                                          psoDest->iBitmapFormat,
//                                          0,
//                                          0,
//                                          0,
//                                          NULL);
//         if (psurfTemp == NULL)
//         {
//             return FALSE;
//         }
//
//         /* Copy the current target surface bits to the temp surface */
//         ret = EngCopyBits(&psurfTemp->SurfObj,
//                           psoDest,
//                           NULL, // pco
//                           NULL, // pxlo
//                           &rcTemp,
//                           (PPOINTL)&rcDest);
//
//         if (ret)
//         {
//             /* Do the operation on the temp surface */
//             ret = EngMaskBitBlt(&psurfTemp->SurfObj,
//                                 psoMask,
//                                 NULL,
//                                 pxloDest,
//                                 pxloSource,
//                                 &rcTemp,
//                                 &ptMask,
//                                 pbo,
//                                 pptlBrushOrg);
//         }
//
//         if (ret)
//         {
//             /* Copy the result back to the dest surface */
//             ret = EngCopyBits(psoDest,
//                               &psurfTemp->SurfObj,
//                               pco,
//                               NULL,
//                               &rcDest,
//                               (PPOINTL)&rcTemp);
//         }
//
//         /* Delete the temp surface */
//         GDIOBJ_vDeleteObject(&psurfTemp->BaseObject);
//     }
//     else
//     {
//         /* Do the operation on the target surface */
//         ret = EngMaskBitBlt(psoDest,
//                             psoMask,
//                             pco,
//                             pxloDest,
//                             pxloSource,
//                             &rcDest,
//                             &ptMask,
//                             pbo,
//                             pptlBrushOrg);
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
