use crate::prelude::*;
use core::ffi::c_void;


impl CUpdateDatabaseMutex {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(UPDATEDBMUTEX, 1000 * 60 * 10, (FALSE as i32));
        {}
    }
}
