use crate::ntwrapper::*;
use win32k_sys::win32k::*;
use crate::internal::*;
use crate::winbase::*;
use crate::FIELD_OFFSET;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn CopyRect(lprcDst: LPRECT, lprcSrc: *const RECT) -> i32 {
    if lprcDst.is_null() || lprcSrc.is_null() {
        return 0;
    }

    *lprcDst = *lprcSrc;

    1
}

#[no_mangle]
pub unsafe extern "system" fn EqualRect(lprc1: *const RECT, lprc2: *const RECT) -> i32 {
    if lprc1.is_null() || lprc2.is_null() {
        return 0;
    }

    (((*lprc1).left == (*lprc2).left)
        && ((*lprc1).top == (*lprc2).top)
        && ((*lprc1).right == (*lprc2).right)
        && ((*lprc1).bottom == (*lprc2).bottom)) as i32
}


#[no_mangle]
pub unsafe extern "system" fn InflateRect(rect: LPRECT, dx: i32, dy: i32) -> i32 {
    if rect.is_null() {
        return 0;
    }

    (*rect).left -= dx;
    (*rect).top -= dy;
    (*rect).right += dx;
    (*rect).bottom += dy;

    1
}

#[no_mangle]
pub unsafe extern "system" fn IsRectEmpty(lprc: *const RECT) -> BOOL {
    if lprc.is_null() {
        return 1;
    }

    (((*lprc).left >= (*lprc).right) || ((*lprc).top >= (*lprc).bottom)) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn OffsetRect(rect: LPRECT, dx: i32, dy: i32) -> BOOL {
    if rect.is_null() {
        return 0;
    }

    (*rect).left += dx;
    (*rect).top += dy;
    (*rect).right += dx;
    (*rect).bottom += dy;

    1
}

#[no_mangle]
pub unsafe extern "system" fn PtInRect(lprc: *const RECT, pt: POINT) -> i32 {
    if lprc.is_null() {
        return 0;
    }

    ((pt.x >= (*lprc).left)
        && (pt.x < (*lprc).right)
        && (pt.y >= (*lprc).top)
        && (pt.y < (*lprc).bottom)) as i32
}

#[no_mangle]
pub unsafe extern "system" fn SetRect(
    lprc: LPRECT,
    xLeft: core::ffi::c_int,
    yTop: core::ffi::c_int,
    xRight: core::ffi::c_int,
    yBottom: core::ffi::c_int,
) -> core::ffi::c_int {
    if lprc.is_null() {
        return 0;
    }

    (*lprc).left = xLeft;
    (*lprc).top = yTop;
    (*lprc).right = xRight;
    (*lprc).bottom = yBottom;

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetRectEmpty(lprc: LPRECT) -> BOOL {
    if lprc.is_null() {
        return 0;
    }

    (*lprc).left = 0;
    (*lprc).right = 0;
    (*lprc).top = 0;
    (*lprc).bottom = 0;

    1
}

#[no_mangle]
pub unsafe extern "system" fn IntersectRect(
    lprcDst: LPRECT,
    lprcSrc1: *const RECT,
    lprcSrc2: *const RECT,
) -> BOOL {
    if lprcDst.is_null() || lprcSrc1.is_null() || lprcSrc2.is_null() {
        return 0;
    }

    if IsRectEmpty(lprcSrc1) != 0
        || IsRectEmpty(lprcSrc2) != 0
        || (*lprcSrc1).left >= (*lprcSrc2).right
        || (*lprcSrc2).left >= (*lprcSrc1).right
        || (*lprcSrc1).top >= (*lprcSrc2).bottom
        || (*lprcSrc2).top >= (*lprcSrc1).bottom
    {
        SetRectEmpty(lprcDst);
        return 0;
    }

    (*lprcDst).left = core::cmp::max((*lprcSrc1).left, (*lprcSrc2).left);
    (*lprcDst).right = core::cmp::min((*lprcSrc1).right, (*lprcSrc2).right);
    (*lprcDst).top = core::cmp::max((*lprcSrc1).top, (*lprcSrc2).top);
    (*lprcDst).bottom = core::cmp::min((*lprcSrc1).bottom, (*lprcSrc2).bottom);

    1
}

#[no_mangle]
pub unsafe extern "system" fn UnionRect(
    lprcDst: LPRECT,
    lprcSrc1: *const RECT,
    lprcSrc2: *const RECT,
) -> BOOL {
    if lprcDst.is_null() || lprcSrc1.is_null() || lprcSrc2.is_null() {
        return 0;
    }

    if IsRectEmpty(lprcSrc1) != 0 {
        if IsRectEmpty(lprcSrc2) != 0 {
            SetRectEmpty(lprcDst);
            return 0;
        } else {
            *lprcDst = *lprcSrc2;
        }
    } else {
        if IsRectEmpty(lprcSrc2) != 0 {
            *lprcDst = *lprcSrc1;
        } else {
            (*lprcDst).left = core::cmp::min((*lprcSrc1).left, (*lprcSrc2).left);
            (*lprcDst).top = core::cmp::min((*lprcSrc1).top, (*lprcSrc2).top);
            (*lprcDst).right = core::cmp::max((*lprcSrc1).right, (*lprcSrc2).right);
            (*lprcDst).bottom = core::cmp::max((*lprcSrc1).bottom, (*lprcSrc2).bottom);
        }
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SubtractRect(
    lprcDst: LPRECT,
    lprcSrc1: *const RECT,
    lprcSrc2: *const RECT,
) -> i32 {
    let mut tempRect: RECT = core::mem::zeroed();

    if lprcDst.is_null() || lprcSrc1.is_null() || lprcSrc2.is_null() {
        return 0;
    }

    if IntersectRect(core::ptr::addr_of_mut!(tempRect), lprcSrc1, lprcSrc2) == 0 {
        *lprcDst = *lprcSrc1;
        return 1;
    }

    if EqualRect(core::ptr::addr_of_mut!(tempRect), lprcSrc1) != 0 {
        SetRectEmpty(lprcDst);
        return 0;
    }

    *lprcDst = *lprcSrc1;

    if (*lprcDst).top == tempRect.top && (*lprcDst).bottom == tempRect.bottom {
        if (*lprcDst).left == tempRect.left {
            (*lprcDst).left = tempRect.right;
        } else if (*lprcDst).right == tempRect.right {
            (*lprcDst).right = tempRect.left;
        }
    } else if (*lprcDst).left == tempRect.left && (*lprcDst).right == tempRect.right {
        if (*lprcDst).top == tempRect.top {
            (*lprcDst).top = tempRect.bottom;
        } else if (*lprcDst).bottom == tempRect.bottom {
            (*lprcDst).bottom = tempRect.top;
        }
    }

    1
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 1998, 1999, 2000, 2001 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License
//  *  along with this program; if not, write to the Free Software
//  *  Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.
//  */
// /*
//  * PROJECT:         ReactOS user32.dll
//  * FILE:            win32ss/user/user32/windows/rect.c
//  * PURPOSE:         Input
//  * PROGRAMMER:      Casper S. Hornstrup (chorns@users.sourceforge.net)
//  * UPDATE HISTORY:
//  *      09-05-2001  CSH  Created
//  */
//
// #include <user32.h>
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CopyRect(LPRECT lprcDst,
//          CONST RECT *lprcSrc)
// {
//     if (lprcDst == NULL || lprcSrc == NULL)
//         return FALSE;
//
//     *lprcDst = *lprcSrc;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// EqualRect(CONST RECT *lprc1,
//           CONST RECT *lprc2)
// {
//     if (lprc1 == NULL || lprc2 == NULL)
//         return FALSE;
//
//     return (lprc1->left == lprc2->left) && (lprc1->top == lprc2->top) &&
//            (lprc1->right == lprc2->right) && (lprc1->bottom == lprc2->bottom);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InflateRect(LPRECT rect,
//             int dx,
//             int dy)
// {
//     if (rect == NULL)
//         return FALSE;
//
//     rect->left -= dx;
//     rect->top -= dy;
//     rect->right += dx;
//     rect->bottom += dy;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IntersectRect(LPRECT lprcDst,
//               CONST RECT *lprcSrc1,
//               CONST RECT *lprcSrc2)
// {
//     if (lprcDst == NULL || lprcSrc1 == NULL || lprcSrc2 == NULL)
//         return FALSE;
//
//     if (IsRectEmpty(lprcSrc1) || IsRectEmpty(lprcSrc2) ||
//         lprcSrc1->left >= lprcSrc2->right ||
//         lprcSrc2->left >= lprcSrc1->right ||
//         lprcSrc1->top >= lprcSrc2->bottom ||
//         lprcSrc2->top >= lprcSrc1->bottom)
//     {
//         SetRectEmpty(lprcDst);
//         return FALSE;
//     }
//
//     lprcDst->left = max(lprcSrc1->left, lprcSrc2->left);
//     lprcDst->right = min(lprcSrc1->right, lprcSrc2->right);
//     lprcDst->top = max(lprcSrc1->top, lprcSrc2->top);
//     lprcDst->bottom = min(lprcSrc1->bottom, lprcSrc2->bottom);
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// IsRectEmpty(CONST RECT *lprc)
// {
//     if (lprc == NULL)
//         return TRUE;
//
//     return ((lprc->left >= lprc->right) || (lprc->top >= lprc->bottom));
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// OffsetRect(LPRECT rect,
//            int dx,
//            int dy)
// {
//     if (rect == NULL)
//         return FALSE;
//
//     rect->left += dx;
//     rect->top += dy;
//     rect->right += dx;
//     rect->bottom += dy;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PtInRect(CONST RECT *lprc,
//          POINT pt)
// {
//     if (lprc == NULL)
//         return FALSE;
//
//     return((pt.x >= lprc->left) && (pt.x < lprc->right) &&
//            (pt.y >= lprc->top) && (pt.y < lprc->bottom));
// }
//
// BOOL
// WINAPI
// SetRect(LPRECT lprc,
//         int xLeft,
//         int yTop,
//         int xRight,
//         int yBottom)
// {
//     if (lprc == NULL)
//         return FALSE;
//
//     lprc->left = xLeft;
//     lprc->top = yTop;
//     lprc->right = xRight;
//     lprc->bottom = yBottom;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetRectEmpty(LPRECT lprc)
// {
//     if (lprc == NULL)
//         return FALSE;
//
//     lprc->left = lprc->right = lprc->top = lprc->bottom = 0;
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SubtractRect(LPRECT lprcDst,
//              CONST RECT *lprcSrc1,
//              CONST RECT *lprcSrc2)
// {
//     RECT tempRect;
//
//     if (lprcDst == NULL || lprcSrc1 == NULL || lprcSrc2 == NULL)
//         return FALSE;
//
//     if (!IntersectRect(&tempRect, lprcSrc1, lprcSrc2))
//     {
//         *lprcDst = *lprcSrc1;
//         return TRUE;
//     }
//
//     if (EqualRect(&tempRect, lprcSrc1))
//     {
//         SetRectEmpty(lprcDst);
//         return FALSE;
//     }
//
//     *lprcDst = *lprcSrc1;
//
//     if (lprcDst->top == tempRect.top && lprcDst->bottom == tempRect.bottom)
//     {
//         if (lprcDst->left == tempRect.left)
//             lprcDst->left = tempRect.right;
//         else if (lprcDst->right == tempRect.right)
//             lprcDst->right = tempRect.left;
//     }
//     else if (lprcDst->left == tempRect.left && lprcDst->right == tempRect.right)
//     {
//         if (lprcDst->top == tempRect.top)
//             lprcDst->top = tempRect.bottom;
//         else if (lprcDst->bottom == tempRect.bottom)
//             lprcDst->bottom = tempRect.top;
//     }
//
//     return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// UnionRect(LPRECT lprcDst,
//           CONST RECT *lprcSrc1,
//           CONST RECT *lprcSrc2)
// {
//     if (lprcDst == NULL || lprcSrc1 == NULL || lprcSrc2 == NULL)
//         return FALSE;
//
//     if (IsRectEmpty(lprcSrc1))
//     {
//         if (IsRectEmpty(lprcSrc2))
//         {
//             SetRectEmpty(lprcDst);
//             return FALSE;
//         }
//         else
//         {
//             *lprcDst = *lprcSrc2;
//         }
//     }
//     else
//     {
//         if (IsRectEmpty(lprcSrc2))
//         {
//             *lprcDst = *lprcSrc1;
//         }
//         else
//         {
//             lprcDst->left = min(lprcSrc1->left, lprcSrc2->left);
//             lprcDst->top = min(lprcSrc1->top, lprcSrc2->top);
//             lprcDst->right = max(lprcSrc1->right, lprcSrc2->right);
//             lprcDst->bottom = max(lprcSrc1->bottom, lprcSrc2->bottom);
//         }
//     }
//
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
