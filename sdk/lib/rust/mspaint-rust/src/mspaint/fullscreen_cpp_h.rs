use crate::prelude::*;
use core::ffi::c_void;






impl CFullscreenWindow {
    pub unsafe fn DoCreate(&mut self) -> HWND {
        let this_: *mut Self = self;
        if !(*this_).__base0.m_hWnd.is_null() {
            return (*this_).__base0.m_hWnd;
        }
    
        let rc = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        }; // Rely on SW_SHOWMAXIMIZED
        return (*this_).Create(
            HWND_DESKTOP,
            rc,
            core::ptr::null_mut(),
            WS_POPUPWINDOW,
            WS_EX_TOPMOST,
        );
    }
    pub unsafe fn OnCreate(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_BIG) as UINT, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
        (*this_).__base0.SendMessageW__0(WM_SETICON, (ICON_SMALL) as UINT, LoadIconW(g_hinstExe, MAKEINTRESOURCEW!(IDI_APPICON)) as LPARAM);
        return 0;
    }
    pub unsafe fn OnCloseOrKeyDownOrLButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        mainWindow.__base0.ShowWindow((SW_SHOW) as i32);
        (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        return 0;
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hDC: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
        let mut rcWnd: RECT = core::mem::zeroed();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rcWnd));
        let cxDest: INT = imageModel.GetWidth();
        let cyDest: INT = imageModel.GetHeight();
        let xDest: INT = (rcWnd.right - rcWnd.left - cxDest) / 2;
        let yDest: INT = (rcWnd.bottom - rcWnd.top - cyDest) / 2;
        BitBlt(hDC, xDest, yDest, cxDest, cyDest, imageModel.GetDC(), 0, 0, SRCCOPY);
        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnGetText(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // return caption of the main window, instead
        return mainWindow.__base0.SendMessageW__0(nMsg, wParam, lParam);
    }
}
