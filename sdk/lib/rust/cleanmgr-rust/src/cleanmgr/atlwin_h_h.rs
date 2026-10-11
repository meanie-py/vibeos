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
    pub unsafe fn GetClientRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetClientRect((*this_).m_hWnd, lpRect);
    }
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetWindowTextW__0(&mut self, lpszStringBuf: LPTSTR, nMaxCount: i32) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetWindowText((*this_).m_hWnd, lpszStringBuf, nMaxCount);
    }
    pub unsafe fn MessageBoxW(&mut self, lpszText: LPCTSTR, lpszCaption: LPCTSTR, nType: UINT) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return MessageBox((*this_).m_hWnd, lpszText, lpszCaption, nType);
    }
    pub unsafe fn SendDlgItemMessageW(&mut self, nID: i32, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendDlgItemMessage((*this_).m_hWnd, nID, message, wParam, lParam);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetDlgItemTextW(&mut self, nID: i32, lpszString: LPCTSTR) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetDlgItemText((*this_).m_hWnd, nID, lpszString);
    }
    pub unsafe fn SetWindowTextW(&mut self, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetWindowText((*this_).m_hWnd, lpszString);
    }
    pub unsafe fn ShowWindow(&mut self, nCmdShow: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return ShowWindow((*this_).m_hWnd, nCmdShow);
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
