use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn IsWindow(hWnd: HWND) -> BOOL;
}








impl CWindow {
    pub unsafe fn __ctor_0(&mut self, hWnd: HWND) {
        let this_: *mut Self = self;
        {
            (*this_).m_hWnd = hWnd;
        }
    }
    pub unsafe fn DestroyWindow(&mut self) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);

        if DestroyWindow((*this_).m_hWnd) == 0 {
            return (FALSE as i32);
        }

        (*this_).m_hWnd = core::ptr::null_mut();
        return (TRUE as i32);
    }
    pub unsafe fn KillTimer(&mut self, nIDEvent: UINT_PTR) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd));
        return KillTimer((*this_).m_hWnd, nIDEvent);
    }
    pub unsafe fn SendMessageW__0(&mut self, message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
        let this_: *mut Self = self;
        ASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SendMessage((*this_).m_hWnd, message, wParam, lParam);
    }
    pub unsafe fn SetTimer(&mut self, nIDEvent: UINT_PTR, nElapse: UINT, lpfnTimer: Option<unsafe extern "system" fn(HWND, UINT, UINT_PTR, DWORD)>) -> UINT_PTR {
        let this_: *mut Self = self;
        ATLASSERT!(IsWindow((*this_).m_hWnd) != 0);
        return SetTimer((*this_).m_hWnd, nIDEvent, nElapse, core::mem::transmute::<_, TIMERPROC>(lpfnTimer));
    }
}

impl _IDHWNDPAIR {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
