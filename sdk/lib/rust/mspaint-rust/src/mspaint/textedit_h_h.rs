use crate::prelude::*;
use core::ffi::c_void;



impl CTextEditWindow {
    pub unsafe fn GetFont(&mut self) -> HFONT {
        let this_: *mut Self = self;
        (*this_).m_hFont
    }
}
