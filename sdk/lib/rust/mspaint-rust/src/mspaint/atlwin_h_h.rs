use crate::prelude::*;
use core::ffi::c_void;

impl CWindow {
    pub unsafe fn __ctor_0(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        {
            (*this_).m_hWnd = hWnd;
        }
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
    pub unsafe fn CheckDlgButton(&mut self, nIDButton: i32, nCheck: UINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CheckDlgButton((*this_).m_hWnd, nIDButton, nCheck);
    }
    pub unsafe fn CheckRadioButton(&mut self, nIDFirstButton: i32, nIDLastButton: i32, nIDCheckButton: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return CheckRadioButton((*this_).m_hWnd, nIDFirstButton, nIDLastButton, nIDCheckButton);
    }
    pub unsafe fn ClientToScreen__0(&mut self, lpPoint: LPPOINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return ClientToScreen((*this_).m_hWnd, lpPoint);
    }
    pub unsafe fn Create(&mut self, lpstrWndClass: LPCTSTR, hWndParent: HWND, rect: _U_RECT, szWindowName: LPCTSTR, dwStyle: DWORD, dwExStyle: DWORD, MenuOrID: _U_MENUorID, lpCreateParam: LPVOID) -> HWND {
        let this_: *mut Self = self;
        let mut hWnd: HWND;
        ATLASSERT!((*this_).m_hWnd == core::ptr::null_mut());
        hWnd = CreateWindowEx(
            dwExStyle,
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
            lpCreateParam,
        );
        if hWnd != core::ptr::null_mut() {
            (*this_).m_hWnd = hWnd;
        }
        return hWnd;
    }
    pub unsafe fn DestroyWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
    
        if DestroyWindow((*this_).m_hWnd) == 0 {
            return (FALSE) as BOOL;
        }
    
        (*this_).m_hWnd = core::ptr::null_mut();
        return (TRUE) as BOOL;
    }
    pub unsafe fn Detach(&mut self) -> HWND {
        let this_: *mut Self = self;
            let mut hWnd: HWND = (*this_).m_hWnd;
            (*this_).m_hWnd = core::ptr::null_mut();
            return hWnd;
        }
    pub unsafe fn EndPaint(&mut self, lpPaint: LPPAINTSTRUCT) {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
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
    pub unsafe fn GetDCEx(&mut self, hRgnClip: HRGN, flags: DWORD) -> HDC {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hWnd != 0 && IsWindow((*this_).m_hWnd) != 0);
        return GetDCEx((*this_).m_hWnd, hRgnClip, flags);
    }
    pub unsafe fn GetDlgItem(&mut self, nID: i32) -> HWND {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetDlgItem((*this_).m_hWnd, nID);
    }
    pub unsafe fn GetDlgItemInt(&mut self, nID: i32, lpTrans: *mut BOOL, bSigned: BOOL) -> UINT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetDlgItemInt((*this_).m_hWnd, nID, lpTrans, bSigned);
    }
    pub unsafe fn GetDlgItemTextW__0(&mut self, nID: i32, lpStr: LPTSTR, nMaxCount: i32) -> UINT {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetDlgItemText((*this_).m_hWnd, nID, lpStr, nMaxCount);
    }
    pub unsafe fn GetDlgItemTextW__1(&mut self, nID: i32, strText: &mut CSimpleString) -> UINT {
        let this_: *mut Self = self;
        let item: HWND = (*this_).GetDlgItem(nID);
        if item.is_null() {
            strText.Empty();
            return 0;
        }
        return ((CWindow__make_0(item)).GetWindowTextW__1(strText)) as UINT;
    }
    pub unsafe fn GetFont(&mut self) -> HFONT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendMessage((*this_).m_hWnd, WM_GETFONT as UINT, 0 as WPARAM, 0 as LPARAM) as HFONT;
    }
    pub unsafe fn GetMenu(&mut self) -> HMENU {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return GetMenu((*this_).m_hWnd);
    }
    pub unsafe fn GetParent(&mut self) -> CWindow {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return CWindow__make_0(GetParent((*this_).m_hWnd));
    }
    pub unsafe fn GetScrollInfo(&mut self, nBar: i32, lpScrollInfo: LPSCROLLINFO) -> BOOL {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return GetScrollInfo((*this_).m_hWnd, nBar, lpScrollInfo);
        }
    pub unsafe fn GetScrollPos(&mut self, nBar: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetScrollPos((*this_).m_hWnd, nBar);
    }
    pub unsafe fn GetWindowPlacement(&mut self, lpwndpl: *mut WINDOWPLACEMENT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return GetWindowPlacement((*this_).m_hWnd, lpwndpl);
    }
    pub unsafe fn GetWindowRect(&mut self, lpRect: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowRect((*this_).m_hWnd, lpRect);
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
        strText.ReleaseBuffer(len);
        return len;
    }
    pub unsafe fn GetWindowTextLengthW(&mut self) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return GetWindowTextLength((*this_).m_hWnd);
    }
    pub unsafe fn Invalidate(&mut self, bErase: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return InvalidateRect((*this_).m_hWnd, core::ptr::null_mut(), bErase);
    }
    pub unsafe fn IsDialogMessageW(&mut self, lpMsg: LPMSG) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsDialogMessage((*this_).m_hWnd, lpMsg);
    }
    pub unsafe fn IsDlgButtonChecked(&mut self, nIDButton: i32) -> UINT {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return IsDlgButtonChecked((*this_).m_hWnd, nIDButton);
        }
    pub unsafe fn IsIconic(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsIconic((*this_).m_hWnd);
    }
    pub unsafe fn IsWindow(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return IsWindow((*this_).m_hWnd);
    }
    pub unsafe fn IsWindowVisible(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return IsWindowVisible((*this_).m_hWnd);
    }
    pub unsafe fn IsZoomed(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return IsZoomed((*this_).m_hWnd);
    }
    pub unsafe fn MapWindowPoints__0(&mut self, hWndTo: HWND, lpPoint: LPPOINT, nCount: UINT) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return MapWindowPoints((*this_).m_hWnd, hWndTo, lpPoint, nCount);
    }
    pub unsafe fn MapWindowPoints__1(&mut self, hWndTo: HWND, lpRect: LPRECT) -> i32 {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return MapWindowPoints((*this_).m_hWnd, hWndTo, lpRect as LPPOINT, (core::mem::size_of::<RECT>() / (core::mem::size_of::<POINT>() as usize)) as UINT);
    }
    pub unsafe fn MessageBoxW(&mut self, lpszText: LPCTSTR, lpszCaption: LPCTSTR, nType: UINT) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return MessageBox((*this_).m_hWnd, lpszText, lpszCaption, nType);
    }
    pub unsafe fn MoveWindow(&mut self, x: i32, y: i32, nWidth: i32, nHeight: i32, bRepaint: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return MoveWindow((*this_).m_hWnd, x, y, nWidth, nHeight, bRepaint);
    }
    pub unsafe fn OpenClipboard(&mut self) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return OpenClipboard((*this_).m_hWnd);
    }
    pub unsafe fn PostMessageW(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return PostMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn ReleaseDC(&mut self, hDC: HDC) -> i32 {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return ReleaseDC((*this_).m_hWnd, hDC);
    }
    pub unsafe fn ScreenToClient(&mut self, lpPoint: LPPOINT) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return ScreenToClient((*this_).m_hWnd, lpPoint);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SendMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetCapture(&mut self) -> HWND {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetCapture((*this_).m_hWnd);
    }
    pub unsafe fn SetDlgItemInt(&mut self, nID: i32, nValue: UINT, bSigned: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetDlgItemInt((*this_).m_hWnd, nID, nValue, bSigned);
    }
    pub unsafe fn SetDlgItemTextW(&mut self, nID: i32, lpszString: LPCTSTR) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetDlgItemText((*this_).m_hWnd, nID, lpszString);
    }
    pub unsafe fn SetFocus(&mut self) -> HWND {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return SetFocus((*this_).m_hWnd);
    }
    pub unsafe fn SetMenu(&mut self, hMenu: HMENU) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return SetMenu((*this_).m_hWnd, hMenu);
    }
    pub unsafe fn SetScrollInfo(&mut self, nBar: i32, lpScrollInfo: LPSCROLLINFO, bRedraw: BOOL) -> i32 {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return SetScrollInfo((*this_).m_hWnd, nBar, lpScrollInfo, bRedraw);
        }
    pub unsafe fn SetScrollPos(&mut self, nBar: i32, nPos: i32, bRedraw: BOOL) -> i32 {
        let this_: *mut Self = self;
            ATLASSERT!(IsWindow((*this_).m_hWnd));
            return SetScrollPos((*this_).m_hWnd, nBar, nPos, bRedraw);
        }
    pub unsafe fn SetWindowLongPtr(&mut self, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return SetWindowLongPtr((*this_).m_hWnd, nIndex, dwNewLong);
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
    pub unsafe fn ShowScrollBar(&mut self, nBar: UINT, bShow: BOOL) -> BOOL {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd));
        return ShowScrollBar((*this_).m_hWnd, (nBar) as i32, bShow);
    }
    pub unsafe fn ShowWindow(&mut self, nCmdShow: i32) -> BOOL {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return ShowWindow((*this_).m_hWnd, nCmdShow);
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
