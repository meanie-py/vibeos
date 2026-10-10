use crate::prelude::*;
use core::ffi::c_void;













unsafe extern "C" fn drawColorBox(hDC: HDC, prc: LPCRECT, hbr: HBRUSH, nBorder: UINT) {
    let mut rc: RECT = *prc;
    FillRect(hDC, core::ptr::addr_of_mut!(rc), (COLOR_3DFACE + 1) as usize as HBRUSH);
    DrawEdge(hDC, core::ptr::addr_of_mut!(rc), nBorder, BF_RECT | BF_ADJUST);
    FillRect(hDC, core::ptr::addr_of_mut!(rc), hbr);
}

unsafe extern "C" fn getColorBoxRect(prc: LPRECT, rcClient: LPCRECT, iColor: i32) {
    let dx: i32 = (iColor % HALF_COLOR_COUNT) * (CXY_COLORBOX as i32); /* delta x */
    let dy: i32 = (iColor / HALF_COLOR_COUNT) * (CXY_COLORBOX as i32); /* delta y */
    (*prc).left = X_COLORBOX_OFFSET + dx;
    (*prc).right = (*prc).left + (CXY_COLORBOX as i32);
    (*prc).top = Y_MARGIN + dy;
    (*prc).bottom = (*prc).top + (CXY_COLORBOX as i32);
}

impl CPaletteWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
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
    pub unsafe fn OnEraseBkgnd(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return (TRUE) as LRESULT; /* Avoid flickering */
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        let mut rcClient: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient));
    
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hDC: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
    
        /* To avoid flickering, we use a memory bitmap.
           The left and top values are zeros in client rectangle */
        let hMemDC: HDC = CreateCompatibleDC(hDC);
        (*this_).m_hbmCached = CachedBufferDIB((*this_).m_hbmCached, rcClient.right, rcClient.bottom);
        let hbmOld: HGDIOBJ = SelectObject(hMemDC, ((*this_).m_hbmCached).cast());
    
        /* Fill the background (since WM_ERASEBKGND handling is disabled) */
        FillRect(hMemDC, core::ptr::addr_of_mut!(rcClient), (COLOR_3DFACE + 1) as usize as HBRUSH);
    
        /* Draw the big box that contains the black box and the white box */
        rc = RECT { left: X_MARGIN, top: Y_MARGIN, right: X_MARGIN + (CXY_BIGBOX as i32), bottom: Y_MARGIN + CXY_BIGBOX };
        DrawEdge(hMemDC, core::ptr::addr_of_mut!(rc), EDGE_SUNKEN as usize as HBRUSH, BF_RECT | BF_ADJUST);
        let rgbLight: COLORREF = GetSysColor((COLOR_3DHIGHLIGHT) as i32);
        let mut y: INT = rc.top;
        while y < rc.bottom {
            let mut bLight: BOOL = y & 1;
            let mut x: INT = rc.left;
            while x < rc.right {
                if bLight != 0 {
                    SetPixelV(hMemDC, x, y, rgbLight);
                }
                bLight = (bLight == 0) as BOOL;
                x += 1;
            }
            y += 1;
        }
    
        /* Draw the white box in the big box, at 5/8 position */
        rc.left = X_MARGIN + ((CXY_BIGBOX * 5 / 8)) as i32 - (CXY_SELECTEDBOX / 2);
        rc.top = Y_MARGIN + (CXY_BIGBOX * 5 / 8) - (CXY_SELECTEDBOX / 2);
        rc.right = rc.left + CXY_SELECTEDBOX;
        rc.bottom = rc.top + CXY_SELECTEDBOX;
        drawColorBox(hMemDC, core::ptr::addr_of_mut!(rc), toolsModel.GetBgBrush(), BDR_RAISEDINNER);
    
        /* Draw the black box (overlapping the white box), at 3/8 position */
        rc.left = X_MARGIN + ((CXY_BIGBOX * 3 / 8)) as i32 - (CXY_SELECTEDBOX / 2);
        rc.top = Y_MARGIN + (CXY_BIGBOX * 3 / 8) - (CXY_SELECTEDBOX / 2);
        rc.right = rc.left + CXY_SELECTEDBOX;
        rc.bottom = rc.top + CXY_SELECTEDBOX;
        drawColorBox(hMemDC, core::ptr::addr_of_mut!(rc), toolsModel.GetFgBrush(), BDR_RAISEDINNER);
    
        /* Draw the normal color boxes */
        let palette: PAL_TYPE = paletteModel.SelectedPalette();
        let mut i: INT = 0;
        while i < COLOR_COUNT {
            getColorBoxRect(core::ptr::addr_of_mut!(rc), core::ptr::addr_of!(rcClient), i);
            let hbr: HBRUSH = toolsModel.CreateBrush(palette, paletteModel.GetColor((i) as u32));
            drawColorBox(hMemDC, core::ptr::addr_of_mut!(rc), hbr, BDR_SUNKENOUTER);
            DeleteObject(hbr);
            i += 1;
        }
    
        /* Transfer bits (hDC <-- hMemDC) */
        BitBlt(hDC, 0, 0, rcClient.right, rcClient.bottom, hMemDC, 0, 0, SRCCOPY);
    
        SelectObject(hMemDC, hbmOld);
        DeleteDC(hMemDC);
        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnLButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let iColor: INT = (*this_).DoHitTest(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam));
        if iColor != -1 {
            paletteModel.SetFgColor(paletteModel.GetColor((iColor) as u32));
        }
        (*this_).__base0.SetCapture();
        return 0;
    }
    pub unsafe fn OnRButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let iColor: INT = (*this_).DoHitTest(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam));
        if iColor != -1 {
            paletteModel.SetBgColor(paletteModel.GetColor((iColor) as u32));
        }
        return 0;
    }
    pub unsafe fn OnLButtonDblClk(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let iColor: INT = (*this_).DoHitTest(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam));
        let mut rgbColor: COLORREF = paletteModel.GetFgColor();
        if iColor != -1 && (mainWindow.ChooseColor(core::ptr::addr_of_mut!(rgbColor)) != 0)
        {
            paletteModel.SetColor((iColor) as u32, rgbColor);
            paletteModel.SetFgColor(rgbColor);
        }
        return 0;
    }
    pub unsafe fn OnRButtonDblClk(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let iColor: INT = (*this_).DoHitTest(GET_X_LPARAM!(lParam), GET_Y_LPARAM!(lParam));
        let mut rgbColor: COLORREF = paletteModel.GetBgColor();
        if iColor != -1 && mainWindow.ChooseColor(core::ptr::addr_of_mut!(rgbColor)) != 0 {
            paletteModel.SetColor((iColor) as u32, rgbColor);
            paletteModel.SetBgColor(rgbColor);
        }
        return 0;
    }
    pub unsafe fn OnMouseMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() != (*this_).__base0.m_hWnd {
            return 0;
        }
    
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
        (*this_).__base0.ClientToScreen__0(core::ptr::addr_of_mut!(pt));
    
        let mut rc: RECT = core::mem::zeroed();
        mainWindow.__base0.GetWindowRect(core::ptr::addr_of_mut!(rc));
    
        let ptCenter: POINT = POINT {
            x: (rc.left + rc.right) / 2,
            y: (rc.top + rc.bottom) / 2,
        };
    
        let dwExpectedBar1ID: DWORD = if pt.y < ptCenter.y { BAR1ID_TOP } else { BAR1ID_BOTTOM };
    
        if registrySettings.Bar1ID != dwExpectedBar1ID {
            registrySettings.Bar1ID = dwExpectedBar1ID;
            mainWindow.__base0.PostMessageW(WM_SIZE, 0, 0);
        }
    
        return 0;
    }
    pub unsafe fn OnLButtonUp(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if GetCapture() != (*this_).__base0.m_hWnd {
            return 0;
        }
    
        ReleaseCapture();
        return 0;
    }
    pub unsafe fn OnPaletteModelColorChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.Invalidate((FALSE) as i32);
        return 0;
    }
    pub unsafe fn DoHitTest(&mut self, xPos: INT, yPos: INT) -> INT {
        let this_: *mut Self = self;
        let mut rcClient: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient));
    
        /* delta x and y */
        let dx: INT = xPos - X_COLORBOX_OFFSET;
        let dy: INT = yPos - Y_MARGIN;
    
        /* horizontal and vertical indexes */
        let ix: INT = dx / (CXY_COLORBOX as i32);
        let iy: INT = dy / (CXY_COLORBOX as i32);
    
        /* Is it inside of a color box? */
        if 0 <= ix && ix < HALF_COLOR_COUNT && 0 <= iy && iy < 2 {
            return ix + (iy * HALF_COLOR_COUNT); /* return the color index */
        }
    
        return -1; /* Not found */
    }
}
