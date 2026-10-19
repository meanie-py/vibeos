use crate::prelude::*;
use core::ffi::c_void;


impl CFontFolderViewCB {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pFontExt = core::ptr::null_mut();
        (*this_).m_hwndView = core::ptr::null_mut();
        {}
    }
}
