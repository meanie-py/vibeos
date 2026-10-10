use crate::prelude::*;
use core::ffi::c_void;




impl IMAGE_PART {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hbmImage = core::ptr::null_mut();
        (*this_).m_bPartial = (FALSE as i32);
    }
}

impl ImageModel {
    pub unsafe fn CanUndo(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_undoSteps > 0 { 1 } else { 0 }
    }
    pub unsafe fn CanRedo(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((*this_).m_redoSteps > 0) as BOOL;
    }
}
