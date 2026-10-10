use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_16A2910FBF5B: [WCHAR; 9] = [0x0025, 0x006C, 0x0064, 0x002C, 0x0020, 0x0025, 0x006C, 0x0064, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9768A8408F0A: [WCHAR; 8] = [0x0025, 0x0064, 0x0020, 0x0078, 0x0020, 0x0025, 0x0064, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
}


































impl CStyledCursor {
    pub unsafe fn SetStyle(&mut self, style: BrushStyle, zoom: INT, radius: INT, color: COLORREF, is_rubber: BOOL) {
        let this_: *mut Self = self;
        if !(*this_).m_hCursor.is_null() &&
            (*this_).m_style == style &&
            (*this_).m_zoom == zoom &&
            (*this_).m_radius == radius &&
            (*this_).m_color == color &&
            (*this_).m_is_rubber == is_rubber
        {
            return;
        }
    
        if !(*this_).m_hCursor.is_null() {
            DestroyCursor((*this_).m_hCursor);
        }
    
        (*this_).m_hCursor = CStyledCursor::CreateStyledCursor(style, zoom, radius, color, is_rubber);
        (*this_).m_style = style;
        (*this_).m_zoom = zoom;
        (*this_).m_radius = radius;
        (*this_).m_color = color;
        (*this_).m_is_rubber = is_rubber;
    }
    pub unsafe fn CreateStyledCursor(style: BrushStyle, zoom: INT, radius: INT, color: COLORREF, is_rubber: BOOL) -> HCURSOR {
        let diameter: INT = 2 * radius;
        if diameter * zoom / (DEFAULT_ZOOM as i32) <= 2 {
            let hCursor: HCURSOR = LoadCursor(core::ptr::null_mut(), IDC_CROSS);
            return if !hCursor.is_null() { CopyCursor!(hCursor) } else { core::ptr::null_mut() };
        }
    
        let crosshair1: INT = 6;
        let crosshair2: INT = crosshair1 - 2;
        let width: INT = diameter + 2 * crosshair1;
        let height: INT = diameter + 2 * crosshair1;
        let mut hotX: DWORD = (width / 2) as DWORD;
        let mut hotY: DWORD = (height / 2) as DWORD;
    
        let hdcScreen: HDC = GetDC(core::ptr::null_mut());
        if hdcScreen.is_null() {
            return core::ptr::null_mut();
        }
        let hdcMem: HDC = CreateCompatibleDC(hdcScreen);
        if hdcMem.is_null() {
            ReleaseDC(core::ptr::null_mut(), hdcScreen);
            return core::ptr::null_mut();
        }
    
        let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };
    
        // Create the AND mask bitmap. This must be monochrome (1bpp):
        // white bits are transparent, black bits are opaque.
        let mut hbmMask: HBITMAP = CreateBitmap(width, height, 1, 1, core::ptr::null_mut());
        if !hbmMask.is_null() {
            let hbmOld: HBITMAP = SelectObject(hdcMem, (hbmMask).cast()) as HBITMAP;
    
            // Fill with white brush
            FillRect(hdcMem, core::ptr::addr_of_mut!(rc), GetStockObject((WHITE_BRUSH as i32)) as HBRUSH);
    
            if is_rubber == 0 {
                // Draw crosshair with white pen
                SelectObject(hdcMem, (GetStockObject((WHITE_PEN as i32)) as HPEN).cast());
                MoveToEx(hdcMem, 0, hotY as INT, core::ptr::null_mut());
                LineTo(hdcMem, crosshair2, hotY as INT);
                MoveToEx(hdcMem, width - crosshair2, hotY as INT, core::ptr::null_mut());
                LineTo(hdcMem, width, hotY as INT);
                MoveToEx(hdcMem, hotX as INT, 0, core::ptr::null_mut());
                LineTo(hdcMem, hotX as INT, crosshair2);
                MoveToEx(hdcMem, hotX as INT, height - crosshair2, core::ptr::null_mut());
                LineTo(hdcMem, hotX as INT, height);
            }
    
            // Draw brush or erase with black color
            if is_rubber != 0 {
                Erase(hdcMem, hotX as INT, hotY as INT, hotX as INT, hotY as INT, RGB!(0, 0, 0), radius + 1);
            } else {
                Brush(hdcMem, hotX as INT, hotY as INT, hotX as INT, hotY as INT, RGB!(0, 0, 0), style, diameter);
            }
    
            if is_rubber != 0 {
                InflateRect(core::ptr::addr_of_mut!(rc), -1, -1);
            }
    
            SelectObject(hdcMem, (hbmOld).cast());
        }
    
        // Create the color (XOR) bitmap
        let mut hbmColor: HBITMAP = CreateCompatibleBitmap(hdcScreen, width, height);
        if !hbmColor.is_null() {
            let hbmOld: HBITMAP = SelectObject(hdcMem, (hbmColor).cast()) as HBITMAP;
    
            // Fill with black brush
            FillRect(hdcMem, core::ptr::addr_of_mut!(rc), GetStockObject((BLACK_BRUSH as i32)) as HBRUSH);
    
            if is_rubber != 0 {
                // Draw for rubber border
                let avg: INT = ((GetRValue!(color) as INT) + (GetGValue!(color) as INT) + (GetBValue!(color) as INT)) / 3;
                let color2: COLORREF = if avg > 255 / 2 {
                    RGB!(0, 0, 0)
                } else {
                    RGB!(255, 255, 255)
                };
                Erase(hdcMem, hotX as INT, hotY as INT, hotX as INT, hotY as INT, color2, radius + 1);
            } else {
                // Draw crosshair with white pen
                SelectObject(hdcMem, (GetStockObject((WHITE_PEN as i32)) as HPEN).cast());
                MoveToEx(hdcMem, 0, hotY as INT, core::ptr::null_mut());
                LineTo(hdcMem, crosshair2, hotY as INT);
                MoveToEx(hdcMem, width - crosshair2, hotY as INT, core::ptr::null_mut());
                LineTo(hdcMem, width, hotY as INT);
                MoveToEx(hdcMem, hotX as INT, 0, core::ptr::null_mut());
                LineTo(hdcMem, hotX as INT, crosshair2);
                MoveToEx(hdcMem, hotX as INT, height - crosshair2, core::ptr::null_mut());
                LineTo(hdcMem, hotX as INT, height);
            }
    
            // Draw brush or erase with color
            if is_rubber != 0 {
                Erase(hdcMem, hotX as INT, hotY as INT, hotX as INT, hotY as INT, color, radius);
            } else {
                Brush(hdcMem, hotX as INT, hotY as INT, hotX as INT, hotY as INT, color, style, diameter);
            }
    
            SelectObject(hdcMem, (hbmOld).cast());
        }
    
        ReleaseDC(core::ptr::null_mut(), hdcScreen);
        DeleteDC(hdcMem);
    
        if zoom != (DEFAULT_ZOOM as i32) {
            let newWidth: INT = width * zoom / (DEFAULT_ZOOM as i32);
            let newHeight: INT = height * zoom / (DEFAULT_ZOOM as i32);
            let hbmMaskNew: HBITMAP = CopyMonoImage(hbmMask, newWidth, newHeight, (STRETCH_DELETESCANS) as INT);
            let hbmColorNew: HBITMAP = CopyDIBImage(hbmColor, newWidth, newHeight, (STRETCH_DELETESCANS as i32));
            DeleteObject((hbmMask).cast());
            DeleteObject((hbmColor).cast());
            hbmMask = hbmMaskNew;
            hbmColor = hbmColorNew;
            hotX = (width * zoom / ((2 * DEFAULT_ZOOM)) as i32) as DWORD;
            hotY = (height * zoom / ((2 * DEFAULT_ZOOM)) as i32) as DWORD;
        }
    
        let mut ii: ICONINFO = ICONINFO { fIcon: (FALSE) as i32, xHotspot: hotX, yHotspot: hotY, hbmMask: hbmMask, hbmColor: hbmColor };
        let hCursor: HCURSOR = CreateIconIndirect(core::ptr::addr_of_mut!(ii)) as HCURSOR;
    
        DeleteObject((hbmMask).cast());
        DeleteObject((hbmColor).cast());
    
        return hCursor;
    }
}

impl CCanvasWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_drawing = (FALSE) as i32;
        (*this_).m_hitCanvasSizeBox = HIT_NONE;
        (*this_).m_ptOrig = POINT { x: -1, y: -1 };
        (*this_).m_nMouseDownMsg = 0;
        {
            (*this_).m_rcResizing.SetRectEmpty();
        }
    }
    pub unsafe fn OnEndDraw(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_drawing = (FALSE) as i32;
        toolsModel.OnEndDraw(bCancel);
        (*this_).__base0.Invalidate((FALSE) as BOOL);
    }
    pub unsafe fn updateScrollRange(&mut self) {
        let this_: *mut Self = self;
        let mut rcClient: CRect = CRect__make_0();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient.__base0));
    
        let mut sizePage: CSize = CSize__make_1(rcClient.__base0.right, rcClient.__base0.bottom);
        let mut sizeZoomed: CSize = CSize__make_1(
            Zoomed(imageModel.GetWidth()),
            Zoomed(imageModel.GetHeight()),
        );
        let mut sizeWhole: CSize = CSize__make_1(
            sizeZoomed.__base0.cx + ((GRIP_SIZE * 2)) as i32,
            sizeZoomed.__base0.cy + ((GRIP_SIZE * 2)) as i32,
        );
    
        // show/hide the scrollbars
        (*this_).__base0.ShowScrollBar(SB_HORZ, (sizePage.__base0.cx < sizeWhole.__base0.cx) as i32);
        (*this_).__base0.ShowScrollBar(SB_VERT, (sizePage.__base0.cy < sizeWhole.__base0.cy) as i32);
    
        if sizePage.__base0.cx < sizeWhole.__base0.cx || sizePage.__base0.cy < sizeWhole.__base0.cy {
            (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient.__base0)); // Scrollbars might change, get client rectangle again
            sizePage = CSize__make_1(rcClient.__base0.right, rcClient.__base0.bottom);
        }
    
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as u32;
        si.fMask = SIF_PAGE | SIF_RANGE;
        si.nMin = 0;
    
        si.nMax = sizeWhole.__base0.cx;
        si.nPage = sizePage.__base0.cx as u32;
        (*this_).__base0.SetScrollInfo((SB_HORZ) as i32, core::ptr::addr_of_mut!(si), (TRUE) as i32);
    
        si.nMax = sizeWhole.__base0.cy;
        si.nPage = sizePage.__base0.cy as u32;
        (*this_).__base0.SetScrollInfo((SB_VERT) as i32, core::ptr::addr_of_mut!(si), (TRUE) as i32);
    }
    pub unsafe fn updateScrollPos(&mut self, x: INT, y: INT) {
        let this_: *mut Self = self;
        (*this_).__base0.SetScrollPos((SB_HORZ) as i32, x, (TRUE) as i32);
        (*this_).__base0.SetScrollPos((SB_VERT) as i32, y, (TRUE) as i32);
    }
    pub unsafe fn ImageToCanvas__0(&mut self, pt: &mut POINT) {
        let this_: *mut Self = self;
        Zoomed(pt);
        pt.x += (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_HORZ) as i32)) as u32) as i32;
        pt.y += (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_VERT) as i32)) as u32) as i32;
    }
    pub unsafe fn ImageToCanvas__1(&mut self, rc: &mut RECT) {
        let this_: *mut Self = self;
        Zoomed(rc);
        OffsetRect(rc as *mut RECT, (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_HORZ) as i32)) as u32) as i32, (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_VERT) as i32)) as u32) as i32);
    }
    pub unsafe fn CanvasToImage__0(&mut self, pt: &mut POINT, bRound: BOOL) {
        let this_: *mut Self = self;
        pt.x -= (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_HORZ) as i32)) as u32) as i32;
        pt.y -= (GRIP_SIZE - ((*this_).__base0.GetScrollPos((SB_VERT) as i32)) as u32) as i32;
        UnZoomed(pt, bRound);
    }
    pub unsafe fn CanvasToImage__1(&mut self, rc: &mut RECT, bRound: BOOL) {
        let this_: *mut Self = self;
        OffsetRect(rc as *mut RECT, (*this_).__base0.GetScrollPos((SB_HORZ) as i32) - (GRIP_SIZE as i32), (*this_).__base0.GetScrollPos((SB_VERT) as i32) - (GRIP_SIZE as i32));
        UnZoomed(*rc, bRound);
    }
    pub unsafe fn GetImageRect(&mut self, rc: &mut RECT) {
        let this_: *mut Self = self;
        *rc = RECT { left: 0, top: 0, right: imageModel.GetWidth(), bottom: imageModel.GetHeight() };
    }
    pub unsafe fn getNewZoomRect(&mut self, rcView: &mut CRect, newZoom: INT, ptTarget: CPoint) {
        let this_: *mut Self = self;
        let mut rcImage: CRect = CRect__make_0();
        (*this_).GetImageRect(&mut rcImage.__base0);
        (*this_).ImageToCanvas__1(&mut rcImage.__base0);
    
        // Calculate the zoom rectangle
        let oldZoom: INT = toolsModel.GetZoom();
        (*this_).__base0.GetClientRect((rcView.op_to_tagRECT()).cast());
        let cxView: LONG = rcView.__base0.right * oldZoom / newZoom;
        let cyView: LONG = rcView.__base0.bottom * oldZoom / newZoom;
        rcView.SetRect(
            ptTarget.__base0.x - cxView / 2,
            ptTarget.__base0.y - cyView / 2,
            ptTarget.__base0.x + cxView / 2,
            ptTarget.__base0.y + cyView / 2,
        );
    
        // Shift the rectangle if necessary
        let mut dx: INT = 0;
        let mut dy: INT = 0;
        if rcView.__base0.left < rcImage.__base0.left {
            dx = rcImage.__base0.left - rcView.__base0.left;
        } else if rcImage.__base0.right < rcView.__base0.right {
            dx = rcImage.__base0.right - rcView.__base0.right;
        }
        if rcView.__base0.top < rcImage.__base0.top {
            dy = rcImage.__base0.top - rcView.__base0.top;
        } else if rcImage.__base0.bottom < rcView.__base0.bottom {
            dy = rcImage.__base0.bottom - rcView.__base0.bottom;
        }
        rcView.OffsetRect__0(dx, dy);
    
        rcView.IntersectRect(
            core::ptr::addr_of!(rcView.__base0),
            core::ptr::addr_of!(rcImage.__base0),
        );
    }
    pub unsafe fn zoomTo(&mut self, newZoom: INT, left: LONG, top: LONG) {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: left, y: top };
        (*this_).CanvasToImage__0(&mut pt, (FALSE) as BOOL);
    
        toolsModel.SetZoom(newZoom);
        (*this_).ImageToCanvas__0(&mut pt);
        pt.x += (*this_).__base0.GetScrollPos((SB_HORZ) as i32);
        pt.y += (*this_).__base0.GetScrollPos((SB_VERT) as i32);
    
        (*this_).updateScrollRange();
        (*this_).updateScrollPos(pt.x, pt.y);
        (*this_).__base0.Invalidate((TRUE) as BOOL);
    }
    pub unsafe fn CanvasHitTest(&mut self, pt: POINT) -> HITTEST {
        let this_: *mut Self = self;
        if selectionModel.m_bShow != 0 || IsWindowVisible(textEditWindow.op_to_HWND()) != 0 {
            return HIT_INNER;
        }
        let mut rcBase: RECT = (*this_).GetBaseRect();
        return getSizeBoxHitTest(pt, core::ptr::addr_of_mut!(rcBase));
    }
    pub unsafe fn GetBaseRect(&mut self) -> RECT {
        let this_: *mut Self = self;
        let mut rcBase: CRect = CRect__make_0();
        (*this_).GetImageRect(&mut (rcBase).__base0);
        (*this_).ImageToCanvas__1(&mut (rcBase).__base0);
        rcBase.InflateRect__0((GRIP_SIZE as i32), (GRIP_SIZE as i32));
        return (rcBase).__base0;
    }
    pub unsafe fn DoDraw(&mut self, hDC: HDC, rcClient: &mut RECT, rcPaint: &mut RECT) -> BOOL {
        let this_: *mut Self = self;
        // This is the target area we have to draw on
        let mut rcCanvasDraw: CRect = CRect__make_0();
        rcCanvasDraw.IntersectRect(rcClient as *const RECT, rcPaint as *const RECT);
    
        // Calculate image size
        let mut rcImage: CRect = CRect__make_0();
        (*this_).GetImageRect(&mut rcImage.__base0);
        let mut sizeImage: SIZE = SIZE { cx: imageModel.GetWidth(), cy: imageModel.GetHeight() };
    
        // We use a memory bitmap to reduce flickering
        let hbmCache1: HBITMAP = CreateDIBWithProperties(rcClient.right, rcClient.bottom);
        if hbmCache1.is_null() {
            return (FALSE) as BOOL; // Out of memory
        }
        let hbmCache2: HBITMAP = CreateDIBWithProperties(sizeImage.cx, sizeImage.cy);
        if hbmCache2.is_null() {
            DeleteObject((hbmCache1).cast());
            return (FALSE) as BOOL; // Out of memory
        }
    
        let hdcMem0: HDC = CreateCompatibleDC(hDC);
        let hbm0Old: HGDIOBJ = SelectObject(hdcMem0, (hbmCache1).cast());
    
        // Fill the background on hdcMem0
        FillRect(hdcMem0, core::ptr::addr_of_mut!(rcCanvasDraw.__base0), (COLOR_APPWORKSPACE + 1) as HBRUSH);
    
        // Draw the sizeboxes if necessary
        let mut rcBase: RECT = (*this_).GetBaseRect();
        if selectionModel.m_bShow == 0 && IsWindowVisible(textEditWindow.op_to_HWND()) == 0 {
            drawSizeBoxes(hdcMem0, core::ptr::addr_of_mut!(rcBase), (FALSE) as BOOL, core::ptr::addr_of_mut!(rcCanvasDraw.__base0));
        }
    
        // Calculate the target area on the image
        let mut rcImageDraw: CRect = {
            let mut tmp: CRect = CRect__make_0();
            tmp = rcCanvasDraw;
            tmp
        };
        (*this_).CanvasToImage__1(&mut rcImageDraw.__base0, (FALSE) as BOOL);
        rcImageDraw.IntersectRect(core::ptr::addr_of_mut!(rcImageDraw.__base0) as *const RECT, core::ptr::addr_of_mut!(rcImage.__base0) as *const RECT);
    
        // Consider rounding down by zooming
        rcImageDraw.__base0.right += 1;
        rcImageDraw.__base0.bottom += 1;
    
        // hdcMem1 <-- imageModel
        let hdcMem1: HDC = CreateCompatibleDC(hDC);
        let hbm1Old: HGDIOBJ = SelectObject(hdcMem1, (hbmCache2).cast());
        BitBlt(hdcMem1, rcImageDraw.__base0.left, rcImageDraw.__base0.top, rcImageDraw.Width(), rcImageDraw.Height(),
                 imageModel.GetDC(), rcImageDraw.__base0.left, rcImageDraw.__base0.top, SRCCOPY);
    
        // Draw overlay #1 on hdcMem1
        toolsModel.OnDrawOverlayOnImage(hdcMem1);
    
        // Transfer the bits with stretch (hdcMem0 <-- hdcMem1)
        (*this_).ImageToCanvas__1(&mut rcImage.__base0);
        StretchBlt(hdcMem0, rcImage.__base0.left, rcImage.__base0.top, rcImage.Width(), rcImage.Height(),
                     hdcMem1, 0, 0, sizeImage.cx, sizeImage.cy, SRCCOPY);
    
        // Clean up hdcMem1
        SelectObject(hdcMem1, hbm1Old);
        DeleteDC(hdcMem1);
    
        // Draw the grid on hdcMem0
        if g_showGrid != 0 && toolsModel.GetZoom() >= 4000 {
            let oldPen: HPEN = SelectObject(hdcMem0, (CreatePen((PS_SOLID as i32), 1, RGB!(160, 160, 160))).cast()) as HPEN;
            let mut counter: INT = 0;
            while counter < sizeImage.cy {
                let mut pt0: POINT = POINT { x: 0, y: counter };
                let mut pt1: POINT = POINT { x: sizeImage.cx, y: counter };
                (*this_).ImageToCanvas__0(&mut pt0);
                (*this_).ImageToCanvas__0(&mut pt1);
                MoveToEx(hdcMem0, pt0.x, pt0.y, core::ptr::null_mut());
                LineTo(hdcMem0, pt1.x, pt1.y);
                counter += 1;
            }
            let mut counter: INT = 0;
            while counter < sizeImage.cx {
                let mut pt0: POINT = POINT { x: counter, y: 0 };
                let mut pt1: POINT = POINT { x: counter, y: sizeImage.cy };
                (*this_).ImageToCanvas__0(&mut pt0);
                (*this_).ImageToCanvas__0(&mut pt1);
                MoveToEx(hdcMem0, pt0.x, pt0.y, core::ptr::null_mut());
                LineTo(hdcMem0, pt1.x, pt1.y);
                counter += 1;
            }
            DeleteObject(SelectObject(hdcMem0, (oldPen).cast()));
        }
    
        // Draw overlay #2 on hdcMem0
        toolsModel.OnDrawOverlayOnCanvas(hdcMem0);
    
        // Draw new frame on hdcMem0 if any
        if (*this_).m_hitCanvasSizeBox != HIT_NONE && (*this_).m_rcResizing.IsRectEmpty() == 0 {
            DrawXorRect(hdcMem0, core::ptr::addr_of_mut!((*this_).m_rcResizing.__base0));
        }
    
        // Transfer the bits (hDC <-- hdcMem0)
        BitBlt(hDC, rcCanvasDraw.__base0.left, rcCanvasDraw.__base0.top, rcCanvasDraw.Width(), rcCanvasDraw.Height(),
                 hdcMem0, rcCanvasDraw.__base0.left, rcCanvasDraw.__base0.top, SRCCOPY);
    
        // Clean up hdcMem0
        SelectObject(hdcMem0, hbm0Old);
        DeleteDC(hdcMem0);
        DeleteObject((hbmCache2).cast());
        DeleteObject((hbmCache1).cast());
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnHVScroll(&mut self, wParam: WPARAM, fnBar: INT) {
        let this_: *mut Self = self;
        let mut si: SCROLLINFO = core::mem::zeroed();
        si.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
        si.fMask = SIF_ALL;
        (*this_).__base0.GetScrollInfo(fnBar, core::ptr::addr_of_mut!(si));
    
        match (LOWORD!(wParam)) as u32 {
            SB_THUMBTRACK | SB_THUMBPOSITION => {
                si.nPos = (HIWORD!(wParam) as SHORT) as _;
            }
            SB_LINELEFT => {
                // SB_LINEUP
                si.nPos -= 15;
            }
            SB_LINERIGHT => {
                // SB_LINEDOWN
                si.nPos += 15;
            }
            SB_PAGELEFT => {
                // SB_PAGEUP
                si.nPos -= (si.nPage) as i32;
            }
            SB_PAGERIGHT => {
                // SB_PAGEDOWN
                si.nPos += (si.nPage) as i32;
            }
            _ => {}
        }
        si.nPos = max(min(si.nPos, si.nMax), si.nMin);
        (*this_).__base0.SetScrollInfo(fnBar, core::ptr::addr_of_mut!(si), (TRUE) as i32);
        (*this_).__base0.Invalidate((TRUE) as BOOL);
    }
    pub unsafe fn OnSize(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.m_hWnd as usize != 0 {
            (*this_).updateScrollRange();
        }
    
        return 0;
    }
    pub unsafe fn OnHScroll(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).OnHVScroll(wParam, (SB_HORZ) as INT);
        return 0;
    }
    pub unsafe fn OnVScroll(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).OnHVScroll(wParam, (SB_VERT) as INT);
        return 0;
    }
    pub unsafe fn OnEraseBkgnd(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let _ = this_;
        let _ = nMsg;
        let _ = wParam;
        let _ = lParam;
        let _ = bHandled;
        return (TRUE) as LRESULT; // do nothing => transparent background
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rcClient: RECT = core::mem::zeroed();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient));
    
        static mut s_bShowedOutOfMemory: BOOL = (FALSE) as i32; // Don't show "Out Of Memory" message multiple time
    
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hDC: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
    
        if (*this_).DoDraw(hDC, &mut rcClient, &mut ps.rcPaint) != 0 {
            s_bShowedOutOfMemory = (FALSE) as i32;
        } else if s_bShowedOutOfMemory == 0 {
            ShowOutOfMemory();
            s_bShowedOutOfMemory = (TRUE) as i32;
            imageModel.ClearHistory(); // Reduce memory usage
        }
    
        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnMouseMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
    
        if (*this_).m_nMouseDownMsg == WM_MBUTTONDOWN
        {
            let x: INT = (*this_).__base0.GetScrollPos((SB_HORZ) as i32) - (pt.x - (*this_).m_ptOrig.x);
            let y: INT = (*this_).__base0.GetScrollPos((SB_VERT) as i32) - (pt.y - (*this_).m_ptOrig.y);
            (*this_).__base0.SendMessageW__0(WM_HSCROLL, MAKEWPARAM!(SB_THUMBPOSITION, x), 0);
            (*this_).__base0.SendMessageW__0(WM_VSCROLL, MAKEWPARAM!(SB_THUMBPOSITION, y), 0);
            (*this_).m_ptOrig = pt;
            return 0;
        }
    
        (*this_).CanvasToImage__0(&mut pt, (TRUE) as BOOL);
    
        if toolsModel.GetActiveTool() == TOOL_ZOOM
        {
            (*this_).__base0.Invalidate((TRUE) as BOOL);
        }
    
        if (*this_).m_drawing == 0 || toolsModel.GetActiveTool() <= TOOL_AIRBRUSH
        {
            let mut tme: TRACKMOUSEEVENT = core::mem::zeroed();
            tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
            tme.dwFlags = TME_LEAVE;
            tme.hwndTrack = (*this_).__base0.m_hWnd;
            tme.dwHoverTime = 0;
            TrackMouseEvent(core::ptr::addr_of_mut!(tme));
    
            if (*this_).m_drawing == 0
            {
                let mut rcImage: CRect = CRect__make_0();
                (*this_).GetImageRect(&mut rcImage.__base0);
    
                let mut strCoord: CStringW = CStringW::new();
                if rcImage.PtInRect(pt) != 0
                {
                    atl_format!(strCoord, CPP_COM_WSTR_16A2910FBF5B.as_ptr(), pt.x, pt.y);
                }
                SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 1, strCoord.GetString() as LPARAM);
            }
        }
    
        if (*this_).m_drawing != 0 || toolsModel.IsSelection() != 0
        {
            toolsModel.DrawWithMouseTool(pt, wParam);
            return 0;
        }
    
        if (*this_).m_hitCanvasSizeBox == HIT_NONE || GetCapture() != (*this_).__base0.m_hWnd
        {
            return 0;
        }
    
        // Dragging now... Calculate the new size
        let mut cxImage: INT = imageModel.GetWidth();
        let mut cyImage: INT = imageModel.GetHeight();
        let cxDelta: INT = pt.x - (*this_).m_ptOrig.x;
        let cyDelta: INT = pt.y - (*this_).m_ptOrig.y;
        // switch (this_->m_hitCanvasSizeBox)
        let hit: HITTEST = (*this_).m_hitCanvasSizeBox;
        if hit == HIT_UPPER_LEFT
        {
            cxImage -= cxDelta;
            cyImage -= cyDelta;
        }
        else if hit == HIT_UPPER_CENTER
        {
            cyImage -= cyDelta;
        }
        else if hit == HIT_UPPER_RIGHT
        {
            cxImage += cxDelta;
            cyImage -= cyDelta;
        }
        else if hit == HIT_MIDDLE_LEFT
        {
            cxImage -= cxDelta;
        }
        else if hit == HIT_MIDDLE_RIGHT
        {
            cxImage += cxDelta;
        }
        else if hit == HIT_LOWER_LEFT
        {
            cxImage -= cxDelta;
            cyImage += cyDelta;
        }
        else if hit == HIT_LOWER_CENTER
        {
            cyImage += cyDelta;
        }
        else if hit == HIT_LOWER_RIGHT
        {
            cxImage += cxDelta;
            cyImage += cyDelta;
        }
        else
        {
            return 0;
        }
    
        // Limit bitmap size
        cxImage = max(1, cxImage);
        cyImage = max(1, cyImage);
        cxImage = min((MAXWORD) as i32, cxImage);
        cyImage = min((MAXWORD) as i32, cyImage);
    
        // Display new size
        let mut strSize: CStringW = CStringW::new();
        atl_format!(strSize, CPP_COM_WSTR_9768A8408F0A.as_ptr(), cxImage, cyImage);
        SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 2, strSize.GetString() as LPARAM);
    
        // Dragging now... Fix the position...
        let mut rcResizing: CRect = CRect__make_1(0, 0, cxImage, cyImage);
        // switch (this_->m_hitCanvasSizeBox)
        let hit2: HITTEST = (*this_).m_hitCanvasSizeBox;
        if hit2 == HIT_UPPER_LEFT
        {
            rcResizing.OffsetRect__0(cxDelta, cyDelta);
        }
        else if hit2 == HIT_UPPER_CENTER
        {
            rcResizing.OffsetRect__0(0, cyDelta);
        }
        else if hit2 == HIT_UPPER_RIGHT
        {
            rcResizing.OffsetRect__0(0, cyDelta);
        }
        else if hit2 == HIT_MIDDLE_LEFT
        {
            rcResizing.OffsetRect__0(cxDelta, 0);
        }
        else if hit2 == HIT_LOWER_LEFT
        {
            rcResizing.OffsetRect__0(cxDelta, 0);
        }
        else
        {
        }
        (*this_).ImageToCanvas__1(&mut rcResizing.__base0);
        (*this_).m_rcResizing = rcResizing;
        (*this_).__base0.Invalidate((TRUE) as BOOL);
    
        return 0;
    }
    pub unsafe fn OnKeyDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == (VK_ESCAPE) as _ {
            (*this_).OnEndDraw((TRUE) as BOOL);
            ReleaseCapture();
            (*this_).m_nMouseDownMsg = 0;
            (*this_).m_hitCanvasSizeBox = HIT_NONE;
            (*this_).m_rcResizing.SetRectEmpty();
            (*this_).__base0.Invalidate((TRUE) as BOOL);
        }
    
        return 0;
    }
    pub unsafe fn OnSetCursor(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if CWaitCursor::IsWaiting() != 0 {
            *bHandled = (FALSE) as i32;
            return 0;
        }
    
        if (*this_).m_nMouseDownMsg == WM_MBUTTONDOWN {
            SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_HANDDRAG)));
            return 0;
        }
    
        let mut pt: POINT = core::mem::zeroed();
        GetCursorPos(core::ptr::addr_of_mut!(pt));
        (*this_).__base0.ScreenToClient(core::ptr::addr_of_mut!(pt));
    
        let mut rcClient: CRect = CRect__make_0();
        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(rcClient.__base0));
    
        if rcClient.PtInRect(pt) == 0 {
            *bHandled = (FALSE) as i32;
            return 0;
        }
    
        let mut hitSelection: HITTEST = selectionModel.hitTest(pt);
        if hitSelection != HIT_NONE {
            if setCursorOnSizeBox(hitSelection) == 0 {
                SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_SIZEALL as LPCWSTR));
            }
            return 0;
        }
    
        let mut rcImage: CRect = CRect__make_0();
        (*this_).GetImageRect(&mut rcImage.__base0);
        (*this_).ImageToCanvas__1(&mut rcImage.__base0);
    
        if rcImage.PtInRect(pt) != 0 {
            let activeTool = toolsModel.GetActiveTool();
            if activeTool == TOOL_FILL {
                SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_FILL)));
            } else if activeTool == TOOL_COLOR {
                SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_COLOR)));
            } else if activeTool == TOOL_ZOOM {
                SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_ZOOM)));
            } else if activeTool == TOOL_PEN {
                SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_PEN)));
            } else if activeTool == TOOL_AIRBRUSH {
                SetCursor(LoadCursorW(g_hinstExe, MAKEINTRESOURCEW!(IDC_AIRBRUSH)));
            } else if activeTool == TOOL_RUBBER {
                (*this_).m_hRubberCursor.SetStyle(BrushStyleSquare,
                                         toolsModel.GetZoom(),
                                         toolsModel.GetRubberRadius(),
                                         paletteModel.GetBgColor(), (TRUE) as BOOL);
                (*this_).m_hRubberCursor.SetCursor();
            } else if activeTool == TOOL_BRUSH {
                (*this_).m_hBrushCursor.SetStyle(toolsModel.GetBrushStyle(),
                                        toolsModel.GetZoom(),
                                        toolsModel.GetBrushWidth() / 2,
                                        paletteModel.GetFgColor(), (FALSE) as BOOL);
                (*this_).m_hBrushCursor.SetCursor();
            } else {
                SetCursor(LoadCursorW(core::ptr::null_mut(), IDC_CROSS as LPCWSTR));
            }
            return 0;
        }
    
        if selectionModel.m_bShow != 0 || setCursorOnSizeBox((*this_).CanvasHitTest(pt)) == 0 {
            *bHandled = (FALSE) as i32;
        }
    
        return 0;
    }
    pub unsafe fn OnMouseWheel(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return SendMessageW((*this_).__base0.GetParent().op_to_HWND(), nMsg, wParam, lParam);
    }
    pub unsafe fn OnCancelMode(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // Cancel dragging
        (*this_).m_hitCanvasSizeBox = HIT_NONE;
        (*this_).m_rcResizing.SetRectEmpty();
        (*this_).__base0.Invalidate((TRUE) as BOOL);
        return 0;
    }
    pub unsafe fn OnCaptureChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 2, (CPP_COM_WSTR_96A296D224F2.as_ptr()) as LPARAM);
        return 0;
    }
    pub unsafe fn OnCtlColorEdit(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        SetTextColor(wParam as HDC, paletteModel.GetFgColor());
        SetBkMode(wParam as HDC, (TRANSPARENT as i32));
        return GetStockObject((NULL_BRUSH as i32)) as LRESULT;
    }
    pub unsafe fn OnPaletteModelColorChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        imageModel.NotifyImageChanged();
        return 0;
    }
    pub unsafe fn OnButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
    
        (*this_).m_nMouseDownMsg = nMsg;
        let bLeftButton: BOOL = (nMsg == WM_LBUTTONDOWN) as BOOL;
    
        if nMsg == WM_MBUTTONDOWN {
            (*this_).m_ptOrig = pt;
            (*this_).__base0.SetCapture();
            SetCursor(LoadCursorW(g_hinstExe, (IDC_HANDDRAG as usize) as LPCWSTR));
            return 0;
        }
    
        let hitSelection: HITTEST = selectionModel.hitTest(pt);
        if hitSelection != HIT_NONE {
            (*this_).m_drawing = (TRUE) as i32;
            (*this_).CanvasToImage__0(&mut pt, (TRUE) as BOOL);
            (*this_).__base0.SetCapture();
            toolsModel.OnButtonDown(bLeftButton, pt.x, pt.y, (FALSE) as i32);
            (*this_).__base0.Invalidate((TRUE) as BOOL);
            return 0;
        }
    
        let hit: HITTEST = (*this_).CanvasHitTest(pt);
        if hit == HIT_NONE || hit == HIT_BORDER {
            let tool = toolsModel.GetActiveTool();
            if tool == TOOL_BEZIER || tool == TOOL_SHAPE {
                toolsModel.OnEndDraw((TRUE) as BOOL);
                (*this_).__base0.Invalidate((TRUE) as BOOL);
            } else if tool == TOOL_FREESEL || tool == TOOL_RECTSEL {
                toolsModel.OnEndDraw((FALSE) as BOOL);
                (*this_).__base0.Invalidate((TRUE) as BOOL);
            } else {
            }
    
            toolsModel.resetTool(); // resets the point-buffer of the polygon and bezier functions
            return 0;
        }
    
        (*this_).CanvasToImage__0(&mut pt, (TRUE) as BOOL);
    
        if hit == HIT_INNER {
            (*this_).m_drawing = (TRUE) as i32;
            (*this_).__base0.SetCapture();
            toolsModel.OnButtonDown(bLeftButton, pt.x, pt.y, (FALSE) as i32);
            (*this_).__base0.Invalidate((TRUE) as BOOL);
            return 0;
        }
    
        if bLeftButton != 0 {
            (*this_).m_hitCanvasSizeBox = hit;
            (*this_).m_ptOrig = pt;
            (*this_).__base0.SetCapture();
        }
    
        0
    }
    pub unsafe fn OnButtonDblClk(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
        (*this_).CanvasToImage__0(&mut pt, (TRUE) as BOOL);
    
        (*this_).m_drawing = (FALSE) as i32;
        ReleaseCapture();
        (*this_).m_nMouseDownMsg = 0;
    
        toolsModel.OnButtonDown((nMsg == WM_LBUTTONDBLCLK) as BOOL, pt.x, pt.y, (TRUE) as i32);
        toolsModel.resetTool();
        (*this_).__base0.Invalidate((TRUE) as BOOL);
        return 0;
    }
    pub unsafe fn OnButtonUp(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
        (*this_).CanvasToImage__0(&mut pt, (TRUE) as BOOL);
    
        ReleaseCapture();
    
        let mut bLeftButton: BOOL = ((*this_).m_nMouseDownMsg == WM_LBUTTONDOWN) as BOOL;
        (*this_).m_nMouseDownMsg = 0;
    
        if (*this_).m_drawing != 0 {
            (*this_).m_drawing = (FALSE) as i32;
            toolsModel.OnButtonUp(bLeftButton, pt.x, pt.y);
            (*this_).__base0.Invalidate((FALSE) as BOOL);
            SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 2, CPP_COM_WSTR_96A296D224F2.as_ptr() as LPARAM);
            return 0;
        }
    
        if (*this_).m_hitCanvasSizeBox == HIT_NONE || bLeftButton == 0 {
            return 0;
        }
    
        // Resize the image
        let mut cxImage: INT = imageModel.GetWidth();
        let mut cyImage: INT = imageModel.GetHeight();
        let mut cxDelta: INT = pt.x - (*this_).m_ptOrig.x;
        let mut cyDelta: INT = pt.y - (*this_).m_ptOrig.y;
        match (*this_).m_hitCanvasSizeBox {
            v if v == HIT_UPPER_LEFT => {
                imageModel.Crop(cxImage - cxDelta, cyImage - cyDelta, cxDelta, cyDelta);
            }
            v if v == HIT_UPPER_CENTER => {
                imageModel.Crop(cxImage, cyImage - cyDelta, 0, cyDelta);
            }
            v if v == HIT_UPPER_RIGHT => {
                imageModel.Crop(cxImage + cxDelta, cyImage - cyDelta, 0, cyDelta);
            }
            v if v == HIT_MIDDLE_LEFT => {
                imageModel.Crop(cxImage - cxDelta, cyImage, cxDelta, 0);
            }
            v if v == HIT_MIDDLE_RIGHT => {
                imageModel.Crop(cxImage + cxDelta, cyImage, 0, 0);
            }
            v if v == HIT_LOWER_LEFT => {
                imageModel.Crop(cxImage - cxDelta, cyImage + cyDelta, cxDelta, 0);
            }
            v if v == HIT_LOWER_CENTER => {
                imageModel.Crop(cxImage, cyImage + cyDelta, 0, 0);
            }
            v if v == HIT_LOWER_RIGHT => {
                imageModel.Crop(cxImage + cxDelta, cyImage + cyDelta, 0, 0);
            }
            _ => {
            }
        }
        (*this_).m_rcResizing.SetRectEmpty();
    
        g_imageSaved = (FALSE) as i32;
    
        (*this_).m_hitCanvasSizeBox = HIT_NONE;
        toolsModel.resetTool(); // resets the point-buffer of the polygon and bezier functions
        (*this_).updateScrollRange();
        (*this_).__base0.Invalidate((TRUE) as BOOL);
        return 0;
    }
}
