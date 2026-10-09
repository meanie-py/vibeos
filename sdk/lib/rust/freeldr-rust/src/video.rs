use ntkrnl_sys::base::*;
use crate::frldr::*;
use crate::ui_keystone::*;
use core::ffi::c_void;

// video.c file-local data/#defines rustconv dropped.
const RGB_MAX: ULONG = 64;
const RGB_MAX_PER_ITERATION: ULONG = 64;
const TAG_PALETTE_COLORS: ULONG = u32::from_be_bytes(*b"claP") as ULONG;
static mut VideoOffScreenBuffer: PVOID = core::ptr::null_mut();

extern "C" {
    fn MmFreeMemory(ptr: *mut core::ffi::c_void);
    fn MachVideoCopyOffScreenBufferToVRAM(buffer: PVOID);
    fn MachVideoGetPaletteColor( color: UCHAR, red: *mut UCHAR, green: *mut UCHAR, blue: *mut UCHAR, );
    fn MachVideoSync();
    fn MachVideoSetPaletteColor(Color: UCHAR, Red: UCHAR, Green: UCHAR, Blue: UCHAR);
    fn MachVideoGetBufferSize() -> u32;
    // mm.c canonical: `PVOID MmAllocateMemoryWithType(SIZE_T MemorySize, TYPE_OF_MEMORY MemoryType)`.
    // SIZE_T = usize; on arm64+amd64 that's 8 bytes. rustconv declared this
    // as `(u32, u32)` which writes only w0 (low 32 of x0); the high 32 bits
    // of x0 are then caller-side garbage, so MmAllocateMemoryWithType saw a
    // multi-GB size and failed. Critical for arm64 boot (Bug #5).
    fn MmAllocateMemoryWithType(size: usize, type_: u32) -> *mut core::ffi::c_void;
}

#[no_mangle]
pub unsafe extern "C" fn VideoAllocateOffScreenBuffer() -> *mut core::ffi::c_void {

    let BufferSize: u32;

    VideoFreeOffScreenBuffer();

    BufferSize = MachVideoGetBufferSize();

    // No display (UEFI with no GOP, as on a server): nothing to buffer. The C asks
    // for 0 bytes anyway, and the allocator reports that through the UI being set
    // up, whose message box then waits for a key nobody can press. NULL makes the
    // text UI fail cleanly, and UiInitialize carries on without one.
    if BufferSize == 0 {
        return core::ptr::null_mut();
    }

    VideoOffScreenBuffer = MmAllocateMemoryWithType(BufferSize as usize, LoaderFirmwareTemporary);

    return VideoOffScreenBuffer;
}

#[no_mangle]
pub unsafe extern "C" fn VideoFreeOffScreenBuffer() {
    if VideoOffScreenBuffer.is_null() {
        return;
    }

    MmFreeMemory(VideoOffScreenBuffer as *mut core::ffi::c_void);
    VideoOffScreenBuffer = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn VideoCopyOffScreenBufferToVRAM() {
    MachVideoCopyOffScreenBufferToVRAM(VideoOffScreenBuffer);
}

#[no_mangle]
pub unsafe extern "C" fn VideoSavePaletteState(Palette: *mut PALETTE_ENTRY, ColorCount: ULONG) {

    let mut Color: ULONG = 0;
    while Color < ColorCount {
        MachVideoGetPaletteColor(
            Color as UCHAR,
            core::ptr::addr_of_mut!((*Palette.add(Color as usize)).Red),
            core::ptr::addr_of_mut!((*Palette.add(Color as usize)).Green),
            core::ptr::addr_of_mut!((*Palette.add(Color as usize)).Blue),
        );
        Color += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn VideoRestorePaletteState(Palette: PPALETTE_ENTRY, ColorCount: ULONG) {

    let mut Color: ULONG;

    MachVideoSync();

    Color = 0;
    while Color < ColorCount {
        MachVideoSetPaletteColor(
            Color as UCHAR,
            (*Palette.offset(Color as isize)).Red,
            (*Palette.offset(Color as isize)).Green,
            (*Palette.offset(Color as isize)).Blue,
        );
        Color += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn VideoSetAllColorsToBlack(ColorCount: ULONG) {

    MachVideoSync();

    let mut Color: UCHAR = 0;
    while Color < ColorCount as UCHAR {
        MachVideoSetPaletteColor(Color, 0, 0, 0);
        Color = Color.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn VideoFadeIn(Palette: PPALETTE_ENTRY, ColorCount: ULONG) {
    let mut Index: ULONG;
    let mut Color: UCHAR;
    let PaletteColors: PPALETTE_ENTRY;

    PaletteColors = FrLdrTempAlloc(
        core::mem::size_of::<PALETTE_ENTRY>() * ColorCount as usize,
        TAG_PALETTE_COLORS
    ) as PPALETTE_ENTRY;
    if PaletteColors.is_null() {
        return;
    }

    Index = 0;
    while Index < RGB_MAX {
        Color = 0;
        while Color < ColorCount as UCHAR {
            MachVideoGetPaletteColor(
                Color,
                core::ptr::addr_of_mut!((*PaletteColors.offset(Color as isize)).Red),
                core::ptr::addr_of_mut!((*PaletteColors.offset(Color as isize)).Green),
                core::ptr::addr_of_mut!((*PaletteColors.offset(Color as isize)).Blue)
            );

            if (*PaletteColors.offset(Color as isize)).Red < (*Palette.offset(Color as isize)).Red {
                (*PaletteColors.offset(Color as isize)).Red += 1;
            }
            if (*PaletteColors.offset(Color as isize)).Green < (*Palette.offset(Color as isize)).Green {
                (*PaletteColors.offset(Color as isize)).Green += 1;
            }
            if (*PaletteColors.offset(Color as isize)).Blue < (*Palette.offset(Color as isize)).Blue {
                (*PaletteColors.offset(Color as isize)).Blue += 1;
            }

            if (*PaletteColors.offset(Color as isize)).Red > (*Palette.offset(Color as isize)).Red {
                (*PaletteColors.offset(Color as isize)).Red = (*Palette.offset(Color as isize)).Red;
            }
            if (*PaletteColors.offset(Color as isize)).Green > (*Palette.offset(Color as isize)).Green {
                (*PaletteColors.offset(Color as isize)).Green = (*Palette.offset(Color as isize)).Green;
            }
            if (*PaletteColors.offset(Color as isize)).Blue > (*Palette.offset(Color as isize)).Blue {
                (*PaletteColors.offset(Color as isize)).Blue = (*Palette.offset(Color as isize)).Blue;
            }

            Color += 1;
        }

        Color = 0;
        while Color < ColorCount as UCHAR {
            if (Color % RGB_MAX_PER_ITERATION as UCHAR) == 0 {
                MachVideoSync();
            }

            MachVideoSetPaletteColor(
                Color,
                (*PaletteColors.offset(Color as isize)).Red,
                (*PaletteColors.offset(Color as isize)).Green,
                (*PaletteColors.offset(Color as isize)).Blue
            );

            Color += 1;
        }

        Index += 1;
    }

    FrLdrTempFree(PaletteColors as PVOID, TAG_PALETTE_COLORS);
}

#[no_mangle]
pub unsafe extern "C" fn VideoFadeOut(ColorCount: ULONG) {
    let mut Index: ULONG;
    let mut Color: UCHAR;
    let mut Red: UCHAR = 0;
    let mut Green: UCHAR = 0;
    let mut Blue: UCHAR = 0;

    Index = 0;
    while Index < RGB_MAX {
        Color = 0;
        while Color < ColorCount as UCHAR {
            if (Color % RGB_MAX_PER_ITERATION as UCHAR) == 0 {
                MachVideoSync();
            }

            MachVideoGetPaletteColor(Color, &mut Red, &mut Green, &mut Blue);

            if Red > 0 {
                Red = Red - 1;
            }
            if Green > 0 {
                Green = Green - 1;
            }
            if Blue > 0 {
                Blue = Blue - 1;
            }

            MachVideoSetPaletteColor(Color, Red, Green, Blue);
            
            Color = Color + 1;
        }
        Index = Index + 1;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     FreeLoader
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     UI Video helpers for special effects.
//  * COPYRIGHT:   Copyright 1998-2003 Brian Palmer <brianp@sginet.com>
//  */
//
// #include <freeldr.h>
//
// #define RGB_MAX                 64
// #define RGB_MAX_PER_ITERATION   64
// #define TAG_PALETTE_COLORS      'claP'
//
// static PVOID VideoOffScreenBuffer = NULL;
//
// PVOID VideoAllocateOffScreenBuffer(VOID)
// {
//     ULONG BufferSize;
//
//     VideoFreeOffScreenBuffer();
//
//     BufferSize = MachVideoGetBufferSize();
//
//     VideoOffScreenBuffer = MmAllocateMemoryWithType(BufferSize, LoaderFirmwareTemporary);
//
//     return VideoOffScreenBuffer;
// }
//
// VOID VideoFreeOffScreenBuffer(VOID)
// {
//     if (!VideoOffScreenBuffer)
//         return;
//
//     MmFreeMemory(VideoOffScreenBuffer);
//     VideoOffScreenBuffer = NULL;
// }
//
// VOID VideoCopyOffScreenBufferToVRAM(VOID)
// {
//     MachVideoCopyOffScreenBufferToVRAM(VideoOffScreenBuffer);
// }
//
//
// VOID VideoSavePaletteState(PPALETTE_ENTRY Palette, ULONG ColorCount)
// {
//     ULONG Color;
//
//     for (Color = 0; Color < ColorCount; Color++)
//     {
//         MachVideoGetPaletteColor((UCHAR)Color, &Palette[Color].Red, &Palette[Color].Green, &Palette[Color].Blue);
//     }
// }
//
// VOID VideoRestorePaletteState(PPALETTE_ENTRY Palette, ULONG ColorCount)
// {
//     ULONG Color;
//
//     MachVideoSync();
//
//     for (Color = 0; Color < ColorCount; Color++)
//     {
//         MachVideoSetPaletteColor((UCHAR)Color, Palette[Color].Red, Palette[Color].Green, Palette[Color].Blue);
//     }
// }
//
//
// VOID VideoSetAllColorsToBlack(ULONG ColorCount)
// {
//     UCHAR Color;
//
//     MachVideoSync();
//
//     for (Color = 0; Color < ColorCount; Color++)
//     {
//         MachVideoSetPaletteColor(Color, 0, 0, 0);
//     }
// }
//
// VOID VideoFadeIn(PPALETTE_ENTRY Palette, ULONG ColorCount)
// {
//     ULONG Index;
//     UCHAR Color;
//     PPALETTE_ENTRY PaletteColors;
//
//     PaletteColors = FrLdrTempAlloc(sizeof(PALETTE_ENTRY) * ColorCount, TAG_PALETTE_COLORS);
//     if (!PaletteColors) return;
//
//     for (Index = 0; Index < RGB_MAX; Index++)
//     {
//         for (Color = 0; Color < ColorCount; Color++)
//         {
//             MachVideoGetPaletteColor(Color, &PaletteColors[Color].Red, &PaletteColors[Color].Green, &PaletteColors[Color].Blue);
//
//             // Increment each color so it approaches its real value
//             if (PaletteColors[Color].Red < Palette[Color].Red)
//             {
//                 PaletteColors[Color].Red++;
//             }
//             if (PaletteColors[Color].Green < Palette[Color].Green)
//             {
//                 PaletteColors[Color].Green++;
//             }
//             if (PaletteColors[Color].Blue < Palette[Color].Blue)
//             {
//                 PaletteColors[Color].Blue++;
//             }
//
//             // Make sure we haven't exceeded the real value
//             if (PaletteColors[Color].Red > Palette[Color].Red)
//             {
//                 PaletteColors[Color].Red = Palette[Color].Red;
//             }
//             if (PaletteColors[Color].Green > Palette[Color].Green)
//             {
//                 PaletteColors[Color].Green = Palette[Color].Green;
//             }
//             if (PaletteColors[Color].Blue > Palette[Color].Blue)
//             {
//                 PaletteColors[Color].Blue = Palette[Color].Blue;
//             }
//         }
//
//         // Set the colors
//         for (Color = 0; Color < ColorCount; Color++)
//         {
//             if ((Color % RGB_MAX_PER_ITERATION) == 0)
//             {
//                 MachVideoSync();
//             }
//
//             MachVideoSetPaletteColor(Color, PaletteColors[Color].Red, PaletteColors[Color].Green, PaletteColors[Color].Blue);
//         }
//     }
//
//     FrLdrTempFree(PaletteColors, TAG_PALETTE_COLORS);
// }
//
// VOID VideoFadeOut(ULONG ColorCount)
// {
//     ULONG Index;
//     UCHAR Color;
//     UCHAR Red;
//     UCHAR Green;
//     UCHAR Blue;
//
//     for (Index = 0; Index < RGB_MAX; Index++)
//     {
//         for (Color = 0; Color < ColorCount; Color++)
//         {
//             if ((Color % RGB_MAX_PER_ITERATION) == 0)
//             {
//                 MachVideoSync();
//             }
//
//             MachVideoGetPaletteColor(Color, &Red, &Green, &Blue);
//
//             if (Red > 0)
//             {
//                 Red--;
//             }
//             if (Green > 0)
//             {
//                 Green--;
//             }
//             if (Blue > 0)
//             {
//                 Blue--;
//             }
//
//             MachVideoSetPaletteColor(Color, Red, Green, Blue);
//         }
//     }
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
