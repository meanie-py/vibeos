use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn ExtraPageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let _ = lParam;
    let _ = wParam;
    let _ = hwndDlg;

    match uMsg {
        WM_INITDIALOG => {}
        WM_COMMAND => {}
        _ => {}
    }

    return FALSE as INT_PTR;
}
