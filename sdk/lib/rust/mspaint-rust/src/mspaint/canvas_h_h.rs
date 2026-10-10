use crate::prelude::*;
use core::ffi::c_void;





impl CStyledCursor {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hCursor.is_null() {
            DestroyCursor((*this_).m_hCursor);
        }
    }
    pub unsafe fn SetCursor(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_hCursor.is_null() {
            SetCursor((*this_).m_hCursor);
        }
    }
    pub unsafe fn op_to_HICON(&mut self) -> HCURSOR {
        let this_: *mut Self = self;
        return (*this_).m_hCursor;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hCursor = core::ptr::null_mut();
        (*this_).m_zoom = -1;
        (*this_).m_radius = -1;
        (*this_).m_color = CLR_INVALID;
        (*this_).m_is_rubber = (FALSE) as i32;
    }
}
