use win32k_sys::win32k::*;
use core::ffi::c_void;
use crate::dib_helpers::*;

extern "C" {
    fn DIB_DoRop(Rop4: u32, Dest: u32, Source: u32, Pattern: u32) -> u32;
    fn DIB_4BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    fn DIB_8BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    fn DIB_16BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    fn DIB_24BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    fn DIB_32BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32;
    fn DbgPrint(format: *const u8, ...) -> i32;
}

// ---- file-scope preamble (translated) ----
pub const DEC_OR_INC: () = ();

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_PutPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32, c: u32) {
    let addr = ((*SurfObj).pvScan0 as PBYTE).offset((y * (*SurfObj).lDelta + (x >> 3)) as isize);

    if 0 == (c & 0x01) {
        *addr &= !MASK1BPP(x);
    } else {
        *addr |= MASK1BPP(x);
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_GetPixel(SurfObj: *mut SURFOBJ, x: i32, y: i32) -> u32 {
    
    let addr: PBYTE = ((*SurfObj).pvScan0 as PBYTE)
        .offset((y * (*SurfObj).lDelta) as isize)
        .offset((x >> 3) as isize);
    
    if (*addr & MASK1BPP(x)) != 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    let mut x1 = x1;
    while x1 < x2 {
        DIB_1BPP_PutPixel(SurfObj, x1, y, c);
        x1 += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    let mut y1 = y1;
    while y1 < y2 {
        DIB_1BPP_PutPixel(SurfObj, x, y1, c);
        y1 += 1;
    }
}

unsafe fn DIB_1BPP_BitBltSrcCopy_From1BPP(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    pxlo: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourcePoint: *mut POINTL,
    bTopToBottom: u8,
    bLeftToRight: u8
) -> u8 {

    let Height = RECTL_lGetHeight(DestRect);
    let Width = RECTL_lGetWidth(DestRect);
    let XorBit = if XLATEOBJ_iXlate(pxlo, 0) != 0 { 1u8 } else { 0u8 };
    // C: `IgnoreSrc = XorBit ^ !XLATEOBJ_iXlate(pxlo, 1)` — the translation had
    // dropped the `!`, inverting all four cases: every identity 1bpp->1bpp
    // SRCCOPY zero-filled its destination (all-opaque masks -> the black
    // shortcut-icon class; also broke drag images / ImageList_Merge).
    let IgnoreSrc = if XorBit != 0 {
        XLATEOBJ_iXlate(pxlo, 1) != 0
    } else {
        XLATEOBJ_iXlate(pxlo, 1) == 0
    };
    let mut Overlap: u8 = 0u8;
    let mut DstStart: *mut u8;
    let mut DstEnd: *mut u8;
    let mut SrcStart: *mut u8;
    let mut SrcEnd: *mut u8;

    DstStart = ((*DestSurf).pvScan0 as *mut u8)
        .offset(((*DestRect).top * (*DestSurf).lDelta) as isize)
        .offset(((*DestRect).left / 8) as isize);
    DstEnd = ((*DestSurf).pvScan0 as *mut u8)
        .offset((((*DestRect).bottom - 1) * (*DestSurf).lDelta) as isize)
        .offset(((*DestRect).right / 8) as isize);
    SrcStart = ((*SourceSurf).pvScan0 as *mut u8)
        .offset(((*SourcePoint).y * (*SourceSurf).lDelta) as isize)
        .offset(((*SourcePoint).x / 8) as isize);
    SrcEnd = ((*SourceSurf).pvScan0 as *mut u8)
        .offset((((*SourcePoint).y + Height - 1) * (*SourceSurf).lDelta) as isize)
        .offset((((*SourcePoint).x + Width) / 8) as isize);

    if DstStart > DstEnd {
        let tmp = DstStart;
        DstStart = DstEnd;
        DstEnd = tmp;
    }

    if SrcStart > SrcEnd {
        let tmp = SrcStart;
        SrcStart = SrcEnd;
        SrcEnd = tmp;
    }

    Overlap = if ((SrcStart >= DstStart) && (SrcStart < DstEnd)) || ((SrcEnd >= DstStart) && (SrcEnd < DstEnd)) {
        1u8
    } else {
        0u8
    };

    if IgnoreSrc {
        let mut y: i32 = 0;
        while y < Height {
            let mut x: i32 = 0;
            while x < Width {
                DIB_1BPP_PutPixel(DestSurf, (*DestRect).left + x, (*DestRect).top + y, XorBit as u32);
                x += 1;
            }
            y += 1;
        }
    } else if Overlap == 0 {
        let mut y: i32 = 0;
        while y < Height {
            let ySrc = if bTopToBottom != 0 { (*SourcePoint).y + Height - y - 1 } else { (*SourcePoint).y + y };
            let mut x: i32 = 0;

            while x < Width {
                let xSrc = if bLeftToRight != 0 { (*SourcePoint).x + Width - x - 1 } else { (*SourcePoint).x + x };
                let mut Pixel = DIB_1BPP_GetPixel(SourceSurf, xSrc, ySrc);
                if XorBit != 0 {
                    Pixel = if Pixel != 0 { 0 } else { 1 };
                }
                DIB_1BPP_PutPixel(DestSurf, (*DestRect).left + x, (*DestRect).top + y, Pixel);
                x += 1;
            }
            y += 1;
        }
    } else {
        let mut y: i32;
        let PixBuf = ExAllocatePoolZero(PagedPool as u32, ((Height as usize) * (ALIGN_UP_BY(Width as usize, 8) / 8)) as SIZE_T, 0x20424944u32) as *mut u8;
        if PixBuf.is_null() {
            return 0u8;
        }
        y = 0;
        while y < Height {
            let Row = PixBuf.offset((y * (ALIGN_UP_BY(Width as usize, 8) / 8) as i32) as isize);
            let ySrc = if bTopToBottom != 0 { (*SourcePoint).y + Height - y - 1 } else { (*SourcePoint).y + y };
            let mut x: i32 = 0;

            while x < Width {
                let xSrc = if bLeftToRight != 0 { (*SourcePoint).x + Width - x - 1 } else { (*SourcePoint).x + x };
                if (DIB_1BPP_GetPixel(SourceSurf, xSrc, ySrc) == 0) == (XorBit != 0) {
                    *Row.offset((x / 8) as isize) |= 1 << (x & 7);
                }
                x += 1;
            }
            y += 1;
        }

        y = 0;
        while y < Height {
            let Row = PixBuf.offset((y * (ALIGN_UP_BY(Width as usize, 8) / 8) as i32) as isize);
            let mut x: i32 = 0;
            while x < Width {
                DIB_1BPP_PutPixel(DestSurf, (*DestRect).left + x, (*DestRect).top + y, 
                    if (*Row.offset((x / 8) as isize) & (1 << (x & 7))) != 0 { 1 } else { 0 });
                x += 1;
            }
            y += 1;
        }

        ExFreePoolWithTag(PixBuf as *mut core::ffi::c_void, 0x20424944u32);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_BitBltSrcCopy(BltInfo: PBLTINFO) -> u8 {

    let mut Color: u32;
    let mut i: i32;
    let mut j: i32;
    let mut sx: i32;
    let mut sy: i32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;

    sy = (*BltInfo).SourcePoint.y;

    bLeftToRight = if (*BltInfo).DestRect.left > (*BltInfo).DestRect.right { 1 } else { 0 };
    bTopToBottom = if (*BltInfo).DestRect.top > (*BltInfo).DestRect.bottom { 1 } else { 0 };

    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!((*BltInfo).DestRect));

    match (*(*BltInfo).SourceSurface).iBitmapFormat {
        BMF_1BPP => {
            return DIB_1BPP_BitBltSrcCopy_From1BPP(
                (*BltInfo).DestSurface,
                (*BltInfo).SourceSurface,
                (*BltInfo).XlateSourceToDest,
                core::ptr::addr_of_mut!((*BltInfo).DestRect),
                core::ptr::addr_of_mut!((*BltInfo).SourcePoint),
                bTopToBottom,
                bLeftToRight
            );
        },

        BMF_4BPP => {
            if bTopToBottom != 0 {
                sy += ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    Color = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, DIB_4BPP_GetPixel((*BltInfo).SourceSurface, sx, sy));
                    DIB_1BPP_PutPixel((*BltInfo).DestSurface, i, j, Color);

                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        },

        BMF_8BPP => {
            if bTopToBottom != 0 {
                sy += ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    Color = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, DIB_8BPP_GetPixel((*BltInfo).SourceSurface, sx, sy));
                    DIB_1BPP_PutPixel((*BltInfo).DestSurface, i, j, Color);

                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        },

        BMF_16BPP => {
            if bTopToBottom != 0 {
                sy += ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    Color = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, DIB_16BPP_GetPixel((*BltInfo).SourceSurface, sx, sy));
                    DIB_1BPP_PutPixel((*BltInfo).DestSurface, i, j, Color);
                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        },

        BMF_24BPP => {
            if bTopToBottom != 0 {
                sy += ((*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1) * (*(*BltInfo).SourceSurface).lDelta;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    Color = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, DIB_24BPP_GetPixel((*BltInfo).SourceSurface, sx, sy));
                    DIB_1BPP_PutPixel((*BltInfo).DestSurface, i, j, Color);
                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        },

        BMF_32BPP => {
            if bTopToBottom != 0 {
                sy += (*BltInfo).DestRect.bottom - (*BltInfo).DestRect.top - 1;
            }

            j = (*BltInfo).DestRect.top;
            while j < (*BltInfo).DestRect.bottom {
                sx = (*BltInfo).SourcePoint.x;

                if bLeftToRight != 0 {
                    sx += (*BltInfo).DestRect.right - (*BltInfo).DestRect.left - 1;
                }

                i = (*BltInfo).DestRect.left;
                while i < (*BltInfo).DestRect.right {
                    Color = XLATEOBJ_iXlate((*BltInfo).XlateSourceToDest, DIB_32BPP_GetPixel((*BltInfo).SourceSurface, sx, sy));
                    DIB_1BPP_PutPixel((*BltInfo).DestSurface, i, j, Color);
                    sx = if bLeftToRight != 0 { sx - 1 } else { sx + 1 };
                    i += 1;
                }
                sy = if bTopToBottom != 0 { sy - 1 } else { sy + 1 };
                j += 1;
            }
        },

        _ => {
            DbgPrint(b"DIB_1BPP_BitBlt: Unhandled Source BPP: %u\n\0".as_ptr(), BitsPerFormat((*(*BltInfo).SourceSurface).iBitmapFormat));
            return 0u8;
        }
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_BitBlt(BltInfo: PBLTINFO) -> u8 {

    let mut DestX: i32;
    let mut DestY: i32;
    let mut SourceX: i32;
    let mut SourceY: i32;
    let mut PatternY: i32 = 0;
    let mut Dest: u32;
    let mut Source: u32 = 0;
    let mut Pattern: u32 = 0;
    let mut Index: u32;
    let UsesSource: u32;
    let UsesPattern: u32;
    let mut DestBits: *mut u32;
    let RoundedRight: i32;

    UsesSource = ROP4_USES_SOURCE((*BltInfo).Rop4);
    UsesPattern = ROP4_USES_PATTERN((*BltInfo).Rop4);

    RoundedRight = (*BltInfo).DestRect.right -
        (((*BltInfo).DestRect.right - (*BltInfo).DestRect.left) & 31);
    SourceY = (*BltInfo).SourcePoint.y;

    if UsesPattern != 0 {
        if !(*BltInfo).PatternSurface.is_null() {
            PatternY = ((*BltInfo).DestRect.top + (*BltInfo).BrushOrigin.y) %
                (*(*BltInfo).PatternSurface).sizlBitmap.cy;
        } else {
            if !(*BltInfo).Brush.is_null() {
                Pattern = (*(*BltInfo).Brush).iSolidColor;
            }
        }
    }

    DestY = (*BltInfo).DestRect.top;
    while DestY < (*BltInfo).DestRect.bottom {
        DestX = (*BltInfo).DestRect.left;
        SourceX = (*BltInfo).SourcePoint.x;
        DestBits = (((*(*BltInfo).DestSurface).pvScan0 as PBYTE)
            .offset(((*BltInfo).DestRect.left >> 3) as isize)
            .offset((DestY * (*(*BltInfo).DestSurface).lDelta) as isize)) as *mut u32;

        if (DestX & 31) != 0 {
        }

        while DestX < RoundedRight {
            Dest = *DestBits;

            if UsesSource != 0 {
                Source = 0;
                Index = 0;
                while Index < 8 {
                    Source |= DIB_GetSource((*BltInfo).SourceSurface, SourceX + Index as i32, SourceY, (*BltInfo).XlateSourceToDest) << (7 - Index);
                    Source |= DIB_GetSource((*BltInfo).SourceSurface, SourceX + Index as i32 + 8, SourceY, (*BltInfo).XlateSourceToDest) << (8 + (7 - Index));
                    Source |= DIB_GetSource((*BltInfo).SourceSurface, SourceX + Index as i32 + 16, SourceY, (*BltInfo).XlateSourceToDest) << (16 + (7 - Index));
                    Source |= DIB_GetSource((*BltInfo).SourceSurface, SourceX + Index as i32 + 24, SourceY, (*BltInfo).XlateSourceToDest) << (24 + (7 - Index));
                    Index += 1;
                }
            }

            if !(*BltInfo).PatternSurface.is_null() {
                Pattern = 0;
                Index = 0;
                while Index < 8 {
                    Pattern |= DIB_GetSourceIndex((*BltInfo).PatternSurface, (DestX + (*BltInfo).BrushOrigin.x + Index as i32) % (*(*BltInfo).PatternSurface).sizlBitmap.cx, PatternY) << (7 - Index);
                    Pattern |= DIB_GetSourceIndex((*BltInfo).PatternSurface, (DestX + (*BltInfo).BrushOrigin.x + Index as i32 + 8) % (*(*BltInfo).PatternSurface).sizlBitmap.cx, PatternY) << (8 + (7 - Index));
                    Pattern |= DIB_GetSourceIndex((*BltInfo).PatternSurface, (DestX + (*BltInfo).BrushOrigin.x + Index as i32 + 16) % (*(*BltInfo).PatternSurface).sizlBitmap.cx, PatternY) << (16 + (7 - Index));
                    Pattern |= DIB_GetSourceIndex((*BltInfo).PatternSurface, (DestX + (*BltInfo).BrushOrigin.x + Index as i32 + 24) % (*(*BltInfo).PatternSurface).sizlBitmap.cx, PatternY) << (24 + (7 - Index));
                    Index += 1;
                }
            }

            *DestBits = DIB_DoRop((*BltInfo).Rop4, Dest, Source, Pattern);

            DestX += 32;
            DestBits = DestBits.offset(1);
            SourceX += 32;
        }

        if DestX < (*BltInfo).DestRect.right {
            while DestX < (*BltInfo).DestRect.right {
                Dest = DIB_1BPP_GetPixel((*BltInfo).DestSurface, DestX, DestY);

                if UsesSource != 0 {
                    Source = DIB_GetSource((*BltInfo).SourceSurface, SourceX, SourceY, (*BltInfo).XlateSourceToDest);
                }

                if !(*BltInfo).PatternSurface.is_null() {
                    Pattern = DIB_GetSourceIndex((*BltInfo).PatternSurface, (DestX + (*BltInfo).BrushOrigin.x) % (*(*BltInfo).PatternSurface).sizlBitmap.cx, PatternY);
                }

                DIB_1BPP_PutPixel((*BltInfo).DestSurface, DestX, DestY, DIB_DoRop((*BltInfo).Rop4, Dest, Source, Pattern) & 0xF);

                DestX += 1;
                SourceX += 1;
            }
        }

        SourceY += 1;
        if !(*BltInfo).PatternSurface.is_null() {
            PatternY += 1;
            PatternY %= (*(*BltInfo).PatternSurface).sizlBitmap.cy;
        }

        DestY += 1;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_ColorFill(DestSurface: *mut SURFOBJ, DestRect: *mut RECTL, color: u32) -> u8 {

    RECTL_vMakeWellOrdered(DestRect);

    let mut DestY = (*DestRect).top;
    while DestY < (*DestRect).bottom {
        DIB_1BPP_HLine(DestSurface, (*DestRect).left, (*DestRect).right, DestY, color);
        DestY += 1;
    }
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn DIB_1BPP_TransparentBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ColorTranslation: *mut XLATEOBJ,
    iTransColor: u32,
) -> u8 {
    0u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            win32ss/gdi/dib/dib1bpp.c
//  * PURPOSE:         Device Independant Bitmap functions, 1bpp
//  * PROGRAMMERS:     Jason Filby
//  *                  Doug Lyons
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define DEC_OR_INC(var, decTrue, amount) \
//     ((var) = (decTrue) ? ((var) - (amount)) : ((var) + (amount)))
//
// VOID
// DIB_1BPP_PutPixel(SURFOBJ *SurfObj, LONG x, LONG y, ULONG c)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + (x >> 3);
//
//   if (0 == (c & 0x01))
//     *addr &= ~MASK1BPP(x);
//   else
//     *addr |= MASK1BPP(x);
// }
//
// ULONG
// DIB_1BPP_GetPixel(SURFOBJ *SurfObj, LONG x, LONG y)
// {
//   PBYTE addr = (PBYTE)SurfObj->pvScan0 + y * SurfObj->lDelta + (x >> 3);
//
//   return (*addr & MASK1BPP(x) ? 1 : 0);
// }
//
// VOID
// DIB_1BPP_HLine(SURFOBJ *SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   while(x1 < x2)
//   {
//     DIB_1BPP_PutPixel(SurfObj, x1, y, c);
//     x1++;
//   }
// }
//
// VOID
// DIB_1BPP_VLine(SURFOBJ *SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
//   while(y1 < y2)
//   {
//     DIB_1BPP_PutPixel(SurfObj, x, y1, c);
//     y1++;
//   }
// }
//
// static
// BOOLEAN
// DIB_1BPP_BitBltSrcCopy_From1BPP (
//                                  SURFOBJ* DestSurf,
//                                  SURFOBJ* SourceSurf,
//                                  XLATEOBJ* pxlo,
//                                  PRECTL DestRect,
//                                  POINTL *SourcePoint,
//                                  BOOLEAN bTopToBottom,
//                                  BOOLEAN bLeftToRight )
// {
//     LONG Height = RECTL_lGetHeight(DestRect);
//     LONG Width = RECTL_lGetWidth(DestRect);
//     BOOLEAN XorBit = !!XLATEOBJ_iXlate(pxlo, 0);
//     BOOLEAN IgnoreSrc = XorBit ^ !XLATEOBJ_iXlate(pxlo, 1);
//     BOOLEAN Overlap = FALSE;
//     BYTE *DstStart, *DstEnd, *SrcStart, *SrcEnd;
//
//     /* Make sure this is as expected */
//     ASSERT(DestRect->left >= 0);
//     ASSERT(DestRect->top >= 0);
//     ASSERT(Height > 0);
//     ASSERT(Width > 0);
//
//     /*
//      * Check if we need to allocate a buffer for our operation.
//      */
//     DstStart = (BYTE*)DestSurf->pvScan0 + DestRect->top * DestSurf->lDelta + DestRect->left / 8;
//     DstEnd = (BYTE*)DestSurf->pvScan0 + (DestRect->bottom - 1) * DestSurf->lDelta + (DestRect->right) / 8;
//     SrcStart = (BYTE*)SourceSurf->pvScan0 + SourcePoint->y * SourceSurf->lDelta + SourcePoint->x / 8;
//     SrcEnd = (BYTE*)SourceSurf->pvScan0 + (SourcePoint->y + Height - 1) * SourceSurf->lDelta + (SourcePoint->x + Width) / 8;
//
//     /* Beware of bitmaps with negative pitch! */
//     if (DstStart > DstEnd)
//     {
//         BYTE* tmp = DstStart;
//         DstStart = DstEnd;
//         DstEnd = tmp;
//     }
//
//     if (SrcStart > SrcEnd)
//     {
//         BYTE* tmp = SrcStart;
//         SrcStart = SrcEnd;
//         SrcEnd = tmp;
//     }
//
//     /* We allocate a new buffer when the two buffers overlap */
//     Overlap = ((SrcStart >= DstStart) && (SrcStart < DstEnd)) || ((SrcEnd >= DstStart) && (SrcEnd < DstEnd));
//
//     if (IgnoreSrc)
//     {
//         LONG y;
//         for (y = 0; y < Height; y++)
//         {
//             for(LONG x = 0; x < Width; x++)
//             {
//                 DIB_1BPP_PutPixel(DestSurf, DestRect->left + x, DestRect->top + y, XorBit);
//             }
//         }
//     }
//     else if (!Overlap)
//     {
//         LONG y;
//         for (y = 0; y < Height; y++)
//         {
//             LONG ySrc = bTopToBottom ? SourcePoint->y + Height - y - 1 : SourcePoint->y + y;
//             LONG x;
//
//             for(x = 0; x < Width; x++)
//             {
//                 LONG xSrc = bLeftToRight ? SourcePoint->x + Width - x - 1 : SourcePoint->x + x;
//                 ULONG Pixel = DIB_1BPP_GetPixel(SourceSurf, xSrc, ySrc);
//                 if (XorBit)
//                     Pixel = !Pixel;
//                 DIB_1BPP_PutPixel(DestSurf, DestRect->left + x, DestRect->top + y, Pixel);
//             }
//         }
//     }
//     else
//     {
//         LONG y;
//         BYTE* PixBuf = ExAllocatePoolZero(PagedPool, Height * (ALIGN_UP_BY(Width, 8) / 8), TAG_DIB);
//         if (!PixBuf)
//             return FALSE;
//         for (y = 0; y < Height; y++)
//         {
//             BYTE* Row = PixBuf + y * ALIGN_UP_BY(Width, 8) / 8;
//             LONG ySrc = bTopToBottom ? SourcePoint->y + Height - y - 1 : SourcePoint->y + y;
//             LONG x;
//
//             for (x = 0; x < Width; x++)
//             {
//                 LONG xSrc = bLeftToRight ? SourcePoint->x + Width - x - 1 : SourcePoint->x + x;
//                 if ((!DIB_1BPP_GetPixel(SourceSurf, xSrc, ySrc)) == XorBit)
//                     Row[x / 8] |= 1 << (x & 7);
//             }
//         }
//
//         for (y = 0; y < Height; y++)
//         {
//             BYTE* Row = PixBuf + y * ALIGN_UP_BY(Width, 8) / 8;
//             LONG x;
//             for (x = 0; x < Width; x++)
//                 DIB_1BPP_PutPixel(DestSurf, DestRect->left + x, DestRect->top + y, !!(Row[x/8] & (1 << (x&7))));
//         }
//
//         ExFreePoolWithTag(PixBuf, TAG_DIB);
//     }
//
//     return TRUE;
// }
//
// BOOLEAN
// DIB_1BPP_BitBltSrcCopy(PBLTINFO BltInfo)
// {
//   ULONG Color;
//   LONG i, j, sx, sy;
//   BOOLEAN bTopToBottom, bLeftToRight;
//
//   // This sets sy to the top line
//   sy = BltInfo->SourcePoint.y;
//
//   DPRINT("DIB_1BPP_BitBltSrcCopy: SrcSurf cx/cy (%d/%d), DestSuft cx/cy (%d/%d) dstRect: (%d,%d)-(%d,%d)\n",
//          BltInfo->SourceSurface->sizlBitmap.cx, BltInfo->SourceSurface->sizlBitmap.cy,
//          BltInfo->DestSurface->sizlBitmap.cx, BltInfo->DestSurface->sizlBitmap.cy,
//          BltInfo->DestRect.left, BltInfo->DestRect.top, BltInfo->DestRect.right, BltInfo->DestRect.bottom);
//
//   /* Get back left to right flip here */
//   bLeftToRight = (BltInfo->DestRect.left > BltInfo->DestRect.right);
//
//   /* Check for top to bottom flip needed. */
//   bTopToBottom = BltInfo->DestRect.top > BltInfo->DestRect.bottom;
//
//   // Make WellOrdered with top < bottom and left < right
//   RECTL_vMakeWellOrdered(&BltInfo->DestRect);
//
//   DPRINT("BPP is '%d' & BltInfo->SourcePoint.x is '%d' & BltInfo->SourcePoint.y is '%d'.\n",
//          BltInfo->SourceSurface->iBitmapFormat, BltInfo->SourcePoint.x, BltInfo->SourcePoint.y);
//
//   switch ( BltInfo->SourceSurface->iBitmapFormat )
//   {
//   case BMF_1BPP:
//     DPRINT("1BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     return DIB_1BPP_BitBltSrcCopy_From1BPP ( BltInfo->DestSurface, BltInfo->SourceSurface,
//       BltInfo->XlateSourceToDest, &BltInfo->DestRect, &BltInfo->SourcePoint,
//       bTopToBottom, bLeftToRight );
//
//   case BMF_4BPP:
//     DPRINT("4BPP Case Selected with DestRect Width of '%d'.\n",
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     if (bTopToBottom)
//     {
//       // This sets sy to the bottom line
//       sy += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//        // This sets the sx to the rightmost pixel
//        sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         Color = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, DIB_4BPP_GetPixel(BltInfo->SourceSurface, sx, sy));
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, i, j, Color);
//
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_8BPP:
//     DPRINT("8BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     if (bTopToBottom)
//     {
//       // This sets sy to the bottom line
//       sy += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         // This sets sx to the rightmost pixel
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         Color = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, DIB_8BPP_GetPixel(BltInfo->SourceSurface, sx, sy));
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, i, j, Color);
//
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_16BPP:
//     DPRINT("16BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     if (bTopToBottom)
//     {
//       // This sets sy to the bottom line
//       sy += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;;
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         // This sets the sx to the rightmost pixel
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         Color = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, DIB_16BPP_GetPixel(BltInfo->SourceSurface, sx, sy));
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, i, j, Color);
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_24BPP:
//
//     DPRINT("24BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//       if (bTopToBottom)
//       {
//         // This sets sy to the bottom line
//         sy += (BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1) * BltInfo->SourceSurface->lDelta;
//       }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         // This sets the sx to the rightmost pixel
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         Color = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, DIB_24BPP_GetPixel(BltInfo->SourceSurface, sx, sy));
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, i, j, Color);
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   case BMF_32BPP:
//
//     DPRINT("32BPP-dstRect: (%d,%d)-(%d,%d) and Width of '%d'.\n",
//            BltInfo->DestRect.left, BltInfo->DestRect.top,
//            BltInfo->DestRect.right, BltInfo->DestRect.bottom,
//            BltInfo->DestRect.right - BltInfo->DestRect.left);
//
//     if (bTopToBottom)
//     {
//       // This sets sy to the bottom line
//       sy += BltInfo->DestRect.bottom - BltInfo->DestRect.top - 1;
//     }
//
//     for (j=BltInfo->DestRect.top; j<BltInfo->DestRect.bottom; j++)
//     {
//       sx = BltInfo->SourcePoint.x;
//
//       if (bLeftToRight)
//       {
//         // This sets the sx to the rightmost pixel
//         sx += (BltInfo->DestRect.right - BltInfo->DestRect.left - 1);
//       }
//
//       for (i=BltInfo->DestRect.left; i<BltInfo->DestRect.right; i++)
//       {
//         Color = XLATEOBJ_iXlate(BltInfo->XlateSourceToDest, DIB_32BPP_GetPixel(BltInfo->SourceSurface, sx, sy));
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, i, j, Color);
//         DEC_OR_INC(sx, bLeftToRight, 1);
//       }
//       DEC_OR_INC(sy, bTopToBottom, 1);
//     }
//     break;
//
//   default:
//     DbgPrint("DIB_1BPP_BitBlt: Unhandled Source BPP: %u\n", BitsPerFormat(BltInfo->SourceSurface->iBitmapFormat));
//     return FALSE;
//   }
//
//   return TRUE;
// }
//
// #ifndef _USE_DIBLIB_
// BOOLEAN
// DIB_1BPP_BitBlt(PBLTINFO BltInfo)
// {
//   LONG DestX, DestY;
//   LONG SourceX, SourceY;
//   LONG PatternY = 0;
//   ULONG Dest, Source = 0, Pattern = 0;
//   ULONG Index;
//   BOOLEAN UsesSource;
//   BOOLEAN UsesPattern;
//   PULONG DestBits;
//   LONG RoundedRight;
//
//   UsesSource = ROP4_USES_SOURCE(BltInfo->Rop4);
//   UsesPattern = ROP4_USES_PATTERN(BltInfo->Rop4);
//
//   RoundedRight = BltInfo->DestRect.right -
//     ((BltInfo->DestRect.right - BltInfo->DestRect.left) & 31);
//   SourceY = BltInfo->SourcePoint.y;
//
//   if (UsesPattern)
//   {
//     if (BltInfo->PatternSurface)
//     {
//       PatternY = (BltInfo->DestRect.top + BltInfo->BrushOrigin.y) %
//         BltInfo->PatternSurface->sizlBitmap.cy;
//     }
//     else
//     {
//       /* FIXME: Shouldn't it be expanded? */
//       if (BltInfo->Brush)
//         Pattern = BltInfo->Brush->iSolidColor;
//     }
//   }
//
//   for (DestY = BltInfo->DestRect.top; DestY < BltInfo->DestRect.bottom; DestY++)
//   {
//     DestX = BltInfo->DestRect.left;
//     SourceX = BltInfo->SourcePoint.x;
//     DestBits = (PULONG)(
//       (PBYTE)BltInfo->DestSurface->pvScan0 +
//       (BltInfo->DestRect.left >> 3) +
//       DestY * BltInfo->DestSurface->lDelta);
//
//     if (DestX & 31)
//     {
// #if 0
//       /* FIXME: This case is completely untested!!! */
//
//       Dest = *((PBYTE)DestBits);
//       NoBits = 31 - (DestX & 31);
//
//       if (UsesSource)
//       {
//         Source = 0;
//         /* FIXME: This is incorrect! */
//         for (Index = 31 - NoBits; Index >= 0; Index++)
//           Source |= (DIB_GetSource(SourceSurf, SourceX + Index, SourceY, ColorTranslation) << (31 - Index));
//       }
//
//       if (BltInfo->PatternSurface)
//       {
//         Pattern = 0;
//         for (k = 31 - NoBits; k >= 0; k++)
//           Pattern |= (DIB_GetSourceIndex(PatternObj, (X + BrushOrigin.x + k) % PatternWidth, PatternY) << (31 - k));
//       }
//
//       Dest = DIB_DoRop(BltInfo->Rop4, Dest, Source, Pattern);
//       Dest &= ~((1 << (31 - NoBits)) - 1);
//       Dest |= *((PBYTE)DestBits) & ((1 << (31 - NoBits)) - 1);
//
//       *DestBits = Dest;
//
//       DestX += NoBits;
//       SourceX += NoBits;
// #endif
//     }
//
//     for (; DestX < RoundedRight; DestX += 32, DestBits++, SourceX += 32)
//     {
//       Dest = *DestBits;
//
//       if (UsesSource)
//       {
//         Source = 0;
//         for (Index = 0; Index < 8; Index++)
//         {
//           Source |= DIB_GetSource(BltInfo->SourceSurface, SourceX + Index, SourceY, BltInfo->XlateSourceToDest) << (7 - Index);
//           Source |= DIB_GetSource(BltInfo->SourceSurface, SourceX + Index + 8, SourceY, BltInfo->XlateSourceToDest) << (8 + (7 - Index));
//           Source |= DIB_GetSource(BltInfo->SourceSurface, SourceX + Index + 16, SourceY, BltInfo->XlateSourceToDest) << (16 + (7 - Index));
//           Source |= DIB_GetSource(BltInfo->SourceSurface, SourceX + Index + 24, SourceY, BltInfo->XlateSourceToDest) << (24 + (7 - Index));
//         }
//       }
//
//       if (BltInfo->PatternSurface)
//       {
//         Pattern = 0;
//         for (Index = 0; Index < 8; Index++)
//         {
//           Pattern |= DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX + BltInfo->BrushOrigin.x + Index) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY) << (7 - Index);
//           Pattern |= DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX + BltInfo->BrushOrigin.x + Index + 8) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY) << (8 + (7 - Index));
//           Pattern |= DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX + BltInfo->BrushOrigin.x + Index + 16) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY) << (16 + (7 - Index));
//           Pattern |= DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX + BltInfo->BrushOrigin.x + Index + 24) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY) << (24 + (7 - Index));
//         }
//       }
//
//       *DestBits = DIB_DoRop(BltInfo->Rop4, Dest, Source, Pattern);
//     }
//
//     if (DestX < BltInfo->DestRect.right)
//     {
//       for (; DestX < BltInfo->DestRect.right; DestX++, SourceX++)
//       {
//         Dest = DIB_1BPP_GetPixel(BltInfo->DestSurface, DestX, DestY);
//
//         if (UsesSource)
//         {
//           Source = DIB_GetSource(BltInfo->SourceSurface, SourceX, SourceY, BltInfo->XlateSourceToDest);
//         }
//
//         if (BltInfo->PatternSurface)
//         {
//           Pattern = DIB_GetSourceIndex(BltInfo->PatternSurface, (DestX + BltInfo->BrushOrigin.x) % BltInfo->PatternSurface->sizlBitmap.cx, PatternY);
//         }
//
//         DIB_1BPP_PutPixel(BltInfo->DestSurface, DestX, DestY, DIB_DoRop(BltInfo->Rop4, Dest, Source, Pattern) & 0xF);
//       }
//     }
//
//     SourceY++;
//     if (BltInfo->PatternSurface)
//     {
//       PatternY++;
//       PatternY %= BltInfo->PatternSurface->sizlBitmap.cy;
//     }
//   }
//
//   return TRUE;
// }
//
// /* BitBlt Optimize */
// BOOLEAN
// DIB_1BPP_ColorFill(SURFOBJ* DestSurface, RECTL* DestRect, ULONG color)
// {
//   LONG DestY;
//
//   /* Make WellOrdered with top < bottom and left < right */
//   RECTL_vMakeWellOrdered(DestRect);
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     DIB_1BPP_HLine(DestSurface, DestRect->left, DestRect->right, DestY, color);
//   }
//   return TRUE;
// }
// #endif // !_USE_DIBLIB_
//
// BOOLEAN
// DIB_1BPP_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                         RECTL*  DestRect,  RECTL *SourceRect,
//                         XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
