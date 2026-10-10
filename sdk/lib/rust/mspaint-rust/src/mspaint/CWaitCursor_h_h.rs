use crate::prelude::*;
use core::ffi::c_void;



impl CWaitCursor {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        {
            if {
                let __old = CWaitCursor__s_nLock;
                CWaitCursor__s_nLock += 1;
                __old
            } == 0
            {
                if CWaitCursor__s_hWaitCursor.is_null() {
                    CWaitCursor__s_hWaitCursor = LoadCursor(core::ptr::null_mut(), IDC_WAIT);
                }
                CWaitCursor__s_hOldCursor = SetCursor(CWaitCursor__s_hWaitCursor);
            } else {
                SetCursor(CWaitCursor__s_hWaitCursor);
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        CWaitCursor__s_nLock -= 1;
        if CWaitCursor__s_nLock == 0 {
            SetCursor(CWaitCursor__s_hOldCursor);
            CWaitCursor__s_hOldCursor = core::ptr::null_mut();
        }
    }
}
