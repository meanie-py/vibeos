//! Translated from boot/freeldr/freeldr/arch/vidfb.c
//!
//! Linear framebuffer driver + FbCons* console wrappers.
//! UEFI-only (PC fallback lives in vidfb_pc_fallback.rs under
//! `not(feature = "uefi")`).
//!
//! Key cross-module surface:
//!   * VidFbInitializeVideo — called by uefivid.rs UefiInitializeVideo.
//!     Signature must match uefivid.rs's extern decl:
//!       fn VidFbInitializeVideo(
//!           pFbData: *mut PCM_FRAMEBUF_DEVICE_DATA, ...,
//!           PixelMasks: PPIXEL_BITMASK) -> BOOLEAN
//!     i.e. PixelMasks is *mut EFI_PIXEL_BITMASK.
//!   * FbConsClearScreen/PutChar/GetDisplaySize/GetBufferSize/
//!     CopyOffScreenBufferToVRAM/ScrollUp — called by uefivid.rs.
//!     vidfb_pc_fallback.rs owns the symbols under not(feature="uefi").
//!   * PixelBitmasksToBpp — vidfb.h FORCEINLINE.
//!     uefivid.rs owns the symbol under feature="uefi" (it's the
//!     per-binary-owner; vidfb.c didn't export it). Extern-consume.

#[allow(unused_imports)] use crate::{ERR, TRACE};
#[allow(dead_code, non_upper_case_globals)] const DbgDefaultChannel: u32 = crate::frldr::DPRINT_UI; // vidfb.c: DBG_DEFAULT_CHANNEL(UI)
use ntkrnl_sys::base::*;
use crate::frldr::*;
use core::ffi::c_void;
use crate::vgafont::{BitmapFont8x16, CHAR_WIDTH, CHAR_HEIGHT};

extern "C" {
    // vidfb.h FORCEINLINE — provided by uefivid.rs's `#[cfg(feature="uefi")]`
    // path. Pure popcount of OR'd masks; uefivid is the per-binary owner.
    fn PixelBitmasksToBpp(
        RedMask: ULONG,
        GreenMask: ULONG,
        BlueMask: ULONG,
        ReservedMask: ULONG,
    ) -> ULONG;
}

// ULONG-aliased so the rustconv emissions of `UINT32`/`PUINT32` work.
type UINT32 = u32;
type PUINT32 = *mut u32;

// vidfb.c's `UNIMPLEMENTED` macro is a debug-only abort path; under the
// freeldr no_std build we just ignore the unreachable arms.
macro_rules! UNIMPLEMENTED { () => { () }; }

// ---- file-scope preamble (translated) ----
const TOP_BOTTOM_LINES: u32 = 0;
const VGA_CHAR_SIZE: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy)]
struct _FRAMEBUFFER_INFO {
    BaseAddress: ULONG_PTR,
    BufferSize: ULONG,
    ScreenWidth: ULONG,
    ScreenHeight: ULONG,
    PixelsPerScanLine: ULONG,
    BitsPerPixel: ULONG,
    PixelMasks: EFI_PIXEL_BITMASK,
    BytesPerPixel: ULONG,
    Delta: ULONG,
}
type FRAMEBUFFER_INFO = _FRAMEBUFFER_INFO;

static mut framebufInfo: _FRAMEBUFFER_INFO = _FRAMEBUFFER_INFO {
    BaseAddress: 0,
    BufferSize: 0,
    ScreenWidth: 0,
    ScreenHeight: 0,
    PixelsPerScanLine: 0,
    BitsPerPixel: 0,
    PixelMasks: EFI_PIXEL_BITMASK {
        RedMask: 0,
        GreenMask: 0,
        BlueMask: 0,
        ReservedMask: 0,
    },
    BytesPerPixel: 0,
    Delta: 0,
};

// vidfb.c `static CM_FRAMEBUF_DEVICE_DATA FrameBufferData = {0};` — file-local
// (no #[no_mangle]). uefivid.rs has its OWN `FrameBufferData` (pointer type);
// they are distinct entities, so per-module statics keep them isolated.
static mut FrameBufferData: CM_FRAMEBUF_DEVICE_DATA = CM_FRAMEBUF_DEVICE_DATA {
    Version: 0,
    Revision: 0,
    VideoClock: 0,
    FrameBufferOffset: 0,
    ScreenWidth: 0,
    ScreenHeight: 0,
    PixelsPerScanLine: 0,
    BitsPerPixel: 0,
    PixelMasks: CM_FRAMEBUF_PIXEL_MASKS {
        RedMask: 0,
        GreenMask: 0,
        BlueMask: 0,
        ReservedMask: 0,
    },
};

#[allow(dead_code)]
unsafe fn VidFbPrintFramebufferInfo() {
    TRACE!("Framebuffer format:\n");
    TRACE!("    BaseAddress       : 0x%X\n", framebufInfo.BaseAddress);
    TRACE!("    BufferSize        : %lu\n", framebufInfo.BufferSize);
    TRACE!("    ScreenWidth       : %lu\n", framebufInfo.ScreenWidth);
    TRACE!("    ScreenHeight      : %lu\n", framebufInfo.ScreenHeight);
    TRACE!("    PixelsPerScanLine : %lu\n", framebufInfo.PixelsPerScanLine);
    TRACE!("    BitsPerPixel      : %lu\n", framebufInfo.BitsPerPixel);
    TRACE!("    BytesPerPixel     : %lu\n", framebufInfo.BytesPerPixel);
    TRACE!("    Delta             : %lu\n", framebufInfo.Delta);
    TRACE!("    ARGB masks:       : %08x/%08x/%08x/%08x\n", framebufInfo.PixelMasks.ReservedMask, framebufInfo.PixelMasks.RedMask, framebufInfo.PixelMasks.GreenMask, framebufInfo.PixelMasks.BlueMask);
}

#[no_mangle]
pub unsafe extern "C" fn VidFbInitializeVideo(
    pFbData: *mut PCM_FRAMEBUF_DEVICE_DATA,
    BaseAddress: ULONG_PTR,
    BufferSize: ULONG,
    ScreenWidth: UINT32,
    ScreenHeight: UINT32,
    PixelsPerScanLine: UINT32,
    BitsPerPixel: UINT32,
    PixelMasks: PPIXEL_BITMASK,
) -> BOOLEAN {
    let BitMasks: *mut EFI_PIXEL_BITMASK = core::ptr::addr_of_mut!(framebufInfo.PixelMasks);

    if !pFbData.is_null() {
        *pFbData = core::ptr::null_mut();
    }

    core::ptr::write_bytes(&mut framebufInfo as *mut FRAMEBUFFER_INFO, 0u8, 1);

    if (ScreenWidth < 1) || (ScreenHeight < 1) {
        ERR!("Invalid framebuffer dimensions\n");
        return 0u8;
    }

    framebufInfo.BaseAddress = BaseAddress;
    framebufInfo.BufferSize = BufferSize;
    framebufInfo.ScreenWidth = ScreenWidth;
    framebufInfo.ScreenHeight = ScreenHeight;
    framebufInfo.PixelsPerScanLine = PixelsPerScanLine;
    framebufInfo.BitsPerPixel = BitsPerPixel;

    framebufInfo.BytesPerPixel = (BitsPerPixel + 7) / 8;
    framebufInfo.Delta = (PixelsPerScanLine * framebufInfo.BytesPerPixel + 3) & !3u32;

    if !(ScreenHeight * framebufInfo.Delta <= BufferSize) {
        ERR!("Framebuffer doesn't fit inside the video RAM (FB size: %lu, VRAM size: %lu)\n", ScreenHeight * framebufInfo.Delta, BufferSize);
        return 0u8;
    }

    if BitsPerPixel != 32 {
        ERR!("Unsupported %lu bits per pixel format\n", BitsPerPixel);
        return 0u8;
    }

    if BitsPerPixel > 8 {
        if PixelMasks.is_null() ||
            ((*PixelMasks).RedMask == 0 &&
             (*PixelMasks).GreenMask == 0 &&
             (*PixelMasks).BlueMask == 0) {
            match BitsPerPixel {
                32 | 24 => {
                    (*BitMasks).RedMask = 0x00FF0000;
                    (*BitMasks).GreenMask = 0x0000FF00;
                    (*BitMasks).BlueMask = 0x000000FF;
                    (*BitMasks).ReservedMask = ((1u32 << (BitsPerPixel - 24)) - 1) << 24;
                }
                16 => {
                    (*BitMasks).RedMask = 0xF800;
                    (*BitMasks).GreenMask = 0x07E0;
                    (*BitMasks).BlueMask = 0x001F;
                    (*BitMasks).ReservedMask = 0;
                }
                15 => {
                    (*BitMasks).RedMask = 0x7C00;
                    (*BitMasks).GreenMask = 0x03E0;
                    (*BitMasks).BlueMask = 0x001F;
                    (*BitMasks).ReservedMask = 0x8000;
                }
                _ => {
                    UNIMPLEMENTED!();
                    core::ptr::write_bytes(BitMasks, 0u8, 1);
                }
            }
        } else {
            core::ptr::copy_nonoverlapping(PixelMasks, BitMasks, 1);
        }
    } else {
        core::ptr::write_bytes(BitMasks, 0u8, 1);
    }

    #[cfg(debug_assertions)]
    {
        VidFbPrintFramebufferInfo();
        {
            let BppFromMasks =
                PixelBitmasksToBpp((*BitMasks).RedMask,
                                   (*BitMasks).GreenMask,
                                   (*BitMasks).BlueMask,
                                   (*BitMasks).ReservedMask);
            TRACE!("BitsPerPixel = %lu , BppFromMasks = %lu\n", BitsPerPixel, BppFromMasks);
        }
    }

    if !pFbData.is_null() {
        FrameBufferData.FrameBufferOffset = 0;
        FrameBufferData.ScreenWidth = framebufInfo.ScreenWidth;
        FrameBufferData.ScreenHeight = framebufInfo.ScreenHeight;
        FrameBufferData.PixelsPerScanLine = framebufInfo.PixelsPerScanLine;
        FrameBufferData.BitsPerPixel = framebufInfo.BitsPerPixel;

        // CM_FRAMEBUF_PIXEL_MASKS and EFI_PIXEL_BITMASK are layout-identical
        // (4x ULONG: Red/Green/Blue/Reserved). C uses RtlCopyMemory.
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!(framebufInfo.PixelMasks) as *const u8,
            core::ptr::addr_of_mut!(FrameBufferData.PixelMasks) as *mut u8,
            core::mem::size_of::<CM_FRAMEBUF_PIXEL_MASKS>(),
        );

        *pFbData = core::ptr::addr_of_mut!(FrameBufferData);
    }

    return 1u8;
}

#[no_mangle]
pub unsafe extern "C" fn VidFbClearScreenColor(
    Color: UINT32,
    FullScreen: BOOLEAN,
) {
    let mut Line: u32;
    let mut Col: u32;
    let mut p: PUINT32;

    Line = 0;
    while Line < framebufInfo.ScreenHeight - (if FullScreen != 0 { 0 } else { 2 * TOP_BOTTOM_LINES }) {
        p = (framebufInfo.BaseAddress as *mut u8)
            .add(((Line + (if FullScreen != 0 { 0 } else { TOP_BOTTOM_LINES })) * framebufInfo.Delta) as usize)
            as PUINT32;
        Col = 0;
        while Col < framebufInfo.ScreenWidth {
            *p = Color;
            p = p.add(1);
            Col += 1;
        }
        Line += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn VidFbOutputChar(
    Char: UCHAR,
    X: ULONG,
    Y: ULONG,
    FgColor: UINT32,
    BgColor: UINT32)
{
    let FontPtr: *const u8;
    let mut Pixel: PUINT32;
    let mut Mask: u8;
    let mut Line: u32;
    let mut Col: u32;

    let cw: u32 = CHAR_WIDTH as u32;
    let ch: u32 = CHAR_HEIGHT as u32;

    /* Don't display outside of the screen, nor partial characters */
    if (X + cw - 1 >= framebufInfo.ScreenWidth) ||
        (Y + ch - 1 >= (framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES))
    {
        return;
    }

    FontPtr = BitmapFont8x16.as_ptr().offset((Char as isize) * (ch as isize));
    Pixel = ((framebufInfo.BaseAddress as *mut u8)
        .offset(((Y + TOP_BOTTOM_LINES) * framebufInfo.Delta + X * core::mem::size_of::<UINT32>() as u32) as isize)) as PUINT32;

    Line = 0;
    while Line < ch {
        Mask = 0x80;
        Col = 0;
        while Col < cw {
            *Pixel.offset(Col as isize) = if 0 != (*FontPtr.offset(Line as isize) & Mask) { FgColor } else { BgColor };
            Mask = Mask >> 1;
            Col += 1;
        }
        Pixel = (Pixel as *mut u8).offset(framebufInfo.Delta as isize) as PUINT32;
        Line += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn VidFbGetDisplaySize(
    Width: *mut ULONG,
    Height: *mut ULONG,
    Depth: *mut ULONG)
{
    *Width = framebufInfo.ScreenWidth;
    *Height = framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES;
    *Depth = framebufInfo.BitsPerPixel;
}

#[no_mangle]
pub unsafe extern "C" fn VidFbGetBufferSize() -> ULONG {
    (framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) *
        framebufInfo.ScreenWidth * framebufInfo.BytesPerPixel
}

#[no_mangle]
pub unsafe extern "C" fn VidFbScrollUp(
    Color: UINT32,
    Scroll: ULONG,
) {
    let mut Dst = ((framebufInfo.BaseAddress as *mut u8)
        .offset((TOP_BOTTOM_LINES * framebufInfo.Delta) as isize)) as PUINT32;
    let mut Src = ((framebufInfo.BaseAddress as *mut u8)
        .offset(((TOP_BOTTOM_LINES + Scroll) * framebufInfo.Delta) as isize)) as PUINT32;
    let mut PixelCount: ULONG =
        framebufInfo.ScreenWidth * ((framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) - Scroll);

    while PixelCount > 0 {
        PixelCount -= 1;
        *Dst = *Src;
        Dst = Dst.offset(1);
        Src = Src.offset(1);
    }

    PixelCount = 0;
    while PixelCount < framebufInfo.ScreenWidth * Scroll {
        *Dst = Color;
        Dst = Dst.offset(1);
        PixelCount += 1;
    }
}

// vidfb.c #if 0 block — keep as dead-stubs to satisfy any future external
// reference. NOT marked #[no_mangle] (C compile is `#if 0` so no symbol).
#[allow(dead_code)]
unsafe fn VidFbSetTextCursorPosition(_X: UCHAR, _Y: UCHAR) {}
#[allow(dead_code)]
unsafe fn VidFbHideShowTextCursor(_Show: BOOLEAN) {}
#[allow(dead_code)]
unsafe fn VidFbIsPaletteFixed() -> BOOLEAN { 0u8 }
#[allow(dead_code)]
unsafe fn VidFbSetPaletteColor(_Color: UCHAR, _Red: UCHAR, _Green: UCHAR, _Blue: UCHAR) {}
#[allow(dead_code)]
unsafe fn VidFbGetPaletteColor(_Color: UCHAR, _Red: PUCHAR, _Green: PUCHAR, _Blue: PUCHAR) {}

unsafe fn FbConsAttrToSingleColor(Attr: UCHAR) -> UINT32 {
    let Intensity: UCHAR = if 0 == (Attr & 0x08) { 127 } else { 255 };

    0xff000000u32 |
        (if 0 == (Attr & 0x04) { 0 } else { (Intensity as UINT32) << 16 }) |
        (if 0 == (Attr & 0x02) { 0 } else { (Intensity as UINT32) << 8 }) |
        (if 0 == (Attr & 0x01) { 0 } else { Intensity as UINT32 })
}

unsafe fn FbConsAttrToColors(
    Attr: UCHAR,
    FgColor: PUINT32,
    BgColor: PUINT32)
{
    *FgColor = FbConsAttrToSingleColor(Attr & 0x0F);
    *BgColor = FbConsAttrToSingleColor((Attr >> 4) & 0x0F);
}

#[no_mangle]
pub unsafe extern "C" fn FbConsClearScreen(Attr: UCHAR) {
    let mut FgColor: UINT32 = 0;
    let mut BgColor: UINT32 = 0;
    FbConsAttrToColors(Attr, &mut FgColor, &mut BgColor);
    VidFbClearScreenColor(BgColor, 0u8);
}

#[no_mangle]
pub unsafe extern "C" fn FbConsOutputChar(
    Char: UCHAR,
    Column: ULONG,
    Row: ULONG,
    FgColor: UINT32,
    BgColor: UINT32)
{
    let cw: u32 = CHAR_WIDTH as u32;
    let ch: u32 = CHAR_HEIGHT as u32;
    if (Column >= (framebufInfo.ScreenWidth / cw)) ||
       (Row >= ((framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) / ch))
    {
        return;
    }
    VidFbOutputChar(Char, Column * cw, Row * ch, FgColor, BgColor);
}

// uefivid.rs declares FbConsPutChar with `Ch: i32` (rust's C `int`); on i386
// it's ABI-identical to UCHAR for stack args. Match the C header here (UCHAR).
#[no_mangle]
pub unsafe extern "C" fn FbConsPutChar(
    Char: UCHAR,
    Attr: UCHAR,
    Column: ULONG,
    Row: ULONG)
{
    let mut FgColor: UINT32 = 0;
    let mut BgColor: UINT32 = 0;
    FbConsAttrToColors(Attr, &mut FgColor, &mut BgColor);
    FbConsOutputChar(Char, Column, Row, FgColor, BgColor);
}

#[no_mangle]
pub unsafe extern "C" fn FbConsGetDisplaySize(
    Width: *mut ULONG,
    Height: *mut ULONG,
    Depth: *mut ULONG)
{
    let cw: u32 = CHAR_WIDTH as u32;
    let ch: u32 = CHAR_HEIGHT as u32;
    *Width = framebufInfo.ScreenWidth / cw;
    *Height = (framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) / ch;
    *Depth = framebufInfo.BitsPerPixel;
}

#[no_mangle]
pub unsafe extern "C" fn FbConsGetBufferSize() -> ULONG {
    let cw: u32 = CHAR_WIDTH as u32;
    let ch: u32 = CHAR_HEIGHT as u32;
    ((framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) / ch) *
        (framebufInfo.ScreenWidth / cw) * VGA_CHAR_SIZE
}

#[no_mangle]
pub unsafe extern "C" fn FbConsCopyOffScreenBufferToVRAM(Buffer: PVOID) {
    let mut OffScreenBuffer = Buffer as PUCHAR;
    let mut Row: ULONG;
    let mut Col: ULONG;

    let cw: u32 = CHAR_WIDTH as u32;
    let ch: u32 = CHAR_HEIGHT as u32;
    let Width = framebufInfo.ScreenWidth / cw;
    let Height = (framebufInfo.ScreenHeight - 2 * TOP_BOTTOM_LINES) / ch;

    Row = 0;
    while Row < Height {
        Col = 0;
        while Col < Width {
            FbConsPutChar(*OffScreenBuffer.offset(0), *OffScreenBuffer.offset(1), Col, Row);
            OffScreenBuffer = OffScreenBuffer.offset(VGA_CHAR_SIZE as isize);
            Col += 1;
        }
        Row += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn FbConsScrollUp(Attr: UCHAR) {
    let mut BgColor: UINT32 = 0;
    let mut Dummy: UINT32 = 0;
    FbConsAttrToColors(Attr, &mut Dummy as PUINT32, &mut BgColor as PUINT32);
    VidFbScrollUp(BgColor, CHAR_HEIGHT as ULONG);
}


// ============================================================================
// ORIGINAL C SOURCE (preserved for reference; see boot/freeldr/freeldr/arch/vidfb.c)
// ============================================================================
