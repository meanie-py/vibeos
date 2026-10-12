use crate::prelude::*;
use core::ffi::c_void;




impl CRecentFileEntry {
    pub unsafe fn FileName(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_FileName;
    }
    pub unsafe fn DisplayName(&mut self) -> &CAtlString {
        let this_: *mut Self = self;
        return &(*this_).m_DisplayName;
    }
}
