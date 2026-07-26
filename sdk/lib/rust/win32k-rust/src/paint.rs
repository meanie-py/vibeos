use win32k_sys::win32k::*;

// `gajRop2ToRop3` was defined in `win32ss/gdi/eng/paint.c:15` as
// `const BYTE gajRop2ToRop3[16] = {...}`. paint.c is now cohabitation-commented,
// so paint.rs OWNS the symbol — bindgen extern would not resolve at link.
// Other still-C consumers (eng.h MIX_TO_ROP4 macro inlined at every site) and
// in-rlib consumers (mix_to_rop4 below) both bind to this `#[no_mangle]` static.
#[no_mangle]
pub static gajRop2ToRop3: [BYTE; 16] = [
    0x00, //  1: R2_BLACK        0
    0x05, //  2: R2_NOTMERGEPEN  DPon
    0x0A, //  3: R2_MASKNOTPEN   DPna
    0x0F, //  4: R2_NOTCOPYPEN   Pn
    0x50, //  5: R2_MASKPENNOT   PDna
    0x55, //  6: R2_NOT          Dn
    0x5A, //  7: R2_XORPEN       DPx
    0x5F, //  8: R2_NOTMASKPEN   DPan
    0xA0, //  9: R2_MASKPEN      DPa
    0xA5, // 10: R2_NOTXORPEN    PDxn
    0xAA, // 11: R2_NOP          D
    0xAF, // 12: R2_MERGENOTPEN  DPno
    0xF0, // 13: R2_COPYPEN      P
    0xF5, // 14: R2_MERGEPENNOT  PDno
    0xFA, // 15: R2_MERGEPEN     DPo
    0xFF, // 16: R2_WHITE        1
];

// Inline-expand of `MIX_TO_ROP4` from `win32ss/gdi/eng/eng.h:5`:
//   ((ULONG)gajRop2ToRop3[(mix - 1) & 0xF] |
//    ((ULONG)gajRop2ToRop3[((mix >> 8) - 1) & 0xF] << 8))
#[inline(always)]
unsafe fn mix_to_rop4(mix: MIX) -> ROP4 {
    let lo_idx = ((mix.wrapping_sub(1)) & 0xF) as usize;
    let hi_idx = (((mix >> 8).wrapping_sub(1)) & 0xF) as usize;
    let lo = gajRop2ToRop3[lo_idx] as ULONG;
    let hi = gajRop2ToRop3[hi_idx] as ULONG;
    (lo | (hi << 8)) as ROP4
}

// Inline-expand of `ROP4_USES_SOURCE` from `win32ss/gdi/eng/inteng.h:45`:
//   ((((Rop4) & 0xCCCC) >> 2) != ((Rop4) & 0x3333))
#[inline(always)]
fn rop4_uses_source(rop4: ROP4) -> bool {
    ((rop4 & 0xCCCC) >> 2) != (rop4 & 0x3333)
}

// Inline-expand of `GDIDEVFUNCS(SurfObj)` from `win32ss/gdi/eng/surface.h:106`:
//   ((PDEVOBJ *)((SurfObj)->hdev))->DriverFunctions
// In bindgen, `DriverFunctions` is the first field of the `_PDEVOBJ`
// anonymous union `__bindgen_anon_1`. Returns a raw pointer to the union
// field so the caller can fetch any `PFN_Drv*` member.
#[inline(always)]
unsafe fn gdidev_paint(pso: *mut SURFOBJ) -> PFN_DrvPaint {
    let pdev = (*pso).hdev as *mut PDEVOBJ;
    (*pdev).__bindgen_anon_1.DriverFunctions.Paint
}

#[no_mangle]
pub unsafe extern "system" fn FillSolid(
    pso: *mut SURFOBJ,
    pRect: PRECTL,
    iColor: ULONG,
) -> BOOL {
    let mut y: LONG;
    let _LineWidth: ULONG;

    _LineWidth = ((*pRect).right - (*pRect).left) as ULONG;
    y = (*pRect).top;
    while y < (*pRect).bottom {
        // `DibFunctionsForBitmapFormat` is a flexible-array extern (`[T; 0]`
        // in bindgen). Use pointer arithmetic to index it; `DIB_HLine` is an
        // `Option<unsafe extern fn>` which must be unwrapped before call.
        let base = core::ptr::addr_of_mut!(DibFunctionsForBitmapFormat) as *mut DIB_FUNCTIONS;
        let entry = base.add((*pso).iBitmapFormat as usize);
        if let Some(hline) = (*entry).DIB_HLine {
            hline(
                pso,
                (*pRect).left,
                (*pRect).right,
                y,
                iColor,
            );
        }
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
    let rop4: ROP4 = mix_to_rop4(mix);
    ntkrnl_sys::ASSERT!(!rop4_uses_source(rop4));

    IntEngBitBlt(
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
    )
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
    let rop4: ROP4 = mix_to_rop4(mix);
    ntkrnl_sys::ASSERT!(!rop4_uses_source(rop4));

    IntEngBitBlt(
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
    )
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
    let psurf: *mut SURFACE =
        (pso as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if ((*pso).iType as u32 != STYPE_BITMAP) && ((*psurf).flags & HOOK_PAINT as FLONG != 0) {
        if let Some(paint) = gdidev_paint(pso) {
            return paint(pso, pco, pbo, pptlBrushOrg, mix);
        }
    }

    EngPaint(pso, pco, pbo, pptlBrushOrg, mix)
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
    let psurf: *mut SURFACE =
        (pso as *mut u8).offset(-(core::mem::offset_of!(SURFACE, SurfObj) as isize)) as *mut SURFACE;

    if ((*pso).iType as u32 != STYPE_BITMAP) && ((*psurf).flags & HOOK_PAINT as FLONG != 0) {
        if let Some(paint) = gdidev_paint(pso) {
            return paint(pso, pco, pbo, pptlBrushOrg, mix);
        }
    }

    EngPaint(pso, pco, pbo, pptlBrushOrg, mix)
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
