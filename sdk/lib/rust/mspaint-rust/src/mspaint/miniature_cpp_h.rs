use crate::prelude::*;
use core::ffi::c_void;










impl CMiniatureWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_hbmCached = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hbmCached.is_null() {
            DeleteObject(((*this_).m_hbmCached).cast());
        }
    }
    pub unsafe fn DoCreate(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        if !(*this_).__base0.m_hWnd.is_null() {
            return (*this_).__base0.m_hWnd;
        }
    
        let rc: RECT = RECT {
            left: registrySettings.ThumbXPos as i32,
            top: registrySettings.ThumbYPos as i32,
            right: (registrySettings.ThumbXPos + registrySettings.ThumbWidth) as i32,
            bottom: (registrySettings.ThumbYPos + registrySettings.ThumbHeight) as i32,
        };
    
        let mut strTitle: [WCHAR; 100] = [0 as WCHAR; 100];
        LoadStringW(g_hinstExe, IDS_MINIATURETITLE, strTitle.as_mut_ptr(), (_countof!(strTitle)) as i32);
    
        let style: u32 = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME;
        return (*this_).Create(hwndParent, &mut rc, strTitle.as_mut_ptr(), style, WS_EX_PALETTEWINDOW);
    }
    pub unsafe fn OnMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsWindowVisible() != 0 && (*this_).__base0.IsIconic() == 0 && (*this_).__base0.IsZoomed() == 0 {
            let mut rc: CRect = CRect__make_0();
            (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rc.__base0));
            registrySettings.ThumbXPos = (rc.__base0.left) as u32;
            registrySettings.ThumbYPos = (rc.__base0.top) as u32;
        }
        return 0;
    }
    pub unsafe fn OnSize(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsWindowVisible() != 0 && !((*this_).__base0.IsIconic() != 0) && !((*this_).__base0.IsZoomed() != 0) {
            let mut rc: CRect = CRect__make_0();
            (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rc.__base0));
            registrySettings.ThumbWidth = (rc.Width()) as u32;
            registrySettings.ThumbHeight = (rc.Height()) as u32;
        }
        return 0;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        registrySettings.ShowThumbnail = FALSE;
        return 0;
    }
    pub unsafe fn OnEraseBkgnd(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return (TRUE) as LRESULT; /* Avoid flickering */
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
    
        // Start painting
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hDC: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
        if hDC.is_null() {
            return 0;
        }
    
        // Use a memory bitmap to reduce flickering
        let hdcMem: HDC = CreateCompatibleDC(hDC);
        (*this_).m_hbmCached = CachedBufferDIB((*this_).m_hbmCached, rc.right, rc.bottom);
        let hbmOld: HGDIOBJ = SelectObject(hdcMem, ((*this_).m_hbmCached).cast());
    
        // FIXME: Consider aspect ratio
    
        // Fill the background
        FillRect(hdcMem, core::ptr::addr_of_mut!(rc), (COLOR_BTNFACE + 1) as HBRUSH);
    
        // Draw the image (hdcMem <-- imageModel)
        let cxImage: i32 = imageModel.GetWidth();
        let cyImage: i32 = imageModel.GetHeight();
        StretchBlt(hdcMem, 0, 0, rc.right, rc.bottom,
                     imageModel.GetDC(), 0, 0, cxImage, cyImage,
                     SRCCOPY);
    
        // Move the image (hDC <-- hdcMem)
        BitBlt(hDC, 0, 0, rc.right, rc.bottom, hdcMem, 0, 0, SRCCOPY);
    
        // Clean up
        SelectObject(hdcMem, hbmOld);
        DeleteDC(hdcMem);
    
        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnGetMinMaxInfo(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let pInfo: LPMINMAXINFO = lParam as LPMINMAXINFO;
        (*pInfo).ptMinTrackSize = POINT { x: 100, y: 75 };
        return 0;
    }
}
