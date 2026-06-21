//! windows/dc.c → Rust. user32 DC release/lookup client.

use crate::ntwrapper::*;
use win32k_sys::win32k::*;

extern "system" {
    fn GdiReleaseDC(hdc: HDC) -> i32; // gdi32 export
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT {
    if hDC.is_null() {
        return 0;
    }

    GdiReleaseDC(hDC);

    NtUserxReleaseDC(hDC)
}

#[no_mangle]
pub unsafe extern "system" fn WindowFromDC(hDC: HDC) -> HWND {
    NtUserxWindowFromDC(hDC)
}
