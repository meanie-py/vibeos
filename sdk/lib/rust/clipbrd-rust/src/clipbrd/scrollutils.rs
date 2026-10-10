use crate::clipbrd::cliputils::SendClipboardOwnerMessage;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn OnKeyScroll(hWnd: HWND, wParam: WPARAM, lParam: LPARAM, state: LPSCROLLSTATE) {
    // NOTE: Windows uses an offset of 16 pixels
    match wParam as u32 {
        VK_UP => {
            OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_LINEUP, 0)) as WPARAM, 5, state);
        }
        VK_DOWN => {
            OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_LINEDOWN, 0)) as WPARAM, 5, state);
        }
        VK_LEFT => {
            OnScroll(hWnd, (SB_HORZ) as INT, (MAKELONG!(SB_LINELEFT, 0)) as WPARAM, 5, state);
        }
        VK_RIGHT => {
            OnScroll(hWnd, (SB_HORZ) as INT, (MAKELONG!(SB_LINERIGHT, 0)) as WPARAM, 5, state);
        }
        VK_PRIOR => {
            OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_PAGEUP, 0)) as WPARAM, (*state).nPageY, state);
        }
        VK_NEXT => {
            OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_PAGEDOWN, 0)) as WPARAM, (*state).nPageY, state);
        }
        VK_HOME => {
            OnScroll(hWnd, (SB_HORZ) as INT, (MAKELONG!(SB_LEFT, 0)) as WPARAM, 0, state);
            if (GetKeyState(VK_CONTROL as ::core::ffi::c_int) as i32 & 0x8000) != 0 {
                OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_TOP, 0)) as WPARAM, 0, state);
            }
        }
        VK_END => {
            OnScroll(hWnd, (SB_HORZ) as INT, (MAKELONG!(SB_RIGHT, 0)) as WPARAM, 0, state);
            if (GetKeyState(VK_CONTROL as ::core::ffi::c_int) as i32 & 0x8000) != 0 {
                OnScroll(hWnd, (SB_VERT) as INT, (MAKELONG!(SB_BOTTOM, 0)) as WPARAM, 0, state);
            }
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "C" fn OnMouseScroll(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, state: LPSCROLLSTATE) {
    let mut nBar: INT;
    let mut nPage: INT;
    let mut iDelta: INT;
    let mut uLinesToScroll: UINT = (*state).uLinesToScroll;
    let mut zDelta: INT = (GET_WHEEL_DELTA_WPARAM!(wParam)) as i32;
    let mut sbCode: WORD;

    assert!(uMsg == (WM_MOUSEWHEEL as UINT) || uMsg == (WM_MOUSEHWHEEL as UINT));

    if uMsg == (WM_MOUSEWHEEL as UINT) {
        nBar = SB_VERT as INT;
        nPage = (*state).nPageY;

        /* Accumulate wheel rotation ticks */
        zDelta += (*state).iWheelCarryoverY;
        (*state).iWheelCarryoverY = zDelta % (WHEEL_DELTA as INT);
    } else {
        nBar = SB_HORZ as INT;
        nPage = (*state).nPageX;

        /* Accumulate wheel rotation ticks */
        zDelta += (*state).iWheelCarryoverX;
        (*state).iWheelCarryoverX = zDelta % (WHEEL_DELTA as INT);
    }

    /*
     * If the user specified scrolling by pages, do so.
     * Due to a bug on Windows where, if the window height is
     * less than the scroll lines delta default value (== 3),
     * several lines would be skipped when scrolling if we
     * used the WHEEL_PAGESCROLL value. Instead of this, use
     * the number of lines per page as the limiting value.
     * See https://www.strchr.com/corrections_to_raymond_chen_s_wheel_scrolling_code
     * for more details.
     */
    if uLinesToScroll > (nPage as UINT) {
        uLinesToScroll = nPage as UINT;
    }
    /* If the user specified no wheel scrolling, don't do anything */
    else if uLinesToScroll == (0 as UINT) {
        return;
    }

    /* Compute the scroll direction and the absolute delta value */
    if zDelta > (0 as INT) {
        sbCode = SB_LINEUP as WORD;
    } else {
        sbCode = SB_LINEDOWN as WORD;
        zDelta = -zDelta;
    }

    /* Compute how many lines we should scroll (in absolute value) */
    iDelta = ((uLinesToScroll as INT) * zDelta) / (WHEEL_DELTA as INT);

    OnScroll(hWnd, nBar, MAKELONG!(sbCode, 0 as WORD) as WPARAM as ::core::ffi::c_long as LONG as WPARAM, iDelta, state);
}
#[no_mangle]
pub unsafe extern "C" fn OnScroll(mut hWnd: HWND, mut nBar: INT, mut wParam: WPARAM, mut iDelta: INT, mut state: LPSCROLLSTATE) {
    let mut si: SCROLLINFO = core::mem::zeroed();
    let mut pCurrent: PINT = core::ptr::null_mut();
    let mut Maximum: INT = 0;
    let mut NewPos: INT = 0;
    let mut OldX: INT = 0;
    let mut OldY: INT = 0;

    assert!(((nBar) as u32 == SB_HORZ || (nBar) as u32 == SB_VERT));

    if Globals.uDisplayFormat == CF_OWNERDISPLAY {
        if (nBar) as u32 == SB_HORZ {
            SendClipboardOwnerMessage(
                (TRUE) as BOOL,
                WM_HSCROLLCLIPBOARD,
                hWnd as WPARAM,
                wParam as LPARAM,
            );
        } else {
            SendClipboardOwnerMessage(
                (TRUE) as BOOL,
                WM_VSCROLLCLIPBOARD,
                hWnd as WPARAM,
                wParam as LPARAM,
            );
        }
        return;
    }

    if (nBar) as u32 == SB_HORZ {
        pCurrent = core::ptr::addr_of_mut!((*state).CurrentX) as PINT;
        Maximum = (*state).MaxX;
    } else {
        pCurrent = core::ptr::addr_of_mut!((*state).CurrentY) as PINT;
        Maximum = (*state).MaxY;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void, core::mem::size_of::<SCROLLINFO>());
    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = SIF_RANGE | SIF_PAGE | SIF_POS | SIF_TRACKPOS;
    GetScrollInfo(hWnd, nBar, core::ptr::addr_of_mut!(si));

    match (LOWORD!(wParam as DWORD) as WORD) as u32 {
        SB_THUMBPOSITION | SB_THUMBTRACK => {
            NewPos = si.nTrackPos;
        }
        SB_LINEUP => {
            // SB_LINELEFT:
            NewPos = si.nPos - iDelta;
        }
        SB_LINEDOWN => {
            // SB_LINERIGHT:
            NewPos = si.nPos + iDelta;
        }
        SB_PAGEUP => {
            // SB_PAGELEFT:
            NewPos = si.nPos - (si.nPage as INT);
        }
        SB_PAGEDOWN => {
            // SB_PAGERIGHT:
            NewPos = si.nPos + (si.nPage as INT);
        }
        SB_TOP => {
            // SB_LEFT:
            NewPos = si.nMin;
        }
        SB_BOTTOM => {
            // SB_RIGHT:
            NewPos = si.nMax;
        }
        _ => {
            NewPos = si.nPos;
        }
    }

    NewPos = min(max(NewPos, 0), Maximum);

    if si.nPos == NewPos {
        return;
    }

    OldX = (*state).CurrentX;
    OldY = (*state).CurrentY;
    *pCurrent = NewPos;

    ScrollWindowEx(
        hWnd,
        OldX - (*state).CurrentX,
        OldY - (*state).CurrentY,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SW_ERASE | SW_INVALIDATE,
    );
    UpdateWindow(hWnd);

    si.fMask = SIF_POS;
    si.nPos = NewPos;
    SetScrollInfo(hWnd, nBar, core::ptr::addr_of_mut!(si), (TRUE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateLinesToScroll(state: LPSCROLLSTATE) {
    let mut uLinesToScroll: UINT = 0;
    if SystemParametersInfo(
        SPI_GETWHEELSCROLLLINES,
        0,
        core::ptr::addr_of_mut!(uLinesToScroll) as PVOID,
        0,
    ) == 0
    {
        /* Default value on Windows */
        (*state).uLinesToScroll = 3;
    } else {
        (*state).uLinesToScroll = uLinesToScroll;
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateWindowScrollState(hWnd: HWND, nMaxWidth: INT, nMaxHeight: INT, lpState: LPSCROLLSTATE) {
    let mut rc: RECT = core::mem::zeroed();
    let mut si: SCROLLINFO = core::mem::zeroed();

    if GetClientRect(hWnd, core::ptr::addr_of_mut!(rc)) == 0 {
        SetRectEmpty(core::ptr::addr_of_mut!(rc));
    }

    ZeroMemory!(core::ptr::addr_of_mut!(si) as *mut core::ffi::c_void, core::mem::size_of::<SCROLLINFO>() as usize);
    si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    si.fMask = (SIF_RANGE | SIF_PAGE | SIF_POS | SIF_DISABLENOSCROLL) as UINT;

    (*lpState).nMaxWidth = nMaxWidth;
    (*lpState).MaxX = max(nMaxWidth - rc.right, 0);
    (*lpState).CurrentX = min((*lpState).CurrentX, (*lpState).MaxX);
    (*lpState).nPageX = rc.right;
    si.nMin = 0;
    si.nMax = nMaxWidth;
    si.nPage = (*lpState).nPageX as UINT;
    si.nPos = (*lpState).CurrentX;
    SetScrollInfo(hWnd, (SB_HORZ) as i32, core::ptr::addr_of_mut!(si), (TRUE) as BOOL);

    (*lpState).nMaxHeight = nMaxHeight;
    (*lpState).MaxY = max(nMaxHeight - rc.bottom, 0);
    (*lpState).CurrentY = min((*lpState).CurrentY, (*lpState).MaxY);
    (*lpState).nPageY = rc.bottom;
    si.nMin = 0;
    si.nMax = nMaxHeight;
    si.nPage = (*lpState).nPageY as UINT;
    si.nPos = (*lpState).CurrentY;
    SetScrollInfo(hWnd, (SB_VERT) as i32, core::ptr::addr_of_mut!(si), (TRUE) as BOOL);
}
