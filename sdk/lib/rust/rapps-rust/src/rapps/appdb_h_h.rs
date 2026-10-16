use crate::prelude::*;
use core::ffi::c_void;



impl CAppDB {
    pub unsafe fn FindByPackageName(&mut self, name: &CStringW) -> *mut CAppInfo {
        let this_: *mut Self = self;
        let p: *mut CAvailableApplicationInfo = (*this_).FindAvailableByPackageName(name);
        return core::ptr::addr_of_mut!((*p).__base0) as *mut CAppInfo;
    }
    pub unsafe fn GetAvailableCount(&mut self) -> size_t {
        let this_: *mut Self = self;
        return (*this_).m_Available.GetCount();
    }
}
