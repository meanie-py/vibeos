use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn RemoveStartMenuItemsProc(hwndCPl: HWND, Message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    EndDialog(hwndCPl, IDOK as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwndCPl, IDCANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {
            return FALSE as INT_PTR;
        }
    }
    return TRUE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn ShowRemoveStartMenuItems(hwndCPl: HWND, szPath: LPWSTR) -> LONG {
    DialogBox!(hApplet, MAKEINTRESOURCEW!(IDD_CONFIG_STARTMENU), hwndCPl, Some(RemoveStartMenuItemsProc));
    return (TRUE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn ConfigStartMenu(hwndCPl: HWND, uMsg: UINT, lParam1: LPARAM, lParam2: LPARAM) -> LONG {
    return ShowRemoveStartMenuItems(hwndCPl, lParam1 as LPWSTR);
}
