use crate::prelude::*;
use core::ffi::c_void;


impl CicFileMapping {
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.Finalize();
    }
}
