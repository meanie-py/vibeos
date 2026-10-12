use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn IsWindow(hWnd: HWND) -> i32;
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
    pub unsafe fn DeferWindowPos(&mut self, hWinPosInfo: HDWP, hWndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, uFlags: u32) -> HDWP {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return DeferWindowPos(hWinPosInfo, (*this_).m_hWnd, hWndInsertAfter, x, y, cx, cy, uFlags);
    }
    pub unsafe fn GetClientRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetClientRect((*this_).m_hWnd, lpRect);
    }
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetWindowLongPtr(&mut self, nIndex: i32) -> isize {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return (GetWindowLongPtr((*this_).m_hWnd, nIndex)) as _;
    }
    pub unsafe fn ScreenToClient(&mut self, lpPoint: LPPOINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ScreenToClient((*this_).m_hWnd, lpPoint);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SendMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetCapture(&mut self) -> HWND {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetCapture((*this_).m_hWnd);
    }
    pub unsafe fn SetIcon(&mut self, hIcon: HICON, bBigIcon: BOOL) -> HICON {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SendMessage((*this_).m_hWnd, WM_SETICON, bBigIcon as WPARAM, hIcon as LPARAM) as HICON;
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
