use win32k_sys::win32k::*;
use crate::dib_helpers::*;

// RECTL_bPointInRect is FORCEINLINE in rect.h (no exported symbol) — inline it.
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: INT, y: INT) -> BOOL {
    (x >= (*prcl).left && x < (*prcl).right && y >= (*prcl).top && y < (*prcl).bottom) as BOOL
}
// DibFunctionsForBitmapFormat (defined #[no_mangle] in dib.rs) — declared with
// the real [_; 11] layout so indexing uses the correct stride (the win32k_sys
// bindgen glob is `[DIB_FUNCTIONS; 0]`, useless). dib_helpers cross-module pattern.
extern "C" {
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 11];
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _floodItem {
    pub x: ULONG,
    pub y: ULONG,
}

pub type FLOODITEM = _floodItem;

#[repr(C)]
pub struct _floodInfo {
    pub floodLen: ULONG,
    pub floodStart: *mut FLOODITEM,
    pub floodData: *mut FLOODITEM,
}

pub type FLOODINFO = _floodInfo;

#[inline(always)]

unsafe fn initFlood(info: *mut FLOODINFO, DstRect: *mut RECTL) -> u8 {
    let width = (*DstRect).right - (*DstRect).left;
    let height = (*DstRect).bottom - (*DstRect).top;
    (*info).floodData = ExAllocatePoolWithTag(
        NonPagedPool,
        (width as u32 * height as u32 * core::mem::size_of::<FLOODITEM>() as u32) as SIZE_T,
        0x20424944u32,
    ) as *mut FLOODITEM;
    if (*info).floodData.is_null() {
        return 0u8;
    }
    (*info).floodStart = (*info).floodData.add((width * height) as usize);
    1u8
}

unsafe fn finalizeFlood(info: *mut FLOODINFO) {
    ExFreePoolWithTag((*info).floodData as *mut core::ffi::c_void, TAG_DIB);
}

unsafe fn addItemFlood(
    info: *mut FLOODINFO,
    x: ULONG,
    y: ULONG,
    DstSurf: *mut SURFOBJ,
    DstRect: *mut RECTL,
    Color: ULONG,
    isSurf: BOOL,
) {

    if RECTL_bPointInRect(DstRect, x as INT, y as INT) != 0 {
        if isSurf != 0
            && DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize]
                .DIB_GetPixel
                .unwrap()(DstSurf, x as LONG, y as LONG)
                != Color
        {
            return;
        } else if isSurf == 0
            && DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize]
                .DIB_GetPixel
                .unwrap()(DstSurf, x as LONG, y as LONG)
                == Color
        {
            return;
        }
        (*info).floodStart = (*info).floodStart.offset(-1);
        (*(*info).floodStart).x = x;
        (*(*info).floodStart).y = y;
        (*info).floodLen = (*info).floodLen.wrapping_add(1);
    }
}

unsafe fn removeItemFlood(info: *mut FLOODINFO) {
    (*info).floodStart = (*info).floodStart.offset(1);
    (*info).floodLen = (*info).floodLen.wrapping_sub(1);
}

#[no_mangle]
pub unsafe extern "C" fn DIB_XXBPP_FloodFillSolid(
    DstSurf: *mut SURFOBJ,
    Brush: *mut BRUSHOBJ,
    DstRect: *mut RECTL,
    Origin: *mut POINTL,
    ConvColor: ULONG,
    FillType: UINT,
) -> BOOLEAN {
    let mut x: ULONG;
    let mut y: ULONG;
    let BrushColor: ULONG;
    let mut flood: FLOODINFO = FLOODINFO {
        floodLen: 0,
        floodStart: core::ptr::null_mut(),
        floodData: core::ptr::null_mut(),
    };

    BrushColor = (*Brush).iSolidColor;
    x = (*Origin).x as ULONG;
    y = (*Origin).y as ULONG;

    if FillType == FLOODFILLBORDER {
        if DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize].DIB_GetPixel.unwrap()(DstSurf, x as i32, y as i32) == ConvColor {
            return 0u8;
        }

        if initFlood(&mut flood, DstRect) == 0u8 {
            return 0u8;
        }
        addItemFlood(&mut flood, x, y, DstSurf, DstRect, ConvColor, 0);
        while flood.floodLen != 0 {
            x = (*flood.floodStart).x;
            y = (*flood.floodStart).y;
            removeItemFlood(&mut flood);

            DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize].DIB_PutPixel.unwrap()(DstSurf, x as i32, y as i32, BrushColor);
            if flood.floodStart.offset(-4) < flood.floodData {
                finalizeFlood(&mut flood);
                return 0u8;
            }
            addItemFlood(&mut flood, x, y + 1, DstSurf, DstRect, ConvColor, 0);
            addItemFlood(&mut flood, x, y.wrapping_sub(1), DstSurf, DstRect, ConvColor, 0);
            addItemFlood(&mut flood, x + 1, y, DstSurf, DstRect, ConvColor, 0);
            addItemFlood(&mut flood, x.wrapping_sub(1), y, DstSurf, DstRect, ConvColor, 0);
        }
        finalizeFlood(&mut flood);
    } else if FillType == FLOODFILLSURFACE {
        if DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize].DIB_GetPixel.unwrap()(DstSurf, x as i32, y as i32) != ConvColor {
            return 0u8;
        }

        if initFlood(&mut flood, DstRect) == 0u8 {
            return 0u8;
        }
        addItemFlood(&mut flood, x, y, DstSurf, DstRect, ConvColor, 1);
        while flood.floodLen != 0 {
            x = (*flood.floodStart).x;
            y = (*flood.floodStart).y;
            removeItemFlood(&mut flood);

            DibFunctionsForBitmapFormat[(*DstSurf).iBitmapFormat as usize].DIB_PutPixel.unwrap()(DstSurf, x as i32, y as i32, BrushColor);
            if flood.floodStart.offset(-4) < flood.floodData {
                finalizeFlood(&mut flood);
                return 0u8;
            }
            addItemFlood(&mut flood, x, y + 1, DstSurf, DstRect, ConvColor, 1);
            addItemFlood(&mut flood, x, y.wrapping_sub(1), DstSurf, DstRect, ConvColor, 1);
            addItemFlood(&mut flood, x + 1, y, DstSurf, DstRect, ConvColor, 1);
            addItemFlood(&mut flood, x.wrapping_sub(1), y, DstSurf, DstRect, ConvColor, 1);
        }
        finalizeFlood(&mut flood);
    } else {
        return 0u8;
    }
    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// * COPYRIGHT:         See COPYING in the top level directory
// * PROJECT:           ReactOS win32 subsystem
// * PURPOSE:           Flood filling support
// * FILE:              win32ss/gdi/dib/floodfill.c
// * PROGRAMMER:        Gregor Schneider <grschneider AT gmail DOT com>
// */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /*
// *  This floodfill algorithm is an iterative four neighbors version. It works with an internal stack like data structure.
// *  The stack is kept in an array, sized for the worst case scenario of having to add all pixels of the surface.
// *  This avoids having to allocate and free memory blocks  all the time. The stack grows from the end of the array towards the start.
// *  All pixels are checked before being added, against belonging to the fill rule (FLOODFILLBORDER or FLOODFILLSURFACE)
// *  and the position in respect to the clip region. This guarantees all pixels lying on the stack belong to the filled surface.
// *  Further optimisations of the algorithm are possible.
// */
//
// /* Floodfil helper structures and functions */
// typedef struct _floodItem
// {
//   ULONG x;
//   ULONG y;
// } FLOODITEM;
//
// typedef struct _floodInfo
// {
//   ULONG floodLen;
//   FLOODITEM *floodStart;
//   FLOODITEM *floodData;
// } FLOODINFO;
//
// static __inline BOOL initFlood(FLOODINFO *info, RECTL *DstRect)
// {
//   ULONG width = DstRect->right - DstRect->left;
//   ULONG height = DstRect->bottom - DstRect->top;
//   info->floodData = ExAllocatePoolWithTag(NonPagedPool, width * height * sizeof(FLOODITEM), TAG_DIB);
//   if (info->floodData == NULL)
//   {
//     return FALSE;
//   }
//   info->floodStart = info->floodData + (width * height);
//   DPRINT("Allocated flood stack from %p to %p\n", info->floodData, info->floodStart);
//   return TRUE;
// }
// static __inline VOID finalizeFlood(FLOODINFO *info)
// {
//   ExFreePoolWithTag(info->floodData, TAG_DIB);
// }
// static __inline VOID addItemFlood(FLOODINFO *info,
//                                   ULONG x,
//                                   ULONG y,
//                                   SURFOBJ *DstSurf,
//                                   RECTL *DstRect,
//                                   ULONG Color,
//                                   BOOL isSurf)
// {
//   if (RECTL_bPointInRect(DstRect,x,y))
//   {
//     if (isSurf &&
//       DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_GetPixel(DstSurf, x, y) != Color)
//     {
//       return;
//     }
//     else if (isSurf == FALSE &&
//       DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_GetPixel(DstSurf, x, y) == Color)
//     {
//       return;
//     }
//     info->floodStart--;
//     info->floodStart->x = x;
//     info->floodStart->y = y;
//     info->floodLen++;
//   }
// }
// static __inline VOID removeItemFlood(FLOODINFO *info)
// {
//   info->floodStart++;
//   info->floodLen--;
// }
//
// BOOLEAN DIB_XXBPP_FloodFillSolid(SURFOBJ *DstSurf,
//                                  BRUSHOBJ *Brush,
//                                  RECTL *DstRect,
//                                  POINTL *Origin,
//                                  ULONG ConvColor,
//                                  UINT FillType)
// {
//   ULONG x, y;
//   ULONG BrushColor;
//   FLOODINFO flood = {0, NULL, NULL};
//
//   BrushColor = Brush->iSolidColor;
//   x = Origin->x;
//   y = Origin->y;
//
//   if (FillType == FLOODFILLBORDER)
//   {
//     /* Check if the start pixel has the border color */
//     if (DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_GetPixel(DstSurf, x, y) == ConvColor)
//     {
//       return FALSE;
//     }
//
//     if (initFlood(&flood, DstRect) == FALSE)
//     {
//       return FALSE;
//     }
//     addItemFlood(&flood, x, y, DstSurf, DstRect, ConvColor, FALSE);
//     while (flood.floodLen != 0)
//     {
//       x = flood.floodStart->x;
//       y = flood.floodStart->y;
//       removeItemFlood(&flood);
//
//       DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_PutPixel(DstSurf, x, y, BrushColor);
//       if (flood.floodStart - 4 < flood.floodData)
//       {
//         DPRINT1("Can't finish flooding!\n");
//         finalizeFlood(&flood);
//         return FALSE;
//       }
//       addItemFlood(&flood, x, y + 1, DstSurf, DstRect, ConvColor, FALSE);
//       addItemFlood(&flood, x, y - 1, DstSurf, DstRect, ConvColor, FALSE);
//       addItemFlood(&flood, x + 1, y, DstSurf, DstRect, ConvColor, FALSE);
//       addItemFlood(&flood, x - 1, y, DstSurf, DstRect, ConvColor, FALSE);
//     }
//     finalizeFlood(&flood);
//   }
//   else if (FillType == FLOODFILLSURFACE)
//   {
//     /* Check if the start pixel has the surface color */
//     if (DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_GetPixel(DstSurf, x, y) != ConvColor)
//     {
//       return FALSE;
//     }
//
//     if (initFlood(&flood, DstRect) == FALSE)
//     {
//       return FALSE;
//     }
//     addItemFlood(&flood, x, y, DstSurf, DstRect, ConvColor, TRUE);
//     while (flood.floodLen != 0)
//     {
//       x = flood.floodStart->x;
//       y = flood.floodStart->y;
//       removeItemFlood(&flood);
//
//       DibFunctionsForBitmapFormat[DstSurf->iBitmapFormat].DIB_PutPixel(DstSurf, x, y, BrushColor);
//       if (flood.floodStart - 4 < flood.floodData)
//       {
//         DPRINT1("Can't finish flooding!\n");
//         finalizeFlood(&flood);
//         return FALSE;
//       }
//       addItemFlood(&flood, x, y + 1, DstSurf, DstRect, ConvColor, TRUE);
//       addItemFlood(&flood, x, y - 1, DstSurf, DstRect, ConvColor, TRUE);
//       addItemFlood(&flood, x + 1, y, DstSurf, DstRect, ConvColor, TRUE);
//       addItemFlood(&flood, x - 1, y, DstSurf, DstRect, ConvColor, TRUE);
//     }
//     finalizeFlood(&flood);
//   }
//   else
//   {
//     DPRINT1("Unsupported FloodFill type!\n");
//     return FALSE;
//   }
//   return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
