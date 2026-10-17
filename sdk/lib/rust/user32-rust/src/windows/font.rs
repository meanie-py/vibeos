use crate::ntwrapper::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
use win32k_sys::win32k::*;
use crate::internal::*;
use crate::winbase::*;
use crate::FIELD_OFFSET;
use core::ffi::c_void;

extern "system" {
    fn GdiGetCodePage(hdc: HDC) -> DWORD;
    fn GetProcessHeap() -> HANDLE;
    fn GetTextMetricsA(hdc: HDC, lptm: *mut TEXTMETRICA) -> BOOL;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn MultiByteToWideChar( CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: INT, lpWideCharStr: LPWSTR, cchWideChar: INT, ) -> INT;
    fn WideCharToMultiByte( CodePage: UINT, dwFlags: DWORD, lpWideCharStr: LPCWSTR, cchWideChar: INT, lpMultiByteStr: LPSTR, cbMultiByte: INT, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut BOOL, ) -> INT;
}

// strlen is a CRT cdecl function — must be extern "C" (`_strlen`), not stdcall (`_strlen@4`).
extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// DrawTextExWorker — the core DrawText layout engine — lives in win32ss/user/rtl/text.c,
// which stays C (audit §5) and is compiled into user32. font.c only forward-declares it,
// so call it as an external symbol; do NOT translate it (its static TEXT_* helpers aren't linkable).
extern "system" {
    pub fn DrawTextExWorker(
        hdc: HDC,
        str: LPWSTR,
        i_count: INT,
        rect: LPRECT,
        flags: UINT,
        dtp: LPDRAWTEXTPARAMS,
    ) -> INT;
}

// ---- file-scope preamble (translated) ----
pub const MAX_BUFFER: u32 = 1024;

pub const PREFIX: u32 = 38;

pub const ALPHA_PREFIX: u32 = 30;

pub const KANA_PREFIX: u32 = 31;
#[no_mangle]
pub unsafe extern "C" fn TEXT_TabbedTextOut(
    hdc: HDC,
    x: INT,
    y: INT,
    lpstr: LPCWSTR,
    mut count: INT,
    cTabStops: INT,
    lpTabPos: *const INT,
    nTabOrg: INT,
    fDisplayText: BOOL,
) -> LONG {
    let mut defWidth: INT;
    let mut extent: SIZE = core::mem::zeroed();
    let mut i: i32;
    let mut j: i32;
    let start = x;
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    let mut x = x;
    let mut lpstr = lpstr;
    let mut lpTabPos = lpTabPos;
    let mut cTabStops = cTabStops;

    if lpTabPos.is_null() {
        cTabStops = 0;
    }

    GetTextMetricsW(hdc, &mut tm);

    if cTabStops == 1 {
        defWidth = *lpTabPos;
        cTabStops = 0;
    } else {
        defWidth = 8 * tm.tmAveCharWidth as INT;
    }

    while count > 0 {
        let mut r: RECT = core::mem::zeroed();
        let mut x0: INT;
        x0 = x;
        r.left = x0;

        i = 0;
        while i < count {
            if *lpstr.offset(i as isize) != b'\t' as u16 {
                break;
            }
            i += 1;
        }
        j = i;
        while j < count {
            if *lpstr.offset(j as isize) == b'\t' as u16 {
                break;
            }
            j += 1;
        }

        GetTextExtentPointW(hdc, lpstr.offset(i as isize), j - i, &mut extent);

        if i != 0 {
            while cTabStops > i {
                if nTabOrg + abs(*lpTabPos) > x {
                    if *lpTabPos.offset((i - 1) as isize) >= 0 {
                        x = nTabOrg + *lpTabPos.offset((i - 1) as isize) + extent.cx;
                        break;
                    } else {
                        if nTabOrg - *lpTabPos.offset((i - 1) as isize) - extent.cx > x {
                            x = nTabOrg - *lpTabPos.offset((i - 1) as isize);
                            x0 = x - extent.cx;
                            break;
                        }
                    }
                }
                lpTabPos = lpTabPos.offset(1);
                cTabStops -= 1;
            }

            if cTabStops <= i && defWidth > 0 {
                x0 = nTabOrg + ((x - nTabOrg) / defWidth + i) * defWidth;
                x = x0 + extent.cx;
            } else if cTabStops <= i && defWidth < 0 {
                x = nTabOrg + ((x - nTabOrg + extent.cx) / -defWidth + i) * -defWidth;
                x0 = x - extent.cx;
            }
        } else {
            x += extent.cx;
        }

        if fDisplayText != 0 {
            r.top = y;
            r.right = x;
            r.bottom = y + extent.cy;

            ExtTextOutW(
                hdc,
                x0,
                y,
                if GetBkMode(hdc) == OPAQUE as INT {
                    ETO_OPAQUE
                } else {
                    0
                },
                &mut r,
                lpstr.offset(i as isize),
                (j - i) as UINT,
                core::ptr::null_mut(),
            );
        }
        count -= j;
        lpstr = lpstr.offset(j as isize);
    }

    if extent.cy == 0 {
        extent.cy = tm.tmHeight as INT;
    }

    MAKELONG((x - start) as u16, extent.cy as u16)
}

#[no_mangle]
pub unsafe extern "system" fn TabbedTextOutA(
    hDC: HDC,
    X: INT,
    Y: INT,
    lpString: LPCSTR,
    nCount: INT,
    nTabPositions: INT,
    lpnTabStopPositions: *const INT,
    nTabOrigin: INT,
) -> LONG {
    let ret: LONG;
    let len: DWORD;
    let strW: LPWSTR;
    let cp: UINT = GdiGetCodePage(hDC);

    len = MultiByteToWideChar(cp, 0, lpString, nCount, core::ptr::null_mut(), 0) as DWORD;
    if len == 0 {
        return 0;
    }
    strW = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as LPWSTR;
    if strW.is_null() {
        return 0;
    }
    MultiByteToWideChar(cp, 0, lpString, nCount, strW, len as INT);
    ret = TabbedTextOutW(
        hDC,
        X,
        Y,
        strW,
        len as INT,
        nTabPositions,
        lpnTabStopPositions,
        nTabOrigin,
    );
    HeapFree(GetProcessHeap(), 0, strW as *mut core::ffi::c_void);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn TabbedTextOutW(
    hDC: HDC,
    X: INT,
    Y: INT,
    lpString: LPCWSTR,
    nCount: INT,
    nTabPositions: INT,
    lpnTabStopPositions: *const INT,
    nTabOrigin: INT,
) -> LONG {
    TEXT_TabbedTextOut(
        hDC,
        X,
        Y,
        lpString,
        nCount,
        nTabPositions,
        lpnTabStopPositions,
        nTabOrigin,
        TRUE as BOOL,
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetTabbedTextExtentA(
    hDC: HDC,
    lpString: LPCSTR,
    nCount: INT,
    nTabPositions: INT,
    lpnTabStopPositions: *const INT,
) -> DWORD {
    let ret: LONG;
    let cp: UINT = GdiGetCodePage(hDC);
    let len: DWORD;
    let strW: LPWSTR;

    len = MultiByteToWideChar(cp, 0, lpString, nCount, core::ptr::null_mut(), 0) as DWORD;
    if len == 0 {
        return 0;
    }
    strW = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;
    if strW.is_null() {
        return 0;
    }
    MultiByteToWideChar(cp, 0, lpString, nCount, strW, len as INT);
    ret = GetTabbedTextExtentW(
        hDC,
        strW,
        len as INT,
        nTabPositions,
        lpnTabStopPositions,
    ) as LONG;
    HeapFree(GetProcessHeap(), 0, strW as *mut core::ffi::c_void);
    ret as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn GetTabbedTextExtentW(
    hDC: HDC,
    lpString: LPCWSTR,
    nCount: INT,
    nTabPositions: INT,
    lpnTabStopPositions: *const INT,
) -> DWORD {
    TEXT_TabbedTextOut(
        hDC,
        0,
        0,
        lpString,
        nCount,
        nTabPositions,
        lpnTabStopPositions,
        0,
        0,
    ) as DWORD
}


#[no_mangle]
pub unsafe extern "system" fn DrawTextExW(
    hdc: HDC,
    str: LPWSTR,
    i_count: INT,
    rect: LPRECT,
    flags: UINT,
    dtp: LPDRAWTEXTPARAMS,
) -> INT {
    DrawTextExWorker(hdc, str, i_count, rect, flags, dtp)
}

#[no_mangle]
pub unsafe extern "system" fn DrawTextExA(
    hdc: HDC,
    mut str: LPSTR,
    mut count: INT,
    rect: LPRECT,
    flags: UINT,
    dtp: LPDRAWTEXTPARAMS,
) -> INT {

    let wstr: *mut WCHAR;
    let mut p: *mut WCHAR;
    let mut ret: INT = 0;
    let mut i: i32;
    let wcount: DWORD;
    let mut wmax: DWORD;
    let amax: DWORD;
    let cp: UINT;

    if count == 0 {
        return 0;
    }
    if str.is_null() && count > 0 {
        return 0;
    }
    if str.is_null() || {
        if count == -1 {
            count = strlen(str as *const core::ffi::c_char) as INT;
            count == 0
        } else {
            false
        }
    } {
        let lh: i32;
        let mut tm: TEXTMETRICA = core::mem::zeroed();

        if !dtp.is_null() && (*dtp).cbSize != core::mem::size_of::<DRAWTEXTPARAMS>() as UINT {
            return 0;
        }

        GetTextMetricsA(hdc, &mut tm);
        if flags & DT_EXTERNALLEADING != 0 {
            lh = tm.tmHeight + tm.tmExternalLeading;
        } else {
            lh = tm.tmHeight;
        }

        if flags & DT_CALCRECT != 0 {
            (*rect).right = (*rect).left;
            if flags & DT_SINGLELINE != 0 {
                (*rect).bottom = (*rect).top + lh;
            } else {
                (*rect).bottom = (*rect).top;
            }
        }
        return lh;
    }

    cp = GdiGetCodePage(hdc);
    let mut wcount_v = MultiByteToWideChar(cp, 0, str, count, core::ptr::null_mut(), 0) as DWORD;
    wcount = wcount_v;
    wmax = wcount;
    amax = count as DWORD;
    if flags & DT_MODIFYSTRING != 0 {
        wmax += 4;
    }
    let amax = if flags & DT_MODIFYSTRING != 0 {
        amax + 4
    } else {
        amax
    };

    wstr = HeapAlloc(
        GetProcessHeap(),
        0,
        (wmax as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
    ) as *mut WCHAR;
    if !wstr.is_null() {
        MultiByteToWideChar(cp, 0, str, count, wstr, wcount as INT);
        if flags & DT_MODIFYSTRING != 0 {
            i = 4;
            p = wstr.add(wcount as usize);
            while i != 0 {
                i -= 1;
                *p = 0xFFFE;
                p = p.add(1);
            }
        }
        ret = DrawTextExW(hdc, wstr, wcount as INT, rect, flags, dtp);
        if flags & DT_MODIFYSTRING != 0 {
            i = 4;
            p = wstr.add(wcount as usize);
            while i != 0 && *p != 0xFFFE {
                i -= 1;
                wcount_v += 1;
                p = p.add(1);
            }
            WideCharToMultiByte(
                cp,
                0,
                wstr,
                wcount_v as INT,
                str,
                amax as INT,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
        }
        HeapFree(GetProcessHeap(), 0, wstr as LPVOID);
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn DrawTextW(
    hdc: HDC,
    str: LPCWSTR,
    count: INT,
    rect: LPRECT,
    mut flags: UINT,
) -> INT {
    let mut dtp: DRAWTEXTPARAMS = core::mem::zeroed();

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(dtp),
        0,
        core::mem::size_of::<DRAWTEXTPARAMS>() / core::mem::size_of::<DRAWTEXTPARAMS>(),
    );
    dtp.cbSize = core::mem::size_of::<DRAWTEXTPARAMS>() as UINT;
    if flags & DT_TABSTOP != 0 {
        dtp.iTabLength = ((flags >> 8) & 0xff) as INT;
        flags &= 0xffff00ff;
    }
    DrawTextExW(
        hdc,
        str as LPWSTR,
        count,
        rect,
        flags,
        core::ptr::addr_of_mut!(dtp),
    )
}

#[no_mangle]
pub unsafe extern "system" fn DrawTextA(
    hdc: HDC,
    str: LPCSTR,
    count: INT,
    rect: LPRECT,
    mut flags: UINT,
) -> INT {
    let mut dtp: DRAWTEXTPARAMS = core::mem::zeroed();

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(dtp),
        0,
        core::mem::size_of::<DRAWTEXTPARAMS>() / core::mem::size_of::<DRAWTEXTPARAMS>(),
    );
    dtp.cbSize = core::mem::size_of::<DRAWTEXTPARAMS>() as UINT;
    if (flags & DT_TABSTOP) != 0 {
        dtp.iTabLength = ((flags >> 8) & 0xff) as INT;
        flags &= 0xffff00ff;
    }
    DrawTextExA(
        hdc,
        str as LPSTR,
        count,
        rect,
        flags,
        core::ptr::addr_of_mut!(dtp),
    )
}

#[no_mangle]
pub unsafe extern "system" fn UserLpkPSMTextOut(
    hdc: HDC,
    x: core::ffi::c_int,
    y: core::ffi::c_int,
    lpString: LPCWSTR,
    cString: core::ffi::c_int,
    dwFlags: u32,
) -> core::ffi::c_int {
    let mut size: SIZE = core::mem::zeroed();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let len: core::ffi::c_int;
    let mut i: core::ffi::c_int = 0;
    let mut j: core::ffi::c_int = 0;
    let mut prefix_count: core::ffi::c_int = 0;
    let mut prefix_offset: core::ffi::c_int = -1;
    let mut display_str: LPWSTR = core::ptr::null_mut();
    let prefix_x: core::ffi::c_int;
    let prefix_end: core::ffi::c_int;
    let hpen: HPEN;
    let oldPen: HPEN;

    if lpString.is_null() || cString <= 0 {
        return 0;
    }

    if dwFlags & DT_NOPREFIX != 0 {
        TextOutW(hdc, x, y, lpString, cString);
        GetTextExtentPointW(hdc, lpString, cString, &mut size);
        return size.cx;
    }

    display_str = HeapAlloc(
        GetProcessHeap(),
        0,
        ((cString + 1) as usize * core::mem::size_of::<u16>()) as _,
    ) as LPWSTR;

    if display_str.is_null() {
        return 0;
    }

    while i < cString {
        let ch = *lpString.offset(i as isize);
        if ch == PREFIX as u16 || (iswspace(ch as core::ffi::c_int) != 0 && ch != b' ' as u16) {
            if i < cString - 1 && *lpString.offset((i + 1) as isize) == PREFIX as u16 {
                *display_str.offset(j as isize) = *lpString.offset(i as isize);
                j += 1;
                i += 1;
            } else {
                i += 1;
            }
        } else {
            *display_str.offset(j as isize) = *lpString.offset(i as isize);
            j += 1;
            i += 1;
        }
    }

    *display_str.offset(j as isize) = 0u16;
    len = wcslen(display_str) as core::ffi::c_int;

    if dwFlags & DT_PREFIXONLY == 0 {
        TextOutW(hdc, x, y, display_str, len);
    }

    if dwFlags & DT_HIDEPREFIX == 0 {
        i = 0;
        while i < cString - 1 {
            if *lpString.offset(i as isize) == PREFIX as u16
                && *lpString.offset((i + 1) as isize) != PREFIX as u16
            {
                prefix_offset = i - prefix_count;
                prefix_count += 1;
            } else if *lpString.offset(i as isize) == PREFIX as u16
                && *lpString.offset((i + 1) as isize) == PREFIX as u16
            {
                i += 1;
            }
            i += 1;
        }

        GetTextMetricsW(hdc, &mut tm);

        if prefix_offset != -1 {
            GetTextExtentPointW(hdc, display_str, prefix_offset, &mut size);
            prefix_x = x + size.cx;
            GetTextExtentPointW(hdc, display_str, prefix_offset + 1, &mut size);
            prefix_end = x + size.cx - 1;
            hpen = CreatePen(PS_SOLID as core::ffi::c_int, 1, GetTextColor(hdc));
            oldPen = SelectObject(hdc, hpen as _) as HPEN;
            MoveToEx(hdc, prefix_x, y + tm.tmAscent + 1, core::ptr::null_mut());
            LineTo(hdc, prefix_end, y + tm.tmAscent + 1);
            SelectObject(hdc, oldPen as _);
            DeleteObject(hpen as _);
        }
    }

    GetTextExtentPointW(hdc, display_str, len + 1, &mut size);
    HeapFree(GetProcessHeap(), 0, display_str as _);

    size.cx
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
//  * FILE:            win32ss/user/user32/windows/font.c
//  * PURPOSE:         Draw Text
//  * PROGRAMMER:      Casper S. Hornstrup (chorns@users.sourceforge.net)
//  * UPDATE HISTORY:
//  *      09-05-2001  CSH  Created
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <user32.h>
//
// DWORD WINAPI GdiGetCodePage(HDC hdc);
//
// INT WINAPI DrawTextExWorker( HDC hdc, LPWSTR str, INT i_count,
//                         LPRECT rect, UINT flags, LPDRAWTEXTPARAMS dtp );
//
//
// /* FUNCTIONS *****************************************************************/
//
//
// /***********************************************************************
//  *           TEXT_TabbedTextOut
//  *
//  * Helper function for TabbedTextOut() and GetTabbedTextExtent().
//  * Note: this doesn't work too well for text-alignment modes other
//  *       than TA_LEFT|TA_TOP. But we want bug-for-bug compatibility :-)
//  */
// /* WINE synced 22-May-2006 */
// LONG TEXT_TabbedTextOut( HDC hdc,
//                          INT x,
//                          INT y,
//                          LPCWSTR lpstr,
//                          INT count,
//                          INT cTabStops,
//                          const INT *lpTabPos,
//                          INT nTabOrg,
//                          BOOL fDisplayText )
// {
//     INT defWidth;
//     SIZE extent;
//     int i, j;
//     int start = x;
//     TEXTMETRICW tm;
//
//     if (!lpTabPos)
//         cTabStops=0;
//
//     GetTextMetricsW( hdc, &tm );
//
//     if (cTabStops == 1)
//     {
//         defWidth = *lpTabPos;
//         cTabStops = 0;
//     }
//     else
//     {
//         defWidth = 8 * tm.tmAveCharWidth;
//     }
//
//     while (count > 0)
//     {
//         RECT r;
//         INT x0;
//         x0 = x;
//         r.left = x0;
//         /* chop the string into substrings of 0 or more <tabs>
//          * possibly followed by 1 or more normal characters */
//         for (i = 0; i < count; i++)
//             if (lpstr[i] != '\t') break;
//         for (j = i; j < count; j++)
//             if (lpstr[j] == '\t') break;
//         /* get the extent of the normal character part */
//         GetTextExtentPointW( hdc, lpstr + i, j - i , &extent );
//         /* and if there is a <tab>, calculate its position */
//         if( i) {
//             /* get x coordinate for the drawing of this string */
//             for (; cTabStops > i; lpTabPos++, cTabStops--)
//             {
//                 if( nTabOrg + abs( *lpTabPos) > x) {
//                     if( lpTabPos[ i - 1] >= 0) {
//                         /* a left aligned tab */
//                         x = nTabOrg + lpTabPos[ i-1] + extent.cx;
//                         break;
//                     }
//                     else
//                     {
//                         /* if tab pos is negative then text is right-aligned
//                          * to tab stop meaning that the string extends to the
//                          * left, so we must subtract the width of the string */
//                         if (nTabOrg - lpTabPos[ i - 1] - extent.cx > x)
//                         {
//                             x = nTabOrg - lpTabPos[ i - 1];
//                             x0 = x - extent.cx;
//                             break;
//                         }
//                     }
//                 }
//             }
//             /* if we have run out of tab stops and we have a valid default tab
//              * stop width then round x up to that width */
//             if ((cTabStops <= i) && (defWidth > 0)) {
//                 x0 = nTabOrg + ((x - nTabOrg) / defWidth + i) * defWidth;
//                 x = x0 + extent.cx;
//             } else if ((cTabStops <= i) && (defWidth < 0)) {
//                 x = nTabOrg + ((x - nTabOrg + extent.cx) / -defWidth + i)
//                     * -defWidth;
//                 x0 = x - extent.cx;
//             }
//         } else
//             x += extent.cx;
//
//         if (fDisplayText)
//         {
//             r.top    = y;
//             r.right  = x;
//             r.bottom = y + extent.cy;
//
//             ExtTextOutW( hdc, x0, y, GetBkMode(hdc) == OPAQUE ? ETO_OPAQUE : 0,
//                          &r, lpstr + i, j - i, NULL );
//         }
//         count -= j;
//         lpstr += j;
//     }
//
//     if(!extent.cy)
//         extent.cy = tm.tmHeight;
//
//     return MAKELONG(x - start, extent.cy);
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// TabbedTextOutA(
//   HDC hDC,
//   int X,
//   int Y,
//   LPCSTR lpString,
//   int nCount,
//   int nTabPositions,
//   CONST INT *lpnTabStopPositions,
//   int nTabOrigin)
// {
//   LONG ret;
//   DWORD len;
//   LPWSTR strW;
//   UINT cp = GdiGetCodePage( hDC ); // CP_ACP
//
//   len = MultiByteToWideChar(cp, 0, lpString, nCount, NULL, 0);
//   if (!len) return 0;
//   strW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//   if (!strW) return 0;
//   MultiByteToWideChar(cp, 0, lpString, nCount, strW, len);
//   ret = TabbedTextOutW(hDC, X, Y, strW, len, nTabPositions, lpnTabStopPositions, nTabOrigin);
//   HeapFree(GetProcessHeap(), 0, strW);
//   return ret;
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// TabbedTextOutW(
//   HDC hDC,
//   int X,
//   int Y,
//   LPCWSTR lpString,
//   int nCount,
//   int nTabPositions,
//   CONST INT *lpnTabStopPositions,
//   int nTabOrigin)
// {
//   return TEXT_TabbedTextOut(hDC, X, Y, lpString, nCount, nTabPositions, lpnTabStopPositions, nTabOrigin, TRUE);
// }
//
// /* WINE synced 22-May-2006 */
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTabbedTextExtentA(
//   HDC hDC,
//   LPCSTR lpString,
//   int nCount,
//   int nTabPositions,
//   CONST INT *lpnTabStopPositions)
// {
//     LONG ret;
//     UINT cp = GdiGetCodePage( hDC ); // CP_ACP
//     DWORD len;
//     LPWSTR strW;
//
//     len = MultiByteToWideChar(cp, 0, lpString, nCount, NULL, 0);
//     if (!len) return 0;
//     strW = HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//     if (!strW) return 0;
//     MultiByteToWideChar(cp, 0, lpString, nCount, strW, len);
//     ret = GetTabbedTextExtentW(hDC, strW, len, nTabPositions, lpnTabStopPositions);
//     HeapFree(GetProcessHeap(), 0, strW);
//     return ret;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// GetTabbedTextExtentW(
//   HDC hDC,
//   LPCWSTR lpString,
//   int nCount,
//   int nTabPositions,
//   CONST INT *lpnTabStopPositions)
// {
//    return TEXT_TabbedTextOut(hDC, 0, 0, lpString, nCount, nTabPositions, lpnTabStopPositions, 0, FALSE);
// }
//
//
// /***********************************************************************
//  *           DrawTextExW    (USER32.@)
//  *
//  * The documentation on the extra space required for DT_MODIFYSTRING at MSDN
//  * is not quite complete, especially with regard to \0.  We will assume that
//  * the returned string could have a length of up to i_count+3 and also have
//  * a trailing \0 (which would be 4 more than a not-null-terminated string but
//  * 3 more than a null-terminated string).  If this is not so then increase
//  * the allowance in DrawTextExA.
//  */
// #define MAX_BUFFER 1024
// /* 
//  * @implemented
//  *
//  * Synced with Wine Staging 1.7.37
//  */
// INT WINAPI DrawTextExW( HDC hdc, LPWSTR str, INT i_count,
//                         LPRECT rect, UINT flags, LPDRAWTEXTPARAMS dtp )
// {
//    return DrawTextExWorker( hdc, str, i_count, rect, flags, dtp );
// }
//
// /***********************************************************************
//  *           DrawTextExA    (USER32.@)
//  *
//  * If DT_MODIFYSTRING is specified then there must be room for up to
//  * 4 extra characters.  We take great care about just how much modified
//  * string we return.
//  *
//  * @implemented
//  *
//  * Synced with Wine Staging 1.7.37
//  */
// INT WINAPI DrawTextExA( HDC hdc, LPSTR str, INT count,
//                         LPRECT rect, UINT flags, LPDRAWTEXTPARAMS dtp )
// {
//    WCHAR *wstr;
//    WCHAR *p;
//    INT ret = 0;
//    int i;
//    DWORD wcount;
//    DWORD wmax;
//    DWORD amax;
//    UINT cp;
//
//    if (!count) return 0;
//    if (!str && count > 0) return 0;
//    if( !str || ((count == -1) && !(count = strlen(str))))
//    {
//         int lh;
//         TEXTMETRICA tm;
//
//         if (dtp && dtp->cbSize != sizeof(DRAWTEXTPARAMS))
//             return 0;
//
//         GetTextMetricsA(hdc, &tm);
//         if (flags & DT_EXTERNALLEADING)
//             lh = tm.tmHeight + tm.tmExternalLeading;
//         else
//             lh = tm.tmHeight;
//
//         if( flags & DT_CALCRECT)
//         {
//             rect->right = rect->left;
//             if( flags & DT_SINGLELINE)
//                 rect->bottom = rect->top + lh;
//             else
//                 rect->bottom = rect->top;
//         }
//         return lh;
//    }
//    cp = GdiGetCodePage( hdc );
//    wcount = MultiByteToWideChar( cp, 0, str, count, NULL, 0 );
//    wmax = wcount;
//    amax = count;
//    if (flags & DT_MODIFYSTRING)
//    {
//         wmax += 4;
//         amax += 4;
//    }
//    wstr = HeapAlloc(GetProcessHeap(), 0, wmax * sizeof(WCHAR));
//    if (wstr)
//    {
//        MultiByteToWideChar( cp, 0, str, count, wstr, wcount );
//        if (flags & DT_MODIFYSTRING)
//            for (i=4, p=wstr+wcount; i--; p++) *p=0xFFFE;
//            /* Initialise the extra characters so that we can see which ones
//             * change.  U+FFFE is guaranteed to be not a unicode character and
//             * so will not be generated by DrawTextEx itself.
//             */
//        ret = DrawTextExW( hdc, wstr, wcount, rect, flags, dtp );
//        if (flags & DT_MODIFYSTRING)
//        {
//             /* Unfortunately the returned string may contain multiple \0s
//              * and so we need to measure it ourselves.
//              */
//             for (i=4, p=wstr+wcount; i-- && *p != 0xFFFE; p++) wcount++;
//             WideCharToMultiByte( cp, 0, wstr, wcount, str, amax, NULL, NULL );
//        }
//        HeapFree(GetProcessHeap(), 0, wstr);
//    }
//    return ret;
// }
//
// /***********************************************************************
//  *           DrawTextW    (USER32.@)
//  *
//  * @implemented
//  * Synced with Wine Staging 1.7.37
//  */
// INT WINAPI DrawTextW( HDC hdc, LPCWSTR str, INT count, LPRECT rect, UINT flags )
// {
//     DRAWTEXTPARAMS dtp;
//
//     memset (&dtp, 0, sizeof(dtp));
//     dtp.cbSize = sizeof(dtp);
//     if (flags & DT_TABSTOP)
//     {
//         dtp.iTabLength = (flags >> 8) & 0xff;
//         flags &= 0xffff00ff;
//     }
//     return DrawTextExW(hdc, (LPWSTR)str, count, rect, flags, &dtp);
// }
//
// /***********************************************************************
//  *           DrawTextA    (USER32.@)
//  *
//  * @implemented
//  * Synced with Wine Staging 1.7.37
//  */
// INT WINAPI DrawTextA( HDC hdc, LPCSTR str, INT count, LPRECT rect, UINT flags )
// {
//     DRAWTEXTPARAMS dtp;
//
//     memset (&dtp, 0, sizeof(dtp));
//     dtp.cbSize = sizeof(dtp);
//     if (flags & DT_TABSTOP)
//     {
//         dtp.iTabLength = (flags >> 8) & 0xff;
//         flags &= 0xffff00ff;
//     }
//     return DrawTextExA( hdc, (LPSTR)str, count, rect, flags, &dtp );
// }
//
// /***************************************************************************
//  *       UserLpkPSMTextOut
//  * 
//  * Stripped down version of DrawText, can only draw single line text and
//  * Prefix underscore (only on the last found amperstand)
//  * only flags to be found to be of use in testing:
//  * 
//  * DT_NOPREFIX   - Draw the string as is it without any changes
//  * DT_HIDEPREFIX - Draw the string without underscore
//  * DT_PREFIXONLY - Draw only the underscore
//  * 
//  * without any of these flags the behavior is the string being drawn without the amperstands and
//  * with the underscore.
//  * 
//  * lpk has an equivalent function - LpkPSMTextOut
//  * Notes by testing:
//  * This function in windows doesn't check if lpString is NULL, which results a crash,
//  * returns seemingly random values without any logic, and ignores the DT_NOPREFIX value.
//  * All of these issues don't exist in the LPK version.
//  * 
//  * Note: lpString does not need to be null terminated
//  */
// #define PREFIX 38
// #define ALPHA_PREFIX 30 /* Win16: Alphabet prefix */
// #define KANA_PREFIX  31 /* Win16: Katakana prefix */
//
// INT WINAPI UserLpkPSMTextOut(HDC hdc, int x, int y, LPCWSTR lpString, int cString, DWORD dwFlags)
// {
//     SIZE size;
//     TEXTMETRICW tm;
//     int len, i = 0, j = 0;
//     int prefix_count = 0, prefix_offset = -1;
//     LPWSTR display_str = NULL;
//     int prefix_x, prefix_end;
//     HPEN hpen;
//     HPEN oldPen;
//
//     if (!lpString || cString <= 0)
//         return 0;
//
//     if (dwFlags & DT_NOPREFIX) /* Windows ignores this */
//     {
//         TextOutW(hdc, x, y, lpString, cString);
//         GetTextExtentPointW(hdc, lpString, cString, &size);
//         return size.cx;
//     }
//
//     display_str = HeapAlloc(GetProcessHeap(), 0, (cString + 1) * sizeof(WCHAR));
//
//     if (!display_str)
//         return 0;
//
//     while (i < cString)
//     {
//         if (lpString[i] == PREFIX || (iswspace(lpString[i]) && lpString[i] != L' '))
//         {
//             if (i < cString - 1 && lpString[i + 1] == PREFIX)
//                 display_str[j++] = lpString[i++];
//             else
//                 i++;
//         }
//         else
//         {
//             display_str[j++] = lpString[i++];
//         }
//     }
//
//     display_str[j] = L'\0';   
//     len = wcslen(display_str);
//
//     if (!(dwFlags & DT_PREFIXONLY))
//         TextOutW(hdc, x, y, display_str, len);
//
//     if (!(dwFlags & DT_HIDEPREFIX))
//     {
//         
//         for (i = 0; i < cString - 1; i++)
//         {
//             if (lpString[i] == PREFIX && lpString[i + 1] != PREFIX)
//             {
//                 prefix_offset = i - prefix_count;
//                 prefix_count++;
//             }
//             else if (lpString[i] == PREFIX && lpString[i + 1] == PREFIX)
//             {
//                 i++;
//             }
//         }
//
//         GetTextMetricsW(hdc, &tm);
//         
//         if (prefix_offset != -1)
//         {
//             GetTextExtentPointW(hdc, display_str, prefix_offset, &size);
//             prefix_x = x + size.cx;
//             GetTextExtentPointW(hdc, display_str, prefix_offset + 1, &size);
//             prefix_end = x + size.cx - 1;
//             hpen = CreatePen(PS_SOLID, 1, GetTextColor(hdc));
//             oldPen = SelectObject(hdc, hpen);
//             MoveToEx(hdc, prefix_x, y + tm.tmAscent + 1, NULL);
//             LineTo(hdc, prefix_end, y + tm.tmAscent + 1);
//             SelectObject(hdc, oldPen);
//             DeleteObject(hpen);
//         }
//     }
//
//     GetTextExtentPointW(hdc, display_str, len + 1, &size);
//     HeapFree(GetProcessHeap(), 0, display_str);
//
//     return size.cx;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
