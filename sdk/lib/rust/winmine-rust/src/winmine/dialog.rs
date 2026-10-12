use crate::winmine::main::CheckLevel;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn CustomDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut IsRet: BOOL = 0;
    static mut p_board: *mut BOARD = core::ptr::null_mut();
    match uMsg {
        WM_INITDIALOG => {
            p_board = lParam as *mut BOARD;
            SetDlgItemInt(hDlg, (IDC_EDITROWS as i32), (*p_board).rows as UINT, (FALSE) as BOOL);
            SetDlgItemInt(hDlg, (IDC_EDITCOLS as i32), (*p_board).cols as UINT, (FALSE) as BOOL);
            SetDlgItemInt(hDlg, (IDC_EDITMINES as i32), (*p_board).mines as UINT, (FALSE) as BOOL);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK => {
                    (*p_board).rows = GetDlgItemInt(hDlg, (IDC_EDITROWS as i32), &mut IsRet, (FALSE) as BOOL);
                    (*p_board).cols = GetDlgItemInt(hDlg, (IDC_EDITCOLS as i32), &mut IsRet, (FALSE) as BOOL);
                    (*p_board).mines = GetDlgItemInt(hDlg, (IDC_EDITMINES as i32), &mut IsRet, (FALSE) as BOOL);
                    CheckLevel(p_board);
                    EndDialog(hDlg, 0 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                IDCANCEL => {
                    EndDialog(hDlg, 1 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn CongratsDlgProc(mut hDlg: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    static mut p_board: *mut BOARD = core::ptr::null_mut::<BOARD>();
    match uMsg {
        WM_INITDIALOG => {
            p_board = lParam as *mut BOARD;
            SetDlgItemTextW(
                hDlg,
                (IDC_EDITNAME as i32),
                (core::ptr::addr_of_mut!((*p_board).best_name) as *mut [WCHAR; 32])
                    .offset((*p_board).difficulty as isize) as *mut WCHAR as LPCWSTR,
            );
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam as DWORD) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    GetDlgItemTextW(
                        hDlg,
                        (IDC_EDITNAME as i32),
                        (core::ptr::addr_of_mut!((*p_board).best_name) as *mut [WCHAR; 32])
                            .offset((*p_board).difficulty as isize) as *mut WCHAR as LPWSTR,
                        (core::mem::size_of::<[WCHAR; 32]>() / core::mem::size_of::<WCHAR>())
                            as ::core::ffi::c_int,
                    );
                    EndDialog(hDlg, 0 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                IDCANCEL => {
                    EndDialog(hDlg, 0 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn TimesDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut p_board: *mut BOARD = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint;

    match uMsg {
        WM_INITDIALOG => {
            p_board = lParam as *mut BOARD;

            /* set best names */
            i = 0 as ::core::ffi::c_uint;
            while i < 3 as ::core::ffi::c_uint {
                SetDlgItemTextW(
                    hDlg,
                    (1014 as ::core::ffi::c_uint).wrapping_add(i) as ::core::ffi::c_int, // IDC_NAME1 + i
                    &raw mut *(&raw mut (*p_board).best_name as *mut [WCHAR; 32]).offset(i as isize)
                        as *mut WCHAR as LPCWSTR,
                );
                i = i.wrapping_add(1);
            }

            /* set best times */
            i = 0 as ::core::ffi::c_uint;
            while i < 3 as ::core::ffi::c_uint {
                SetDlgItemInt(
                    hDlg,
                    (1011 as ::core::ffi::c_uint).wrapping_add(i) as ::core::ffi::c_int, // IDC_TIME1 + i
                    (*p_board).best_time[i as usize] as UINT,
                    (FALSE) as BOOL,
                );
                i = i.wrapping_add(1);
            }
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK | IDCANCEL => {
                    EndDialog(hDlg, 0 as INT_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }

        _ => {}
    }
    return FALSE as INT_PTR;
}
