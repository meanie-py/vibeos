use crate::prelude::*;
use core::ffi::c_void;


impl CAboutSnapinDialog {
    pub unsafe fn __ctor_0(&mut self, MainWnd: *mut CMainWnd, Snapin: *mut CSnapin) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_SnapinIcon.__ctor_0(core::ptr::null_mut());
        (*this_).m_SnapinInfo.__ctor_0(core::ptr::null_mut());
        (*this_).m_Description.__ctor_0(core::ptr::null_mut());
        {
            (*this_).m_MainWnd = MainWnd;
            (*this_).m_Snapin = Snapin;
        }
    }
}
