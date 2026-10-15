use crate::prelude::*;
use core::ffi::c_void;


impl CUpdateDatabaseMutex {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(UPDATEDBMUTEX.as_ptr() as *const u16, 1000 * 60 * 10, (FALSE as i32));
        {}
    }
}
