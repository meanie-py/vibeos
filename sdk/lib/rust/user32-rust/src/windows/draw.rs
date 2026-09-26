use win32k_sys::win32k::*;
use core::ffi::c_void;
// user32-internal cohab symbols (UserApiHook trio, LoadUserApiHook, guah,
// gfServerProcess) live in the surviving C objects / internal.rs.
use crate::internal::{
    BeginIfHookedUserApiHook, EndUserApiHook, LoadUserApiHook, guah,
};

// RGB(r,g,b) is a wingdi.h macro, not a function — reimplement inline.
#[inline]
fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}

// ---- gdi32.dll / user32.dll exports (WINAPI = stdcall) ----
// PatBlt/SelectObject are gdi32 stdcall exports — must be extern "system" (the
// translator mis-placed them in an extern "C" block).
extern "system" {
    fn PatBlt(hdc: HDC, x: i32, y: i32, cx: i32, cy: i32, rop: u32) -> i32;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn GetStockObject(i: core::ffi::c_int) -> HGDIOBJ;
    fn GetSysColor(nIndex: core::ffi::c_int) -> COLORREF;
    fn LineTo(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int) -> BOOL;
    fn MoveToEx(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int, lppt: *mut POINT) -> BOOL;
    fn Polygon(hdc: HDC, apt: *const POINT, cpt: core::ffi::c_int) -> BOOL;
    fn Ellipse(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) -> BOOL;
    fn SetDCBrushColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetDCPenColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn BitBlt(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, hdcSrc: HDC, x1: core::ffi::c_int, y1: core::ffi::c_int, rop: u32) -> i32;
    fn CreateBitmap(nWidth: core::ffi::c_int, nHeight: core::ffi::c_int, nPlanes: u32, nBitCount: u32, lpBits: *const core::ffi::c_void) -> HBITMAP;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn CreateFontIndirectW(lplf: *const LOGFONTW) -> HFONT;
    fn DeleteDC(hdc: HDC) -> i32;
    fn DeleteObject(ho: HGDIOBJ) -> i32;
    fn GetCurrentObject(hdc: HDC, type_: u32) -> HGDIOBJ;
    fn GetObjectW(h: HGDIOBJ, c: core::ffi::c_int, pv: *mut core::ffi::c_void) -> core::ffi::c_int;
    fn GetTextExtentPoint32A(hdc: HDC, lpString: *const core::ffi::c_char, c: core::ffi::c_int, psizl: *mut SIZE) -> i32;
    fn GetTextExtentPoint32W(hdc: HDC, lpString: *const u16, c: core::ffi::c_int, psizl: *mut SIZE) -> i32;
    fn GetBkMode(hdc: HDC) -> i32;
    fn GetTextColor(hdc: HDC) -> COLORREF;
    fn GetLayout(hdc: HDC) -> DWORD;
    fn SetLayout(hdc: HDC, l: DWORD) -> DWORD;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetRect(lprc: LPRECT, xLeft: core::ffi::c_int, yTop: core::ffi::c_int, xRight: core::ffi::c_int, yBottom: core::ffi::c_int) -> i32;
    fn OffsetRect(lprc: LPRECT, dx: core::ffi::c_int, dy: core::ffi::c_int) -> i32;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn TextOutA(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int, lpString: *const core::ffi::c_char, c: core::ffi::c_int) -> i32;
    fn TextOutW(hdc: HDC, x: core::ffi::c_int, y: core::ffi::c_int, lpString: *const u16, c: core::ffi::c_int) -> i32;
    fn lstrlenA(lpString: *const core::ffi::c_char) -> core::ffi::c_int;
    fn lstrlenW(lpString: *const u16) -> core::ffi::c_int;
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    fn DrawIconEx(hdc: HDC, xLeft: INT, yTop: INT, hIcon: HICON, cxWidth: INT, cyHeight: INT, istepIfAniCur: UINT, hbrFlickerFreeDraw: HBRUSH, diFlags: UINT) -> BOOL;
    fn DrawTextA(hdc: HDC, lpchText: LPSTR, cchText: INT, lprc: LPRECT, format: UINT) -> INT;
    fn DrawTextW(hdc: HDC, lpchText: LPWSTR, cchText: INT, lprc: LPRECT, format: UINT) -> INT;
    fn OffsetViewportOrgEx(hdc: HDC, x: INT, y: INT, lppt: *mut POINT) -> BOOL;
    fn IsRectEmpty(lprc: *mut RECT) -> i32;
    fn GetSysColorBrush(nIndex: i32) -> HBRUSH;
    fn InflateRect(lprc: LPRECT, dx: i32, dy: i32) -> BOOL;
    fn GetMapMode(hdc: HDC) -> i32;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
}

// get_icon_size is a cdecl C helper defined in cursoricon.c (still C — Wave 3c
// hasn't flipped it). Must be extern "C" (cdecl), NOT extern "system".
extern "C" {
    fn get_icon_size(hIcon: HICON, size: *mut SIZE) -> BOOL;
}

// ---- module-local UNICODE/winnt helpers (user32 is built with UNICODE) ----
// Under UNICODE: TCHAR=WCHAR, TEXT(x)=L##x, TextOut=TextOutW, lstrcpy=lstrcpyW,
// CreateFontIndirect=CreateFontIndirectW. The bindgen TCHAR is c_char, so shadow
// it here as WCHAR to match the C's effective (UNICODE) types.
#[allow(non_camel_case_types)]
type TCHAR = WCHAR;

// ERR!(...) — ReactOS debug print; no-op in the cohab build.

/// winnt.h `PtrToUlong` — truncating pointer→u32.
#[inline]
unsafe fn PtrToUlong(p: *mut c_void) -> u32 {
    (p as usize) as u32
}

/// `TEXT(L"...")` facename literal — the only TEXT() use here is "Marlett".
/// Returns a stable pointer to a NUL-terminated UTF-16 array.
#[inline]
fn marlett_face() -> *const WCHAR {
    static MARLETT: [u16; 8] = [
        b'M' as u16, b'a' as u16, b'r' as u16, b'l' as u16,
        b'e' as u16, b't' as u16, b't' as u16, 0,
    ];
    MARLETT.as_ptr()
}

/// `TextOut` (UNICODE) → TextOutW.
#[inline]
unsafe fn TextOut(hdc: HDC, x: i32, y: i32, s: *const TCHAR, c: i32) -> i32 {
    TextOutW(hdc, x, y, s as *const u16, c)
}

/// `lstrcpy` (UNICODE) → lstrcpyW.
#[inline]
unsafe fn lstrcpy(dst: *mut TCHAR, src: *const TCHAR) -> LPWSTR {
    lstrcpyW(dst as LPWSTR, src as LPCWSTR)
}

/// `CreateFontIndirect` (UNICODE) → CreateFontIndirectW.
#[inline]
unsafe fn CreateFontIndirect(lplf: *const LOGFONTW) -> HFONT {
    CreateFontIndirectW(lplf)
}

/// `ZeroMemory(dst, len)` → RtlZeroMemory.
#[inline]
unsafe fn ZeroMemory(dst: *mut c_void, len: usize) {
    core::ptr::write_bytes(dst as *mut u8, 0, len);
}

// ---- file-scope preamble (translated) ----
pub static LTInnerNormal: [::core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, -1,
    -1, COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static LTOuterNormal: [::core::ffi::c_schar; 16] = [
    -1, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
    COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
    COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
    -1, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
];

pub static RBInnerNormal: [::core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
    -1, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_3DLIGHT as ::core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static RBOuterNormal: [::core::ffi::c_schar; 16] = [
    -1, COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, -1,
    COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, -1,
    COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, -1,
    -1, COLOR_3DDKSHADOW as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, -1,
];

pub static LTInnerSoft: [::core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_3DLIGHT as ::core::ffi::c_schar, -1,
    -1, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, -1,
    -1, -1, -1, -1,
];

pub static LTOuterSoft: [::core::ffi::c_schar; 16] = [
    -1, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, -1,
    COLOR_3DLIGHT as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, -1,
    COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, -1,
    -1, COLOR_BTNHIGHLIGHT as ::core::ffi::c_schar, COLOR_3DDKSHADOW as ::core::ffi::c_schar, -1,
];

pub static LTRBOuterMono: [::core::ffi::c_schar; 16] = [
    -1, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar,
    COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar,
    COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar,
    COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar, COLOR_WINDOWFRAME as ::core::ffi::c_schar,
];

pub static LTRBInnerMono: [::core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar,
    -1, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar,
    -1, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar, COLOR_WINDOW as ::core::ffi::c_schar,
];

pub static LTRBOuterFlat: [::core::ffi::c_schar; 16] = [
    -1, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar,
    COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar,
    COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar,
    COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar, COLOR_BTNSHADOW as ::core::ffi::c_schar,
];

pub static LTRBInnerFlat: [::core::ffi::c_schar; 16] = [
    -1, -1, -1, -1,
    -1, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar,
    -1, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar,
    -1, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar, COLOR_BTNFACE as ::core::ffi::c_schar,
];

unsafe fn IntDrawDiagEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT) -> BOOL {

    let mut Points: [POINT; 4] = core::mem::zeroed();
    let InnerI: i8;
    let OuterI: i8;
    let mut InnerPen: HPEN;
    let mut OuterPen: HPEN;
    let mut SavePoint: POINT = core::mem::zeroed();
    let SavePen: HPEN;
    let mut spx: core::ffi::c_int;
    let mut spy: core::ffi::c_int;
    let mut epx: core::ffi::c_int;
    let mut epy: core::ffi::c_int;
    let Width = (*rc).right - (*rc).left;
    let Height = (*rc).bottom - (*rc).top;
    let SmallDiam = if Width > Height { Height } else { Width };
    let retval: BOOL = (!((((uType & BDR_INNER) == BDR_INNER)
        || ((uType & BDR_OUTER) == BDR_OUTER))
        && ((uFlags & (BF_FLAT | BF_MONO)) == 0))) as BOOL;
    let idx = (uType & (BDR_INNER | BDR_OUTER)) as usize;
    let add = (if LTRBInnerMono[idx] != -1 { 1 } else { 0 })
        + (if LTRBOuterMono[idx] != -1 { 1 } else { 0 });

    OuterPen = GetStockObject(NULL_PEN as core::ffi::c_int) as HPEN;
    InnerPen = OuterPen;
    SavePen = SelectObject(hdc, InnerPen as HGDIOBJ) as HPEN;
    spx = 0;
    spy = 0;
    epx = 0;
    epy = 0;

    if (uFlags & BF_MONO) != 0 {
        InnerI = LTRBInnerMono[idx];
        OuterI = LTRBOuterMono[idx];
    } else if (uFlags & BF_FLAT) != 0 {
        InnerI = LTRBInnerFlat[idx];
        OuterI = LTRBOuterFlat[idx];
    } else if (uFlags & BF_SOFT) != 0 {
        if (uFlags & BF_BOTTOM) != 0 {
            InnerI = RBInnerNormal[idx];
            OuterI = RBOuterNormal[idx];
        } else {
            InnerI = LTInnerSoft[idx];
            OuterI = LTOuterSoft[idx];
        }
    } else {
        if (uFlags & BF_BOTTOM) != 0 {
            InnerI = RBInnerNormal[idx];
            OuterI = RBOuterNormal[idx];
        } else {
            InnerI = LTInnerNormal[idx];
            OuterI = LTOuterNormal[idx];
        }
    }

    if InnerI != -1 {
        InnerPen = GetStockObject(DC_PEN as core::ffi::c_int) as HPEN;
    }
    if OuterI != -1 {
        OuterPen = GetStockObject(DC_PEN as core::ffi::c_int) as HPEN;
    }

    MoveToEx(hdc, 0, 0, &mut SavePoint);

    match uFlags & BF_RECT {
        x if x == 0
            || x == BF_LEFT
            || x == BF_BOTTOM
            || x == BF_BOTTOMLEFT =>
        {
            epx = (*rc).left - 1;
            spx = epx + SmallDiam;
            epy = (*rc).bottom;
            spy = epy - SmallDiam;
        }
        x if x == BF_TOPLEFT || x == BF_BOTTOMRIGHT => {
            epx = (*rc).left - 1;
            spx = epx + SmallDiam;
            epy = (*rc).top - 1;
            spy = epy + SmallDiam;
        }
        x if x == BF_TOP
            || x == BF_RIGHT
            || x == BF_TOPRIGHT
            || x == (BF_RIGHT | BF_LEFT)
            || x == (BF_RIGHT | BF_LEFT | BF_TOP)
            || x == (BF_BOTTOM | BF_TOP)
            || x == (BF_BOTTOM | BF_TOP | BF_LEFT)
            || x == (BF_BOTTOMRIGHT | BF_LEFT)
            || x == (BF_BOTTOMRIGHT | BF_TOP)
            || x == BF_RECT =>
        {
            spx = (*rc).left;
            epx = spx + SmallDiam;
            spy = (*rc).bottom - 1;
            epy = spy - SmallDiam;
        }
        _ => {}
    }

    MoveToEx(hdc, spx, spy, core::ptr::null_mut());
    SelectObject(hdc, OuterPen as HGDIOBJ);
    SetDCPenColor(hdc, GetSysColor(OuterI as core::ffi::c_int));
    LineTo(hdc, epx, epy);

    SelectObject(hdc, InnerPen as HGDIOBJ);
    SetDCPenColor(hdc, GetSysColor(InnerI as core::ffi::c_int));

    match uFlags & (BF_RECT | BF_DIAGONAL) {
        x if x == BF_DIAGONAL_ENDBOTTOMLEFT
            || x == (BF_DIAGONAL | BF_BOTTOM)
            || x == BF_DIAGONAL
            || x == (BF_DIAGONAL | BF_LEFT) =>
        {
            MoveToEx(hdc, spx - 1, spy, core::ptr::null_mut());
            LineTo(hdc, epx, epy - 1);
            Points[0].x = spx - add;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).top;
            Points[2].x = epx + 1;
            Points[2].y = epy - 1 - add;
            Points[3] = Points[2];
        }
        x if x == BF_DIAGONAL_ENDBOTTOMRIGHT => {
            MoveToEx(hdc, spx - 1, spy, core::ptr::null_mut());
            LineTo(hdc, epx, epy + 1);
            Points[0].x = spx - add;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).bottom - 1;
            Points[2].x = epx + 1;
            Points[2].y = epy + 1 + add;
            Points[3] = Points[2];
        }
        x if x == (BF_DIAGONAL | BF_BOTTOM | BF_RIGHT | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_RIGHT | BF_TOP | BF_LEFT)
            || x == BF_DIAGONAL_ENDTOPRIGHT
            || x == (BF_DIAGONAL | BF_RIGHT | BF_TOP | BF_LEFT) =>
        {
            MoveToEx(hdc, spx + 1, spy, core::ptr::null_mut());
            LineTo(hdc, epx, epy + 1);
            Points[0].x = epx - 1;
            Points[0].y = epy + 1 + add;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top + add;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1;
            Points[3].x = spx + add;
            Points[3].y = spy;
        }
        x if x == BF_DIAGONAL_ENDTOPLEFT => {
            MoveToEx(hdc, spx, spy - 1, core::ptr::null_mut());
            LineTo(hdc, epx + 1, epy);
            Points[0].x = epx + 1 + add;
            Points[0].y = epy + 1;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1 - add;
            Points[3].x = spx;
            Points[3].y = spy - add;
        }
        x if x == (BF_DIAGONAL | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_TOP)
            || x == (BF_DIAGONAL | BF_BOTTOM | BF_TOP | BF_LEFT) =>
        {
            MoveToEx(hdc, spx + 1, spy - 1, core::ptr::null_mut());
            LineTo(hdc, epx, epy);
            Points[0].x = epx - 1;
            Points[0].y = epy + 1;
            Points[1].x = (*rc).right - 1;
            Points[1].y = (*rc).top;
            Points[2].x = (*rc).right - 1;
            Points[2].y = (*rc).bottom - 1 - add;
            Points[3].x = spx + add;
            Points[3].y = spy - add;
        }
        x if x == (BF_DIAGONAL | BF_RIGHT)
            || x == (BF_DIAGONAL | BF_RIGHT | BF_LEFT)
            || x == (BF_DIAGONAL | BF_RIGHT | BF_LEFT | BF_BOTTOM) =>
        {
            MoveToEx(hdc, spx, spy, core::ptr::null_mut());
            LineTo(hdc, epx - 1, epy + 1);
            Points[0].x = spx;
            Points[0].y = spy;
            Points[1].x = (*rc).left;
            Points[1].y = (*rc).top + add;
            Points[2].x = epx - 1 - add;
            Points[2].y = epy + 1 + add;
            Points[3] = Points[2];
        }
        _ => {}
    }

    if ((uFlags & BF_MIDDLE) != 0) && retval != 0 {
        let hbsave: HBRUSH = SelectObject(hdc, GetStockObject(DC_BRUSH as core::ffi::c_int)) as HBRUSH;
        let hpsave: HPEN = SelectObject(hdc, GetStockObject(DC_PEN as core::ffi::c_int)) as HPEN;
        SetDCBrushColor(hdc, GetSysColor(if (uFlags & BF_MONO) != 0 { COLOR_WINDOW as core::ffi::c_int } else { COLOR_BTNFACE as core::ffi::c_int }));
        SetDCPenColor(hdc, GetSysColor(if (uFlags & BF_MONO) != 0 { COLOR_WINDOW as core::ffi::c_int } else { COLOR_BTNFACE as core::ffi::c_int }));
        Polygon(hdc, Points.as_ptr(), 4);
        SelectObject(hdc, hbsave as HGDIOBJ);
        SelectObject(hdc, hpsave as HGDIOBJ);
    }

    if (uFlags & BF_ADJUST) != 0 {
        if (uFlags & BF_LEFT) != 0 {
            (*rc).left += add;
        }
        if (uFlags & BF_RIGHT) != 0 {
            (*rc).right -= add;
        }
        if (uFlags & BF_TOP) != 0 {
            (*rc).top += add;
        }
        if (uFlags & BF_BOTTOM) != 0 {
            (*rc).bottom -= add;
        }
    }

    SelectObject(hdc, SavePen as HGDIOBJ);
    MoveToEx(hdc, SavePoint.x, SavePoint.y, core::ptr::null_mut());

    retval
}

unsafe fn IntDrawRectEdge(hdc: HDC, rc: LPRECT, uType: UINT, uFlags: UINT, width: UINT) -> BOOL {

    let LTInnerI: i8;
    let LTOuterI: i8;
    let RBInnerI: i8;
    let RBOuterI: i8;
    let mut lti_brush: HBRUSH;
    let mut lto_brush: HBRUSH;
    let mut rbi_brush: HBRUSH;
    let mut rbo_brush: HBRUSH;
    let mut InnerRect: RECT = *rc;
    let mut fill_rect: RECT = core::mem::zeroed();
    let mut lbi_offset: i32 = 0;
    let mut lti_offset: i32 = 0;
    let mut rti_offset: i32 = 0;
    let mut rbi_offset: i32 = 0;

    let retval: BOOL = (!(((uType & BDR_INNER) == BDR_INNER
        || (uType & BDR_OUTER) == BDR_OUTER)
        && (uFlags & (BF_FLAT | BF_MONO)) == 0)) as BOOL;

    lti_brush = GetStockObject(NULL_BRUSH as i32) as HBRUSH;
    lto_brush = lti_brush;
    rbi_brush = lti_brush;
    rbo_brush = lti_brush;

    let idx = (uType & (BDR_INNER | BDR_OUTER)) as usize;

    if (uFlags & BF_MONO) != 0 {
        LTInnerI = LTRBInnerMono[idx];
        RBInnerI = LTInnerI;
        LTOuterI = LTRBOuterMono[idx];
        RBOuterI = LTOuterI;
    } else if (uFlags & BF_FLAT) != 0 {
        let mut lti = LTRBInnerFlat[idx];
        let mut rbi = lti;
        LTOuterI = LTRBOuterFlat[idx];
        RBOuterI = LTOuterI;
        if lti != -1 {
            lti = COLOR_BTNFACE as i8;
            rbi = lti;
        }
        LTInnerI = lti;
        RBInnerI = rbi;
    } else if (uFlags & BF_SOFT) != 0 {
        LTInnerI = LTInnerSoft[idx];
        LTOuterI = LTOuterSoft[idx];
        RBInnerI = RBInnerNormal[idx];
        RBOuterI = RBOuterNormal[idx];
    } else {
        LTInnerI = LTInnerNormal[idx];
        LTOuterI = LTOuterNormal[idx];
        RBInnerI = RBInnerNormal[idx];
        RBOuterI = RBOuterNormal[idx];
    }

    if (uFlags & BF_BOTTOMLEFT) == BF_BOTTOMLEFT { lbi_offset = width as i32; }
    if (uFlags & BF_TOPRIGHT) == BF_TOPRIGHT { rti_offset = width as i32; }
    if (uFlags & BF_BOTTOMRIGHT) == BF_BOTTOMRIGHT { rbi_offset = width as i32; }
    if (uFlags & BF_TOPLEFT) == BF_TOPLEFT { lti_offset = width as i32; }

    if LTInnerI != -1 { lti_brush = GetSysColorBrush(LTInnerI as i32); }
    if LTOuterI != -1 { lto_brush = GetSysColorBrush(LTOuterI as i32); }
    if RBInnerI != -1 { rbi_brush = GetSysColorBrush(RBInnerI as i32); }
    if RBOuterI != -1 { rbo_brush = GetSysColorBrush(RBOuterI as i32); }

    if (uFlags & BF_TOP) != 0 {
        fill_rect = InnerRect;
        fill_rect.bottom = fill_rect.top + width as i32;
        FillRect(hdc, &fill_rect, lto_brush);
    }
    if (uFlags & BF_LEFT) != 0 {
        fill_rect = InnerRect;
        fill_rect.right = fill_rect.left + width as i32;
        FillRect(hdc, &fill_rect, lto_brush);
    }
    if (uFlags & BF_BOTTOM) != 0 {
        fill_rect = InnerRect;
        fill_rect.top = fill_rect.bottom - width as i32;
        FillRect(hdc, &fill_rect, rbo_brush);
    }
    if (uFlags & BF_RIGHT) != 0 {
        fill_rect = InnerRect;
        fill_rect.left = fill_rect.right - width as i32;
        FillRect(hdc, &fill_rect, rbo_brush);
    }

    if (uFlags & BF_TOP) != 0 {
        SetRect(&mut fill_rect, InnerRect.left + lti_offset, InnerRect.top + width as i32,
            InnerRect.right - rti_offset, InnerRect.top + 2 * width as i32);
        FillRect(hdc, &fill_rect, lti_brush);
    }
    if (uFlags & BF_LEFT) != 0 {
        SetRect(&mut fill_rect, InnerRect.left + width as i32, InnerRect.top + lti_offset,
            InnerRect.left + 2 * width as i32, InnerRect.bottom - lbi_offset);
        FillRect(hdc, &fill_rect, lti_brush);
    }
    if (uFlags & BF_BOTTOM) != 0 {
        SetRect(&mut fill_rect, InnerRect.left + lbi_offset, InnerRect.bottom - 2 * width as i32,
            InnerRect.right - rbi_offset, InnerRect.bottom - width as i32);
        FillRect(hdc, &fill_rect, rbi_brush);
    }
    if (uFlags & BF_RIGHT) != 0 {
        SetRect(&mut fill_rect, InnerRect.right - 2 * width as i32, InnerRect.top + rti_offset,
            InnerRect.right - width as i32, InnerRect.bottom - rbi_offset);
        FillRect(hdc, &fill_rect, rbi_brush);
    }

    if (((uFlags & BF_MIDDLE) != 0 && retval != 0) || (uFlags & BF_ADJUST) != 0) {
        let add: i32 = (if LTRBInnerMono[idx] != -1 { width as i32 } else { 0 })
            + (if LTRBOuterMono[idx] != -1 { width as i32 } else { 0 });

        if (uFlags & BF_LEFT) != 0 { InnerRect.left += add; }
        if (uFlags & BF_RIGHT) != 0 { InnerRect.right -= add; }
        if (uFlags & BF_TOP) != 0 { InnerRect.top += add; }
        if (uFlags & BF_BOTTOM) != 0 { InnerRect.bottom -= add; }

        if (uFlags & BF_MIDDLE) != 0 && retval != 0 {
            FillRect(hdc, &InnerRect, GetSysColorBrush(if (uFlags & BF_MONO) != 0 {
                COLOR_WINDOW as i32
            } else {
                COLOR_BTNFACE as i32
            }));
        }

        if (uFlags & BF_ADJUST) != 0 {
            *rc = InnerRect;
        }
    }

    retval
}

unsafe fn UITOOLS_MakeSquareRect(src: LPRECT, dst: LPRECT) -> i32 {
    let Width: i32 = (*src).right - (*src).left;
    let Height: i32 = (*src).bottom - (*src).top;
    let SmallDiam: i32 = if Width > Height { Height } else { Width };

    *dst = *src;

    if Width < Height {
        (*dst).top += (Height - Width) / 2;
        (*dst).bottom = (*dst).top + SmallDiam;
    } else if Width > Height {
        (*dst).left += (Width - Height) / 2;
        (*dst).right = (*dst).left + SmallDiam;
    }

    SmallDiam
}

unsafe fn UITOOLS_DrawCheckedRect(dc: HDC, rect: LPRECT) {

    if GetSysColor(COLOR_BTNHIGHLIGHT as i32) == RGB(255, 255, 255) {
        FillRect(dc, rect, GetSysColorBrush(COLOR_BTNFACE as i32));
        let bg = SetBkColor(dc, RGB(255, 255, 255));
        let hbsave = SelectObject(dc, if !gpsi.is_null() { crate::wow64::psi::hbr_gray() as HGDIOBJ } else { core::ptr::null_mut() }) as HBRUSH;
        PatBlt(dc, (*rect).left, (*rect).top, (*rect).right - (*rect).left, (*rect).bottom - (*rect).top, 0x00FA0089);
        SelectObject(dc, hbsave as HGDIOBJ);
        SetBkColor(dc, bg);
    } else {
        FillRect(dc, rect, GetSysColorBrush(COLOR_BTNHIGHLIGHT as i32));
    }
}

unsafe fn UITOOLS95_DFC_ButtonPush(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let edge: UINT;
    let mut myr: RECT = *r;

    if uFlags & (DFCS_PUSHED | DFCS_CHECKED | DFCS_FLAT) != 0 {
        edge = EDGE_SUNKEN;
    } else {
        edge = EDGE_RAISED;
    }

    if uFlags & DFCS_CHECKED != 0 {
        if uFlags & DFCS_MONO != 0 {
            IntDrawRectEdge(dc, &mut myr, edge, BF_MONO | BF_RECT | BF_ADJUST, 1);
        } else {
            IntDrawRectEdge(dc, &mut myr, edge, (uFlags & DFCS_FLAT) | BF_RECT | BF_SOFT | BF_ADJUST, 1);
        }

        UITOOLS_DrawCheckedRect(dc, &mut myr);
    } else {
        if uFlags & DFCS_MONO != 0 {
            IntDrawRectEdge(dc, &mut myr, edge, BF_MONO | BF_RECT | BF_ADJUST, 1);
            FillRect(dc, &myr, GetSysColorBrush(COLOR_BTNFACE as i32));
        } else {
            IntDrawRectEdge(dc, r, edge, (uFlags & DFCS_FLAT) | BF_MIDDLE | BF_RECT | BF_SOFT, 1);
        }
    }

    if uFlags & DFCS_ADJUSTRECT != 0 {
        InflateRect(r, -2, -2);
    }

    1
}

unsafe fn UITOOLS95_DFC_ButtonCheckRadio(dc: HDC, r: LPRECT, uFlags: UINT, Radio: BOOL) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let i: i32;
    let OutRight: TCHAR;
    let OutLeft: TCHAR;
    let InRight: TCHAR;
    let InLeft: TCHAR;
    let Center: TCHAR;
    let X: INT;
    let Y: INT;
    let Width: INT;
    let Height: INT;
    let Shorter: INT;
    let BkMode: INT = GetBkMode(dc);
    let TextColor: COLORREF = GetTextColor(dc);

    Width = (*r).right - (*r).left;
    Height = (*r).bottom - (*r).top;
    Shorter = if Width < Height { Width } else { Height };
    X = (*r).left + (Width - Shorter) / 2;
    Y = (*r).top + (Height - Shorter) / 2;

    if Radio != 0 {
        OutRight = b'j' as TCHAR;
        OutLeft = b'k' as TCHAR;
        InRight = b'l' as TCHAR;
        InLeft = b'm' as TCHAR;
        Center = b'n' as TCHAR;
    } else {
        OutRight = b'c' as TCHAR;
        OutLeft = b'd' as TCHAR;
        InRight = b'e' as TCHAR;
        InLeft = b'f' as TCHAR;
        Center = b'g' as TCHAR;
    }

    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    lf.lfHeight = Shorter;
    lf.lfCharSet = DEFAULT_CHARSET as _;
    let marlett: [u16; 8] = [
        b'M' as u16, b'a' as u16, b'r' as u16, b'l' as u16,
        b'e' as u16, b't' as u16, b't' as u16, 0,
    ];
    lstrcpy(lf.lfFaceName.as_mut_ptr(), marlett.as_ptr());
    if Radio != 0 && (uFlags & 0xFF) == DFCS_BUTTONRADIOMASK {
        lf.lfQuality = NONANTIALIASED_QUALITY as _;
    }
    // C: hFont = CreateFontIndirect(&lf); hOldFont = SelectObject(dc, hFont);
    let hFont: HFONT = CreateFontIndirect(&mut lf);
    let hOldFont: HFONT = SelectObject(dc, hFont as HGDIOBJ) as HFONT;

    if Radio != 0 && (uFlags & 0xFF) == DFCS_BUTTONRADIOMASK {
        let mut Rect: RECT = core::mem::zeroed();
        let hbrOld: HGDIOBJ;
        let hpenOld: HGDIOBJ;
        SetRect(&mut Rect, X, Y, X + Shorter, Y + Shorter);
        InflateRect(&mut Rect, -(Shorter * 8) / 54, -(Shorter * 8) / 54);
        hbrOld = SelectObject(dc, GetStockObject(BLACK_BRUSH as i32));
        hpenOld = SelectObject(dc, GetStockObject(NULL_PEN as i32));
        Ellipse(dc, Rect.left, Rect.top, Rect.right, Rect.bottom);
        SelectObject(dc, hbrOld);
        SelectObject(dc, hpenOld);
    } else {
        SetBkMode(dc, TRANSPARENT as i32);

        if uFlags & (DFCS_INACTIVE | DFCS_PUSHED) != 0 {
            i = COLOR_BTNFACE as i32;
        } else {
            i = COLOR_WINDOW as i32;
        }
        SetTextColor(dc, GetSysColor(i));
        TextOut(dc, X, Y, &Center, 1);

        if uFlags & (DFCS_FLAT | DFCS_MONO) != 0 {
            SetTextColor(dc, GetSysColor(COLOR_WINDOWFRAME as i32));
            TextOut(dc, X, Y, &OutRight, 1);
            TextOut(dc, X, Y, &OutLeft, 1);
            TextOut(dc, X, Y, &InRight, 1);
            TextOut(dc, X, Y, &InLeft, 1);
        } else {
            SetTextColor(dc, GetSysColor(COLOR_BTNSHADOW as i32));
            TextOut(dc, X, Y, &OutRight, 1);
            SetTextColor(dc, GetSysColor(COLOR_BTNHIGHLIGHT as i32));
            TextOut(dc, X, Y, &OutLeft, 1);
            SetTextColor(dc, GetSysColor(COLOR_3DDKSHADOW as i32));
            TextOut(dc, X, Y, &InRight, 1);
            SetTextColor(dc, GetSysColor(COLOR_3DLIGHT as i32));
            TextOut(dc, X, Y, &InLeft, 1);
        }

        if uFlags & DFCS_CHECKED != 0 {
            let Check: TCHAR = if Radio != 0 { b'i' as TCHAR } else { b'b' as TCHAR };

            SetTextColor(dc, GetSysColor(if uFlags & DFCS_INACTIVE != 0 {
                COLOR_BTNSHADOW as i32
            } else {
                COLOR_WINDOWTEXT as i32
            }));
            TextOut(dc, X, Y, &Check, 1);
        }
    }

    SelectObject(dc, hOldFont as HGDIOBJ);
    DeleteObject(hFont as HGDIOBJ);

    SetTextColor(dc, TextColor);
    SetBkMode(dc, BkMode);

    1
}

unsafe fn UITOOLS95_DrawFrameButton(hdc: HDC, rc: LPRECT, uState: UINT) -> BOOL {
    match uState & 0x1f {
        DFCS_BUTTONPUSH => {
            return UITOOLS95_DFC_ButtonPush(hdc, rc, uState);
        }
        DFCS_BUTTONCHECK | DFCS_BUTTON3STATE => {
            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 0);
        }
        DFCS_BUTTONRADIOIMAGE | DFCS_BUTTONRADIOMASK => {
            if uState & DFCS_BUTTONRADIOIMAGE != 0 {
                FillRect(hdc, rc, GetStockObject(BLACK_BRUSH as i32) as HBRUSH);
            } else {
                FillRect(hdc, rc, GetStockObject(WHITE_BRUSH as i32) as HBRUSH);
            }
            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 1);
        }
        DFCS_BUTTONRADIO => {
            return UITOOLS95_DFC_ButtonCheckRadio(hdc, rc, uState, 1);
        }
        _ => {}
    }

    0
}

unsafe fn UITOOLS95_DrawFrameCaption(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let clrsave: COLORREF;
    let mut myr: RECT = core::mem::zeroed();
    let bkmode: INT;
    let Symbol: TCHAR;

    match uFlags & 0xf {
        x if x == DFCS_CAPTIONCLOSE => {
            Symbol = b'r' as TCHAR;
        }
        x if x == DFCS_CAPTIONHELP => {
            Symbol = b's' as TCHAR;
        }
        x if x == DFCS_CAPTIONMIN => {
            Symbol = b'0' as TCHAR;
        }
        x if x == DFCS_CAPTIONMAX => {
            Symbol = b'1' as TCHAR;
        }
        x if x == DFCS_CAPTIONRESTORE => {
            Symbol = b'2' as TCHAR;
        }
        _ => {
            ERR!("Invalid caption; flags=0x%04x\n", uFlags);
            return 0;
        }
    }

    IntDrawRectEdge(
        dc,
        r,
        if (uFlags & DFCS_PUSHED) != 0 { EDGE_SUNKEN } else { EDGE_RAISED },
        BF_RECT | BF_MIDDLE | BF_SOFT,
        1,
    );

    ZeroMemory(
        core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>() as _,
    );

    UITOOLS_MakeSquareRect(r, core::ptr::addr_of_mut!(myr));
    myr.left += 1;
    myr.top += 1;
    myr.right -= 1;
    myr.bottom -= 1;

    if (uFlags & DFCS_PUSHED) != 0 {
        OffsetRect(core::ptr::addr_of_mut!(myr), 1, 1);
    }

    lf.lfHeight = myr.bottom - myr.top;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as _;
    lf.lfCharSet = DEFAULT_CHARSET as _;

    lstrcpy(
        lf.lfFaceName.as_mut_ptr(),
        marlett_face(),
    );

    hFont = CreateFontIndirect(core::ptr::addr_of!(lf));

    hOldFont = SelectObject(dc, hFont as HGDIOBJ) as HFONT;
    clrsave = GetTextColor(dc);
    bkmode = GetBkMode(dc);

    SetBkMode(dc, TRANSPARENT as _);

    if (uFlags & DFCS_INACTIVE) != 0 {
        SetTextColor(dc, GetSysColor(COLOR_BTNHIGHLIGHT as _));
        TextOut(dc, myr.left + 1, myr.top + 1, core::ptr::addr_of!(Symbol) as *const TCHAR, 1);
    }

    SetTextColor(
        dc,
        GetSysColor(if (uFlags & DFCS_INACTIVE) != 0 {
            COLOR_BTNSHADOW as _
        } else {
            COLOR_BTNTEXT as _
        }),
    );

    TextOut(dc, myr.left, myr.top, core::ptr::addr_of!(Symbol) as *const TCHAR, 1);

    SetTextColor(dc, clrsave);
    SelectObject(dc, hOldFont as HGDIOBJ);
    SetBkMode(dc, bkmode);
    DeleteObject(hFont as HGDIOBJ);

    1
}

unsafe fn UITOOLS95_DrawFrameScroll(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut hFont: HFONT;
    let mut hOldFont: HFONT;
    let mut clrsave: COLORREF;
    let mut myr: RECT = core::mem::zeroed();
    let mut bkmode: INT;
    let mut Symbol: TCHAR;

    match uFlags & 0x1f {
        x if x == DFCS_SCROLLCOMBOBOX || x == DFCS_SCROLLDOWN => {
            Symbol = b'6' as TCHAR;
        }
        x if x == DFCS_SCROLLUP => {
            Symbol = b'5' as TCHAR;
        }
        x if x == DFCS_SCROLLLEFT => {
            Symbol = b'3' as TCHAR;
        }
        x if x == DFCS_SCROLLRIGHT => {
            Symbol = b'4' as TCHAR;
        }
        x if x == DFCS_SCROLLSIZEGRIP || x == DFCS_SCROLLSIZEGRIPRIGHT => {
            core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
            UITOOLS_MakeSquareRect(r, &mut myr);
            lf.lfHeight = myr.bottom - myr.top;
            lf.lfWidth = 0;
            lf.lfWeight = FW_NORMAL as i32;
            lf.lfCharSet = DEFAULT_CHARSET as u8;
            lstrcpy(lf.lfFaceName.as_mut_ptr(), marlett_face());
            hFont = CreateFontIndirect(&mut lf);
            hOldFont = SelectObject(dc, hFont as HGDIOBJ) as HFONT;
            clrsave = GetTextColor(dc);
            bkmode = GetBkMode(dc);
            SetBkMode(dc, TRANSPARENT as i32);
            if (uFlags & (DFCS_MONO | DFCS_FLAT)) == 0 {
                SetTextColor(dc, GetSysColor(COLOR_BTNHIGHLIGHT as i32));
                Symbol = if (uFlags & 0xff) == DFCS_SCROLLSIZEGRIP { b'o' as TCHAR } else { b'x' as TCHAR };
                TextOut(dc, myr.left, myr.top, &Symbol, 1);
                SetTextColor(dc, GetSysColor(COLOR_BTNSHADOW as i32));
            } else {
                SetTextColor(dc, GetSysColor(COLOR_WINDOWFRAME as i32));
            }
            Symbol = if (uFlags & 0xff) == DFCS_SCROLLSIZEGRIP { b'p' as TCHAR } else { b'y' as TCHAR };
            TextOut(dc, myr.left, myr.top, &Symbol, 1);
            SetTextColor(dc, clrsave);
            SelectObject(dc, hOldFont as HGDIOBJ);
            SetBkMode(dc, bkmode);
            DeleteObject(hFont as HGDIOBJ);
            return 1;
        }
        _ => {
            ERR!("Invalid scroll; flags=0x%04x\n", uFlags);
            return 0;
        }
    }

    IntDrawRectEdge(
        dc,
        r,
        if (uFlags & DFCS_PUSHED) != 0 { EDGE_SUNKEN } else { EDGE_RAISED },
        (uFlags & DFCS_FLAT) | BF_MIDDLE | BF_RECT,
        1,
    );
    core::ptr::write_bytes(&mut lf as *mut LOGFONTW, 0, 1);
    UITOOLS_MakeSquareRect(r, &mut myr);
    myr.left += 1;
    myr.top += 1;
    myr.right -= 1;
    myr.bottom -= 1;
    if (uFlags & DFCS_PUSHED) != 0 {
        OffsetRect(&mut myr, 1, 1);
    }
    lf.lfHeight = myr.bottom - myr.top;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as i32;
    lf.lfCharSet = DEFAULT_CHARSET as u8;
    lstrcpy(lf.lfFaceName.as_mut_ptr(), marlett_face());
    hFont = CreateFontIndirect(&mut lf);
    hOldFont = SelectObject(dc, hFont as HGDIOBJ) as HFONT;
    clrsave = GetTextColor(dc);
    bkmode = GetBkMode(dc);
    SetBkMode(dc, TRANSPARENT as i32);
    if (uFlags & DFCS_INACTIVE) != 0 {
        SetTextColor(dc, GetSysColor(COLOR_BTNHIGHLIGHT as i32));
        TextOut(dc, myr.left + 1, myr.top + 1, &Symbol, 1);
    }
    SetTextColor(
        dc,
        GetSysColor(if (uFlags & DFCS_INACTIVE) != 0 { COLOR_BTNSHADOW as i32 } else { COLOR_BTNTEXT as i32 }),
    );
    TextOut(dc, myr.left, myr.top, &Symbol, 1);
    SetTextColor(dc, clrsave);
    SelectObject(dc, hOldFont as HGDIOBJ);
    SetBkMode(dc, bkmode);
    DeleteObject(hFont as HGDIOBJ);
    1
}

unsafe fn UITOOLS95_DrawFrameMenu(dc: HDC, r: LPRECT, uFlags: UINT) -> BOOL {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFont: HFONT;
    let hOldFont: HFONT;
    let Symbol: TCHAR;
    let mut myr: RECT = core::mem::zeroed();
    let cxy: INT;

    cxy = UITOOLS_MakeSquareRect(r, &mut myr);

    match uFlags & 0x1f {
        x if x == DFCS_MENUARROWUP => {
            Symbol = b'5' as TCHAR;
        }
        x if x == DFCS_MENUARROWDOWN => {
            Symbol = b'6' as TCHAR;
        }
        x if x == DFCS_MENUARROW => {
            Symbol = b'8' as TCHAR;
        }
        x if x == DFCS_MENUARROWRIGHT => {
            Symbol = b'w' as TCHAR;
        }
        x if x == DFCS_MENUBULLET => {
            Symbol = b'h' as TCHAR;
        }
        x if x == DFCS_MENUCHECK || x == (DFCS_MENUCHECK | DFCS_MENUBULLET) => {
            Symbol = b'a' as TCHAR;
        }
        _ => {
            ERR!("Invalid menu; flags=0x%04x\n", uFlags);
            return 0;
        }
    }

    ZeroMemory(&mut lf as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LOGFONTW>());
    lf.lfHeight = cxy;
    lf.lfWidth = 0;
    lf.lfWeight = FW_NORMAL as _;
    lf.lfCharSet = DEFAULT_CHARSET as _;
    lstrcpy(lf.lfFaceName.as_mut_ptr(), marlett_face());

    hFont = CreateFontIndirect(&mut lf);
    hOldFont = SelectObject(dc, hFont as HGDIOBJ) as HFONT;

    if (uFlags & 0x1f) == DFCS_MENUARROWUP || (uFlags & 0x1f) == DFCS_MENUARROWDOWN {
        SetTextColor(
            dc,
            GetSysColor(if (uFlags & DFCS_INACTIVE) != 0 {
                COLOR_BTNSHADOW as i32
            } else {
                COLOR_BTNTEXT as i32
            }),
        );
    }

    TextOut(dc, myr.left, myr.top, &Symbol, 1);

    SelectObject(dc, hOldFont as HGDIOBJ);
    DeleteObject(hFont as HGDIOBJ);
    1
}

unsafe fn IntGrayString(
    hDC: HDC,
    mut hBrush: HBRUSH,
    lpOutputFunc: GRAYSTRINGPROC,
    lpData: LPARAM,
    mut nCount: core::ffi::c_int,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    mut nWidth: core::ffi::c_int,
    mut nHeight: core::ffi::c_int,
    unicode: i32,
) -> i32 {

    let mut success: i32 = 0;
    let mut hbsave: HBRUSH;
    let mut MemDC: HDC = core::ptr::null_mut();
    let mut MemBMP: HBITMAP = core::ptr::null_mut();
    let mut OldBMP: HBITMAP = core::ptr::null_mut();
    let mut OldBrush: HBRUSH = core::ptr::null_mut();
    let mut OldFont: HFONT = core::ptr::null_mut();
    let mut r: RECT = core::mem::zeroed();
    let ForeColor: COLORREF;
    let BackColor: COLORREF;

    ForeColor = SetTextColor(hDC, RGB(0, 0, 0));
    BackColor = SetBkColor(hDC, RGB(255, 255, 255));

    if hBrush.is_null() {
        hBrush = GetCurrentObject(hDC, OBJ_BRUSH) as HBRUSH;
    }

    if nCount == -1 && lpOutputFunc.is_none() {
        return 0;
    }

    if nCount == 0 {
        if unicode != 0 {
            nCount = lstrlenW(lpData as *const u16);
        } else {
            nCount = lstrlenA(lpData as *const core::ffi::c_char);
        }
    }

    'cleanup: loop {
        if nWidth == 0 || nHeight == 0 {
            let mut s: SIZE = core::mem::zeroed();

            if unicode != 0 {
                success = GetTextExtentPoint32W(hDC, lpData as *const u16, nCount, &mut s);
            } else {
                success = GetTextExtentPoint32A(hDC, lpData as *const core::ffi::c_char, nCount, &mut s);
            }

            if success == 0 {
                break 'cleanup;
            }

            if nWidth == 0 {
                nWidth = s.cx;
            }
            if nHeight == 0 {
                nHeight = s.cy;
            }
        }

        SetRect(&mut r, X, Y, X + nWidth, Y + nHeight);

        MemDC = CreateCompatibleDC(hDC);
        if MemDC.is_null() {
            break 'cleanup;
        }
        MemBMP = CreateBitmap(nWidth, nHeight, 1, 1, core::ptr::null_mut());
        if MemBMP.is_null() {
            break 'cleanup;
        }
        OldBMP = SelectObject(MemDC, MemBMP as HGDIOBJ) as HBITMAP;
        if OldBMP.is_null() {
            break 'cleanup;
        }
        OldFont = SelectObject(MemDC, GetCurrentObject(hDC, OBJ_FONT)) as HFONT;
        if OldFont.is_null() {
            break 'cleanup;
        }
        OldBrush = SelectObject(MemDC, hBrush as HGDIOBJ) as HBRUSH;
        if OldBrush.is_null() {
            break 'cleanup;
        }

        if BitBlt(MemDC, 0, 0, nWidth, nHeight, hDC, X, Y, SRCCOPY) == 0 {
            break 'cleanup;
        }

        SetTextColor(MemDC, RGB(255, 255, 255));
        SetBkColor(MemDC, RGB(0, 0, 0));

        if let Some(func) = lpOutputFunc {
            success = func(MemDC, lpData, nCount);

            if nCount == -1 && success == 0 {
                success = BitBlt(hDC, X, Y, nWidth, nHeight, MemDC, 0, 0, SRCCOPY);
                break 'cleanup;
            }
        } else {
            if unicode != 0 {
                success = TextOutW(MemDC, 0, 0, lpData as *const u16, nCount);
            } else {
                success = TextOutA(MemDC, 0, 0, lpData as *const core::ffi::c_char, nCount);
            }

            if success == 0 {
                break 'cleanup;
            }

            hbsave = SelectObject(MemDC, if !gpsi.is_null() { crate::wow64::psi::hbr_gray() as HGDIOBJ } else { core::ptr::null_mut() }) as HBRUSH;
            PatBlt(MemDC, 0, 0, nWidth, nHeight, 0x000A0329);
            SelectObject(MemDC, hbsave as HGDIOBJ);
        }

        if BitBlt(hDC, X, Y, nWidth, nHeight, MemDC, 0, 0, SRCCOPY) == 0 {
            break 'cleanup;
        }

        break 'cleanup;
    }

    SetTextColor(hDC, ForeColor);
    SetBkColor(hDC, BackColor);

    if !MemDC.is_null() {
        if !OldFont.is_null() {
            SelectObject(MemDC, OldFont as HGDIOBJ);
        }
        if !OldBrush.is_null() {
            SelectObject(MemDC, OldBrush as HGDIOBJ);
        }
        if !OldBMP.is_null() {
            SelectObject(MemDC, OldBMP as HGDIOBJ);
        }
        if !MemBMP.is_null() {
            DeleteObject(MemBMP as HGDIOBJ);
        }
        DeleteDC(MemDC);
    }

    success
}

unsafe fn PAINTING_DrawStateJam(hdc: HDC, opcode: UINT,
                                func: DRAWSTATEPROC, lp: LPARAM, wp: WPARAM,
                                rc: LPRECT, dtflags: UINT, unicode: BOOL) -> BOOL {
    let cx: INT = (*rc).right - (*rc).left;
    let cy: INT = (*rc).bottom - (*rc).top;

    match opcode {
        x if x == DST_TEXT || x == DST_PREFIXTEXT => {
            if unicode != 0 {
                return DrawTextW(hdc, lp as LPWSTR, wp as INT, rc, dtflags);
            } else {
                return DrawTextA(hdc, lp as LPSTR, wp as INT, rc, dtflags);
            }
        }

        x if x == DST_ICON => {
            return DrawIconEx(hdc, (*rc).left, (*rc).top, lp as HICON, cx, cy, 0, core::ptr::null_mut(), DI_NORMAL);
        }

        x if x == DST_BITMAP => {
            let memdc: HDC = CreateCompatibleDC(hdc);
            if memdc.is_null() {
                return 0;
            }
            let hbmsave: HBITMAP = SelectObject(memdc, lp as HGDIOBJ) as HBITMAP;
            if hbmsave.is_null() {
                DeleteDC(memdc);
                return 0;
            }
            let retval: BOOL = BitBlt(hdc, (*rc).left, (*rc).top, cx, cy, memdc, 0, 0, SRCCOPY);
            SelectObject(memdc, hbmsave as HGDIOBJ);
            DeleteDC(memdc);
            return retval;
        }

        x if x == DST_COMPLEX => {
            if let Some(f) = func {
                OffsetViewportOrgEx(hdc, (*rc).left, (*rc).top, core::ptr::null_mut());
                let bRet: BOOL = f(hdc, lp, wp, cx, cy);
                OffsetViewportOrgEx(hdc, -(*rc).left, -(*rc).top, core::ptr::null_mut());
                return bRet;
            } else {
                return 0;
            }
        }

        _ => {}
    }
    0
}

unsafe fn IntDrawState(hdc: HDC, mut hbr: HBRUSH, func: DRAWSTATEPROC, lp: LPARAM, wp: WPARAM,
                       x: INT, y: INT, mut cx: INT, mut cy: INT, flags: UINT, unicode: BOOL) -> BOOL
{
    let mut hbm: HBITMAP = 0 as HBITMAP;
    let mut hbmsave: HBITMAP;
    let hfsave: HFONT;
    let mut hbsave: HBRUSH = 0 as HBRUSH;
    let mut hbrtmp: HBRUSH = 0 as HBRUSH;
    let mut memdc: HDC = 0 as HDC;
    let mut rc: RECT = core::mem::zeroed();
    let mut dtflags: UINT = DT_NOCLIP;
    let mut fg: COLORREF = 0;
    let mut bg: COLORREF = 0;
    let opcode: UINT = flags & 0xf;
    let mut len: INT = wp as INT;
    let mut retval: BOOL;
    let mut tmp: BOOL;
    let mut lf: LOGFONTW = core::mem::zeroed();
    let hFontOriginal: HFONT;
    let mut hNaaFont: HFONT = 0 as HFONT;

    if (opcode == DST_TEXT || opcode == DST_PREFIXTEXT) && len == 0
    {
        if unicode != 0 {
            len = lstrlenW(lp as LPWSTR);
        } else {
            len = lstrlenA(lp as LPSTR);
        }
    }

    hFontOriginal = GetCurrentObject(hdc, OBJ_FONT) as HFONT;
    if (flags & (DSS_MONO | DSS_DISABLED)) != 0
    {
        GetObjectW(hFontOriginal as HGDIOBJ, core::mem::size_of::<LOGFONTW>() as i32,
                   core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void);
        lf.lfQuality = NONANTIALIASED_QUALITY as u8;
        hNaaFont = CreateFontIndirectW(core::ptr::addr_of!(lf));
    }

    if cx == 0 || cy == 0
    {
        let mut s: SIZE = core::mem::zeroed();
        let mut bm: BITMAP = core::mem::zeroed();

        match opcode
        {
            x if x == DST_TEXT || x == DST_PREFIXTEXT =>
            {
                if unicode != 0 {
                    retval = GetTextExtentPoint32W(hdc, lp as LPWSTR, len, core::ptr::addr_of_mut!(s));
                } else {
                    retval = GetTextExtentPoint32A(hdc, lp as LPSTR, len, core::ptr::addr_of_mut!(s));
                }
                if retval == 0 {
                    return 0;
                }
            }
            x if x == DST_ICON =>
            {
                if get_icon_size(lp as HICON, core::ptr::addr_of_mut!(s)) == 0 {
                    return 0;
                }
            }
            x if x == DST_BITMAP =>
            {
                if GetObjectW(lp as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32,
                              core::ptr::addr_of_mut!(bm) as *mut core::ffi::c_void) == 0 {
                    return 0;
                }
                s.cx = bm.bmWidth;
                s.cy = bm.bmHeight;
            }
            x if x == DST_COMPLEX =>
            {
                return 0;
            }
            _ =>
            {
                ERR!("Invalid opcode: %u\n", opcode);
                return 0;
            }
        }

        if cx == 0 { cx = s.cx; }
        if cy == 0 { cy = s.cy; }
    }

    SetRect(core::ptr::addr_of_mut!(rc), x, y, x + cx, y + cy);

    if (flags & DSS_RIGHT) != 0 {
        dtflags |= DT_RIGHT;
    }
    if opcode == DST_TEXT {
        dtflags |= DT_NOPREFIX;
    } else if opcode == DST_PREFIXTEXT {
        if (flags & DSS_HIDEPREFIX) != 0 {
            dtflags |= DT_HIDEPREFIX;
        }
        if (flags & DSS_PREFIXONLY) != 0 {
            dtflags |= DT_PREFIXONLY;
        }
    }

    if (flags & 0x79f0) == DSS_NORMAL
    {
        return PAINTING_DrawStateJam(hdc, opcode, func, lp, len as WPARAM, core::ptr::addr_of_mut!(rc), dtflags, unicode);
    }

    fg = SetTextColor(hdc, RGB(0, 0, 0));
    bg = SetBkColor(hdc, RGB(255, 255, 255));
    hbm = 0 as HBITMAP;
    hbmsave = 0 as HBITMAP;
    memdc = 0 as HDC;
    hbsave = 0 as HBRUSH;
    retval = 0;

    'cleanup: {
        hbm = CreateBitmap(cx, cy, 1, 1, core::ptr::null_mut());
        if hbm.is_null() { break 'cleanup; }
        memdc = CreateCompatibleDC(hdc);
        if memdc.is_null() { break 'cleanup; }
        hbmsave = SelectObject(memdc, hbm as HGDIOBJ) as HBITMAP;
        if hbmsave.is_null() { break 'cleanup; }
        SetRect(core::ptr::addr_of_mut!(rc), 0, 0, cx, cy);
        if FillRect(memdc, core::ptr::addr_of!(rc), GetStockObject(WHITE_BRUSH as i32) as HBRUSH) == 0 { break 'cleanup; }
        SetBkColor(memdc, RGB(255, 255, 255));
        SetTextColor(memdc, RGB(0, 0, 0));
        if !hNaaFont.is_null() {
            hfsave = SelectObject(memdc, hNaaFont as HGDIOBJ) as HFONT;
        } else {
            hfsave = SelectObject(memdc, hFontOriginal as HGDIOBJ) as HFONT;
        }
        SetLayout(memdc, GetLayout(hdc));

        if hfsave.is_null() && (opcode <= DST_PREFIXTEXT) { break 'cleanup; }
        tmp = PAINTING_DrawStateJam(memdc, opcode, func, lp, len as WPARAM, core::ptr::addr_of_mut!(rc), dtflags, unicode);
        if !hfsave.is_null() { SelectObject(memdc, hfsave as HGDIOBJ); }
        if !hNaaFont.is_null() { DeleteObject(hNaaFont as HGDIOBJ); }
        if tmp == 0 { break 'cleanup; }

        if (flags & DSS_UNION) != 0
        {
            let gray = if !gpsi.is_null() { crate::wow64::psi::hbr_gray() } else { 0 as HBRUSH };
            hbsave = SelectObject(memdc, gray as HGDIOBJ) as HBRUSH;
            if hbsave.is_null() { break 'cleanup; }
            tmp = PatBlt(memdc, 0, 0, cx, cy, 0x00FA0089);
            SelectObject(memdc, hbsave as HGDIOBJ);
            if tmp == 0 { break 'cleanup; }
        }

        if (flags & DSS_DISABLED) != 0 {
            hbrtmp = GetSysColorBrush(COLOR_3DHILIGHT as i32);
        } else if (flags & DSS_DEFAULT) != 0 {
            hbrtmp = GetSysColorBrush(COLOR_3DSHADOW as i32);
        }

        if (flags & (DSS_DISABLED | DSS_DEFAULT)) != 0
        {
            if hbrtmp.is_null() { break 'cleanup; }
            hbsave = SelectObject(hdc, hbrtmp as HGDIOBJ) as HBRUSH;
            if hbsave.is_null() { break 'cleanup; }
            if BitBlt(hdc, x + 1, y + 1, cx, cy, memdc, 0, 0, 0x00B8074A) == 0 { break 'cleanup; }
            SelectObject(hdc, hbsave as HGDIOBJ);
        }

        if (flags & DSS_DISABLED) != 0
        {
            hbrtmp = GetSysColorBrush(COLOR_3DSHADOW as i32);
            hbr = hbrtmp;
            if hbrtmp.is_null() { break 'cleanup; }
        }
        else if hbr.is_null()
        {
            hbr = GetStockObject(BLACK_BRUSH as i32) as HBRUSH;
        }

        hbsave = SelectObject(hdc, hbr as HGDIOBJ) as HBRUSH;

        if BitBlt(hdc, x, y, cx, cy, memdc, 0, 0, 0x00B8074A) == 0 { break 'cleanup; }

        retval = 1;
    }

    SetTextColor(hdc, fg);
    SetBkColor(hdc, bg);

    if !hbsave.is_null() { SelectObject(hdc, hbsave as HGDIOBJ); }
    if !hbmsave.is_null() { SelectObject(memdc, hbmsave as HGDIOBJ); }
    if !hbm.is_null() { DeleteObject(hbm as HGDIOBJ); }
    if !memdc.is_null() { DeleteDC(memdc); }

    retval
}

#[no_mangle]
pub unsafe extern "system" fn RealDrawFrameControl(hDC: HDC, rc: LPRECT, uType: UINT, uState: UINT) -> BOOL {

    if GetMapMode(hDC) != MM_TEXT as i32 {
        return 0;
    }

    match uType {
        DFC_BUTTON => UITOOLS95_DrawFrameButton(hDC, rc, uState),
        DFC_CAPTION => UITOOLS95_DrawFrameCaption(hDC, rc, uState),
        DFC_MENU => {
            let ret: BOOL;
            let rgbOldText: COLORREF;
            let iOldBackMode: i32;

            if uState & (DFCS_MENUARROWUP | DFCS_MENUARROWDOWN) != 0 {
                if uState & DFCS_TRANSPARENT == 0 {
                    FillRect(hDC, rc, (COLOR_MENU + 1) as HBRUSH);
                }
            } else {
                FillRect(hDC, rc, GetStockObject(WHITE_BRUSH as i32) as HBRUSH);
            }

            rgbOldText = SetTextColor(hDC, RGB(0, 0, 0));
            iOldBackMode = SetBkMode(hDC, TRANSPARENT as i32);
            ret = UITOOLS95_DrawFrameMenu(hDC, rc, uState);
            SetBkMode(hDC, iOldBackMode);
            SetTextColor(hDC, rgbOldText);
            ret
        }
        DFC_SCROLL => UITOOLS95_DrawFrameScroll(hDC, rc, uState),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn DrawFrameControl(hDC: HDC, rc: LPRECT, uType: UINT, uState: UINT) -> BOOL {
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    let Hook: BOOL = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealDrawFrameControl(hDC, rc, uType, uState);
    }

    Ret = guah.DrawFrameControl.unwrap()(hDC, rc as LPRECTL, uType, uState);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn DrawEdge(hDC: HDC, rc: LPRECT, edge: UINT, flags: UINT) -> BOOL {
    if flags & BF_DIAGONAL != 0 {
        IntDrawDiagEdge(hDC, rc, edge, flags)
    } else {
        IntDrawRectEdge(hDC, rc, edge, flags, 1)
    }
}

#[no_mangle]
pub unsafe extern "system" fn GrayStringA(
    hDC: HDC,
    hBrush: HBRUSH,
    lpOutputFunc: GRAYSTRINGPROC,
    lpData: LPARAM,
    nCount: core::ffi::c_int,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
) -> i32 {
    IntGrayString(hDC, hBrush, lpOutputFunc, lpData, nCount, X, Y, nWidth, nHeight, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GrayStringW(
    hDC: HDC,
    hBrush: HBRUSH,
    lpOutputFunc: GRAYSTRINGPROC,
    lpData: LPARAM,
    nCount: core::ffi::c_int,
    X: core::ffi::c_int,
    Y: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
) -> i32 {
    IntGrayString(hDC, hBrush, lpOutputFunc, lpData, nCount, X, Y, nWidth, nHeight, 1)
}

#[no_mangle]
pub unsafe extern "system" fn InvertRect(hDC: HDC, lprc: *const RECT) -> i32 {
    PatBlt(
        hDC,
        (*lprc).left,
        (*lprc).top,
        (*lprc).right - (*lprc).left,
        (*lprc).bottom - (*lprc).top,
        DSTINVERT,
    )
}

#[no_mangle]
pub unsafe extern "system" fn FrameRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32 {
    let mut r: RECT = *lprc;

    if IsRectEmpty(core::ptr::addr_of_mut!(r)) != 0 {
        return 0;
    }
    let oldbrush = SelectObject(hDC, hbr as HGDIOBJ);
    if oldbrush.is_null() {
        return 0;
    }

    PatBlt(hDC, r.left, r.top, 1, r.bottom - r.top, PATCOPY as u32);
    PatBlt(hDC, r.right - 1, r.top, 1, r.bottom - r.top, PATCOPY as u32);
    PatBlt(hDC, r.left, r.top, r.right - r.left, 1, PATCOPY as u32);
    PatBlt(hDC, r.left, r.bottom - 1, r.right - r.left, 1, PATCOPY as u32);

    SelectObject(hDC, oldbrush);
    1
}

#[no_mangle]
pub unsafe extern "system" fn FlashWindow(hWnd: HWND, bInvert: i32) -> i32 {
    let mut FlashWndInfo: FLASHWINFO = core::mem::zeroed();

    FlashWndInfo.cbSize = core::mem::size_of::<FLASHWINFO>() as u32;
    FlashWndInfo.hwnd = hWnd;
    FlashWndInfo.dwFlags = if bInvert == 0 { 0 } else { FLASHW_TRAY | FLASHW_CAPTION };
    FlashWndInfo.uCount = 1;
    FlashWndInfo.dwTimeout = 0;

    NtUserFlashWindowEx(&mut FlashWndInfo)
}

#[no_mangle]
pub unsafe extern "system" fn FillRect(hDC: HDC, lprc: *const RECT, mut hbr: HBRUSH) -> i32 {
    let Ret: i32;
    let mut prevhbr: HBRUSH = core::ptr::null_mut();

    if !hbr.is_null() {
        if hbr <= (COLOR_MENUBAR + 1) as usize as HBRUSH {
            hbr = GetSysColorBrush(PtrToUlong(hbr as *mut core::ffi::c_void).wrapping_sub(1) as i32);
        }

        prevhbr = SelectObject(hDC, hbr as HGDIOBJ) as HBRUSH;
        if prevhbr.is_null() {
            return 0;
        }
    }

    Ret = PatBlt(
        hDC,
        (*lprc).left,
        (*lprc).top,
        (*lprc).right - (*lprc).left,
        (*lprc).bottom - (*lprc).top,
        PATCOPY,
    );

    if !prevhbr.is_null() {
        SelectObject(hDC, prevhbr as HGDIOBJ);
    }

    Ret as i32
}

#[no_mangle]
pub unsafe extern "system" fn DrawFocusRect(hdc: HDC, rect: *const RECT) -> i32 {

    let mut cx: u32 = 0;
    let mut cy: u32 = 0;

    NtUserSystemParametersInfo(
        SPI_GETFOCUSBORDERWIDTH,
        0,
        core::ptr::addr_of_mut!(cx) as *mut core::ffi::c_void,
        0,
    );
    NtUserSystemParametersInfo(
        SPI_GETFOCUSBORDERHEIGHT,
        0,
        core::ptr::addr_of_mut!(cy) as *mut core::ffi::c_void,
        0,
    );

    let brush = if !gpsi.is_null() {
        crate::wow64::psi::hbr_gray() as HGDIOBJ
    } else {
        core::ptr::null_mut()
    };

    let OldObj: HGDIOBJ = SelectObject(hdc, brush);

    let cx = cx as i32;
    let cy = cy as i32;

    /* top */
    PatBlt(
        hdc,
        (*rect).left,
        (*rect).top,
        (*rect).right - (*rect).left,
        cy,
        PATINVERT,
    );
    /* bottom */
    PatBlt(
        hdc,
        (*rect).left,
        (*rect).bottom - cy,
        (*rect).right - (*rect).left,
        cy,
        PATINVERT,
    );
    /* left */
    PatBlt(
        hdc,
        (*rect).left,
        (*rect).top + cy,
        cx,
        (*rect).bottom - (*rect).top - (2 * cy),
        PATINVERT,
    );
    /* right */
    PatBlt(
        hdc,
        (*rect).right - cx,
        (*rect).top + cy,
        cx,
        (*rect).bottom - (*rect).top - (2 * cy),
        PATINVERT,
    );

    SelectObject(hdc, OldObj);
    1
}

#[no_mangle]
pub unsafe extern "system" fn DrawStateA(hDC: HDC, hBrush: HBRUSH, lpOutputFunc: DRAWSTATEPROC, lData: LPARAM,
                                         wData: WPARAM, x: INT, y: INT, cx: INT, cy: INT, fuFlags: UINT) -> BOOL {
    IntDrawState(hDC, hBrush, lpOutputFunc, lData, wData, x, y, cx, cy, fuFlags, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DrawStateW(hDC: HDC, hBrush: HBRUSH, lpOutputFunc: DRAWSTATEPROC,
                                         lData: LPARAM, wData: WPARAM, x: INT, y: INT, cx: INT,
                                         cy: INT, fuFlags: UINT) -> BOOL {
    IntDrawState(hDC, hBrush, lpOutputFunc, lData, wData, x, y, cx, cy, fuFlags, 1 as BOOL)
}
