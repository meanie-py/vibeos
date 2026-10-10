use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_66A0B53312C1: [WCHAR; 2] = [0x0078, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
}



































impl CTextEditWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_hFont = core::ptr::null_mut();
        (*this_).m_hFontZoomed = core::ptr::null_mut();
        {
            SetRectEmpty(core::ptr::addr_of_mut!((*this_).m_rc));
        }
    }
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        (*this_).m_hwndParent = hwndParent;
    
        let style: DWORD = ES_LEFT | ES_MULTILINE | ES_WANTRETURN | ES_AUTOVSCROLL |
                            WS_CHILD | WS_THICKFRAME;
        let mut hwnd: HWND = CreateWindowEx(0, WC_EDIT, core::ptr::null_mut(), style, 0, 0, 0, 0,
                                     hwndParent, core::ptr::null_mut(), g_hinstExe, core::ptr::null_mut());
        if !hwnd.is_null()
        {
            // #undef SubclassWindow // Don't use this macro
            (*this_).SubclassWindow(hwnd);
    
            (*this_).UpdateFont();
    
            (*this_).__base0.PostMessageW(WM_SIZE, 0, 0);
        }
    
        return (*this_).__base0.m_hWnd;
    }
    pub unsafe fn DoFillBack(&mut self, hwnd: HWND, hDC: HDC) {
        let this_: *mut Self = self;
        if toolsModel.IsBackgroundTransparent() != 0 {
            return;
        }
    
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.SendMessageW__0(EM_GETRECT, 0, core::ptr::addr_of_mut!(rc) as LPARAM);
        (*this_).__base0.MapWindowPoints__0(hwnd, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
    
        let hbr: HBRUSH = CreateSolidBrush(paletteModel.GetBgColor());
        FillRect(hDC, core::ptr::addr_of_mut!(rc), hbr);
        DeleteObject((hbr).cast());
    }
    pub unsafe fn FixEditPos(&mut self, pszOldText: LPCWSTR) {
        let this_: *mut Self = self;
        let mut szText = CStringW::new();
        (*this_).__base0.GetWindowTextW__1(&mut szText);
    
        let mut rcParent: RECT = core::mem::zeroed();
        GetWindowRect((*this_).m_hwndParent, core::ptr::addr_of_mut!(rcParent));
    
        let mut rc = CRect__make_0();
        let mut rcWnd = CRect__make_0();
        let mut rcText = CRect__make_0();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rcWnd.__base0));
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(rcWnd), core::ptr::addr_of_mut!(rcText), 1);
    
        let hDC = (*this_).__base0.GetDC();
        if hDC as usize != 0 {
            SelectObject(hDC, ((*this_).m_hFontZoomed).cast());
            let mut tm: TEXTMETRIC = core::mem::zeroed();
            GetTextMetrics(hDC, core::ptr::addr_of_mut!(tm));
            szText += CPP_COM_WSTR_66A0B53312C1.as_ptr(); // This is a trick to enable the g_ptEnd newlines
            let uFormat: UINT = DT_LEFT | DT_TOP | DT_EDITCONTROL | DT_NOPREFIX | DT_NOCLIP |
                                 DT_EXPANDTABS | DT_WORDBREAK;
            DrawTextW(hDC, szText.GetString(), -1, core::ptr::addr_of_mut!(rcText.__base0), uFormat | DT_CALCRECT);
            if tm.tmDescent > 0 {
                rcText.__base0.bottom += tm.tmDescent;
            }
            (*this_).__base0.ReleaseDC(hDC);
        }
    
        UnionRect(core::ptr::addr_of_mut!(rc.__base0), core::ptr::addr_of_mut!(rcText.__base0), core::ptr::addr_of_mut!(rcWnd.__base0));
        MapWindowPoints(core::ptr::null_mut(), (*this_).m_hwndParent, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
    
        core::ptr::copy_nonoverlapping(core::ptr::addr_of!(rc), core::ptr::addr_of_mut!(rcWnd), 1);
        GetClientRect((*this_).m_hwndParent, core::ptr::addr_of_mut!(rcParent));
        rc.IntersectRect(core::ptr::addr_of!(rcParent), core::ptr::addr_of!(rcWnd.__base0));
    
        (*this_).__base0.MoveWindow(rc.__base0.left, rc.__base0.top, rc.__base0.right - rc.__base0.left, rc.__base0.bottom - rc.__base0.top, (FALSE) as i32);
    
        (*this_).DefWindowProc(WM_HSCROLL, (SB_LEFT) as WPARAM, 0);
        (*this_).DefWindowProc(WM_VSCROLL, (SB_TOP) as WPARAM, 0);
    
        InvalidateRect((*this_).m_hwndParent, core::ptr::addr_of_mut!(rc.__base0), (TRUE) as BOOL);
    }
    pub unsafe fn InvalidateEditRect(&mut self) {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rc));
        ::MapWindowPoints(core::ptr::null_mut(), (*this_).m_hwndParent, core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
        ::InvalidateRect((*this_).m_hwndParent, core::ptr::addr_of_mut!(rc), TRUE);
    
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
        (*this_).__base0.MapWindowPoints__0((canvasWindow).op_to_HWND(), core::ptr::addr_of_mut!(rc) as LPPOINT, 2);
        canvasWindow.CanvasToImage__1(&mut rc, (FALSE) as BOOL);
        (*this_).m_rc = rc;
    }
    pub unsafe fn UpdateFont(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hFont.is_null() {
            DeleteObject(((*this_).m_hFont).cast());
            (*this_).m_hFont = core::ptr::null_mut();
        }
        if !(*this_).m_hFontZoomed.is_null() {
            DeleteObject(((*this_).m_hFontZoomed).cast());
            (*this_).m_hFontZoomed = core::ptr::null_mut();
        }
    
        let mut lf: LOGFONTW = core::mem::zeroed();
        ZeroMemory!(core::ptr::addr_of_mut!(lf) as *mut _, core::mem::size_of::<LOGFONTW>());
        lf.lfCharSet = (DEFAULT_CHARSET as u8); // registrySettings.CharSet; // Ignore
        lf.lfWeight = if registrySettings.Bold != 0 { (FW_BOLD as i32) } else { (FW_NORMAL as i32) };
        lf.lfItalic = registrySettings.Italic as u8;
        lf.lfUnderline = registrySettings.Underline as u8;
        StringCchCopyW(
            core::ptr::addr_of_mut!(lf.lfFaceName) as *mut u16,
            _countof!(core::ptr::addr_of!(lf.lfFaceName)),
            core::ptr::addr_of!(registrySettings.strFontName) as LPCWSTR,
        );
    
        let hdc: HDC = (*this_).__base0.GetDC();
        if !hdc.is_null() {
            let nFontSize: INT = (registrySettings.PointSize) as i32;
            lf.lfHeight = -MulDiv(
                nFontSize,
                GetDeviceCaps(hdc, (LOGPIXELSY as i32)),
                72,
            );
            (*this_).__base0.ReleaseDC(hdc);
        }
    
        (*this_).m_hFont = CreateFontIndirect(core::ptr::addr_of_mut!(lf));
    
        lf.lfHeight = Zoomed(lf.lfHeight);
        (*this_).m_hFontZoomed = CreateFontIndirect(core::ptr::addr_of_mut!(lf));
    
        SetWindowFont!((*this_).__base0.m_hWnd, (*this_).m_hFontZoomed, TRUE);
        (*this_).DefWindowProc(
            EM_SETMARGINS,
            (EC_LEFTMARGIN | EC_RIGHTMARGIN) as WPARAM,
            (MAKELONG!(0, 0)) as LPARAM,
        );
    
        (*this_).FixEditPos(core::ptr::null_mut());
    
        (*this_).__base0.Invalidate((TRUE) as BOOL);
    }
    pub unsafe fn GetEditRect(&mut self, prc: LPRECT) -> BOOL {
        let this_: *mut Self = self;
        *prc = (*this_).m_rc;
        return (TRUE) as BOOL;
    }
    pub unsafe fn ValidateEditRect(&mut self, prc: LPCRECT) {
        let this_: *mut Self = self;
        if !prc.is_null() {
            (*this_).m_rc = *prc;
        }
    
        let mut rc: CRect = CRect { __base0: (*this_).m_rc };
        canvasWindow.ImageToCanvas__1(&mut rc.__base0);
    
        (*this_).__base0.MoveWindow(rc.__base0.left, rc.__base0.top, rc.Width(), rc.Height(), (TRUE) as i32);
    }
    pub unsafe fn OnCreate(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateFont();
        return 0;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        if !(*this_).m_hFont.is_null() {
            DeleteObject(((*this_).m_hFont).cast());
            (*this_).m_hFont = core::ptr::null_mut();
        }
        if !(*this_).m_hFontZoomed.is_null() {
            DeleteObject(((*this_).m_hFontZoomed).cast());
            (*this_).m_hFontZoomed = core::ptr::null_mut();
        }
        return 0;
    }
    pub unsafe fn OnToolsModelToolChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == (TOOL_TEXT) as _ {
            (*this_).UpdateFont();
        } else {
            (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        }
        return 0;
    }
    pub unsafe fn OnToolsModelSettingsChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateFont();
        return 0;
    }
    pub unsafe fn OnToolsModelZoomChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateFont();
        (*this_).ValidateEditRect(core::ptr::null_mut());
        return 0;
    }
    pub unsafe fn OnPaletteModelColorChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).UpdateFont();
        return 0;
    }
    pub unsafe fn OnMoving(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Restrict the window position to the image area
        let prcMoving: LPRECT = lParam as LPRECT;
        let mut rcMoving: CRect = CRect { __base0: *prcMoving };
    
        let mut rcImage: CRect = CRect__make_0();
        canvasWindow.GetImageRect(core::ptr::addr_of_mut!(rcImage.__base0));
        canvasWindow.ImageToCanvas__1(core::ptr::addr_of_mut!(rcImage.__base0));
        canvasWindow.__base0.MapWindowPoints__1(core::ptr::null_mut(), core::ptr::addr_of_mut!(rcImage.__base0));
    
        let mut rcWnd: CRect = CRect__make_0();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rcWnd.__base0));
        let cx: INT = rcWnd.Width();
        let cy: INT = rcWnd.Height();
    
        if rcMoving.__base0.left < rcImage.__base0.left {
            rcMoving.__base0.left = rcImage.__base0.left;
            rcMoving.__base0.right = rcImage.__base0.left + cx;
        } else if rcMoving.__base0.right > rcImage.__base0.right {
            rcMoving.__base0.right = rcImage.__base0.right;
            rcMoving.__base0.left = rcImage.__base0.right - cx;
        }
    
        if rcMoving.__base0.top < rcImage.__base0.top {
            rcMoving.__base0.top = rcImage.__base0.top;
            rcMoving.__base0.bottom = rcImage.__base0.top + cy;
        } else if rcMoving.__base0.bottom > rcImage.__base0.bottom {
            rcMoving.__base0.bottom = rcImage.__base0.bottom;
            rcMoving.__base0.top = rcImage.__base0.bottom - cy;
        }
    
        *prcMoving = rcMoving.__base0;
        (*this_).__base0.Invalidate((TRUE) as BOOL);
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnSizing(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Restrict the window size to the image area
        let prcSizing: LPRECT = lParam as LPRECT;
        let mut rcSizing: CRect = CRect { __base0: *prcSizing };
    
        let mut rcImage: CRect = CRect__make_0();
        canvasWindow.GetImageRect(&mut rcImage.__base0);
        canvasWindow.ImageToCanvas__1(&mut rcImage.__base0);
        canvasWindow.__base0.MapWindowPoints__1(core::ptr::null_mut(), core::ptr::addr_of_mut!(rcImage.__base0));
    
        // Horizontally
        match wParam {
            WMSZ_BOTTOMLEFT | WMSZ_LEFT | WMSZ_TOPLEFT => {
                if rcSizing.__base0.left < rcImage.__base0.left {
                    rcSizing.__base0.left = rcImage.__base0.left;
                }
            }
            WMSZ_BOTTOMRIGHT | WMSZ_RIGHT | WMSZ_TOPRIGHT => {
                if rcSizing.__base0.right > rcImage.__base0.right {
                    rcSizing.__base0.right = rcImage.__base0.right;
                }
            }
            WMSZ_TOP | WMSZ_BOTTOM => {}
            _ => {}
        }
    
        // Vertically
        match wParam {
            WMSZ_BOTTOM | WMSZ_BOTTOMLEFT | WMSZ_BOTTOMRIGHT => {
                if rcSizing.__base0.bottom > rcImage.__base0.bottom {
                    rcSizing.__base0.bottom = rcImage.__base0.bottom;
                }
            }
            WMSZ_TOP | WMSZ_TOPLEFT | WMSZ_TOPRIGHT => {
                if rcSizing.__base0.top < rcImage.__base0.top {
                    rcSizing.__base0.top = rcImage.__base0.top;
                }
            }
            WMSZ_LEFT | WMSZ_RIGHT => {}
            _ => {}
        }
    
        *prcSizing = rcSizing.__base0;
        (*this_).__base0.Invalidate((TRUE) as BOOL);
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnChar(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == (VK_TAB) as _ {
            return 0; // FIXME: Tabs
        }
    
        let mut szText = CStringW::new();
        (*this_).__base0.GetWindowTextW__1(&mut szText);
    
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).FixEditPos(szText.GetString());
    
        return ret;
    }
    pub unsafe fn OnKeyDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == (VK_ESCAPE) as _ {
            toolsModel.OnEndDraw((TRUE) as BOOL);
            return 0;
        }
    
        let mut szText = CStringW::new();
        (*this_).__base0.GetWindowTextW__1(&mut szText);
    
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).FixEditPos(szText.GetString());
        return ret;
    }
    pub unsafe fn OnLButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).DefWindowProc(WM_HSCROLL, (SB_LEFT) as WPARAM, 0);
        (*this_).DefWindowProc(WM_VSCROLL, (SB_TOP) as WPARAM, 0);
        return ret;
    }
    pub unsafe fn OnEraseBkGnd(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let hDC: HDC = wParam as HDC;
        if toolsModel.IsBackgroundTransparent() == 0 {
            let mut rc: RECT = core::mem::zeroed();
            (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
            let hbr: HBRUSH = CreateSolidBrush(paletteModel.GetBgColor());
            FillRect(hDC, core::ptr::addr_of_mut!(rc), hbr);
            DeleteObject((hbr).cast());
        }
        SetTextColor(hDC, paletteModel.GetFgColor());
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
    
        (*this_).DefWindowProc(nMsg, wParam, lParam);
    
        let hDC: HDC = (*this_).__base0.GetDC();
        if !hDC.is_null() {
            (*this_).DrawGrip(hDC, &mut rc);
            (*this_).__base0.ReleaseDC(hDC);
        }
    
        return 0;
    }
    pub unsafe fn OnNCPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rc: CRect = CRect__make_0();
        (*this_).__base0.GetWindowRect(&mut rc.__base0);
    
        let hDC: HDC = (*this_).__base0.GetDCEx(core::ptr::null_mut(), DCX_WINDOW | DCX_PARENTCLIP);
        if !hDC.is_null() {
            rc.OffsetRect__0(-rc.__base0.left, -rc.__base0.top);
            (*this_).DrawGrip(hDC, &mut rc.__base0);
            (*this_).__base0.ReleaseDC(hDC);
        }
    
        return 0;
    }
    pub unsafe fn OnNCCalcSize(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return 0; // No frame.
    }
    pub unsafe fn OnNCHitTest(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT {
            x: GET_X_LPARAM!(lParam),
            y: GET_Y_LPARAM!(lParam),
        };
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rc));
        return (*this_).DoHitTest(&mut rc, pt) as LRESULT;
    }
    pub unsafe fn OnSetCursor(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if CWaitCursor::IsWaiting() != 0
        {
            *bHandled = (FALSE) as i32;
            return 0;
        }
    
        let nHitTest: UINT = (LOWORD!(lParam)) as u32;
        if nHitTest == HTCAPTION
        {
            SetCursor(::LoadCursorW(core::ptr::null_mut(), (IDC_SIZEALL as usize) as LPCWSTR)); // Enable drag move
            return (FALSE) as LRESULT;
        }
        return (*this_).DefWindowProc(nMsg, wParam, lParam);
    }
    pub unsafe fn OnMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        return ret;
    }
    pub unsafe fn OnSize(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
    
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rc));
        (*this_).__base0.SendMessageW__0(EM_SETRECTNP, 0, core::ptr::addr_of_mut!(rc) as LPARAM);
        (*this_).__base0.SendMessageW__0(EM_SETMARGINS, (EC_LEFTMARGIN | EC_RIGHTMARGIN) as WPARAM, (MAKELONG!(0, 0)) as LPARAM);
    
        return ret;
    }
    pub unsafe fn OnSetSel(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).DefWindowProc(WM_HSCROLL, (SB_LEFT) as WPARAM, 0);
        (*this_).DefWindowProc(WM_VSCROLL, (SB_TOP) as WPARAM, 0);
        (*this_).InvalidateEditRect();
        return ret;
    }
    pub unsafe fn OnMouseWheel(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return SendMessageW((*this_).__base0.GetParent().op_to_HWND(), nMsg, wParam, lParam);
    }
    pub unsafe fn OnCut(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).__base0.Invalidate((TRUE) as BOOL); // Redraw
        return ret;
    }
    pub unsafe fn OnPaste(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).FixEditPos(core::ptr::null());
        return ret;
    }
    pub unsafe fn OnClear(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ret: LRESULT = (*this_).DefWindowProc(nMsg, wParam, lParam);
        (*this_).__base0.Invalidate((TRUE) as BOOL); // Redraw
        return ret;
    }
    pub unsafe fn DoHitTest(&mut self, rc: &mut RECT, pt: POINT) -> INT {
        let this_: *mut Self = self;
        match getSizeBoxHitTest(pt, rc) {
            HIT_NONE => { return (HTNOWHERE) as INT; }
            HIT_UPPER_LEFT => { return (HTTOPLEFT) as INT; }
            HIT_UPPER_CENTER => { return (HTTOP) as INT; }
            HIT_UPPER_RIGHT => { return (HTTOPRIGHT) as INT; }
            HIT_MIDDLE_LEFT => { return (HTLEFT) as INT; }
            HIT_MIDDLE_RIGHT => { return (HTRIGHT) as INT; }
            HIT_LOWER_LEFT => { return (HTBOTTOMLEFT) as INT; }
            HIT_LOWER_CENTER => { return (HTBOTTOM) as INT; }
            HIT_LOWER_RIGHT => { return (HTBOTTOMRIGHT) as INT; }
            HIT_BORDER => { return (HTCAPTION) as INT; } // Enable drag move
            HIT_INNER => { return (HTCLIENT) as INT; }
            _ => {}
        }
        return (HTNOWHERE) as INT;
    }
    pub unsafe fn DrawGrip(&mut self, hDC: HDC, rc: &mut RECT) {
        let this_: *mut Self = self;
        drawSizeBoxes(hDC, rc as *mut RECT, (TRUE) as BOOL, core::ptr::null_mut());
    }
}
