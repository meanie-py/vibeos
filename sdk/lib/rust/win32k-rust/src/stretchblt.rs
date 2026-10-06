use win32k_sys::win32k::*;
use core::ffi::c_void;
use ntkrnl_sys::base::NT_SUCCESS;
// bindgen flexible-array extern is [T;0] — indexing it panics; use the real [DIB_FUNCTIONS; 11].
use crate::dib::DibFunctionsForBitmapFormat;

// `RtlCopyMemory` is a Microsoft `#define memcpy` macro — inline locally
// (same pattern as win32k-rust/src/clip.rs).
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}

extern "C" {
    fn abs(x: i32) -> i32;
    static gxcoTrivial: CLIPOBJ;
}

// C macro `GDIDEVFUNCS(SurfObj)` from surface.h:106:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
#[inline(always)]
unsafe fn GDIDEVFUNCS(pso: *mut SURFOBJ) -> *const core::ffi::c_void {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    core::ptr::addr_of!((*pdev).__bindgen_anon_1.DriverFunctions) as *const core::ffi::c_void
}

// C macros from win32ss/gdi/eng/inteng.h — inline-expand.
#[inline(always)]
unsafe fn ROP4_USES_SOURCE(rop4: u32) -> u8 {
    if ((rop4 & 0xCCCC) >> 2) != (rop4 & 0x3333) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_USES_PATTERN(rop4: u32) -> u32 {
    if ((rop4 & 0xF0F0) >> 4) != (rop4 & 0x0F0F) { 1 } else { 0 }
}
#[inline(always)]
unsafe fn IS_VALID_ROP4(rop4: u32) -> u8 {
    if (rop4 & 0xFFFF0000) == 0 { 1 } else { 0 }
}
#[inline(always)]
unsafe fn ROP4_FROM_INDEX(index: u32) -> u32 {
    index | (index << 8)
}

// RECTL_* are FASTCALL on i386 (@Name@N decoration), cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(dest: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> u8;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_vMakeWellOrdered(rect: *mut RECTL);
    fn RECTL_bIntersectRect(dest: *mut RECTL, src1: *const RECTL, src2: *const RECTL) -> u8;
}
extern "system" {
    fn EBRUSHOBJ_psoPattern(pebo: *mut EBRUSHOBJ) -> *mut SURFOBJ;
    fn CLIPOBJ_bEnum(ClipRegion: *mut CLIPOBJ, size: u32, RectEnum: *mut core::ffi::c_void) -> u8;
    fn CLIPOBJ_cEnumStart(ClipRegion: *mut CLIPOBJ, bAll: u32, iType: u32, iDirection: u32, cLimit: u32) -> u32;
    fn SafeProbeForRead(Address: *const core::ffi::c_void, Length: usize, Alignment: u32) -> i32;
}

// BRUSHOBJ_psoPattern is a macro (brush.h) — inline-expand.
#[inline(always)]
unsafe fn BRUSHOBJ_psoPattern(pbo: *mut BRUSHOBJ) -> *mut SURFOBJ {
    EBRUSHOBJ_psoPattern(pbo as *mut EBRUSHOBJ)
}

// ---- file-scope preamble (translated) ----
pub type PSTRETCHRECTFUNC = Option<unsafe extern "system" fn(
    OutputObj: *mut SURFOBJ,
    InputObj: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ColorTranslation: *mut XLATEOBJ,
    OutputRect: *mut RECTL,
    InputRect: *mut RECTL,
    MaskOrigin: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
    Rop4: ROP4,
) -> u8>;

const ROP4_NOOP: ROP4 = 0xAA0029;

unsafe extern "system" fn CallDibStretchBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ColorTranslation: *mut XLATEOBJ,
    OutputRect: *mut RECTL,
    InputRect: *mut RECTL,
    MaskOrigin: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
    Rop4: ROP4,
) -> u8 {

    let mut RealBrushOrigin: POINTL = core::mem::zeroed();
    let psoPattern: *mut SURFOBJ;
    let bResult: u8;

    if BrushOrigin.is_null() {
        RealBrushOrigin.x = 0;
        RealBrushOrigin.y = 0;
    } else {
        RealBrushOrigin = *BrushOrigin;
    }

    if ROP4_USES_PATTERN(Rop4) != 0 && !pbo.is_null() && (*pbo).iSolidColor == 0xFFFFFFFF {
        psoPattern = BRUSHOBJ_psoPattern(pbo);

        if psoPattern.is_null() {
            return 0u8;
        }
    } else {
        psoPattern = core::ptr::null_mut();
    }

    bResult = (DibFunctionsForBitmapFormat[(*psoDest).iBitmapFormat as usize].DIB_StretchBlt.unwrap())(
        psoDest,
        psoSource,
        Mask,
        psoPattern,
        OutputRect,
        InputRect,
        MaskOrigin,
        pbo,
        &mut RealBrushOrigin,
        ColorTranslation,
        Rop4,
    );

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn EngStretchBltROP(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    BrushOrigin: *mut POINTL,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    MaskOrigin: *mut POINTL,
    Mode: u32,
    pbo: *mut BRUSHOBJ,
    Rop4: ROP4,
) -> u8 {
    let mut InputRect: RECTL = core::mem::zeroed();
    let mut OutputRect: RECTL = core::mem::zeroed();
    let mut Translate: POINTL = core::mem::zeroed();
    let mut EnterLeaveSource: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut EnterLeaveDest: INTENG_ENTER_LEAVE = core::mem::zeroed();
    let mut psoInput: *mut SURFOBJ = core::ptr::null_mut();
    let mut psoOutput: *mut SURFOBJ = core::ptr::null_mut();
    let BltRectFunc: PSTRETCHRECTFUNC;
    let mut Ret: u8 = 1u8;
    let mut AdjustedBrushOrigin: POINTL = core::mem::zeroed();
    let UsesSource: u8 = ROP4_USES_SOURCE(Rop4);

    let clippingType: u8;
    let mut ClipRect: RECTL = core::mem::zeroed();
    let mut RectEnum: RECT_ENUM = core::mem::zeroed();
    let mut EnumMore: u8;
    let Direction: u32;
    let mut CombinedRect: RECTL = core::mem::zeroed();
    let mut InputToCombinedRect: RECTL = core::mem::zeroed();
    let mut i: u32;

    let DstHeight: i32;
    let DstWidth: i32;
    let SrcHeight: i32;
    let SrcWidth: i32;

    let cxSrc: i32;
    let cySrc: i32;
    let cxDest: i32;
    let cyDest: i32;
    let bLeftToRight: u8;
    let bTopToBottom: u8;
    let mut lTmp: i32;

    cxSrc = (*prclSrc).right - (*prclSrc).left;
    cySrc = (*prclSrc).bottom - (*prclSrc).top;
    cxDest = (*prclDest).right - (*prclDest).left;
    cyDest = (*prclDest).bottom - (*prclDest).top;

    if ((cxSrc < 0) && (cxDest < 0)) || ((cxSrc >= 0) && (cxDest >= 0)) {
        bLeftToRight = 0u8;
    } else {
        bLeftToRight = 1u8;
    }

    if ((cySrc < 0) && (cyDest < 0)) || ((cySrc >= 0) && (cyDest >= 0)) {
        bTopToBottom = 0u8;
    } else {
        bTopToBottom = 1u8;
    }

    OutputRect = *prclDest;
    RECTL_vMakeWellOrdered(&mut OutputRect);
    *prclDest = OutputRect;

    if Rop4 == ROP4_NOOP {
        return 1u8;
    }

    if ClipRegion == core::ptr::null_mut() {
        clippingType = DC_TRIVIAL as u8;
    } else {
        clippingType = (*ClipRegion).iDComplexity as u8;
    }

    OutputRect = *prclDest;
    if OutputRect.right < OutputRect.left {
        OutputRect.left = (*prclDest).right;
        OutputRect.right = (*prclDest).left;
    }
    if OutputRect.bottom < OutputRect.top {
        OutputRect.top = (*prclDest).bottom;
        OutputRect.bottom = (*prclDest).top;
    }

    if UsesSource != 0 {
        if prclSrc == core::ptr::null_mut() {
            return 0u8;
        }
        InputRect = *prclSrc;

        if IntEngEnter(&mut EnterLeaveSource, psoSource, &mut InputRect, 1i32, &mut Translate, &mut psoInput) == 0 {
            return 0u8;
        }

        InputRect.left += Translate.x;
        InputRect.right += Translate.x;
        InputRect.top += Translate.y;
        InputRect.bottom += Translate.y;
    } else {
        InputRect.left = 0;
        InputRect.right = OutputRect.right - OutputRect.left;
        InputRect.top = 0;
        InputRect.bottom = OutputRect.bottom - OutputRect.top;
        psoInput = core::ptr::null_mut();
    }

    if ClipRegion != core::ptr::null_mut() {
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

    if OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top {
        if UsesSource != 0 {
            IntEngLeave(&mut EnterLeaveSource);
        }
        return 1u8;
    }

    if IntEngEnter(&mut EnterLeaveDest, psoDest, &mut OutputRect, 0i32, &mut Translate, &mut psoOutput) == 0 {
        if UsesSource != 0 {
            IntEngLeave(&mut EnterLeaveSource);
        }
        return 0u8;
    }

    OutputRect.left += Translate.x;
    OutputRect.right += Translate.x;
    OutputRect.top += Translate.y;
    OutputRect.bottom += Translate.y;

    if BrushOrigin != core::ptr::null_mut() {
        AdjustedBrushOrigin.x = (*BrushOrigin).x + Translate.x;
        AdjustedBrushOrigin.y = (*BrushOrigin).y + Translate.y;
    } else {
        AdjustedBrushOrigin = Translate;
    }

    BltRectFunc = Some(CallDibStretchBlt);

    DstHeight = OutputRect.bottom - OutputRect.top;
    DstWidth = OutputRect.right - OutputRect.left;
    SrcHeight = InputRect.bottom - InputRect.top;
    SrcWidth = InputRect.right - InputRect.left;

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

            Ret = (BltRectFunc.unwrap())(
                psoOutput,
                psoInput,
                Mask,
                ColorTranslation,
                &mut OutputRect,
                &mut InputRect,
                MaskOrigin,
                pbo,
                &mut AdjustedBrushOrigin,
                Rop4,
            );
        }
        DC_RECT => {
            ClipRect.left = (*ClipRegion).rclBounds.left + Translate.x;
            ClipRect.right = (*ClipRegion).rclBounds.right + Translate.x;
            ClipRect.top = (*ClipRegion).rclBounds.top + Translate.y;
            ClipRect.bottom = (*ClipRegion).rclBounds.bottom + Translate.y;
            if RECTL_bIntersectRect(&mut CombinedRect, &OutputRect, &ClipRect) != 0 {
                InputToCombinedRect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
                InputToCombinedRect.bottom = InputRect.top + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
                InputToCombinedRect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
                InputToCombinedRect.right = InputRect.left + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;

                if bLeftToRight != 0 {
                    lTmp = CombinedRect.left;
                    CombinedRect.left = CombinedRect.right;
                    CombinedRect.right = lTmp;
                }

                if bTopToBottom != 0 {
                    lTmp = CombinedRect.top;
                    CombinedRect.top = CombinedRect.bottom;
                    CombinedRect.bottom = lTmp;
                }

                Ret = (BltRectFunc.unwrap())(
                    psoOutput,
                    psoInput,
                    Mask,
                    ColorTranslation,
                    &mut CombinedRect,
                    &mut InputToCombinedRect,
                    MaskOrigin,
                    pbo,
                    &mut AdjustedBrushOrigin,
                    Rop4,
                );
            }
        }
        DC_COMPLEX => {
            if psoOutput == psoInput {
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
            CLIPOBJ_cEnumStart(ClipRegion, (0u8) as _, CT_RECTANGLES, Direction, 0);
            loop {
                EnumMore = CLIPOBJ_bEnum(
                    ClipRegion,
                    core::mem::size_of::<RECT_ENUM>() as u32,
                    &mut RectEnum as *mut _ as *mut core::ffi::c_void,
                );
                i = 0;
                while i < RectEnum.c {
                    ClipRect.left = RectEnum.arcl[i as usize].left + Translate.x;
                    ClipRect.right = RectEnum.arcl[i as usize].right + Translate.x;
                    ClipRect.top = RectEnum.arcl[i as usize].top + Translate.y;
                    ClipRect.bottom = RectEnum.arcl[i as usize].bottom + Translate.y;
                    if RECTL_bIntersectRect(&mut CombinedRect, &OutputRect, &ClipRect) != 0 {
                        InputToCombinedRect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
                        InputToCombinedRect.bottom = InputRect.top + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
                        InputToCombinedRect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
                        InputToCombinedRect.right = InputRect.left + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;

                        if bLeftToRight != 0 {
                            lTmp = CombinedRect.left;
                            CombinedRect.left = CombinedRect.right;
                            CombinedRect.right = lTmp;
                        }

                        if bTopToBottom != 0 {
                            lTmp = CombinedRect.top;
                            CombinedRect.top = CombinedRect.bottom;
                            CombinedRect.bottom = lTmp;
                        }

                        Ret = (BltRectFunc.unwrap())(
                            psoOutput,
                            psoInput,
                            Mask,
                            ColorTranslation,
                            &mut CombinedRect,
                            &mut InputToCombinedRect,
                            MaskOrigin,
                            pbo,
                            &mut AdjustedBrushOrigin,
                            Rop4,
                        );
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

    IntEngLeave(&mut EnterLeaveDest);
    if UsesSource != 0 {
        IntEngLeave(&mut EnterLeaveSource);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn EngStretchBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    BrushOrigin: *mut POINTL,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    MaskOrigin: *mut POINTL,
    Mode: u32,
) -> u8 {

    EngStretchBltROP(
        psoDest,
        psoSource,
        Mask,
        ClipRegion,
        ColorTranslation,
        pca,
        BrushOrigin,
        prclDest,
        prclSrc,
        MaskOrigin,
        Mode,
        core::ptr::null_mut(),
        ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY),
    )
}
#[no_mangle]
pub unsafe extern "system" fn IntEngStretchBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    MaskSurf: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    pMaskOrigin: *mut POINTL,
    pbo: *mut BRUSHOBJ,
    BrushOrigin: *mut POINTL,
    Rop4: u32,
) -> u8 {

    let mut ret: u8;
    let mut MaskOrigin = POINTL { x: 0, y: 0 };
    let psurfDest: *mut SURFACE;
    let mut InputClippedRect: RECTL;
    let mut InputRect: RECTL;
    let mut OutputRect: RECTL;
    let UsesSource = ROP4_USES_SOURCE(Rop4);
    let mut InputClWidth: i32;
    let mut InputClHeight: i32;
    let mut InputWidth: i32;
    let mut InputHeight: i32;
    let mut lTmp: i32;
    let mut cxSrc: i32;
    let mut cySrc: i32;
    let mut cxDest: i32;
    let mut cyDest: i32;
    let bTopToBottom: u8;
    let bLeftToRight: u8;
    let Case0000: i32;
    let Case0001: i32;
    let Case0010: i32;
    let Case0011: i32;
    let Case0100: i32;
    let Case0101: i32;
    let Case0110: i32;
    let Case0111: i32;
    let Case1000: i32;
    let Case1001: i32;
    let Case1010: i32;
    let Case1011: i32;
    let Case1100: i32;
    let Case1101: i32;
    let Case1110: i32;

    let ClipRegion = if ClipRegion.is_null() {
        core::ptr::addr_of!(gxcoTrivial) as *mut CLIPOBJ
    } else {
        ClipRegion
    };

    psurfDest = (psoDest as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    cxSrc = (*SourceRect).right - (*SourceRect).left;
    cySrc = (*SourceRect).bottom - (*SourceRect).top;
    cxDest = (*DestRect).right - (*DestRect).left;
    cyDest = (*DestRect).bottom - (*DestRect).top;

    Case1110 = ((cxDest > 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc < 0)) as i32;
    Case1101 = ((cxDest > 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc > 0)) as i32;
    Case1100 = ((cxDest > 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc < 0)) as i32;
    Case1011 = ((cxDest > 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc > 0)) as i32;
    Case1010 = ((cxDest > 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc < 0)) as i32;
    Case1001 = ((cxDest > 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc > 0)) as i32;
    Case1000 = ((cxDest > 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc < 0)) as i32;
    Case0111 = ((cxDest < 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc > 0)) as i32;
    Case0110 = ((cxDest < 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc < 0)) as i32;
    Case0101 = ((cxDest < 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc > 0)) as i32;
    Case0100 = ((cxDest < 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc < 0)) as i32;
    Case0011 = ((cxDest < 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc > 0)) as i32;
    Case0010 = ((cxDest < 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc < 0)) as i32;
    Case0001 = ((cxDest < 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc > 0)) as i32;
    Case0000 = ((cxDest < 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc < 0)) as i32;

    RECTL_vMakeWellOrdered(DestRect);
    OutputRect = *DestRect;

    let bLeftToRight = if ((cxSrc < 0) && (cxDest < 0)) || ((cxSrc >= 0) && (cxDest >= 0)) {
        0u8
    } else {
        1u8
    };

    let bTopToBottom = if ((cySrc < 0) && (cyDest < 0)) || ((cySrc >= 0) && (cyDest >= 0)) {
        0u8
    } else {
        1u8
    };

    if (abs((*DestRect).right - (*DestRect).left) == abs((*SourceRect).right - (*SourceRect).left)) &&
       (abs((*DestRect).bottom - (*DestRect).top) == abs((*SourceRect).bottom - (*SourceRect).top))
    {
        if Case0000 != 0 || Case0001 != 0 || Case0010 != 0 || Case0011 != 0 {
            lTmp = (*SourceRect).left;
            (*SourceRect).left = (*SourceRect).right;
            (*SourceRect).right = lTmp;

            lTmp = (*SourceRect).top;
            (*SourceRect).top = (*SourceRect).bottom;
            (*SourceRect).bottom = lTmp;
        }

        if Case0100 != 0 || Case0101 != 0 || Case0110 != 0 || Case0111 != 0 {
            lTmp = (*SourceRect).left;
            (*SourceRect).left = (*SourceRect).right;
            (*SourceRect).right = lTmp;
        }

        if Case1000 != 0 || Case1001 != 0 || Case1010 != 0 || Case1011 != 0 {
            lTmp = (*SourceRect).top;
            (*SourceRect).top = (*SourceRect).bottom;
            (*SourceRect).bottom = lTmp;
        }

        if bLeftToRight != 0 {
            lTmp = (*DestRect).left;
            (*DestRect).left = (*DestRect).right;
            (*DestRect).right = lTmp;
        }

        if bTopToBottom != 0 {
            lTmp = (*DestRect).top;
            (*DestRect).top = (*DestRect).bottom;
            (*DestRect).bottom = lTmp;
        }

        return IntEngBitBlt(
            psoDest,
            psoSource,
            MaskSurf,
            ClipRegion,
            ColorTranslation,
            DestRect,
            SourceRect as *mut POINTL,
            pMaskOrigin,
            pbo,
            BrushOrigin,
            Rop4,
        ) as u8;
    }

    if cxSrc < 0 {
        lTmp = (*SourceRect).left;
        (*SourceRect).left = (*SourceRect).right;
        (*SourceRect).right = lTmp;
        lTmp = (*DestRect).left;
        (*DestRect).left = (*DestRect).right;
        (*DestRect).right = lTmp;
        cxSrc = -cxSrc;
        cxDest = -cxDest;
    }

    if cySrc < 0 {
        lTmp = (*DestRect).top;
        (*DestRect).top = (*DestRect).bottom;
        (*DestRect).bottom = lTmp;
        lTmp = (*SourceRect).top;
        (*SourceRect).top = (*SourceRect).bottom;
        (*SourceRect).bottom = lTmp;
        cySrc = -cySrc;
        cyDest = -cyDest;
    }

    if Case0010 != 0 || Case0111 != 0 {
        (*DestRect).left -= 1;
    }

    if Case0010 != 0 || Case0111 != 0 || Case1000 != 0 || Case1101 != 0 {
        (*SourceRect).left -= 1;
        (*SourceRect).right -= 1;
    }

    if Case0001 != 0 || Case0100 != 0 || Case1011 != 0 || Case1110 != 0 {
        (*SourceRect).top -= 1;
        (*SourceRect).bottom -= 1;
    }

    if Case0011 != 0 || Case0110 != 0 || Case1001 != 0 || Case1100 != 0 {
        (*SourceRect).left -= 1;
        (*SourceRect).right -= 1;

        (*SourceRect).top -= 1;
        (*SourceRect).bottom -= 1;
    }

    if Case0000 != 0 || Case1010 != 0 {
        (*SourceRect).top += 1;
        (*SourceRect).bottom += 1;

        (*DestRect).top += 1;
        (*DestRect).bottom += 1;
    }

    if Case0000 != 0 || Case0101 != 0 {
        (*SourceRect).left += 1;
        (*SourceRect).right += 1;

        (*DestRect).left += 1;
        (*DestRect).right += 1;
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

    if psoSource.is_null() {
        return 0u8;
    }
    InputRect = *SourceRect;

    if InputRect.right < InputRect.left || InputRect.bottom < InputRect.top {
        return 1u8;
    }

    if (*ClipRegion).iDComplexity as u32 != DC_TRIVIAL {
        if RECTL_bIntersectRect(
            core::ptr::addr_of_mut!(OutputRect),
            core::ptr::addr_of!(InputClippedRect),
            core::ptr::addr_of!((*ClipRegion).rclBounds),
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

    if !pMaskOrigin.is_null() {
        MaskOrigin.x = (*pMaskOrigin).x;
        MaskOrigin.y = (*pMaskOrigin).y;
    }

    ret = 0u8;

    if UsesSource != 0 {
    }

    if ((*psurfDest).flags & HOOK_STRETCHBLTROP) != 0 {
        
        type StretchBltROPFunc = unsafe extern "system" fn(
            *mut SURFOBJ,
            *mut SURFOBJ,
            *mut SURFOBJ,
            *mut CLIPOBJ,
            *mut XLATEOBJ,
            *mut COLORADJUSTMENT,
            *mut POINTL,
            *mut RECTL,
            *mut RECTL,
            *mut POINTL,
            u32,
            *mut BRUSHOBJ,
            u32,
        ) -> u8;

        let funcs = GDIDEVFUNCS(psoDest);
        let stretch_func = *(funcs as *const StretchBltROPFunc);
        
        ret = stretch_func(
            psoDest,
            psoSource,
            MaskSurf,
            ClipRegion,
            ColorTranslation,
            pca,
            BrushOrigin,
            core::ptr::addr_of_mut!(OutputRect),
            core::ptr::addr_of_mut!(InputRect),
            core::ptr::addr_of_mut!(MaskOrigin),
            COLORONCOLOR,
            pbo,
            Rop4,
        );
    }

    if ret == 0 {
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

        ret = EngStretchBltROP(
            psoDest,
            psoSource,
            MaskSurf,
            ClipRegion,
            ColorTranslation,
            pca,
            BrushOrigin,
            core::ptr::addr_of_mut!(OutputRect),
            core::ptr::addr_of_mut!(InputRect),
            core::ptr::addr_of_mut!(MaskOrigin),
            COLORONCOLOR,
            pbo,
            Rop4,
        );
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngStretchBlt(
    psoDest: *mut SURFOBJ,
    psoSource: *mut SURFOBJ,
    Mask: *mut SURFOBJ,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    pca: *mut COLORADJUSTMENT,
    BrushOrigin: *mut POINTL,
    prclDest: *mut RECTL,
    prclSrc: *mut RECTL,
    MaskOrigin: *mut POINTL,
    Mode: u32,
) -> u32 {
    if crate::safe_user_io::umpd_call_refused() {
        return 0;
    }

    let mut ca: COLORADJUSTMENT = core::mem::zeroed();
    let mut lBrushOrigin: POINTL = core::mem::zeroed();
    let mut rclDest: RECTL = core::mem::zeroed();
    let mut rclSrc: RECTL = core::mem::zeroed();
    let mut lMaskOrigin: POINTL = core::mem::zeroed();
    let mut Status: i32;
    let pca_local: *mut COLORADJUSTMENT;

    if !pca.is_null() {
        Status = SafeProbeForRead(pca as *const core::ffi::c_void, core::mem::size_of::<COLORADJUSTMENT>(), 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(BrushOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(prclDest as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(prclSrc as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(MaskOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
        }
        if !NT_SUCCESS(Status) {
            return 0;
        }
        // The probes above VALIDATE only. Each capture is the access, so every one
        // of them has to be the protected copy and take the probe-failure exit.
        if !crate::copy::capture_from_user(&mut ca as *mut _ as *mut core::ffi::c_void, pca as *const core::ffi::c_void, core::mem::size_of::<COLORADJUSTMENT>()) {
            return 0;
        }
        pca_local = &mut ca;
        if !crate::copy::capture_from_user(&mut lBrushOrigin as *mut _ as *mut core::ffi::c_void, BrushOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut rclDest as *mut _ as *mut core::ffi::c_void, prclDest as *const core::ffi::c_void, core::mem::size_of::<RECTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut rclSrc as *mut _ as *mut core::ffi::c_void, prclSrc as *const core::ffi::c_void, core::mem::size_of::<RECTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut lMaskOrigin as *mut _ as *mut core::ffi::c_void, MaskOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>()) {
            return 0;
        }
    } else {
        Status = SafeProbeForRead(BrushOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(prclDest as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(prclSrc as *const core::ffi::c_void, core::mem::size_of::<RECTL>(), 1);
        }
        if NT_SUCCESS(Status) {
            Status = SafeProbeForRead(MaskOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>(), 1);
        }
        if !NT_SUCCESS(Status) {
            return 0;
        }
        // The probes above VALIDATE only. Each capture is the access, so every one
        // of them has to be the protected copy and take the probe-failure exit.
        if !crate::copy::capture_from_user(&mut lBrushOrigin as *mut _ as *mut core::ffi::c_void, BrushOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut rclDest as *mut _ as *mut core::ffi::c_void, prclDest as *const core::ffi::c_void, core::mem::size_of::<RECTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut rclSrc as *mut _ as *mut core::ffi::c_void, prclSrc as *const core::ffi::c_void, core::mem::size_of::<RECTL>()) {
            return 0;
        }
        if !crate::copy::capture_from_user(&mut lMaskOrigin as *mut _ as *mut core::ffi::c_void, MaskOrigin as *const core::ffi::c_void, core::mem::size_of::<POINTL>()) {
            return 0;
        }
        pca_local = core::ptr::null_mut();
    }

    EngStretchBlt(psoDest, psoSource, Mask, ClipRegion, ColorTranslation, pca_local, &mut lBrushOrigin, &mut rclDest, &mut rclSrc, &mut lMaskOrigin, Mode) as u32 // nolint: raw-user-access (umpd_call_refused: kernel-mode callers only)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          GDI stretch blt functions
//  * FILE:             win32ss/gdi/eng/stretchblt.c
//  * PROGRAMERS:       Jason Filby
//  *                   Doug Lyons
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /***************************************************************************************************************************
//   We want to receive and send the flip state along to existing functions without changing their parameter lists.
//   So a way that we can do this is to use the DestRect to carry this information along with it.
//   Since there are four values, we can use their relative positions (coordinates) to indicate the four flip conditions.
//   if delta-x == 0 then there can be no Left-to-Right flip. If delta-y == 0 there can be no Top-to-Bottom flip.
//   So we can set the four flip conditions based on BOOLEAN flags as follows:
//
//   We will use internal bits bTopToBottom and bLeftToRight as follows:
//
//   !bTopToBottom && !bLeftToRight means no flips      therefore left < right and top < bottom (normal well-formed rectangle)
//   bTopToBottom means there is a Top-To-Bottom flip   therefore left < right and top > bottom
//   bLeftToRight means there is a Left-To-Right flip   therefore left > right and top < bottom
//   bLeftToRight && bTopToBottom means both flips      therefore left > right and top > bottom
// ****************************************************************************************************************************/
//
// typedef BOOLEAN (APIENTRY *PSTRETCHRECTFUNC)(SURFOBJ* OutputObj,
//                                             SURFOBJ* InputObj,
//                                             SURFOBJ* Mask,
//                                             XLATEOBJ* ColorTranslation,
//                                             RECTL* OutputRect,
//                                             RECTL* InputRect,
//                                             POINTL* MaskOrigin,
//                                             BRUSHOBJ* pbo,
//                                             POINTL* BrushOrigin,
//                                             ROP4 Rop4);
//
// static BOOLEAN APIENTRY
// CallDibStretchBlt(SURFOBJ* psoDest,
//                   SURFOBJ* psoSource,
//                   SURFOBJ* Mask,
//                   XLATEOBJ* ColorTranslation,
//                   RECTL* OutputRect,
//                   RECTL* InputRect,
//                   POINTL* MaskOrigin,
//                   BRUSHOBJ* pbo,
//                   POINTL* BrushOrigin,
//                   ROP4 Rop4)
// {
//     POINTL RealBrushOrigin;
//     SURFOBJ* psoPattern;
//     BOOL bResult;
//
//     DPRINT("Entering CallDibStretchBlt: psoSource cx/cy (%d/%d), psoDest cx/cy (%d/%d) OutputRect: (%d,%d)-(%d,%d)\n",
//            psoSource->sizlBitmap.cx, psoSource->sizlBitmap.cy,
//            psoDest->sizlBitmap.cx, psoDest->sizlBitmap.cy,
//            OutputRect->left, OutputRect->top, OutputRect->right, OutputRect->bottom);
//
//     if (BrushOrigin == NULL)
//     {
//         RealBrushOrigin.x = RealBrushOrigin.y = 0;
//     }
//     else
//     {
//         RealBrushOrigin = *BrushOrigin;
//     }
//
//     /* Pattern brush */
//     if (ROP4_USES_PATTERN(Rop4) && pbo && pbo->iSolidColor == 0xFFFFFFFF)
//     {
//         psoPattern = BRUSHOBJ_psoPattern(pbo);
//
//         if (!psoPattern) return FALSE;
//     }
//     else
//     {
//         psoPattern = NULL;
//     }
//
//     bResult = DibFunctionsForBitmapFormat[psoDest->iBitmapFormat].DIB_StretchBlt(
//                psoDest, psoSource, Mask, psoPattern,
//                OutputRect, InputRect, MaskOrigin, pbo, &RealBrushOrigin,
//                ColorTranslation, Rop4);
//
//     return bResult;
// }
//
//
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngStretchBltROP(
//     IN SURFOBJ  *psoDest,
//     IN SURFOBJ  *psoSource,
//     IN SURFOBJ  *Mask,
//     IN CLIPOBJ  *ClipRegion,
//     IN XLATEOBJ  *ColorTranslation,
//     IN COLORADJUSTMENT  *pca,
//     IN POINTL  *BrushOrigin,
//     IN RECTL  *prclDest,
//     IN RECTL  *prclSrc,
//     IN POINTL  *MaskOrigin,
//     IN ULONG  Mode,
//     IN BRUSHOBJ *pbo,
//     IN ROP4 Rop4)
// {
//     RECTL              InputRect;
//     RECTL              OutputRect;
//     POINTL             Translate;
//     INTENG_ENTER_LEAVE EnterLeaveSource;
//     INTENG_ENTER_LEAVE EnterLeaveDest;
//     SURFOBJ*           psoInput;
//     SURFOBJ*           psoOutput;
//     PSTRETCHRECTFUNC   BltRectFunc;
//     BOOLEAN            Ret = TRUE;
//     POINTL             AdjustedBrushOrigin;
//     BOOL               UsesSource = ROP4_USES_SOURCE(Rop4);
//
//     BYTE               clippingType;
//     RECTL              ClipRect;
//     RECT_ENUM          RectEnum;
//     BOOL               EnumMore;
//     ULONG              Direction;
//     RECTL              CombinedRect;
//     RECTL              InputToCombinedRect;
//     unsigned           i;
//
//     LONG DstHeight;
//     LONG DstWidth;
//     LONG SrcHeight;
//     LONG SrcWidth;
//
//     LONG cxSrc, cySrc, cxDest, cyDest;
//     BOOLEAN bLeftToRight, bTopToBottom;
//     LONG lTmp;
//
//     DPRINT("Entering EngStretchBltROP: prclSrc: (%d/%d)-(%d/%d) prclDest: (%d,%d)-(%d,%d)\n",
//            prclSrc->left, prclSrc->top, prclSrc->right, prclSrc->bottom,
//            prclDest->left, prclDest->top, prclDest->right, prclDest->bottom);
//
//     cxSrc = prclSrc->right - prclSrc->left;
//     cySrc = prclSrc->bottom - prclSrc->top;
//     cxDest = prclDest->right - prclDest->left;
//     cyDest = prclDest->bottom - prclDest->top;
//
//     /* Here we do the tests and set our conditions */
//     if (((cxSrc < 0) && (cxDest < 0)) || ((cxSrc >= 0) && (cxDest >= 0)))
//         bLeftToRight = FALSE;
//     else
//         bLeftToRight = TRUE;
//
//     if (((cySrc < 0) && (cyDest < 0)) || ((cySrc >= 0) && (cyDest >= 0)))
//         bTopToBottom = FALSE;
//     else
//         bTopToBottom = TRUE;
//
//     /* Make Well Ordered to start */
//     OutputRect = *prclDest;
//     RECTL_vMakeWellOrdered(&OutputRect);
//     *prclDest = OutputRect;
//
//     if (Rop4 == ROP4_NOOP)
//     {
//         /* Copy destination onto itself: nop */
//         return TRUE;
//     }
//
//
//     /* Determine clipping type */
//     if (ClipRegion == (CLIPOBJ *) NULL)
//     {
//         clippingType = DC_TRIVIAL;
//     }
//     else
//     {
//         clippingType = ClipRegion->iDComplexity;
//     }
//
//     OutputRect = *prclDest;
//     if (OutputRect.right < OutputRect.left)
//     {
//         OutputRect.left = prclDest->right;
//         OutputRect.right = prclDest->left;
//     }
//     if (OutputRect.bottom < OutputRect.top)
//     {
//         OutputRect.top = prclDest->bottom;
//         OutputRect.bottom = prclDest->top;
//     }
//
//     if (UsesSource)
//     {
//         if (NULL == prclSrc)
//         {
//             return FALSE;
//         }
//         InputRect = *prclSrc;
//
//         if (! IntEngEnter(&EnterLeaveSource, psoSource, &InputRect, TRUE,
//                           &Translate, &psoInput))
//         {
//             return FALSE;
//         }
//
//         InputRect.left += Translate.x;
//         InputRect.right += Translate.x;
//         InputRect.top += Translate.y;
//         InputRect.bottom += Translate.y;
//     }
//     else
//     {
//         InputRect.left = 0;
//         InputRect.right = OutputRect.right - OutputRect.left;
//         InputRect.top = 0;
//         InputRect.bottom = OutputRect.bottom - OutputRect.top;
//         psoInput = NULL;
//     }
//
//     if (NULL != ClipRegion)
//     {
//         if (OutputRect.left < ClipRegion->rclBounds.left)
//         {
//             InputRect.left += ClipRegion->rclBounds.left - OutputRect.left;
//             OutputRect.left = ClipRegion->rclBounds.left;
//         }
//         if (ClipRegion->rclBounds.right < OutputRect.right)
//         {
//             InputRect.right -= OutputRect.right - ClipRegion->rclBounds.right;
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
//     /* Check for degenerate case: if height or width of OutputRect is 0 pixels there's
//        nothing to do */
//     if (OutputRect.right <= OutputRect.left || OutputRect.bottom <= OutputRect.top)
//     {
//         if (UsesSource)
//         {
//             IntEngLeave(&EnterLeaveSource);
//         }
//         return TRUE;
//     }
//
//     if (! IntEngEnter(&EnterLeaveDest, psoDest, &OutputRect, FALSE, &Translate, &psoOutput))
//     {
//         if (UsesSource)
//         {
//             IntEngLeave(&EnterLeaveSource);
//         }
//         return FALSE;
//     }
//
//     OutputRect.left += Translate.x;
//     OutputRect.right += Translate.x;
//     OutputRect.top += Translate.y;
//     OutputRect.bottom += Translate.y;
//
//     if (BrushOrigin)
//     {
//         AdjustedBrushOrigin.x = BrushOrigin->x + Translate.x;
//         AdjustedBrushOrigin.y = BrushOrigin->y + Translate.y;
//     }
//     else
//     {
//         AdjustedBrushOrigin = Translate;
//     }
//
//     BltRectFunc = CallDibStretchBlt;
//
//     DstHeight = OutputRect.bottom - OutputRect.top;
//     DstWidth = OutputRect.right - OutputRect.left;
//     SrcHeight = InputRect.bottom - InputRect.top;
//     SrcWidth = InputRect.right - InputRect.left;
//
//     DPRINT("bLeftToRight is '%d' and bTopToBottom is '%d'.\n", bLeftToRight, bTopToBottom);
//
//     switch (clippingType)
//     {
//         case DC_TRIVIAL:
//             if (bLeftToRight)
//             {
//                lTmp = OutputRect.left;
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
//             DPRINT("About to call CallDibStretchBlt: OutputRect: (%d,%d)-(%d,%d)\n",
//                    OutputRect.left, OutputRect.top, OutputRect.right, OutputRect.bottom);
//
//             Ret = (*BltRectFunc)(psoOutput, psoInput, Mask,
//                          ColorTranslation, &OutputRect, &InputRect, MaskOrigin,
//                          pbo, &AdjustedBrushOrigin, Rop4);
//             break;
//         case DC_RECT:
//             // Clip the blt to the clip rectangle
//             ClipRect.left = ClipRegion->rclBounds.left + Translate.x;
//             ClipRect.right = ClipRegion->rclBounds.right + Translate.x;
//             ClipRect.top = ClipRegion->rclBounds.top + Translate.y;
//             ClipRect.bottom = ClipRegion->rclBounds.bottom + Translate.y;
//             if (RECTL_bIntersectRect(&CombinedRect, &OutputRect, &ClipRect))
//             {
//                 InputToCombinedRect.top = InputRect.top + (CombinedRect.top - OutputRect.top) * SrcHeight / DstHeight;
//                 InputToCombinedRect.bottom = InputRect.top + (CombinedRect.bottom - OutputRect.top) * SrcHeight / DstHeight;
//                 InputToCombinedRect.left = InputRect.left + (CombinedRect.left - OutputRect.left) * SrcWidth / DstWidth;
//                 InputToCombinedRect.right = InputRect.left + (CombinedRect.right - OutputRect.left) * SrcWidth / DstWidth;
//
//                 if (bLeftToRight)
//                 {
//                    lTmp = CombinedRect.left;
//                     CombinedRect.left = CombinedRect.right;
//                     CombinedRect.right = lTmp;
//                 }
//
//                 if (bTopToBottom)
//                 {
//                     lTmp = CombinedRect.top;
//                     CombinedRect.top = CombinedRect.bottom;
//                     CombinedRect.bottom = lTmp;
//                 }
//
//                 DPRINT("About to call CallDibStretchBlt: CombinedRect: (%d,%d)-(%d,%d)\n",
//                        CombinedRect.left, CombinedRect.top, CombinedRect.right, CombinedRect.bottom);
//
//                 Ret = (*BltRectFunc)(psoOutput, psoInput, Mask,
//                            ColorTranslation,
//                            &CombinedRect,
//                            &InputToCombinedRect,
//                            MaskOrigin,
//                            pbo,
//                            &AdjustedBrushOrigin,
//                            Rop4);
//             }
//             break;
//         case DC_COMPLEX:
//             if (psoOutput == psoInput)
//             {
//                 if (OutputRect.top < InputRect.top)
//                 {
//                     Direction = OutputRect.left < InputRect.left ?
//                                 CD_RIGHTDOWN : CD_LEFTDOWN;
//                 }
//                 else
//                 {
//                     Direction = OutputRect.left < InputRect.left ?
//                                 CD_RIGHTUP : CD_LEFTUP;
//                 }
//             }
//             else
//             {
//                 Direction = CD_ANY;
//             }
//             CLIPOBJ_cEnumStart(ClipRegion, FALSE, CT_RECTANGLES, Direction, 0);
//             do
//             {
//                 EnumMore = CLIPOBJ_bEnum(ClipRegion,(ULONG) sizeof(RectEnum),
//                                          (PVOID) &RectEnum);
//                 for (i = 0; i < RectEnum.c; i++)
//                 {
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
//                         if (bLeftToRight)
//                         {
//                             lTmp = CombinedRect.left;
//                             CombinedRect.left = CombinedRect.right;
//                             CombinedRect.right = lTmp;
//                         }
//
//                         if (bTopToBottom)
//                         {
//                             lTmp = CombinedRect.top;
//                             CombinedRect.top = CombinedRect.bottom;
//                             CombinedRect.bottom = lTmp;
//                         }
//
//                         DPRINT("About to call CallDibStretchBlt: CombinedRect: (%d,%d)-(%d,%d)\n",
//                                CombinedRect.left, CombinedRect.top, CombinedRect.right, CombinedRect.bottom);
//
//                         Ret = (*BltRectFunc)(psoOutput, psoInput, Mask,
//                            ColorTranslation,
//                            &CombinedRect,
//                            &InputToCombinedRect,
//                            MaskOrigin,
//                            pbo,
//                            &AdjustedBrushOrigin,
//                            Rop4);
//                     }
//                 }
//             }
//             while (EnumMore);
//             break;
//     }
//
//     IntEngLeave(&EnterLeaveDest);
//     if (UsesSource)
//     {
//         IntEngLeave(&EnterLeaveSource);
//     }
//
//     return Ret;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngStretchBlt(
//     IN SURFOBJ  *psoDest,
//     IN SURFOBJ  *psoSource,
//     IN SURFOBJ  *Mask,
//     IN CLIPOBJ  *ClipRegion,
//     IN XLATEOBJ  *ColorTranslation,
//     IN COLORADJUSTMENT  *pca,
//     IN POINTL  *BrushOrigin,
//     IN RECTL  *prclDest,
//     IN RECTL  *prclSrc,
//     IN POINTL  *MaskOrigin,
//     IN ULONG  Mode)
// {
//     return EngStretchBltROP(
//         psoDest,
//         psoSource,
//         Mask,
//         ClipRegion,
//         ColorTranslation,
//         pca,
//         BrushOrigin,
//         prclDest,
//         prclSrc,
//         MaskOrigin,
//         Mode,
//         NULL,
//         ROP4_FROM_INDEX(R3_OPINDEX_SRCCOPY));
// }
//
// BOOL APIENTRY
// IntEngStretchBlt(SURFOBJ *psoDest,
//                  SURFOBJ *psoSource,
//                  SURFOBJ *MaskSurf,
//                  CLIPOBJ *ClipRegion,
//                  XLATEOBJ *ColorTranslation,
//                  COLORADJUSTMENT *pca,
//                  RECTL *DestRect,
//                  RECTL *SourceRect,
//                  POINTL *pMaskOrigin,
//                  BRUSHOBJ *pbo,
//                  POINTL *BrushOrigin,
//                  DWORD Rop4)
// {
//     BOOLEAN ret;
//     POINTL MaskOrigin = {0, 0};
//     SURFACE *psurfDest;
//     //SURFACE *psurfSource = NULL;
//     RECTL InputClippedRect;
//     RECTL InputRect;
//     RECTL OutputRect;
//     BOOL UsesSource = ROP4_USES_SOURCE(Rop4);
//     LONG InputClWidth, InputClHeight, InputWidth, InputHeight;
//     LONG lTmp, cxSrc, cySrc, cxDest, cyDest;
//     BOOLEAN bTopToBottom, bLeftToRight;
//     INT Case0000, Case0001, Case0010, Case0011;
//     INT Case0100, Case0101, Case0110, Case0111;
//     INT Case1000, Case1001, Case1010, Case1011;
//     INT Case1100, Case1101, Case1110;
//
//     DPRINT("Source cx/cy (%d/%d) and Destination cx/cy (%d/%d).\n",
//             psoSource->sizlBitmap.cx, psoSource->sizlBitmap.cy, psoDest->sizlBitmap.cx, psoDest->sizlBitmap.cy);
//
//     DPRINT("Source lDelta is '%d' and Destination lDelta is '%d'.\n",
//             psoSource->lDelta, psoDest->lDelta);
//
//     ASSERT(psoDest);
//     //ASSERT(psoSource); // FIXME!
//     ASSERT(DestRect);
//     ASSERT(SourceRect);
//     //ASSERT(!RECTL_bIsEmptyRect(SourceRect)); // FIXME!
//
//     /* If no clip object is given, use trivial one */
//     if (!ClipRegion)
//     {
//         DPRINT("Using trivial clip region.\n");
//         ClipRegion = (CLIPOBJ *)&gxcoTrivial;
//     }
//     else
//     {
//         DPRINT("ClipRegion->rclBounds is (%d,%d)-(%d,%d).\n",
//            ClipRegion->rclBounds.left, ClipRegion->rclBounds.top,
//            ClipRegion->rclBounds.right, ClipRegion->rclBounds.bottom);
//     }
//
//     psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//
//     /* Sanity check */
//     ASSERT(IS_VALID_ROP4(Rop4));
//
//     cxSrc = SourceRect->right - SourceRect->left;
//     cySrc = SourceRect->bottom - SourceRect->top;
//     cxDest = DestRect->right - DestRect->left;
//     cyDest = DestRect->bottom - DestRect->top;
//
//     Case1110 = ((cxDest > 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc < 0));
//     Case1101 = ((cxDest > 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc > 0));
//     Case1100 = ((cxDest > 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc < 0));
//     Case1011 = ((cxDest > 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc > 0));
//     Case1010 = ((cxDest > 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc < 0));
//     Case1001 = ((cxDest > 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc > 0));
//     Case1000 = ((cxDest > 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc < 0));
//     Case0111 = ((cxDest < 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc > 0));
//     Case0110 = ((cxDest < 0) && (cyDest > 0) && (cxSrc > 0) && (cySrc < 0));
//     Case0101 = ((cxDest < 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc > 0));
//     Case0100 = ((cxDest < 0) && (cyDest > 0) && (cxSrc < 0) && (cySrc < 0));
//     Case0011 = ((cxDest < 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc > 0));
//     Case0010 = ((cxDest < 0) && (cyDest < 0) && (cxSrc > 0) && (cySrc < 0));
//     Case0001 = ((cxDest < 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc > 0));
//     Case0000 = ((cxDest < 0) && (cyDest < 0) && (cxSrc < 0) && (cySrc < 0));
//
//     /* Make DestRect & OutputRect Well Ordered to start */
//     RECTL_vMakeWellOrdered(DestRect);
//     OutputRect = *DestRect;
//
//     /* Here we do the tests and set our conditions */
//     if (((cxSrc < 0) && (cxDest < 0)) || ((cxSrc >= 0) && (cxDest >= 0)))
//         bLeftToRight = FALSE;
//     else
//         bLeftToRight = TRUE;
//
//     if (((cySrc < 0) && (cyDest < 0)) || ((cySrc >= 0) && (cyDest >= 0)))
//         bTopToBottom = FALSE;
//     else
//         bTopToBottom = TRUE;
//
//     DPRINT("bTopToBottom is '%d' and bLeftToRight is '%d'.\n", bTopToBottom, bLeftToRight);
//
//     /* Check if source and dest size are equal */
//     if ((abs(DestRect->right - DestRect->left) == abs(SourceRect->right - SourceRect->left)) &&
//         (abs(DestRect->bottom - DestRect->top) == abs(SourceRect->bottom - SourceRect->top)))
//     {
//         DPRINT("source and dest size are equal.\n");
//
//         DPRINT("IntEngStretchBlt: dstRect: (%d,%d)-(%d,%d)\n",
//                 DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//         if (Case0000 || Case0001 || Case0010 || Case0011) // Destinations X & Y are both negative
//         {
//             lTmp = SourceRect->left;
//             SourceRect->left = SourceRect->right;
//             SourceRect->right = lTmp;
//
//             lTmp = SourceRect->top;
//             SourceRect->top = SourceRect->bottom;
//             SourceRect->bottom = lTmp;
//         }
//
//         if (Case0100 || Case0101 || Case0110 || Case0111)  // Destination X is negative and Y is positive
//         {
//             lTmp = SourceRect->left;
//             SourceRect->left = SourceRect->right;
//             SourceRect->right = lTmp;
//         }
//
//         if (Case1000 || Case1001 || Case1010 || Case1011)  // Destination X is positive and Y is negative
//         {
//             lTmp = SourceRect->top;
//             SourceRect->top = SourceRect->bottom;
//             SourceRect->bottom = lTmp;
//         }
//
//         if (bLeftToRight)
//         {
//             lTmp = DestRect->left;
//             DestRect->left = DestRect->right;
//             DestRect->right = lTmp;
//         }
//
//         if (bTopToBottom)
//         {
//             lTmp = DestRect->top;
//             DestRect->top = DestRect->bottom;
//             DestRect->bottom = lTmp;
//         }
//
//         DPRINT("Calling IntEngBitBlt: SourceRect (%d,%d)-(%d,%d) DestRect: (%d,%d)-(%d,%d)\n",
//            SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom,
//            DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//         /* Pass the request to IntEngBitBlt */
//         return IntEngBitBlt(psoDest,
//                             psoSource,
//                             MaskSurf,
//                             ClipRegion,
//                             ColorTranslation,
//                             DestRect,
//                             (PPOINTL)SourceRect,
//                             pMaskOrigin,
//                             pbo,
//                             BrushOrigin,
//                             Rop4);
//     }
//
//     DPRINT("source and dest size are NOT equal.\n");
//
//     DPRINT("SourceRect: (%d,%d)-(%d,%d) and DestRect: (%d,%d)-(%d,%d)\n",
//            SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom,
//            DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
//
//     /* if cxSrc < 0 then we change the signs for both cxSrc and cxDest and
//      * we reverse their coordinates, because these outcomes are the same.
//      */
//     if (cxSrc < 0)
//     {
//         lTmp = SourceRect->left;
//         SourceRect->left = SourceRect->right;
//         SourceRect->right = lTmp;
//         lTmp = DestRect->left;
//         DestRect->left = DestRect->right;
//         DestRect->right = lTmp;
//         cxSrc = -cxSrc;
//         cxDest = -cxDest;
//     }
//     /* if cySrc < 0 then we change the signs for both cySrc and cyDest and
//      * we reverse their coordinates, because these outcomes are the same.
//       */
//     if (cySrc < 0)
//     {
//         lTmp = DestRect->top;
//         DestRect->top = DestRect->bottom;
//         DestRect->bottom = lTmp;
//         lTmp = SourceRect->top;
//         SourceRect->top = SourceRect->bottom;
//         SourceRect->bottom = lTmp;
//         cySrc = -cySrc;
//         cyDest = -cyDest;
//     }
//
//     if (Case0010 || Case0111)  // Horizontal Flips
//     {
//         DestRect->left--;
//     }
//
//     if (Case0010 || Case0111 || Case1000 || Case1101)  // Horizontal Flips
//     {
//         SourceRect->left--;
//         SourceRect->right--;
//     }
//
//     if (Case0001 || Case0100 || Case1011 || Case1110)  // Vertical Flips
//     {
//         SourceRect->top--;
//         SourceRect->bottom--;
//     }
//
//     if (Case0011 || Case0110 || Case1001 || Case1100)  // Horizontal and Vertical Flips
//     {
//         SourceRect->left--;
//         SourceRect->right--;
//
//         SourceRect->top--;
//         SourceRect->bottom--;
//     }
//
//     if (Case0000 || Case1010)  // No Flip - Just Copy
//     {
//         SourceRect->top++;
//         SourceRect->bottom++;
//
//         DestRect->top++;
//         DestRect->bottom++;
//     }
//
//     if (Case0000 || Case0101)  // No Flip - Just Copy
//     {
//         SourceRect->left++;
//         SourceRect->right++;
//
//         DestRect->left++;
//         DestRect->right++;
//
//     }
//
//     DPRINT("SourceRect: (%d,%d)-(%d,%d) and DestRect: (%d,%d)-(%d,%d)\n",
//                 SourceRect->left, SourceRect->top, SourceRect->right, SourceRect->bottom,
//                 DestRect->left, DestRect->top, DestRect->right, DestRect->bottom);
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
//     if (NULL == psoSource)
//     {
//         DPRINT("Returning FALSE.\n");
//         return FALSE;
//     }
//     InputRect = *SourceRect;
//
//     if (InputRect.right < InputRect.left ||
//             InputRect.bottom < InputRect.top)
//     {
//         DPRINT("Returning TRUE.\n");
//         /* Everything clipped away, nothing to do */
//         return TRUE;
//     }
//
//     DPRINT("InputRect: (%d,%d)-(%d,%d) and InputClippedRect: (%d,%d)-(%d,%d)\n",
//         InputRect.left, InputRect.top, InputRect.right, InputRect.bottom,
//         InputClippedRect.left, InputClippedRect.top, InputClippedRect.right, InputClippedRect.bottom);
//
//     if (ClipRegion->iDComplexity != DC_TRIVIAL)
//     {
//         if (!RECTL_bIntersectRect(&OutputRect, &InputClippedRect,
//                                &ClipRegion->rclBounds))
//         {
//             DPRINT("Returning TRUE.\n");
//             return TRUE;
//         }
//
//         DPRINT("InputClippedRect: (%d,%d)-(%d,%d) and OutputRect: (%d,%d)-(%d,%d)\n",
//                InputClippedRect.left, InputClippedRect.top, InputClippedRect.right, InputClippedRect.bottom,
//                OutputRect.left, OutputRect.top, OutputRect.right, OutputRect.bottom);
//
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
//         DPRINT("Complexity = DC_TRIVIAL.\n");
//         OutputRect = InputClippedRect;
//     }
//
//
//     DPRINT("InputRect: (%d,%d)-(%d,%d) and OutputRect: (%d,%d)-(%d,%d)\n",
//            InputRect.left, InputRect.top, InputRect.right, InputRect.bottom,
//            OutputRect.left, OutputRect.top, OutputRect.right, OutputRect.bottom);
//
//     if (pMaskOrigin != NULL)
//     {
//         MaskOrigin.x = pMaskOrigin->x;
//         MaskOrigin.y = pMaskOrigin->y;
//     }
//
//     /* No success yet */
//     ret = FALSE;
//
//     if (UsesSource)
//     {
//         //psurfSource = CONTAINING_RECORD(psoSource, SURFACE, SurfObj);
//     }
//
//     /* Call the driver's DrvStretchBlt if available */
//     if (psurfDest->flags & HOOK_STRETCHBLTROP)
//     {
//         DPRINT("About to call GDIDEVFUNCS(psoDest).StretchBltROP.\n");
//         /* Drv->StretchBltROP (look at http://www.osronline.com/ddkx/graphics/ddifncs_0z3b.htm ) */
//         ret = GDIDEVFUNCS(psoDest).StretchBltROP(psoDest,
//                                                  psoSource,
//                                                  MaskSurf,
//                                                  ClipRegion,
//                                                  ColorTranslation,
//                                                  pca,
//                                                  BrushOrigin,
//                                                  &OutputRect,
//                                                  &InputRect,
//                                                  &MaskOrigin,
//                                                  COLORONCOLOR,
//                                                  pbo,
//                                                  Rop4);
//     }
//
//     if (! ret)
//     {
//         /* set OutputRect to follow flip */
//         if (bLeftToRight)
//         {
//             lTmp = OutputRect.left;
//             OutputRect.left = OutputRect.right;
//             OutputRect.right = lTmp;
//         }
//
//         if (bTopToBottom)
//         {
//             lTmp = OutputRect.top;
//             OutputRect.top = OutputRect.bottom;
//             OutputRect.bottom = lTmp;
//         }
//
//         DPRINT("Calling EngStretchBltROP: InputRect: (%d,%d)-(%d,%d) and OutputRect: (%d,%d)-(%d,%d)\n",
//                InputRect.left, InputRect.top, InputRect.right, InputRect.bottom,
//                OutputRect.left, OutputRect.top, OutputRect.right, OutputRect.bottom);
//
//         ret = EngStretchBltROP(psoDest,
//                                psoSource,
//                                MaskSurf,
//                                ClipRegion,
//                                ColorTranslation,
//                                pca,
//                                BrushOrigin,
//                                &OutputRect,
//                                &InputRect,
//                                &MaskOrigin,
//                                COLORONCOLOR,
//                                pbo,
//                                Rop4);
//     }
//
//     return ret;
// }
//
// BOOL
// APIENTRY
// NtGdiEngStretchBlt(
//     IN SURFOBJ  *psoDest,
//     IN SURFOBJ  *psoSource,
//     IN SURFOBJ  *Mask,
//     IN CLIPOBJ  *ClipRegion,
//     IN XLATEOBJ  *ColorTranslation,
//     IN COLORADJUSTMENT  *pca,
//     IN POINTL  *BrushOrigin,
//     IN RECTL  *prclDest,
//     IN RECTL  *prclSrc,
//     IN POINTL  *MaskOrigin,
//     IN ULONG  Mode)
// {
//     COLORADJUSTMENT  ca;
//     POINTL  lBrushOrigin;
//     RECTL rclDest;
//     RECTL rclSrc;
//     POINTL lMaskOrigin;
//     NTSTATUS Status;
//
//     if (pca)
//     {
//         Status = SafeProbeForRead(pca, sizeof(COLORADJUSTMENT), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(BrushOrigin, sizeof(POINTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(prclDest, sizeof(RECTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(prclSrc, sizeof(RECTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(MaskOrigin, sizeof(POINTL), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             return FALSE;
//         }
//         else
//         {
//             RtlCopyMemory(&ca,pca, sizeof(COLORADJUSTMENT));
//             pca = &ca;
//             RtlCopyMemory(&lBrushOrigin, BrushOrigin, sizeof(POINTL));
//             RtlCopyMemory(&rclDest, prclDest, sizeof(RECTL));
//             RtlCopyMemory(&rclSrc, prclSrc, sizeof(RECTL));
//             RtlCopyMemory(&lMaskOrigin, MaskOrigin, sizeof(POINTL));
//         }
//     }
//     else
//     {
//         Status = SafeProbeForRead(BrushOrigin, sizeof(POINTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(prclDest, sizeof(RECTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(prclSrc, sizeof(RECTL), 1);
//         if (NT_SUCCESS(Status)) Status = SafeProbeForRead(MaskOrigin, sizeof(POINTL), 1);
//         if (!NT_SUCCESS(Status))
//         {
//             return FALSE;
//         }
//         else
//         {
//             RtlCopyMemory(&lBrushOrigin, BrushOrigin, sizeof(POINTL));
//             RtlCopyMemory(&rclDest, prclDest, sizeof(RECTL));
//             RtlCopyMemory(&rclSrc, prclSrc, sizeof(RECTL));
//             RtlCopyMemory(&lMaskOrigin, MaskOrigin, sizeof(POINTL));
//         }
//     }
//
//     return EngStretchBlt(psoDest, psoSource, Mask, ClipRegion, ColorTranslation, pca, &lBrushOrigin, &rclDest, &rclSrc, &lMaskOrigin, Mode);
// }
//
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
