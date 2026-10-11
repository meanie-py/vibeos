use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn HelpPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    match message {
        WM_INITDIALOG => {
            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDC_BUTTON_SYSINFO => {}
                IDC_BUTTON_DDRAW_REFRESH => {}
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}
