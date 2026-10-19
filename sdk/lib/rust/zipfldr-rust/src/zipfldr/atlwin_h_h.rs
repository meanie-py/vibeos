use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    // winuser.h: SetWindowLongPtrW is a #define onto SetWindowLongW on i386.
    #[cfg(target_arch = "x86")]
    fn SetWindowLongW(hWnd: HWND, nIndex: i32, dwNewLong: LONG) -> LONG;
    #[cfg(not(target_arch = "x86"))]
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR;
}

extern "system" {
    fn IsWindow(hWnd: HWND) -> BOOL;
}



























impl CWindow {
    pub unsafe fn __ctor_0(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        {
            (*this_).m_hWnd = hWnd;
        }
    }
    pub unsafe fn op_to_HWND(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).m_hWnd;
    }
    pub unsafe fn Attach(&mut self, hWndNew: HWND) {
        let this_: *mut Self = self;
        (*this_).m_hWnd = hWndNew;
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

        if (GetWindowLong(hWndCenter, GWL_STYLE) & (WS_CHILD as i32)) == 0
        {
            let mut mi: MONITORINFO = core::mem::zeroed();
            mi.cbSize = core::mem::size_of::<MONITORINFO>() as u32;
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
            (TRUE) as BOOL,
        );
    }
    pub unsafe fn CheckDlgButton(&mut self, nIDButton: i32, nCheck: UINT) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CheckDlgButton((*this_).m_hWnd, nIDButton, nCheck);
    }
    pub unsafe fn Detach(&mut self) -> HWND {
        let this_: *mut Self = self;
            let hWnd: HWND = (*this_).m_hWnd;
            (*this_).m_hWnd = core::ptr::null_mut();
            return hWnd;
        }
    pub unsafe fn GetDC(&mut self) -> HDC {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetDC((*this_).m_hWnd);
    }
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetDlgItemTextW__1(&mut self, nID: i32, strText: &mut CSimpleString) -> UINT {
        let this_: *mut Self = self;
        let item: HWND = (*this_).GetDlgItem(nID);
        if item.is_null() {
            strText.Empty();
            return 0;
        }
        return (CWindow__make_0(item).GetWindowTextW__1(strText)) as UINT;
    }
    pub unsafe fn GetParent(&mut self) -> CWindow {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CWindow__make_0(GetParent((*this_).m_hWnd));
    }
    pub unsafe fn GetWindowTextW__0(&mut self, lpszStringBuf: LPTSTR, nMaxCount: i32) -> i32 {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
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
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowTextLength((*this_).m_hWnd);
    }
    pub unsafe fn IsDlgButtonChecked(&mut self, nIDButton: i32) -> UINT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsDlgButtonChecked((*this_).m_hWnd, nIDButton);
    }
    pub unsafe fn ReleaseDC(&mut self, hDC: HDC) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ReleaseDC((*this_).m_hWnd, hDC);
    }
    pub unsafe fn SendDlgItemMessageW(&mut self, nID: i32, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendDlgItemMessage((*this_).m_hWnd, nID, message, wParam, lParam);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
            ASSERT!(IsWindow((*this_).m_hWnd));
            return SendMessage((*this_).m_hWnd, message, wParam, lParam);
        }
    pub unsafe fn SetDlgItemTextW(&mut self, nID: i32, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetDlgItemText((*this_).m_hWnd, nID, lpszString);
    }
    pub unsafe fn SetIcon(&mut self, hIcon: HICON, bBigIcon: BOOL) -> HICON {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SendMessage((*this_).m_hWnd, WM_SETICON, bBigIcon as WPARAM, hIcon as LPARAM) as HICON;
    }
    pub unsafe fn SetWindowLongPtr(&mut self, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        // winuser.h: the Ptr form is a #define onto SetWindowLongW on i386.
        #[cfg(target_arch = "x86")]
        return SetWindowLongW((*this_).m_hWnd, nIndex, dwNewLong as LONG) as LONG_PTR;
        #[cfg(not(target_arch = "x86"))]
        return SetWindowLongPtrW((*this_).m_hWnd, nIndex, dwNewLong);
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
