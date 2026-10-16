use crate::prelude::*;
use core::ffi::c_void;


impl GdiplusStartupInput {
    pub unsafe fn __ctor_0(&mut self, debugEventCallback: Gdiplus_DebugEventProc, suppressBackgroundThread: BOOL, suppressExternalCodecs: BOOL) {
        let this_: *mut Self = self;

        {
            (*this_).GdiplusVersion = 1;
            (*this_).DebugEventCallback = debugEventCallback;
            (*this_).SuppressBackgroundThread = suppressBackgroundThread;
            (*this_).SuppressExternalCodecs = suppressExternalCodecs;
        }
    }
}
