use crate::prelude::*;
use core::ffi::c_void;








impl Brush {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        GdipDeleteBrush((*this_).nativeBrush);
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, brush: *mut GpBrush, status: Status) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).nativeBrush = brush;
        (*this_).lastStatus = status;
        {
        }
    }
    pub unsafe fn SetNativeBrush(&mut self, brush: *mut Gdiplus_GpBrush) {
        let this_: *mut Self = self;
        (*this_).nativeBrush = brush;
    }
}

impl SolidBrush {
    pub unsafe fn __ctor_0(&mut self, color: &Color) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
            let mut brush: *mut Gdiplus_GpSolidFill = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateSolidFill(color.GetValue(), core::ptr::addr_of_mut!(brush));
            (*this_).__base0.SetNativeBrush((brush).cast());
        }
    }
    pub unsafe fn __ctor_1(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
}
