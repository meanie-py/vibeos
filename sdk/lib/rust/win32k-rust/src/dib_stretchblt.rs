use win32k_sys::win32k::*;
use crate::dib_helpers::*;

// DIB_DoRop is cdecl in C (dib.rs). The fn*_GetPixel/PutPixel are LOCAL
// fn-pointer vars below (the rustconv output also phantom-externed them, and
// IS_VALID_ROP4 which is a macro) — dropped. DibFunctionsForBitmapFormat
// (dib.rs) declared with the real [_; 11] layout for correct stride/offsets
// (the win32k_sys bindgen glob is `[DIB_FUNCTIONS; 0]`).
extern "C" {
    fn DIB_DoRop(rop: ROP4, dest: u32, source: u32, pattern: u32) -> u32;
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11];
}

#[no_mangle]
pub unsafe extern "C" fn DIB_XXBPP_StretchBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    MaskSurf: *mut SURFOBJ,
    PatternSurface: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    MaskOrigin: *mut POINTL,
    Brush: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
    ColorTranslation: *mut XLATEOBJ,
    ROP: ROP4,
) -> u8 {
    let mut sx: i32 = 0;
    let mut sy: i32 = 0;
    let mut DesX: i32;
    let mut DesY: i32;

    let mut DstHeight: i32;
    let mut DstWidth: i32;
    let mut SrcHeight: i32;
    let mut SrcWidth: i32;
    let mut MaskCy: i32 = 0;
    let mut SourceCy: i32 = 0;

    let mut Color: u32;
    let mut Dest: u32;
    let mut Source: u32 = 0;
    let mut Pattern: u32 = 0;
    let mut xxBPPMask: u32;
    let mut CanDraw: u8;

    let mut fnSource_GetPixel: PFN_DIB_GetPixel = None;
    let mut fnDest_GetPixel: PFN_DIB_GetPixel;
    let mut fnDest_PutPixel: PFN_DIB_PutPixel;
    let mut fnPattern_GetPixel: PFN_DIB_GetPixel = None;
    let mut fnMask_GetPixel: PFN_DIB_GetPixel = None;

    let mut PatternX: i32 = 0;
    let mut PatternY: i32 = 0;

    let UsesSource: u32 = ROP4_USES_SOURCE(ROP);
    let UsesPattern: u32 = ROP4_USES_PATTERN(ROP);
    let mut bTopToBottom: u8;
    let mut bLeftToRight: u8;

    fnDest_GetPixel = DibFunctionsForBitmapFormat[(*DestSurf).iBitmapFormat as usize].DIB_GetPixel;
    fnDest_PutPixel = DibFunctionsForBitmapFormat[(*DestSurf).iBitmapFormat as usize].DIB_PutPixel;

    DstHeight = (*DestRect).bottom - (*DestRect).top;
    DstWidth = (*DestRect).right - (*DestRect).left;
    SrcHeight = (*SourceRect).bottom - (*SourceRect).top;
    SrcWidth = (*SourceRect).right - (*SourceRect).left;

    if ((SrcWidth < 0) && (DstWidth < 0)) || ((SrcWidth >= 0) && (DstWidth >= 0)) {
        bLeftToRight = 0;
    } else {
        bLeftToRight = 1;
    }

    if ((SrcHeight < 0) && (DstHeight < 0)) || ((SrcHeight >= 0) && (DstHeight >= 0)) {
        bTopToBottom = 0;
    } else {
        bTopToBottom = 1;
    }

    RECTL_vMakeWellOrdered(DestRect);

    if UsesSource != 0 {
        SourceCy = (*SourceSurf).sizlBitmap.cy;
        fnSource_GetPixel = DibFunctionsForBitmapFormat[(*SourceSurf).iBitmapFormat as usize].DIB_GetPixel;
    }

    if !MaskSurf.is_null() {
        fnMask_GetPixel = DibFunctionsForBitmapFormat[(*MaskSurf).iBitmapFormat as usize].DIB_GetPixel;
        MaskCy = (*MaskSurf).sizlBitmap.cy;
    }

    DstHeight = (*DestRect).bottom - (*DestRect).top;
    DstWidth = (*DestRect).right - (*DestRect).left;
    SrcHeight = (*SourceRect).bottom - (*SourceRect).top;
    SrcWidth = (*SourceRect).right - (*SourceRect).left;

    match (*DestSurf).iBitmapFormat {
        BMF_1BPP => xxBPPMask = 0x1,
        BMF_4BPP => xxBPPMask = 0xF,
        BMF_8BPP => xxBPPMask = 0xFF,
        BMF_16BPP => xxBPPMask = 0xFFFF,
        BMF_24BPP => xxBPPMask = 0xFFFFFF,
        _ => xxBPPMask = 0xFFFFFFFF,
    }

    if UsesPattern != 0 {
        if !PatternSurface.is_null() {
            PatternY = ((*DestRect).top - (*BrushOrigin).y) % (*PatternSurface).sizlBitmap.cy;
            if PatternY < 0 {
                PatternY += (*PatternSurface).sizlBitmap.cy;
            }
            fnPattern_GetPixel = DibFunctionsForBitmapFormat[(*PatternSurface).iBitmapFormat as usize].DIB_GetPixel;
        } else {
            if !Brush.is_null() {
                Pattern = (*Brush).iSolidColor;
            }
        }
    }

    DesY = (*DestRect).top;
    while DesY < (*DestRect).bottom {
        if !PatternSurface.is_null() {
            PatternX = ((*DestRect).left - (*BrushOrigin).x) % (*PatternSurface).sizlBitmap.cx;
            if PatternX < 0 {
                PatternX += (*PatternSurface).sizlBitmap.cx;
            }
        }
        if UsesSource != 0 {
            if bTopToBottom != 0 {
                sy = (*SourceRect).bottom - (DesY - (*DestRect).top) * SrcHeight / DstHeight;
            } else {
                sy = (*SourceRect).top + (DesY - (*DestRect).top) * SrcHeight / DstHeight;
            }
        }

        DesX = (*DestRect).left;
        while DesX < (*DestRect).right {
            CanDraw = 1;

            if fnMask_GetPixel.is_some() {
                if bLeftToRight != 0 {
                    sx = (*SourceRect).right - (DesX - (*DestRect).left) * SrcWidth / DstWidth;
                } else {
                    sx = (*SourceRect).left + (DesX - (*DestRect).left) * SrcWidth / DstWidth;
                }
                if sx < 0 || sy < 0 ||
                    (*MaskSurf).sizlBitmap.cx <= sx || MaskCy <= sy ||
                    fnMask_GetPixel.unwrap()(MaskSurf, sx, sy) != 0
                {
                    CanDraw = 0;
                }
            }

            if UsesSource != 0 && CanDraw != 0 {
                if bLeftToRight != 0 {
                    sx = (*SourceRect).right - (DesX - (*DestRect).left) * SrcWidth / DstWidth;
                } else {
                    sx = (*SourceRect).left + (DesX - (*DestRect).left) * SrcWidth / DstWidth;
                }
                if sx >= 0 && sy >= 0 &&
                    (*SourceSurf).sizlBitmap.cx > sx && SourceCy > sy
                {
                    Source = XLATEOBJ_iXlate(ColorTranslation, fnSource_GetPixel.unwrap()(SourceSurf, sx, sy));
                } else {
                    Source = 0;
                    CanDraw = if (ROP & 0xFF) != R3_OPINDEX_SRCCOPY { 1 } else { 0 };
                }
            }

            if CanDraw != 0 {
                if UsesPattern != 0 && !PatternSurface.is_null() {
                    Pattern = fnPattern_GetPixel.unwrap()(PatternSurface, PatternX, PatternY);
                    PatternX += 1;
                    PatternX %= (*PatternSurface).sizlBitmap.cx;
                }

                Dest = fnDest_GetPixel.unwrap()(DestSurf, DesX, DesY);
                Color = DIB_DoRop(ROP, Dest, Source, Pattern) & xxBPPMask;

                fnDest_PutPixel.unwrap()(DestSurf, DesX, DesY, Color);
            }

            DesX += 1;
        }

        if !PatternSurface.is_null() {
            PatternY += 1;
            PatternY %= (*PatternSurface).sizlBitmap.cy;
        }

        DesY += 1;
    }

    1
}
