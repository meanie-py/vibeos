use crate::prelude::*;
use core::ffi::c_void;




impl audio_receiver {
    pub unsafe fn __ctor_0(&mut self, ) {
            let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).bytes_received = 0;
        {
        }
    }
    pub unsafe fn set_b_received(&mut self,  r: u32) {
            let this_: *mut Self = self;
        (*this_).bytes_received = r;
    }
}
