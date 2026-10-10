use win32k_sys::win32k::*;

// `RECTL_bIntersectRect` is FASTCALL on i386 (per win32ss/gdi/ntgdi/rect.h), cdecl elsewhere.
// `CLIPOBJ_cEnumStart` / `CLIPOBJ_bEnum` are ENGAPI (stdcall on i386, cdecl elsewhere).
// `BOOL` is `i32`; the bindgen catalog uses `BOOL` directly so we mirror that here.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(rcDst: *mut RECTL, rc1: *const RECTL, rc2: *const RECTL) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(rcDst: *mut RECTL, rc1: *const RECTL, rc2: *const RECTL) -> BOOL;
}
extern "system" {
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: *mut ULONG) -> i32;
}
extern "C" {
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11];
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub unsafe extern "system" fn EngTransparentBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourceRect: PRECTL,
    iTransColor: u32,
    Reserved: u32,
) -> u8 {
    eng_transparent_blt_body(
        psoDest, psoSource, Clip, ColorTranslation, DestRect, SourceRect, iTransColor, Reserved,
    )
}

#[inline(always)]
unsafe fn eng_transparent_blt_body(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourceRect: PRECTL,
    iTransColor: u32,
    _Reserved: u32,
) -> u8 {
    let mut Ret: u8 = 1u8;
    // C uses BYTE ClippingType but the DC_* constants are ULONG; widen so we can
    // match against the bindgen u32 constants without an `as u8` ladder.
    let ClippingType: u32;
    let mut EnterLeaveSource: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut EnterLeaveDest: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut InputObj: *mut SURFOBJ = core::ptr::null_mut();
    let mut OutputObj: *mut SURFOBJ = core::ptr::null_mut();
    let mut OutputRect: RECTL;
    let mut InputRect: RECTL;
    let mut Translate: POINTL = core::mem::zeroed();

    let DstHeight: i32;
    let DstWidth: i32;
    let SrcHeight: i32;
    let SrcWidth: i32;

    InputRect = *SourceRect;

    if IntEngEnter(
        &mut EnterLeaveSource,
        psoSource,
        &mut InputRect,
        1 as BOOL,
        &mut Translate,
        &mut InputObj,
    ) == 0
    {
        return 0u8;
    }
    InputRect.left += Translate.x;
    InputRect.right += Translate.x;
    InputRect.top += Translate.y;
    InputRect.bottom += Translate.y;

    OutputRect = *DestRect;
    if OutputRect.right < OutputRect.left {
        OutputRect.left = (*DestRect).right;
        OutputRect.right = (*DestRect).left;
    }
    if OutputRect.bottom < OutputRect.top {
        OutputRect.top = (*DestRect).bottom;
        OutputRect.bottom = (*DestRect).top;
    }

    if !Clip.is_null() {
        if OutputRect.left < (*Clip).rclBounds.left {
            InputRect.left += (*Clip).rclBounds.left - OutputRect.left;
            OutputRect.left = (*Clip).rclBounds.left;
        }
        if (*Clip).rclBounds.right < OutputRect.right {
            InputRect.right -= OutputRect.right - (*Clip).rclBounds.right;
            OutputRect.right = (*Clip).rclBounds.right;
        }
        if OutputRect.top < (*Clip).rclBounds.top {
            InputRect.top += (*Clip).rclBounds.top - OutputRect.top;
            OutputRect.top = (*Clip).rclBounds.top;
        }
        if (*Clip).rclBounds.bottom < OutputRect.bottom {
            InputRect.bottom -= OutputRect.bottom - (*Clip).rclBounds.bottom;
            OutputRect.bottom = (*Clip).rclBounds.bottom;
        }
    }

    if OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top {
        IntEngLeave(&mut EnterLeaveSource);
        return 1u8;
    }

    if IntEngEnter(
        &mut EnterLeaveDest,
        psoDest,
        &mut OutputRect,
        0 as BOOL,
        &mut Translate,
        &mut OutputObj,
    ) == 0
    {
        IntEngLeave(&mut EnterLeaveSource);
        return 0u8;
    }

    OutputRect.left = (*DestRect).left + Translate.x;
    OutputRect.right = (*DestRect).right + Translate.x;
    OutputRect.top = (*DestRect).top + Translate.y;
    OutputRect.bottom = (*DestRect).bottom + Translate.y;

    ClippingType = if !Clip.is_null() {
        (*Clip).iDComplexity as u32
    } else {
        DC_TRIVIAL
    };

    DstHeight = OutputRect.bottom - OutputRect.top;
    DstWidth = OutputRect.right - OutputRect.left;
    SrcHeight = InputRect.bottom - InputRect.top;
    SrcWidth = InputRect.right - InputRect.left;

    match ClippingType {
        DC_TRIVIAL => {
            Ret = (DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize]
                .DIB_TransparentBlt)(
                OutputObj,
                InputObj,
                &mut OutputRect,
                &mut InputRect,
                ColorTranslation,
                iTransColor,
            );
        }
        DC_RECT => {
            let mut ClipRect: RECTL = core::mem::zeroed();
            let mut CombinedRect: RECTL = core::mem::zeroed();
            let mut InputToCombinedRect: RECTL = core::mem::zeroed();

            ClipRect.left = (*Clip).rclBounds.left + Translate.x;
            ClipRect.right = (*Clip).rclBounds.right + Translate.x;
            ClipRect.top = (*Clip).rclBounds.top + Translate.y;
            ClipRect.bottom = (*Clip).rclBounds.bottom + Translate.y;
            if RECTL_bIntersectRect(&mut CombinedRect, &OutputRect, &ClipRect) != 0 {
                InputToCombinedRect.top = InputRect.top
                    + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
                InputToCombinedRect.bottom = InputRect.top
                    + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
                InputToCombinedRect.left = InputRect.left
                    + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
                InputToCombinedRect.right = InputRect.left
                    + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;
                Ret = (DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize]
                    .DIB_TransparentBlt)(
                    OutputObj,
                    InputObj,
                    &mut CombinedRect,
                    &mut InputToCombinedRect,
                    ColorTranslation,
                    iTransColor,
                );
            }
        }
        DC_COMPLEX => {
            let Direction: u32;
            let mut i: u32;
            let mut RectEnum: RECT_ENUM = core::mem::zeroed();
            let mut EnumMore: BOOL;

            if OutputObj == InputObj {
                if OutputRect.top < InputRect.top {
                    Direction = if OutputRect.left < InputRect.left {
                        CD_RIGHTDOWN
                    } else {
                        CD_LEFTDOWN
                    };
                } else {
                    Direction = if OutputRect.left < InputRect.left {
                        CD_RIGHTUP
                    } else {
                        CD_LEFTUP
                    };
                }
            } else {
                Direction = CD_ANY;
            }

            CLIPOBJ_cEnumStart(Clip, 0 as BOOL, CT_RECTANGLES, Direction, 0);
            loop {
                EnumMore = CLIPOBJ_bEnum(
                    Clip,
                    core::mem::size_of::<RECT_ENUM>() as ULONG,
                    core::ptr::addr_of_mut!(RectEnum) as *mut ULONG,
                ) as i32;
                i = 0;
                while i < RectEnum.c {
                    let mut ClipRect: RECTL = core::mem::zeroed();
                    let mut CombinedRect: RECTL = core::mem::zeroed();
                    let mut InputToCombinedRect: RECTL = core::mem::zeroed();

                    ClipRect.left = RectEnum.arcl[i as usize].left + Translate.x;
                    ClipRect.right = RectEnum.arcl[i as usize].right + Translate.x;
                    ClipRect.top = RectEnum.arcl[i as usize].top + Translate.y;
                    ClipRect.bottom = RectEnum.arcl[i as usize].bottom + Translate.y;
                    if RECTL_bIntersectRect(&mut CombinedRect, &OutputRect, &ClipRect) != 0 {
                        InputToCombinedRect.top = InputRect.top
                            + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
                        InputToCombinedRect.bottom = InputRect.top
                            + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
                        InputToCombinedRect.left = InputRect.left
                            + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
                        InputToCombinedRect.right = InputRect.left
                            + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;

                        Ret = (DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize]
                            .DIB_TransparentBlt)(
                            OutputObj,
                            InputObj,
                            &mut CombinedRect,
                            &mut InputToCombinedRect,
                            ColorTranslation,
                            iTransColor,
                        );
                        if Ret == 0 {
                            break;
                        }
                    }
                    i += 1;
                }
                if EnumMore == 0 || Ret == 0 {
                    break;
                }
            }
        }
        _ => {
            Ret = 0u8;
        }
    }

    IntEngLeave(&mut EnterLeaveDest);
    IntEngLeave(&mut EnterLeaveSource);

    Ret
}

#[repr(C)]
struct DIB_FUNCTIONS {
    DIB_TransparentBlt: unsafe extern "C" fn(
        *mut SURFOBJ,
        *mut SURFOBJ,
        *mut RECTL,
        *mut RECTL,
        *mut XLATEOBJ,
        u32,
    ) -> u8,
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngTransparentBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourceRect: PRECTL,
    iTransColor: u32,
    Reserved: u32,
) -> u8 {
    int_eng_transparent_blt_body(
        psoDest, psoSource, Clip, ColorTranslation, DestRect, SourceRect, iTransColor, Reserved,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngTransparentBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourceRect: PRECTL,
    iTransColor: u32,
    Reserved: u32,
) -> u8 {
    int_eng_transparent_blt_body(
        psoDest, psoSource, Clip, ColorTranslation, DestRect, SourceRect, iTransColor, Reserved,
    )
}

#[inline(always)]
unsafe fn int_eng_transparent_blt_body(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Clip: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    DestRect: PRECTL,
    SourceRect: PRECTL,
    iTransColor: u32,
    Reserved: u32,
) -> u8 {
    let mut Ret: u8;
    let mut OutputRect: RECTL = core::mem::zeroed();
    let mut InputClippedRect: RECTL;
    let psurfDest: *mut SURFACE;
    let _psurfSource: *mut SURFACE;
    let mut InputRect: RECTL;
    let InputClWidth: i32;
    let InputClHeight: i32;
    let InputWidth: i32;
    let InputHeight: i32;

    // CONTAINING_RECORD(psoDest, SURFACE, SurfObj)
    psurfDest = (psoDest as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
    _psurfSource = (psoSource as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    let mut Clip = Clip;
    if Clip.is_null() {
        Clip = core::ptr::addr_of_mut!(gxcoTrivial) as *mut CLIPOBJ;
    }

    InputClippedRect = *DestRect;
    if InputClippedRect.right < InputClippedRect.left {
        InputClippedRect.left = (*DestRect).right;
        InputClippedRect.right = (*DestRect).left;
    }
    if InputClippedRect.bottom < InputClippedRect.top {
        InputClippedRect.top = (*DestRect).bottom;
        InputClippedRect.bottom = (*DestRect).top;
    }

    InputRect = *SourceRect;

    if (*Clip).iDComplexity as u32 != DC_TRIVIAL {
        if RECTL_bIntersectRect(
            core::ptr::addr_of_mut!(OutputRect),
            core::ptr::addr_of!(InputClippedRect),
            core::ptr::addr_of!((*Clip).rclBounds),
        ) == 0
        {
            return 1u8;
        }

        InputClWidth = InputClippedRect.right - InputClippedRect.left;
        InputClHeight = InputClippedRect.bottom - InputClippedRect.top;
        InputWidth = InputRect.right - InputRect.left;
        InputHeight = InputRect.bottom - InputRect.top;

        InputRect.left += (InputWidth * (OutputRect.left - InputClippedRect.left)) / InputClWidth;
        InputRect.right -= (InputWidth * (InputClippedRect.right - OutputRect.right)) / InputClWidth;
        InputRect.top += (InputHeight * (OutputRect.top - InputClippedRect.top)) / InputClHeight;
        InputRect.bottom -= (InputHeight * (InputClippedRect.bottom - OutputRect.bottom)) / InputClHeight;
    } else {
        OutputRect = InputClippedRect;
    }

    if ((*psurfDest).flags & HOOK_TRANSPARENTBLT) != 0 {
        // GDIDEVFUNCS(psoDest).TransparentBlt — inline-expanded per win32k-translation §6:
        //   #define GDIDEVFUNCS(SurfObj) ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
        // (win32ss/gdi/eng/surface.h:106). DriverFunctions is a DRIVER_FUNCTIONS struct;
        // TransparentBlt is Option<unsafe extern "stdcall" fn(...) -> BOOL>.
        let pdev = (*psoDest).hdev as *mut PDEVOBJ;
        // DriverFunctions lives inside the bindgen anonymous union (`__bindgen_anon_1`).
        let pfn = (*pdev).__bindgen_anon_1.DriverFunctions.TransparentBlt;
        let bool_ret: BOOL = match pfn {
            Some(f) => f(
                psoDest,
                psoSource,
                Clip,
                ColorTranslation,
                core::ptr::addr_of_mut!(OutputRect),
                core::ptr::addr_of_mut!(InputRect),
                iTransColor,
                Reserved,
            ),
            None => 0,
        };
        Ret = if bool_ret != 0 { 1u8 } else { 0u8 };
    } else {
        Ret = 0u8;
    }

    if Ret == 0u8 {
        Ret = EngTransparentBlt(
            psoDest,
            psoSource,
            Clip,
            ColorTranslation,
            core::ptr::addr_of_mut!(OutputRect),
            core::ptr::addr_of_mut!(InputRect),
            iTransColor,
            Reserved,
        );
    }

    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          GDI TransparentBlt Function
//  * FILE:             win32ss/gdi/eng/transblt.c
//  * PROGRAMER:        Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// BOOL
// APIENTRY
// EngTransparentBlt(
//     SURFOBJ *psoDest,
//     SURFOBJ *psoSource,
//     CLIPOBJ *Clip,
//     XLATEOBJ *ColorTranslation,
//     PRECTL DestRect,
//     PRECTL SourceRect,
//     ULONG iTransColor,
//     ULONG Reserved)
// {
//     BOOL Ret = TRUE;
//     BYTE ClippingType;
//     INTENG_ENTER_LEAVE EnterLeaveSource, EnterLeaveDest;
//     SURFOBJ *InputObj, *OutputObj;
//     RECTL OutputRect, InputRect;
//     POINTL Translate;
//
//     LONG DstHeight;
//     LONG DstWidth;
//     LONG SrcHeight;
//     LONG SrcWidth;
//
//     InputRect = *SourceRect;
//
//     if (!IntEngEnter(&EnterLeaveSource, psoSource, &InputRect, TRUE, &Translate, &InputObj))
//     {
//         return FALSE;
//     }
//     InputRect.left += Translate.x;
//     InputRect.right += Translate.x;
//     InputRect.top += Translate.y;
//     InputRect.bottom += Translate.y;
//
//     OutputRect = *DestRect;
//     if (OutputRect.right < OutputRect.left)
//     {
//         OutputRect.left = DestRect->right;
//         OutputRect.right = DestRect->left;
//     }
//     if (OutputRect.bottom < OutputRect.top)
//     {
//         OutputRect.top = DestRect->bottom;
//         OutputRect.bottom = DestRect->top;
//     }
//
//     if (Clip)
//     {
//         if (OutputRect.left < Clip->rclBounds.left)
//         {
//             InputRect.left += Clip->rclBounds.left - OutputRect.left;
//             OutputRect.left = Clip->rclBounds.left;
//         }
//         if (Clip->rclBounds.right < OutputRect.right)
//         {
//             InputRect.right -=  OutputRect.right - Clip->rclBounds.right;
//             OutputRect.right = Clip->rclBounds.right;
//         }
//         if (OutputRect.top < Clip->rclBounds.top)
//         {
//             InputRect.top += Clip->rclBounds.top - OutputRect.top;
//             OutputRect.top = Clip->rclBounds.top;
//         }
//         if (Clip->rclBounds.bottom < OutputRect.bottom)
//         {
//             InputRect.bottom -=  OutputRect.bottom - Clip->rclBounds.bottom;
//             OutputRect.bottom = Clip->rclBounds.bottom;
//         }
//     }
//
//     /* Check for degenerate case: if height or width of OutputRect is 0 pixels there's
//        nothing to do */
//     if (OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top)
//     {
//         IntEngLeave(&EnterLeaveSource);
//         return TRUE;
//     }
//
//     if (!IntEngEnter(&EnterLeaveDest, psoDest, &OutputRect, FALSE, &Translate, &OutputObj))
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
//     ClippingType = (Clip ? Clip->iDComplexity : DC_TRIVIAL);
//
//     DstHeight = OutputRect.bottom - OutputRect.top;
//     DstWidth = OutputRect.right - OutputRect.left;
//     SrcHeight = InputRect.bottom - InputRect.top;
//     SrcWidth = InputRect.right - InputRect.left;
//     switch (ClippingType)
//     {
//         case DC_TRIVIAL:
//         {
//             Ret = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_TransparentBlt(
//                       OutputObj, InputObj, &OutputRect, &InputRect, ColorTranslation, iTransColor);
//             break;
//         }
//         case DC_RECT:
//         {
//             RECTL ClipRect, CombinedRect;
//             RECTL InputToCombinedRect;
//
//             ClipRect.left = Clip->rclBounds.left + Translate.x;
//             ClipRect.right = Clip->rclBounds.right + Translate.x;
//             ClipRect.top = Clip->rclBounds.top + Translate.y;
//             ClipRect.bottom = Clip->rclBounds.bottom + Translate.y;
//             if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//             {
//                 InputToCombinedRect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
//                 InputToCombinedRect.bottom = InputRect.top + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
//                 InputToCombinedRect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
//                 InputToCombinedRect.right = InputRect.left + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;
//                 Ret = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_TransparentBlt(
//                           OutputObj, InputObj, &CombinedRect, &InputToCombinedRect, ColorTranslation, iTransColor);
//             }
//             break;
//         }
//         case DC_COMPLEX:
//         {
//             ULONG Direction, i;
//             RECT_ENUM RectEnum;
//             BOOL EnumMore;
//
//             if (OutputObj == InputObj)
//             {
//                 if (OutputRect.top < InputRect.top)
//                 {
//                     Direction = OutputRect.left < (InputRect.left ? CD_RIGHTDOWN : CD_LEFTDOWN);
//                 }
//                 else
//                 {
//                     Direction = OutputRect.left < (InputRect.left ? CD_RIGHTUP : CD_LEFTUP);
//                 }
//             }
//             else
//             {
//                 Direction = CD_ANY;
//             }
//
//             CLIPOBJ_cEnumStart(Clip, FALSE, CT_RECTANGLES, Direction, 0);
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(Clip, sizeof(RectEnum), (PVOID)&RectEnum) as i32;
//                 for (i = 0; i < RectEnum.c; i++)
//                 {
//                     RECTL ClipRect, CombinedRect;
//                     RECTL InputToCombinedRect;
//
//                     ClipRect.left = RectEnum.arcl[i].left + Translate.x;
//                     ClipRect.right = RectEnum.arcl[i].right + Translate.x;
//                     ClipRect.top = RectEnum.arcl[i].top + Translate.y;
//                     ClipRect.bottom = RectEnum.arcl[i].bottom + Translate.y;
//                     if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//                     {
//                         InputToCombinedRect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
//                         InputToCombinedRect.bottom = InputRect.top + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
//                         InputToCombinedRect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
//                         InputToCombinedRect.right = InputRect.left + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;
//
//                         Ret = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_TransparentBlt(
//                                   OutputObj, InputObj, &CombinedRect, &InputToCombinedRect, ColorTranslation, iTransColor);
//                         if (!Ret)
//                         {
//                             break;
//                         }
//                     }
//                 }
//             }
//             while (EnumMore && Ret);
//             break;
//         }
//         default:
//         {
//             Ret = FALSE;
//             break;
//         }
//     }
//
//     IntEngLeave(&EnterLeaveDest);
//     IntEngLeave(&EnterLeaveSource);
//
//     return Ret;
// }
//
// BOOL
// FASTCALL
// IntEngTransparentBlt(
//     SURFOBJ *psoDest,
//     SURFOBJ *psoSource,
//     CLIPOBJ *Clip,
//     XLATEOBJ *ColorTranslation,
//     PRECTL DestRect,
//     PRECTL SourceRect,
//     ULONG iTransColor,
//     ULONG Reserved)
// {
//     BOOL Ret;
//     RECTL OutputRect, InputClippedRect;
//     SURFACE *psurfDest;
//     SURFACE *psurfSource;
//     RECTL InputRect;
//     LONG InputClWidth, InputClHeight, InputWidth, InputHeight;
//
//     ASSERT(psoDest);
//     ASSERT(psoSource);
//     ASSERT(DestRect);
//
//     psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//     psurfSource = CONTAINING_RECORD(psoSource, SURFACE, SurfObj);
//
//     ASSERT(psurfDest);
//     ASSERT(psurfSource);
//
//     /* If no clip object is given, use trivial one */
//     if (!Clip) Clip = (CLIPOBJ *)&gxcoTrivial;
//
//     InputClippedRect = *DestRect;
//     if (InputClippedRect.right < InputClippedRect.left)
//     {
//         InputClippedRect.left = DestRect->right;
//         InputClippedRect.right = DestRect->left;
//     }
//     if (InputClippedRect.bottom < InputClippedRect.top)
//     {
//         InputClippedRect.top = DestRect->bottom;
//         InputClippedRect.bottom = DestRect->top;
//     }
//
//     InputRect = *SourceRect;
//     /* Clip against the bounds of the clipping region so we won't try to write
//      * outside the surface */
//     if (Clip->iDComplexity != DC_TRIVIAL)
//     {
//         if (!RECTL_bIntersectRect(&OutputRect, &InputClippedRect, &Clip->rclBounds))
//         {
//             return TRUE;
//         }
//         /* Update source rect */
//         InputClWidth = InputClippedRect.right - InputClippedRect.left;
//         InputClHeight = InputClippedRect.bottom - InputClippedRect.top;
//         InputWidth = InputRect.right - InputRect.left;
//         InputHeight = InputRect.bottom - InputRect.top;
//
//         InputRect.left += (InputWidth * (OutputRect.left - InputClippedRect.left)) / InputClWidth;
//         InputRect.right -= (InputWidth * (InputClippedRect.right - OutputRect.right)) / InputClWidth;
//         InputRect.top += (InputHeight * (OutputRect.top - InputClippedRect.top)) / InputClHeight;
//         InputRect.bottom -= (InputHeight * (InputClippedRect.bottom - OutputRect.bottom)) / InputClHeight;
//     }
//     else
//     {
//         OutputRect = InputClippedRect;
//     }
//
//     if (psurfDest->flags & HOOK_TRANSPARENTBLT)
//     {
//         Ret = GDIDEVFUNCS(psoDest).TransparentBlt(psoDest,
//                                                   psoSource,
//                                                   Clip,
//                                                   ColorTranslation,
//                                                   &OutputRect,
//                                                   &InputRect,
//                                                   iTransColor,
//                                                   Reserved);
//     }
//     else
//         Ret = FALSE;
//
//     if (!Ret)
//     {
//         Ret = EngTransparentBlt(psoDest,
//                                 psoSource,
//                                 Clip,
//                                 ColorTranslation,
//                                 &OutputRect,
//                                 &InputRect,
//                                 iTransColor,
//                                 Reserved);
//     }
//
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
