use win32k_sys::win32k::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const gajRop2ToRop3: [u8; 16] = [
    0x00,
    0x05,
    0x0A,
    0x0F,
    0x50,
    0x55,
    0x5A,
    0x5F,
    0xA0,
    0xA5,
    0xAA,
    0xAF,
    0xF0,
    0xF5,
    0xFA,
    0xFF,
];

pub unsafe extern "system" fn FillSolid(pso: *mut SURFOBJ, pRect: PRECTL, iColor: u32) -> i32 {
    let y: i32;
    let LineWidth: u32;

    LineWidth = ((*pRect).right - (*pRect).left) as u32;
    
    y = (*pRect).top;
    while y < (*pRect).bottom {
        (DibFunctionsForBitmapFormat[(*pso).iBitmapFormat as usize].DIB_HLine)(
            pso, (*pRect).left, (*pRect).right, y, iColor);
        y += 1;
    }
    
    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngPaint(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
) -> BOOL {
    let rop4: ROP4;

    rop4 = MIX_TO_ROP4(mix);

    NT_ASSERT(!ROP4_USES_SOURCE(rop4));

    return IntEngBitBlt(
        pso,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pco,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pco).rclBounds),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pbo,
        pptlBrushOrg,
        rop4,
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngPaint(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
) -> BOOL {
    let rop4: ROP4;

    rop4 = MIX_TO_ROP4(mix);

    NT_ASSERT(!ROP4_USES_SOURCE(rop4));

    return IntEngBitBlt(
        pso,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pco,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*pco).rclBounds),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pbo,
        pptlBrushOrg,
        rop4,
    );
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn IntEngPaint(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
) -> BOOL {
    let psurf = (pso as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if ((*pso).iType != STYPE_BITMAP) && (((*psurf).flags & HOOK_PAINT) != 0) {
        return GDIDEVFUNCS(pso).Paint.unwrap()(pso, pco, pbo, pptlBrushOrg, mix);
    }

    return EngPaint(pso, pco, pbo, pptlBrushOrg, mix);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngPaint(
    pso: *mut SURFOBJ,
    pco: *mut CLIPOBJ,
    pbo: *mut BRUSHOBJ,
    pptlBrushOrg: *mut POINTL,
    mix: MIX,
) -> BOOL {
    let psurf = (pso as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if ((*pso).iType != STYPE_BITMAP) && (((*psurf).flags & HOOK_PAINT) != 0) {
        return GDIDEVFUNCS(pso).Paint.unwrap()(pso, pco, pbo, pptlBrushOrg, mix);
    }

    return EngPaint(pso, pco, pbo, pptlBrushOrg, mix);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Driver Paint Functions
//  * FILE:              win32ss/gdi/eng/paint.c
//  * PROGRAMERS:        Timo Kreuzer (timo.kreuzer@reactos.org)
//  *                    Jason Filby
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// const BYTE gajRop2ToRop3[16] =
// {
//     0x00, //  1: R2_BLACK        0
//     0x05, //  2: R2_NOTMERGEPEN  DPon
//     0x0A, //  3: R2_MASKNOTPEN   DPna
//     0x0F, //  4: R2_NOTCOPYPEN   Pn
//     0x50, //  5: R2_MASKPENNOT   PDna
//     0x55, //  6: R2_NOT          Dn
//     0x5A, //  7: R2_XORPEN       DPx
//     0x5F, //  8: R2_NOTMASKPEN   DPan
//     0xA0, //  9: R2_MASKPEN      DPa
//     0xA5, // 10: R2_NOTXORPEN    PDxn
//     0xAA, // 11: R2_NOP          D
//     0xAF, // 12: R2_MERGENOTPEN  DPno
//     0xF0, // 13: R2_COPYPEN      P
//     0xF5, // 14: R2_MERGEPENNOT  PDno
//     0xFA, // 15: R2_MERGEPEN     DPo
//     0xFF, // 16: R2_WHITE        1
// };
//
// BOOL APIENTRY FillSolid(SURFOBJ *pso, PRECTL pRect, ULONG iColor)
// {
//   LONG y;
//   ULONG LineWidth;
//
//   ASSERT(pso);
//   ASSERT(pRect);
//   LineWidth  = pRect->right - pRect->left;
//   DPRINT(" LineWidth: %lu, top: %ld, bottom: %ld\n", LineWidth, pRect->top, pRect->bottom);
//   for (y = pRect->top; y < pRect->bottom; y++)
//   {
//     DibFunctionsForBitmapFormat[pso->iBitmapFormat].DIB_HLine(
//       pso, pRect->left, pRect->right, y, iColor);
//   }
//   return TRUE;
// }
//
// BOOL
// APIENTRY
// EngPaint(
//     _In_ SURFOBJ *pso,
//     _In_ CLIPOBJ *pco,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ __in_data_source(USER_MODE) MIX mix)
// {
//     ROP4 rop4;
//
//     /* Convert the MIX, consisting of 2 ROP2 codes into a ROP4 */
//     rop4 = MIX_TO_ROP4(mix);
//
//     /* Sanity check */
//     NT_ASSERT(!ROP4_USES_SOURCE(rop4));
//
//     /* Forward the call to Eng/DrvBitBlt */
//     return IntEngBitBlt(pso,
//                         NULL,
//                         NULL,
//                         pco,
//                         NULL,
//                         &pco->rclBounds,
//                         NULL,
//                         NULL,
//                         pbo,
//                         pptlBrushOrg,
//                         rop4);
// }
//
// BOOL
// APIENTRY
// IntEngPaint(
//     _In_ SURFOBJ *pso,
//     _In_ CLIPOBJ *pco,
//     _In_ BRUSHOBJ *pbo,
//     _In_ POINTL *pptlBrushOrg,
//     _In_ __in_data_source(USER_MODE) MIX mix)
// {
//     SURFACE *psurf = CONTAINING_RECORD(pso, SURFACE, SurfObj);
//
//     /* Is the surface's Paint function hooked? */
//     if ((pso->iType != STYPE_BITMAP) && (psurf->flags & HOOK_PAINT))
//     {
//         /* Call the driver's DrvPaint */
//         return GDIDEVFUNCS(pso).Paint(pso, pco, pbo, pptlBrushOrg, mix);
//     }
//
//     return EngPaint(pso, pco, pbo, pptlBrushOrg, mix);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
