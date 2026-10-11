use crate::prelude::*;
use core::ffi::c_void;


impl CicCaret {
    pub unsafe fn op_to_HWND(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).m_hWnd;
    }
}
