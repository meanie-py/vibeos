use win32k_sys::win32k::*;

// FORCEINLINE in rect.h (no symbol) + min/max macros — inline them.
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> i32 {
    ((*prcl).left >= (*prcl).right || (*prcl).top >= (*prcl).bottom) as i32
}
#[inline(always)]
unsafe fn RECTL_vSetEmptyRect(prcl: *mut RECTL) {
    (*prcl).left = 0;
    (*prcl).top = 0;
    (*prcl).right = 0;
    (*prcl).bottom = 0;
}
#[inline(always)]
fn min(a: i32, b: i32) -> i32 { if a < b { a } else { b } }
#[inline(always)]
fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate from this preamble

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RECTL_bUnionRect(
    prclDst: *mut RECTL,
    prcl1: *const RECTL,
    prcl2: *const RECTL,
) -> i32 {
    if RECTL_bIsEmptyRect(prcl1) != 0 {
        if RECTL_bIsEmptyRect(prcl2) != 0 {
            RECTL_vSetEmptyRect(prclDst);
            return 0;
        } else {
            *prclDst = *prcl2;
        }
    } else {
        if RECTL_bIsEmptyRect(prcl2) != 0 {
            *prclDst = *prcl1;
        } else {
            (*prclDst).left = min((*prcl1).left, (*prcl2).left);
            (*prclDst).top = min((*prcl1).top, (*prcl2).top);
            (*prclDst).right = max((*prcl1).right, (*prcl2).right);
            (*prclDst).bottom = max((*prcl1).bottom, (*prcl2).bottom);
        }
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RECTL_bUnionRect(
    prclDst: *mut RECTL,
    prcl1: *const RECTL,
    prcl2: *const RECTL,
) -> i32 {
    if RECTL_bIsEmptyRect(prcl1) != 0 {
        if RECTL_bIsEmptyRect(prcl2) != 0 {
            RECTL_vSetEmptyRect(prclDst);
            return 0;
        } else {
            *prclDst = *prcl2;
        }
    } else {
        if RECTL_bIsEmptyRect(prcl2) != 0 {
            *prclDst = *prcl1;
        } else {
            (*prclDst).left = min((*prcl1).left, (*prcl2).left);
            (*prclDst).top = min((*prcl1).top, (*prcl2).top);
            (*prclDst).right = max((*prcl1).right, (*prcl2).right);
            (*prclDst).bottom = max((*prcl1).bottom, (*prcl2).bottom);
        }
    }

    1
}

// rect.h declares BOOL. This returned u8, and every caller (C and the Rust decls alike) reads
// 32 bits: the bits above the byte are whatever the register held, so "no intersection"
// could read as TRUE.
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn RECTL_bIntersectRect(
    prclDst: *mut RECTL,
    prcl1: *const RECTL,
    prcl2: *const RECTL
) -> BOOL {

    (*prclDst).left = core::cmp::max((*prcl1).left, (*prcl2).left);
    (*prclDst).right = core::cmp::min((*prcl1).right, (*prcl2).right);

    if (*prclDst).left < (*prclDst).right {
        (*prclDst).top = core::cmp::max((*prcl1).top, (*prcl2).top);
        (*prclDst).bottom = core::cmp::min((*prcl1).bottom, (*prcl2).bottom);

        if (*prclDst).top < (*prclDst).bottom {
            return 1;
        }
    }

    RECTL_vSetEmptyRect(prclDst);

    return 0;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn RECTL_bIntersectRect(
    prclDst: *mut RECTL,
    prcl1: *const RECTL,
    prcl2: *const RECTL
) -> BOOL {

    (*prclDst).left = core::cmp::max((*prcl1).left, (*prcl2).left);
    (*prclDst).right = core::cmp::min((*prcl1).right, (*prcl2).right);

    if (*prclDst).left < (*prclDst).right {
        (*prclDst).top = core::cmp::max((*prcl1).top, (*prcl2).top);
        (*prclDst).bottom = core::cmp::min((*prcl1).bottom, (*prcl2).bottom);

        if (*prclDst).top < (*prclDst).bottom {
            return 1;
        }
    }

    RECTL_vSetEmptyRect(prclDst);

    return 0;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RECTL_vMakeWellOrdered(
    prcl: *mut RECTL)
{
    let mut lTmp: i32;
    if (*prcl).left > (*prcl).right {
        lTmp = (*prcl).left;
        (*prcl).left = (*prcl).right;
        (*prcl).right = lTmp;
    }
    if (*prcl).top > (*prcl).bottom {
        lTmp = (*prcl).top;
        (*prcl).top = (*prcl).bottom;
        (*prcl).bottom = lTmp;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RECTL_vMakeWellOrdered(
    prcl: *mut RECTL)
{
    let mut lTmp: i32;
    if (*prcl).left > (*prcl).right {
        lTmp = (*prcl).left;
        (*prcl).left = (*prcl).right;
        (*prcl).right = lTmp;
    }
    if (*prcl).top > (*prcl).bottom {
        lTmp = (*prcl).top;
        (*prcl).top = (*prcl).bottom;
        (*prcl).bottom = lTmp;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn RECTL_vInflateRect(
    rect: *mut RECTL,
    dx: i32,
    dy: i32,
) {
    (*rect).left -= dx;
    (*rect).top -= dy;
    (*rect).right += dx;
    (*rect).bottom += dy;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn RECTL_vInflateRect(
    rect: *mut RECTL,
    dx: i32,
    dy: i32,
) {
    (*rect).left -= dx;
    (*rect).top -= dy;
    (*rect).right += dx;
    (*rect).bottom += dy;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/rect.c
//  * PURPOSE:         Rect functions
//  * PROGRAMMER:      Timo Kreuzer
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// BOOL
// FASTCALL
// RECTL_bUnionRect(
//     _Out_ RECTL *prclDst,
//     _In_ const RECTL *prcl1,
//     _In_ const RECTL *prcl2)
// {
//     if (RECTL_bIsEmptyRect(prcl1))
//     {
//         if (RECTL_bIsEmptyRect(prcl2))
// 	    {
// 	        RECTL_vSetEmptyRect(prclDst);
// 	        return FALSE;
// 	    }
//         else
// 	    {
// 	        *prclDst = *prcl2;
// 	    }
//     }
//     else
//     {
//         if (RECTL_bIsEmptyRect(prcl2))
// 	    {
// 	        *prclDst = *prcl1;
// 	    }
//         else
// 	    {
// 	        prclDst->left = min(prcl1->left, prcl2->left);
// 	        prclDst->top = min(prcl1->top, prcl2->top);
// 	        prclDst->right = max(prcl1->right, prcl2->right);
// 	        prclDst->bottom = max(prcl1->bottom, prcl2->bottom);
// 	    }
//     }
//
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// RECTL_bIntersectRect(
//     _Out_ RECTL* prclDst,
//     _In_ const RECTL* prcl1,
//     _In_ const RECTL* prcl2)
// {
//     prclDst->left  = max(prcl1->left, prcl2->left);
//     prclDst->right = min(prcl1->right, prcl2->right);
//
//     if (prclDst->left < prclDst->right)
//     {
//         prclDst->top    = max(prcl1->top, prcl2->top);
//         prclDst->bottom = min(prcl1->bottom, prcl2->bottom);
//
//         if (prclDst->top < prclDst->bottom)
//         {
//             return TRUE;
//         }
//     }
//
//     RECTL_vSetEmptyRect(prclDst);
//
//     return FALSE;
// }
//
// VOID
// FASTCALL
// RECTL_vMakeWellOrdered(
//     _Inout_ RECTL *prcl)
// {
//     LONG lTmp;
//     if (prcl->left > prcl->right)
//     {
//         lTmp = prcl->left;
//         prcl->left = prcl->right;
//         prcl->right = lTmp;
//     }
//     if (prcl->top > prcl->bottom)
//     {
//         lTmp = prcl->top;
//         prcl->top = prcl->bottom;
//         prcl->bottom = lTmp;
//     }
// }
//
// VOID
// FASTCALL
// RECTL_vInflateRect(
//     _Inout_ RECTL *rect,
//     _In_ INT dx,
//     _In_ INT dy)
// {
//     rect->left -= dx;
//     rect->top -= dy;
//     rect->right += dx;
//     rect->bottom += dy;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
