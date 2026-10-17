use crate::prelude::*;
use core::ffi::c_void;





impl CActionWnd {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn OnCreate(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return 0;
    }
    pub unsafe fn OnDestroy(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        return 0;
    }
    pub unsafe fn OnPaint(&mut self, nMessage: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let hdc: HDC = (*this_).__base0.BeginPaint(&mut ps);

        FillRect(hdc, core::ptr::addr_of_mut!(ps.rcPaint), (COLOR_WINDOW + 1) as HBRUSH);

        (*this_).__base0.EndPaint(&mut ps);

        return 0;
    }
}
