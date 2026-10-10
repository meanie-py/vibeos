use crate::prelude::*;
use core::ffi::c_void;













pub unsafe extern "C" fn Zoomed__0(xy: i32) -> i32 {
    MulDiv(xy, toolsModel.GetZoom(), (DEFAULT_ZOOM as i32))
}

pub unsafe extern "C" fn UnZoomed__0(xy: i32, bRound: BOOL) -> i32 {
    if bRound != 0 {
        return MulDiv(xy, (DEFAULT_ZOOM as i32), toolsModel.GetZoom());
    }
    return xy * (DEFAULT_ZOOM as i32) / toolsModel.GetZoom();
}

pub unsafe extern "C" fn Zoomed__1(pt: *mut POINT) {
    *pt = POINT { x: Zoomed__0((*pt).x), y: Zoomed__0((*pt).y) };
}

pub unsafe extern "C" fn Zoomed__2(rc: *mut RECT) {
    let __tmp = RECT {
        left: Zoomed__0((*rc).left),
        top: Zoomed__0((*rc).top),
        right: Zoomed__0((*rc).right),
        bottom: Zoomed__0((*rc).bottom),
    };
    *rc = __tmp;
}

pub unsafe extern "C" fn UnZoomed__1(pt: *mut POINT, bRound: BOOL) {
    let x = UnZoomed__0((*pt).x, bRound);
    let y = UnZoomed__0((*pt).y, bRound);
    *pt = POINT { x: x, y: y };
}
pub unsafe extern "C" fn UnZoomed__2(rc: *mut RECT, bRound: BOOL) {
    *rc = RECT {
        left: UnZoomed__0((*rc).left, bRound),
        top: UnZoomed__0((*rc).top, bRound),
        right: UnZoomed__0((*rc).right, bRound),
        bottom: UnZoomed__0((*rc).bottom, bRound),
    };
}

impl ToolBase {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hdc = core::ptr::null_mut();
        {}
    }
    pub unsafe fn OnButtonDown__body(&mut self, bLeftButton: BOOL,
        x: LONG,
        y: LONG,
        bDoubleClick: BOOL,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnMouseMove__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self; return (TRUE) as BOOL; }
    pub unsafe fn OnButtonUp__body(&mut self, bLeftButton: BOOL, x: &mut LONG, y: &mut LONG) -> BOOL {
        let this_: *mut Self = self; return (TRUE) as BOOL; }
    pub unsafe fn OnDrawOverlayOnImage__body(&mut self, hdc: HDC) {
        let this_: *mut Self = self;}
    pub unsafe fn OnDrawOverlayOnCanvas__body(&mut self, hdc: HDC,
    ) {
        let this_: *mut Self = self;}
    pub unsafe fn OnSpecialTweak__body(&mut self, bMinus: BOOL) {
        let this_: *mut Self = self;}
}

impl VirtualBrush {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_rgbColor = RGB!(0, 0, 0);
        (*this_).m_palette = PAL_MODERN;
        (*this_).m_hBrush = core::ptr::null_mut();
    }
}
