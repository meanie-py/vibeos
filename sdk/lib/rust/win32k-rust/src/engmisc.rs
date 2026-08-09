use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{NtQuerySystemInformation, RtlGetDefaultCodePage};
use ntkrnl_sys::base::NT_SUCCESS;

// SharedUserData lives at a fixed kernel VA — NOT exported by ntoskrnl.exe.
// Raw-dylib import is WRONG (freeldr's PE loader chokes on the fake
// "ntoskrnl.exe imports SharedUserData" descriptor it generates and OOMs in
// MmFindLocationForPageLookupTable). Cast the fixed VA directly instead.
// Per-arch: amd64/arm64 = 0xFFFFF78000000000, i386 = 0xFFDF0000.
#[inline(always)]
unsafe fn shared_user_data() -> *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA {
    ntkrnl_sys::mm::KI_USER_SHARED_DATA as usize as *mut ntkrnl_sys::ddk::_KUSER_SHARED_DATA
}

// ---- file-scope helpers (preserve C macro semantics) ----
//
// `BitsPerFormat(fmt)` is a macro: `gajBitsPerFormat[Format]` (an
// array exported by `surface.c` / win32k-rust `surface.rs`). NOT a
// real fn — do NOT `extern "C" fn BitsPerFormat`. See
// win32ss/gdi/eng/surface.h:108-109.
#[inline(always)]
unsafe fn BitsPerFormat(format: u32) -> u32 {
    // Use bindgen path so this works whether the surface mod is built into
    // the umbrella or not — gajBitsPerFormat is a real exported global in
    // either case (cohabitation cycles between .c and .rs owner).
    *core::ptr::addr_of!(win32k_sys::win32k::gajBitsPerFormat).cast::<u8>().add(format as usize) as u32
}

// `WIDTH_BYTES_ALIGN32(cx, bpp)` macro from win32ss/gdi/gdi32/objects/bitmap.c:55:
//     ((((cx) * (bpp) + 31) & ~31) >> 3)
#[inline(always)]
fn WIDTH_BYTES_ALIGN32(cx: i32, bpp: u32) -> i32 {
    ((cx.wrapping_mul(bpp as i32).wrapping_add(31)) & !31i32) >> 3
}

// `Int64ShrlMod32(a,b)` macro (sdk/include/xdk/ntbasedef.h:782/787):
//     ((unsigned __int64)(a)) >> (b)
#[inline(always)]
fn Int64ShrlMod32(value: u64, shift: u32) -> u64 {
    value >> shift
}

// `UInt32x32To64(a,b)` macro (sdk/include/xdk/ntbasedef.h:772/775):
//     (unsigned __int64)(unsigned int)a * (unsigned __int64)(unsigned int)b
#[inline(always)]
fn UInt32x32To64(a: u32, b: u32) -> u64 {
    (a as u64).wrapping_mul(b as u64)
}

// `GDIDEVFUNCS(SurfObj)` macro from win32ss/gdi/eng/surface.h:106:
//     ((PDEVOBJ *)(SurfObj)->hdev)->DriverFunctions
// Returns a `&DRIVER_FUNCTIONS` view of the device's function table.
// Do NOT `extern "C" fn GDIDEVFUNCS` — it's a C macro, not a real fn.
#[inline(always)]
unsafe fn GDIDEVFUNCS(pso: *mut SURFOBJ) -> *mut DRIVER_FUNCTIONS {
    let pdev = (*pso).hdev as *mut _PDEVOBJ;
    core::ptr::addr_of_mut!((*pdev).__bindgen_anon_1.DriverFunctions)
}

// ---- file-scope preamble (translated) ----
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn IntEngEnter(
    EnterLeave: PINTENG_ENTER_LEAVE,
    psoDest: *mut SURFOBJ,
    DestRect: *mut RECTL,
    ReadOnly: BOOL,
    Translate: *mut POINTL,
    ppsoOutput: *mut *mut SURFOBJ,
) -> BOOL {
    let mut Exchange: i32;
    let mut BitmapSize: SIZEL = core::mem::zeroed();
    let mut SrcPoint: POINTL = core::mem::zeroed();
    let mut Width: i32;
    let mut ClippedDestRect: RECTL = core::mem::zeroed();

    if (*DestRect).right < (*DestRect).left {
        Exchange = (*DestRect).left;
        (*DestRect).left = (*DestRect).right;
        (*DestRect).right = Exchange;
    }
    if (*DestRect).bottom < (*DestRect).top {
        Exchange = (*DestRect).top;
        (*DestRect).top = (*DestRect).bottom;
        (*DestRect).bottom = Exchange;
    }

    if !psoDest.is_null() && (*psoDest).iType != STYPE_BITMAP as u16 &&
        ((*psoDest).pvScan0.is_null() || (*psoDest).lDelta == 0) {
        let psurfDest = (psoDest as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
        if ((*psurfDest).flags & HOOK_COPYBITS as u32) == 0 {
            return 0;
        }

        BitmapSize.cx = (*DestRect).right - (*DestRect).left;
        BitmapSize.cy = (*DestRect).bottom - (*DestRect).top;
        Width = WIDTH_BYTES_ALIGN32(BitmapSize.cx, BitsPerFormat((*psoDest).iBitmapFormat));
        (*EnterLeave).OutputBitmap = EngCreateBitmap(BitmapSize, Width,
                                                     (*psoDest).iBitmapFormat,
                                                     BMF_TOPDOWN | BMF_NOZEROINIT,
                                                     core::ptr::null_mut());

        if (*EnterLeave).OutputBitmap.is_null() {
            return 0;
        }

        *ppsoOutput = EngLockSurface((*EnterLeave).OutputBitmap as HSURF);
        if (*ppsoOutput).is_null() {
            EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
            return 0;
        }

        (*EnterLeave).DestRect.left = 0;
        (*EnterLeave).DestRect.top = 0;
        (*EnterLeave).DestRect.right = BitmapSize.cx;
        (*EnterLeave).DestRect.bottom = BitmapSize.cy;
        SrcPoint.x = (*DestRect).left;
        SrcPoint.y = (*DestRect).top;
        ClippedDestRect = (*EnterLeave).DestRect;
        if SrcPoint.x < 0 {
            ClippedDestRect.left -= SrcPoint.x;
            SrcPoint.x = 0;
        }
        if (*psoDest).sizlBitmap.cx < SrcPoint.x + ClippedDestRect.right - ClippedDestRect.left {
            ClippedDestRect.right = ClippedDestRect.left + (*psoDest).sizlBitmap.cx - SrcPoint.x;
        }
        if SrcPoint.y < 0 {
            ClippedDestRect.top -= SrcPoint.y;
            SrcPoint.y = 0;
        }
        if (*psoDest).sizlBitmap.cy < SrcPoint.y + ClippedDestRect.bottom - ClippedDestRect.top {
            ClippedDestRect.bottom = ClippedDestRect.top + (*psoDest).sizlBitmap.cy - SrcPoint.y;
        }
        (*EnterLeave).TrivialClipObj = EngCreateClip();
        if (*EnterLeave).TrivialClipObj.is_null() {
            EngUnlockSurface(*ppsoOutput);
            EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
            return 0;
        }
        (*(*EnterLeave).TrivialClipObj).iDComplexity = DC_TRIVIAL as u8;
        if ClippedDestRect.left < (**ppsoOutput).sizlBitmap.cx &&
            0 <= ClippedDestRect.right &&
            SrcPoint.x < (*psoDest).sizlBitmap.cx &&
            ClippedDestRect.top <= (**ppsoOutput).sizlBitmap.cy &&
            0 <= ClippedDestRect.bottom &&
            SrcPoint.y < (*psoDest).sizlBitmap.cy {
            let funcs = GDIDEVFUNCS(psoDest);
            if let Some(copy_bits) = (*funcs).CopyBits {
                if copy_bits(*ppsoOutput, psoDest, (*EnterLeave).TrivialClipObj,
                             core::ptr::null_mut(), &mut ClippedDestRect, &mut SrcPoint) == 0 {
                    EngDeleteClip((*EnterLeave).TrivialClipObj);
                    EngUnlockSurface(*ppsoOutput);
                    EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
                    return 0;
                }
            }
        }
        (*EnterLeave).DestRect.left = (*DestRect).left;
        (*EnterLeave).DestRect.top = (*DestRect).top;
        (*EnterLeave).DestRect.right = (*DestRect).right;
        (*EnterLeave).DestRect.bottom = (*DestRect).bottom;
        (*Translate).x = -(*DestRect).left;
        (*Translate).y = -(*DestRect).top;
    } else {
        (*Translate).x = 0;
        (*Translate).y = 0;
        *ppsoOutput = psoDest;
    }

    if !(*ppsoOutput).is_null() {
        let psurfOutput = (*ppsoOutput as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
        if ((*psurfOutput).flags & HOOK_SYNCHRONIZE) != 0 {
            let funcs = GDIDEVFUNCS(*ppsoOutput);
            if let Some(sync_surface) = (*funcs).SynchronizeSurface {
                sync_surface(*ppsoOutput, DestRect, 0);
            } else if (**ppsoOutput).iType == STYPE_BITMAP as u16 {
                if let Some(synchronize) = (*funcs).Synchronize {
                    synchronize((**ppsoOutput).dhpdev, DestRect);
                }
            }
        }
    } else {
        return 0;
    }

    (*EnterLeave).DestObj = psoDest;
    (*EnterLeave).OutputObj = *ppsoOutput;
    (*EnterLeave).ReadOnly = ReadOnly;

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn IntEngEnter(
    EnterLeave: PINTENG_ENTER_LEAVE,
    psoDest: *mut SURFOBJ,
    DestRect: *mut RECTL,
    ReadOnly: BOOL,
    Translate: *mut POINTL,
    ppsoOutput: *mut *mut SURFOBJ,
) -> BOOL {
    let mut Exchange: i32;
    let mut BitmapSize: SIZEL = core::mem::zeroed();
    let mut SrcPoint: POINTL = core::mem::zeroed();
    let mut Width: i32;
    let mut ClippedDestRect: RECTL = core::mem::zeroed();

    if (*DestRect).right < (*DestRect).left {
        Exchange = (*DestRect).left;
        (*DestRect).left = (*DestRect).right;
        (*DestRect).right = Exchange;
    }
    if (*DestRect).bottom < (*DestRect).top {
        Exchange = (*DestRect).top;
        (*DestRect).top = (*DestRect).bottom;
        (*DestRect).bottom = Exchange;
    }

    if !psoDest.is_null() && (*psoDest).iType != STYPE_BITMAP as u16 &&
        ((*psoDest).pvScan0.is_null() || (*psoDest).lDelta == 0) {
        let psurfDest = (psoDest as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
        if ((*psurfDest).flags & HOOK_COPYBITS as u32) == 0 {
            return 0;
        }

        BitmapSize.cx = (*DestRect).right - (*DestRect).left;
        BitmapSize.cy = (*DestRect).bottom - (*DestRect).top;
        Width = WIDTH_BYTES_ALIGN32(BitmapSize.cx, BitsPerFormat((*psoDest).iBitmapFormat));
        (*EnterLeave).OutputBitmap = EngCreateBitmap(BitmapSize, Width,
                                                     (*psoDest).iBitmapFormat,
                                                     BMF_TOPDOWN | BMF_NOZEROINIT,
                                                     core::ptr::null_mut());

        if (*EnterLeave).OutputBitmap.is_null() {
            return 0;
        }

        *ppsoOutput = EngLockSurface((*EnterLeave).OutputBitmap as HSURF);
        if (*ppsoOutput).is_null() {
            EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
            return 0;
        }

        (*EnterLeave).DestRect.left = 0;
        (*EnterLeave).DestRect.top = 0;
        (*EnterLeave).DestRect.right = BitmapSize.cx;
        (*EnterLeave).DestRect.bottom = BitmapSize.cy;
        SrcPoint.x = (*DestRect).left;
        SrcPoint.y = (*DestRect).top;
        ClippedDestRect = (*EnterLeave).DestRect;
        if SrcPoint.x < 0 {
            ClippedDestRect.left -= SrcPoint.x;
            SrcPoint.x = 0;
        }
        if (*psoDest).sizlBitmap.cx < SrcPoint.x + ClippedDestRect.right - ClippedDestRect.left {
            ClippedDestRect.right = ClippedDestRect.left + (*psoDest).sizlBitmap.cx - SrcPoint.x;
        }
        if SrcPoint.y < 0 {
            ClippedDestRect.top -= SrcPoint.y;
            SrcPoint.y = 0;
        }
        if (*psoDest).sizlBitmap.cy < SrcPoint.y + ClippedDestRect.bottom - ClippedDestRect.top {
            ClippedDestRect.bottom = ClippedDestRect.top + (*psoDest).sizlBitmap.cy - SrcPoint.y;
        }
        (*EnterLeave).TrivialClipObj = EngCreateClip();
        if (*EnterLeave).TrivialClipObj.is_null() {
            EngUnlockSurface(*ppsoOutput);
            EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
            return 0;
        }
        (*(*EnterLeave).TrivialClipObj).iDComplexity = DC_TRIVIAL as u8;
        if ClippedDestRect.left < (**ppsoOutput).sizlBitmap.cx &&
            0 <= ClippedDestRect.right &&
            SrcPoint.x < (*psoDest).sizlBitmap.cx &&
            ClippedDestRect.top <= (**ppsoOutput).sizlBitmap.cy &&
            0 <= ClippedDestRect.bottom &&
            SrcPoint.y < (*psoDest).sizlBitmap.cy {
            let funcs = GDIDEVFUNCS(psoDest);
            if let Some(copy_bits) = (*funcs).CopyBits {
                if copy_bits(*ppsoOutput, psoDest, (*EnterLeave).TrivialClipObj,
                             core::ptr::null_mut(), &mut ClippedDestRect, &mut SrcPoint) == 0 {
                    EngDeleteClip((*EnterLeave).TrivialClipObj);
                    EngUnlockSurface(*ppsoOutput);
                    EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
                    return 0;
                }
            }
        }
        (*EnterLeave).DestRect.left = (*DestRect).left;
        (*EnterLeave).DestRect.top = (*DestRect).top;
        (*EnterLeave).DestRect.right = (*DestRect).right;
        (*EnterLeave).DestRect.bottom = (*DestRect).bottom;
        (*Translate).x = -(*DestRect).left;
        (*Translate).y = -(*DestRect).top;
    } else {
        (*Translate).x = 0;
        (*Translate).y = 0;
        *ppsoOutput = psoDest;
    }

    if !(*ppsoOutput).is_null() {
        let psurfOutput = (*ppsoOutput as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;
        if ((*psurfOutput).flags & HOOK_SYNCHRONIZE as u32) != 0 {
            let funcs = GDIDEVFUNCS(*ppsoOutput);
            if let Some(sync_surface) = (*funcs).SynchronizeSurface {
                sync_surface(*ppsoOutput, DestRect, 0);
            } else if (**ppsoOutput).iType == STYPE_BITMAP as u16 {
                if let Some(synchronize) = (*funcs).Synchronize {
                    synchronize((**ppsoOutput).dhpdev, DestRect);
                }
            }
        }
    } else {
        return 0;
    }

    (*EnterLeave).DestObj = psoDest;
    (*EnterLeave).OutputObj = *ppsoOutput;
    (*EnterLeave).ReadOnly = ReadOnly;

    1
}
#[no_mangle]
pub unsafe extern "system" fn IntEngLeave(EnterLeave: PINTENG_ENTER_LEAVE) -> BOOL {
    let mut SrcPoint: POINTL = core::mem::zeroed();
    let mut Result: BOOL = 1;

    if (*EnterLeave).OutputObj != (*EnterLeave).DestObj && !(*EnterLeave).OutputObj.is_null() {
        if (*EnterLeave).ReadOnly == 0 {
            SrcPoint.x = 0;
            SrcPoint.y = 0;
            if (*EnterLeave).DestRect.left < 0 {
                SrcPoint.x = -(*EnterLeave).DestRect.left;
                (*EnterLeave).DestRect.left = 0;
            }
            if (*(*EnterLeave).DestObj).sizlBitmap.cx < (*EnterLeave).DestRect.right {
                (*EnterLeave).DestRect.right = (*(*EnterLeave).DestObj).sizlBitmap.cx;
            }
            if (*EnterLeave).DestRect.top < 0 {
                SrcPoint.y = -(*EnterLeave).DestRect.top;
                (*EnterLeave).DestRect.top = 0;
            }
            if (*(*EnterLeave).DestObj).sizlBitmap.cy < (*EnterLeave).DestRect.bottom {
                (*EnterLeave).DestRect.bottom = (*(*EnterLeave).DestObj).sizlBitmap.cy;
            }
            if SrcPoint.x < (*(*EnterLeave).OutputObj).sizlBitmap.cx &&
               (*EnterLeave).DestRect.left <= (*EnterLeave).DestRect.right &&
               (*EnterLeave).DestRect.left < (*(*EnterLeave).DestObj).sizlBitmap.cx &&
               SrcPoint.y < (*(*EnterLeave).OutputObj).sizlBitmap.cy &&
               (*EnterLeave).DestRect.top <= (*EnterLeave).DestRect.bottom &&
               (*EnterLeave).DestRect.top < (*(*EnterLeave).DestObj).sizlBitmap.cy {
                let funcs = GDIDEVFUNCS((*EnterLeave).DestObj);
                Result = if let Some(copy_bits) = (*funcs).CopyBits {
                    copy_bits(
                        (*EnterLeave).DestObj,
                        (*EnterLeave).OutputObj,
                        (*EnterLeave).TrivialClipObj,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!((*EnterLeave).DestRect),
                        core::ptr::addr_of_mut!(SrcPoint),
                    )
                } else {
                    0
                };
            } else {
                Result = 1;
            }
        }
        EngUnlockSurface((*EnterLeave).OutputObj);
        EngDeleteSurface((*EnterLeave).OutputBitmap as HSURF);
        EngDeleteClip((*EnterLeave).TrivialClipObj);
    } else {
        Result = 1;
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn EngGetProcessHandle() -> HANDLE {
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn EngGetCurrentCodePage(
    OemCodePage: PUSHORT,
    AnsiCodePage: PUSHORT,
) {
    RtlGetDefaultCodePage(AnsiCodePage, OemCodePage);
}
#[no_mangle]
pub unsafe extern "system" fn EngQuerySystemAttribute(
    CapNum: ENG_SYSTEM_ATTRIBUTE,
    pCapability: PDWORD,
) -> BOOL {
    let mut sbi: SYSTEM_BASIC_INFORMATION = core::mem::zeroed();
    let mut spi: SYSTEM_PROCESSOR_INFORMATION = core::mem::zeroed();
    let status: NTSTATUS;

    match CapNum {
        _ENG_SYSTEM_ATTRIBUTE_EngNumberOfProcessors => {
            status = NtQuerySystemInformation(
                SystemBasicInformation,
                &mut sbi as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as u32,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(status) {
                return 0;
            }

            *pCapability = sbi.NumberOfProcessors as u32;
            return 1;
        }

        _ENG_SYSTEM_ATTRIBUTE_EngProcessorFeature => {
            status = NtQuerySystemInformation(
                SystemProcessorInformation,
                &mut spi as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<SYSTEM_PROCESSOR_INFORMATION>() as u32,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(status) {
                return 0;
            }
            *pCapability = spi.ProcessorFeatureBits;
            return 1;
        }

        _ => {}
    }

    0
}
// `KeQueryTickCount(t)` is a C macro (sdk/include/xdk/x86/ke.h) that
// reads `KeTickCount` (a `KSYSTEM_TIME` exported by the kernel) and
// composes the result into a `LARGE_INTEGER`. The non-bindgen-friendly
// macro form means we inline it here; identical to the canonical
// loop-read in SharedUserData->TickCount (kernel-side these alias).
#[inline(always)]
unsafe fn read_tick_count() -> u64 {
    // SharedUserData->TickCount.{High1Time, LowPart, High2Time} —
    // the C macro on x86 reads KeTickCount with a retry loop until
    // High1Time == High2Time; SharedUserData mirrors it so we do
    // the equivalent read here. `TickCount` lives in an anonymous
    // union (`__bindgen_anon_2`) on the bindgen output.
    let tc = core::ptr::addr_of!((*shared_user_data()).__bindgen_anon_2.TickCount);
    loop {
        let high1 = core::ptr::read_volatile(core::ptr::addr_of!((*tc).High1Time));
        let low   = core::ptr::read_volatile(core::ptr::addr_of!((*tc).LowPart));
        let high2 = core::ptr::read_volatile(core::ptr::addr_of!((*tc).High2Time));
        if high1 == high2 {
            return ((high1 as u64) << 32) | (low as u64);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EngGetTickCount() -> u64 {
    let tick_qword = read_tick_count();
    let low: u32 = tick_qword as u32;
    let high: i32 = (tick_qword >> 32) as i32;
    let Multiplier: u32 = (*shared_user_data()).TickCountMultiplier;

    Int64ShrlMod32(UInt32x32To64(Multiplier, low), 24)
        + (Multiplier as u64).wrapping_mul((high as u32 as u64) << 8)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS Win32k subsystem
//  * PURPOSE:           ENG misc Functions
//  * FILE:              win32ss/gdi/eng/engmisc.c
//  * PROGRAMER:         ReactOS Team
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// BOOL APIENTRY
// IntEngEnter(PINTENG_ENTER_LEAVE EnterLeave,
//             SURFOBJ *psoDest,
//             RECTL *DestRect,
//             BOOL ReadOnly,
//             POINTL *Translate,
//             SURFOBJ **ppsoOutput)
// {
//   LONG Exchange;
//   SIZEL BitmapSize;
//   POINTL SrcPoint;
//   LONG Width;
//   RECTL ClippedDestRect;
//
//   /* Normalize */
//   if (DestRect->right < DestRect->left)
//     {
//     Exchange = DestRect->left;
//     DestRect->left = DestRect->right;
//     DestRect->right = Exchange;
//     }
//   if (DestRect->bottom < DestRect->top)
//     {
//     Exchange = DestRect->top;
//     DestRect->top = DestRect->bottom;
//     DestRect->bottom = Exchange;
//     }
//
//   if (NULL != psoDest && STYPE_BITMAP != psoDest->iType &&
//       (NULL == psoDest->pvScan0 || 0 == psoDest->lDelta))
//     {
//     /* Driver needs to support DrvCopyBits, else we can't do anything */
//     SURFACE *psurfDest = CONTAINING_RECORD(psoDest, SURFACE, SurfObj);
//     if (!(psurfDest->flags & HOOK_COPYBITS))
//     {
//       return FALSE;
//     }
//
//     /* Allocate a temporary bitmap */
//     BitmapSize.cx = DestRect->right - DestRect->left;
//     BitmapSize.cy = DestRect->bottom - DestRect->top;
//     Width = WIDTH_BYTES_ALIGN32(BitmapSize.cx, BitsPerFormat(psoDest->iBitmapFormat));
//     EnterLeave->OutputBitmap = EngCreateBitmap(BitmapSize, Width,
//                                                psoDest->iBitmapFormat,
//                                                BMF_TOPDOWN | BMF_NOZEROINIT, NULL);
//
//     if (!EnterLeave->OutputBitmap)
//       {
//       DPRINT1("EngCreateBitmap() failed\n");
//       return FALSE;
//       }
//
//     *ppsoOutput = EngLockSurface((HSURF)EnterLeave->OutputBitmap);
//     if (*ppsoOutput == NULL)
//     {
//       EngDeleteSurface((HSURF)EnterLeave->OutputBitmap);
//       return FALSE;
//     }
//
//     EnterLeave->DestRect.left = 0;
//     EnterLeave->DestRect.top = 0;
//     EnterLeave->DestRect.right = BitmapSize.cx;
//     EnterLeave->DestRect.bottom = BitmapSize.cy;
//     SrcPoint.x = DestRect->left;
//     SrcPoint.y = DestRect->top;
//     ClippedDestRect = EnterLeave->DestRect;
//     if (SrcPoint.x < 0)
//       {
//         ClippedDestRect.left -= SrcPoint.x;
//         SrcPoint.x = 0;
//       }
//     if (psoDest->sizlBitmap.cx < SrcPoint.x + ClippedDestRect.right - ClippedDestRect.left)
//       {
//         ClippedDestRect.right = ClippedDestRect.left + psoDest->sizlBitmap.cx - SrcPoint.x;
//       }
//     if (SrcPoint.y < 0)
//       {
//         ClippedDestRect.top -= SrcPoint.y;
//         SrcPoint.y = 0;
//       }
//     if (psoDest->sizlBitmap.cy < SrcPoint.y + ClippedDestRect.bottom - ClippedDestRect.top)
//       {
//         ClippedDestRect.bottom = ClippedDestRect.top + psoDest->sizlBitmap.cy - SrcPoint.y;
//       }
//     EnterLeave->TrivialClipObj = EngCreateClip();
//     if (EnterLeave->TrivialClipObj == NULL)
//     {
//       EngUnlockSurface(*ppsoOutput);
//       EngDeleteSurface((HSURF)EnterLeave->OutputBitmap);
//       return FALSE;
//     }
//     EnterLeave->TrivialClipObj->iDComplexity = DC_TRIVIAL;
//     if (ClippedDestRect.left < (*ppsoOutput)->sizlBitmap.cx &&
//         0 <= ClippedDestRect.right &&
//         SrcPoint.x < psoDest->sizlBitmap.cx &&
//         ClippedDestRect.top <= (*ppsoOutput)->sizlBitmap.cy &&
//         0 <= ClippedDestRect.bottom &&
//         SrcPoint.y < psoDest->sizlBitmap.cy &&
//         ! GDIDEVFUNCS(psoDest).CopyBits(
//                                         *ppsoOutput, psoDest,
//                                         EnterLeave->TrivialClipObj, NULL,
//                                         &ClippedDestRect, &SrcPoint))
//       {
//           EngDeleteClip(EnterLeave->TrivialClipObj);
//           EngUnlockSurface(*ppsoOutput);
//           EngDeleteSurface((HSURF)EnterLeave->OutputBitmap);
//           return FALSE;
//       }
//     EnterLeave->DestRect.left = DestRect->left;
//     EnterLeave->DestRect.top = DestRect->top;
//     EnterLeave->DestRect.right = DestRect->right;
//     EnterLeave->DestRect.bottom = DestRect->bottom;
//     Translate->x = - DestRect->left;
//     Translate->y = - DestRect->top;
//     }
//   else
//     {
//     Translate->x = 0;
//     Translate->y = 0;
//     *ppsoOutput = psoDest;
//     }
//
//   if (NULL != *ppsoOutput)
//   {
//     SURFACE* psurfOutput = CONTAINING_RECORD(*ppsoOutput, SURFACE, SurfObj);
//     if (0 != (psurfOutput->flags & HOOK_SYNCHRONIZE))
//     {
//       if (NULL != GDIDEVFUNCS(*ppsoOutput).SynchronizeSurface)
//         {
//           GDIDEVFUNCS(*ppsoOutput).SynchronizeSurface(*ppsoOutput, DestRect, 0);
//         }
//       else if (STYPE_BITMAP == (*ppsoOutput)->iType
//                && NULL != GDIDEVFUNCS(*ppsoOutput).Synchronize)
//         {
//           GDIDEVFUNCS(*ppsoOutput).Synchronize((*ppsoOutput)->dhpdev, DestRect);
//         }
//     }
//   }
//   else return FALSE;
//
//   EnterLeave->DestObj = psoDest;
//   EnterLeave->OutputObj = *ppsoOutput;
//   EnterLeave->ReadOnly = ReadOnly;
//
//   return TRUE;
// }
//
// BOOL APIENTRY
// IntEngLeave(PINTENG_ENTER_LEAVE EnterLeave)
// {
//   POINTL SrcPoint;
//   BOOL Result = TRUE;
//
//   if (EnterLeave->OutputObj != EnterLeave->DestObj && NULL != EnterLeave->OutputObj)
//     {
//     if (! EnterLeave->ReadOnly)
//       {
//       SrcPoint.x = 0;
//       SrcPoint.y = 0;
//       if (EnterLeave->DestRect.left < 0)
//         {
//           SrcPoint.x = - EnterLeave->DestRect.left;
//           EnterLeave->DestRect.left = 0;
//         }
//       if (EnterLeave->DestObj->sizlBitmap.cx < EnterLeave->DestRect.right)
//         {
//           EnterLeave->DestRect.right = EnterLeave->DestObj->sizlBitmap.cx;
//         }
//       if (EnterLeave->DestRect.top < 0)
//         {
//           SrcPoint.y = - EnterLeave->DestRect.top;
//           EnterLeave->DestRect.top = 0;
//         }
//       if (EnterLeave->DestObj->sizlBitmap.cy < EnterLeave->DestRect.bottom)
//         {
//           EnterLeave->DestRect.bottom = EnterLeave->DestObj->sizlBitmap.cy;
//         }
//       if (SrcPoint.x < EnterLeave->OutputObj->sizlBitmap.cx &&
//           EnterLeave->DestRect.left <= EnterLeave->DestRect.right &&
//           EnterLeave->DestRect.left < EnterLeave->DestObj->sizlBitmap.cx &&
//           SrcPoint.y < EnterLeave->OutputObj->sizlBitmap.cy &&
//           EnterLeave->DestRect.top <= EnterLeave->DestRect.bottom &&
//           EnterLeave->DestRect.top < EnterLeave->DestObj->sizlBitmap.cy)
//         {
//           Result = GDIDEVFUNCS(EnterLeave->DestObj).CopyBits(
//                                                  EnterLeave->DestObj,
//                                                  EnterLeave->OutputObj,
//                                                  EnterLeave->TrivialClipObj, NULL,
//                                                  &EnterLeave->DestRect, &SrcPoint);
//         }
//       else
//         {
//           Result = TRUE;
//         }
//       }
//     EngUnlockSurface(EnterLeave->OutputObj);
//     EngDeleteSurface((HSURF)EnterLeave->OutputBitmap);
//     EngDeleteClip(EnterLeave->TrivialClipObj);
//     }
//   else
//     {
//     Result = TRUE;
//     }
//
//   return Result;
// }
//
// HANDLE APIENTRY
// EngGetProcessHandle(VOID)
// {
//   /* http://www.osr.com/ddk/graphics/gdifncs_3tif.htm
//      In Windows 2000 and later, the EngGetProcessHandle function always returns NULL.
//      FIXME: What does NT4 return? */
//   return NULL;
// }
//
// VOID
// APIENTRY
// EngGetCurrentCodePage(
//     _Out_ PUSHORT OemCodePage,
//     _Out_ PUSHORT AnsiCodePage)
// {
//     /* Forward to kernel */
//     RtlGetDefaultCodePage(AnsiCodePage, OemCodePage);
// }
//
// BOOL
// APIENTRY
// EngQuerySystemAttribute(
//    _In_ ENG_SYSTEM_ATTRIBUTE CapNum,
//    _Out_ PDWORD pCapability)
// {
//     SYSTEM_BASIC_INFORMATION sbi;
//     SYSTEM_PROCESSOR_INFORMATION spi;
//     NTSTATUS status;
//
//     switch (CapNum)
//     {
//         case EngNumberOfProcessors:
//             status = NtQuerySystemInformation(SystemBasicInformation,
//                                               &sbi,
//                                               sizeof(SYSTEM_BASIC_INFORMATION),
//                                               NULL);
//             if (!NT_SUCCESS(status))
//             {
//                 DPRINT1("Failed to query basic information: 0x%lx\n", status);
//                 return FALSE;
//             }
//
//             *pCapability = sbi.NumberOfProcessors;
//             return TRUE;
//
//         case EngProcessorFeature:
//             status = NtQuerySystemInformation(SystemProcessorInformation,
//                                               &spi,
//                                               sizeof(SYSTEM_PROCESSOR_INFORMATION),
//                                               NULL);
//             if (!NT_SUCCESS(status))
//             {
//                 DPRINT1("Failed to query processor information: 0x%lx\n", status);
//                 return FALSE;
//             }
//             *pCapability = spi.ProcessorFeatureBits;
//             return TRUE;
//
//         default:
//             break;
//     }
//
//     return FALSE;
// }
//
// ULONGLONG
// APIENTRY
// EngGetTickCount(VOID)
// {
//     ULONG Multiplier;
//     LARGE_INTEGER TickCount;
//
//     /* Get the multiplier and current tick count */
//     KeQueryTickCount(&TickCount);
//     Multiplier = SharedUserData->TickCountMultiplier;
//
//     /* Convert to milliseconds and return */
//     return (Int64ShrlMod32(UInt32x32To64(Multiplier, TickCount.LowPart), 24) +
//             (Multiplier * (TickCount.HighPart << 8)));
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
