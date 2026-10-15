use crate::prelude::*;
use core::ffi::c_void;







































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
    pub unsafe fn BeginPaint(&mut self, lpPaint: LPPAINTSTRUCT) -> HDC {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return BeginPaint((*this_).m_hWnd, lpPaint);
    }
    pub unsafe fn Create(&mut self, lpstrWndClass: LPCTSTR, hWndParent: HWND, rect: ATL__U_RECT, szWindowName: LPCTSTR, dwStyle: DWORD, dwExStyle: DWORD, MenuOrID: ATL__U_MENUorID, lpCreateParam: LPVOID) -> HWND {
        let this_: *mut Self = self;
        let mut hWnd: HWND;
        ATLASSERT!((*this_).m_hWnd == core::ptr::null_mut());
        hWnd = CreateWindowEx(dwExStyle,
                                lpstrWndClass,
                                szWindowName,
                                dwStyle,
                                (*rect.m_lpRect).left,
                                (*rect.m_lpRect).top,
                                (*rect.m_lpRect).right - (*rect.m_lpRect).left,
                                (*rect.m_lpRect).bottom - (*rect.m_lpRect).top,
                                hWndParent,
                                MenuOrID.m_hMenu,
                                _AtlBaseModule.GetModuleInstance(),
                                lpCreateParam);
        if hWnd != core::ptr::null_mut() {
            (*this_).m_hWnd = hWnd;
        }
        return hWnd;
    }
    pub unsafe fn DeferWindowPos(&mut self, hWinPosInfo: HDWP, hWndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, uFlags: UINT) -> HDWP {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return DeferWindowPos(hWinPosInfo, (*this_).m_hWnd, hWndInsertAfter, x, y, cx, cy, uFlags);
    }
    pub unsafe fn DestroyWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));

            if !DestroyWindow((*this_).m_hWnd) != 0 {
                return (FALSE) as BOOL;
            }

            (*this_).m_hWnd = core::ptr::null_mut();
            return (TRUE) as BOOL;
        }
    pub unsafe fn Detach(&mut self) -> HWND {
        let this_: *mut Self = self;
        let hWnd: HWND = (*this_).m_hWnd;
        (*this_).m_hWnd = core::ptr::null_mut();
        return hWnd;
    }
    pub unsafe fn EndPaint(&mut self, lpPaint: LPPAINTSTRUCT) {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        EndPaint((*this_).m_hWnd, lpPaint);
    }
    pub unsafe fn GetClientRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetClientRect((*this_).m_hWnd, lpRect);
    }
    pub unsafe fn GetDC(&mut self) -> HDC {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetDC((*this_).m_hWnd);
    }
    pub unsafe fn GetMenu(&mut self) -> HMENU {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetMenu((*this_).m_hWnd);
    }
    pub unsafe fn GetParent(&mut self) -> CWindow {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CWindow__make_0(GetParent((*this_).m_hWnd));
    }
    pub unsafe fn GetWindowLongPtr(&mut self, nIndex: i32) -> LONG_PTR {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowLongPtr((*this_).m_hWnd, nIndex);
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
    pub unsafe fn GetWindowTextLengthW(&mut self) -> i32 {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowTextLength((*this_).m_hWnd);
    }
    pub unsafe fn InvalidateRect(&mut self, lpRect: LPCRECT, bErase: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return InvalidateRect((*this_).m_hWnd, lpRect, bErase);
    }
    pub unsafe fn IsWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return IsWindow((*this_).m_hWnd);
    }
    pub unsafe fn IsWindowVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsWindowVisible((*this_).m_hWnd);
    }
    pub unsafe fn KillTimer(&mut self, nIDEvent: UINT_PTR) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return KillTimer((*this_).m_hWnd, nIDEvent);
    }
    pub unsafe fn MessageBoxW(&mut self, lpszText: LPCTSTR, lpszCaption: LPCTSTR, nType: UINT) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return MessageBox((*this_).m_hWnd, lpszText, lpszCaption, nType);
    }
    pub unsafe fn ModifyStyle(&mut self, dwRemove: DWORD, dwAdd: DWORD, nFlags: UINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        SetWindowLong((*this_).m_hWnd, GWL_STYLE, (((GetWindowLong((*this_).m_hWnd, GWL_STYLE) as DWORD) & !dwRemove) | dwAdd) as i32);
        if nFlags != 0 {
            return SetWindowPos((*this_).m_hWnd, core::ptr::null_mut(), 0, 0, 0, 0, nFlags | SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
        return (TRUE) as BOOL;
    }
    pub unsafe fn PostMessageW(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return PostMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn RedrawWindow(&mut self, lpRectUpdate: LPCRECT, hRgnUpdate: HRGN, flags: UINT) -> BOOL {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return RedrawWindow((*this_).m_hWnd, lpRectUpdate, hRgnUpdate, flags);
        }
    pub unsafe fn ReleaseDC(&mut self, hDC: HDC) -> i32 {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return ReleaseDC((*this_).m_hWnd, hDC);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetCapture(&mut self) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetCapture((*this_).m_hWnd);
    }
    pub unsafe fn SetFocus(&mut self) -> HWND {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return SetFocus((*this_).m_hWnd);
    }
    pub unsafe fn SetRedraw(&mut self, bRedraw: BOOL) {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        SendMessage((*this_).m_hWnd, WM_SETREDRAW, bRedraw as WPARAM, 0);
    }
    pub unsafe fn SetTimer(&mut self, nIDEvent: UINT_PTR, nElapse: UINT, lpfnTimer: Option<unsafe extern "C" fn(HWND, UINT, UINT_PTR, DWORD)>) -> UINT_PTR {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return SetTimer((*this_).m_hWnd, nIDEvent, nElapse, lpfnTimer as TIMERPROC);
        }
    pub unsafe fn SetWindowPos(&mut self, hWndInsertAfter: HWND, x: i32, y: i32, cx: i32, cy: i32, nFlags: UINT) -> BOOL {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return SetWindowPos((*this_).m_hWnd, hWndInsertAfter, x, y, cx, cy, nFlags);
        }
    pub unsafe fn SetWindowTextW(&mut self, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetWindowText((*this_).m_hWnd, lpszString);
    }
    pub unsafe fn ShowWindow(&mut self, nCmdShow: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
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
