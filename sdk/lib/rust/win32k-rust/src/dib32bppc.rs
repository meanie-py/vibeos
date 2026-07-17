use win32k_sys::win32k::*;
use crate::dib_helpers::*;

extern "system" {
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble.
// The #define NDEBUG is a preprocessor directive that affects compilation
// but does not create any runtime constants or types.

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_HLine(SurfObj: *mut SURFOBJ, x1: i32, x2: i32, y: i32, c: u32) {
    let byteaddr: PBYTE = ((*SurfObj).pvScan0 as usize + (y as usize) * ((*SurfObj).lDelta as usize)) as PBYTE;
    let mut addr: PDWORD = (byteaddr as PDWORD).offset(x1 as isize);
    let mut cx: i32 = x1;

    while cx < x2 {
        *addr = c as u32;
        addr = addr.offset(1);
        cx += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_32BPP_ColorFill(DestSurface: *mut SURFOBJ, DestRect: *mut RECTL, color: u32) -> u8 {

    RECTL_vMakeWellOrdered(DestRect);

    let mut DestY: u32 = (*DestRect).top as u32;
    while DestY < (*DestRect).bottom as u32 {
        DIB_32BPP_HLine(DestSurface, (*DestRect).left, (*DestRect).right, DestY as i32, color);
        DestY += 1;
    }

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         Win32 subsystem
//  * LICENSE:         See COPYING in the top level directory
//  * FILE:            win32ss/gdi/dib/dib32bppc.c
//  * PURPOSE:         C language equivalents of asm optimised 32bpp functions
//  * PROGRAMMERS:     Jason Filby
//  *                  Magnus Olsen
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// DIB_32BPP_HLine(SURFOBJ *SurfObj, LONG x1, LONG x2, LONG y, ULONG c)
// {
//   PBYTE byteaddr = (PBYTE)((ULONG_PTR)SurfObj->pvScan0 + y * SurfObj->lDelta);
//   PDWORD addr = (PDWORD)byteaddr + x1;
//   LONG cx = x1;
//
//   while(cx < x2)
//   {
//     *addr = (DWORD)c;
//     ++addr;
//     ++cx;
//   }
// }
//
// BOOLEAN
// DIB_32BPP_ColorFill(SURFOBJ* DestSurface, RECTL* DestRect, ULONG color)
// {
//   ULONG DestY;
//
//   /* Make WellOrdered by making top < bottom and left < right */
//   RECTL_vMakeWellOrdered(DestRect);
//
//   for (DestY = DestRect->top; DestY< DestRect->bottom; DestY++)
//   {
//     DIB_32BPP_HLine (DestSurface, DestRect->left, DestRect->right, DestY, color);
//   }
//
//   return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
