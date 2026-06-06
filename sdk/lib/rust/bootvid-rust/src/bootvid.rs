use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{MmAllocateContiguousMemorySpecifyCache, MmMapIoSpace, MmUnmapIoSpace, PHYSICAL_ADDRESS, INTERFACE_TYPE};
use core::ffi::c_void;
// precomp.h / display.h / framebuf.h substrate (RGBQUAD, BV_*, SCREEN_*,
// BOOTCHAR_*, GetFontPtr, InitializePalette, the Vidp* globals, CM_FRAMEBUF_*).
use crate::pc::*;
// console.c provides VidResetDisplay; framebuf.c (framebuf_impl) provides
// FindBootDisplay + BootTranslateBusAddress.
use crate::console::VidResetDisplay;
use crate::framebuf_impl::{FindBootDisplay, BootTranslateBusAddress};

// MEMORY_CACHING_TYPE::MmFrameBufferCached (xdk/mmtypes.h = 2).
const MmFrameBufferCached: ULONG = 2;

#[inline(always)]
fn min(a: u32, b: u32) -> u32 { if a < b { a } else { b } }

// ---- file-scope preamble (translated) ----
pub static mut FrameBufferStart: ULONG_PTR = 0;
pub static mut FrameBufferSize: ULONG = 0;
pub static mut ScreenWidth: ULONG = 0;
pub static mut ScreenHeight: ULONG = 0;
pub static mut BytesPerScanLine: ULONG = 0;
pub static mut BytesPerPixel: UCHAR = 0;
pub static mut BackBuffer: PUCHAR = core::ptr::null_mut();
pub static mut BackBufferSize: SIZE_T = 0;
pub static mut VidpXScale: USHORT = 1;
pub static mut VidpYScale: USHORT = 1;
pub static mut PanH: ULONG = 0;
pub static mut PanV: ULONG = 0;
pub static mut CachedPalette: [RGBQUAD; BV_MAX_COLORS] = [0 as RGBQUAD; BV_MAX_COLORS];

unsafe fn ApplyPalette() {
    let mut Frame: PULONG;
    let mut x: ULONG;
    let mut y: ULONG;

    /* Screen redraw */
    let mut Back: PUCHAR = BackBuffer;
    y = 0;
    while y < SCREEN_HEIGHT {
        Frame = (FrameBufferStart
            + (PanV + VidpYScale as ULONG * y) as ULONG_PTR * BytesPerScanLine as ULONG_PTR
            + (PanH + VidpXScale as ULONG * 0) as ULONG_PTR * BytesPerPixel as ULONG_PTR)
            as PULONG;
        let mut Pixel: PULONG = Frame;
        x = 0;
        while x < SCREEN_WIDTH {
            let mut j = VidpXScale as ULONG;
            while j > 0 {
                *Pixel = CachedPalette[*Back as usize];
                Pixel = Pixel.add(1);
                j -= 1;
            }
            Back = Back.add(1);
            x += 1;
        }
        Pixel = Frame;
        let mut i = VidpYScale as ULONG - 1;
        while i > 0 {
            Pixel = ((Pixel as ULONG_PTR) + BytesPerScanLine as ULONG_PTR) as PULONG;
            core::ptr::copy_nonoverlapping(
                Frame as *const u8,
                Pixel as *mut u8,
                (VidpXScale as ULONG * SCREEN_WIDTH * BytesPerPixel as ULONG) as usize,
            );
            i -= 1;
        }
        y += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn VidInitialize(SetMode: BOOLEAN) -> BOOLEAN {
    let mut FrameBuffer: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut VramAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut VramSize: ULONG = 0;
    let mut VideoConfigData: CM_FRAMEBUF_DEVICE_DATA = core::mem::zeroed();
    let mut Interface: INTERFACE_TYPE = core::mem::zeroed();
    let mut BusNumber: ULONG = 0;
    let Status: NTSTATUS;

    /* Find boot-time framebuffer display information from the LoaderBlock */
    Status = FindBootDisplay(
        &mut VramAddress,
        &mut VramSize,
        &mut VideoConfigData,
        core::ptr::null_mut(),
        &mut Interface,
        &mut BusNumber,
    );
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    /* Retrieve the framebuffer address, its visible screen dimensions, and its attributes */
    FrameBuffer.QuadPart = VramAddress.QuadPart + VideoConfigData.FrameBufferOffset as i64;
    ScreenWidth = VideoConfigData.ScreenWidth;
    ScreenHeight = VideoConfigData.ScreenHeight;
    if ScreenWidth < SCREEN_WIDTH || ScreenHeight < SCREEN_HEIGHT {
        return 0u8;
    }

    BytesPerPixel = ((VideoConfigData.BitsPerPixel + 7) / 8) as UCHAR;
    if BytesPerPixel != 4 {
        return 0u8;
    }

    BytesPerScanLine = VideoConfigData.PixelsPerScanLine * BytesPerPixel as ULONG;
    if BytesPerScanLine < 1 {
        return 0u8;
    }

    /* Compute the visible framebuffer size */
    FrameBufferSize = ScreenHeight * BytesPerScanLine;

    /* Verify that the framebuffer actually fits inside the video RAM */
    if FrameBuffer.QuadPart + FrameBufferSize as i64 > VramAddress.QuadPart + VramSize as i64 {
        return 0u8;
    }

    /* Translate the framebuffer from bus-relative to physical address */
    let mut TranslatedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut AddressSpace: ULONG = 0;
    if BootTranslateBusAddress(
        Interface,
        BusNumber,
        FrameBuffer,
        &mut AddressSpace,
        &mut TranslatedAddress,
    ) == 0
    {
        return 0u8;
    }

    /* Map it into system space if necessary */
    let mut MappedSize: ULONG = 0;
    let mut FrameBufferBase: PVOID = core::ptr::null_mut();
    if AddressSpace == 0 {
        FrameBuffer.__bindgen_anon_1.HighPart = TranslatedAddress.__bindgen_anon_1.HighPart;
        FrameBuffer.__bindgen_anon_1.LowPart =
            ALIGN_DOWN_BY(TranslatedAddress.__bindgen_anon_1.LowPart as usize, PAGE_SIZE as usize) as ULONG;
        MappedSize = FrameBufferSize;
        MappedSize += (TranslatedAddress.QuadPart - FrameBuffer.QuadPart) as ULONG;
        MappedSize = ROUND_TO_PAGES(MappedSize as usize) as ULONG;
        FrameBufferBase = MmMapIoSpace(FrameBuffer, MappedSize as _, MmFrameBufferCached);
        if FrameBufferBase.is_null() {
            FrameBufferBase = MmMapIoSpace(FrameBuffer, MappedSize as _, MmNonCached);
        }
        if FrameBufferBase.is_null() {
            if !FrameBufferBase.is_null() && AddressSpace == 0 {
                MmUnmapIoSpace(FrameBufferBase, MappedSize as _);
            }
            return 0u8;
        }
        FrameBufferStart = FrameBufferBase as ULONG_PTR;
        FrameBufferStart += (TranslatedAddress.QuadPart - FrameBuffer.QuadPart) as ULONG_PTR;
    } else {
        FrameBufferStart = TranslatedAddress.QuadPart as ULONG_PTR;
    }

    BackBufferSize = (SCREEN_WIDTH * (SCREEN_HEIGHT + (BOOTCHAR_HEIGHT + 1))) as SIZE_T;

    if (VideoConfigData.FrameBufferOffset + FrameBufferSize + BackBufferSize as ULONG)
        <= (if AddressSpace == 0 { MappedSize } else { VramSize })
    {
        BackBuffer = (FrameBufferStart + FrameBufferSize as ULONG_PTR) as PUCHAR;
    } else {
        let mut NullAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
        NullAddress.QuadPart = 0;
        let mut HighestAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
        HighestAddress.QuadPart = -1;
        BackBuffer = MmAllocateContiguousMemorySpecifyCache(
            BackBufferSize as _,
            NullAddress,
            HighestAddress,
            NullAddress,
            MmNonCached,
        ) as PUCHAR;
        if BackBuffer.is_null() {
            if !FrameBufferBase.is_null() && AddressSpace == 0 {
                MmUnmapIoSpace(FrameBufferBase, MappedSize as _);
            }
            return 0u8;
        }
    }

    /* Compute autoscaling; only integer (not fractional) scaling is supported */
    VidpXScale = (ScreenWidth / SCREEN_WIDTH) as USHORT;
    VidpYScale = (ScreenHeight / SCREEN_HEIGHT) as USHORT;
    VidpXScale = min(VidpXScale as u32, VidpYScale as u32) as USHORT;
    VidpYScale = VidpXScale;

    /* Calculate left/right and top/bottom border values */
    PanH = (ScreenWidth - VidpXScale as ULONG * SCREEN_WIDTH) / 2;
    PanV = (ScreenHeight - VidpYScale as ULONG * SCREEN_HEIGHT) / 2;

    /* Reset the video mode if requested */
    if SetMode != 0 {
        VidResetDisplay(1u8);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn VidCleanUp() {
    VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK as UCHAR);
}
#[no_mangle]
pub unsafe extern "C" fn ResetDisplay(SetMode: BOOLEAN) {
    core::ptr::write_bytes(BackBuffer as *mut u8, 0, BackBufferSize as usize);
    core::ptr::write_bytes(FrameBufferStart as *mut u8, 0, FrameBufferSize as usize);

    /* Re-initialize the palette and fill the screen black */
    InitializePalette();
    VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK as UCHAR);
}
#[no_mangle]
pub unsafe extern "C" fn InitPaletteWithTable(Table: *const ULONG, Count: ULONG) {
    let mut Entry: *const ULONG = Table;
    let mut i: ULONG;
    let mut HasChanged: BOOLEAN = 0u8;

    i = 0;
    while i < Count {
        if ((CachedPalette[i as usize] ^ *Entry) & 0x00FFFFFF) != 0 {
            HasChanged |= 1u8;
        }
        CachedPalette[i as usize] = *Entry | 0xFF000000;

        i += 1;
        Entry = Entry.add(1);
    }

    /* Re-apply the palette if it has changed */
    if HasChanged != 0 {
        ApplyPalette();
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetPixel(Left: ULONG, Top: ULONG, Color: UCHAR) {
    let Back: PUCHAR = (BackBuffer as PUCHAR).add((Top * SCREEN_WIDTH + Left) as usize);
    let mut Frame: PULONG = (FrameBufferStart as PUCHAR)
        .add(((PanV + VidpYScale as ULONG * Top) * BytesPerScanLine
            + (PanH + VidpXScale as ULONG * Left) * BytesPerPixel as ULONG) as usize)
        as PULONG;

    *Back = Color;
    let mut i = VidpYScale as ULONG;
    while i > 0 {
        let mut Pixel: PULONG = Frame;
        let mut j = VidpXScale as ULONG;
        while j > 0 {
            *Pixel = CachedPalette[Color as usize];
            Pixel = Pixel.add(1);
            j -= 1;
        }
        Frame = (Frame as ULONG_PTR + BytesPerScanLine as ULONG_PTR) as PULONG;
        i -= 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PreserveRow(CurrentTop: ULONG, TopDelta: ULONG, Restore: BOOLEAN) {
    let mut NewPosition: PUCHAR;
    let OldPosition: PUCHAR;

    if Restore != 0 {
        NewPosition = (BackBuffer as PUCHAR).add((CurrentTop as usize) * (SCREEN_WIDTH as usize));
        OldPosition = (BackBuffer as PUCHAR).add((SCREEN_HEIGHT as usize) * (SCREEN_WIDTH as usize));
    } else {
        NewPosition = (BackBuffer as PUCHAR).add((SCREEN_HEIGHT as usize) * (SCREEN_WIDTH as usize));
        OldPosition = (BackBuffer as PUCHAR).add((CurrentTop as usize) * (SCREEN_WIDTH as usize));
    }

    let mut Count: ULONG = TopDelta * SCREEN_WIDTH;
    core::ptr::copy_nonoverlapping(OldPosition as *const u8, NewPosition as *mut u8, Count as usize);

    if Restore != 0 {
        NewPosition = (BackBuffer as PUCHAR).add((CurrentTop as usize) * (SCREEN_WIDTH as usize));
        let mut y: ULONG = 0;
        while y < TopDelta {
            let Frame = (FrameBufferStart as *mut u8).add(
                ((PanV + (VidpYScale as ULONG) * (CurrentTop + y)) as usize) * (BytesPerScanLine as usize)
                    + ((PanH + (VidpXScale as ULONG) * 0) as usize) * (BytesPerPixel as usize),
            ) as PULONG;
            let mut Pixel: PULONG = Frame;
            Count = 0;
            while Count < SCREEN_WIDTH {
                let mut j: ULONG = VidpXScale as ULONG;
                while j > 0 {
                    *Pixel = CachedPalette[*NewPosition as usize];
                    Pixel = Pixel.add(1);
                    j -= 1;
                }
                NewPosition = NewPosition.add(1);
                Count += 1;
            }
            Pixel = Frame;
            let mut i: ULONG = (VidpYScale as ULONG).wrapping_sub(1);
            while i > 0 {
                Pixel = ((Pixel as ULONG_PTR) + (BytesPerScanLine as ULONG_PTR)) as PULONG;
                core::ptr::copy_nonoverlapping(
                    Frame as *const u8,
                    Pixel as *mut u8,
                    ((VidpXScale as ULONG) * SCREEN_WIDTH * (BytesPerPixel as ULONG)) as usize,
                );
                i -= 1;
            }
            y += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DoScroll(Scroll: ULONG) {
    let RowSize: ULONG = (VidpScrollRegion.Right - VidpScrollRegion.Left + 1) as ULONG;

    let mut OldPosition: PUCHAR = (BackBuffer as ULONG_PTR
        + ((VidpScrollRegion.Top + Scroll) as ULONG_PTR) * SCREEN_WIDTH as ULONG_PTR
        + VidpScrollRegion.Left as ULONG_PTR) as PUCHAR;
    let mut NewPosition: PUCHAR = (BackBuffer as ULONG_PTR
        + (VidpScrollRegion.Top as ULONG_PTR) * SCREEN_WIDTH as ULONG_PTR
        + VidpScrollRegion.Left as ULONG_PTR) as PUCHAR;

    let mut Top: ULONG = VidpScrollRegion.Top as ULONG;
    while Top <= VidpScrollRegion.Bottom as ULONG {
        core::ptr::copy_nonoverlapping(OldPosition as *const u8, NewPosition as *mut u8, RowSize as usize);

        let Frame: PULONG = (FrameBufferStart
            + (PanV + VidpYScale as ULONG * Top) as ULONG_PTR * BytesPerScanLine as ULONG_PTR
            + (PanH + VidpXScale as ULONG * VidpScrollRegion.Left as ULONG) as ULONG_PTR * BytesPerPixel as ULONG_PTR)
            as PULONG;
        let mut Pixel: PULONG = Frame;
        let mut Count: ULONG = 0;
        while Count < RowSize {
            let mut j: ULONG = VidpXScale as ULONG;
            while j > 0 {
                *Pixel = CachedPalette[*NewPosition.add(Count as usize) as usize];
                Pixel = Pixel.add(1);
                j -= 1;
            }
            Count += 1;
        }
        Pixel = Frame;
        let mut i: ULONG = VidpYScale as ULONG - 1;
        while i > 0 {
            Pixel = (Pixel as ULONG_PTR + BytesPerScanLine as ULONG_PTR) as PULONG;
            core::ptr::copy_nonoverlapping(
                Frame as *const u8,
                Pixel as *mut u8,
                (VidpXScale as ULONG * RowSize * BytesPerPixel as ULONG) as usize,
            );
            i -= 1;
        }

        OldPosition = OldPosition.add(SCREEN_WIDTH as usize);
        NewPosition = NewPosition.add(SCREEN_WIDTH as usize);
        Top += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn DisplayCharacter(
    Character: CHAR,
    Left: ULONG,
    Top: ULONG,
    TextColor: ULONG,
    BackColor: ULONG,
) {
    let mut FontChar: *const UCHAR = GetFontPtr(Character);

    let mut y: ULONG = Top;
    while y < Top + BOOTCHAR_HEIGHT {
        let mut x: ULONG = Left;
        let mut bit: UCHAR = 1 << (BOOTCHAR_WIDTH - 1);
        while bit > 0 {
            if *FontChar & bit != 0 {
                SetPixel(x, y, TextColor as UCHAR);
            } else if BackColor < BV_COLOR_NONE {
                SetPixel(x, y, BackColor as UCHAR);
            }
            bit >>= 1;
            x += 1;
        }
        y += 1;
        FontChar = FontChar.add(FONT_PTR_DELTA as usize);
    }
}

#[no_mangle]
pub unsafe extern "system" fn VidSolidColorFill(
    Left: ULONG,
    Top: ULONG,
    Right: ULONG,
    Bottom: ULONG,
    Color: UCHAR,
) {
    let mut Top = Top;
    while Top <= Bottom {
        let mut Back = (BackBuffer as ULONG_PTR + (Top as ULONG_PTR) * SCREEN_WIDTH as ULONG_PTR + Left as ULONG_PTR) as PUCHAR;
        // NOTE: Assumes 32bpp
        let Frame = (FrameBufferStart
            + ((PanV + VidpYScale as ULONG * Top) as ULONG_PTR) * BytesPerScanLine as ULONG_PTR
            + ((PanH + VidpXScale as ULONG * Left) as ULONG_PTR) * BytesPerPixel as ULONG_PTR)
            as PULONG;
        let mut Pixel = Frame;
        let mut L = Left;
        while L <= Right {
            *Back = Color;
            Back = Back.add(1);
            let mut j = VidpXScale;
            while j > 0 {
                *Pixel = CachedPalette[Color as usize];
                Pixel = Pixel.add(1);
                j -= 1;
            }
            L += 1;
        }
        Pixel = Frame;
        let mut i = VidpYScale - 1;
        while i > 0 {
            Pixel = ((Pixel as ULONG_PTR) + BytesPerScanLine as ULONG_PTR) as PULONG;
            core::ptr::copy_nonoverlapping(
                Frame as *const u8,
                Pixel as *mut u8,
                (VidpXScale as ULONG * (Right - Left + 1) * BytesPerPixel as ULONG) as usize,
            );
            i -= 1;
        }
        Top += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn VidScreenToBufferBlt(
    Buffer: PUCHAR,
    Left: ULONG,
    Top: ULONG,
    Width: ULONG,
    Height: ULONG,
    Delta: ULONG,
) {
    let mut x: ULONG;
    let mut y: ULONG;

    /* Clear the destination buffer */
    core::ptr::write_bytes(Buffer as *mut u8, 0, (Delta * Height) as usize);

    /* Start the outer Y height loop */
    y = 0;
    while y < Height {
        /* Set current scanline */
        let mut Back: PUCHAR =
            (BackBuffer).add(((Top + y) * SCREEN_WIDTH + Left) as usize);
        let mut Buf: PUCHAR = Buffer.add((y * Delta) as usize);

        /* Start the X inner loop */
        x = 0;
        while x < Width {
            /* Read the current value */
            *Buf = (*Back & 0xF) << 4;
            Back = Back.add(1);
            *Buf |= *Back & 0xF;
            Back = Back.add(1);
            Buf = Buf.add(1);

            x += core::mem::size_of::<USHORT>() as ULONG;
        }

        y += 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Generic Framebuffer Boot Video Driver
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  *              or MIT (https://spdx.org/licenses/MIT)
//  * PURPOSE:     Main file
//  * COPYRIGHT:   Copyright 2023-2026 Hermès Bélusca-Maïto <hermes.belusca-maito@reactos.org>
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* Include the Boot-time (POST) display discovery helper functions */
// #include <drivers/bootvid/framebuf.c>
//
// /* Scaling of the bootvid 640x480 default virtual screen to the larger video framebuffer */
// #define SCALING_SUPPORT
// #define SCALING_PROPORTIONAL
//
// /* Keep borders black or controlled with palette */
// // #define COLORED_BORDERS
//
//
// /* GLOBALS ********************************************************************/
//
// #define BB_PIXEL(x, y) \
//     ((PUCHAR)BackBuffer + (y) * SCREEN_WIDTH + (x))
//
// #define FB_PIXEL(x, y) \
//     ((PUCHAR)FrameBufferStart + (PanV + VidpYScale * (y)) * BytesPerScanLine \
//                               + (PanH + VidpXScale * (x)) * BytesPerPixel)
//
// static ULONG_PTR FrameBufferStart = 0;
// static ULONG FrameBufferSize;
// static ULONG ScreenWidth, ScreenHeight, BytesPerScanLine;
// static UCHAR BytesPerPixel;
// static PUCHAR BackBuffer = NULL;
// static SIZE_T BackBufferSize;
//
// #ifdef SCALING_SUPPORT
// static USHORT VidpXScale = 1;
// static USHORT VidpYScale = 1;
// #else
// #define VidpXScale 1
// #define VidpYScale 1
// #endif
// static ULONG PanH, PanV;
//
// static RGBQUAD CachedPalette[BV_MAX_COLORS];
//
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// static VOID
// ApplyPalette(VOID)
// {
//     PULONG Frame = (PULONG)FrameBufferStart;
//     ULONG x, y;
//
// #ifdef COLORED_BORDERS
//     /* Top border */
//     for (x = 0; x < PanV * ScreenWidth; ++x)
//     {
//         *Frame++ = CachedPalette[BV_COLOR_BLACK];
//     }
//
//     /* Left border */
//     for (y = 0; y < VidpYScale * SCREEN_HEIGHT; ++y)
//     {
//         // Frame = (PULONG)(FrameBufferStart + FB_OFFSET(-(LONG)PanH, y));
//         Frame = (PULONG)(FrameBufferStart + (PanV + y) * BytesPerScanLine);
//         for (x = 0; x < PanH; ++x)
//         {
//             *Frame++ = CachedPalette[BV_COLOR_BLACK];
//         }
//     }
// #endif // COLORED_BORDERS
//
//     /* Screen redraw */
//     PUCHAR Back = BackBuffer;
//     for (y = 0; y < SCREEN_HEIGHT; ++y)
//     {
//         Frame = (PULONG)FB_PIXEL(0, y);
//         PULONG Pixel = Frame;
//         for (x = 0; x < SCREEN_WIDTH; ++x)
//         {
//             for (ULONG j = VidpXScale; j > 0; --j)
//                 *Pixel++ = CachedPalette[*Back];
//             Back++;
//         }
//         Pixel = Frame;
//         for (ULONG i = VidpYScale-1; i > 0; --i)
//         {
//             Pixel = (PULONG)((ULONG_PTR)Pixel + BytesPerScanLine);
//             RtlCopyMemory(Pixel, Frame, VidpXScale * SCREEN_WIDTH * BytesPerPixel);
//         }
//     }
//
// #ifdef COLORED_BORDERS
//     /* Right border */
//     for (y = 0; y < VidpYScale * SCREEN_HEIGHT; ++y)
//     {
//         // Frame = (PULONG)(FrameBufferStart + FB_OFFSET(SCREEN_WIDTH, y));
//         Frame = (PULONG)(FrameBufferStart + (PanV + y) * BytesPerScanLine + (PanH + VidpXScale * SCREEN_WIDTH) * BytesPerPixel);
//         for (x = 0; x < PanH; ++x)
//         {
//             *Frame++ = CachedPalette[BV_COLOR_BLACK];
//         }
//     }
//
//     /* Bottom border */
//     // Frame = (PULONG)(FrameBufferStart + FB_OFFSET(-(LONG)PanH, SCREEN_HEIGHT));
//     Frame = (PULONG)(FrameBufferStart + (PanV + VidpYScale * SCREEN_HEIGHT) * BytesPerScanLine);
//     for (x = 0; x < PanV * ScreenWidth; ++x)
//     {
//         *Frame++ = CachedPalette[BV_COLOR_BLACK];
//     }
// #endif // COLORED_BORDERS
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// BOOLEAN
// NTAPI
// VidInitialize(
//      BOOLEAN SetMode)
// {
//     PHYSICAL_ADDRESS FrameBuffer;
//     PHYSICAL_ADDRESS VramAddress;
//     ULONG VramSize;
//     CM_FRAMEBUF_DEVICE_DATA VideoConfigData; /* Configuration data from hardware tree */
//     INTERFACE_TYPE Interface;
//     ULONG BusNumber;
//     NTSTATUS Status;
//
//     /* Find boot-time framebuffer display information from the LoaderBlock */
//     Status = FindBootDisplay(&VramAddress,
//                              &VramSize,
//                              &VideoConfigData,
//                              NULL, // MonitorConfigData
//                              &Interface,
//                              &BusNumber);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Boot framebuffer does not exist!\n");
//         return FALSE;
//     }
//
//     /* The VRAM address must be page-aligned */
//     if (VramAddress.QuadPart % PAGE_SIZE != 0) // DPRINTed for diagnostics on some systems
//         DPRINT1("** VramAddress 0x%I64X isn't PAGE_SIZE aligned\n", VramAddress.QuadPart);
//     ASSERT(VramAddress.QuadPart % PAGE_SIZE == 0);
//     if (VramSize % PAGE_SIZE != 0)
//         DPRINT1("** VramSize %lu (0x%lx) isn't multiple of PAGE_SIZE\n", VramSize, VramSize);
//     // ASSERT(VramSize % PAGE_SIZE == 0); // This assert may fail, e.g. 800x600@32bpp UEFI GOP display
//
//     /* Retrieve the framebuffer address, its visible screen dimensions, and its attributes */
//     FrameBuffer.QuadPart = VramAddress.QuadPart + VideoConfigData.FrameBufferOffset;
//     ScreenWidth  = VideoConfigData.ScreenWidth;
//     ScreenHeight = VideoConfigData.ScreenHeight;
//     if (ScreenWidth < SCREEN_WIDTH || ScreenHeight < SCREEN_HEIGHT)
//     {
//         DPRINT1("Unsupported screen resolution!\n");
//         return FALSE;
//     }
//
//     BytesPerPixel = (VideoConfigData.BitsPerPixel + 7) / 8; // Round up to nearest byte.
//     ASSERT(BytesPerPixel >= 1 && BytesPerPixel <= 4);
//     if (BytesPerPixel != 4)
//     {
//         UNIMPLEMENTED;
//         DPRINT1("Unsupported BytesPerPixel = %u\n", BytesPerPixel);
//         return FALSE;
//     }
//
//     ASSERT(ScreenWidth <= VideoConfigData.PixelsPerScanLine);
//     BytesPerScanLine = VideoConfigData.PixelsPerScanLine * BytesPerPixel;
//     if (BytesPerScanLine < 1)
//     {
//         DPRINT1("Invalid BytesPerScanLine = %lu\n", BytesPerScanLine);
//         return FALSE;
//     }
//
//     /* Compute the visible framebuffer size */
//     FrameBufferSize = ScreenHeight * BytesPerScanLine;
//
//     /* Verify that the framebuffer actually fits inside the video RAM */
//     if (FrameBuffer.QuadPart + FrameBufferSize > VramAddress.QuadPart + VramSize)
//     {
//         DPRINT1("The framebuffer exceeds video memory bounds!\n");
//         return FALSE;
//     }
//
//     /* Translate the framebuffer from bus-relative to physical address */
//     PHYSICAL_ADDRESS TranslatedAddress;
//     ULONG AddressSpace = 0; /* MMIO space */
//     if (!BootTranslateBusAddress(Interface,
//                                  BusNumber,
//                                  FrameBuffer,
//                                  &AddressSpace,
//                                  &TranslatedAddress))
//     {
//         DPRINT1("Could not translate framebuffer bus address 0x%I64X\n", FrameBuffer.QuadPart);
//         return FALSE;
//     }
//
//     /* Map it into system space if necessary */
//     ULONG MappedSize = 0;
//     PVOID FrameBufferBase = NULL;
//     if (AddressSpace == 0)
//     {
//         /* Calculate page-aligned address and size for MmMapIoSpace() */
//         FrameBuffer.HighPart = TranslatedAddress.HighPart;
//         FrameBuffer.LowPart  = ALIGN_DOWN_BY(TranslatedAddress.LowPart, PAGE_SIZE);
//         MappedSize = FrameBufferSize;
//         MappedSize += (ULONG)(TranslatedAddress.QuadPart - FrameBuffer.QuadPart); // BYTE_OFFSET()
//         MappedSize = ROUND_TO_PAGES(MappedSize);
//         /* Essentially MmMapVideoDisplay() */
//         FrameBufferBase = MmMapIoSpace(FrameBuffer, MappedSize, MmFrameBufferCached);
//         if (!FrameBufferBase)
//             FrameBufferBase = MmMapIoSpace(FrameBuffer, MappedSize, MmNonCached);
//         if (!FrameBufferBase)
//         {
//             DPRINT1("Could not map framebuffer 0x%I64X (%lu bytes)\n",
//                     FrameBuffer.QuadPart, MappedSize);
//             goto Failure;
//         }
//         FrameBufferStart = (ULONG_PTR)FrameBufferBase;
//         FrameBufferStart += (TranslatedAddress.QuadPart - FrameBuffer.QuadPart); // BYTE_OFFSET()
//     }
//     else
//     {
//         /* The base is the translated address, no need to map */
//         FrameBufferStart = (ULONG_PTR)TranslatedAddress.QuadPart;
//     }
//
//
//     /*
//      * Reserve off-screen area for the backbuffer that contains
//      * 8-bit indexed color screen image, plus preserved row data.
//      */
//     BackBufferSize = SCREEN_WIDTH * (SCREEN_HEIGHT + (BOOTCHAR_HEIGHT + 1));
//
//     /* If there is enough video memory in the physical framebuffer,
//      * place the backbuffer in the hidden part of the framebuffer,
//      * otherwise allocate a zone for the backbuffer. */
//     if (VideoConfigData.FrameBufferOffset + FrameBufferSize + BackBufferSize
//             <= ((AddressSpace == 0) ? MappedSize : VramSize))
//     {
//         /* Backbuffer placed following the framebuffer in the hidden part */
//         BackBuffer = (PUCHAR)(FrameBufferStart + FrameBufferSize);
//         // BackBuffer = (PUCHAR)(VramAddress + VramSize - BackBufferSize); // Or at the end of VRAM.
//     }
//     else
//     {
//         /* Allocate the backbuffer */
//         PHYSICAL_ADDRESS NullAddress = {{0, 0}};
//         PHYSICAL_ADDRESS HighestAddress = {{-1, -1}};
//         BackBuffer = MmAllocateContiguousMemorySpecifyCache(
//                         BackBufferSize, NullAddress, HighestAddress,
//                         NullAddress, MmNonCached);
//         if (!BackBuffer)
//         {
//             DPRINT1("Could not allocate backbuffer (size: %lu)\n", (ULONG)BackBufferSize);
//             goto Failure;
//         }
//     }
//
// #ifdef SCALING_SUPPORT
//     /* Compute autoscaling; only integer (not fractional) scaling is supported */
//     VidpXScale = ScreenWidth / SCREEN_WIDTH;
//     VidpYScale = ScreenHeight / SCREEN_HEIGHT;
//     ASSERT(VidpXScale >= 1);
//     ASSERT(VidpYScale >= 1);
// #ifdef SCALING_PROPORTIONAL
//     VidpXScale = min(VidpXScale, VidpYScale);
//     VidpYScale = VidpXScale;
// #endif
//     DPRINT1("Scaling X = %hu, Y = %hu\n", VidpXScale, VidpYScale);
// #endif // SCALING_SUPPORT
//
//     /* Calculate left/right and top/bottom border values
//      * to keep the displayed area centered on the screen */
//     PanH = (ScreenWidth - VidpXScale * SCREEN_WIDTH) / 2;
//     PanV = (ScreenHeight - VidpYScale * SCREEN_HEIGHT) / 2;
//     DPRINT1("Borders X = %lu, Y = %lu\n", PanH, PanV);
//
//     /* Reset the video mode if requested */
//     if (SetMode)
//         VidResetDisplay(TRUE);
//
//     return TRUE;
//
// Failure:
//     /* We failed somewhere; unmap the framebuffer if we mapped it */
//     if (FrameBufferBase && (AddressSpace == 0))
//         MmUnmapIoSpace(FrameBufferBase, MappedSize);
//
//     return FALSE;
// }
//
// VOID
// NTAPI
// VidCleanUp(VOID)
// {
//     /* Just fill the screen black */
//     VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK);
// }
//
// VOID
// ResetDisplay(
//      BOOLEAN SetMode)
// {
//     RtlZeroMemory(BackBuffer, BackBufferSize);
//     RtlZeroMemory((PVOID)FrameBufferStart, FrameBufferSize);
//
//     /* Re-initialize the palette and fill the screen black */
//     InitializePalette();
//     VidSolidColorFill(0, 0, SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1, BV_COLOR_BLACK);
// }
//
// VOID
// InitPaletteWithTable(
//      const ULONG* Table,
//      ULONG Count)
// {
//     const ULONG* Entry = Table;
//     ULONG i;
//     BOOLEAN HasChanged = FALSE;
//
//     for (i = 0; i < Count; i++, Entry++)
//     {
//         HasChanged |= !!((CachedPalette[i] ^ *Entry) & 0x00FFFFFF);
//         CachedPalette[i] = *Entry | 0xFF000000;
//     }
//
//     /* Re-apply the palette if it has changed */
//     if (HasChanged)
//         ApplyPalette();
// }
//
// VOID
// SetPixel(
//      ULONG Left,
//      ULONG Top,
//      UCHAR Color)
// {
//     PUCHAR Back = BB_PIXEL(Left, Top);
//     PULONG Frame = (PULONG)FB_PIXEL(Left, Top);
//
//     *Back = Color;
//     for (ULONG i = VidpYScale; i > 0; --i)
//     {
//         PULONG Pixel = Frame;
//         for (ULONG j = VidpXScale; j > 0; --j)
//             *Pixel++ = CachedPalette[Color];
//         Frame = (PULONG)((ULONG_PTR)Frame + BytesPerScanLine);
//     }
// }
//
// VOID
// PreserveRow(
//      ULONG CurrentTop,
//      ULONG TopDelta,
//      BOOLEAN Restore)
// {
//     PUCHAR NewPosition, OldPosition;
//
//     /* Calculate the position in memory for the row */
//     if (Restore)
//     {
//         /* Restore the row by copying back the contents saved off-screen */
//         NewPosition = BB_PIXEL(0, CurrentTop);
//         OldPosition = BB_PIXEL(0, SCREEN_HEIGHT);
//     }
//     else
//     {
//         /* Preserve the row by saving its contents off-screen */
//         NewPosition = BB_PIXEL(0, SCREEN_HEIGHT);
//         OldPosition = BB_PIXEL(0, CurrentTop);
//     }
//
//     /* Set the count and copy the pixel data back to the other position in the backbuffer */
//     ULONG Count = TopDelta * SCREEN_WIDTH;
//     RtlCopyMemory(NewPosition, OldPosition, Count);
//
//     /* On restore, mirror the backbuffer changes to the framebuffer */
//     if (Restore)
//     {
//         NewPosition = BB_PIXEL(0, CurrentTop);
//         for (ULONG y = 0; y < TopDelta; ++y)
//         {
//             PULONG Frame = (PULONG)FB_PIXEL(0, CurrentTop + y);
//             PULONG Pixel = Frame;
//             for (Count = 0; Count < SCREEN_WIDTH; ++Count)
//             {
//                 for (ULONG j = VidpXScale; j > 0; --j)
//                     *Pixel++ = CachedPalette[*NewPosition];
//                 NewPosition++;
//             }
//             Pixel = Frame;
//             for (ULONG i = VidpYScale-1; i > 0; --i)
//             {
//                 Pixel = (PULONG)((ULONG_PTR)Pixel + BytesPerScanLine);
//                 RtlCopyMemory(Pixel, Frame, VidpXScale * SCREEN_WIDTH * BytesPerPixel);
//             }
//         }
//     }
// }
//
// VOID
// DoScroll(
//      ULONG Scroll)
// {
//     ULONG RowSize = VidpScrollRegion.Right - VidpScrollRegion.Left + 1;
//
//     /* Calculate the position in memory for the row */
//     PUCHAR OldPosition = BB_PIXEL(VidpScrollRegion.Left, VidpScrollRegion.Top + Scroll);
//     PUCHAR NewPosition = BB_PIXEL(VidpScrollRegion.Left, VidpScrollRegion.Top);
//
//     /* Start loop */
//     for (ULONG Top = VidpScrollRegion.Top; Top <= VidpScrollRegion.Bottom; ++Top)
//     {
//         /* Scroll the row */
//         RtlCopyMemory(NewPosition, OldPosition, RowSize);
//
//         PULONG Frame = (PULONG)FB_PIXEL(VidpScrollRegion.Left, Top);
//         PULONG Pixel = Frame;
//         for (ULONG Count = 0; Count < RowSize; ++Count)
//         {
//             for (ULONG j = VidpXScale; j > 0; --j)
//                 *Pixel++ = CachedPalette[NewPosition[Count]];
//         }
//         Pixel = Frame;
//         for (ULONG i = VidpYScale-1; i > 0; --i)
//         {
//             Pixel = (PULONG)((ULONG_PTR)Pixel + BytesPerScanLine);
//             RtlCopyMemory(Pixel, Frame, VidpXScale * RowSize * BytesPerPixel);
//         }
//
//         OldPosition += SCREEN_WIDTH;
//         NewPosition += SCREEN_WIDTH;
//     }
// }
//
// VOID
// DisplayCharacter(
//      CHAR Character,
//      ULONG Left,
//      ULONG Top,
//      ULONG TextColor,
//      ULONG BackColor)
// {
//     /* Get the font line for this character */
//     const UCHAR* FontChar = GetFontPtr(Character);
//
//     /* Loop each pixel height */
//     for (ULONG y = Top; y < Top + BOOTCHAR_HEIGHT; ++y, FontChar += FONT_PTR_DELTA)
//     {
//         /* Loop each pixel width */
//         ULONG x = Left;
//         for (UCHAR bit = 1 << (BOOTCHAR_WIDTH - 1); bit > 0; bit >>= 1, ++x)
//         {
//             /* If we should draw this pixel, use the text color. Otherwise
//              * this is a background pixel, draw it unless it's transparent. */
//             if (*FontChar & bit)
//                 SetPixel(x, y, (UCHAR)TextColor);
//             else if (BackColor < BV_COLOR_NONE)
//                 SetPixel(x, y, (UCHAR)BackColor);
//         }
//     }
// }
//
// VOID
// NTAPI
// VidSolidColorFill(
//      ULONG Left,
//      ULONG Top,
//      ULONG Right,
//      ULONG Bottom,
//      UCHAR Color)
// {
//     for (; Top <= Bottom; ++Top)
//     {
//         PUCHAR Back = BB_PIXEL(Left, Top);
//         // NOTE: Assumes 32bpp
//         PULONG Frame = (PULONG)FB_PIXEL(Left, Top);
//         PULONG Pixel = Frame;
//         for (ULONG L = Left; L <= Right; ++L)
//         {
//             *Back++ = Color;
//             for (ULONG j = VidpXScale; j > 0; --j)
//                 *Pixel++ = CachedPalette[Color];
//         }
//         Pixel = Frame;
//         for (ULONG i = VidpYScale-1; i > 0; --i)
//         {
//             Pixel = (PULONG)((ULONG_PTR)Pixel + BytesPerScanLine);
//             RtlCopyMemory(Pixel, Frame, VidpXScale * (Right - Left + 1) * BytesPerPixel);
//         }
//     }
// }
//
// VOID
// NTAPI
// VidScreenToBufferBlt(
//      PUCHAR Buffer,
//      ULONG Left,
//      ULONG Top,
//      ULONG Width,
//      ULONG Height,
//      ULONG Delta)
// {
//     ULONG x, y;
//
//     /* Clear the destination buffer */
//     RtlZeroMemory(Buffer, Delta * Height);
//
//     /* Start the outer Y height loop */
//     for (y = 0; y < Height; ++y)
//     {
//         /* Set current scanline */
//         PUCHAR Back = BB_PIXEL(Left, Top + y);
//         PUCHAR Buf = Buffer + y * Delta;
//
//         /* Start the X inner loop */
//         for (x = 0; x < Width; x += sizeof(USHORT))
//         {
//             /* Read the current value */
//             *Buf = (*Back++ & 0xF) << 4;
//             *Buf |= *Back++ & 0xF;
//             Buf++;
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
