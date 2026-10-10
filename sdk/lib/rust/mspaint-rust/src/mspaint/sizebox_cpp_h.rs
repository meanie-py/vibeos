use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
}

// ---- file-scope preamble (translated) ----
pub static mut s_cursor_shapes: [LPCWSTR; 8] = [
    32642usize as LPCWSTR, // IDC_SIZENWSE
    32645usize as LPCWSTR, // IDC_SIZENS
    32643usize as LPCWSTR, // IDC_SIZENESW
    32644usize as LPCWSTR, // IDC_SIZEWE
    32644usize as LPCWSTR, // IDC_SIZEWE
    32643usize as LPCWSTR, // IDC_SIZENESW
    32645usize as LPCWSTR, // IDC_SIZENS
    32642usize as LPCWSTR, // IDC_SIZENWSE
];
#[no_mangle]
pub unsafe extern "C" fn setCursorOnSizeBox(hit: HITTEST) -> BOOL {
    if HIT_UPPER_LEFT <= hit && hit <= HIT_LOWER_RIGHT {
        SetCursor(LoadCursorW(core::ptr::null_mut(), s_cursor_shapes[(hit - HIT_UPPER_LEFT) as usize]));
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn getSizeBoxRect(prc: LPRECT, hit: HITTEST, prcBase: LPCRECT) -> BOOL {
    match hit {
        h if h == HIT_UPPER_LEFT => {
            (*prc).left = (*prcBase).left;
            (*prc).top = (*prcBase).top;
        }
        h if h == HIT_UPPER_CENTER => {
            (*prc).left = ((*prcBase).left + (*prcBase).right - (GRIP_SIZE as i32)) / 2;
            (*prc).top = (*prcBase).top;
        }
        h if h == HIT_UPPER_RIGHT => {
            (*prc).left = (*prcBase).right - (GRIP_SIZE as i32);
            (*prc).top = (*prcBase).top;
        }
        h if h == HIT_MIDDLE_LEFT => {
            (*prc).left = (*prcBase).left;
            (*prc).top = ((*prcBase).top + (*prcBase).bottom - (GRIP_SIZE as i32)) / 2;
        }
        h if h == HIT_MIDDLE_RIGHT => {
            (*prc).left = (*prcBase).right - (GRIP_SIZE as i32);
            (*prc).top = ((*prcBase).top + (*prcBase).bottom - (GRIP_SIZE as i32)) / 2;
        }
        h if h == HIT_LOWER_LEFT => {
            (*prc).left = (*prcBase).left;
            (*prc).top = (*prcBase).bottom - (GRIP_SIZE as i32);
        }
        h if h == HIT_LOWER_CENTER => {
            (*prc).left = ((*prcBase).left + (*prcBase).right - (GRIP_SIZE as i32)) / 2;
            (*prc).top = (*prcBase).bottom - (GRIP_SIZE as i32);
        }
        h if h == HIT_LOWER_RIGHT => {
            (*prc).left = (*prcBase).right - (GRIP_SIZE as i32);
            (*prc).top = (*prcBase).bottom - (GRIP_SIZE as i32);
        }
        h if h == HIT_INNER => {
            *prc = *prcBase;
            InflateRect(prc, (-(GRIP_SIZE as i32)) as i32, (-(GRIP_SIZE as i32)) as i32);
            return (TRUE) as BOOL;
        }
        _ => {
            SetRectEmpty(prc);
            return (FALSE) as BOOL;
        }
    }

    (*prc).right = (*prc).left + (GRIP_SIZE as i32);
    (*prc).bottom = (*prc).top + (GRIP_SIZE as i32);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn getSizeBoxHitTest(pt: POINT, prcBase: LPCRECT) -> HITTEST {
    let mut rc: CRect = CRect__make_0();

    if PtInRect(prcBase, pt) == 0 {
        return HIT_NONE;
    }

    rc.__base0 = *prcBase;
    rc.InflateRect__0((-(GRIP_SIZE as i32)) as i32, (-(GRIP_SIZE as i32)) as i32);
    if rc.PtInRect(pt) != 0 {
        return HIT_INNER;
    }

    let mut i: i32 = (HIT_UPPER_LEFT) as i32;
    while ((i) as u32) <= HIT_LOWER_RIGHT {
        let hit: HITTEST = i as HITTEST;
        getSizeBoxRect(core::ptr::addr_of_mut!(rc.__base0), hit, prcBase);
        if rc.PtInRect(pt) != 0 {
            return hit;
        }
        i += 1;
    }

    return HIT_BORDER;
}
#[no_mangle]
pub unsafe extern "C" fn drawSizeBoxes(hdc: HDC, prcBase: LPCRECT, bDrawFrame: BOOL, prcPaint: LPCRECT) {
    let mut rc: CRect = CRect__make_0();
    let mut rcIntersect: CRect = CRect__make_0();

    if !prcPaint.is_null() && IntersectRect(core::ptr::addr_of_mut!(rcIntersect.__base0), prcPaint, prcBase) == 0 {
        return;
    }

    if bDrawFrame != 0 {
        rc.__base0 = *prcBase;
        rc.InflateRect__0(((-(GRIP_SIZE as i32)) / 2) as i32, ((-(GRIP_SIZE as i32)) / 2) as i32);

        let mut logBrush: LOGBRUSH = LOGBRUSH { lbStyle: BS_HOLLOW, lbColor: 0, lbHatch: 0 };
        let rgbHighlight: COLORREF = GetSysColor((COLOR_HIGHLIGHT as i32));
        let oldPen: HGDIOBJ = SelectObject(hdc, (CreatePen((PS_DOT as i32), 1, rgbHighlight)).cast());
        let oldBrush: HGDIOBJ = SelectObject(hdc, (CreateBrushIndirect(core::ptr::addr_of!(logBrush))).cast());
        Rectangle(hdc, rc.__base0.left, rc.__base0.top, rc.__base0.right, rc.__base0.bottom);
        DeleteObject(SelectObject(hdc, oldBrush));
        DeleteObject(SelectObject(hdc, oldPen));
    }

    let mut i: INT = (HIT_UPPER_LEFT) as i32;
    while ((i) as u32) <= HIT_LOWER_RIGHT {
        getSizeBoxRect(core::ptr::addr_of_mut!(rc.__base0), i as HITTEST, prcBase);
        if prcPaint.is_null() || IntersectRect(core::ptr::addr_of_mut!(rcIntersect.__base0), core::ptr::addr_of!(rc.__base0), prcPaint) != 0 {
            FillRect(hdc, core::ptr::addr_of!(rc.__base0), (COLOR_HIGHLIGHT + 1) as usize as HBRUSH);
        }
        i += 1;
    }
}
