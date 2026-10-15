use crate::prelude::*;
use core::ffi::c_void;












impl Graphics {
    pub unsafe fn __ctor_0(&mut self, image: *mut Image) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        {
            let mut graphics: *mut GpGraphics = core::ptr::null_mut();
            if !image.is_null() {
                (*this_).lastStatus = GdipGetImageGraphicsContext(getNat(image), core::ptr::addr_of_mut!(graphics));
            }
            (*this_).SetNativeGraphics(graphics);
        }
    }
    pub unsafe fn __ctor_1(&mut self, hdc: HDC) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        {
            let mut graphics: *mut GpGraphics = core::ptr::null_mut();
            (*this_).lastStatus = GdipCreateFromHDC(hdc, core::ptr::addr_of_mut!(graphics));
            (*this_).SetNativeGraphics(graphics);
        }
    }
    pub unsafe fn __ctor_2(&mut self, hdc: HDC, hdevice: HANDLE) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        {
            let mut graphics: *mut Gdiplus_GpGraphics = core::ptr::null_mut();
            (*this_).lastStatus = GdipCreateFromHDC2(hdc, hdevice, core::ptr::addr_of_mut!(graphics));
            (*this_).SetNativeGraphics(graphics);
        }
    }
    pub unsafe fn __ctor_3(&mut self, hwnd: HWND, icm: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        {
            let mut graphics: *mut GpGraphics = core::ptr::null_mut();
            if icm != 0 {
                (*this_).lastStatus = GdipCreateFromHWNDICM(hwnd, core::ptr::addr_of_mut!(graphics));
            } else {
                (*this_).lastStatus = GdipCreateFromHWND(hwnd, core::ptr::addr_of_mut!(graphics));
            }
            (*this_).SetNativeGraphics(graphics);
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        GdipDeleteGraphics((*this_).nativeGraphics);
    }
    pub unsafe fn DrawImage__12(&mut self, image: *mut Image, x: REAL, y: REAL, width: REAL, height: REAL) -> Status {
        let this_: *mut Self = self;
        return (*this_).SetStatus(
            GdipDrawImageRect((*this_).nativeGraphics, if !image.is_null() { getNat(image) } else { core::ptr::null_mut() }, x, y, width, height));
    }
    pub unsafe fn FillEllipse__1(&mut self, brush: *const Brush, x: REAL, y: REAL, width: REAL, height: REAL) -> Status {
        let this_: *mut Self = self;
        return (*this_).SetStatus(GdipFillEllipse((*this_).nativeGraphics, if !brush.is_null() { getNat(brush) } else { core::ptr::null_mut() }, x, y, width, height));
    }
    pub unsafe fn SetSmoothingMode(&mut self, smoothingMode: SmoothingMode) -> Status {
        let this_: *mut Self = self;
            return (*this_).SetStatus(GdipSetSmoothingMode((*this_).nativeGraphics, smoothingMode));
        }
    pub unsafe fn SetStatus(&mut self, status: Status) -> Status {
        let this_: *mut Self = self;
        if status != Ok {
            (*this_).lastStatus = status;
        }
        return status;
    }
    pub unsafe fn SetNativeGraphics(&mut self, graphics: *mut Gdiplus_GpGraphics,
    ) {
        let this_: *mut Self = self;
        (*this_).nativeGraphics = graphics;
    }
}
