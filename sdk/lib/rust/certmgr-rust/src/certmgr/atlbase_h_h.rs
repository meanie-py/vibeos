use crate::prelude::*;
use core::ffi::c_void;







impl CRegKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hKey = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, key: &mut CRegKey) {
        let this_: *mut Self = self;
        (*this_).m_hKey = key.Detach();
        {
        }
    }
    pub unsafe fn __ctor_2(&mut self, hKey: HKEY) {
        let this_: *mut Self = self;
        (*this_).m_hKey = hKey;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Close();
    }
    pub unsafe fn Close(&mut self) -> LONG {
        let this_: *mut Self = self;
        if (*this_).m_hKey as usize != 0 {
            let hKey: HKEY = (*this_).Detach();
            return RegCloseKey(hKey);
        }
        return (ERROR_SUCCESS) as LONG;
    }
    pub unsafe fn Detach(&mut self) -> HKEY {
        let this_: *mut Self = self;
        let hKey: HKEY = (*this_).m_hKey;
        (*this_).m_hKey = core::ptr::null_mut();
        return hKey;
    }
}
