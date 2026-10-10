use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_2D9285714724: [WCHAR; 8] = [0x0025, 0x0064, 0x002E, 0x0025, 0x0064, 0x0025, 0x0025, 0x0000];
pub static CPP_COM_WSTR_C5D4AD672797: [WCHAR; 5] = [0x0025, 0x0064, 0x0025, 0x0025, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn SetCapture(hWnd: HWND) -> HWND;
}

// ---- file-scope preamble (translated) ----
#[allow(non_upper_case_globals)]
pub static s_AirRadius: [BYTE; 4] = [5, 8, 3, 12];

#[allow(non_upper_case_globals)]





















unsafe extern "C" fn getSplitRects(rects: *mut RECT, cColumns: i32, cRows: i32, prc: LPCRECT, ppt: LPPOINT) -> i32 {
    let cx: i32 = (*prc).right - (*prc).left;
    let cy: i32 = (*prc).bottom - (*prc).top;
    let mut i: i32 = 0;
    let mut iRow: i32 = 0;
    while iRow < cRows {
        let mut iColumn: i32 = 0;
        while iColumn < cColumns {
            let rc = rects.offset(i as isize) as *mut RECT;
            (*rc).left = (*prc).left + (iColumn * cx / cColumns);
            (*rc).top = (*prc).top + (iRow * cy / cRows);
            (*rc).right = (*prc).left + ((iColumn + 1) * cx / cColumns);
            (*rc).bottom = (*prc).top + ((iRow + 1) * cy / cRows);
            if !ppt.is_null() && PtInRect(rc, *ppt) != 0 {
                return i;
            }
            i += 1;
            iColumn += 1;
        }
        iRow += 1;
    }
    return -1;
}

unsafe extern "C" fn getTransRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> INT {
    return getSplitRects(rects, 1, 2, prc, ppt);
}

unsafe extern "C" fn getRubberRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> INT {
    return getSplitRects(rects, 1, 4, prc, ppt);
}

unsafe extern "C" fn getBrushRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> i32 {
    return getSplitRects(rects, 3, 4, prc, ppt);
}

unsafe extern "C" fn getLineRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> INT {
    return getSplitRects(rects, 1, 5, prc, ppt);
}

unsafe extern "C" fn getAirBrushRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> INT {
    let cx: INT = (*prc).right - (*prc).left;
    let cy: INT = (*prc).bottom - (*prc).top;

    (*rects.add(0)) = *prc;
    (*rects.add(1)) = *prc;
    (*rects.add(2)) = *prc;
    (*rects.add(3)) = *prc;

    let xv: LONG = (*prc).left + cx * 3 / 8;
    (*rects.add(0)).right = xv;
    (*rects.add(1)).left = xv;

    let yv: LONG = (*prc).top + cy / 2;
    (*rects.add(0)).bottom = yv;
    (*rects.add(1)).bottom = yv;

    let tv: LONG = (*prc).top + cy / 2;
    (*rects.add(2)).top = tv;
    (*rects.add(3)).top = tv;

    let xv2: LONG = (*prc).left + cx * 2 / 8;
    (*rects.add(2)).right = xv2;
    (*rects.add(3)).left = xv2;

    if !ppt.is_null() {
        let mut i: INT = 0;
        while i < 4 {
            if PtInRect(rects.add(i as usize), *ppt) != 0 {
                return i;
            }
            i += 1;
        }
    }
    return -1;
}

unsafe extern "C" fn getBoxRects(rects: *mut RECT, prc: LPCRECT, ppt: LPPOINT) -> INT {
    return getSplitRects(rects, 1, 3, prc, ppt);
}

impl CToolSettingsWindow {
    pub unsafe fn DoCreate(&mut self, hwndParent: HWND) -> BOOL {
        let this_: *mut Self = self;
        let toolSettingsWindowPos: RECT = RECT {
            left: X_TOOLSETTINGS,
            top: Y_TOOLSETTINGS,
            right: X_TOOLSETTINGS + CX_TOOLSETTINGS,
            bottom: Y_TOOLSETTINGS + CY_TOOLSETTINGS,
        };
        return (!(*this_).Create(&mut toolBoxContainer, &mut toolSettingsWindowPos, core::ptr::null_mut(), WS_CHILD | WS_VISIBLE).is_null()) as BOOL;
    }
    pub unsafe fn drawTrans(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rc: [RECT; 2 as usize] = core::mem::zeroed();
        getTransRects(rc.as_mut_ptr(), prc, core::ptr::null_mut());
    
        FillRect(hdc, core::ptr::addr_of_mut!(rc[toolsModel.IsBackgroundTransparent() as usize]), (COLOR_HIGHLIGHT + 1) as usize as HBRUSH);
        DrawIconEx(hdc, rc[0 as usize].left, rc[0 as usize].top, (*this_).m_hNontranspIcon,
                     CX_TRANS_ICON, CY_TRANS_ICON, 0, core::ptr::null_mut(), DI_NORMAL);
        DrawIconEx(hdc, rc[1 as usize].left, rc[1 as usize].top, (*this_).m_hTranspIcon,
                     CX_TRANS_ICON, CY_TRANS_ICON, 0, core::ptr::null_mut(), DI_NORMAL);
    }
    pub unsafe fn drawRubber(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rects: [RECT; 4] = core::mem::zeroed();
        let mut rcRubber: RECT = core::mem::zeroed();
        getRubberRects(rects.as_mut_ptr(), prc, core::ptr::null_mut());
        let xCenter: INT = ((*prc).left + (*prc).right) / 2;
        let mut i: INT = 0;
        while i < 4 {
            let mut iColor: INT;
            let radius: INT = i + 2;
            if toolsModel.GetRubberRadius() == radius {
                FillRect(hdc, core::ptr::addr_of_mut!(rects[i as usize]), GetSysColorBrush((COLOR_HIGHLIGHT) as i32));
                iColor = (COLOR_HIGHLIGHTTEXT) as i32;
            } else {
                iColor = (COLOR_WINDOWTEXT) as i32;
            }
    
            let yCenter: INT = (rects[i as usize].top + rects[i as usize].bottom) / 2;
            rcRubber.left = xCenter - radius;
            rcRubber.top = yCenter - radius;
            rcRubber.right = rcRubber.left + radius * 2;
            rcRubber.bottom = rcRubber.top + radius * 2;
            FillRect(hdc, core::ptr::addr_of_mut!(rcRubber), GetSysColorBrush(iColor));
            i += 1;
        }
    }
    pub unsafe fn drawBrush(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rects: [RECT; 12] = core::mem::zeroed();
        getBrushRects(rects.as_mut_ptr(), prc);
    
        let mut i: INT = 0;
        while i < 12 {
            let mut rcItem: RECT = rects[i as usize];
            let x: INT = (rcItem.left + rcItem.right) / 2;
            let y: INT = (rcItem.top + rcItem.bottom) / 2;
    
            let mut iColor: INT;
            let data: &BrushStyleAndWidth = &c_BrushPresets[i as usize];
            if data.width == toolsModel.GetBrushWidth() && data.style == toolsModel.GetBrushStyle() {
                iColor = (COLOR_HIGHLIGHTTEXT) as i32;
                FillRect(
                    hdc,
                    core::ptr::addr_of_mut!(rcItem),
                    ((COLOR_HIGHLIGHT + 1) as usize) as HBRUSH,
                );
            } else {
                iColor = (COLOR_WINDOWTEXT) as i32;
            }
    
            Brush(
                hdc,
                x,
                y,
                x,
                y,
                GetSysColor(iColor),
                data.style,
                data.width,
            );
    
            i += 1;
        }
    }
    pub unsafe fn drawLine(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rects: [RECT; 5] = core::mem::zeroed();
        getLineRects(rects.as_mut_ptr(), prc);
    
        let mut i: INT = 0;
        while i < 5 {
            let penWidth: INT = i + 1;
            let mut rcLine: CRect = core::ptr::read(core::ptr::addr_of!(rects[i as usize]) as *const CRect);
            rcLine.InflateRect(-2, 0);
            rcLine.top = (rcLine.top + rcLine.bottom - penWidth) / 2;
            rcLine.bottom = rcLine.top + penWidth;
            if toolsModel.GetLineWidth() == penWidth {
                FillRect(
                    hdc,
                    core::ptr::addr_of!(rects[i as usize]) as *const RECT,
                    GetSysColorBrush((COLOR_HIGHLIGHT) as i32),
                );
                FillRect(
                    hdc,
                    core::ptr::addr_of!(rcLine) as *const RECT,
                    GetSysColorBrush((COLOR_HIGHLIGHTTEXT) as i32),
                );
            } else {
                FillRect(
                    hdc,
                    core::ptr::addr_of!(rcLine) as *const RECT,
                    GetSysColorBrush((COLOR_WINDOWTEXT) as i32),
                );
            }
            i += 1;
        }
    }
    pub unsafe fn drawBox(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let mut rects: [CRect; 3] = core::mem::zeroed();
        getBoxRects(rects.as_mut_ptr() as *mut RECT, prc);
    
        let mut iItem: INT = 0;
        while iItem < 3 {
            let rcItem: *mut CRect = core::ptr::addr_of_mut!(rects[iItem as usize]);
    
            if toolsModel.GetShapeStyle() == iItem {
                FillRect(hdc, rcItem as *const RECT, GetSysColorBrush((COLOR_HIGHLIGHT) as i32));
            }
    
            (*rcItem).InflateRect(-5, -5);
    
            if iItem <= 1 {
                let rgbPen: COLORREF;
                if toolsModel.GetShapeStyle() == iItem {
                    rgbPen = GetSysColor((COLOR_HIGHLIGHTTEXT) as i32);
                } else {
                    rgbPen = GetSysColor((COLOR_WINDOWTEXT) as i32);
                }
                let hOldBrush: HGDIOBJ;
                if iItem == 0 {
                    hOldBrush = SelectObject(hdc, GetStockObject((NULL_BRUSH as i32)));
                } else {
                    hOldBrush = SelectObject(hdc, (GetSysColorBrush((COLOR_APPWORKSPACE) as i32)).cast());
                }
                let hOldPen: HGDIOBJ = SelectObject(hdc, (CreatePen((PS_SOLID as i32), 1, rgbPen)).cast());
                Rectangle(hdc, (*rcItem).left, (*rcItem).top, (*rcItem).right, (*rcItem).bottom);
                DeleteObject(SelectObject(hdc, hOldPen));
                SelectObject(hdc, hOldBrush);
            } else {
                if toolsModel.GetShapeStyle() == iItem {
                    FillRect(hdc, rcItem as *const RECT, GetSysColorBrush((COLOR_HIGHLIGHTTEXT) as i32));
                } else {
                    FillRect(hdc, rcItem as *const RECT, GetSysColorBrush((COLOR_WINDOWTEXT) as i32));
                }
            }
    
            iItem += 1;
        }
    }
    pub unsafe fn drawAirBrush(&mut self, hdc: HDC, prc: LPCRECT) {
        let this_: *mut Self = self;
        let _ = this_;
        let mut rects: [RECT; 4] = core::mem::zeroed();
        getAirBrushRects(rects.as_mut_ptr(), prc);
    
        srand(0);
        let mut i: usize = 0;
        while i < 4 {
            let rc: *mut RECT = rects.as_mut_ptr().add(i);
            let x: i32 = ((*rc).left + (*rc).right) / 2;
            let y: i32 = ((*rc).top + (*rc).bottom) / 2;
            let bHigh: i32 = ((s_AirRadius[i]) as i32 == toolsModel.GetAirBrushRadius()) as i32;
            if bHigh != 0 {
                FillRect(hdc, rc, GetSysColorBrush((COLOR_HIGHLIGHT) as i32));
    
                let mut k: i32 = 0;
                while k < 3 {
                    Airbrush(hdc, x, y, GetSysColor((COLOR_HIGHLIGHTTEXT) as i32), s_AirRadius[i] as i32);
                    k += 1;
                }
            } else {
                let mut k: i32 = 0;
                while k < 3 {
                    Airbrush(hdc, x, y, GetSysColor((COLOR_WINDOWTEXT) as i32), s_AirRadius[i] as i32);
                    k += 1;
                }
            }
            i += 1;
        }
    }
    pub unsafe fn calculateTwoBoxes(&mut self, rect1: &mut CRect, rect2: &mut CRect) {
        let this_: *mut Self = self;
        let mut rcClient: CRect = CRect__zeroed();
        (*this_).__base0.GetClientRect((core::ptr::addr_of_mut!(rcClient)).cast());
        rcClient.InflateRect(-MARGIN1, -MARGIN1);
    
        let yCenter: i32 = (rcClient.top + rcClient.bottom) / 2;
        rect1.SetRect(rcClient.left, rcClient.top, rcClient.right, yCenter);
        rect2.SetRect(rcClient.left, yCenter, rcClient.right, rcClient.bottom);
    
        rect1.InflateRect(-MARGIN2, -MARGIN2);
        rect2.InflateRect(-MARGIN2, -MARGIN2);
    }
    pub unsafe fn OnCreate(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        /* preloading the draw transparent/nontransparent icons for later use */
        (*this_).m_hNontranspIcon = LoadImageW(g_hinstExe, MAKEINTRESOURCEW!(IDI_NONTRANSPARENT),
                                             IMAGE_ICON, CX_TRANS_ICON, CY_TRANS_ICON, LR_DEFAULTCOLOR) as HICON;
        (*this_).m_hTranspIcon = LoadImageW(g_hinstExe, MAKEINTRESOURCEW!(IDI_TRANSPARENT),
                                          IMAGE_ICON, CX_TRANS_ICON, CY_TRANS_ICON, LR_DEFAULTCOLOR) as HICON;
    
        let mut trackbarZoomPos: CRect = CRect__zeroed();
        let mut rect2: CRect = CRect__zeroed();
        (*this_).calculateTwoBoxes(&mut trackbarZoomPos, &mut rect2);
        trackbarZoomPos.InflateRect(-1, -1);
    
        (*this_).trackbarZoom.Create(TRACKBAR_CLASS, (*this_).__base0.m_hWnd, trackbarZoomPos, core::ptr::null_mut(), WS_CHILD | TBS_VERT | TBS_AUTOTICKS, 0, 0, core::ptr::null_mut());
        (*this_).trackbarZoom.SendMessageW__0(TBM_SETRANGE, ((TRUE) as UINT) as WPARAM, MAKELPARAM!(MIN_ZOOM_TRACK, MAX_ZOOM_TRACK));
        (*this_).trackbarZoom.SendMessageW__0(TBM_SETPOS, ((TRUE) as UINT) as WPARAM, ((DEFAULT_ZOOM_TRACK) as WPARAM) as LPARAM);
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        DestroyIcon((*this_).m_hNontranspIcon);
        DestroyIcon((*this_).m_hTranspIcon);
        return 0;
    }
    pub unsafe fn OnVScroll(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let trackPos: INT = MAX_ZOOM_TRACK - ((*this_).trackbarZoom.SendMessageW__0(TBM_GETPOS, 0, 0) as INT);
        canvasWindow.zoomTo((MIN_ZOOM << trackPos) as i32, 0, 0);
    
        let zoomRate: INT = toolsModel.GetZoom();
    
        let mut strZoom = CStringW::new();
        if zoomRate % 10 == 0 {
            atl_format!(strZoom, CPP_COM_WSTR_C5D4AD672797.as_ptr(), zoomRate / 10);
        } else {
            atl_format!(strZoom, CPP_COM_WSTR_2D9285714724.as_ptr(), zoomRate / 10, zoomRate % 10);
        }
    
        SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 1, strZoom.GetString() as LPARAM);
    
        (*this_).OnToolsModelZoomChanged(nMsg, wParam, lParam, bHandled);
        return 0;
    }
    pub unsafe fn OnPaint(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rect1: CRect = CRect__zeroed();
        let mut rect2: CRect = CRect__zeroed();
        (*this_).calculateTwoBoxes(&mut rect1, &mut rect2);
    
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hdc: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
    
        if toolsModel.GetActiveTool() != TOOL_ZOOM {
            DrawEdge(hdc, (core::ptr::addr_of_mut!(rect1)).cast(), BDR_SUNKENOUTER, BF_RECT | BF_MIDDLE);
        }
    
        if toolsModel.GetActiveTool() >= TOOL_RECT {
            DrawEdge(hdc, (core::ptr::addr_of_mut!(rect2)).cast(), BDR_SUNKENOUTER, BF_RECT | BF_MIDDLE);
        }
    
        rect1.InflateRect(-MARGIN2, -MARGIN2);
        rect2.InflateRect(-MARGIN2, -MARGIN2);
    
        let tool = toolsModel.GetActiveTool();
        if tool == TOOL_FREESEL || tool == TOOL_RECTSEL || tool == TOOL_TEXT {
            (*this_).drawTrans(hdc, core::ptr::addr_of_mut!(rect1));
        } else if tool == TOOL_RUBBER {
            (*this_).drawRubber(hdc, core::ptr::addr_of_mut!(rect1));
        } else if tool == TOOL_BRUSH {
            (*this_).drawBrush(hdc, core::ptr::addr_of_mut!(rect1));
        } else if tool == TOOL_AIRBRUSH {
            (*this_).drawAirBrush(hdc, core::ptr::addr_of_mut!(rect1));
        } else if tool == TOOL_LINE || tool == TOOL_BEZIER {
            (*this_).drawLine(hdc, core::ptr::addr_of_mut!(rect1));
        } else if tool == TOOL_RECT || tool == TOOL_SHAPE || tool == TOOL_ELLIPSE || tool == TOOL_RRECT {
            (*this_).drawBox(hdc, core::ptr::addr_of_mut!(rect1));
            (*this_).drawLine(hdc, core::ptr::addr_of_mut!(rect2));
        } else if tool == TOOL_COLOR {
            if (*this_).m_rgbPickColor != CLR_INVALID {
                let hbr: HBRUSH = CreateSolidBrush((*this_).m_rgbPickColor);
                FillRect(hdc, core::ptr::addr_of_mut!(rect1), hbr);
                DeleteObject((hbr).cast());
            }
        } else if tool == TOOL_FILL || tool == TOOL_ZOOM || tool == TOOL_PEN {
        }
    
        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnLButtonDown(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: GET_X_LPARAM!(lParam), y: GET_Y_LPARAM!(lParam) };
    
        let mut rect1: CRect = CRect__zeroed();
        let mut rect2: CRect = CRect__zeroed();
        (*this_).calculateTwoBoxes(&mut rect1, &mut rect2);
        let mut rects: [RECT; 12] = core::mem::zeroed();
    
        let mut iItem: INT;
        let sw = toolsModel.GetActiveTool();
        match sw {
            _ if sw == TOOL_FREESEL || sw == TOOL_RECTSEL || sw == TOOL_TEXT => {
                iItem = getTransRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetBackgroundTransparent(iItem);
                }
            }
            _ if sw == TOOL_RUBBER => {
                iItem = getRubberRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetRubberRadius(iItem + 2);
                }
            }
            _ if sw == TOOL_BRUSH => {
                iItem = getBrushRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    let data: &BrushStyleAndWidth = &c_BrushPresets[iItem as usize];
                    toolsModel.SetBrushStyle(data.style);
                    toolsModel.SetBrushWidth(data.width);
                }
            }
            _ if sw == TOOL_AIRBRUSH => {
                iItem = getAirBrushRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetAirBrushRadius((s_AirRadius[iItem as usize]) as _);
                }
            }
            _ if sw == TOOL_LINE || sw == TOOL_BEZIER => {
                iItem = getLineRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetLineWidth(iItem + 1);
                }
            }
            _ if sw == TOOL_RECT || sw == TOOL_SHAPE || sw == TOOL_ELLIPSE || sw == TOOL_RRECT => {
                iItem = getBoxRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect1) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetShapeStyle(iItem);
                }
    
                iItem = getLineRects(rects.as_mut_ptr(), core::ptr::addr_of!(rect2) as LPCRECT, core::ptr::addr_of_mut!(pt));
                if iItem != -1 {
                    toolsModel.SetLineWidth(iItem + 1);
                }
            }
            _ if sw == TOOL_FILL || sw == TOOL_COLOR || sw == TOOL_ZOOM || sw == TOOL_PEN => {
            }
            _ => {}
        }
    
        SetCapture(GetParent((*this_).__base0.m_hWnd));
        return 0;
    }
    pub unsafe fn OnNotify(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let pnmhdr: *mut NMHDR = lParam as *mut NMHDR;
        if (*pnmhdr).code == NM_CUSTOMDRAW {
            let pCustomDraw: *mut NMCUSTOMDRAW = pnmhdr as *mut NMCUSTOMDRAW;
            (*pCustomDraw).uItemState &= !CDIS_FOCUS; // Do not draw the focus
        }
        return 0;
    }
    pub unsafe fn OnToolsModelToolChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_rgbPickColor = CLR_INVALID;
        (*this_).__base0.Invalidate((TRUE) as i32);
        (*this_).trackbarZoom.ShowWindow(if wParam == (TOOL_ZOOM as u64) { (SW_SHOW) as i32 } else { (SW_HIDE) as i32 });
        return 0;
    }
    pub unsafe fn OnToolsModelSettingsChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.Invalidate((TRUE) as i32);
        return 0;
    }
    pub unsafe fn OnToolsModelColorPicked(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_rgbPickColor = wParam as COLORREF;
        (*this_).__base0.Invalidate((TRUE) as i32);
        return 0;
    }
    pub unsafe fn OnToolsModelZoomChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut tbPos: i32 = MIN_ZOOM_TRACK;
        let mut tempZoom: i32 = toolsModel.GetZoom();
    
        while tempZoom > (MIN_ZOOM as i32) {
            tbPos += 1;
            tempZoom = tempZoom >> 1;
        }
    
        (*this_).trackbarZoom.SendMessageW__0(TBM_SETPOS as UINT, TRUE as WPARAM, (MAX_ZOOM_TRACK - tbPos) as LPARAM);
        return 0 as LRESULT;
    }
}

impl BrushStyleAndWidth {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
