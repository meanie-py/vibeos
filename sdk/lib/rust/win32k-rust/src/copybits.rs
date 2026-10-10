use win32k_sys::win32k::*;

// CLIPOBJ_* helpers aren't in the win32k-sys bindgen extern set (they are
// real symbols inside win32k.sys though — see win32ss/gdi/eng/clipobj.c).
// Declare them locally with the in-kernel ABI (NTAPI = stdcall on i386,
// cdecl elsewhere). Mirrors the lineto.rs precedent.

// FORCEINLINE in win32ss/gdi/ntgdi/rect.h:55 — inline-expand.
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    if (*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom { 1 } else { 0 }
}

// RECTL_bIntersectRect is FASTCALL in C (rect.c) — @Name@N on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
}

// Inline-expand of `ROP4_FROM_INDEX(index)` from `win32ss/gdi/eng/inteng.h:42`:
//   ((index) | ((index) << 8))
#[inline(always)]
fn rop4_from_index(index: u32) -> ROP4 {
    (index | (index << 8)) as ROP4
}

// Inline-expand of `GDIDEVFUNCS(SurfObj)` from `win32ss/gdi/eng/surface.h:106`:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
// `DriverFunctions` lives inside the `__bindgen_anon_1` union on PDEVOBJ.
// Returns the `CopyBits` `PFN_DrvCopyBits` directly (callers always
// dereference `.CopyBits`).
#[inline(always)]
unsafe fn gdidev_copybits(pso: *mut SURFOBJ) -> PFN_DrvCopyBits {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    (*pdev).__bindgen_anon_1.DriverFunctions.CopyBits
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from the preamble.

#[no_mangle]
pub unsafe extern "system" fn EngCopyBits(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: *mut RECTL,
    SourcePoint: *mut POINTL,
) -> BOOL {
    eng_copybits_body(psoDest, psoSource, Clip, ColorTranslation, DestRect, SourcePoint)
}

/// Shared body for `EngCopyBits` across architectures.
#[inline]
unsafe fn eng_copybits_body(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: *mut RECTL,
    SourcePoint: *mut POINTL,
) -> BOOL {
    let mut ret: BOOL;
    // The C uses `BYTE clippingType` (u8) but compares it against
    // `DC_TRIVIAL`/`DC_RECT`/`DC_COMPLEX` which are `u32` in bindgen.
    // Track as u32 to avoid lossy round-tripping in the match below.
    let clippingType: u32;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: BOOL;
    let mut BltInfo: BLTINFO = core::mem::zeroed();
    let psurfDest: *mut SURFACE;
    let psurfSource: *mut SURFACE;
    let mut rclDest: RECTL = *DestRect;
    let ptlSrc: POINTL = *SourcePoint;
    let mut lTmp: LONG;
    let bTopToBottom: BOOL;

    if (*DestRect).top > (*DestRect).bottom {
        bTopToBottom = 1;
        lTmp = (*DestRect).top;
        (*DestRect).top = (*DestRect).bottom;
        (*DestRect).bottom = lTmp;
        rclDest = *DestRect;
    } else {
        bTopToBottom = 0;
    }

    psurfSource = (psoSource as *mut u8)
        .offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    psurfDest = (psoDest as *mut u8)
        .offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if (*psoSource).sizlBitmap.cx - ptlSrc.x < rclDest.right - rclDest.left {
        rclDest.right = rclDest.left + (*psoSource).sizlBitmap.cx - ptlSrc.x;
    }
    if (*psoSource).sizlBitmap.cy - ptlSrc.y < rclDest.bottom - rclDest.top {
        rclDest.bottom = rclDest.top + (*psoSource).sizlBitmap.cy - ptlSrc.y;
    }

    if rclDest.right > (*psoDest).sizlBitmap.cx {
        rclDest.right = (*psoDest).sizlBitmap.cx;
    }
    if rclDest.bottom > (*psoDest).sizlBitmap.cy {
        rclDest.bottom = (*psoDest).sizlBitmap.cy;
    }
    if RECTL_bIsEmptyRect(&rclDest) != 0 {
        return 1;
    }
    let DestRect = &mut rclDest as *mut RECTL;

    // SURFOBJ.iType is a `u16` field in bindgen; STYPE_BITMAP is a `u32` const.
    if (*psoDest).iType as u32 != STYPE_BITMAP
        || (*psoSource).iType as u32 != STYPE_BITMAP
    {
        if (*psoDest).iType as u32 != STYPE_BITMAP {
            if ((*psurfDest).flags & HOOK_COPYBITS as FLONG) != 0 {
                let pfn = gdidev_copybits(psoDest).unwrap_unchecked();
                ret = pfn(
                    psoDest,
                    psoSource,
                    Clip,
                    ColorTranslation,
                    DestRect,
                    SourcePoint,
                );
                return ret;
            }
        }

        if (*psoSource).iType as u32 != STYPE_BITMAP {
            if ((*psurfSource).flags & HOOK_COPYBITS as FLONG) != 0 {
                let pfn = gdidev_copybits(psoSource).unwrap_unchecked();
                ret = pfn(
                    psoDest,
                    psoSource,
                    Clip,
                    ColorTranslation,
                    DestRect,
                    SourcePoint,
                );
                return ret;
            }
        }

        ret = IntEngBitBlt(
            psoDest,
            psoSource,
            core::ptr::null_mut(),
            Clip,
            ColorTranslation,
            DestRect,
            SourcePoint,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            rop4_from_index(R3_OPINDEX_SRCCOPY),
        );
        return ret;
    }

    if Clip.is_null() {
        clippingType = DC_TRIVIAL;
    } else {
        clippingType = (*Clip).iDComplexity as u32;
    }

    BltInfo.DestSurface = psoDest;
    BltInfo.SourceSurface = psoSource;
    BltInfo.PatternSurface = core::ptr::null_mut();
    BltInfo.XlateSourceToDest = ColorTranslation;
    BltInfo.Rop4 = rop4_from_index(R3_OPINDEX_SRCCOPY);

    // `DibFunctionsForBitmapFormat` is a flexible-array extern (`[T; 0]`)
    // in win32k-sys bindgen — index via pointer arithmetic.
    let dib_base =
        core::ptr::addr_of_mut!(DibFunctionsForBitmapFormat) as *mut DIB_FUNCTIONS;

    match clippingType {
        DC_TRIVIAL => {
            BltInfo.DestRect = *DestRect;
            BltInfo.SourcePoint = *SourcePoint;

            if bTopToBottom != 0 {
                lTmp = BltInfo.DestRect.top;
                BltInfo.DestRect.top = BltInfo.DestRect.bottom;
                BltInfo.DestRect.bottom = lTmp;
            }

            let entry = dib_base.add((*psoDest).iBitmapFormat as usize);
            let pfn = (*entry).DIB_BitBltSrcCopy.unwrap_unchecked();
            ret = pfn(&mut BltInfo) as BOOL;
        }

        DC_RECT => {
            RECTL_bIntersectRect(&mut BltInfo.DestRect, DestRect, &(*Clip).rclBounds);

            BltInfo.SourcePoint.x =
                (*SourcePoint).x + BltInfo.DestRect.left - (*DestRect).left;
            BltInfo.SourcePoint.y =
                (*SourcePoint).y + BltInfo.DestRect.top - (*DestRect).top;

            if bTopToBottom != 0 {
                lTmp = BltInfo.DestRect.top;
                BltInfo.DestRect.top = BltInfo.DestRect.bottom;
                BltInfo.DestRect.bottom = lTmp;
            }

            let entry = dib_base.add((*psoDest).iBitmapFormat as usize);
            let pfn = (*entry).DIB_BitBltSrcCopy.unwrap_unchecked();
            ret = pfn(&mut BltInfo) as BOOL;
        }

        DC_COMPLEX => {
            CLIPOBJ_cEnumStart(Clip, 0, CT_RECTANGLES, CD_ANY, 0);

            loop {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    &mut RectEnum as *mut _ as PVOID,
                ) as i32;

                if RectEnum.c > 0 {
                    let prclEnd = RectEnum.arcl.as_mut_ptr().add(RectEnum.c as usize);
                    let mut prcl = RectEnum.arcl.as_mut_ptr();

                    while prcl < prclEnd {
                        RECTL_bIntersectRect(&mut BltInfo.DestRect, prcl, DestRect);

                        BltInfo.SourcePoint.x =
                            (*SourcePoint).x + BltInfo.DestRect.left - (*DestRect).left;
                        BltInfo.SourcePoint.y =
                            (*SourcePoint).y + BltInfo.DestRect.top - (*DestRect).top;

                        if bTopToBottom != 0 {
                            lTmp = BltInfo.DestRect.top;
                            BltInfo.DestRect.top = BltInfo.DestRect.bottom;
                            BltInfo.DestRect.bottom = lTmp;
                        }

                        let entry = dib_base.add((*psoDest).iBitmapFormat as usize);
                        let pfn = (*entry).DIB_BitBltSrcCopy.unwrap_unchecked();
                        if pfn(&mut BltInfo) == 0 {
                            ret = 0;
                            return ret;
                        }

                        prcl = prcl.add(1);
                    }
                }

                if EnumMore == 0 {
                    break;
                }
            }
            ret = 1;
        }

        _ => {
            ret = 0;
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IntEngCopyBits(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    ptlSource: *mut POINTL,
) -> BOOL {
    EngCopyBits(psoDest, psoSource, pco, pxlo, prclDest, ptlSource)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          GDI EngCopyBits Function
//  * FILE:             win32ss/gdi/eng/copybits.c
//  * PROGRAMERS:       Jason Filby
//  *                   Doug Lyons
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
//  * @implemented
//  */
// BOOL APIENTRY
// EngCopyBits(
//     _In_ SURFOBJ *psoDest,
//     _In_ SURFOBJ *psoSource,
//     _In_opt_ CLIPOBJ *Clip,
//     _In_opt_ XLATEOBJ *ColorTranslation,
//     _In_ RECTL *DestRect,
//     _In_ POINTL *SourcePoint)
// {
//     BOOL      ret;
//     BYTE      clippingType;
//     RECT_ENUM RectEnum;
//     BOOL      EnumMore;
//     BLTINFO   BltInfo;
//     SURFACE *psurfDest;
//     SURFACE *psurfSource;
//     RECTL rclDest = *DestRect;
//     POINTL ptlSrc = *SourcePoint;
//     LONG      lTmp;
//     BOOL      bTopToBottom;
//
//     DPRINT("Entering EngCopyBits with SourcePoint (%d,%d) and DestRect (%d,%d)-(%d,%d).\n",
//            SourcePoint->x, SourcePoint->y, DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//     DPRINT("psoSource cx/cy is %d/%d and psoDest cx/cy is %d/%d.\n",
//            psoSource->sizlBitmap.cx, psoSource->sizlBitmap.cy, psoDest->sizlBitmap.cx, psoDest->sizlBitmap.cy);
//
//     /* Retrieve Top Down/flip here and then make Well-Ordered again */
//     if (DestRect->top > DestRect->bottom)
//     {
//         bTopToBottom = TRUE;
//         lTmp = DestRect->top;
//         DestRect->top = DestRect->bottom;
//         DestRect->bottom = lTmp;
//         rclDest = *DestRect;
//     }
//     else
//     {
//         bTopToBottom = FALSE;
//     }
//
//     DPRINT("bTopToBottom is '%d'.\n", bTopToBottom);
//
//     ASSERT(psoDest != NULL && psoSource != NULL && DestRect != NULL && SourcePoint != NULL);
//
//     psurfSource = CONTAINING_RECORD(psoSource, SURFACE, SurfObj);
//     psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//
//     /* Clip dest rect against source surface size / source point */
//     if (psoSource->sizlBitmap.cx - ptlSrc.x < rclDest.right - rclDest.left)
//         rclDest.right = rclDest.left + psoSource->sizlBitmap.cx - ptlSrc.x;
//     if (psoSource->sizlBitmap.cy - ptlSrc.y < rclDest.bottom - rclDest.top)
//         rclDest.bottom = rclDest.top + psoSource->sizlBitmap.cy - ptlSrc.y;
//
//     /* Clip dest rect against target surface size */
//     if (rclDest.right > psoDest->sizlBitmap.cx)
//         rclDest.right = psoDest->sizlBitmap.cx;
//     if (rclDest.bottom > psoDest->sizlBitmap.cy)
//         rclDest.bottom = psoDest->sizlBitmap.cy;
//     if (RECTL_bIsEmptyRect(&rclDest)) return TRUE;
//     DestRect = &rclDest;
//
//     // FIXME: Don't punt to the driver's DrvCopyBits immediately. Instead,
//     //        mark the copy block function to be DrvCopyBits instead of the
//     //        GDI's copy bit function so as to remove clipping from the
//     //        driver's responsibility
//
//     // If one of the surfaces isn't managed by the GDI
//     if ((psoDest->iType!=STYPE_BITMAP) || (psoSource->iType!=STYPE_BITMAP))
//     {
//         // Destination surface is device managed
//         if (psoDest->iType!=STYPE_BITMAP)
//         {
//             /* FIXME: Eng* functions shouldn't call Drv* functions. ? */
//             if (psurfDest->flags & HOOK_COPYBITS)
//             {
//                 ret = GDIDEVFUNCS(psoDest).CopyBits(
//                           psoDest, psoSource, Clip, ColorTranslation, DestRect, SourcePoint);
//
//                 goto cleanup;
//             }
//         }
//
//         // Source surface is device managed
//         if (psoSource->iType!=STYPE_BITMAP)
//         {
//             /* FIXME: Eng* functions shouldn't call Drv* functions. ? */
//             if (psurfSource->flags & HOOK_COPYBITS)
//             {
//                 ret = GDIDEVFUNCS(psoSource).CopyBits(
//                           psoDest, psoSource, Clip, ColorTranslation, DestRect, SourcePoint);
//
//                 goto cleanup;
//             }
//         }
//
//         // If CopyBits wasn't hooked, BitBlt must be
//         ret = IntEngBitBlt(psoDest, psoSource,
//                            NULL, Clip, ColorTranslation, DestRect, SourcePoint,
//                            NULL, NULL, NULL, ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY));
//
//         goto cleanup;
//     }
//
//     // Determine clipping type
//     if (!Clip)
//     {
//         clippingType = DC_TRIVIAL;
//     }
//     else
//     {
//         clippingType = Clip->iDComplexity;
//     }
//
//     BltInfo.DestSurface = psoDest;
//     BltInfo.SourceSurface = psoSource;
//     BltInfo.PatternSurface = NULL;
//     BltInfo.XlateSourceToDest = ColorTranslation;
//     BltInfo.Rop4 = ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY);
//
//     switch (clippingType)
//     {
//         case DC_TRIVIAL:
//             DPRINT("DC_TRIVIAL.\n");
//             BltInfo.DestRect = *DestRect;
//             BltInfo.SourcePoint = *SourcePoint;
//
//             /* Now we set the Dest Rect top and bottom based on Top Down/flip */
//             if (bTopToBottom)
//             {
//                 lTmp = BltInfo.DestRect.top;
//                 BltInfo.DestRect.top = BltInfo.DestRect.bottom;
//                 BltInfo.DestRect.bottom = lTmp;
//             }
//
//             ret = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_BitBltSrcCopy(&BltInfo);
//             break;
//
//         case DC_RECT:
//             DPRINT("DC_RECT.\n");
//             // Clip the blt to the clip rectangle
//             RECTL_bIntersectRect(&BltInfo.DestRect, DestRect, &Clip->rclBounds);
//
//             BltInfo.SourcePoint.x = SourcePoint->x + BltInfo.DestRect.left - DestRect->left;
//             BltInfo.SourcePoint.y = SourcePoint->y + BltInfo.DestRect.top  - DestRect->top;
//
//             /* Now we set the Dest Rect top and bottom based on Top Down/flip */
//             if (bTopToBottom)
//             {
//                 lTmp = BltInfo.DestRect.top;
//                 BltInfo.DestRect.top = BltInfo.DestRect.bottom;
//                 BltInfo.DestRect.bottom = lTmp;
//             }
//
//             ret = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_BitBltSrcCopy(&BltInfo);
//             break;
//
//         case DC_COMPLEX:
//             DPRINT("DC_COMPLEX.\n");
//             CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, CD_ANY, 0);
//
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip,(ULONG) sizeof(RectEnum), (PVOID) &RectEnum) as i32;
//
//                 if (RectEnum.c > 0)
//                 {
//                     RECTL* prclEnd = &RectEnum.arcl[RectEnum.c];
//                     RECTL* prcl    = &RectEnum.arcl[0];
//
//                     do
//                     {
//                         RECTL_bIntersectRect(&BltInfo.DestRect, prcl, DestRect);
//
//                         BltInfo.SourcePoint.x = SourcePoint->x + BltInfo.DestRect.left - DestRect->left;
//                         BltInfo.SourcePoint.y = SourcePoint->y + BltInfo.DestRect.top - DestRect->top;
//
//                         /* Now we set the Dest Rect top and bottom based on Top Down/flip */
//                         if (bTopToBottom)
//                         {
//                             lTmp = BltInfo.DestRect.top;
//                             BltInfo.DestRect.top = BltInfo.DestRect.bottom;
//                             BltInfo.DestRect.bottom = lTmp;
//                         }
//
//                         if (!DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_BitBltSrcCopy(&BltInfo))
//                         {
//                             ret = FALSE;
//                             goto cleanup;
//                         }
//
//                         prcl++;
//
//                     } while (prcl < prclEnd);
//                 }
//
//             } while (EnumMore);
//             ret = TRUE;
//             break;
//
//         default:
//             ASSERT(FALSE);
//             ret = FALSE;
//             break;
//     }
//
// cleanup:
//     return ret;
// }
//
// BOOL APIENTRY
// IntEngCopyBits(
//     SURFOBJ *psoDest,
//     SURFOBJ *psoSource,
//     CLIPOBJ *pco,
//     XLATEOBJ *pxlo,
//     RECTL *prclDest,
//     POINTL *ptlSource)
// {
//     return EngCopyBits(psoDest, psoSource, pco, pxlo, prclDest, ptlSource);
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: PVOID) -> i32;
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
}