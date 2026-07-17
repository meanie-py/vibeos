use win32k_sys::win32k::*;

// Per-format renderer entrypoints + generic stretch/alpha (DIB_XXBPP_*),
// referenced by DibFunctionsForBitmapFormat. The DIB_NBPP_* live in the
// sibling renderer modules (#[no_mangle] extern "C") or i386 hand-asm;
// DIB_XXBPP_{StretchBlt,AlphaBlend} are the generic C impls (stretchblt.c/
// alphablend.c, still live). Declared here so the table can take their
// addresses; the linker resolves each by name (the bitblt.rs/dib_helpers
// cross-module pattern).
extern "C" {
    fn DIB_1BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_1BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_1BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_1BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_1BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_1BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_1BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_1BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_4BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_4BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_4BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_4BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_4BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_4BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_4BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_4BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_8BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_8BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_8BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_8BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_8BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_8BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_8BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_8BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_16BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_16BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_16BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_16BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_16BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_16BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_16BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_16BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_24BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_24BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_24BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_24BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_24BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_24BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_24BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_24BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_32BPP_PutPixel(so: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG);
    fn DIB_32BPP_GetPixel(so: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG;
    fn DIB_32BPP_HLine(so: *mut SURFOBJ, x1: LONG, x2: LONG, y: LONG, c: ULONG);
    fn DIB_32BPP_VLine(so: *mut SURFOBJ, x: LONG, y1: LONG, y2: LONG, c: ULONG);
    fn DIB_32BPP_BitBlt(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_32BPP_BitBltSrcCopy(bi: PBLTINFO) -> BOOLEAN;
    fn DIB_32BPP_TransparentBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, xlo: *mut XLATEOBJ, tc: ULONG) -> BOOLEAN;
    fn DIB_32BPP_ColorFill(d: *mut SURFOBJ, dr: *mut RECTL, c: ULONG) -> BOOLEAN;
    fn DIB_16BPP_AlphaBlend(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, clip: *mut CLIPOBJ, xlo: *mut XLATEOBJ, bo: *mut BLENDOBJ) -> BOOLEAN;
    fn DIB_24BPP_AlphaBlend(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, clip: *mut CLIPOBJ, xlo: *mut XLATEOBJ, bo: *mut BLENDOBJ) -> BOOLEAN;
    fn DIB_32BPP_AlphaBlend(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, clip: *mut CLIPOBJ, xlo: *mut XLATEOBJ, bo: *mut BLENDOBJ) -> BOOLEAN;
    fn DIB_XXBPP_StretchBlt(d: *mut SURFOBJ, sr: *mut SURFOBJ, ps: *mut SURFOBJ, ms: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, mo: *mut POINTL, br: *mut BRUSHOBJ, bo: *mut POINTL, xlo: *mut XLATEOBJ, rop: ROP4) -> BOOLEAN;
    fn DIB_XXBPP_AlphaBlend(d: *mut SURFOBJ, sr: *mut SURFOBJ, dr: *mut RECTL, srr: *mut RECTL, clip: *mut CLIPOBJ, xlo: *mut XLATEOBJ, bo: *mut BLENDOBJ) -> BOOLEAN;
}


// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut notmask: [u8; 2] = [0x0f, 0xf0];
#[no_mangle]
pub static mut altnotmask: [u8; 2] = [0xf0, 0x0f];
#[no_mangle]
pub static mut DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11] = [
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(Dummy_PutPixel),
        DIB_GetPixel: Some(Dummy_GetPixel),
        DIB_HLine: Some(Dummy_HLine),
        DIB_VLine: Some(Dummy_VLine),
        DIB_BitBlt: Some(Dummy_BitBlt),
        DIB_BitBltSrcCopy: Some(Dummy_BitBlt),
        DIB_StretchBlt: Some(Dummy_StretchBlt),
        DIB_TransparentBlt: Some(Dummy_TransparentBlt),
        DIB_ColorFill: Some(Dummy_ColorFill),
        DIB_AlphaBlend: Some(Dummy_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_1BPP_PutPixel),
        DIB_GetPixel: Some(DIB_1BPP_GetPixel),
        DIB_HLine: Some(DIB_1BPP_HLine),
        DIB_VLine: Some(DIB_1BPP_VLine),
        DIB_BitBlt: Some(DIB_1BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_1BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_1BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_1BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_XXBPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_4BPP_PutPixel),
        DIB_GetPixel: Some(DIB_4BPP_GetPixel),
        DIB_HLine: Some(DIB_4BPP_HLine),
        DIB_VLine: Some(DIB_4BPP_VLine),
        DIB_BitBlt: Some(DIB_4BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_4BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_4BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_4BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_XXBPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_8BPP_PutPixel),
        DIB_GetPixel: Some(DIB_8BPP_GetPixel),
        DIB_HLine: Some(DIB_8BPP_HLine),
        DIB_VLine: Some(DIB_8BPP_VLine),
        DIB_BitBlt: Some(DIB_8BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_8BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_8BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_8BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_XXBPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_16BPP_PutPixel),
        DIB_GetPixel: Some(DIB_16BPP_GetPixel),
        DIB_HLine: Some(DIB_16BPP_HLine),
        DIB_VLine: Some(DIB_16BPP_VLine),
        DIB_BitBlt: Some(DIB_16BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_16BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_16BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_16BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_16BPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_24BPP_PutPixel),
        DIB_GetPixel: Some(DIB_24BPP_GetPixel),
        DIB_HLine: Some(DIB_24BPP_HLine),
        DIB_VLine: Some(DIB_24BPP_VLine),
        DIB_BitBlt: Some(DIB_24BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_24BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_24BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_24BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_24BPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(DIB_32BPP_PutPixel),
        DIB_GetPixel: Some(DIB_32BPP_GetPixel),
        DIB_HLine: Some(DIB_32BPP_HLine),
        DIB_VLine: Some(DIB_32BPP_VLine),
        DIB_BitBlt: Some(DIB_32BPP_BitBlt),
        DIB_BitBltSrcCopy: Some(DIB_32BPP_BitBltSrcCopy),
        DIB_StretchBlt: Some(DIB_XXBPP_StretchBlt),
        DIB_TransparentBlt: Some(DIB_32BPP_TransparentBlt),
        DIB_ColorFill: Some(DIB_32BPP_ColorFill),
        DIB_AlphaBlend: Some(DIB_32BPP_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(Dummy_PutPixel),
        DIB_GetPixel: Some(Dummy_GetPixel),
        DIB_HLine: Some(Dummy_HLine),
        DIB_VLine: Some(Dummy_VLine),
        DIB_BitBlt: Some(Dummy_BitBlt),
        DIB_BitBltSrcCopy: Some(Dummy_BitBlt),
        DIB_StretchBlt: Some(Dummy_StretchBlt),
        DIB_TransparentBlt: Some(Dummy_TransparentBlt),
        DIB_ColorFill: Some(Dummy_ColorFill),
        DIB_AlphaBlend: Some(Dummy_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(Dummy_PutPixel),
        DIB_GetPixel: Some(Dummy_GetPixel),
        DIB_HLine: Some(Dummy_HLine),
        DIB_VLine: Some(Dummy_VLine),
        DIB_BitBlt: Some(Dummy_BitBlt),
        DIB_BitBltSrcCopy: Some(Dummy_BitBlt),
        DIB_StretchBlt: Some(Dummy_StretchBlt),
        DIB_TransparentBlt: Some(Dummy_TransparentBlt),
        DIB_ColorFill: Some(Dummy_ColorFill),
        DIB_AlphaBlend: Some(Dummy_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(Dummy_PutPixel),
        DIB_GetPixel: Some(Dummy_GetPixel),
        DIB_HLine: Some(Dummy_HLine),
        DIB_VLine: Some(Dummy_VLine),
        DIB_BitBlt: Some(Dummy_BitBlt),
        DIB_BitBltSrcCopy: Some(Dummy_BitBlt),
        DIB_StretchBlt: Some(Dummy_StretchBlt),
        DIB_TransparentBlt: Some(Dummy_TransparentBlt),
        DIB_ColorFill: Some(Dummy_ColorFill),
        DIB_AlphaBlend: Some(Dummy_AlphaBlend),
    },
    DIB_FUNCTIONS {
        DIB_PutPixel: Some(Dummy_PutPixel),
        DIB_GetPixel: Some(Dummy_GetPixel),
        DIB_HLine: Some(Dummy_HLine),
        DIB_VLine: Some(Dummy_VLine),
        DIB_BitBlt: Some(Dummy_BitBlt),
        DIB_BitBltSrcCopy: Some(Dummy_BitBlt),
        DIB_StretchBlt: Some(Dummy_StretchBlt),
        DIB_TransparentBlt: Some(Dummy_TransparentBlt),
        DIB_ColorFill: Some(Dummy_ColorFill),
        DIB_AlphaBlend: Some(Dummy_AlphaBlend),
    },
];

#[no_mangle]
pub unsafe extern "C" fn DIB_DoRop(Rop: ULONG, Dest: ULONG, Source: ULONG, Pattern: ULONG) -> ULONG {
    let mut ResultNibble: ULONG;
    let mut Result: ULONG = 0;
    let mut i: ULONG;
    let ExpandDest: [ULONG; 16] = [
        0x55555555,
        0x555555AA,
        0x5555AA55,
        0x5555AAAA,
        0x55AA5555,
        0x55AA55AA,
        0x55AAAA55,
        0x55AAAAAA,
        0xAA555555,
        0xAA5555AA,
        0xAA55AA55,
        0xAA55AAAA,
        0xAAAA5555,
        0xAAAA55AA,
        0xAAAAAA55,
        0xAAAAAAAA,
    ];
    let ExpandSource: [ULONG; 16] = [
        0x33333333,
        0x333333CC,
        0x3333CC33,
        0x3333CCCC,
        0x33CC3333,
        0x33CC33CC,
        0x33CCCC33,
        0x33CCCCCC,
        0xCC333333,
        0xCC3333CC,
        0xCC33CC33,
        0xCC33CCCC,
        0xCCCC3333,
        0xCCCC33CC,
        0xCCCCCC33,
        0xCCCCCCCC,
    ];
    let ExpandPattern: [ULONG; 16] = [
        0x0F0F0F0F,
        0x0F0F0FF0,
        0x0F0FF00F,
        0x0F0FF0F0,
        0x0FF00F0F,
        0x0FF00FF0,
        0x0FF0F00F,
        0x0FF0F0F0,
        0xF00F0F0F,
        0xF00F0FF0,
        0xF00FF00F,
        0xF00FF0F0,
        0xF0F00F0F,
        0xF0F00FF0,
        0xF0F0F00F,
        0xF0F0F0F0,
    ];

    let mut Rop = Rop & 0xFF;
    match Rop {
        R3_OPINDEX_NOOP => return Dest,
        R3_OPINDEX_BLACKNESS => return 0,
        R3_OPINDEX_NOTSRCERASE => return !(Dest | Source),
        R3_OPINDEX_NOTSRCCOPY => return !Source,
        R3_OPINDEX_SRCERASE => return (!Dest) & Source,
        R3_OPINDEX_DSTINVERT => return !Dest,
        R3_OPINDEX_PATINVERT => return Dest ^ Pattern,
        R3_OPINDEX_SRCINVERT => return Dest ^ Source,
        R3_OPINDEX_SRCAND => return Dest & Source,
        R3_OPINDEX_MERGEPAINT => return Dest | (!Source),
        R3_OPINDEX_SRCPAINT => return Dest | Source,
        R3_OPINDEX_MERGECOPY => return Source & Pattern,
        R3_OPINDEX_SRCCOPY => return Source,
        R3_OPINDEX_PATCOPY => return Pattern,
        R3_OPINDEX_PATPAINT => return Dest | (!Source) | Pattern,
        R3_OPINDEX_WHITENESS => return 0xFFFFFFFF,
        _ => {}
    }

    Rop |= (Rop << 24) | (Rop << 16) | (Rop << 8);
    Result = 0;
    let mut Dest = Dest;
    let mut Source = Source;
    let mut Pattern = Pattern;
    i = 0;
    while i < 8 {
        ResultNibble = Rop & ExpandDest[(Dest & 0xF) as usize] & ExpandSource[(Source & 0xF) as usize] & ExpandPattern[(Pattern & 0xF) as usize];
        Result |= ((if ResultNibble & 0xFF000000 != 0 { 0x8 } else { 0x0 }) | 
                   (if ResultNibble & 0x00FF0000 != 0 { 0x4 } else { 0x0 }) |
                   (if ResultNibble & 0x0000FF00 != 0 { 0x2 } else { 0x0 }) | 
                   (if ResultNibble & 0x000000FF != 0 { 0x1 } else { 0x0 })) << (i * 4);
        Dest >>= 4;
        Source >>= 4;
        Pattern >>= 4;
        i += 1;
    }
    return Result;
}

pub unsafe extern "C" fn Dummy_PutPixel(SurfObj: *mut SURFOBJ, x: LONG, y: LONG, c: ULONG) {
    return;
}

unsafe extern "C" fn Dummy_GetPixel(SurfObj: *mut SURFOBJ, x: LONG, y: LONG) -> ULONG {
    return 0;
}

unsafe extern "C" fn Dummy_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    return;
}

unsafe extern "C" fn Dummy_VLine(SurfObj: *mut SURFOBJ, x: i32, y1: i32, y2: i32, c: u32) {
    return;
}

unsafe extern "C" fn Dummy_BitBlt(BltInfo: PBLTINFO) -> BOOLEAN {
    0u8
}

unsafe extern "C" fn Dummy_StretchBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    PatternSurface: *mut SURFOBJ,
    MaskSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    MaskOrigin: *mut POINTL,
    Brush: *mut BRUSHOBJ,
    BrushOrign: *mut POINTL,
    ColorTranslation: *mut XLATEOBJ,
    Rop: ROP4,
) -> u8 {
    0u8
}

unsafe extern "C" fn Dummy_TransparentBlt(
    DestSurf: *mut SURFOBJ,
    SourceSurf: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ColorTranslation: *mut XLATEOBJ,
    iTransColor: u32,
) -> u8 {
    0u8
}

unsafe extern "C" fn Dummy_ColorFill(Dest: *mut SURFOBJ, DestRect: *mut RECTL, Color: u32) -> u8 {
    0u8
}

unsafe extern "C" fn Dummy_AlphaBlend(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    BlendObj: *mut BLENDOBJ,
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
//  * FILE:            win32ss/gdi/dib/dib.c
//  * PURPOSE:         ROP handling, function pointer arrays, misc
//  * PROGRAMMERS:     Ge van Geldorp
//  */
//
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* Static data */
//
// unsigned char notmask[2] = { 0x0f, 0xf0 };
// unsigned char altnotmask[2] = { 0xf0, 0x0f };
//
// DIB_FUNCTIONS DibFunctionsForBitmapFormat[] =
// {
//   /* 0 */
//   {
//     Dummy_PutPixel, Dummy_GetPixel, Dummy_HLine, Dummy_VLine,
//     Dummy_BitBlt, Dummy_BitBlt, Dummy_StretchBlt, Dummy_TransparentBlt,
//     Dummy_ColorFill, Dummy_AlphaBlend
//   },
//   /* BMF_1BPP */
//   {
//     DIB_1BPP_PutPixel, DIB_1BPP_GetPixel, DIB_1BPP_HLine, DIB_1BPP_VLine,
//     DIB_1BPP_BitBlt, DIB_1BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_1BPP_TransparentBlt, DIB_1BPP_ColorFill, DIB_XXBPP_AlphaBlend
//   },
//   /* BMF_4BPP */
//   {
//     DIB_4BPP_PutPixel, DIB_4BPP_GetPixel, DIB_4BPP_HLine, DIB_4BPP_VLine,
//     DIB_4BPP_BitBlt, DIB_4BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_4BPP_TransparentBlt, DIB_4BPP_ColorFill, DIB_XXBPP_AlphaBlend
//   },
//   /* BMF_8BPP */
//   {
//     DIB_8BPP_PutPixel, DIB_8BPP_GetPixel, DIB_8BPP_HLine, DIB_8BPP_VLine,
//     DIB_8BPP_BitBlt, DIB_8BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_8BPP_TransparentBlt, DIB_8BPP_ColorFill, DIB_XXBPP_AlphaBlend
//   },
//   /* BMF_16BPP */
//   {
//     DIB_16BPP_PutPixel, DIB_16BPP_GetPixel, DIB_16BPP_HLine, DIB_16BPP_VLine,
//     DIB_16BPP_BitBlt, DIB_16BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_16BPP_TransparentBlt, DIB_16BPP_ColorFill, DIB_16BPP_AlphaBlend
//   },
//   /* BMF_24BPP */
//   {
//     DIB_24BPP_PutPixel, DIB_24BPP_GetPixel, DIB_24BPP_HLine, DIB_24BPP_VLine,
//     DIB_24BPP_BitBlt, DIB_24BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_24BPP_TransparentBlt, DIB_24BPP_ColorFill, DIB_24BPP_AlphaBlend
//   },
//   /* BMF_32BPP */
//   {
//     DIB_32BPP_PutPixel, DIB_32BPP_GetPixel, DIB_32BPP_HLine, DIB_32BPP_VLine,
//     DIB_32BPP_BitBlt, DIB_32BPP_BitBltSrcCopy, DIB_XXBPP_StretchBlt,
//     DIB_32BPP_TransparentBlt, DIB_32BPP_ColorFill, DIB_32BPP_AlphaBlend
//   },
//   /* BMF_4RLE */
//   {
//     Dummy_PutPixel, Dummy_GetPixel, Dummy_HLine, Dummy_VLine,
//     Dummy_BitBlt, Dummy_BitBlt, Dummy_StretchBlt, Dummy_TransparentBlt,
//     Dummy_ColorFill, Dummy_AlphaBlend
//   },
//   /* BMF_8RLE */
//   {
//     Dummy_PutPixel, Dummy_GetPixel, Dummy_HLine, Dummy_VLine,
//     Dummy_BitBlt, Dummy_BitBlt, Dummy_StretchBlt, Dummy_TransparentBlt,
//     Dummy_ColorFill, Dummy_AlphaBlend
//   },
//   /* BMF_JPEG */
//   {
//     Dummy_PutPixel, Dummy_GetPixel, Dummy_HLine, Dummy_VLine,
//     Dummy_BitBlt, Dummy_BitBlt, Dummy_StretchBlt, Dummy_TransparentBlt,
//     Dummy_ColorFill, Dummy_AlphaBlend
//   },
//   /* BMF_PNG */
//   {
//     Dummy_PutPixel, Dummy_GetPixel, Dummy_HLine, Dummy_VLine,
//     Dummy_BitBlt, Dummy_BitBlt, Dummy_StretchBlt, Dummy_TransparentBlt,
//     Dummy_ColorFill, Dummy_AlphaBlend
//   }
// };
//
//
// ULONG
// DIB_DoRop(ULONG Rop, ULONG Dest, ULONG Source, ULONG Pattern)
// {
//   ULONG ResultNibble;
//   ULONG Result = 0;
//   ULONG i;
// static const ULONG ExpandDest[16] =
//     {
//       0x55555555 /* 0000 */,
//       0x555555AA /* 0001 */,
//       0x5555AA55 /* 0010 */,
//       0x5555AAAA /* 0011 */,
//       0x55AA5555 /* 0100 */,
//       0x55AA55AA /* 0101 */,
//       0x55AAAA55 /* 0110 */,
//       0x55AAAAAA /* 0111 */,
//       0xAA555555 /* 1000 */,
//       0xAA5555AA /* 1001 */,
//       0xAA55AA55 /* 1010 */,
//       0xAA55AAAA /* 1011 */,
//       0xAAAA5555 /* 1100 */,
//       0xAAAA55AA /* 1101 */,
//       0xAAAAAA55 /* 1110 */,
//       0xAAAAAAAA /* 1111 */,
//     };
//   static const ULONG ExpandSource[16] =
//     {
//       0x33333333 /* 0000 */,
//       0x333333CC /* 0001 */,
//       0x3333CC33 /* 0010 */,
//       0x3333CCCC /* 0011 */,
//       0x33CC3333 /* 0100 */,
//       0x33CC33CC /* 0101 */,
//       0x33CCCC33 /* 0110 */,
//       0x33CCCCCC /* 0111 */,
//       0xCC333333 /* 1000 */,
//       0xCC3333CC /* 1001 */,
//       0xCC33CC33 /* 1010 */,
//       0xCC33CCCC /* 1011 */,
//       0xCCCC3333 /* 1100 */,
//       0xCCCC33CC /* 1101 */,
//       0xCCCCCC33 /* 1110 */,
//       0xCCCCCCCC /* 1111 */,
//     };
//   static const ULONG ExpandPattern[16] =
//     {
//       0x0F0F0F0F /* 0000 */,
//       0x0F0F0FF0 /* 0001 */,
//       0x0F0FF00F /* 0010 */,
//       0x0F0FF0F0 /* 0011 */,
//       0x0FF00F0F /* 0100 */,
//       0x0FF00FF0 /* 0101 */,
//       0x0FF0F00F /* 0110 */,
//       0x0FF0F0F0 /* 0111 */,
//       0xF00F0F0F /* 1000 */,
//       0xF00F0FF0 /* 1001 */,
//       0xF00FF00F /* 1010 */,
//       0xF00FF0F0 /* 1011 */,
//       0xF0F00F0F /* 1100 */,
//       0xF0F00FF0 /* 1101 */,
//       0xF0F0F00F /* 1110 */,
//       0xF0F0F0F0 /* 1111 */,
//     };
//
//     Rop &=0xFF;
//     switch(Rop)
//     {
//
//         /* Optimized code for the various named rop codes. */
//         case R3_OPINDEX_NOOP:        return(Dest);
//         case R3_OPINDEX_BLACKNESS:   return(0);
//         case R3_OPINDEX_NOTSRCERASE: return(~(Dest | Source));
//         case R3_OPINDEX_NOTSRCCOPY:  return(~Source);
//         case R3_OPINDEX_SRCERASE:    return((~Dest) & Source);
//         case R3_OPINDEX_DSTINVERT:   return(~Dest);
//         case R3_OPINDEX_PATINVERT:   return(Dest ^ Pattern);
//         case R3_OPINDEX_SRCINVERT:   return(Dest ^ Source);
//         case R3_OPINDEX_SRCAND:      return(Dest & Source);
//         case R3_OPINDEX_MERGEPAINT:  return(Dest | (~Source));
//         case R3_OPINDEX_SRCPAINT:    return(Dest | Source);
//         case R3_OPINDEX_MERGECOPY:   return(Source & Pattern);
//         case R3_OPINDEX_SRCCOPY:     return(Source);
//         case R3_OPINDEX_PATCOPY:     return(Pattern);
//         case R3_OPINDEX_PATPAINT:    return(Dest | (~Source) | Pattern);
//         case R3_OPINDEX_WHITENESS:   return(0xFFFFFFFF);
//     }
//
//   /* Expand the ROP operation to all four bytes */
//   Rop |= (Rop << 24) | (Rop << 16) | (Rop << 8);
//   /* Do the operation on four bits simultaneously. */
//   Result = 0;
//   for (i = 0; i < 8; i++)
//   {
//     ResultNibble = Rop & ExpandDest[Dest & 0xF] & ExpandSource[Source & 0xF] & ExpandPattern[Pattern & 0xF];
//     Result |= (((ResultNibble & 0xFF000000) ? 0x8 : 0x0) | ((ResultNibble & 0x00FF0000) ? 0x4 : 0x0) |
//     ((ResultNibble & 0x0000FF00) ? 0x2 : 0x0) | ((ResultNibble & 0x000000FF) ? 0x1 : 0x0)) << (i * 4);
//     Dest >>= 4;
//     Source >>= 4;
//     Pattern >>= 4;
//   }
//   return(Result);
// }
//
// VOID Dummy_PutPixel(SURFOBJ* SurfObj, LONG x, LONG y, ULONG c)
// {
//   return;
// }
//
// ULONG Dummy_GetPixel(SURFOBJ* SurfObj, LONG x, LONG y)
// {
//   return 0;
// }
//
// VOID Dummy_HLine(SURFOBJ* SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   return;
// }
//
// VOID Dummy_VLine(SURFOBJ* SurfObj, LONG x, LONG y1, LONG y2, ULONG c)
// {
//   return;
// }
//
// BOOLEAN Dummy_BitBlt(PBLTINFO BltInfo)
// {
//   return FALSE;
// }
//
// BOOLEAN Dummy_StretchBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                          SURFOBJ *PatternSurface, SURFOBJ *MaskSurf,
//                          RECTL*  DestRect,  RECTL  *SourceRect,
//                          POINTL* MaskOrigin, BRUSHOBJ* Brush,
//                          POINTL* BrushOrign,
//                          XLATEOBJ *ColorTranslation,
//                          ROP4 Rop)
// {
//   return FALSE;
// }
//
// BOOLEAN Dummy_TransparentBlt(SURFOBJ *DestSurf, SURFOBJ *SourceSurf,
//                              RECTL*  DestRect,  RECTL  *SourceRect,
//                              XLATEOBJ *ColorTranslation, ULONG iTransColor)
// {
//   return FALSE;
// }
//
// BOOLEAN Dummy_ColorFill(SURFOBJ* Dest, RECTL* DestRect, ULONG Color)
// {
//   return FALSE;
// }
//
//
// BOOLEAN
// Dummy_AlphaBlend(SURFOBJ* Dest, SURFOBJ* Source, RECTL* DestRect,
//                  RECTL* SourceRect, CLIPOBJ* ClipRegion,
//                  XLATEOBJ* ColorTranslation, BLENDOBJ* BlendObj)
// {
//   return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
