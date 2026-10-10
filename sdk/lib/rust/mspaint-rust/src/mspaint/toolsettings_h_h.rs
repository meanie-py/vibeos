use crate::prelude::*;
use core::ffi::c_void;


impl CToolSettingsWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_rgbPickColor = CLR_INVALID;
    }
}
