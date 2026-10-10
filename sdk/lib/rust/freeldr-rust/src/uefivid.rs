#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_UI; // uefivid.c: DBG_DEFAULT_CHANNEL(UI)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use ntkrnl_sys::ddk::PCSTR;
// freeldr_ddk_imports dropped (not in ddk): Depth, Init, Red
use core::ffi::c_void;

// machine.h — enum tagVIDEODISPLAYMODE (file-local: ui_keystone.rs owns the
// crate-shared copy, but uefivid only globs crate::frldr — keep them here).
pub type VIDEODISPLAYMODE = u32;
pub const VideoTextMode: VIDEODISPLAYMODE = 0;
pub const VideoGraphicsMode: VIDEODISPLAYMODE = 1;
pub type PVIDEODISPLAYMODE = *mut VIDEODISPLAYMODE;

extern "C" {
}

extern "C" {
    fn FbConsClearScreen(Attr: u8);
    fn FbConsPutChar(Ch: i32, Attr: u8, X: u32, Y: u32);
    fn FbConsGetDisplaySize(Width: PULONG, Height: PULONG, Depth: PULONG);
    fn FbConsCopyOffScreenBufferToVRAM(Buffer: PVOID);
    // vidfb.c exports `_FbConsGetBufferSize` cdecl (no `@0`); declaring
    // it `extern "system"` made the i386 link fail with undefined
    // `_FbConsGetBufferSize@0`. The vidfb.c FbCons* family is cdecl.
    fn FbConsGetBufferSize() -> u32;
}

// vidfb.h:41 `FORCEINLINE ULONG PixelBitmasksToBpp(R,G,B,Reserved)` —
// pure (popcount of OR'd masks), no C symbol (inlined into vidfb.c).
// not(uefi): vidfb_pc_fallback.rs owns the `#[no_mangle]` symbol →
// extern-consume here. uefi: vidfb_pc_fallback is gated out and the
// FORCEINLINE has no exported symbol, so uefivid.rs (the uefi-side
// consumer) is the per-binary owner — define it. UNIQUE under uefi.
#[cfg(not(feature = "uefi"))]
extern "C" {
    fn PixelBitmasksToBpp(RedMask: ULONG, GreenMask: ULONG, BlueMask: ULONG, ReservedMask: ULONG) -> ULONG;
}
#[cfg(feature = "uefi")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PixelBitmasksToBpp(
    RedMask: ULONG,
    GreenMask: ULONG,
    BlueMask: ULONG,
    ReservedMask: ULONG,
) -> ULONG {
    (RedMask | GreenMask | BlueMask | ReservedMask).count_ones() as ULONG
}

extern "C" {
    // cross-module freeldr fns (vidfb.c; FLDRAPI empty → cdecl).
    fn VidFbInitializeVideo(
        pFbData: *mut PCM_FRAMEBUF_DEVICE_DATA,
        BaseAddress: ULONG_PTR,
        BufferSize: ULONG,
        ScreenWidth: UINT32,
        ScreenHeight: UINT32,
        PixelsPerScanLine: UINT32,
        BitsPerPixel: UINT32,
        PixelMasks: PPIXEL_BITMASK,
    ) -> BOOLEAN;
    // cross-module data from uefildr.c.
    static mut GlobalSystemTable: *mut EFI_SYSTEM_TABLE;
}

const LOWEST_SUPPORTED_RES: UINT32 = 1;

// ntbasedef.h RTL_BITS_OF(x) == sizeof(x) * 8.
macro_rules! RTL_BITS_OF {
    ($t:ty) => { (core::mem::size_of::<$t>() * 8) as ULONG };
}

// uefivid.c file-scope globals (external linkage — this TU is the definer).
#[no_mangle]
pub static mut EfiGraphicsOutputProtocol: EFI_GUID = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;
#[no_mangle]
pub static mut VramAddress: ULONG_PTR = 0;
#[no_mangle]
pub static mut VramSize: ULONG = 0;
#[no_mangle]
pub static mut FrameBufferData: PCM_FRAMEBUF_DEVICE_DATA = core::ptr::null_mut();

// uefivid.c EfiPixelMasks[] — indexed by EFI_GRAPHICS_PIXEL_FORMAT.
static mut EfiPixelMasks: [EFI_PIXEL_BITMASK; 5] = [
    EFI_PIXEL_BITMASK { RedMask: 0x000000FF, GreenMask: 0x0000FF00, BlueMask: 0x00FF0000, ReservedMask: 0xFF000000 },
    EFI_PIXEL_BITMASK { RedMask: 0x00FF0000, GreenMask: 0x0000FF00, BlueMask: 0x000000FF, ReservedMask: 0xFF000000 },
    EFI_PIXEL_BITMASK { RedMask: 0, GreenMask: 0, BlueMask: 0, ReservedMask: 0 },
    EFI_PIXEL_BITMASK { RedMask: 0, GreenMask: 0, BlueMask: 0, ReservedMask: 0 },
    EFI_PIXEL_BITMASK { RedMask: 0, GreenMask: 0, BlueMask: 0, ReservedMask: 0 },
];

#[no_mangle]
pub unsafe extern "C" fn UefiInitializeVideo() -> EFI_STATUS {
    let mut Status: EFI_STATUS;
    let mut gop: *mut EFI_GRAPHICS_OUTPUT_PROTOCOL = core::ptr::null_mut();

    let PixelFormat: EFI_GRAPHICS_PIXEL_FORMAT;
    let pPixelBitmask: *mut EFI_PIXEL_BITMASK;
    let BitsPerPixel: ULONG;

    Status = ((*(*GlobalSystemTable).BootServices).LocateProtocol).unwrap()(
        &EfiGraphicsOutputProtocol as *const _ as *mut _,
        core::ptr::null_mut(),
        &mut gop as *mut _ as *mut *mut core::ffi::c_void,
    );
    if Status != EFI_SUCCESS {
        TRACE!("Failed to find GOP with status %d\n", (Status) as usize);
        return Status;
    }

    ((*gop).SetMode).unwrap()(gop, LOWEST_SUPPORTED_RES);

    PixelFormat = (*(*(*gop).Mode).Info).PixelFormat;
    match PixelFormat {
        PixelRedGreenBlueReserved8BitPerColor | PixelBlueGreenRedReserved8BitPerColor => {
            pPixelBitmask = core::ptr::addr_of_mut!(EfiPixelMasks[PixelFormat as usize]);
            BitsPerPixel = RTL_BITS_OF!(EFI_GRAPHICS_OUTPUT_BLT_PIXEL);
        }

        PixelBitMask => {
            pPixelBitmask = &mut (*(*(*gop).Mode).Info).PixelInformation;
            BitsPerPixel = PixelBitmasksToBpp(
                (*pPixelBitmask).RedMask,
                (*pPixelBitmask).GreenMask,
                (*pPixelBitmask).BlueMask,
                (*pPixelBitmask).ReservedMask,
            );
        }

        PixelBltOnly | _ => {
            ERR!("Unsupported UEFI GOP format %lu\n", (PixelFormat) as usize);
            pPixelBitmask = core::ptr::null_mut();
            BitsPerPixel = 0;
        }
    }

    VramAddress = (*(*gop).Mode).FrameBufferBase as ULONG_PTR;
    VramSize = (*(*gop).Mode).FrameBufferSize as ULONG;
    if VidFbInitializeVideo(
        &mut FrameBufferData,
        VramAddress,
        VramSize,
        (*(*(*gop).Mode).Info).HorizontalResolution,
        (*(*(*gop).Mode).Info).VerticalResolution,
        (*(*(*gop).Mode).Info).PixelsPerScanLine,
        BitsPerPixel,
        pPixelBitmask as PPIXEL_BITMASK,
    ) == 0
    {
        ERR!("Couldn't initialize video framebuffer\n");
        Status = EFI_UNSUPPORTED;
    }
    Status
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoClearScreen(Attr: u8) {
    FbConsClearScreen(Attr);
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoPutChar(Ch: i32, Attr: u8, X: u32, Y: u32) {
    FbConsPutChar(Ch, Attr, X, Y);
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoGetDisplaySize(Width: PULONG, Height: PULONG, Depth: PULONG) {
    FbConsGetDisplaySize(Width, Height, Depth);
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoSetDisplayMode(
    _DisplayMode: PCSTR,
    _Init: BOOLEAN,
) -> VIDEODISPLAYMODE {
    VideoTextMode
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoGetBufferSize() -> u32 {
    FbConsGetBufferSize()
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoCopyOffScreenBufferToVRAM(Buffer: PVOID) {
    FbConsCopyOffScreenBufferToVRAM(Buffer);
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoSetTextCursorPosition(_X: UCHAR, _Y: UCHAR) {
    /* We don't have a cursor yet */
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoHideShowTextCursor(_Show: BOOLEAN) {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoIsPaletteFixed() -> u8 {
    0u8
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoSetPaletteColor(_Color: UCHAR, _Red: UCHAR, _Green: UCHAR, _Blue: UCHAR) {
}

#[cfg_attr(feature = "uefi", no_mangle)]
pub unsafe extern "C" fn UefiVideoGetPaletteColor(_Color: u8, _Red: *mut u8, _Green: *mut u8, _Blue: *mut u8) {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader UEFI Support
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Video output
//  * COPYRIGHT:   Copyright 2022 Justin Miller <justinmiller100@gmail.com>
//  */
//
// #include <uefildr.h>
// #include "../vidfb.h"
//
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(UI);
//
// /* GLOBALS ********************************************************************/
//
// extern EFI_SYSTEM_TABLE* GlobalSystemTable;
// extern EFI_HANDLE GlobalImageHandle;
// EFI_GUID EfiGraphicsOutputProtocol = EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID;
//
// ULONG_PTR VramAddress;
// ULONG VramSize;
// PCM_FRAMEBUF_DEVICE_DATA FrameBufferData = NULL;
//
// #define LOWEST_SUPPORTED_RES 1
//
// /* FUNCTIONS ******************************************************************/
//
// /* EFI 1.x */
// #ifdef EFI_UGA_DRAW_PROTOCOL_GUID
//
// /* NOTE: EFI UGA does not support any other format than 32-bit xRGB, and
//  * no direct access to the underlying hardware framebuffer is offered */
// C_ASSERT(sizeof(EFI_UGA_PIXEL) == sizeof(ULONG));
//
// #endif /* EFI */
//
// /* UEFI support, see efi/GraphicsOutput.h */
// #ifdef EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID // __GRAPHICS_OUTPUT_H__
//
// C_ASSERT(sizeof(EFI_GRAPHICS_OUTPUT_BLT_PIXEL) == sizeof(ULONG));
//
// /**
//  * @brief   Maps UEFI GOP pixel format to pixel masks.
//  * @see     EFI_PIXEL_BITMASK
//  **/
// static EFI_PIXEL_BITMASK EfiPixelMasks[] =
// { /* Red,        Green,      Blue,       Reserved */
//     {0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000},   // PixelRedGreenBlueReserved8BitPerColor
//     {0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000},   // PixelBlueGreenRedReserved8BitPerColor
//     {0,          0,          0,          0}             // PixelBitMask, PixelBltOnly, ...
// };
//
// #endif /* UEFI */
//
// EFI_STATUS
// UefiInitializeVideo(VOID)
// {
//     EFI_STATUS Status;
//     EFI_GRAPHICS_OUTPUT_PROTOCOL* gop = NULL;
//
//     EFI_GRAPHICS_PIXEL_FORMAT PixelFormat;
//     EFI_PIXEL_BITMASK* pPixelBitmask;
//     ULONG BitsPerPixel;
//
//     Status = GlobalSystemTable->BootServices->LocateProtocol(&EfiGraphicsOutputProtocol, 0, (void**)&gop);
//     if (Status != EFI_SUCCESS)
//     {
//         TRACE("Failed to find GOP with status %d\n", (Status) as usize);
//         return Status;
//     }
//
//     /* We don't need high resolutions for freeldr */
//     gop->SetMode(gop, LOWEST_SUPPORTED_RES);
//
//     /* Physical format of the pixel */
//     PixelFormat = gop->Mode->Info->PixelFormat;
//     switch (PixelFormat)
//     {
//         case PixelRedGreenBlueReserved8BitPerColor:
//         case PixelBlueGreenRedReserved8BitPerColor:
//         {
//             pPixelBitmask = &EfiPixelMasks[PixelFormat];
//             BitsPerPixel = RTL_BITS_OF(EFI_GRAPHICS_OUTPUT_BLT_PIXEL);
//             break;
//         }
//
//         case PixelBitMask:
//         {
//             /*
//              * When the GOP pixel format is given by PixelBitMask, the pixel
//              * element size _may be_ different from 4 bytes.
//              * See UEFI Spec Rev.2.10 Section 12.9 "Graphics Output Protocol":
//              * example code "GetPixelElementSize()" function.
//              */
//             pPixelBitmask = &gop->Mode->Info->PixelInformation;
//             BitsPerPixel =
//                 PixelBitmasksToBpp(pPixelBitmask->RedMask,
//                                    pPixelBitmask->GreenMask,
//                                    pPixelBitmask->BlueMask,
//                                    pPixelBitmask->ReservedMask);
//             break;
//         }
//
//         case PixelBltOnly:
//         default:
//         {
//             ERR("Unsupported UEFI GOP format %lu\n", (PixelFormat) as usize);
//             pPixelBitmask = NULL;
//             BitsPerPixel = 0;
//             break;
//         }
//     }
//
//     VramAddress = (ULONG_PTR)gop->Mode->FrameBufferBase;
//     VramSize = gop->Mode->FrameBufferSize;
//     if (!VidFbInitializeVideo(&FrameBufferData,
//                               VramAddress,
//                               VramSize,
//                               gop->Mode->Info->HorizontalResolution,
//                               gop->Mode->Info->VerticalResolution,
//                               gop->Mode->Info->PixelsPerScanLine,
//                               BitsPerPixel,
//                               (PPIXEL_BITMASK)pPixelBitmask))
//     {
//         ERR("Couldn't initialize video framebuffer\n");
//         Status = EFI_UNSUPPORTED;
//     }
//     return Status;
// }
//
// VOID
// UefiVideoClearScreen(UCHAR Attr)
// {
//     FbConsClearScreen(Attr);
// }
//
// VOID
// UefiVideoPutChar(int Ch, UCHAR Attr, unsigned X, unsigned Y)
// {
//     FbConsPutChar(Ch, Attr, X, Y);
// }
//
// VOID
// UefiVideoGetDisplaySize(PULONG Width, PULONG Height, PULONG Depth)
// {
//     FbConsGetDisplaySize(Width, Height, Depth);
// }
//
// VIDEODISPLAYMODE
// UefiVideoSetDisplayMode(PCSTR DisplayMode, BOOLEAN Init)
// {
//     /* We only have one mode, semi-text */
//     return VideoTextMode;
// }
//
// ULONG
// UefiVideoGetBufferSize(VOID)
// {
//     return FbConsGetBufferSize();
// }
//
// VOID
// UefiVideoCopyOffScreenBufferToVRAM(PVOID Buffer)
// {
//     FbConsCopyOffScreenBufferToVRAM(Buffer);
// }
//
// VOID
// UefiVideoSetTextCursorPosition(UCHAR X, UCHAR Y)
// {
//     /* We don't have a cursor yet */
// }
//
// VOID
// UefiVideoHideShowTextCursor(BOOLEAN Show)
// {
//     /* We don't have a cursor yet */
// }
//
// BOOLEAN
// UefiVideoIsPaletteFixed(VOID)
// {
//     return 0;
// }
//
// VOID
// UefiVideoSetPaletteColor(UCHAR Color, UCHAR Red,
//                          UCHAR Green, UCHAR Blue)
// {
//     /* Not supported */
// }
//
// VOID
// UefiVideoGetPaletteColor(UCHAR Color, UCHAR* Red,
//                          UCHAR* Green, UCHAR* Blue)
// {
//     /* Not supported */
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
