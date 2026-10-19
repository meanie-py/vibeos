use crate::prelude::*;
use core::ffi::c_void;






impl CFontInfo {
    pub unsafe fn IsMarkDeleted(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_bMarkDeleted
    }
    pub unsafe fn MarkDeleted(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_bMarkDeleted = (TRUE as i32);
    }
}

impl CFontCache {
    pub unsafe fn FontPath(&mut self) -> &'static CStringW {
        let this_: *mut Self = self;
        return &(*this_).m_FontFolderPath;
    }
}
