use crate::prelude::*;
use core::ffi::c_void;


impl CMainWindow {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).m_hMenu = core::ptr::null_mut();
        {}
    }
}
