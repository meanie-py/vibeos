use crate::prelude::*;
use core::ffi::c_void;


impl CSysTray {
    pub unsafe fn GetHWnd(&mut self) -> HWND {
        let this_: *mut Self = self;
        return (*this_).__base2.m_hWnd;
    }
}
