use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::comctl32undoc::Str_SetPtrWtoA;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_pager);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_PAGESCROLLERW: [::core::ffi::c_ushort; 9] = unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_ushort; 9]>(*b"S\0y\0s\0P\0a\0g\0e\0r\0\0\0")
};


#[repr(C)]
pub struct PAGER_INFO {
    pub hwndSelf: HWND,
    pub hwndChild: HWND,
    pub hwndNotify: HWND,
    pub bUnicode: BOOL,
    pub dwStyle: DWORD,
    pub clrBk: COLORREF,
    pub nBorder: INT,
    pub nButtonSize: INT,
    pub nPos: INT,
    pub nWidth: INT,
    pub nHeight: INT,
    pub bForward: BOOL,
    pub bCapture: BOOL,
    pub TLbtnState: INT,
    pub BRbtnState: INT,
    pub direction: INT,
    pub pwszBuffer: *mut WCHAR,
    pub nBufferSize: INT,
}

pub type PPAGER_INFO = *mut PAGER_INFO;

pub const TIMERID1: u32 = 1;

pub const TIMERID2: u32 = 2;

pub const INITIAL_DELAY: u32 = 500;

pub const REPEAT_DELAY: u32 = 50;

/* Convert Unicode text to ANSI for parent before sending. If not set, do nothing */
pub const CONVERT_SEND: i32 = 0x01;
/* Convert ANSI text from parent back to Unicode for children */
pub const CONVERT_RECEIVE: i32 = 0x02;
/* Send empty text to parent if text is NULL. Original text pointer still remains NULL */
pub const SEND_EMPTY_IF_NULL: i32 = 0x04;
/* Set text to null after parent received the notification if the required mask is not set before sending notification */
pub const SET_NULL_IF_NO_MASK: i32 = 0x08;
/* Zero out the text buffer before sending it to parent */
pub const ZERO_SEND: i32 = 0x10;

unsafe extern "C" fn PAGER_GetButtonRects(
    infoPtr: *const PAGER_INFO,
    prcTopLeft: *mut RECT,
    prcBottomRight: *mut RECT,
    bClientCoords: BOOL,
) {
    let mut rcWindow: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcWindow));

    if bClientCoords != 0 {
        MapWindowPoints(
            core::ptr::null_mut(),
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!(rcWindow) as LPPOINT,
            2,
        );
    } else {
        OffsetRect(
            core::ptr::addr_of_mut!(rcWindow),
            -(rcWindow.left as core::ffi::c_int),
            -(rcWindow.top as core::ffi::c_int),
        );
    }

    *prcBottomRight = rcWindow;
    *prcTopLeft = *prcBottomRight;
    if ((*infoPtr).dwStyle & PGS_HORZ as DWORD) != 0 {
        (*prcTopLeft).right = ((*prcTopLeft).left as core::ffi::c_int + (*infoPtr).nButtonSize) as LONG;
        (*prcBottomRight).left = ((*prcBottomRight).right as core::ffi::c_int - (*infoPtr).nButtonSize) as LONG;
    } else {
        (*prcTopLeft).bottom = ((*prcTopLeft).top as core::ffi::c_int + (*infoPtr).nButtonSize) as LONG;
        (*prcBottomRight).top = ((*prcBottomRight).bottom as core::ffi::c_int - (*infoPtr).nButtonSize) as LONG;
    }
}

unsafe extern "C" fn PAGER_DrawButton(
    mut hdc: HDC,
    mut clrBk: COLORREF,
    mut rc: RECT,
    mut horz: BOOL,
    mut topLeft: BOOL,
    mut btnState: INT,
) {
    let mut flags: UINT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_DrawButton\0" as *const u8 as *const ::core::ffi::c_char,
            141 as ::core::ffi::c_int,
            b"rc = %s, btnState = %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(rc)),
            btnState,
        );
    }
    if btnState == (PGF_INVISIBLE as i32) {
        return;
    }
    if rc.right - rc.left <= 0
        || rc.bottom - rc.top <= 0
    {
        return;
    }
    if horz != 0 {
        flags = (if topLeft != 0 {
            DFCS_SCROLLLEFT
        } else {
            DFCS_SCROLLRIGHT
        }) as UINT;
    } else {
        flags = (if topLeft != 0 {
            DFCS_SCROLLUP
        } else {
            DFCS_SCROLLDOWN
        }) as UINT;
    }
    match (btnState) as u32 {
        PGF_HOT => {}
        PGF_NORMAL => {
            flags |= DFCS_FLAT as UINT;
        }
        PGF_DEPRESSED => {
            flags |= DFCS_PUSHED as UINT;
        }
        PGF_GRAYED => {
            flags |= (DFCS_INACTIVE | DFCS_FLAT) as UINT;
        }
        _ => {}
    }
    DrawFrameControl(
        hdc,
        core::ptr::addr_of_mut!(rc),
        DFC_SCROLL as UINT,
        flags,
    );
}

unsafe extern "C" fn PAGER_ForwardMouse(infoPtr: *mut PAGER_INFO, bFwd: BOOL) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_ForwardMouse\0" as *const u8 as *const ::core::ffi::c_char,
            176 as ::core::ffi::c_int,
            b"[%p]\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
        );
    }
    (*infoPtr).bForward = bFwd;
    return 0 as LRESULT;
}

unsafe extern "C" fn PAGER_GetButtonState(infoPtr: *const PAGER_INFO, btn: INT) -> LRESULT {
    let mut btnState: LRESULT = PGF_INVISIBLE as LRESULT;

    if btn == PGB_TOPORLEFT as INT {
        btnState = (*infoPtr).TLbtnState as LRESULT;
    } else if btn == PGB_BOTTOMORRIGHT as INT {
        btnState = (*infoPtr).BRbtnState as LRESULT;
    }

    return btnState;
}

unsafe extern "C" fn PAGER_GetPos(infoPtr: *const PAGER_INFO) -> INT {
    // TRACE("[%p] returns %d\n", infoPtr->hwndSelf, infoPtr->nPos);
    (*infoPtr).nPos
}

unsafe extern "C" fn PAGER_GetButtonSize(infoPtr: *const PAGER_INFO) -> INT {
    // TRACE("[%p] returns %d\n", infoPtr->hwndSelf, infoPtr->nButtonSize);
    (*infoPtr).nButtonSize
}

unsafe extern "C" fn PAGER_GetBorder(infoPtr: *const PAGER_INFO) -> INT {
    // TRACE("[%p] returns %d\n", infoPtr->hwndSelf, infoPtr->nBorder);
    (*infoPtr).nBorder
}

unsafe extern "C" fn PAGER_GetBkColor(infoPtr: *const PAGER_INFO) -> COLORREF {
    // TRACE("[%p] returns %06x\n", infoPtr->hwndSelf, infoPtr->clrBk);
    (*infoPtr).clrBk
}

unsafe extern "C" fn PAGER_CalcSize(mut infoPtr: *mut PAGER_INFO) {
    let mut nmpgcs: NMPGCALCSIZE = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(nmpgcs) as *mut ::core::ffi::c_void,
        core::mem::size_of::<NMPGCALCSIZE>(),
    );
    nmpgcs.hdr.hwndFrom = (*infoPtr).hwndSelf;
    nmpgcs.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    nmpgcs.hdr.code = PGN_CALCSIZE as UINT;
    nmpgcs.dwFlag = (if ((*infoPtr).dwStyle & PGS_HORZ) != 0 {
        PGF_CALCWIDTH
    } else {
        PGF_CALCHEIGHT
    }) as DWORD;
    nmpgcs.iWidth = (*infoPtr).nWidth;
    nmpgcs.iHeight = (*infoPtr).nHeight;
    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmpgcs.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmpgcs) as LPARAM,
    );

    if ((*infoPtr).dwStyle & PGS_HORZ) != 0 {
        (*infoPtr).nWidth = nmpgcs.iWidth;
    } else {
        (*infoPtr).nHeight = nmpgcs.iHeight;
    }
}

unsafe extern "C" fn PAGER_PositionChildWnd(infoPtr: *mut PAGER_INFO) {
    if !(*infoPtr).hwndChild.is_null() {
        let mut rcClient: RECT = core::mem::zeroed();
        let mut nPos: i32 = (*infoPtr).nPos;

        /* compensate for a grayed btn, which will soon become invisible */
        if (*infoPtr).TLbtnState == (PGF_GRAYED as i32) {
            nPos += (*infoPtr).nButtonSize;
        }

        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));

        if ((*infoPtr).dwStyle & PGS_HORZ) != 0 {
            let wndSize: i32 = max(0, rcClient.right - rcClient.left);
            if (*infoPtr).nWidth < wndSize {
                (*infoPtr).nWidth = wndSize;
            }

            SetWindowPos(
                (*infoPtr).hwndChild,
                HWND_TOP,
                -nPos,
                0,
                (*infoPtr).nWidth,
                (*infoPtr).nHeight,
                0,
            );
        } else {
            let wndSize: i32 = max(0, rcClient.bottom - rcClient.top);
            if (*infoPtr).nHeight < wndSize {
                (*infoPtr).nHeight = wndSize;
            }

            SetWindowPos(
                (*infoPtr).hwndChild,
                HWND_TOP,
                0,
                -nPos,
                (*infoPtr).nWidth,
                (*infoPtr).nHeight,
                0,
            );
        }

        InvalidateRect(
            (*infoPtr).hwndChild,
            core::ptr::null_mut(),
            TRUE,
        );
    }
}

unsafe extern "C" fn PAGER_GetScrollRange(
    mut infoPtr: *mut PAGER_INFO,
    mut calc_size: BOOL,
) -> INT {
    let mut scrollRange: INT = 0 as INT;
    if !(*infoPtr).hwndChild.is_null() {
        let mut wndSize: INT = 0;
        let mut childSize: INT = 0;
        let mut wndRect: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect((*infoPtr).hwndSelf, &raw mut wndRect);
        if calc_size != 0 {
            PAGER_CalcSize(infoPtr);
        }
        if (*infoPtr).dwStyle & (PGS_HORZ as DWORD) != 0 {
            wndSize = (wndRect.right - wndRect.left) as INT;
            childSize = (*infoPtr).nWidth;
        } else {
            wndSize = (wndRect.bottom - wndRect.top) as INT;
            childSize = (*infoPtr).nHeight;
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/pager.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PAGER_GetScrollRange\0" as *const u8 as *const ::core::ffi::c_char,
                316 as ::core::ffi::c_int,
                b"childSize = %d,  wndSize = %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                childSize,
                wndSize,
            );
        }
        if childSize > wndSize {
            scrollRange = childSize - wndSize + (*infoPtr).nButtonSize;
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_GetScrollRange\0" as *const u8 as *const ::core::ffi::c_char,
            321 as ::core::ffi::c_int,
            b"[%p] returns %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            scrollRange,
        );
    }
    return scrollRange;
}

unsafe extern "C" fn PAGER_UpdateBtns(infoPtr: *mut PAGER_INFO, scrollRange: INT, hideGrayBtns: BOOL) {
    let mut resizeClient: BOOL;
    let mut repaintBtns: BOOL;
    let oldTLbtnState: INT = (*infoPtr).TLbtnState;
    let oldBRbtnState: INT = (*infoPtr).BRbtnState;
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut rcTopLeft: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBottomRight: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    /* get button rects */
    PAGER_GetButtonRects(infoPtr, core::ptr::addr_of_mut!(rcTopLeft), core::ptr::addr_of_mut!(rcBottomRight), TRUE);

    GetCursorPos(core::ptr::addr_of_mut!(pt));
    ScreenToClient((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));

    /* update states based on scroll position */
    if (*infoPtr).nPos > 0 {
        if (*infoPtr).TLbtnState == (PGF_INVISIBLE as i32) || (*infoPtr).TLbtnState == (PGF_GRAYED as i32) {
            (*infoPtr).TLbtnState = (PGF_NORMAL as i32);
        }
    } else if hideGrayBtns == 0 && PtInRect(core::ptr::addr_of_mut!(rcTopLeft) as LPCRECT, pt) != 0 {
        (*infoPtr).TLbtnState = (PGF_GRAYED as i32);
    } else {
        (*infoPtr).TLbtnState = (PGF_INVISIBLE as i32);
    }

    if scrollRange <= 0 {
        (*infoPtr).TLbtnState = (PGF_INVISIBLE as i32);
        (*infoPtr).BRbtnState = (PGF_INVISIBLE as i32);
    } else if (*infoPtr).nPos < scrollRange {
        if (*infoPtr).BRbtnState == (PGF_INVISIBLE as i32) || (*infoPtr).BRbtnState == (PGF_GRAYED as i32) {
            (*infoPtr).BRbtnState = (PGF_NORMAL as i32);
        }
    } else if hideGrayBtns == 0 && PtInRect(core::ptr::addr_of_mut!(rcBottomRight) as LPCRECT, pt) != 0 {
        (*infoPtr).BRbtnState = (PGF_GRAYED as i32);
    } else {
        (*infoPtr).BRbtnState = (PGF_INVISIBLE as i32);
    }

    /* only need to resize when entering or leaving PGF_INVISIBLE state */
    resizeClient = ((((oldTLbtnState == (PGF_INVISIBLE as i32)) as INT) != (((*infoPtr).TLbtnState == (PGF_INVISIBLE as i32)) as INT))
        || (((oldBRbtnState == (PGF_INVISIBLE as i32)) as INT) != (((*infoPtr).BRbtnState == (PGF_INVISIBLE as i32)) as INT))) as INT;
    /* initiate NCCalcSize to resize client wnd if necessary */
    if resizeClient != 0 {
        SetWindowPos(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE
                | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }

    /* repaint when changing any state */
    repaintBtns = ((oldTLbtnState != (*infoPtr).TLbtnState) || (oldBRbtnState != (*infoPtr).BRbtnState)) as INT;
    if repaintBtns != 0 {
        SendMessageW((*infoPtr).hwndSelf, WM_NCPAINT, 0, 0);
    }
}

unsafe extern "C" fn PAGER_SetPos(infoPtr: *mut PAGER_INFO, newPos: INT, fromBtnPress: BOOL, calc_size: BOOL) -> LRESULT {
    let scrollRange: INT = PAGER_GetScrollRange(infoPtr, calc_size);
    let oldPos: INT = (*infoPtr).nPos;

    if (scrollRange <= 0) || (newPos < 0) {
        (*infoPtr).nPos = 0;
    } else if newPos > scrollRange {
        (*infoPtr).nPos = scrollRange;
    } else {
        (*infoPtr).nPos = newPos;
    }

    if (*infoPtr).nPos != oldPos {
        /* gray and restore btns, and if from WM_SETPOS, hide the gray btns */
        PAGER_UpdateBtns(infoPtr, scrollRange, (fromBtnPress == 0) as BOOL);
        PAGER_PositionChildWnd(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn PAGER_RecalcSize(infoPtr: *mut PAGER_INFO) -> LRESULT {
    if !(*infoPtr).hwndChild.is_null() {
        let scrollRange: INT = PAGER_GetScrollRange(infoPtr, TRUE);

        if scrollRange <= 0 {
            (*infoPtr).nPos = -(1) as INT;
            PAGER_SetPos(infoPtr, 0 as INT, FALSE, TRUE);
        } else {
            PAGER_PositionChildWnd(infoPtr);
        }
    }

    return 1 as LRESULT;
}

unsafe extern "C" fn PAGER_SetBkColor(
    mut infoPtr: *mut PAGER_INFO,
    mut clrBk: COLORREF,
) -> COLORREF {
    let mut clrTemp: COLORREF = (*infoPtr).clrBk;
    (*infoPtr).clrBk = clrBk;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_SetBkColor\0" as *const u8 as *const ::core::ffi::c_char,
            449 as ::core::ffi::c_int,
            b"[%p] %06x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            (*infoPtr).clrBk,
        );
    }
    /* the native control seems to do things this way */
    SetWindowPos(
        (*infoPtr).hwndSelf,
        core::ptr::null_mut(),
        0,
        0,
        0,
        0,
        SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE
            | SWP_NOZORDER | SWP_NOACTIVATE,
    );

    RedrawWindow(
        (*infoPtr).hwndSelf,
        core::ptr::null(),
        core::ptr::null_mut(),
        RDW_ERASE | RDW_INVALIDATE,
    );

    return clrTemp;
}

unsafe extern "C" fn PAGER_SetBorder(infoPtr: *mut PAGER_INFO, iBorder: INT) -> INT {
    let mut nTemp: INT = (*infoPtr).nBorder;

    (*infoPtr).nBorder = iBorder;

    PAGER_RecalcSize(infoPtr);

    return nTemp;
}

unsafe extern "C" fn PAGER_SetButtonSize(infoPtr: *mut PAGER_INFO, iButtonSize: INT) -> INT {
    let mut nTemp: INT = (*infoPtr).nButtonSize;

    (*infoPtr).nButtonSize = iButtonSize;

    PAGER_RecalcSize(infoPtr);

    return nTemp;
}

unsafe extern "C" fn PAGER_SetChild(infoPtr: *mut PAGER_INFO, hwndChild: HWND) -> LRESULT {
    (*infoPtr).hwndChild = if IsWindow(hwndChild) != 0 {
        hwndChild
    } else {
        core::ptr::null_mut()
    };

    if !(*infoPtr).hwndChild.is_null() {

        SetWindowPos((*infoPtr).hwndSelf, core::ptr::null_mut(), 0, 0, 0, 0,
            (SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE) as UINT);

        (*infoPtr).nPos = -(1) as INT;
        PAGER_SetPos(infoPtr, 0, FALSE, FALSE);
    }

    return 0;
}

unsafe extern "C" fn PAGER_Scroll(infoPtr: *mut PAGER_INFO, dir: INT) {
    let mut nmpgScroll: NMPGSCROLL = core::mem::zeroed();
    let mut rcWnd: RECT = core::mem::zeroed();

    if !(*infoPtr).hwndChild.is_null() {
        ZeroMemory!(
            core::ptr::addr_of_mut!(nmpgScroll) as *mut core::ffi::c_void,
            core::mem::size_of::<NMPGSCROLL>() as usize,
        );
        nmpgScroll.hdr.hwndFrom = (*infoPtr).hwndSelf;
        nmpgScroll.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID as INT) as UINT_PTR;
        nmpgScroll.hdr.code = PGN_SCROLL as UINT;

        GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcWnd));
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(nmpgScroll.rcParent));
        nmpgScroll.iYpos = 0 as INT;
        nmpgScroll.iXpos = nmpgScroll.iYpos;
        nmpgScroll.iDir = dir;

        if ((*infoPtr).dwStyle & (PGS_HORZ as DWORD)) != 0 {
            nmpgScroll.iScroll = (rcWnd.right - rcWnd.left) as INT;
            nmpgScroll.iXpos = (*infoPtr).nPos;
        } else {
            nmpgScroll.iScroll = (rcWnd.bottom - rcWnd.top) as INT;
            nmpgScroll.iYpos = (*infoPtr).nPos;
        }
        nmpgScroll.iScroll -= (2 as INT) * (*infoPtr).nButtonSize;

        SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            nmpgScroll.hdr.idFrom as WPARAM,
            core::ptr::addr_of_mut!(nmpgScroll) as LPARAM,
        );

        if nmpgScroll.iScroll > (0 as INT) {
            (*infoPtr).direction = dir;

            if dir == (PGF_SCROLLLEFT as INT) || dir == (PGF_SCROLLUP as INT) {
                PAGER_SetPos(
                    infoPtr,
                    (*infoPtr).nPos - nmpgScroll.iScroll,
                    TRUE as BOOL,
                    TRUE as BOOL,
                );
            } else {
                PAGER_SetPos(
                    infoPtr,
                    (*infoPtr).nPos + nmpgScroll.iScroll,
                    TRUE as BOOL,
                    TRUE as BOOL,
                );
            }
        } else {
            (*infoPtr).direction = -(1 as INT);
        }
    }
}

unsafe extern "C" fn PAGER_FmtLines(infoPtr: *const PAGER_INFO) -> LRESULT {
    /* initiate NCCalcSize to resize client wnd and get size */
    SetWindowPos((*infoPtr).hwndSelf, core::ptr::null_mut(), 0, 0, 0, 0,
        SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE |
        SWP_NOZORDER | SWP_NOACTIVATE);

    SetWindowPos((*infoPtr).hwndChild, core::ptr::null_mut(),
        0, 0, (*infoPtr).nWidth, (*infoPtr).nHeight,
        0);

    return DefWindowProcW((*infoPtr).hwndSelf, EM_FMTLINES, 0, 0);
}

unsafe extern "C" fn PAGER_Create(mut hwnd: HWND, mut lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut PAGER_INFO = ::core::ptr::null_mut::<PAGER_INFO>();
    let mut ret: INT = 0;
    infoPtr = heap_alloc_zero(::core::mem::size_of::<PAGER_INFO>() as SIZE_T) as *mut PAGER_INFO;
    if infoPtr.is_null() {
        return -(1 as ::core::ffi::c_int) as LRESULT;
    }
    SetWindowLongPtrW(
        hwnd,
        0 as ::core::ffi::c_int,
        infoPtr as DWORD_PTR as LONG_PTR,
    );
    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).hwndChild = ::core::ptr::null_mut::<HWND__>();
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;
    (*infoPtr).clrBk = GetSysColor(COLOR_BTNFACE as ::core::ffi::c_int) as COLORREF;
    (*infoPtr).nBorder = 0 as INT;
    (*infoPtr).nButtonSize = 12 as INT;
    (*infoPtr).nPos = 0 as INT;
    (*infoPtr).nWidth = 0 as INT;
    (*infoPtr).nHeight = 0 as INT;
    (*infoPtr).bForward = FALSE as BOOL;
    (*infoPtr).bCapture = FALSE as BOOL;
    (*infoPtr).TLbtnState = PGF_INVISIBLE as INT;
    (*infoPtr).BRbtnState = PGF_INVISIBLE as INT;
    (*infoPtr).direction = -(1 as ::core::ffi::c_int) as INT;
    if ((*infoPtr).dwStyle & (PGS_DRAGNDROP as DWORD)) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/pager.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PAGER_Create\0" as *const u8 as *const ::core::ffi::c_char,
                601 as ::core::ffi::c_int,
                b"[%p] Drag and Drop style is not implemented yet.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*infoPtr).hwndSelf,
            );
        }
    }
    ret = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFYFORMAT as UINT,
        (*infoPtr).hwndSelf as WPARAM,
        NF_QUERY as LPARAM,
    ) as INT;
    (*infoPtr).bUnicode = (ret == (NFR_UNICODE as INT)) as ::core::ffi::c_int as BOOL;
    return 0 as LRESULT;
}

unsafe extern "C" fn PAGER_Destroy(mut infoPtr: *mut PAGER_INFO) -> LRESULT {
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    heap_free((*infoPtr).pwszBuffer as *mut ::core::ffi::c_void);
    heap_free(infoPtr as *mut ::core::ffi::c_void);
    return 0;
}

unsafe extern "C" fn PAGER_NCCalcSize(
    mut infoPtr: *mut PAGER_INFO,
    mut wParam: WPARAM,
    mut lpRect: LPRECT,
) -> LRESULT {
    let mut rcChild: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcWindow: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    DefWindowProcW(
        (*infoPtr).hwndSelf,
        WM_NCCALCSIZE,
        wParam,
        lpRect as LPARAM,
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_NCCalcSize\0" as *const u8 as *const ::core::ffi::c_char,
            633 as ::core::ffi::c_int,
            b"orig rect=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(lpRect as *const RECT),
        );
    }
    GetWindowRect((*infoPtr).hwndChild, &raw mut rcChild);
    MapWindowPoints(
        core::ptr::null_mut(),
        (*infoPtr).hwndSelf,
        &raw mut rcChild as LPPOINT,
        2,
    );
    GetWindowRect((*infoPtr).hwndSelf, &raw mut rcWindow);
    (*infoPtr).nWidth = ((*lpRect).right - (*lpRect).left) as INT;
    (*infoPtr).nHeight = ((*lpRect).bottom - (*lpRect).top) as INT;
    PAGER_CalcSize(infoPtr);
    if (*infoPtr).dwStyle & PGS_HORZ != 0 {
        if (*infoPtr).TLbtnState != 0
            && (*lpRect).left as INT + (*infoPtr).nButtonSize < (*lpRect).right
        {
            (*lpRect).left += (*infoPtr).nButtonSize as ::core::ffi::c_int;
        }
        if (*infoPtr).BRbtnState != 0
            && (*lpRect).right as INT - (*infoPtr).nButtonSize > (*lpRect).left
        {
            (*lpRect).right -= (*infoPtr).nButtonSize as ::core::ffi::c_int;
        }
    } else {
        if (*infoPtr).TLbtnState != 0
            && (*lpRect).top as INT + (*infoPtr).nButtonSize < (*lpRect).bottom
        {
            (*lpRect).top += (*infoPtr).nButtonSize as ::core::ffi::c_int;
        }
        if (*infoPtr).BRbtnState != 0
            && (*lpRect).bottom as INT - (*infoPtr).nButtonSize > (*lpRect).top
        {
            (*lpRect).bottom -= (*infoPtr).nButtonSize as ::core::ffi::c_int;
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_NCCalcSize\0" as *const u8 as *const ::core::ffi::c_char,
            658 as ::core::ffi::c_int,
            b"nPos=%d, nHeight=%d, window=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).nPos,
            (*infoPtr).nHeight,
            wine_dbgstr_rect(&raw mut rcWindow),
        );
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_NCCalcSize\0" as *const u8 as *const ::core::ffi::c_char,
            660 as ::core::ffi::c_int,
            b"[%p] client rect set to %s BtnState[%d,%d]\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            wine_dbgstr_rect(lpRect as *const RECT),
            (*infoPtr).TLbtnState,
            (*infoPtr).BRbtnState,
        );
    }
    return 0;
}

unsafe extern "C" fn PAGER_NCPaint(infoPtr: *const PAGER_INFO, hRgn: HRGN) -> LRESULT {
    let mut rcBottomRight: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcTopLeft: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut hdc: HDC;

    if (*infoPtr).dwStyle & WS_MINIMIZE != 0 {
        return 0;
    }

    DefWindowProcW ((*infoPtr).hwndSelf, WM_NCPAINT, hRgn as WPARAM, 0);

    hdc = GetDCEx ((*infoPtr).hwndSelf, core::ptr::null_mut(), DCX_USESTYLE | DCX_WINDOW);
    if hdc.is_null() {
        return 0;
    }

    PAGER_GetButtonRects(infoPtr, &raw mut rcTopLeft, &raw mut rcBottomRight, FALSE);

    PAGER_DrawButton(hdc, (*infoPtr).clrBk, rcTopLeft,
                     ((*infoPtr).dwStyle & PGS_HORZ) as BOOL, TRUE, (*infoPtr).TLbtnState);
    PAGER_DrawButton(hdc, (*infoPtr).clrBk, rcBottomRight,
                     ((*infoPtr).dwStyle & PGS_HORZ) as BOOL, FALSE, (*infoPtr).BRbtnState);

    ReleaseDC( (*infoPtr).hwndSelf, hdc );
    return 0;
}

unsafe extern "C" fn PAGER_HitTest(infoPtr: *const PAGER_INFO, pt: *const POINT) -> INT {
    let mut clientRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcTopLeft: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBottomRight: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut ptWindow: POINT = POINT { x: 0, y: 0 };

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(clientRect));

    if PtInRect(core::ptr::addr_of_mut!(clientRect), *pt) != 0 {
        return -(1);
    }

    ptWindow = *pt;
    PAGER_GetButtonRects(
        infoPtr,
        core::ptr::addr_of_mut!(rcTopLeft),
        core::ptr::addr_of_mut!(rcBottomRight),
        TRUE,
    );

    if ((*infoPtr).TLbtnState != (PGF_INVISIBLE as i32))
        && PtInRect(core::ptr::addr_of_mut!(rcTopLeft), ptWindow) != 0
    {
        return (PGB_TOPORLEFT) as INT;
    } else if ((*infoPtr).BRbtnState != (PGF_INVISIBLE as i32))
        && PtInRect(core::ptr::addr_of_mut!(rcBottomRight), ptWindow) != 0
    {
        return (PGB_BOTTOMORRIGHT) as INT;
    }

    return -(1);
}

unsafe extern "C" fn PAGER_NCHitTest(infoPtr: *const PAGER_INFO, x: INT, y: INT) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut nHit: INT = 0;

    pt.x = x;
    pt.y = y;

    ScreenToClient((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
    nHit = PAGER_HitTest(infoPtr, core::ptr::addr_of_mut!(pt));

    return (if nHit < 0 { HTTRANSPARENT } else { (HTCLIENT) as i32 }) as LRESULT;
}

unsafe extern "C" fn PAGER_MouseMove(infoPtr: *mut PAGER_INFO, keys: INT, x: INT, y: INT) -> LRESULT {
    let mut clpt: POINT = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut wnrect: RECT = core::mem::zeroed();
    let mut topLeft: BOOL = FALSE;
    let mut btnstate: INT = 0;
    let mut hit: INT;
    let mut hdc: HDC;

    pt.x = x;
    pt.y = y;

    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(wnrect));
    if PtInRect(core::ptr::addr_of_mut!(wnrect), pt) != 0 {
        let mut topleft: RECT = core::mem::zeroed();
        let mut bottomright: RECT = core::mem::zeroed();
        let mut rect: *mut RECT = core::ptr::null_mut();

        PAGER_GetButtonRects(
            infoPtr,
            core::ptr::addr_of_mut!(topleft),
            core::ptr::addr_of_mut!(bottomright),
            FALSE,
        );

        clpt = pt;
        MapWindowPoints(
            core::ptr::null_mut(),
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!(clpt),
            1,
        );
        hit = PAGER_HitTest(infoPtr, core::ptr::addr_of_mut!(clpt));
        if (hit == (PGB_TOPORLEFT as i32)) && ((*infoPtr).TLbtnState == (PGF_NORMAL as i32)) {
            topLeft = TRUE;
            rect = core::ptr::addr_of_mut!(topleft);
            (*infoPtr).TLbtnState = (PGF_HOT as i32);
            btnstate = (*infoPtr).TLbtnState;
        } else if (hit == (PGB_BOTTOMORRIGHT as i32)) && ((*infoPtr).BRbtnState == (PGF_NORMAL as i32)) {
            topLeft = FALSE;
            rect = core::ptr::addr_of_mut!(bottomright);
            (*infoPtr).BRbtnState = (PGF_HOT as i32);
            btnstate = (*infoPtr).BRbtnState;
        }

        /* If in one of the buttons the capture and draw buttons */
        if !rect.is_null() {
            if (*infoPtr).bCapture == 0 {
                SetCapture((*infoPtr).hwndSelf);
                (*infoPtr).bCapture = TRUE;
            }
            if ((*infoPtr).dwStyle & PGS_AUTOSCROLL) != 0 {
                SetTimer(
                    (*infoPtr).hwndSelf,
                    (TIMERID1) as UINT_PTR,
                    0x3e,
                    None,
                );
            }
            hdc = GetWindowDC((*infoPtr).hwndSelf);
            /* OffsetRect(wnrect, 0 | 1, 0 | 1) */
            PAGER_DrawButton(
                hdc,
                (*infoPtr).clrBk,
                *rect,
                ((*infoPtr).dwStyle & PGS_HORZ) as BOOL,
                topLeft,
                btnstate,
            );
            ReleaseDC((*infoPtr).hwndSelf, hdc);
            return 0;
        }
    }

    /* If we think we are captured, then do release */
    if ((*infoPtr).bCapture != 0) && (WindowFromPoint(pt) != (*infoPtr).hwndSelf) {
        let mut nmhdr: NMHDR = core::mem::zeroed();

        (*infoPtr).bCapture = FALSE;

        if GetCapture() == (*infoPtr).hwndSelf {
            ReleaseCapture();

            if (*infoPtr).TLbtnState == (PGF_GRAYED as i32) {
                (*infoPtr).TLbtnState = (PGF_INVISIBLE as i32);
                SetWindowPos(
                    (*infoPtr).hwndSelf,
                    core::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE
                        | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            } else if (*infoPtr).TLbtnState == (PGF_HOT as i32) {
                (*infoPtr).TLbtnState = (PGF_NORMAL as i32);
                /* FIXME: just invalidate button rect */
                RedrawWindow(
                    (*infoPtr).hwndSelf,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    RDW_FRAME | RDW_INVALIDATE,
                );
            }

            if (*infoPtr).BRbtnState == (PGF_GRAYED as i32) {
                (*infoPtr).BRbtnState = (PGF_INVISIBLE as i32);
                SetWindowPos(
                    (*infoPtr).hwndSelf,
                    core::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE
                        | SWP_NOZORDER | SWP_NOACTIVATE,
                );
            } else if (*infoPtr).BRbtnState == (PGF_HOT as i32) {
                (*infoPtr).BRbtnState = (PGF_NORMAL as i32);
                /* FIXME: just invalidate button rect */
                RedrawWindow(
                    (*infoPtr).hwndSelf,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                    RDW_FRAME | RDW_INVALIDATE,
                );
            }

            /* Notify parent of released mouse capture */
            memset(
                core::ptr::addr_of_mut!(nmhdr) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<NMHDR>(),
            );
            nmhdr.hwndFrom = (*infoPtr).hwndSelf;
            nmhdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
            nmhdr.code = NM_RELEASEDCAPTURE;
            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY,
                nmhdr.idFrom as WPARAM,
                core::ptr::addr_of_mut!(nmhdr) as LPARAM,
            );
        }
        if IsWindow((*infoPtr).hwndSelf) != 0 {
            KillTimer((*infoPtr).hwndSelf, (TIMERID1) as UINT_PTR);
        }
    }
    return 0;
}

unsafe extern "C" fn PAGER_LButtonDown(
    mut infoPtr: *mut PAGER_INFO,
    mut keys: INT,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    let mut repaintBtns: BOOL = FALSE;
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };
    let mut hit: INT = 0;
    pt.x = x as LONG;
    pt.y = y as LONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_LButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
            862 as ::core::ffi::c_int,
            b"[%p] at (%d,%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            x,
            y,
        );
    }
    hit = PAGER_HitTest(infoPtr, &raw mut pt);
    if hit == (PGB_TOPORLEFT as i32) {
        repaintBtns = ((*infoPtr).TLbtnState != (PGF_DEPRESSED as i32)) as ::core::ffi::c_int as BOOL;
        (*infoPtr).TLbtnState = (PGF_DEPRESSED as i32);
        SetTimer(
            (*infoPtr).hwndSelf,
            TIMERID1 as UINT_PTR,
            INITIAL_DELAY as UINT,
            None,
        );
    } else if hit == (PGB_BOTTOMORRIGHT as i32) {
        repaintBtns = ((*infoPtr).BRbtnState != (PGF_DEPRESSED as i32)) as ::core::ffi::c_int as BOOL;
        (*infoPtr).BRbtnState = (PGF_DEPRESSED as i32);
        SetTimer(
            (*infoPtr).hwndSelf,
            TIMERID1 as UINT_PTR,
            INITIAL_DELAY as UINT,
            None,
        );
    }
    if repaintBtns != 0 {
        SendMessageW(
            (*infoPtr).hwndSelf,
            WM_NCPAINT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
    match (hit) as u32 {
        PGB_TOPORLEFT => {
            if (*infoPtr).dwStyle & PGS_HORZ as DWORD != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/pager.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PAGER_LButtonDown\0" as *const u8
                            as *const ::core::ffi::c_char,
                        888 as ::core::ffi::c_int,
                        b"[%p] PGF_SCROLLLEFT\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                    );
                }
                PAGER_Scroll(infoPtr, (PGF_SCROLLLEFT) as INT);
            } else {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"dll/win32/comctl32/pager.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PAGER_LButtonDown\0" as *const u8
                            as *const ::core::ffi::c_char,
                        893 as ::core::ffi::c_int,
                        b"[%p] PGF_SCROLLUP\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                    );
                }
                PAGER_Scroll(infoPtr, (PGF_SCROLLUP) as INT);
            }
        }
        PGB_BOTTOMORRIGHT => {
            if (*infoPtr).dwStyle & PGS_HORZ as DWORD != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/comctl32/pager.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PAGER_LButtonDown\0" as *const u8
                            as *const ::core::ffi::c_char,
                        900 as ::core::ffi::c_int,
                        b"[%p] PGF_SCROLLRIGHT\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                    );
                }
                PAGER_Scroll(infoPtr, (PGF_SCROLLRIGHT) as INT);
            } else {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"dll/win32/comctl32/pager.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PAGER_LButtonDown\0" as *const u8
                            as *const ::core::ffi::c_char,
                        905 as ::core::ffi::c_int,
                        b"[%p] PGF_SCROLLDOWN\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                    );
                }
                PAGER_Scroll(infoPtr, (PGF_SCROLLDOWN) as INT);
            }
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn PAGER_LButtonUp(
    mut infoPtr: *mut PAGER_INFO,
    mut keys: INT,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    KillTimer((*infoPtr).hwndSelf, TIMERID1 as UINT_PTR);
    KillTimer((*infoPtr).hwndSelf, TIMERID2 as UINT_PTR);

    /* make PRESSED btns NORMAL but don't hide gray btns */
    if ((*infoPtr).TLbtnState & ((PGF_HOT | PGF_DEPRESSED) as i32)) != 0 {
        (*infoPtr).TLbtnState = (PGF_NORMAL as i32);
    }
    if ((*infoPtr).BRbtnState & ((PGF_HOT | PGF_DEPRESSED) as i32)) != 0 {
        (*infoPtr).BRbtnState = (PGF_NORMAL as i32);
    }

    return 0;
}

unsafe extern "C" fn PAGER_Timer(infoPtr: *mut PAGER_INFO, nTimerId: INT) -> LRESULT {
    let mut dir: INT = 0;

    /* if initial timer, kill it and start the repeat timer */
    if nTimerId == TIMERID1 as INT {
        if (*infoPtr).TLbtnState == PGF_HOT as INT {
            dir = (if (*infoPtr).dwStyle & PGS_HORZ as DWORD != 0 {
                PGF_SCROLLLEFT as INT
            } else {
                PGF_SCROLLUP as INT
            }) as INT;
        } else {
            dir = (if (*infoPtr).dwStyle & PGS_HORZ as DWORD != 0 {
                PGF_SCROLLRIGHT as INT
            } else {
                PGF_SCROLLDOWN as INT
            }) as INT;
        }

        KillTimer((*infoPtr).hwndSelf, TIMERID1 as UINT_PTR);
        SetTimer(
            (*infoPtr).hwndSelf,
            TIMERID1 as UINT_PTR,
            REPEAT_DELAY as UINT,
            None,
        );
        if (*infoPtr).dwStyle & PGS_AUTOSCROLL as DWORD != 0 {
            PAGER_Scroll(infoPtr, dir);
            SetWindowPos(
                (*infoPtr).hwndSelf,
                0 as HWND,
                0 as INT,
                0 as INT,
                0 as INT,
                0 as INT,
                (SWP_FRAMECHANGED | SWP_NOSIZE | SWP_NOMOVE
                    | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
            );
        }
        return 0 as LRESULT;
    }

    KillTimer((*infoPtr).hwndSelf, TIMERID2 as UINT_PTR);
    if (*infoPtr).direction > 0 as INT {
        PAGER_Scroll(infoPtr, (*infoPtr).direction);
        SetTimer(
            (*infoPtr).hwndSelf,
            TIMERID2 as UINT_PTR,
            REPEAT_DELAY as UINT,
            None,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn PAGER_EraseBackground(infoPtr: *const PAGER_INFO, hdc: HDC) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut ptorig: POINT = POINT { x: 0, y: 0 };
    let mut parent: HWND = core::ptr::null_mut();
    let mut ret: LRESULT = 0;

    pt.x = 0 as LONG;
    pt.y = 0 as LONG;
    parent = GetParent((*infoPtr).hwndSelf);
    MapWindowPoints(
        (*infoPtr).hwndSelf,
        parent,
        core::ptr::addr_of_mut!(pt),
        1 as UINT,
    );
    OffsetWindowOrgEx(
        hdc,
        pt.x as core::ffi::c_int,
        pt.y as core::ffi::c_int,
        core::ptr::addr_of_mut!(ptorig),
    );
    ret = SendMessageW(
        parent,
        WM_ERASEBKGND as UINT,
        hdc as WPARAM,
        0 as LPARAM,
    );
    SetWindowOrgEx(
        hdc,
        ptorig.x as core::ffi::c_int,
        ptorig.y as core::ffi::c_int,
        core::ptr::null_mut(),
    );

    return ret;
}

unsafe extern "C" fn PAGER_Size(infoPtr: *mut PAGER_INFO, type_: INT, x: INT, y: INT) -> LRESULT {
    /* note that WM_SIZE is sent whenever NCCalcSize resizes the client wnd */

    if ((*infoPtr).dwStyle & (PGS_HORZ as DWORD)) != 0 {
        (*infoPtr).nHeight = y;
    } else {
        (*infoPtr).nWidth = x;
    }

    return PAGER_RecalcSize(infoPtr);
}

unsafe extern "C" fn PAGER_StyleChanged(
    infoPtr: *mut PAGER_INFO,
    wStyleType: WPARAM,
    lpss: *const STYLESTRUCT,
) -> LRESULT {
    let mut oldStyle: DWORD = (*infoPtr).dwStyle;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/pager.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"PAGER_StyleChanged\0" as *const u8 as *const ::core::ffi::c_char,
            1010 as ::core::ffi::c_int,
            b"(styletype=%lx, styleOld=0x%08x, styleNew=0x%08x)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wStyleType,
            (*lpss).styleOld,
            (*lpss).styleNew,
        );
    }
    if wStyleType != GWL_STYLE as WPARAM {
        return 0 as LRESULT;
    }
    (*infoPtr).dwStyle = (*lpss).styleNew;
    if ((oldStyle ^ (*lpss).styleNew) & ((PGS_HORZ | PGS_VERT) as DWORD)) != 0
    {
        PAGER_RecalcSize(infoPtr);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn PAGER_NotifyFormat(infoPtr: *mut PAGER_INFO, command: INT) -> LRESULT {
    let mut ret: INT = 0;
    match (command) as u32 {
        NF_REQUERY => {
            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFYFORMAT as UINT,
                (*infoPtr).hwndSelf as WPARAM,
                NF_QUERY as LPARAM,
            ) as INT;
            (*infoPtr).bUnicode = ((ret) as u32 == NFR_UNICODE) as ::core::ffi::c_int as BOOL;
            return ret as LRESULT;
        }
        NF_QUERY => {
            /* Pager always wants Unicode notifications from children */
            return NFR_UNICODE as LRESULT;
        }
        _ => {
            return 0 as LRESULT;
        }
    };
}

unsafe extern "C" fn PAGER_GetAnsiNtfCode(code: UINT) -> UINT {
    match (code) as i32 {
        CBEN_DRAGBEGINW => return (CBEN_DRAGBEGINA) as u32,
        CBEN_ENDEDITW => return (CBEN_ENDEDITA) as u32,
        CBEN_GETDISPINFOW => return (CBEN_GETDISPINFOA) as u32,
        DTN_FORMATW => return (DTN_FORMATA) as u32,
        DTN_FORMATQUERYW => return (DTN_FORMATQUERYA) as u32,
        DTN_USERSTRINGW => return (DTN_USERSTRINGA) as u32,
        DTN_WMKEYDOWNW => return (DTN_WMKEYDOWNA) as u32,
        HDN_BEGINTRACKW => return (HDN_BEGINTRACKA) as u32,
        HDN_DIVIDERDBLCLICKW => return (HDN_DIVIDERDBLCLICKA) as u32,
        HDN_ENDTRACKW => return (HDN_ENDTRACKA) as u32,
        HDN_GETDISPINFOW => return (HDN_GETDISPINFOA) as u32,
        HDN_ITEMCHANGEDW => return (HDN_ITEMCHANGEDA) as u32,
        HDN_ITEMCHANGINGW => return (HDN_ITEMCHANGINGA) as u32,
        HDN_ITEMCLICKW => return (HDN_ITEMCLICKA) as u32,
        HDN_ITEMDBLCLICKW => return (HDN_ITEMDBLCLICKA) as u32,
        HDN_TRACKW => return (HDN_TRACKA) as u32,
        LVN_BEGINLABELEDITW => return (LVN_BEGINLABELEDITA) as u32,
        LVN_ENDLABELEDITW => return (LVN_ENDLABELEDITA) as u32,
        LVN_GETDISPINFOW => return (LVN_GETDISPINFOA) as u32,
        LVN_GETINFOTIPW => return (LVN_GETINFOTIPA) as u32,
        LVN_INCREMENTALSEARCHW => return (LVN_INCREMENTALSEARCHA) as u32,
        LVN_ODFINDITEMW => return (LVN_ODFINDITEMA) as u32,
        LVN_SETDISPINFOW => return (LVN_SETDISPINFOA) as u32,
        TBN_GETBUTTONINFOW => return (TBN_GETBUTTONINFOA) as u32,
        TBN_GETINFOTIPW => return (TBN_GETINFOTIPA) as u32,
        TTN_GETDISPINFOW => return (TTN_GETDISPINFOA) as u32,
        TVN_BEGINDRAGW => return (TVN_BEGINDRAGA) as u32,
        TVN_BEGINLABELEDITW => return (TVN_BEGINLABELEDITA) as u32,
        TVN_BEGINRDRAGW => return (TVN_BEGINRDRAGA) as u32,
        TVN_DELETEITEMW => return (TVN_DELETEITEMA) as u32,
        TVN_ENDLABELEDITW => return (TVN_ENDLABELEDITA) as u32,
        TVN_GETDISPINFOW => return (TVN_GETDISPINFOA) as u32,
        TVN_GETINFOTIPW => return (TVN_GETINFOTIPA) as u32,
        TVN_ITEMEXPANDEDW => return (TVN_ITEMEXPANDEDA) as u32,
        TVN_ITEMEXPANDINGW => return (TVN_ITEMEXPANDINGA) as u32,
        TVN_SELCHANGEDW => return (TVN_SELCHANGEDA) as u32,
        TVN_SELCHANGINGW => return (TVN_SELCHANGINGA) as u32,
        TVN_SETDISPINFOW => return (TVN_SETDISPINFOA) as u32,
        _ => {}
    }
    return code;
}

unsafe extern "C" fn PAGER_AdjustBuffer(infoPtr: *mut PAGER_INFO, size: INT) -> BOOL {
    if (*infoPtr).pwszBuffer.is_null() {
        (*infoPtr).pwszBuffer = heap_alloc(size as SIZE_T) as *mut WCHAR;
    } else if (*infoPtr).nBufferSize < size {
        (*infoPtr).pwszBuffer = heap_realloc(
            (*infoPtr).pwszBuffer as *mut ::core::ffi::c_void,
            size as SIZE_T,
        ) as *mut WCHAR;
    }

    if (*infoPtr).pwszBuffer.is_null() {
        return FALSE;
    }
    if (*infoPtr).nBufferSize < size {
        (*infoPtr).nBufferSize = size;
    }

    return TRUE;
}

unsafe extern "C" fn PAGER_ConvertText(text: *mut *mut WCHAR) -> *mut WCHAR {
    let mut oldText: *mut WCHAR = *text;
    *text = core::ptr::null_mut::<WCHAR>();
    Str_SetPtrWtoA(text as *mut *mut CHAR, oldText as LPCWSTR);
    return oldText;
}

unsafe extern "C" fn PAGER_RestoreText(text: *mut *mut WCHAR, oldText: *mut WCHAR) {
    if oldText.is_null() {
        return;
    }
    Free(*text as LPVOID);
    *text = oldText;
}

unsafe extern "C" fn PAGER_SendConvertedNotify(
    infoPtr: *mut PAGER_INFO,
    hdr: *mut NMHDR,
    mask: *mut UINT,
    requiredMask: UINT,
    text: *mut *mut WCHAR,
    textMax: *mut INT,
    flags: DWORD,
) -> LRESULT {
    let mut sendBuffer: *mut CHAR = core::ptr::null_mut();
    let mut receiveBuffer: *mut CHAR;
    let mut bufferSize: INT;
    let mut oldText: *mut WCHAR;
    let mut oldTextMax: INT;
    let mut ret: LRESULT = NO_ERROR as LRESULT;

    oldText = *text;
    oldTextMax = if !textMax.is_null() { *textMax } else { 0 };

    (*hdr).code = PAGER_GetAnsiNtfCode((*hdr).code);

    'done: {
        if !mask.is_null() && (*mask & requiredMask) == 0 {
            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                hdr as LPARAM,
            );
            if (flags & SET_NULL_IF_NO_MASK as DWORD) != 0 {
                oldText = core::ptr::null_mut();
            }
            break 'done;
        }

        if oldTextMax < 0 {
            break 'done;
        }

        if (!(*text).is_null() && (flags & (CONVERT_SEND | ZERO_SEND) as DWORD) != 0)
            || ((*text).is_null() && (flags & SEND_EMPTY_IF_NULL as DWORD) != 0)
        {
            bufferSize = if !textMax.is_null() {
                *textMax
            } else {
                lstrlenW(*text as LPCWSTR) + 1
            };
            sendBuffer = heap_alloc_zero(bufferSize as SIZE_T) as *mut CHAR;
            if sendBuffer.is_null() {
                break 'done;
            }
            if (flags & ZERO_SEND as DWORD) == 0 {
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    *text as LPCWSTR,
                    -1,
                    sendBuffer as LPSTR,
                    bufferSize as core::ffi::c_int,
                    core::ptr::null::<CHAR>(),
                    FALSE as *mut i32,
                );
            }
            *text = sendBuffer as *mut WCHAR;
        }

        ret = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            (*hdr).idFrom as WPARAM,
            hdr as LPARAM,
        );

        if !(*text).is_null() && !oldText.is_null() && (flags & CONVERT_RECEIVE as DWORD) != 0 {
            /* MultiByteToWideChar requires that source and destination are not the same buffer */
            if *text == oldText {
                bufferSize = (lstrlenA(*text as *mut CHAR as LPCSTR) + 1) as INT;
                receiveBuffer = heap_alloc(bufferSize as SIZE_T) as *mut CHAR;
                if receiveBuffer.is_null() {
                    break 'done;
                }
                memcpy(
                    receiveBuffer as *mut core::ffi::c_void,
                    *text as *const core::ffi::c_void,
                    bufferSize as usize,
                );
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    receiveBuffer as LPCSTR,
                    bufferSize as core::ffi::c_int,
                    oldText as LPWSTR,
                    oldTextMax as core::ffi::c_int,
                );
                heap_free(receiveBuffer as *mut core::ffi::c_void);
            } else {
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    *text as *mut CHAR as LPCSTR,
                    -1,
                    oldText as LPWSTR,
                    oldTextMax as core::ffi::c_int,
                );
            }
        }
    }

    heap_free(sendBuffer as *mut core::ffi::c_void);
    *text = oldText;
    return ret;
}

unsafe extern "C" fn PAGER_Notify(infoPtr: *mut PAGER_INFO, hdr: *mut NMHDR) -> LRESULT {
    let mut ret: LRESULT = 0;

    if (*infoPtr).bUnicode != 0 {
        return SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            (*hdr).idFrom as WPARAM,
            hdr as LPARAM,
        );
    }

    match ((*hdr).code) as i32 {
        CBEN_GETDISPINFOW => {
            let nmcbe: *mut NMCOMBOBOXEXW = hdr as *mut NMCOMBOBOXEXW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmcbe).ceItem.mask),
                CBEIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmcbe).ceItem.pszText),
                core::ptr::addr_of_mut!((*nmcbe).ceItem.cchTextMax),
                (ZERO_SEND | SET_NULL_IF_NO_MASK | CONVERT_RECEIVE) as DWORD,
            );
        }
        CBEN_DRAGBEGINW => {
            let nmdbW: *mut NMCBEDRAGBEGINW = hdr as *mut NMCBEDRAGBEGINW;
            let mut nmdbA: NMCBEDRAGBEGINA = NMCBEDRAGBEGINA {
                hdr: tagNMHDR {
                    hwndFrom: 0 as HWND,
                    idFrom: 0,
                    code: 0,
                },
                iItemid: 0,
                szText: [0; 260],
            };
            nmdbA.hdr.code = PAGER_GetAnsiNtfCode((*nmdbW).hdr.code);
            nmdbA.hdr.hwndFrom = (*nmdbW).hdr.hwndFrom;
            nmdbA.hdr.idFrom = (*nmdbW).hdr.idFrom;
            nmdbA.iItemid = (*nmdbW).iItemid;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*nmdbW).szText) as *mut WCHAR as LPCWSTR,
                ARRAY_SIZE!((*nmdbW).szText) as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(nmdbA.szText) as LPSTR,
                ARRAY_SIZE!(nmdbA.szText) as ::core::ffi::c_int,
                core::ptr::null_mut::<CHAR>(),
                FALSE as *mut BOOL,
            );
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                core::ptr::addr_of_mut!(nmdbA) as LPARAM,
            );
        }
        CBEN_ENDEDITW => {
            let nmedW: *mut NMCBEENDEDITW = hdr as *mut NMCBEENDEDITW;
            let mut nmedA: NMCBEENDEDITA = NMCBEENDEDITA {
                hdr: tagNMHDR {
                    hwndFrom: 0 as HWND,
                    idFrom: 0,
                    code: 0,
                },
                fChanged: 0,
                iNewSelection: 0,
                szText: [0; 260],
                iWhy: 0,
            };
            nmedA.hdr.code = PAGER_GetAnsiNtfCode((*nmedW).hdr.code);
            nmedA.hdr.hwndFrom = (*nmedW).hdr.hwndFrom;
            nmedA.hdr.idFrom = (*nmedW).hdr.idFrom;
            nmedA.fChanged = (*nmedW).fChanged;
            nmedA.iNewSelection = (*nmedW).iNewSelection;
            nmedA.iWhy = (*nmedW).iWhy;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*nmedW).szText) as *mut WCHAR as LPCWSTR,
                ARRAY_SIZE!((*nmedW).szText) as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(nmedA.szText) as LPSTR,
                ARRAY_SIZE!(nmedA.szText) as ::core::ffi::c_int,
                core::ptr::null_mut::<CHAR>(),
                FALSE as *mut BOOL,
            );
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                core::ptr::addr_of_mut!(nmedA) as LPARAM,
            );
        }
        DTN_FORMATW => {
            let nmdtf: *mut NMDATETIMEFORMATW = hdr as *mut NMDATETIMEFORMATW;
            let mut oldFormat: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            let mut textLength: INT = 0;

            (*hdr).code = PAGER_GetAnsiNtfCode((*hdr).code);
            oldFormat = PAGER_ConvertText(core::ptr::addr_of_mut!((*nmdtf).pszFormat) as *mut *mut WCHAR);
            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                nmdtf as LPARAM,
            );
            PAGER_RestoreText(core::ptr::addr_of_mut!((*nmdtf).pszFormat) as *mut *mut WCHAR, oldFormat);

            if !(*nmdtf).pszDisplay.is_null() {
                textLength = MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*nmdtf).pszDisplay as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    0 as *mut WCHAR,
                    0 as ::core::ffi::c_int,
                ) as INT;
                if PAGER_AdjustBuffer(
                    infoPtr,
                    (textLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as INT,
                ) == 0
                {
                    return ret;
                }
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    (*nmdtf).pszDisplay as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    (*infoPtr).pwszBuffer as LPWSTR,
                    textLength as ::core::ffi::c_int,
                );
                if (*nmdtf).pszDisplay
                    != core::ptr::addr_of_mut!((*nmdtf).szDisplay) as *mut WCHAR as LPCWSTR
                {
                    (*nmdtf).pszDisplay = (*infoPtr).pwszBuffer as LPCWSTR;
                } else {
                    textLength = min(
                        textLength,
                        ARRAY_SIZE!((*nmdtf).szDisplay) as INT,
                    );
                    memcpy(
                        core::ptr::addr_of_mut!((*nmdtf).szDisplay) as *mut ::core::ffi::c_void,
                        (*infoPtr).pwszBuffer as *const ::core::ffi::c_void,
                        (textLength as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                    );
                }
            }

            return ret;
        }
        DTN_FORMATQUERYW => {
            let nmdtfq: *mut NMDATETIMEFORMATQUERYW = hdr as *mut NMDATETIMEFORMATQUERYW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmdtfq).pszFormat) as *mut *mut WCHAR,
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        DTN_WMKEYDOWNW => {
            let nmdtkd: *mut NMDATETIMEWMKEYDOWNW = hdr as *mut NMDATETIMEWMKEYDOWNW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmdtkd).pszFormat) as *mut *mut WCHAR,
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        DTN_USERSTRINGW => {
            let nmdts: *mut NMDATETIMESTRINGW = hdr as *mut NMDATETIMESTRINGW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmdts).pszUserString) as *mut *mut WCHAR,
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        HDN_BEGINTRACKW
        | HDN_DIVIDERDBLCLICKW
        | HDN_ENDTRACKW
        | HDN_ITEMCHANGEDW
        | HDN_ITEMCHANGINGW
        | HDN_ITEMCLICKW
        | HDN_ITEMDBLCLICKW
        | HDN_TRACKW => {
            let nmh: *mut NMHEADERW = hdr as *mut NMHEADERW;
            let mut oldText: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            let mut oldFilterText: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            let mut tf: *mut HD_TEXTFILTERW = core::ptr::null_mut::<HD_TEXTFILTERW>();

            (*hdr).code = PAGER_GetAnsiNtfCode((*hdr).code);

            if (*nmh).pitem.is_null() {
                return SendMessageW(
                    (*infoPtr).hwndNotify,
                    WM_NOTIFY as UINT,
                    (*hdr).idFrom as WPARAM,
                    hdr as LPARAM,
                );
            }
            if ((*(*nmh).pitem).mask & (HDI_TEXT as UINT)) != 0 {
                oldText = PAGER_ConvertText(core::ptr::addr_of_mut!((*(*nmh).pitem).pszText));
            }
            if ((*(*nmh).pitem).mask & (HDI_FILTER as UINT)) != 0
                && ((*(*nmh).pitem).type_ == (HDFT_ISSTRING as UINT))
                && !(*(*nmh).pitem).pvFilter.is_null()
            {
                tf = (*(*nmh).pitem).pvFilter as *mut HD_TEXTFILTERW;
                oldFilterText = PAGER_ConvertText(core::ptr::addr_of_mut!((*tf).pszText));
            }
            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                hdr as LPARAM,
            );
            PAGER_RestoreText(core::ptr::addr_of_mut!((*(*nmh).pitem).pszText), oldText);
            if !tf.is_null() {
                PAGER_RestoreText(core::ptr::addr_of_mut!((*tf).pszText), oldFilterText);
            }
            return ret;
        }
        HDN_GETDISPINFOW => {
            let nmhddi: *mut NMHDDISPINFOW = hdr as *mut NMHDDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmhddi).mask),
                HDI_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmhddi).pszText),
                core::ptr::addr_of_mut!((*nmhddi).cchTextMax),
                (SEND_EMPTY_IF_NULL | CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        LVN_BEGINLABELEDITW | LVN_ENDLABELEDITW | LVN_SETDISPINFOW => {
            let nmlvdi: *mut NMLVDISPINFOW = hdr as *mut NMLVDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmlvdi).item.mask),
                LVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmlvdi).item.pszText),
                core::ptr::addr_of_mut!((*nmlvdi).item.cchTextMax),
                (SET_NULL_IF_NO_MASK | CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        LVN_GETDISPINFOW => {
            let nmlvdi: *mut NMLVDISPINFOW = hdr as *mut NMLVDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmlvdi).item.mask),
                LVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmlvdi).item.pszText),
                core::ptr::addr_of_mut!((*nmlvdi).item.cchTextMax),
                CONVERT_RECEIVE as DWORD,
            );
        }
        LVN_GETINFOTIPW => {
            let nmlvgit: *mut NMLVGETINFOTIPW = hdr as *mut NMLVGETINFOTIPW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmlvgit).pszText),
                core::ptr::addr_of_mut!((*nmlvgit).cchTextMax),
                (CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        LVN_INCREMENTALSEARCHW | LVN_ODFINDITEMW => {
            let nmlvfi: *mut NMLVFINDITEMW = hdr as *mut NMLVFINDITEMW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmlvfi).lvfi.flags),
                (LVFI_STRING | LVFI_SUBSTRING) as UINT,
                core::ptr::addr_of_mut!((*nmlvfi).lvfi.psz) as *mut *mut WCHAR,
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        TBN_GETBUTTONINFOW => {
            let nmtb: *mut NMTOOLBARW = hdr as *mut NMTOOLBARW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmtb).pszText),
                core::ptr::addr_of_mut!((*nmtb).cchText),
                (SEND_EMPTY_IF_NULL | CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        TBN_GETINFOTIPW => {
            let nmtbgit: *mut NMTBGETINFOTIPW = hdr as *mut NMTBGETINFOTIPW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmtbgit).pszText),
                core::ptr::addr_of_mut!((*nmtbgit).cchTextMax),
                CONVERT_RECEIVE as DWORD,
            );
        }
        TTN_GETDISPINFOW => {
            let nmttdiW: *mut NMTTDISPINFOW = hdr as *mut NMTTDISPINFOW;
            let mut nmttdiA: NMTTDISPINFOA = NMTTDISPINFOA {
                hdr: tagNMHDR {
                    hwndFrom: 0 as HWND,
                    idFrom: 0,
                    code: 0,
                },
                lpszText: core::ptr::null_mut::<CHAR>(),
                szText: [0; 80],
                hinst: core::ptr::null_mut::<HINSTANCE__>(),
                uFlags: 0,
                lParam: 0,
            };
            let mut size: INT = 0;

            nmttdiA.hdr.code = PAGER_GetAnsiNtfCode((*nmttdiW).hdr.code);
            nmttdiA.hdr.hwndFrom = (*nmttdiW).hdr.hwndFrom;
            nmttdiA.hdr.idFrom = (*nmttdiW).hdr.idFrom;
            nmttdiA.hinst = (*nmttdiW).hinst;
            nmttdiA.uFlags = (*nmttdiW).uFlags;
            nmttdiA.lParam = (*nmttdiW).lParam;
            nmttdiA.lpszText = core::ptr::addr_of_mut!(nmttdiA.szText) as LPSTR;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*nmttdiW).szText) as *mut WCHAR as LPCWSTR,
                ARRAY_SIZE!((*nmttdiW).szText) as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(nmttdiA.szText) as LPSTR,
                ARRAY_SIZE!(nmttdiA.szText) as ::core::ffi::c_int,
                core::ptr::null_mut::<CHAR>(),
                FALSE as *mut BOOL,
            );

            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                core::ptr::addr_of_mut!(nmttdiA) as LPARAM,
            );

            (*nmttdiW).hinst = nmttdiA.hinst;
            (*nmttdiW).uFlags = nmttdiA.uFlags;
            (*nmttdiW).lParam = nmttdiA.lParam;

            MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(nmttdiA.szText) as LPCSTR,
                ARRAY_SIZE!(nmttdiA.szText) as ::core::ffi::c_int,
                core::ptr::addr_of_mut!((*nmttdiW).szText) as LPWSTR,
                ARRAY_SIZE!((*nmttdiW).szText) as ::core::ffi::c_int,
            );
            if nmttdiA.lpszText.is_null() {
                (*nmttdiW).lpszText = core::ptr::addr_of_mut!((*nmttdiW).szText) as *mut WCHAR as LPWSTR;
            } else if IS_INTRESOURCE!(nmttdiA.lpszText) == false {
                size = MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    nmttdiA.lpszText as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    0 as *mut WCHAR,
                    0 as ::core::ffi::c_int,
                ) as INT;
                if size > (ARRAY_SIZE!((*nmttdiW).szText) as INT) {
                    if PAGER_AdjustBuffer(
                        infoPtr,
                        (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as INT,
                    ) == 0
                    {
                        return ret;
                    }
                    MultiByteToWideChar(
                        CP_ACP as UINT,
                        0 as DWORD,
                        nmttdiA.lpszText as LPCSTR,
                        -(1 as ::core::ffi::c_int),
                        (*infoPtr).pwszBuffer as LPWSTR,
                        size as ::core::ffi::c_int,
                    );
                    (*nmttdiW).lpszText = (*infoPtr).pwszBuffer as LPWSTR;
                    /* Override content in szText */
                    memcpy(
                        core::ptr::addr_of_mut!((*nmttdiW).szText) as *mut WCHAR as *mut ::core::ffi::c_void,
                        (*nmttdiW).lpszText as *const ::core::ffi::c_void,
                        min(
                            core::mem::size_of::<[WCHAR; 80]>(),
                            (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                        ),
                    );
                } else {
                    MultiByteToWideChar(
                        CP_ACP as UINT,
                        0 as DWORD,
                        nmttdiA.lpszText as LPCSTR,
                        -(1 as ::core::ffi::c_int),
                        core::ptr::addr_of_mut!((*nmttdiW).szText) as LPWSTR,
                        ARRAY_SIZE!((*nmttdiW).szText) as ::core::ffi::c_int,
                    );
                    (*nmttdiW).lpszText = core::ptr::addr_of_mut!((*nmttdiW).szText) as *mut WCHAR as LPWSTR;
                }
            } else {
                (*nmttdiW).szText[0 as usize] = 0 as WCHAR;
                (*nmttdiW).lpszText = nmttdiA.lpszText as *mut WCHAR as LPWSTR;
            }

            return ret;
        }
        TVN_BEGINDRAGW
        | TVN_BEGINRDRAGW
        | TVN_ITEMEXPANDEDW
        | TVN_ITEMEXPANDINGW => {
            let nmtv: *mut NMTREEVIEWW = hdr as *mut NMTREEVIEWW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmtv).itemNew.mask),
                TVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmtv).itemNew.pszText),
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        TVN_DELETEITEMW => {
            let nmtv: *mut NMTREEVIEWW = hdr as *mut NMTREEVIEWW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmtv).itemOld.mask),
                TVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmtv).itemOld.pszText),
                core::ptr::null_mut::<INT>(),
                CONVERT_SEND as DWORD,
            );
        }
        TVN_BEGINLABELEDITW | TVN_ENDLABELEDITW => {
            let nmtvdi: *mut NMTVDISPINFOW = hdr as *mut NMTVDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmtvdi).item.mask),
                TVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmtvdi).item.pszText),
                core::ptr::addr_of_mut!((*nmtvdi).item.cchTextMax),
                (SET_NULL_IF_NO_MASK | CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        TVN_SELCHANGINGW | TVN_SELCHANGEDW => {
            let nmtv: *mut NMTREEVIEWW = hdr as *mut NMTREEVIEWW;
            let mut oldItemOldText: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
            let mut oldItemNewText: *mut WCHAR = core::ptr::null_mut::<WCHAR>();

            (*hdr).code = PAGER_GetAnsiNtfCode((*hdr).code);

            if ((((*nmtv).itemNew.mask | (*nmtv).itemOld.mask) & (TVIF_TEXT as UINT))) == 0 {
                return SendMessageW(
                    (*infoPtr).hwndNotify,
                    WM_NOTIFY as UINT,
                    (*hdr).idFrom as WPARAM,
                    hdr as LPARAM,
                );
            }

            if ((*nmtv).itemOld.mask & (TVIF_TEXT as UINT)) != 0 {
                oldItemOldText = PAGER_ConvertText(core::ptr::addr_of_mut!((*nmtv).itemOld.pszText));
            }
            if ((*nmtv).itemNew.mask & (TVIF_TEXT as UINT)) != 0 {
                oldItemNewText = PAGER_ConvertText(core::ptr::addr_of_mut!((*nmtv).itemNew.pszText));
            }

            ret = SendMessageW(
                (*infoPtr).hwndNotify,
                WM_NOTIFY as UINT,
                (*hdr).idFrom as WPARAM,
                hdr as LPARAM,
            );
            PAGER_RestoreText(core::ptr::addr_of_mut!((*nmtv).itemOld.pszText), oldItemOldText);
            PAGER_RestoreText(core::ptr::addr_of_mut!((*nmtv).itemNew.pszText), oldItemNewText);
            return ret;
        }
        TVN_GETDISPINFOW => {
            let nmtvdi: *mut NMTVDISPINFOW = hdr as *mut NMTVDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmtvdi).item.mask),
                TVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmtvdi).item.pszText),
                core::ptr::addr_of_mut!((*nmtvdi).item.cchTextMax),
                (ZERO_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        TVN_SETDISPINFOW => {
            let nmtvdi: *mut NMTVDISPINFOW = hdr as *mut NMTVDISPINFOW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::addr_of_mut!((*nmtvdi).item.mask),
                TVIF_TEXT as UINT,
                core::ptr::addr_of_mut!((*nmtvdi).item.pszText),
                core::ptr::addr_of_mut!((*nmtvdi).item.cchTextMax),
                (SET_NULL_IF_NO_MASK | CONVERT_SEND | CONVERT_RECEIVE) as DWORD,
            );
        }
        TVN_GETINFOTIPW => {
            let nmtvgit: *mut NMTVGETINFOTIPW = hdr as *mut NMTVGETINFOTIPW;
            return PAGER_SendConvertedNotify(
                infoPtr,
                hdr,
                core::ptr::null_mut::<UINT>(),
                0 as UINT,
                core::ptr::addr_of_mut!((*nmtvgit).pszText),
                core::ptr::addr_of_mut!((*nmtvgit).cchTextMax),
                CONVERT_RECEIVE as DWORD,
            );
        }
        _ => {}
    }

    /* Other notifications, no need to convert */
    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*hdr).idFrom as WPARAM,
        hdr as LPARAM,
    );
}

unsafe extern "system" fn PAGER_WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut PAGER_INFO = GetWindowLongPtrW(hwnd, 0) as *mut PAGER_INFO;

    if infoPtr.is_null() && (uMsg != WM_CREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }

    match uMsg {
        EM_FMTLINES => {
            return PAGER_FmtLines(infoPtr);
        }
        PGM_FORWARDMOUSE => {
            return PAGER_ForwardMouse(infoPtr, wParam as BOOL);
        }
        PGM_GETBKCOLOR => {
            return (PAGER_GetBkColor(infoPtr)) as LRESULT;
        }
        PGM_GETBORDER => {
            return (PAGER_GetBorder(infoPtr)) as LRESULT;
        }
        PGM_GETBUTTONSIZE => {
            return (PAGER_GetButtonSize(infoPtr)) as LRESULT;
        }
        PGM_GETPOS => {
            return (PAGER_GetPos(infoPtr)) as LRESULT;
        }
        PGM_GETBUTTONSTATE => {
            return PAGER_GetButtonState(infoPtr, lParam as INT);
        }
        /*      case PGM_GETDROPTARGET: */
        PGM_RECALCSIZE => {
            return PAGER_RecalcSize(infoPtr);
        }
        PGM_SETBKCOLOR => {
            return (PAGER_SetBkColor(infoPtr, lParam as COLORREF)) as LRESULT;
        }
        PGM_SETBORDER => {
            return (PAGER_SetBorder(infoPtr, lParam as INT)) as LRESULT;
        }
        PGM_SETBUTTONSIZE => {
            return (PAGER_SetButtonSize(infoPtr, lParam as INT)) as LRESULT;
        }
        PGM_SETCHILD => {
            return PAGER_SetChild(infoPtr, lParam as HWND);
        }
        PGM_SETPOS => {
            return PAGER_SetPos(infoPtr, lParam as INT, FALSE, TRUE);
        }
        WM_CREATE => {
            return PAGER_Create(hwnd, lParam as LPCREATESTRUCTW);
        }
        WM_DESTROY => {
            return PAGER_Destroy(infoPtr);
        }
        WM_SIZE => {
            return PAGER_Size(
                infoPtr,
                wParam as INT,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            );
        }
        WM_NCPAINT => {
            return PAGER_NCPaint(infoPtr, wParam as HRGN);
        }
        WM_STYLECHANGED => {
            return PAGER_StyleChanged(infoPtr, wParam, lParam as LPSTYLESTRUCT);
        }
        WM_NCCALCSIZE => {
            return PAGER_NCCalcSize(infoPtr, wParam, lParam as LPRECT);
        }
        WM_NCHITTEST => {
            return PAGER_NCHitTest(
                infoPtr,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            );
        }
        WM_MOUSEMOVE => {
            if (*infoPtr).bForward != 0 && !(*infoPtr).hwndChild.is_null() {
                PostMessageW((*infoPtr).hwndChild, WM_MOUSEMOVE, wParam, lParam);
            }
            return PAGER_MouseMove(
                infoPtr,
                wParam as INT,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            );
        }
        WM_LBUTTONDOWN => {
            return PAGER_LButtonDown(
                infoPtr,
                wParam as INT,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            );
        }
        WM_LBUTTONUP => {
            return PAGER_LButtonUp(
                infoPtr,
                wParam as INT,
                (LOWORD!(lParam) as i16) as INT,
                (HIWORD!(lParam) as i16) as INT,
            );
        }
        WM_ERASEBKGND => {
            return PAGER_EraseBackground(infoPtr, wParam as HDC);
        }
        WM_TIMER => {
            return PAGER_Timer(infoPtr, wParam as INT);
        }
        WM_NOTIFYFORMAT => {
            return PAGER_NotifyFormat(infoPtr, (lParam) as INT);
        }
        WM_NOTIFY => {
            return PAGER_Notify(infoPtr, lParam as *mut NMHDR);
        }
        WM_COMMAND => {
            return SendMessageW((*infoPtr).hwndNotify, uMsg, wParam, lParam);
        }
        _ => {
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn PAGER_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(&raw mut wndClass as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSW>());
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc = Some(PAGER_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut PAGER_INFO>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = WC_PAGESCROLLERW.as_ptr() as LPCWSTR;

    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn PAGER_Unregister() {
    UnregisterClassW((WC_PAGESCROLLERW).as_ptr(), core::ptr::null_mut());
}
