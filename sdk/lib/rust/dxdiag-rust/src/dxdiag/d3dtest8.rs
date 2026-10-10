use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn D3D8Test(lpDevice: *mut GUID, hWnd: HWND) -> BOOL {
    (FALSE) as i32
}
