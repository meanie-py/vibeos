use crate::prelude::*;
use crate::mspaint::drawing_cpp_h::Airbrush__1;
use crate::mspaint::drawing_cpp_h::Bezier__1;
use crate::mspaint::drawing_cpp_h::Brush__1;
use crate::mspaint::drawing_cpp_h::Ellp__1;
use crate::mspaint::drawing_cpp_h::Erase__1;
use crate::mspaint::drawing_cpp_h::Fill__1;
use crate::mspaint::drawing_cpp_h::Line__1;
use crate::mspaint::drawing_cpp_h::Poly__0;
use crate::mspaint::drawing_cpp_h::Poly__1;
use crate::mspaint::drawing_cpp_h::RRect__1;
use crate::mspaint::drawing_cpp_h::Rect__1;
use crate::mspaint::drawing_cpp_h::Replace__1;
use crate::mspaint::drawing_cpp_h::Text__1;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1092529DF21F: [WCHAR; 10] = [0x0025, 0x006C, 0x0064, 0x0020, 0x0078, 0x0020, 0x0025, 0x006C, 0x0064, 0x0000];
pub static CPP_COM_WSTR_16A2910FBF5B: [WCHAR; 9] = [0x0025, 0x006C, 0x0064, 0x002C, 0x0020, 0x0025, 0x006C, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NO_DIRECTION: DIRECTION = -1;
pub const DIRECTION_HORIZONTAL: DIRECTION = 0;
pub const DIRECTION_VERTICAL: DIRECTION = 1;
pub const DIRECTION_DIAGONAL_RIGHT_DOWN: DIRECTION = 2;
pub const DIRECTION_DIAGONAL_RIGHT_UP: DIRECTION = 3;

pub static mut s_cPoints: SIZE_T = 0;
// CHeapPtr<POINT, CLocalAllocator>: default-constructed ATL smart pointer, modeled as its raw pointer (null).
pub static mut s_dynamicPoints: *mut POINT = core::ptr::null_mut();
pub static mut s_staticPoints: [POINT; 512] = unsafe { core::mem::zeroed() };
pub static mut s_maxPoints: SIZE_T = 512;
pub static mut s_pPoints: LPPOINT = core::ptr::addr_of_mut!(s_staticPoints) as *mut POINT;
pub static mut g_ptStart: POINT = unsafe { core::mem::zeroed() };
pub static mut g_ptEnd: POINT = unsafe { core::mem::zeroed() };































































































#[no_mangle]
pub unsafe extern "C" fn regularize(x0: LONG, y0: LONG, x1: *mut LONG, y1: *mut LONG) {
    if labs(*x1 - x0) >= labs(*y1 - y0) {
        *y1 = y0 + if *y1 > y0 { labs(*x1 - x0) } else { -labs(*x1 - x0) };
    } else {
        *x1 = x0 + if *x1 > x0 { labs(*y1 - y0) } else { -labs(*y1 - y0) };
    }
}
#[no_mangle]
pub unsafe extern "C" fn roundTo8Directions(x0: LONG, y0: LONG, x1: *mut LONG, y1: *mut LONG) {
    if labs(*x1 - x0) >= labs(*y1 - y0) {
        if labs(*y1 - y0) * 5 < labs(*x1 - x0) * 2 {
            *y1 = y0;
        } else {
            *y1 = y0 + (if *y1 > y0 { labs(*x1 - x0) } else { -labs(*x1 - x0) });
        }
    } else {
        if labs(*x1 - x0) * 5 < labs(*y1 - y0) * 2 {
            *x1 = x0;
        } else {
            *x1 = x0 + (if *x1 > x0 { labs(*y1 - y0) } else { -labs(*y1 - y0) });
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn nearlyEqualPoints(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let cxThreshold: i32 = toolsModel.GetLineWidth() + UnZoomed__0(GetSystemMetrics((SM_CXDRAG) as i32), FALSE);
    let cyThreshold: i32 = toolsModel.GetLineWidth() + UnZoomed__0(GetSystemMetrics((SM_CYDRAG) as i32), FALSE);
    return if (abs(x1 - x0) <= cxThreshold) && (abs(y1 - y0) <= cyThreshold) { 1 } else { 0 };
}
#[no_mangle]
pub unsafe extern "C" fn getBoundaryOfPoints(rcBoundary: &mut RECT, mut cPoints: SIZE_T, mut pPoints: *const POINT) {
    let mut ptMin: POINT = POINT { x: (MAXLONG) as i32, y: (MAXLONG) as i32 };
    let mut ptMax: POINT = POINT { x: (MINLONG) as i32, y: (MINLONG) as i32 };
    loop {
        let old = cPoints;
        cPoints = cPoints.wrapping_sub(1);
        if !(old > 0) {
            break;
        }
        let x: i32 = (*pPoints).x;
        let y: i32 = (*pPoints).y;
        ptMin = POINT { x: min(x, ptMin.x), y: min(y, ptMin.y) };
        ptMax = POINT { x: max(x, ptMax.x), y: max(y, ptMax.y) };
        pPoints = pPoints.add(1);
    }

    ptMax.x += 1;
    ptMax.y += 1;

    let rc = CRect__make_5(ptMin, ptMax);
    *rcBoundary = (rc).__base0;
}
#[no_mangle]
pub unsafe extern "C" fn ShiftPoints(dx: INT, dy: INT) {
    let mut i: SIZE_T = 0;
    while i < s_cPoints {
        let pt: *mut POINT = s_pPoints.add(i as usize);
        (*pt).x += dx;
        (*pt).y += dy;
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn BuildMaskFromPoints() {
    let mut rc = CRect__make_0();
    getBoundaryOfPoints(&mut (rc).__base0, s_cPoints, s_pPoints);
    ShiftPoints(-rc.__base0.left, -rc.__base0.top);

    let hdcMem: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbmMask: HBITMAP = CreateBitmap(rc.Width(), rc.Height(), 1, 1, core::ptr::null_mut());
    let hbmOld: HGDIOBJ = SelectObject(hdcMem, hbmMask as HGDIOBJ);
    FillRect(hdcMem, core::ptr::addr_of_mut!(rc.__base0), GetStockObject((BLACK_BRUSH as i32)) as HBRUSH);
    let hPenOld: HGDIOBJ = SelectObject(hdcMem, GetStockObject((NULL_PEN as i32)));
    let hbrOld: HGDIOBJ = SelectObject(hdcMem, GetStockObject((WHITE_BRUSH as i32)));
    Polygon(hdcMem, s_pPoints, s_cPoints as INT);
    SelectObject(hdcMem, hbrOld);
    SelectObject(hdcMem, hPenOld);
    SelectObject(hdcMem, hbmOld);
    DeleteDC(hdcMem);

    selectionModel.setMask(&rc, hbmMask);
}

unsafe extern "C" fn pushToPoints(x: i32, y: i32) {
    if s_cPoints + 1 >= s_maxPoints {
        let newMax: usize = (s_maxPoints + 512) as _;
        let cbNew: usize = newMax * core::mem::size_of::<POINT>();
        if (*core::ptr::addr_of_mut!(s_dynamicPoints)).ReallocateBytes(cbNew) == 0 {
            ATLTRACE!(
                b"%d, %d, %d\n\0".as_ptr() as *const i8,
                s_cPoints as i32,
                s_maxPoints as i32,
                cbNew as i32,
            );
            return;
        }

        if s_pPoints == (core::ptr::addr_of_mut!(s_staticPoints) as *mut POINT) {
            CopyMemory!(
                (*(*core::ptr::addr_of_mut!(s_dynamicPoints))).m_pData as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(s_staticPoints) as *const core::ffi::c_void,
                s_cPoints * (core::mem::size_of::<POINT>()) as _,
            );
        }

        s_pPoints = (*(*core::ptr::addr_of_mut!(s_dynamicPoints))).m_pData;
        s_maxPoints = (newMax) as _;
    }

    let idx: usize = (s_cPoints) as _;
    s_cPoints += 1;
    *s_pPoints.add(idx) = POINT { x: x, y: y };
}

unsafe extern "C" fn GetDirection(x0: LONG, y0: LONG, x1: LONG, y1: LONG) -> DIRECTION {
    let dx: LONG = x1 - x0;
    let dy: LONG = y1 - y0;

    if labs(dx) <= 8 && labs(dy) <= 8 {
        return NO_DIRECTION;
    }

    let radian: f64 = atan2(dy as f64, dx as f64);
    if radian < DEG2RAD!(-180 + THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_HORIZONTAL: %ld\n", RAD2DEG!(radian));
        return DIRECTION_HORIZONTAL;
    }
    if radian < DEG2RAD!(-90 - THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_DIAGONAL_RIGHT_DOWN: %ld\n", RAD2DEG!(radian));
        return DIRECTION_DIAGONAL_RIGHT_DOWN;
    }
    if radian < DEG2RAD!(-90 + THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_VERTICAL: %ld\n", RAD2DEG!(radian));
        return DIRECTION_VERTICAL;
    }
    if radian < DEG2RAD!(-THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_DIAGONAL_RIGHT_UP: %ld\n", RAD2DEG!(radian));
        return DIRECTION_DIAGONAL_RIGHT_UP;
    }
    if radian < DEG2RAD!(THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_HORIZONTAL: %ld\n", RAD2DEG!(radian));
        return DIRECTION_HORIZONTAL;
    }
    if radian < DEG2RAD!(90 - THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_DIAGONAL_RIGHT_DOWN: %ld\n", RAD2DEG!(radian));
        return DIRECTION_DIAGONAL_RIGHT_DOWN;
    }
    if radian < DEG2RAD!(90 + THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_VERTICAL: %ld\n", RAD2DEG!(radian));
        return DIRECTION_VERTICAL;
    }
    if radian < DEG2RAD!(180 - THRESHOULD_DEG) {
        ATLTRACE!("DIRECTION_DIAGONAL_RIGHT_UP: %ld\n", RAD2DEG!(radian));
        return DIRECTION_DIAGONAL_RIGHT_UP;
    }
    ATLTRACE!("DIRECTION_HORIZONTAL: %ld\n", RAD2DEG!(radian));
    return DIRECTION_HORIZONTAL;
}

unsafe extern "C" fn RestrictDrawDirection(dir: DIRECTION, x0: LONG, y0: LONG, x1: *mut LONG, y1: *mut LONG) {
    match dir {
        NO_DIRECTION | _ => {
            return;
        }

        DIRECTION_HORIZONTAL => {
            *y1 = y0;
        }

        DIRECTION_VERTICAL => {
            *x1 = x0;
        }

        DIRECTION_DIAGONAL_RIGHT_DOWN => {
            *y1 = y0 + (*x1 - x0);
        }

        DIRECTION_DIAGONAL_RIGHT_UP => {
            *x1 = x0 - (*y1 - y0);
        }
    }
}

impl ToolBase {
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        (*this_).reset();
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn beginEvent(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hdc = imageModel.GetDC();
        (*this_).m_fg = paletteModel.GetFgColor();
        (*this_).m_bg = paletteModel.GetBgColor();
    }
    pub unsafe fn endEvent(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hdc = (NULL).cast();
    }
    pub unsafe fn reset(&mut self) {
        let this_: *mut Self = self;
        if s_pPoints != addr_of_mut!(s_staticPoints) as *mut POINT {
            s_dynamicPoints.Free();
            s_pPoints = addr_of_mut!(s_staticPoints) as *mut POINT;
            s_maxPoints = _countof!(s_staticPoints);
        }
    
        s_cPoints = 0;
        g_ptStart = POINT { x: -1, y: -1 };
        g_ptEnd = g_ptStart;
    
        if selectionModel.m_bShow != 0 {
            selectionModel.Landing();
            selectionModel.HideSelection();
        }
    }
    pub unsafe fn createToolObject(type_: TOOLTYPE) -> *mut ToolBase {
        match type_ {
            TOOL_FREESEL => {
                let p = FreeSelTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_RECTSEL => {
                let p = RectSelTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_RUBBER => {
                let p = RubberTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_FILL => {
                let p = FillTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_COLOR => {
                let p = ColorTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_ZOOM => {
                let p = ZoomTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_PEN => {
                let p = PenTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_BRUSH => {
                let p = BrushTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_AIRBRUSH => {
                let p = AirBrushTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_TEXT => {
                let p = TextTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_LINE => {
                let p = LineTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_BEZIER => {
                let p = BezierTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_RECT => {
                let p = RectTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_SHAPE => {
                let p = ShapeTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0) as *mut ToolBase;
            }
            TOOL_ELLIPSE => {
                let p = EllipseTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            TOOL_RRECT => {
                let p = RRectTool__new_0();
                return core::ptr::addr_of_mut!((*p).__base0.__base0) as *mut ToolBase;
            }
            _ => {}
        }
        UNREACHABLE!();
        core::ptr::null_mut()
    }
}

impl ToolsModel {
    pub unsafe fn OnButtonDown(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).beginEvent();
        g_ptStart = POINT { x: x, y: y };
        g_ptEnd = g_ptStart;
        (*(*this_).m_pToolObject).OnButtonDown(bLeftButton, x, y, bDoubleClick);
        (*(*this_).m_pToolObject).endEvent();
    }
    pub unsafe fn OnMouseMove(&mut self, bLeftButton: BOOL, mut x: LONG, mut y: LONG) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).beginEvent();
        if (*(*this_).m_pToolObject).OnMouseMove(bLeftButton, &mut x, &mut y) != 0 {
            g_ptEnd = POINT { x: x, y: y };
        }
    
        (*(*this_).m_pToolObject).endEvent();
    }
    pub unsafe fn OnButtonUp(&mut self, bLeftButton: BOOL, mut x: LONG, mut y: LONG) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).beginEvent();
        if (*(*this_).m_pToolObject).OnButtonUp(bLeftButton, &mut x, &mut y) != 0 {
            g_ptEnd = POINT { x: x, y: y };
        }
    
        (*(*this_).m_pToolObject).endEvent();
    }
    pub unsafe fn OnEndDraw(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        ATLTRACE!("ToolsModel::OnEndDraw(%d)\n", bCancel);
        (*(*this_).m_pToolObject).beginEvent();
        (*(*this_).m_pToolObject).OnEndDraw(bCancel);
        (*(*this_).m_pToolObject).endEvent();
    }
    pub unsafe fn OnDrawOverlayOnImage(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        ((*(*this_).m_pToolObject).OnDrawOverlayOnImage(hdc));
    }
    pub unsafe fn OnDrawOverlayOnCanvas(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).OnDrawOverlayOnCanvas(hdc);
    }
    pub unsafe fn SpecialTweak(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        (*(*this_).m_pToolObject).OnSpecialTweak(bMinus);
    }
    pub unsafe fn DrawWithMouseTool(&mut self, pt: POINT, wParam: WPARAM) {
        let this_: *mut Self = self;
        let mut xRel: LONG = pt.x - g_ptStart.x;
        let mut yRel: LONG = pt.y - g_ptStart.y;
    
        match (*this_).m_activeTool {
            // freesel, rectsel and text tools always show numbers limited to fit into image area
            TOOL_FREESEL | TOOL_RECTSEL | TOOL_TEXT => {
                if xRel < 0 {
                    xRel = if pt.x < 0 { -g_ptStart.x } else { xRel };
                } else if pt.x > imageModel.GetWidth() {
                    xRel = imageModel.GetWidth() - g_ptStart.x;
                }
                if yRel < 0 {
                    yRel = if pt.y < 0 { -g_ptStart.y } else { yRel };
                } else if pt.y > imageModel.GetHeight() {
                    yRel = imageModel.GetHeight() - g_ptStart.y;
                }
            }
    
            // while drawing, update cursor coordinates only for tools 3, 7, 8, 9, 14
            TOOL_RUBBER | TOOL_PEN | TOOL_BRUSH | TOOL_AIRBRUSH | TOOL_SHAPE => {
                let mut strCoord = CStringW::new();
                atl_format!(strCoord, CPP_COM_WSTR_16A2910FBF5B.as_ptr(), pt.x, pt.y);
                SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 1, strCoord.GetString() as LPARAM);
            }
            _ => {}
        }
    
        // rectsel and shape tools always show non-negative numbers when drawing
        if (*this_).m_activeTool == TOOL_RECTSEL || (*this_).m_activeTool == TOOL_SHAPE {
            xRel = labs(xRel);
            yRel = labs(yRel);
        }
    
        if wParam & (MK_LBUTTON) as _ != 0 {
            (*this_).OnMouseMove((TRUE) as i32, pt.x, pt.y);
            canvasWindow.__base0.Invalidate((FALSE) as BOOL);
            if ((*this_).m_activeTool >= TOOL_TEXT) || (*this_).IsSelection() != 0 {
                let mut strSize = CStringW::new();
                if ((*this_).m_activeTool >= TOOL_LINE) && (GetAsyncKeyState(VK_SHIFT) < 0) {
                    yRel = xRel;
                }
                atl_format!(strSize, CPP_COM_WSTR_1092529DF21F.as_ptr(), xRel, yRel);
                SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 2, strSize.GetString() as LPARAM);
            }
        }
    
        if wParam & (MK_RBUTTON) as _ != 0 {
            (*this_).OnMouseMove((FALSE) as i32, pt.x, pt.y);
            canvasWindow.__base0.Invalidate((FALSE) as BOOL);
            if (*this_).m_activeTool >= TOOL_TEXT {
                let mut strSize = CStringW::new();
                if ((*this_).m_activeTool >= TOOL_LINE) && (GetAsyncKeyState(VK_SHIFT) < 0) {
                    yRel = xRel;
                }
                atl_format!(strSize, CPP_COM_WSTR_1092529DF21F.as_ptr(), xRel, yRel);
                SendMessageW(g_hStatusBar, (SB_SETTEXT) as UINT, 2, strSize.GetString() as LPARAM);
            }
        }
    }
}

impl TwoPointDrawTool {
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
            (*this_).m_bLeftButton = bLeftButton;
            (*this_).m_bDrawing = (TRUE) as i32;
            imageModel.NotifyImageChanged();
        }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
            imageModel.NotifyImageChanged();
            return (TRUE) as BOOL;
        }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        let mut rcPartial: CRect = CRect__make_5(g_ptStart, g_ptEnd);
        rcPartial.NormalizeRect();
        let size: SIZE = toolsModel.GetToolSize();
        rcPartial.InflateRect__0((size.cx + 1) / 2, (size.cy + 1) / 2);
        imageModel.PushImageForUndo__2(&((rcPartial).__base0));
    
        (*this_).__base0.OnDrawOverlayOnImage((*this_).__base0.m_hdc);
        (*this_).m_bDrawing = (FALSE) as i32;
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bDrawing = (FALSE) as i32;
        (*this_).__base0.OnEndDraw(bCancel);
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        (*core::ptr::addr_of_mut!(toolsModel)).MakeLineThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_bLeftButton = (FALSE) as i32;
        (*this_).m_bDrawing = (FALSE) as i32;
    }
}

impl SmoothDrawTool {
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
            (*this_).m_direction = NO_DIRECTION;
            (*this_).m_bShiftDown = (GetKeyState(VK_SHIFT as i32) as i32 & 0x8000); // Is Shift key pressed?
            (*this_).m_bLeftButton = bLeftButton;
            s_cPoints = 0;
            pushToPoints(x, y);
            pushToPoints(x, y); // We have to draw the first point
            imageModel.NotifyImageChanged();
        }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_bShiftDown == 0 {
            pushToPoints(*x, *y);
            imageModel.NotifyImageChanged();
            return (TRUE) as BOOL;
        }
    
        if (*this_).m_direction == NO_DIRECTION {
            (*this_).m_direction = GetDirection(g_ptStart.x, g_ptStart.y, *x, *y);
            if (*this_).m_direction == NO_DIRECTION {
                return (FALSE) as BOOL;
            }
        }
    
        RestrictDrawDirection((*this_).m_direction, g_ptStart.x, g_ptStart.y, x, y);
        pushToPoints(*x, *y);
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
            if (*this_).m_bShiftDown != 0 && (*this_).m_direction != NO_DIRECTION {
                RestrictDrawDirection((*this_).m_direction, g_ptStart.x, g_ptStart.y, x, y);
            }
    
            pushToPoints(*x, *y);
    
            let mut rcPartial: CRect = CRect__make_0();
            getBoundaryOfPoints(&mut rcPartial.__base0, s_cPoints, s_pPoints);
    
            let size: SIZE = toolsModel.GetToolSize();
            rcPartial.InflateRect__0((size.cx + 1) / 2, (size.cy + 1) / 2);
    
            imageModel.PushImageForUndo__2(&rcPartial.__base0);
    
            (*this_).OnDrawOverlayOnImage((*this_).__base0.m_hdc);
            imageModel.NotifyImageChanged();
            (*this_).__base0.OnEndDraw((FALSE) as BOOL);
            return (TRUE) as BOOL;
        }
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        let mut i: SIZE_T = 1;
        while i < s_cPoints {
            (*this_).OnDraw(
                hdc,
                (*this_).m_bLeftButton,
                *s_pPoints.add((i - 1) as usize),
                *s_pPoints.add(i as usize),
            );
            i += 1;
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_direction = NO_DIRECTION;
        (*this_).m_bShiftDown = (FALSE) as i32;
        (*this_).m_bLeftButton = (FALSE) as i32;
    }
}

impl SelectionBaseTool {
    pub unsafe fn isRectSelect(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return (toolsModel.GetActiveTool() == TOOL_RECTSEL) as BOOL;
    }
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if selectionModel.IsLanded() != 0 || selectionModel.m_bShow == 0 {
            return;
        }
    
        if (*this_).m_bNoDrawBack == 0 {
            selectionModel.DrawBackground(hdc, selectionModel.m_rgbBack);
        }
    
        selectionModel.DrawSelection__1(
            hdc,
            paletteModel.GetBgColor(),
            toolsModel.IsBackgroundTransparent(),
        );
    }
    pub unsafe fn OnDrawOverlayOnCanvas__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if (*this_).m_bDrawing != 0 || selectionModel.m_bShow != 0 {
            selectionModel.drawFrameOnCanvas(hdc);
        }
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
            (*this_).m_bLeftButton = bLeftButton;
            (*this_).m_bCtrlKey = ((GetKeyState(VK_CONTROL) < 0) as BOOL);
            (*this_).m_bShiftKey = ((GetKeyState(VK_SHIFT) < 0) as BOOL);
            (*this_).m_bDrawing = (FALSE) as i32;
            (*this_).m_hitSelection = HIT_NONE;
    
            let mut pt = POINT { x: x, y: y };
            if (*this_).m_bLeftButton == 0 // Show context menu on Right-click
            {
                canvasWindow.ImageToCanvas__0(&mut pt);
                canvasWindow.__base0.ClientToScreen__0(&mut pt);
                mainWindow.TrackPopupMenu(pt, 0);
                return;
            }
    
            let mut ptCanvas = pt;
            canvasWindow.ImageToCanvas__0(&mut ptCanvas);
            let hit = selectionModel.hitTest(ptCanvas);
            if hit != HIT_NONE // Dragging of selection started?
            {
                if (*this_).m_bCtrlKey != 0 || (*this_).m_bShiftKey != 0
                {
                    imageModel.PushImageForUndo__0();
                    toolsModel.OnDrawOverlayOnImage(imageModel.GetDC());
                }
                (*this_).m_hitSelection = hit;
                selectionModel.m_ptHit = pt;
                selectionModel.TakeOff();
                (*this_).m_bNoDrawBack |= (((*this_).m_bCtrlKey != 0 || (*this_).m_bShiftKey != 0) as BOOL);
                imageModel.NotifyImageChanged();
                return;
            }
    
            selectionModel.Landing();
            (*this_).m_bDrawing = (TRUE) as i32;
    
            imageModel.Clamp(&mut pt);
            if (*this_).isRectSelect() != 0
            {
                selectionModel.SetRectFromPoints(&g_ptStart, &pt);
            }
            else
            {
                s_cPoints = 0;
                pushToPoints(pt.x, pt.y);
            }
    
            imageModel.NotifyImageChanged();
        }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
            let mut pt: POINT = POINT { x: *x, y: *y };
    
            if (*this_).m_bLeftButton == 0 {
                return (TRUE) as BOOL;
            }
    
            if (*this_).m_hitSelection != HIT_NONE { // Now dragging selection?
                if (*this_).m_bShiftKey != 0 {
                    toolsModel.OnDrawOverlayOnImage(imageModel.GetDC());
                }
    
                selectionModel.Dragging((*this_).m_hitSelection, pt);
                imageModel.NotifyImageChanged();
                return (TRUE) as BOOL;
            }
    
            if (*this_).isRectSelect() != 0 && (GetKeyState(VK_SHIFT) as i32) < 0 {
                regularize(g_ptStart.x, g_ptStart.y, &mut pt.x, &mut pt.y);
            }
    
            imageModel.Clamp(&mut pt);
    
            if (*this_).isRectSelect() != 0 {
                selectionModel.SetRectFromPoints(&g_ptStart, &pt);
            } else {
                pushToPoints(pt.x, pt.y);
            }
    
            imageModel.NotifyImageChanged();
            return (TRUE) as BOOL;
        }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        let mut pt = POINT { x: *x, y: *y };
        (*this_).m_bDrawing = (FALSE) as i32;
    
        if (*this_).m_bLeftButton == 0 {
            return (TRUE) as BOOL;
        }
    
        if (*this_).m_hitSelection != HIT_NONE {
            // Dragging of selection ended?
            if (*this_).m_bShiftKey != 0 {
                toolsModel.OnDrawOverlayOnImage(imageModel.GetDC());
            }
    
            selectionModel.Dragging((*this_).m_hitSelection, pt);
            (*this_).m_hitSelection = HIT_NONE;
            imageModel.NotifyImageChanged();
            return (TRUE) as BOOL;
        }
    
        if (*this_).isRectSelect() != 0 && GetKeyState(VK_SHIFT) < 0 {
            regularize(g_ptStart.x, g_ptStart.y, &mut pt.x, &mut pt.y);
        }
    
        imageModel.Clamp(&mut pt);
    
        if (*this_).isRectSelect() != 0 {
            selectionModel.SetRectFromPoints(&g_ptStart, &pt);
            selectionModel.m_bShow = (selectionModel.m_rc.IsRectEmpty() == 0) as BOOL;
        } else {
            if s_cPoints > 2 {
                BuildMaskFromPoints();
                selectionModel.m_bShow = (TRUE) as i32;
            } else {
                s_cPoints = 0;
                selectionModel.m_bShow = (FALSE) as i32;
            }
        }
    
        (*this_).m_bNoDrawBack = (FALSE) as i32;
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        if bCancel != 0 {
            selectionModel.HideSelection();
        } else {
            selectionModel.Landing();
        }
    
        (*this_).m_bDrawing = (FALSE) as i32;
        (*this_).m_hitSelection = HIT_NONE;
        (*this_).__base0.OnEndDraw(bCancel);
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        selectionModel.StretchSelection(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_bLeftButton = (FALSE) as i32;
        (*this_).m_bCtrlKey = (FALSE) as i32;
        (*this_).m_bShiftKey = (FALSE) as i32;
        (*this_).m_bDrawing = (FALSE) as i32;
        (*this_).m_bNoDrawBack = (FALSE) as i32;
        (*this_).m_hitSelection = HIT_NONE;
    }
}

impl FreeSelTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        (*this_).__base0.OnDrawOverlayOnImage(hdc);
    
        if (!(selectionModel.m_bShow != 0) as i32 != 0) && (*this_).__base0.m_bDrawing != 0 {
            /* Draw the freehand selection inverted/xored */
            Poly__0(hdc, s_pPoints, s_cPoints as i32, (0 as COLORREF), (0 as COLORREF), 2, 0, (FALSE) as BOOL, (TRUE) as BOOL);
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl RectSelTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        (*this_).__base0.OnDrawOverlayOnImage(hdc);
    
        if selectionModel.m_bShow == 0 && (*this_).__base0.m_bDrawing != 0 {
            let rc: &mut CRect = &mut selectionModel.m_rc;
            if (rc.IsRectEmpty()) == 0 {
                RectSel(hdc, rc.__base0.left, rc.__base0.top, rc.__base0.right, rc.__base0.bottom);
            }
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl RubberTool {
    pub unsafe fn OnDraw__body(&mut self, hdc: HDC, bLeftButton: BOOL, pt0: POINT, pt1: POINT) {
        let this_: *mut Self = self;
        if bLeftButton != 0 {
            Erase__1(hdc, pt0.x, pt0.y, pt1.x, pt1.y, toolsModel.GetBgBrush(), toolsModel.GetRubberRadius());
        } else {
            Replace__1(hdc, pt0.x, pt0.y, pt1.x, pt1.y, (*this_).__base0.__base0.m_fg, toolsModel.GetBgBrush(), toolsModel.GetRubberRadius());
        }
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        let _ = this_;
        (*core::ptr::addr_of_mut!(toolsModel)).MakeRubberThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl FillTool {
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        imageModel.PushImageForUndo__0();
        let hBrush: HBRUSH = if bLeftButton != 0 { toolsModel.GetFgBrush() } else { toolsModel.GetBgBrush() };
        Fill__1((*this_).__base0.m_hdc, x, y, hBrush);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl ColorTool {
    pub unsafe fn fetchColor(&mut self, x: LONG, y: LONG) -> COLORREF {
        let this_: *mut Self = self;
        if 0 <= x && x < imageModel.GetWidth() && 0 <= y && y < imageModel.GetHeight() {
            return GetPixel((*this_).__base0.m_hdc, x, y);
        }
        return RGB!(255, 255, 255); // Outside is white
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        let rgbColor: COLORREF = (*this_).fetchColor(x, y);
        toolSettingsWindow.__base0.SendMessageW__0(WM_TOOLSMODELCOLORPICKED, (rgbColor) as WPARAM, 0);
    }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        let rgbColor: COLORREF = (*this_).fetchColor(*x, *y);
        toolSettingsWindow.__base0.SendMessageW__0(WM_TOOLSMODELCOLORPICKED, (rgbColor) as WPARAM, 0);
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        let rgbColor: COLORREF = (*this_).fetchColor(*x, *y);
        if bLeftButton != 0 {
            paletteModel.SetFgColor(rgbColor);
        } else {
            paletteModel.SetBgColor(rgbColor);
        }
    
        toolsModel.SetActiveTool(toolsModel.GetOldActiveTool());
        return (TRUE) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl ZoomTool {
    pub unsafe fn getNewZoomRect(&mut self, rcView: &mut CRect, newZoom: INT) -> BOOL {
        let this_: *mut Self = self;
        let mut pt: CPoint = CPoint__make_0();
        GetCursorPos(core::ptr::addr_of_mut!(pt.__base0));
        canvasWindow.__base0.ScreenToClient(core::ptr::addr_of_mut!(pt.__base0));
    
        canvasWindow.getNewZoomRect(rcView, newZoom, pt);
    
        let mut rc: CRect = CRect__make_0();
        canvasWindow.GetImageRect(core::ptr::addr_of_mut!(rc.__base0));
        canvasWindow.ImageToCanvas__1(core::ptr::addr_of_mut!(rc.__base0));
    
        return rc.PtInRect(pt.__base0);
    }
    pub unsafe fn OnDrawOverlayOnCanvas__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        let mut rcView: CRect = CRect__make_0();
        let oldZoom: i32 = toolsModel.GetZoom();
        if oldZoom < (MAX_ZOOM as i32) && (*this_).getNewZoomRect(&mut rcView, oldZoom * 2) != 0 {
            DrawXorRect(hdc, core::ptr::addr_of_mut!(rcView.__base0));
        }
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        let oldZoom: INT = toolsModel.GetZoom();
        let newZoom: INT;
        if bLeftButton != 0 {
            newZoom = if oldZoom < (MAX_ZOOM as i32) { oldZoom * 2 } else { (MIN_ZOOM as i32) };
        } else {
            newZoom = if oldZoom > (MIN_ZOOM as i32) { oldZoom / 2 } else { (MAX_ZOOM as i32) };
        }
    
        (*this_).m_bZoomed = (FALSE) as i32;
    
        if oldZoom != newZoom {
            let mut rcView: CRect = CRect__make_0();
            if (*this_).getNewZoomRect(&mut rcView, newZoom) != 0 {
                canvasWindow.zoomTo(newZoom, rcView.__base0.left, rcView.__base0.top);
                (*this_).m_bZoomed = (TRUE) as i32;
            }
        }
    }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_bZoomed != 0 {
            toolsModel.SetActiveTool(toolsModel.GetOldActiveTool());
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_bZoomed = (FALSE) as i32;
    }
}

impl PenTool {
    pub unsafe fn OnDraw__body(&mut self, hdc: HDC, bLeftButton: BOOL, pt0: POINT, pt1: POINT) {
        let this_: *mut Self = self;
        let hBrush: HBRUSH = if bLeftButton != 0 {
            toolsModel.GetFgBrush()
        } else {
            toolsModel.GetBgBrush()
        };
        Line__1(hdc, pt0.x, pt0.y, pt1.x, pt1.y, hBrush, toolsModel.GetPenWidth());
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        toolsModel.MakePenThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl BrushTool {
    pub unsafe fn OnDraw__body(&mut self, hdc: HDC, bLeftButton: BOOL, pt0: POINT, pt1: POINT) {
        let this_: *mut Self = self;
            let hBrush: HBRUSH = if bLeftButton != 0 { toolsModel.GetFgBrush() } else { toolsModel.GetBgBrush() };
            Brush__1(hdc, pt0.x, pt0.y, pt1.x, pt1.y, hBrush, toolsModel.GetBrushStyle(),
                  toolsModel.GetBrushWidth());
        }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        toolsModel.MakeBrushThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl AirBrushTool {
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_dwTick = GetTickCount();
        (*this_).__base0.OnButtonDown(bLeftButton, x, y, bDoubleClick);
    }
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        srand((*this_).m_dwTick);
        (*this_).__base0.OnDrawOverlayOnImage(hdc);
    }
    pub unsafe fn OnDraw__body(&mut self, hdc: HDC, bLeftButton: BOOL, pt0: POINT, pt1: POINT) {
        let this_: *mut Self = self;
            let hBrush: HBRUSH = if bLeftButton != 0 { toolsModel.GetFgBrush() } else { toolsModel.GetBgBrush() };
            Airbrush__1(hdc, pt1.x, pt1.y, hBrush, toolsModel.GetAirBrushRadius());
        }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        toolsModel.MakeAirBrushThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_dwTick = 0;
    }
}

impl TextTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if canvasWindow.m_drawing != 0 {
            let rc: *mut CRect = core::ptr::addr_of_mut!(selectionModel.m_rc);
            if (*rc).IsRectEmpty() == 0 {
                RectSel(
                    hdc,
                    (*rc).__base0.left,
                    (*rc).__base0.top,
                    (*rc).__base0.right,
                    (*rc).__base0.bottom,
                );
            }
        }
    }
    pub unsafe fn UpdatePoint(&mut self, x: LONG, y: LONG) {
        let this_: *mut Self = self;
        let mut pt = POINT { x: x, y: y };
        (*addr_of_mut!(imageModel)).Clamp(&mut pt);
        (*addr_of_mut!(selectionModel)).SetRectFromPoints(&*addr_of!(g_ptStart), &pt);
        (*addr_of_mut!(imageModel)).NotifyImageChanged();
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        if !textEditWindow.__base0.IsWindow() != 0 {
            textEditWindow.Create(canvasWindow.op_to_HWND());
        }
    
        (*this_).UpdatePoint(x, y);
    }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
            (*this_).UpdatePoint(*x, *y);
            return (TRUE) as BOOL;
        }
    pub unsafe fn draw(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        let mut szText: CStringW = CStringW::new();
        textEditWindow.__base0.GetWindowTextW__1(&mut szText);
    
        let mut rc: CRect = CRect__make_0();
        textEditWindow.InvalidateEditRect();
        textEditWindow.GetEditRect(core::ptr::addr_of_mut!(rc.__base0));
        rc.InflateRect__0((-GRIP_SIZE / 2) as i32, (-GRIP_SIZE / 2) as i32);
    
        // Draw the text
        let style: INT = if ((toolsModel.IsBackgroundTransparent()) != 0) { 0 } else { 1 };
        Text__1(hdc, rc.__base0.left, rc.__base0.top, rc.__base0.right, rc.__base0.bottom, toolsModel.GetFgBrush(), toolsModel.GetBgBrush(), szText.GetString(),
             textEditWindow.GetFont(), style);
    }
    pub unsafe fn quit(&mut self) {
        let this_: *mut Self = self;
        if textEditWindow.__base0.IsWindow() != 0 {
            textEditWindow.__base0.ShowWindow((SW_HIDE) as i32);
        }
        selectionModel.HideSelection();
    }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        let mut pt: POINT = POINT { x: *x, y: *y };
        imageModel.Clamp(&mut pt);
        selectionModel.SetRectFromPoints(&g_ptStart, &pt);
    
        let bTextBoxShown: BOOL = IsWindowVisible(textEditWindow.op_to_HWND());
        if bTextBoxShown != 0 {
            if textEditWindow.__base0.GetWindowTextLengthW() > 0 {
                imageModel.PushImageForUndo__0();
                (*this_).draw((*this_).__base0.m_hdc);
            }
            if selectionModel.m_rc.IsRectEmpty() != 0 {
                (*this_).quit();
                return (TRUE) as BOOL;
            }
        }
    
        if registrySettings.ShowTextTool != 0 {
            if fontsDialog.__base0.IsWindow() == 0 {
                fontsDialog.Create({ let _ = mainWindow; 0 }.op_to_HWND());
            }
    
            fontsDialog.__base0.ShowWindow((SW_SHOWNOACTIVATE) as i32);
        }
    
        // CRect rc = COMMENT_TOKEN_ECHO____() = selectionModel.m_rc;
        let mut rc: CRect = {
            let slot: *mut CRect = CRect__make_0();
            *slot = core::ptr::read(core::ptr::addr_of!(selectionModel.m_rc));
            core::ptr::read(slot)
        };
    
        // Enlarge if tool small
        let mut cxMin: INT = (CX_MINTEXTEDIT as i32);
        let mut cyMin: INT = (CY_MINTEXTEDIT as i32);
        if selectionModel.m_rc.IsRectEmpty() != 0 {
            rc.SetRect(*x, *y, *x + cxMin, *y + cyMin);
        } else {
            if rc.__base0.right - rc.__base0.left < cxMin {
                rc.__base0.right = rc.__base0.left + cxMin;
            }
            if rc.__base0.bottom - rc.__base0.top < cyMin {
                rc.__base0.bottom = rc.__base0.top + cyMin;
            }
        }
    
        if textEditWindow.__base0.IsWindow() == 0 {
            textEditWindow.Create(canvasWindow.op_to_HWND());
        }
    
        textEditWindow.__base0.SetWindowTextW(core::ptr::null_mut());
        textEditWindow.ValidateEditRect(&((*core::ptr::addr_of_mut!(rc)).__base0));
        textEditWindow.__base0.ShowWindow((SW_SHOWNOACTIVATE) as i32);
        textEditWindow.__base0.SetFocus();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        if bCancel == 0 {
            if IsWindowVisible(textEditWindow.op_to_HWND()) != 0
                && textEditWindow.__base0.GetWindowTextLengthW() > 0
            {
                imageModel.PushImageForUndo__0();
                (*this_).draw((*this_).__base0.m_hdc);
            }
        }
        (*this_).quit();
        (*this_).__base0.OnEndDraw(bCancel);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl LineTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_bDrawing == 0 {
            return;
        }
        if (GetAsyncKeyState(VK_SHIFT) as i32) < 0 {
            roundTo8Directions(g_ptStart.x, g_ptStart.y, &mut g_ptEnd.x, &mut g_ptEnd.y);
        }
        let hBrush: HBRUSH = if (*this_).__base0.m_bLeftButton != 0 {
            toolsModel.GetFgBrush()
        } else {
            toolsModel.GetBgBrush()
        };
        Line__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, hBrush, toolsModel.GetLineWidth());
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl BezierTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        let hBrush: HBRUSH = if (*this_).m_bLeftButton != 0 {
            toolsModel.GetFgBrush()
        } else {
            toolsModel.GetBgBrush()
        };
        match s_cPoints {
            2 => {
                Line__1(hdc, (*s_pPoints.add(0)).x, (*s_pPoints.add(0)).y, (*s_pPoints.add(1)).x, (*s_pPoints.add(1)).y, hBrush, toolsModel.GetLineWidth());
            }
            3 => {
                Bezier__1(hdc, *s_pPoints.add(0), *s_pPoints.add(2), *s_pPoints.add(2), *s_pPoints.add(1), hBrush, toolsModel.GetLineWidth());
            }
            4 => {
                Bezier__1(hdc, *s_pPoints.add(0), *s_pPoints.add(2), *s_pPoints.add(3), *s_pPoints.add(1), hBrush, toolsModel.GetLineWidth());
            }
            _ => {}
        }
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, x: LONG, y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
            (*this_).m_bLeftButton = bLeftButton;
    
            if s_cPoints == 0 {
                pushToPoints(x, y);
                pushToPoints(x, y);
            } else {
                (*s_pPoints.offset((s_cPoints - 1) as isize)) = POINT { x: x, y: y };
            }
    
            imageModel.NotifyImageChanged();
        }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
            if s_cPoints > 0 {
                *s_pPoints.add((s_cPoints - 1) as usize) = POINT { x: *x, y: *y };
            }
            imageModel.NotifyImageChanged();
            return (TRUE) as BOOL;
        }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        if s_cPoints >= 4 {
            (*this_).OnEndDraw((FALSE) as BOOL);
            return (TRUE) as BOOL;
        }
        pushToPoints(*x, *y);
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        if bCancel == 0 && s_cPoints > 1 {
            // FIXME: I couldn't calculate boundary rectangle from Bezier curve
            imageModel.PushImageForUndo__0();
            (*this_).OnDrawOverlayOnImage((*this_).__base0.m_hdc);
        }
        (*this_).__base0.OnEndDraw(bCancel);
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        toolsModel.MakeLineThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_bLeftButton = (FALSE) as i32;
    }
}

impl RectTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_bDrawing == 0 {
            return;
        }
        if GetAsyncKeyState(VK_SHIFT) < 0 {
            regularize(
                g_ptStart.x,
                g_ptStart.y,
                core::ptr::addr_of_mut!(g_ptEnd.x),
                core::ptr::addr_of_mut!(g_ptEnd.y),
            );
        }
        if (*this_).__base0.m_bLeftButton != 0 {
            Rect__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetFgBrush(), toolsModel.GetBgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
        } else {
            Rect__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetBgBrush(), toolsModel.GetFgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl ShapeTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if s_cPoints <= 0 {
            return;
        }
    
        if (*this_).m_bLeftButton != 0 {
            Poly__1(hdc, s_pPoints, s_cPoints as i32, toolsModel.GetFgBrush(), toolsModel.GetBgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle(), (*this_).m_bClosed, (FALSE) as BOOL);
        } else {
            Poly__1(hdc, s_pPoints, s_cPoints as i32, toolsModel.GetBgBrush(), toolsModel.GetFgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle(), (*this_).m_bClosed, (FALSE) as BOOL);
        }
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL, mut x: LONG, mut y: LONG, bDoubleClick: BOOL) {
        let this_: *mut Self = self;
        (*this_).m_bLeftButton = bLeftButton;
        (*this_).m_bClosed = (FALSE) as i32;
    
        if (s_cPoints > 0) && ((GetAsyncKeyState(VK_SHIFT) as i32) < 0) {
            roundTo8Directions((*s_pPoints.add((s_cPoints - 1) as _)).x, (*s_pPoints.add((s_cPoints - 1) as _)).y, &mut x, &mut y);
        }
    
        pushToPoints(x, y);
    
        if s_cPoints > 1 && bDoubleClick != 0 {
            (*this_).OnEndDraw((FALSE) as BOOL);
            return;
        }
    
        if s_cPoints == 1 {
            pushToPoints(x, y); // We have to draw the first point
        }
    
        imageModel.NotifyImageChanged();
    }
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        if s_cPoints > 1 {
            if (GetAsyncKeyState(VK_SHIFT) as i32) < 0 {
                roundTo8Directions(
                    (*s_pPoints.add((s_cPoints - 2) as usize)).x,
                    (*s_pPoints.add((s_cPoints - 2) as usize)).y,
                    x,
                    y,
                );
            }
    
            (*s_pPoints.add((s_cPoints - 1) as usize)) = POINT { x: *x, y: *y };
        }
    
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self;
        if s_cPoints > 1 && GetAsyncKeyState(VK_SHIFT) < 0 {
            roundTo8Directions(
                (*s_pPoints.add((s_cPoints - 2) as usize)).x,
                (*s_pPoints.add((s_cPoints - 2) as usize)).y,
                x,
                y,
            );
        }
    
        (*this_).m_bClosed = (FALSE) as i32;
        if nearlyEqualPoints(
            *x,
            *y,
            (*s_pPoints.add(0 as usize)).x,
            (*s_pPoints.add(0 as usize)).y,
        ) != 0
        {
            (*this_).OnEndDraw((FALSE) as BOOL);
            return (TRUE) as BOOL;
        }
    
        pushToPoints(*x, *y);
        imageModel.NotifyImageChanged();
        return (TRUE) as BOOL;
    }
    pub unsafe fn OnEndDraw__body(&mut self, bCancel: BOOL) {
        let this_: *mut Self = self;
        if bCancel == 0 && s_cPoints > 1 {
            let mut rcPartial: CRect = CRect__make_0();
            getBoundaryOfPoints(&mut rcPartial.__base0, s_cPoints, s_pPoints);
    
            let size: SIZE = toolsModel.GetToolSize();
            rcPartial.InflateRect__0(
                (size.cx + 1) / 2,
                (size.cy + 1) / 2,
            );
    
            imageModel.PushImageForUndo__2(&mut rcPartial.__base0);
    
            (*this_).m_bClosed = (TRUE) as i32;
            (*this_).OnDrawOverlayOnImage((*this_).__base0.m_hdc);
        }
        (*this_).m_bClosed = (FALSE) as i32;
        (*this_).__base0.OnEndDraw(bCancel);
    }
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;
        toolsModel.MakeLineThickerOrThinner(bMinus);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_bLeftButton = (FALSE) as i32;
        (*this_).m_bClosed = (FALSE) as i32;
    }
}

impl EllipseTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        if (*this_).__base0.m_bDrawing == 0 {
            return;
        }
        if (GetAsyncKeyState(VK_SHIFT) as i32) < 0 {
            regularize(g_ptStart.x, g_ptStart.y, &mut g_ptEnd.x, &mut g_ptEnd.y);
        }
        if (*this_).__base0.m_bLeftButton != 0 {
            Ellp__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetFgBrush(), toolsModel.GetBgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
        } else {
            Ellp__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetBgBrush(), toolsModel.GetFgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
        }
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}

impl RRectTool {
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
            if (((*this_).__base0.m_bDrawing) == 0) {
                return;
            }
            if (GetAsyncKeyState(VK_SHIFT) as i32) < 0 {
                regularize(g_ptStart.x, g_ptStart.y, &mut g_ptEnd.x, &mut g_ptEnd.y);
            }
            if (((*this_).__base0.m_bLeftButton) != 0) {
                RRect__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetFgBrush(), toolsModel.GetBgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
            } else {
                RRect__1(hdc, g_ptStart.x, g_ptStart.y, g_ptEnd.x, g_ptEnd.y, toolsModel.GetBgBrush(), toolsModel.GetFgBrush(), toolsModel.GetLineWidth(), toolsModel.GetShapeStyle());
            }
        }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
    }
}
