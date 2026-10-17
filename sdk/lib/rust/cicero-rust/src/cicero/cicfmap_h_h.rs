use crate::prelude::*;
use core::ffi::c_void;




impl CicFileMappingStatic {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CicFileMapping {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.Finalize();
    }
}
