use crate::prelude::*;
use core::ffi::c_void;







impl FontPidlEntry {
    pub unsafe fn Name__0(&mut self) -> LPWSTR {
        let this_: *mut Self = self;
        return ((this_ as PBYTE).add((*this_).ibName as usize)) as LPWSTR;
    }
    pub unsafe fn FileName__0(&mut self) -> LPWSTR {
        let this_: *mut Self = self;
        return ((this_ as PBYTE).add((*this_).ibFileName as usize)) as LPWSTR;
    }
    pub unsafe fn Name__1(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        return ((this_ as PBYTE).add((*this_).ibName as usize)) as LPCWSTR;
    }
    pub unsafe fn FileName__1(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        return ((this_ as PBYTE).add((*this_).ibFileName as usize)) as LPCWSTR;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
