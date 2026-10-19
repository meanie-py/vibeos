use crate::prelude::*;
use core::ffi::c_void;




impl ZipPidlEntry {
    pub unsafe fn IsDirectory(&mut self) -> bool {
        let this_: *mut Self = self;
        return (*this_).ZipType == ZIP_PIDL_DIRECTORY;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
