use crate::prelude::*;
use core::ffi::c_void;











impl CSnapinAlias {
    pub unsafe fn __ctor_0(&mut self, ParentAlias: *mut CSnapinAlias, Snapin: *mut CSnapin) {
        let this_: *mut Self = self;
        {
        (*this_).m_ParentAlias = ParentAlias;
        (*this_).m_Snapin = Snapin;
        (*this_).m_Deleted = (FALSE as i32);
    }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_ParentAlias = core::ptr::null_mut();
        (*this_).m_Snapin = core::ptr::null_mut();
    }
    pub unsafe fn Parent(&mut self) -> *mut CSnapinAlias {
        let this_: *mut Self = self;
        (*this_).m_ParentAlias
    }
    pub unsafe fn Snapin(&mut self) -> *mut CSnapin {
        let this_: *mut Self = self;
        (*this_).m_Snapin
    }
    pub unsafe fn IsDeleted(&mut self) -> BOOL {
        let this_: *mut Self = self;
        (*this_).m_Deleted
    }
    pub unsafe fn Deleted(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_Deleted = (TRUE as i32);
    }
}
