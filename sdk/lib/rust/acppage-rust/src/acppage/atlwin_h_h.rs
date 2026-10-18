use crate::prelude::*;
use core::ffi::c_void;

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
        if hWndCenter == core::ptr::null_mut() {
            hWndCenter = GetParent((*this_).m_hWnd);
        }
        if hWndCenter == core::ptr::null_mut() {
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
    pub unsafe fn CheckDlgButton(&mut self, nIDButton: i32, nCheck: UINT) -> BOOL {
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
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetParent(&mut self) -> CWindow {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CWindow__make_0(GetParent((*this_).m_hWnd));
    }
    pub unsafe fn IsDlgButtonChecked(&mut self, nIDButton: i32) -> UINT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsDlgButtonChecked((*this_).m_hWnd, nIDButton);
    }
    pub unsafe fn SetDlgItemTextW(&mut self, nID: i32, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetDlgItemText((*this_).m_hWnd, nID, lpszString);
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
