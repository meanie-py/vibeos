use win32k_sys::win32k::*;

extern "system" {
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
    fn CLIPOBJ_bEnum(clip: *mut CLIPOBJ, cj: u32, pv: *mut core::ffi::c_void) -> u8;
    fn SafeProbeForRead(address: *const core::ffi::c_void, length: usize, alignment: u32) -> i32;
}
extern "C" {
    static gxcoTrivial: CLIPOBJ;
}

// RECTL_* are FASTCALL in C — @Name@N on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(out: *mut RECTL, a: *const RECTL, b: *const RECTL) -> u32;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(out: *mut RECTL, a: *const RECTL, b: *const RECTL) -> u32;
}

// ---- file-scope preamble (translated) ----
// The C preamble only contains `#define NDEBUG`. Two helpers added:
//   * NT_SUCCESS — file-private inline (matches the C macro
//     `((NTSTATUS)(Status) >= 0)`); gdiobj.rs has a private copy.
//   * gdidev_alphablend — inline expansion of the GDIDEVFUNCS(SurfObj)
//     macro from win32ss/gdi/eng/surface.h:106 (canonical pattern lives
//     in paint.rs::gdidev_paint, see win32k_translation_prompt.md §6).

#[inline(always)]
fn NT_SUCCESS(status: i32) -> bool {
    status >= 0
}

// Inline-expand of `GDIDEVFUNCS(SurfObj)` from `win32ss/gdi/eng/surface.h:106`:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
// In bindgen, `DriverFunctions` lives inside the anonymous union
// `__bindgen_anon_1` on `_PDEVOBJ`. Returns the `AlphaBlend` slot.
#[inline(always)]
unsafe fn gdidev_alphablend(pso: *mut SURFOBJ) -> PFN_DrvAlphaBlend {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    (*pdev).__bindgen_anon_1.DriverFunctions.AlphaBlend
}
#[no_mangle]
pub unsafe extern "system" fn EngAlphaBlend(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    BlendObj: *mut BLENDOBJ,
) -> u32 {

    let mut InputRect: RECTL = core::mem::zeroed();
    let mut OutputRect: RECTL = core::mem::zeroed();
    let mut ClipRect: RECTL = core::mem::zeroed();
    let mut CombinedRect: RECTL = core::mem::zeroed();
    let mut Rect: RECTL = core::mem::zeroed();
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeaveSource: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut EnterLeaveDest: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut InputObj: *mut SURFOBJ = core::ptr::null_mut();
    let mut OutputObj: *mut SURFOBJ = core::ptr::null_mut();
    let ClippingType: u32;
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: u32;
    let mut i: u32;
    let mut Ret: u8;

    OutputRect = *DestRect;
    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(OutputRect));

    InputRect = *SourceRect;
    RECTL_vMakeWellOrdered(core::ptr::addr_of_mut!(InputRect));
    if (InputRect.top < 0) || (InputRect.bottom < 0) ||
       (InputRect.left < 0) || (InputRect.right < 0) ||
       InputRect.right > (*psoSource).sizlBitmap.cx ||
       InputRect.bottom > (*psoSource).sizlBitmap.cy
    {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if psoDest == psoSource &&
        !(OutputRect.left >= (*SourceRect).right || InputRect.left >= OutputRect.right ||
          OutputRect.top >= (*SourceRect).bottom || InputRect.top >= OutputRect.bottom)
    {
        return 0;
    }

    if (*BlendObj).BlendFunction.BlendOp != AC_SRC_OVER as u8 {
        return 0;
    }
    if (*BlendObj).BlendFunction.BlendFlags != 0 {
        return 0;
    }
    if ((*BlendObj).BlendFunction.AlphaFormat & !(AC_SRC_ALPHA as u8)) != 0 {
        return 0;
    }

    if !ClipRegion.is_null() &&
        ((*ClipRegion).rclBounds.left >= (*ClipRegion).rclBounds.right ||
         (*ClipRegion).rclBounds.top >= (*ClipRegion).rclBounds.bottom)
    {
        return 1;
    }

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeaveSource),
        psoSource,
        core::ptr::addr_of_mut!(InputRect),
        1,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(InputObj),
    ) == 0
    {
        return 0;
    }
    InputRect.left += Translate.x;
    InputRect.right += Translate.x;
    InputRect.top += Translate.y;
    InputRect.bottom += Translate.y;

    if IntEngEnter(
        core::ptr::addr_of_mut!(EnterLeaveDest),
        psoDest,
        core::ptr::addr_of_mut!(OutputRect),
        0,
        core::ptr::addr_of_mut!(Translate),
        core::ptr::addr_of_mut!(OutputObj),
    ) == 0
    {
        IntEngLeave(core::ptr::addr_of_mut!(EnterLeaveSource));
        return 0;
    }
    OutputRect.left += Translate.x;
    OutputRect.right += Translate.x;
    OutputRect.top += Translate.y;
    OutputRect.bottom += Translate.y;

    Ret = 0;
    ClippingType = if ClipRegion.is_null() { DC_TRIVIAL } else { (*ClipRegion).iDComplexity as u32 };
    // `DibFunctionsForBitmapFormat` is a flexible-array extern (`[T; 0]`)
    // in win32k-sys bindgen — index via pointer arithmetic.
    let dib_base = core::ptr::addr_of_mut!(DibFunctionsForBitmapFormat) as *mut DIB_FUNCTIONS;
    match ClippingType {
        DC_TRIVIAL => {
            let entry = dib_base.add((*OutputObj).iBitmapFormat as usize);
            let pfn = (*entry).DIB_AlphaBlend.unwrap_unchecked();
            Ret = pfn(
                OutputObj,
                InputObj,
                core::ptr::addr_of_mut!(OutputRect),
                core::ptr::addr_of_mut!(InputRect),
                ClipRegion,
                ColorTranslation,
                BlendObj,
            );
        }

        DC_RECT => {
            Ret = 1;
            ClipRect.left = (*ClipRegion).rclBounds.left + Translate.x;
            ClipRect.right = (*ClipRegion).rclBounds.right + Translate.x;
            ClipRect.top = (*ClipRegion).rclBounds.top + Translate.y;
            ClipRect.bottom = (*ClipRegion).rclBounds.bottom + Translate.y;
            if RECTL_bIntersectRect(
                core::ptr::addr_of_mut!(CombinedRect),
                core::ptr::addr_of!(OutputRect),
                core::ptr::addr_of!(ClipRect),
            ) != 0
            {
                Rect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
                Rect.right = InputRect.right + (CombinedRect.right - OutputRect.right) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
                Rect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
                Rect.bottom = InputRect.bottom + (CombinedRect.bottom - OutputRect.bottom) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);

                let entry = dib_base.add((*OutputObj).iBitmapFormat as usize);
                let pfn = (*entry).DIB_AlphaBlend.unwrap_unchecked();
                Ret = pfn(
                    OutputObj,
                    InputObj,
                    core::ptr::addr_of_mut!(CombinedRect),
                    core::ptr::addr_of_mut!(Rect),
                    ClipRegion,
                    ColorTranslation,
                    BlendObj,
                );
            }
        }

        DC_COMPLEX => {
            Ret = 1;
            CLIPOBJ_cEnumStart(ClipRegion, 0, CT_RECTANGLES, CD_ANY, 0);
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
                        core::ptr::addr_of!(OutputRect),
                        core::ptr::addr_of!(ClipRect),
                    ) != 0
                    {
                        Rect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
                        Rect.right = InputRect.right + (CombinedRect.right - OutputRect.right) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
                        Rect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
                        Rect.bottom = InputRect.bottom + (CombinedRect.bottom - OutputRect.bottom) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);

                        let entry = dib_base.add((*OutputObj).iBitmapFormat as usize);
                        let pfn = (*entry).DIB_AlphaBlend.unwrap_unchecked();
                        Ret = pfn(
                            OutputObj,
                            InputObj,
                            core::ptr::addr_of_mut!(CombinedRect),
                            core::ptr::addr_of_mut!(Rect),
                            ClipRegion,
                            ColorTranslation,
                            BlendObj,
                        ) & Ret;
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

    Ret as u32
}
#[no_mangle]
pub unsafe extern "system" fn IntEngAlphaBlend(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pxlo: *mut XLATEOBJ,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    pBlendObj: *mut BLENDOBJ,
) -> u32 {
    let mut ret: u32 = 0;
    let mut pco = pco;

    if pco.is_null() {
        pco = core::ptr::addr_of!(gxcoTrivial) as *mut CLIPOBJ;
    }

    if (*pco).rclBounds.left >= (*pco).rclBounds.right ||
       (*pco).rclBounds.top >= (*pco).rclBounds.bottom
    {
        return 1;
    }

    let psurfDest = (psoDest as *mut u8).sub(core::mem::offset_of!(SURFACE, SurfObj)) as *mut SURFACE;

    if ((*psurfDest).flags & HOOK_ALPHABLEND as FLONG) != 0 {
        if let Some(drv_alpha_blend) = gdidev_alphablend(psoDest) {
            ret = drv_alpha_blend(
                psoDest, psoSource, pco, pxlo,
                prclDest, prclSrc, pBlendObj
            ) as u32;
        }
    }

    if ret == 0 {
        ret = EngAlphaBlend(psoDest, psoSource, pco, pxlo,
                            prclDest, prclSrc, pBlendObj);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngAlphaBlend(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    upDestRect: PRECTL,
    upSourceRect: PRECTL,
    BlendObj: *mut BLENDOBJ,
) -> u32 {

    let mut DestRect: RECTL = core::mem::zeroed();
    let mut SourceRect: RECTL = core::mem::zeroed();

    // C: SafeProbeForRead both rects, then RtlCopyMemory each. The probe proves
    // where the rect is, not that it stays mapped; copy fault-tolerantly so an
    // unmapped page fails the call (return FALSE) instead of bugchecking.
    if crate::safe_user_io::copy_from_user(
        core::ptr::addr_of_mut!(DestRect) as *mut core::ffi::c_void,
        upDestRect as *const core::ffi::c_void,
        core::mem::size_of::<RECTL>(),
        1,
    )
    .is_err()
        || crate::safe_user_io::copy_from_user(
            core::ptr::addr_of_mut!(SourceRect) as *mut core::ffi::c_void,
            upSourceRect as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
            1,
        )
        .is_err()
    {
        return 0;
    }

    return EngAlphaBlend(psoDest, psoSource, ClipRegion, ColorTranslation, core::ptr::addr_of_mut!(DestRect), core::ptr::addr_of_mut!(SourceRect), BlendObj);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          GDI alpha blending functions
//  * FILE:             win32ss/gdi/eng/alphablend.c
//  * PROGRAMER:        Jason Filby
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngAlphaBlend(
//     _Inout_ SURFOBJ *psoDest,
//     _In_ SURFOBJ *psoSource,
//     _In_opt_ CLIPOBJ *ClipRegion,
//     _In_opt_ XLATEOBJ *ColorTranslation,
//     _In_ RECTL *DestRect,
//     _In_ RECTL *SourceRect,
//     _In_ BLENDOBJ *BlendObj)
// {
//     RECTL              InputRect;
//     RECTL              OutputRect;
//     RECTL              ClipRect;
//     RECTL              CombinedRect;
//     RECTL              Rect;
//     POINTL             Translate;
//     INTENG_ENTER_LEAVE EnterLeaveSource;
//     INTENG_ENTER_LEAVE EnterLeaveDest;
//     SURFOBJ*           InputObj;
//     SURFOBJ*           OutputObj;
//     LONG               ClippingType;
//     RECT_ENUM          RectEnum;
//     BOOL               EnumMore;
//     ULONG              i;
//     BOOLEAN            Ret;
//
//     DPRINT("EngAlphaBlend(psoDest:0x%p, psoSource:0x%p, ClipRegion:0x%p, ColorTranslation:0x%p,\n", psoDest, psoSource, ClipRegion, ColorTranslation);
//     DPRINT("              DestRect:{0x%x, 0x%x, 0x%x, 0x%x}, SourceRect:{0x%x, 0x%x, 0x%x, 0x%x},\n",
//            DestRect->left, DestRect->top, DestRect->right, DestRect->bottom,
//            SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom);
//     DPRINT("              BlendObj:{0x%x, 0x%x, 0x%x, 0x%x}\n", BlendObj->BlendFunction.BlendOp,
//            BlendObj->BlendFunction.BlendFlags, BlendObj->BlendFunction.SourceConstantAlpha,
//            BlendObj->BlendFunction.AlphaFormat);
//
//     /* Validate output */
//     OutputRect = *DestRect;
//     RECTL_vMakeWellOrdered(&OutputRect);
//
//     /* Validate input */
//     InputRect = *SourceRect;
//     RECTL_vMakeWellOrdered(&InputRect);
//     if ( (InputRect.top < 0) || (InputRect.bottom < 0) ||
//          (InputRect.left < 0) || (InputRect.right < 0) ||
//          InputRect.right > psoSource->sizlBitmap.cx ||
//          InputRect.bottom > psoSource->sizlBitmap.cy )
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (psoDest == psoSource &&
//             !(OutputRect.left >= SourceRect->right || InputRect.left >= OutputRect.right ||
//               OutputRect.top >= SourceRect->bottom || InputRect.top >= OutputRect.bottom))
//     {
//         DPRINT1("Source and destination rectangles overlap!\n");
//         return FALSE;
//     }
//
//     if (BlendObj->BlendFunction.BlendOp != AC_SRC_OVER)
//     {
//         DPRINT1("BlendOp != AC_SRC_OVER (0x%x)\n", BlendObj->BlendFunction.BlendOp);
//         return FALSE;
//     }
//     if (BlendObj->BlendFunction.BlendFlags != 0)
//     {
//         DPRINT1("BlendFlags != 0 (0x%x)\n", BlendObj->BlendFunction.BlendFlags);
//         return FALSE;
//     }
//     if ((BlendObj->BlendFunction.AlphaFormat & ~AC_SRC_ALPHA) != 0)
//     {
//         DPRINT1("Unsupported AlphaFormat (0x%x)\n", BlendObj->BlendFunction.AlphaFormat);
//         return FALSE;
//     }
//
//     /* Check if there is anything to draw */
//     if (ClipRegion != NULL &&
//             (ClipRegion->rclBounds.left >= ClipRegion->rclBounds.right ||
//              ClipRegion->rclBounds.top >= ClipRegion->rclBounds.bottom))
//     {
//         /* Nothing to do */
//         return TRUE;
//     }
//
//     /* Now call the DIB function */
//     if (!IntEngEnter(&EnterLeaveSource, psoSource, &InputRect, TRUE, &Translate, &InputObj))
//     {
//         return FALSE;
//     }
//     InputRect.left +=  Translate.x;
//     InputRect.right +=  Translate.x;
//     InputRect.top +=  Translate.y;
//     InputRect.bottom +=  Translate.y;
//
//     if (!IntEngEnter(&EnterLeaveDest, psoDest, &OutputRect, FALSE, &Translate, &OutputObj))
//     {
//         IntEngLeave(&EnterLeaveSource);
//         return FALSE;
//     }
//     OutputRect.left += Translate.x;
//     OutputRect.right += Translate.x;
//     OutputRect.top += Translate.y;
//     OutputRect.bottom += Translate.y;
//
//     ASSERT(InputRect.left <= InputRect.right && InputRect.top <= InputRect.bottom);
//
//     Ret = FALSE;
//     ClippingType = (ClipRegion == NULL) ? DC_TRIVIAL : ClipRegion->iDComplexity;
//     switch (ClippingType)
//     {
//         case DC_TRIVIAL:
//             Ret = DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_AlphaBlend(
//                       OutputObj, InputObj, &OutputRect, &InputRect, ClipRegion, ColorTranslation, BlendObj);
//             break;
//
//         case DC_RECT:
//             Ret = TRUE;
//             ClipRect.left = ClipRegion->rclBounds.left + Translate.x;
//             ClipRect.right = ClipRegion->rclBounds.right + Translate.x;
//             ClipRect.top = ClipRegion->rclBounds.top + Translate.y;
//             ClipRect.bottom = ClipRegion->rclBounds.bottom + Translate.y;
//             if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//             {
//                 /* take into acount clipping results when calculating new input rect (scaled to input rect size) */
//                 Rect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
//                 Rect.right = InputRect.right + (CombinedRect.right - OutputRect.right) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
//                 Rect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
//                 Rect.bottom = InputRect.bottom + (CombinedRect.bottom - OutputRect.bottom) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
//
//                 /* Aplha blend one rect */
//                 Ret = DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_AlphaBlend(
//                           OutputObj, InputObj, &CombinedRect, &Rect, ClipRegion, ColorTranslation, BlendObj);
//             }
//             break;
//
//         case DC_COMPLEX:
//             Ret = TRUE;
//             CLIPOBJ_cEnumStart(ClipRegion, FALSE, CT_RECTANGLES, CD_ANY, 0);
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(ClipRegion,(ULONG) sizeof(RectEnum),
//                                          (PVOID) &RectEnum) as u32;
//
//                 for (i = 0; i < RectEnum.c; i++)
//                 {
//                     ClipRect.left = RectEnum.arcl[i].left + Translate.x;
//                     ClipRect.right = RectEnum.arcl[i].right + Translate.x;
//                     ClipRect.top = RectEnum.arcl[i].top + Translate.y;
//                     ClipRect.bottom = RectEnum.arcl[i].bottom + Translate.y;
//                     if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//                     {
//                         /* take into acount clipping results when calculating new input rect (scaled to input rect size) */
//                         Rect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
//                         Rect.right = InputRect.right + (CombinedRect.right - OutputRect.right) * (InputRect.right - InputRect.left) / (OutputRect.right - OutputRect.left);
//                         Rect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
//                         Rect.bottom = InputRect.bottom + (CombinedRect.bottom - OutputRect.bottom) * (InputRect.bottom - InputRect.top) / (OutputRect.bottom - OutputRect.top);
//
//                         /* Alpha blend one rect */
//                         Ret = DibFunctionsForBitmapFormat[OutputObj->iBitmapFormat].DIB_AlphaBlend(
//                                   OutputObj, InputObj, &CombinedRect, &Rect, ClipRegion, ColorTranslation, BlendObj) && Ret;
//                     }
//                 }
//             }
//             while (EnumMore);
//             break;
//
//         default:
//             UNIMPLEMENTED;
//             ASSERT(FALSE);
//             break;
//     }
//
//     IntEngLeave(&EnterLeaveDest);
//     IntEngLeave(&EnterLeaveSource);
//
//     return Ret;
// }
//
// BOOL
// APIENTRY
// IntEngAlphaBlend(
//     _Inout_ SURFOBJ *psoDest,
//     _In_ SURFOBJ *psoSource,
//     _In_opt_ CLIPOBJ *pco,
//     _In_opt_ XLATEOBJ *pxlo,
//     _In_ RECTL *prclDest,
//     _In_ RECTL *prclSrc,
//     _In_ BLENDOBJ *pBlendObj)
// {
//     BOOL ret = FALSE;
//     SURFACE *psurfDest;
//
//     ASSERT(psoDest);
//     ASSERT(psoSource);
//     ASSERT(prclDest);
//     ASSERT(prclSrc);
//     //ASSERT(pBlendObj);
//
//     /* If no clip object is given, use trivial one */
//     if (!pco) pco = (CLIPOBJ *)&gxcoTrivial;
//
//     /* Check if there is anything to draw */
//     if ((pco->rclBounds.left >= pco->rclBounds.right) ||
//         (pco->rclBounds.top >= pco->rclBounds.bottom))
//     {
//         /* Nothing to do */
//         return TRUE;
//     }
//
//     psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//
//     /* Call the driver's DrvAlphaBlend if available */
//     if (psurfDest->flags & HOOK_ALPHABLEND)
//     {
//         ret = GDIDEVFUNCS(psoDest).AlphaBlend(
//                   psoDest, psoSource, pco, pxlo,
//                   prclDest, prclSrc, pBlendObj);
//     }
//
//     if (!ret)
//     {
//         ret = EngAlphaBlend(psoDest, psoSource, pco, pxlo,
//                             prclDest, prclSrc, pBlendObj);
//     }
//
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// NtGdiEngAlphaBlend(IN SURFOBJ *psoDest,
//                    IN SURFOBJ *psoSource,
//                    IN CLIPOBJ *ClipRegion,
//                    IN XLATEOBJ *ColorTranslation,
//                    IN PRECTL upDestRect,
//                    IN PRECTL upSourceRect,
//                    IN BLENDOBJ *BlendObj)
// {
//     RECTL DestRect;
//     RECTL SourceRect;
//     NTSTATUS Status;
//
//     Status = SafeProbeForRead(upDestRect, sizeof(RECTL), 1);
//     if (NT_SUCCESS(Status))
//     {
//         Status = SafeProbeForRead(upSourceRect, sizeof(RECTL), 1);
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//     else
//     {
//         RtlCopyMemory(&DestRect,upDestRect, sizeof(RECTL));
//         RtlCopyMemory(&SourceRect, upSourceRect, sizeof(RECTL));
//     }
//
//     return EngAlphaBlend(psoDest, psoSource, ClipRegion, ColorTranslation, &DestRect, &SourceRect, BlendObj);
// }
//
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
