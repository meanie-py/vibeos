// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
unsafe extern "C" fn LengthOfStrResource(
    hInst: HINSTANCE,
    uID: u32,
) -> i32 {
    let hrSrc: HRSRC;
    let hRes: HGLOBAL;
    let mut lpName: LPWSTR;
    let mut lpStr: LPWSTR;

    if hInst.is_null() {
        return -1;
    }

    lpName = MAKEINTRESOURCE!((uID >> 4) + 1);

    hrSrc = FindResourceW(hInst, lpName, RT_STRING as LPWSTR);
    if !hrSrc.is_null() {
        hRes = LoadResource(hInst, hrSrc);
        if !hRes.is_null() {
            lpStr = LockResource(hRes);
            if !lpStr.is_null() {
                let mut x: u32;
                let mut uID = uID;

                uID &= 0xF;
                x = 0;
                while x < uID {
                    lpStr = lpStr.add(((*lpStr) as usize) + 1);
                    x += 1;
                }

                return (*lpStr) as i32;
            }
        }
    }
    -1
}
#[no_mangle]
pub unsafe extern "C" fn AllocAndLoadString(
    lpTarget: *mut LPTSTR,
    hInst: HINSTANCE,
    uID: u32,
) -> i32 {
    let mut ln: i32;

    ln = LengthOfStrResource(hInst, uID);
    ln += 1;
    if ln - 1 > 0 {
        *lpTarget = LocalAlloc(LMEM_FIXED, (ln as usize) * core::mem::size_of::<TCHAR>()) as LPTSTR;
        if !(*lpTarget).is_null() {
            let Ret: i32;
            Ret = LoadString(hInst, uID, *lpTarget, ln);
            if Ret == 0 {
                LocalFree(*lpTarget as HLOCAL);
            }
            return Ret;
        }
    }
    return 0;
}
unsafe extern "C" fn DrawEtchedLine(
    hTheme: HTHEME,
    hdc: HDC,
    iPart: i32,
    iState: i32,
    pRect: LPRECT,
    uFlags: u32,
) {
    if !hTheme.is_null() {
        DrawThemeEdge(hTheme, hdc, iPart, iState, pRect, EDGE_ETCHED, uFlags, pRect);
    } else {
        DrawEdge(hdc, pRect, EDGE_ETCHED, uFlags);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ClrBtn_CustomDraw(
    pCD: *mut NMCUSTOMDRAW,
    Color: COLORREF,
) -> u32 {
    // misc.c:86 `enum { MARGIN = 1, SEPLINE = 2 };` — a C anonymous enum used purely as two
    // named integers, which is `const` in Rust. A real `enum` here would need a cast at each of
    // the five arithmetic uses below.
    const MARGIN: c_int = 1;
    const SEPLINE: c_int = 2;
    let hTheme: HTHEME;
    let hOrgPen: HGDIOBJ;
    let hOrgBrush: HGDIOBJ;
    let clrOrgPen: COLORREF;
    let clrOrgBrush: COLORREF;
    let clrWidget: COLORREF;
    let nPadW: i32;
    let nPadH: i32;
    let nHalf: i32;
    let nArrow: i32;
    let mut r: RECT = (*pCD).rc;
    let mut pts: [POINT; 3] = core::mem::zeroed();

    if (*pCD).dwDrawStage == CDDS_PREPAINT {
        return CDRF_NOTIFYPOSTPAINT;
    }
    if (*pCD).dwDrawStage != CDDS_POSTPAINT {
        return CDRF_DODEFAULT;
    }

    nPadW = GetSystemMetrics(SM_CXFOCUSBORDER) + MARGIN;
    nPadH = GetSystemMetrics(SM_CYFOCUSBORDER) + MARGIN;

    hTheme = GetWindowTheme((*pCD).hdr.hwndFrom);
    if !hTheme.is_null() {
        let mut margins: MARGINS = core::mem::zeroed();
        GetThemeMargins(hTheme, (*pCD).hdc, BP_PUSHBUTTON, PBS_NORMAL, TMT_CONTENTMARGINS, core::ptr::null_mut(), &mut margins);
        r.left += margins.cxLeftWidth + nPadW;
        r.top += margins.cyTopHeight + nPadH;
        r.right -= margins.cxRightWidth + nPadW;
        r.bottom -= margins.cyBottomHeight + nPadH;
    } else {
    // misc.c:222 `enum { FOCUSADJUST = 2 };` — same anonymous-enum-as-constant shape.
    const FOCUSADJUST: c_int = 2;
        InflateRect(&mut r, -(nPadW + FOCUSADJUST), -(nPadH + FOCUSADJUST));
    }

    hOrgPen = SelectObject((*pCD).hdc, GetStockObject(DC_PEN));
    hOrgBrush = SelectObject((*pCD).hdc, GetStockObject(DC_BRUSH));
    clrWidget = GetSysColor(COLOR_BTNTEXT);
    clrOrgPen = SetDCPenColor((*pCD).hdc, clrWidget);
    clrOrgBrush = SetDCBrushColor((*pCD).hdc, Color);

    nHalf = (r.bottom - r.top) / 2;
    Rectangle((*pCD).hdc, r.left, r.top, r.right - nHalf - SEPLINE * 3, r.bottom);

    SetDCBrushColor((*pCD).hdc, clrWidget);
    nArrow = ((r.bottom - r.top) / 3) & !1;
    pts[0].x = r.right - nHalf + nArrow / 2;
    pts[0].y = r.top + nHalf - nArrow / 3;
    pts[1].x = pts[0].x + nArrow;
    pts[1].y = pts[0].y;
    pts[2].x = pts[0].x + nArrow / 2;
    pts[2].y = pts[1].y + nArrow / 2;
    Polygon((*pCD).hdc, pts.as_mut_ptr(), _countof!(pts));

    r.left = r.right - nHalf - SEPLINE * 3 / 2;
    DrawEtchedLine(hTheme, (*pCD).hdc, BP_PUSHBUTTON, PBS_NORMAL, &mut r, BF_LEFT);

    SetDCBrushColor((*pCD).hdc, clrOrgBrush);
    SetDCPenColor((*pCD).hdc, clrOrgPen);
    SelectObject((*pCD).hdc, hOrgBrush);
    SelectObject((*pCD).hdc, hOrgPen);
    return CDRF_DODEFAULT;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "desk.h"
// #include <uxtheme.h>
// #include <vsstyle.h>
// #include <vssym32.h>
//
// static INT
// LengthOfStrResource( HINSTANCE hInst,
//                      UINT uID)
// {
//     HRSRC hrSrc;
//     HGLOBAL hRes;
//     LPWSTR lpName, lpStr;
//
//     if (hInst == NULL)
//     {
//         return -1;
//     }
//
//     /* There are always blocks of 16 strings */
//     lpName = (LPWSTR)MAKEINTRESOURCE((uID >> 4) + 1);
//
//     /* Find the string table block */
//     if ((hrSrc = FindResourceW(hInst, lpName, (LPWSTR)RT_STRING)) &&
//         (hRes = LoadResource(hInst, hrSrc)) &&
//         (lpStr = LockResource(hRes)))
//     {
//         UINT x;
//
//         /* Find the string we're looking for */
//         uID &= 0xF; /* Position in the block, same as % 16 */
//         for (x = 0; x < uID; x++)
//         {
//             lpStr += (*lpStr) + 1;
//         }
//
//         /* Found the string */
//         return (int)(*lpStr);
//     }
//     return -1;
// }
//
// INT
// AllocAndLoadString( LPTSTR *lpTarget,
//                     HINSTANCE hInst,
//                     UINT uID)
// {
//     INT ln;
//
//     ln = LengthOfStrResource(hInst,
//                              uID);
//     if (ln++ > 0)
//     {
//         (*lpTarget) = (LPTSTR)LocalAlloc(LMEM_FIXED,
//                                          ln * sizeof(TCHAR));
//         if ((*lpTarget) != NULL)
//         {
//             INT Ret;
//             if (!(Ret = LoadString(hInst, uID, *lpTarget, ln)))
//             {
//                 LocalFree((HLOCAL)(*lpTarget));
//             }
//             return Ret;
//         }
//     }
//     return 0;
// }
//
// static void
// DrawEtchedLine(HTHEME hTheme,
//                HDC hdc,
//                int iPart,
//                int iState,
//                LPRECT pRect,
//                UINT uFlags)
// {
//     if (hTheme)
//         DrawThemeEdge(hTheme, hdc, iPart, iState, pRect, EDGE_ETCHED, uFlags, pRect);
//     else
//         DrawEdge(hdc, pRect, EDGE_ETCHED, uFlags);
// }
//
// UINT
// ClrBtn_CustomDraw(NMCUSTOMDRAW *pCD,
//                   COLORREF Color)
// {
//     enum { MARGIN = 1, SEPLINE = 2 };
//     HTHEME hTheme;
//     HGDIOBJ hOrgPen, hOrgBrush;
//     COLORREF clrOrgPen, clrOrgBrush, clrWidget;
//     INT nPadW, nPadH, nHalf, nArrow;
//     RECT r = pCD->rc;
//     POINT pts[3];
//
//     if (pCD->dwDrawStage == CDDS_PREPAINT)
//         return CDRF_NOTIFYPOSTPAINT;
//     if (pCD->dwDrawStage != CDDS_POSTPAINT)
//         return CDRF_DODEFAULT;
//
//     nPadW = GetSystemMetrics(SM_CXFOCUSBORDER) + MARGIN;
//     nPadH = GetSystemMetrics(SM_CYFOCUSBORDER) + MARGIN;
//
//     hTheme = GetWindowTheme(pCD->hdr.hwndFrom);
//     if (hTheme)
//     {
//         MARGINS margins;
//         GetThemeMargins(hTheme, pCD->hdc, BP_PUSHBUTTON, PBS_NORMAL, TMT_CONTENTMARGINS, NULL, &margins);
//         r.left += margins.cxLeftWidth + nPadW;
//         r.top += margins.cyTopHeight + nPadH;
//         r.right -= margins.cxRightWidth + nPadW;
//         r.bottom -= margins.cyBottomHeight + nPadH;
//     }
//     else
//     {
//         enum { FOCUSADJUST = 2 }; /* From comctl32 button.c */
//         InflateRect(&r, -(nPadW + FOCUSADJUST), -(nPadH + FOCUSADJUST));
//     }
//
//     hOrgPen = SelectObject(pCD->hdc, GetStockObject(DC_PEN));
//     hOrgBrush = SelectObject(pCD->hdc, GetStockObject(DC_BRUSH));
//     clrWidget = GetSysColor(COLOR_BTNTEXT);
//     clrOrgPen = SetDCPenColor(pCD->hdc, clrWidget);
//     clrOrgBrush = SetDCBrushColor(pCD->hdc, Color);
//
//     nHalf = (r.bottom - r.top) / 2;
//     Rectangle(pCD->hdc, r.left, r.top, r.right - nHalf - SEPLINE * 3, r.bottom);
//
//     SetDCBrushColor(pCD->hdc, clrWidget);
//     nArrow = ((r.bottom - r.top) / 3) & ~1;
//     pts[0].x = r.right - nHalf + nArrow / 2;
//     pts[0].y = r.top + nHalf - nArrow / 3;
//     pts[1].x = pts[0].x + nArrow;
//     pts[1].y = pts[0].y;
//     pts[2].x = pts[0].x + nArrow / 2;
//     pts[2].y = pts[1].y + nArrow / 2;
//     Polygon(pCD->hdc, pts, _countof(pts));
//
//     r.left = r.right - nHalf - SEPLINE * 3 / 2;
//     DrawEtchedLine(hTheme, pCD->hdc, BP_PUSHBUTTON, PBS_NORMAL, &r, BF_LEFT);
//
//     SetDCBrushColor(pCD->hdc, clrOrgBrush);
//     SetDCPenColor(pCD->hdc, clrOrgPen);
//     SelectObject(pCD->hdc, hOrgBrush);
//     SelectObject(pCD->hdc, hOrgPen);
//     return CDRF_DODEFAULT;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
