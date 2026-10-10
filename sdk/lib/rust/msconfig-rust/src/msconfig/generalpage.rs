use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hGeneralPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hGeneralDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;
#[no_mangle]
pub unsafe extern "C" fn EnableCheckboxControls(hDlg: HWND, bEnable: BOOL) {
    EnableWindow(GetDlgItem(hDlg, (IDC_CBX_SYSTEM_INI as i32)), bEnable);
    EnableWindow(GetDlgItem(hDlg, (IDC_CBX_SYSTEM_SERVICE as i32)), bEnable);
    EnableWindow(GetDlgItem(hDlg, (IDC_CBX_STARTUP_ITEM as i32)), bEnable);
}
#[no_mangle]
pub unsafe extern "system" fn GeneralPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);

    match message {
        WM_INITDIALOG => {
            hGeneralDialog = hDlg;
            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);
            /* FIXME */
            SendDlgItemMessage(hDlg, (IDC_CBX_NORMAL_START as i32), BM_SETCHECK, (BST_CHECKED) as _, 0);
            EnableCheckboxControls(hDlg, (FALSE) as BOOL);
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_CBX_NORMAL_START | IDC_CBX_DIAGNOSTIC_START => {
                    EnableCheckboxControls(hDlg, (FALSE) as BOOL);
                }
                IDC_CBX_SELECTIVE_STARTUP => {
                    EnableCheckboxControls(hDlg, (TRUE) as BOOL);
                }
                _ => {}
            }
        }
        _ => {}
    }
    return 0;
}
