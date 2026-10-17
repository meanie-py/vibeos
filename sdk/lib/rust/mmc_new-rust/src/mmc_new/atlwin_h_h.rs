use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn IsWindow(hWnd: HWND) -> BOOL;
}






































impl CWindow {
    pub unsafe fn __ctor_0(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWnd;
    }
    pub unsafe fn op_to_HWND(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).m_hWnd;
    }
    pub unsafe fn Attach(&mut self, hWndNew: HWND) {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWndNew;
    }
    pub unsafe fn BeginPaint(&mut self, lpPaint: LPPAINTSTRUCT) -> HDC {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return BeginPaint((*this_).m_hWnd, lpPaint);
    }
    pub unsafe fn CenterWindow(&mut self, mut hWndCenter: HWND) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        if hWndCenter.is_null() {
            hWndCenter = GetParent((*this_).m_hWnd);
        }
        if hWndCenter.is_null() {
            return (FALSE) as BOOL;
        }
        let mut wndCenterRect: RECT = core::mem::zeroed();
        let mut wndRect: RECT = core::mem::zeroed();
        if GetWindowRect(hWndCenter, core::ptr::addr_of_mut!(wndCenterRect)) == 0
            || GetWindowRect((*this_).m_hWnd, core::ptr::addr_of_mut!(wndRect)) == 0
        {
            return (FALSE) as BOOL;
        }
        let wndCenterWidth: i32 = wndCenterRect.right - wndCenterRect.left;
        let wndCenterHeight: i32 = wndCenterRect.bottom - wndCenterRect.top;
        let wndWidth: i32 = wndRect.right - wndRect.left;
        let wndHeight: i32 = wndRect.bottom - wndRect.top;
        let mut xPos: i32 = wndCenterRect.left
            + ((wndCenterWidth - wndWidth + 1) >> 1);
        let mut yPos: i32 = wndCenterRect.top
            + ((wndCenterHeight - wndHeight + 1) >> 1);

        if (GetWindowLong(hWndCenter, GWL_STYLE) & (WS_CHILD as i32)) == 0 {
            let mut mi: MONITORINFO = core::mem::zeroed();
            mi.cbSize = core::mem::size_of::<MONITORINFO>() as _;
            let hMonitor: HMONITOR = MonitorFromWindow(hWndCenter, MONITOR_DEFAULTTOPRIMARY);
            GetMonitorInfo(hMonitor, core::ptr::addr_of_mut!(mi));

            if xPos + wndWidth > mi.rcWork.right {
                xPos = mi.rcWork.right - wndWidth;
            } else if xPos < mi.rcWork.left {
                xPos = mi.rcWork.left;
            }

            if yPos + wndHeight > mi.rcWork.bottom {
                yPos = mi.rcWork.bottom - wndHeight;
            }
            if yPos < mi.rcWork.top {
                yPos = mi.rcWork.top;
            }
        }
        return MoveWindow(
            (*this_).m_hWnd,
            xPos,
            yPos,
            wndWidth,
            wndHeight,
            (TRUE as i32),
        );
    }
    pub unsafe fn CheckDlgButton(&mut self, nIDButton: i32, nCheck: UINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return CheckDlgButton((*this_).m_hWnd, nIDButton, nCheck);
    }
    pub unsafe fn ClientToScreen__0(&mut self, lpPoint: LPPOINT) -> i32 {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ClientToScreen((*this_).m_hWnd, lpPoint);
    }
    pub unsafe fn DestroyWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);

        if DestroyWindow((*this_).m_hWnd) == 0 {
            return (FALSE) as BOOL;
        }

        (*this_).m_hWnd = core::ptr::null_mut();
        return (TRUE) as BOOL;
    }
    pub unsafe fn EnableWindow(&mut self, bEnable: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return EnableWindow((*this_).m_hWnd, bEnable);
    }
    pub unsafe fn EndPaint(&mut self, lpPaint: LPPAINTSTRUCT) {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            EndPaint((*this_).m_hWnd, lpPaint);
        }
    pub unsafe fn GetClientRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetClientRect((*this_).m_hWnd, lpRect);
    }
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetMenu(&mut self) -> HMENU {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetMenu((*this_).m_hWnd);
    }
    pub unsafe fn GetWindowPlacement(&mut self, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowPlacement((*this_).m_hWnd, lpwndpl);
    }
    pub unsafe fn GetWindowRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowRect((*this_).m_hWnd, lpRect);
    }
    pub unsafe fn GetWindowTextW__0(&mut self, lpszStringBuf: LPTSTR, nMaxCount: i32) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowText((*this_).m_hWnd, lpszStringBuf, nMaxCount);
    }
    pub unsafe fn GetWindowTextW__1(&mut self, strText: &mut CSimpleString) -> i32 {
        let this_: *mut Self = self;
        let mut len: i32 = (*this_).GetWindowTextLengthW();
        len = (*this_).GetWindowTextW__0(strText.GetBuffer(len + 1), len + 1);
        strText.ReleaseBufferSetLength(len);
        return len;
    }
    pub unsafe fn GetWindowTextLengthW(&mut self) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetWindowTextLength((*this_).m_hWnd);
    }
    pub unsafe fn IsDlgButtonChecked(&mut self, nIDButton: i32) -> UINT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsDlgButtonChecked((*this_).m_hWnd, nIDButton);
    }
    pub unsafe fn MoveWindow(&mut self, x: i32, y: i32, nWidth: i32, nHeight: i32, bRepaint: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return MoveWindow((*this_).m_hWnd, x, y, nWidth, nHeight, bRepaint);
    }
    pub unsafe fn PostMessageW(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return PostMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn ScreenToClient(&mut self, lpPoint: LPPOINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ScreenToClient((*this_).m_hWnd, lpPoint);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
            ASSERT!(IsWindow((*this_).m_hWnd));
            return SendMessage((*this_).m_hWnd, message, wParam, lParam);
        }
    pub unsafe fn SendMessageW__1(hWnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        ATLASSERT!(IsWindow(hWnd));
        return SendMessage(hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetCapture(&mut self) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetCapture((*this_).m_hWnd);
    }
    pub unsafe fn SetMenu(&mut self, hMenu: HMENU) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetMenu((*this_).m_hWnd, hMenu);
    }
    pub unsafe fn SetWindowLongPtr(&mut self, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return SetWindowLongPtr((*this_).m_hWnd, nIndex, (dwNewLong) as _);
    }
    pub unsafe fn SetWindowPlacement(&mut self, lpwndpl: *const WINDOWPLACEMENT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetWindowPlacement((*this_).m_hWnd, lpwndpl);
    }
    pub unsafe fn SetWindowPos(&mut self, hWndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, nFlags: UINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetWindowPos((*this_).m_hWnd, hWndInsertAfter, x, y, cx, cy, nFlags);
    }
    pub unsafe fn SetWindowTextW(&mut self, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetWindowText((*this_).m_hWnd, lpszString);
    }
    pub unsafe fn ShowWindow(&mut self, nCmdShow: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ShowWindow((*this_).m_hWnd, nCmdShow);
    }
    pub unsafe fn UpdateWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return UpdateWindow((*this_).m_hWnd);
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
